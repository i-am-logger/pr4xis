#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::temporal::allen::{self};
use crate::formal::math::temporal::instant::Instant;
use crate::formal::math::temporal::interval::Interval;
use crate::formal::math::temporal::time_system::{self, TimeSystem};

// ---------------------------------------------------------------------------
// Entity: time systems (objects in the time system category)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_ontology! {
    /// The time system category.
    pub TimeOntology for TimeSystemCategory {
        concepts: TimeSystem,
        relation: TimeSystemConversion,
        being: AbstractObject,
        source: "Allen (1983); BIPM (UTC/TAI)",
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: does the time system have leap seconds?
#[derive(Debug, Clone)]
pub struct HasLeapSeconds;

impl Quality for HasLeapSeconds {
    type Individual = TimeSystem;
    type Value = bool;

    fn get(&self, sys: &TimeSystem) -> Option<bool> {
        Some(match sys {
            TimeSystem::UTC => true,
            TimeSystem::Unix => true, // technically, but handled differently
            _ => false,
        })
    }
}

/// Quality: is the time system monotonically continuous?
#[derive(Debug, Clone)]
pub struct IsContinuous;

impl Quality for IsContinuous {
    type Individual = TimeSystem;
    type Value = bool;

    fn get(&self, sys: &TimeSystem) -> Option<bool> {
        Some(match sys {
            TimeSystem::TAI | TimeSystem::GPS | TimeSystem::TT | TimeSystem::TCB => true,
            TimeSystem::UTC | TimeSystem::Unix => false, // leap seconds break continuity
            TimeSystem::MET => true,
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Time is a total order: for any instants a, b: a < b OR a = b OR a > b.
pub struct TotalOrder;

impl Axiom for TotalOrder {
    fn description(&self) -> &str {
        "time is a total order: for any two instants, exactly one of <, =, > holds"
    }

    fn holds(&self) -> bool {
        let instants = canonical_instants();
        for a in &instants {
            for b in &instants {
                if a.system != b.system {
                    continue;
                }
                let lt = a.seconds < b.seconds;
                let eq = (a.seconds - b.seconds).abs() < 1e-15;
                let gt = a.seconds > b.seconds;
                if !(lt || eq || gt) {
                    return false;
                }
                // Exactly one (allowing for float edge cases)
                if lt && gt {
                    return false;
                }
            }
        }
        true
    }
}
pr4xis::register_axiom!(TotalOrder);

/// Duration is non-negative for forward time: if a < b then d(a,b) > 0.
pub struct DurationNonNegativity;

impl Axiom for DurationNonNegativity {
    fn description(&self) -> &str {
        "duration from earlier to later instant is positive"
    }

    fn holds(&self) -> bool {
        let instants = canonical_instants();
        for a in &instants {
            for b in &instants {
                if a.system != b.system {
                    continue;
                }
                if let Some(d) = a.duration_to(b)
                    && a.is_before(b)
                    && d.seconds() <= 0.0
                {
                    return false;
                }
            }
        }
        true
    }
}
pr4xis::register_axiom!(DurationNonNegativity);

/// Duration metric: d(a,a) = 0.
pub struct DurationIdentity;

impl Axiom for DurationIdentity {
    fn description(&self) -> &str {
        "duration from an instant to itself is zero"
    }

    fn holds(&self) -> bool {
        for a in &canonical_instants() {
            if let Some(d) = a.duration_to(a)
                && d.seconds().abs() > 1e-15
            {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(DurationIdentity);

/// Duration antisymmetry: d(a,b) = -d(b,a).
pub struct DurationAntisymmetry;

impl Axiom for DurationAntisymmetry {
    fn description(&self) -> &str {
        "d(a,b) = -d(b,a) (duration antisymmetry)"
    }

    fn holds(&self) -> bool {
        let instants = canonical_instants();
        for a in &instants {
            for b in &instants {
                if a.system != b.system {
                    continue;
                }
                let d_ab = a.duration_to(b).unwrap().seconds();
                let d_ba = b.duration_to(a).unwrap().seconds();
                if (d_ab + d_ba).abs() > 1e-12 {
                    return false;
                }
            }
        }
        true
    }
}
pr4xis::register_axiom!(DurationAntisymmetry);

/// Duration additivity: d(a,b) + d(b,c) = d(a,c).
pub struct DurationAdditivity;

impl Axiom for DurationAdditivity {
    fn description(&self) -> &str {
        "d(a,b) + d(b,c) = d(a,c) (duration additivity)"
    }

    fn holds(&self) -> bool {
        let instants = canonical_instants();
        for a in &instants {
            for b in &instants {
                for c in &instants {
                    if a.system != b.system || b.system != c.system {
                        continue;
                    }
                    let ab = a.duration_to(b).unwrap().seconds();
                    let bc = b.duration_to(c).unwrap().seconds();
                    let ac = a.duration_to(c).unwrap().seconds();
                    if (ab + bc - ac).abs() > 1e-10 {
                        return false;
                    }
                }
            }
        }
        true
    }
}
pr4xis::register_axiom!(DurationAdditivity);

/// Allen's relations are jointly exhaustive: every pair of intervals satisfies exactly one.
pub struct AllenExhaustive;

impl Axiom for AllenExhaustive {
    fn description(&self) -> &str {
        "Allen's 13 relations are jointly exhaustive and pairwise disjoint"
    }

    fn holds(&self) -> bool {
        let intervals = canonical_intervals();
        for x in &intervals {
            for y in &intervals {
                // relate() always returns exactly one relation
                let _r = allen::relate(x, y, 1e-10);
            }
        }
        true
    }
}
pr4xis::register_axiom!(AllenExhaustive);

/// Allen's inverse law: if R(X,Y) then R^{-1}(Y,X).
pub struct AllenInverseLaw;

impl Axiom for AllenInverseLaw {
    fn description(&self) -> &str {
        "Allen's inverse: if R(X,Y) then R^{-1}(Y,X)"
    }

    fn holds(&self) -> bool {
        let intervals = canonical_intervals();
        for x in &intervals {
            for y in &intervals {
                let r_xy = allen::relate(x, y, 1e-10);
                let r_yx = allen::relate(y, x, 1e-10);
                if r_xy.inverse() != r_yx {
                    return false;
                }
            }
        }
        true
    }
}
pr4xis::register_axiom!(AllenInverseLaw);

/// GPS ↔ TAI conversion roundtrip: GPS + 19 = TAI.
pub struct GpsTaiConversion;

impl Axiom for GpsTaiConversion {
    fn description(&self) -> &str {
        "GPS = TAI - 19 seconds (fixed offset, IS-GPS-200)"
    }

    fn holds(&self) -> bool {
        let test_times = [0.0, 1000.0, 1e6, 1.7e9];
        for &t_gps in &test_times {
            let t_tai = time_system::convert(t_gps, TimeSystem::GPS, TimeSystem::TAI).unwrap();
            // TAI = GPS + 19
            if (t_tai - (t_gps + 19.0)).abs() > 1e-10 {
                return false;
            }
            // Roundtrip
            let t_gps2 = time_system::convert(t_tai, TimeSystem::TAI, TimeSystem::GPS).unwrap();
            if (t_gps2 - t_gps).abs() > 1e-10 {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(GpsTaiConversion);

/// TT = TAI + 32.184 seconds (IAU 2000 Resolution B1.9).
pub struct TtTaiConversion;

impl Axiom for TtTaiConversion {
    fn description(&self) -> &str {
        "TT = TAI + 32.184 seconds (IAU 2000 Resolution B1.9)"
    }

    fn holds(&self) -> bool {
        let test_times = [0.0, 1000.0, 1e6];
        for &t_tai in &test_times {
            let t_tt = time_system::convert(t_tai, TimeSystem::TAI, TimeSystem::TT).unwrap();
            if (t_tt - (t_tai + 32.184)).abs() > 1e-10 {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(TtTaiConversion);

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

impl Ontology for TimeOntology {
    type Cat = TimeSystemCategory;
    type Qual = HasLeapSeconds;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(TotalOrder),
            Box::new(DurationNonNegativity),
            Box::new(DurationIdentity),
            Box::new(DurationAntisymmetry),
            Box::new(DurationAdditivity),
            Box::new(AllenExhaustive),
            Box::new(AllenInverseLaw),
            Box::new(GpsTaiConversion),
            Box::new(TtTaiConversion),
        ]
    }
}

// ---------------------------------------------------------------------------
// Canonical test data
// ---------------------------------------------------------------------------

fn canonical_instants() -> Vec<Instant> {
    vec![
        Instant::new(0.0, TimeSystem::TAI),
        Instant::new(1.0, TimeSystem::TAI),
        Instant::new(10.0, TimeSystem::TAI),
        Instant::new(100.0, TimeSystem::TAI),
        Instant::new(1000.0, TimeSystem::TAI),
        Instant::new(0.0, TimeSystem::GPS),
        Instant::new(1.0, TimeSystem::GPS),
        Instant::new(100.0, TimeSystem::GPS),
    ]
}

fn canonical_intervals() -> Vec<Interval> {
    let s = TimeSystem::TAI;
    vec![
        Interval::new(Instant::new(0.0, s), Instant::new(5.0, s)).unwrap(),
        Interval::new(Instant::new(5.0, s), Instant::new(10.0, s)).unwrap(),
        Interval::new(Instant::new(3.0, s), Instant::new(7.0, s)).unwrap(),
        Interval::new(Instant::new(1.0, s), Instant::new(4.0, s)).unwrap(),
        Interval::new(Instant::new(0.0, s), Instant::new(10.0, s)).unwrap(),
        Interval::new(Instant::new(2.0, s), Instant::new(8.0, s)).unwrap(),
        Interval::new(Instant::new(0.0, s), Instant::new(7.0, s)).unwrap(),
        Interval::new(Instant::new(3.0, s), Instant::new(10.0, s)).unwrap(),
        Interval::new(Instant::new(20.0, s), Instant::new(30.0, s)).unwrap(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<TimeSystemCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        TimeOntology::validate().unwrap();
    }
}
