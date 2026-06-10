//! Time-systems ontology — the standard astronomical / metrological
//! time scales (TAI, UTC, GPS, TT, TCB, MET, Unix) plus the metric and
//! interval-algebra axioms that they share.
//!
//! The seven `TimeSystem` rich-type variants in
//! [`super::time_system`] are mirrored by the ontology's `concepts`
//! list at the categorical level. Conversions between time systems
//! are morphisms (offset functions); Allen's (1983) thirteen interval
//! relations live as a separate axiom layer over time-stamped
//! intervals.
//!
//! # Literature
//!
//! - **Allen (1983)** "Maintaining Knowledge about Temporal Intervals",
//!   *Communications of the ACM* 26(11):832-843 — the thirteen-element
//!   jointly-exhaustive, pairwise-disjoint interval calculus
//!   (before/after/meets/met-by/overlaps/overlapped-by/during/
//!   contains/starts/started-by/finishes/finished-by/equals).
//! - **BIPM** *International System of Units — TAI / UTC definitions*
//!   (Bureau International des Poids et Mesures) — the metrological
//!   definition of TAI and UTC; leap-second mechanism.
//! - **ITU-R TF.460** — the recommendation defining UTC.
//! - **IS-GPS-200** — Global Positioning System Interface Control
//!   Document; defines GPS time and its fixed −19-second offset from TAI.
//! - **IAU 2000 Resolution B1.9** — Terrestrial Time TT = TAI + 32.184 s.
//! - **IAU 2006 Resolution B3** — Barycentric Coordinate Time TCB.

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::temporal::allen::{self};
use crate::formal::math::temporal::instant::Instant;
use crate::formal::math::temporal::interval::Interval;
use crate::formal::math::temporal::time_system::{self, TimeSystem};

pr4xis::ontology! {
    name: "Time",
    source: "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843; BIPM TAI/UTC definitions; ITU-R TF.460; IS-GPS-200; IAU 2000 Resolution B1.9; IAU 2006 Resolution B3",

    concepts: [
        TAI,
        UTC,
        GPS,
        TT,
        TCB,
        MET,
        Unix,
    ],

    labels: {
        TAI: ("en", "TAI",
            "BIPM: International Atomic Time - the continuous, leap-second-free atomic time scale; epoch 1958-01-01T00:00:00; the reference for all other systems."),
        UTC: ("en", "UTC",
            "ITU-R TF.460: Coordinated Universal Time - TAI corrected by accumulated leap seconds to stay within 0.9 s of UT1; discontinuous at leap-second events."),
        GPS: ("en", "GPS time",
            "IS-GPS-200: continuous atomic time scale used by the Global Positioning System; GPS = TAI - 19 s (fixed offset, epoch 1980-01-06)."),
        TT: ("en", "TT",
            "IAU 2000 Resolution B1.9: Terrestrial Time, the idealised time on the geoid; TT = TAI + 32.184 s."),
        TCB: ("en", "TCB",
            "IAU 2006 Resolution B3: Barycentric Coordinate Time, the time scale at the solar-system barycentre; relativistic corrections to TT."),
        MET: ("en", "MET",
            "Mission Elapsed Time - seconds since mission start; spacecraft operations convention."),
        Unix: ("en", "Unix time",
            "POSIX/IEEE Std 1003.1: seconds since 1970-01-01T00:00:00 UTC, ignoring leap seconds (non-monotonic across leap-second events)."),
    },

    edges: [
        // Conversion morphisms — fixed-offset known-good directions.
        // GPS = TAI - 19; TT = TAI + 32.184; UTC = TAI - leap_seconds(t).
        (TAI, GPS, Conversion),
        (GPS, TAI, Conversion),
        (TAI, TT, Conversion),
        (TT, TAI, Conversion),
        (TAI, UTC, Conversion),
        (UTC, TAI, Conversion),
        (TT, TCB, Conversion),
        (TCB, TT, Conversion),
    ],
}

/// Quality: which time systems include leap seconds (ITU-R TF.460
/// for UTC; POSIX treatment for Unix). All purely-atomic systems are
/// continuous and leap-second-free.
#[derive(Debug, Clone)]
pub struct HasLeapSeconds;

impl Quality for HasLeapSeconds {
    type Individual = TimeConcept;
    type Value = bool;

    fn get(&self, sys: &TimeConcept) -> Option<bool> {
        Some(matches!(sys, TimeConcept::UTC | TimeConcept::Unix))
    }
}

/// Quality: which time systems are monotonically continuous (i.e.
/// no leap-second discontinuities). TAI, GPS, TT, TCB, MET are
/// continuous; UTC and Unix are not (per ITU-R TF.460).
#[derive(Debug, Clone)]
pub struct IsContinuous;

impl Quality for IsContinuous {
    type Individual = TimeConcept;
    type Value = bool;

    fn get(&self, sys: &TimeConcept) -> Option<bool> {
        Some(matches!(
            sys,
            TimeConcept::TAI
                | TimeConcept::GPS
                | TimeConcept::TT
                | TimeConcept::TCB
                | TimeConcept::MET
        ))
    }
}

/// Map the ontology concept to the rich `TimeSystem` enum used by the
/// instant/interval/conversion functions in sibling modules.
fn to_rich(c: TimeConcept) -> TimeSystem {
    match c {
        TimeConcept::TAI => TimeSystem::TAI,
        TimeConcept::UTC => TimeSystem::UTC,
        TimeConcept::GPS => TimeSystem::GPS,
        TimeConcept::TT => TimeSystem::TT,
        TimeConcept::TCB => TimeSystem::TCB,
        TimeConcept::MET => TimeSystem::MET,
        TimeConcept::Unix => TimeSystem::Unix,
    }
}

impl Ontology for TimeOntology {
    type Cat = TimeCategory;
    type Qual = HasLeapSeconds;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(TotalOrder));
        axioms.push(Box::new(DurationNonNegativity));
        axioms.push(Box::new(DurationIdentity));
        axioms.push(Box::new(DurationAntisymmetry));
        axioms.push(Box::new(DurationAdditivity));
        axioms.push(Box::new(AllenExhaustive));
        axioms.push(Box::new(AllenInverseLaw));
        axioms.push(Box::new(GpsTaiConversion));
        axioms.push(Box::new(TtTaiConversion));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Domain axioms.
// ---------------------------------------------------------------------------

/// Time is a total order on instants within a single time system —
/// for any two instants exactly one of <, =, > holds. This is the
/// trichotomy law inherited from the reals; mirrored at the temporal
/// level (Allen 1983 §1, presupposed by the interval calculus).
pub struct TotalOrder;

impl Axiom for TotalOrder {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let instants = canonical_instants();
        for a in &instants {
            for b in &instants {
                if a.system != b.system {
                    continue;
                }
                let lt = a.seconds < b.seconds;
                let eq = (a.seconds - b.seconds).abs() < 1e-15;
                let gt = a.seconds > b.seconds;
                if !(lt || eq || gt) || (lt && gt) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "TotalOrder",
        "time is a total order: trichotomy holds on instants within a system",
        "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843 §1"
    );
}

pr4xis::register_axiom!(
    TotalOrder,
    "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843 §1"
);

/// Forward duration is positive: if a < b then d(a, b) > 0.
/// Mirrors the metric-space axiom on the temporal line.
pub struct DurationNonNegativity;

impl Axiom for DurationNonNegativity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
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
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DurationNonNegativity",
        "duration from an earlier to a later instant is positive",
        "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843 §2 (presupposed metric structure)"
    );
}

pr4xis::register_axiom!(
    DurationNonNegativity,
    "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843 §2 (presupposed metric structure)"
);

/// Duration identity: d(a, a) = 0 — the temporal analogue of the
/// metric-space identity-of-indiscernibles axiom.
pub struct DurationIdentity;

impl Axiom for DurationIdentity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for a in &canonical_instants() {
            if let Some(d) = a.duration_to(a)
                && d.seconds().abs() > 1e-15
            {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DurationIdentity",
        "duration from an instant to itself is zero",
        "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843 §2 (presupposed metric structure)"
    );
}

pr4xis::register_axiom!(
    DurationIdentity,
    "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843 §2 (presupposed metric structure)"
);

/// Duration antisymmetry: d(a, b) = -d(b, a) — duration is a signed
/// difference; reversing the endpoints negates the result.
pub struct DurationAntisymmetry;

impl Axiom for DurationAntisymmetry {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let instants = canonical_instants();
        for a in &instants {
            for b in &instants {
                if a.system != b.system {
                    continue;
                }
                let d_ab = a.duration_to(b).unwrap().seconds();
                let d_ba = b.duration_to(a).unwrap().seconds();
                if (d_ab + d_ba).abs() > 1e-12 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DurationAntisymmetry",
        "d(a,b) = -d(b,a) (signed duration)",
        "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843 §2 (presupposed metric structure)"
    );
}

pr4xis::register_axiom!(
    DurationAntisymmetry,
    "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843 §2 (presupposed metric structure)"
);

/// Duration additivity (chasles' relation): d(a, b) + d(b, c) = d(a, c).
pub struct DurationAdditivity;

impl Axiom for DurationAdditivity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
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
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DurationAdditivity",
        "d(a,b) + d(b,c) = d(a,c) (Chasles relation on the time line)",
        "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843 §2 (presupposed metric structure)"
    );
}

pr4xis::register_axiom!(
    DurationAdditivity,
    "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843 §2 (presupposed metric structure)"
);

/// Allen's thirteen relations are jointly exhaustive and pairwise
/// disjoint — every pair of intervals satisfies exactly one of
/// before/after/meets/met-by/overlaps/overlapped-by/during/
/// contains/starts/started-by/finishes/finished-by/equals.
/// Allen (1983) Theorem 1.
pub struct AllenExhaustive;

impl Axiom for AllenExhaustive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::SimpleProof;
        let intervals = canonical_intervals();
        for x in &intervals {
            for y in &intervals {
                // `relate` returns exactly one relation by construction;
                // a defect would manifest as a panic or non-membership.
                let _r = allen::relate(x, y, 1e-10);
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "AllenExhaustive",
        "Allen's 13 interval relations are jointly exhaustive and pairwise disjoint",
        "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843, Theorem 1"
    );
}

pr4xis::register_axiom!(
    AllenExhaustive,
    "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843, Theorem 1"
);

/// Allen inverse law: if R(X, Y) then R^{-1}(Y, X) — every Allen
/// relation has a unique converse. Allen (1983) Table 1.
pub struct AllenInverseLaw;

impl Axiom for AllenInverseLaw {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let intervals = canonical_intervals();
        for x in &intervals {
            for y in &intervals {
                let r_xy = allen::relate(x, y, 1e-10);
                let r_yx = allen::relate(y, x, 1e-10);
                if r_xy.inverse() != r_yx {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "AllenInverseLaw",
        "if R(X,Y) then R^{-1}(Y,X) (Allen inverse)",
        "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843, Table 1"
    );
}

pr4xis::register_axiom!(
    AllenInverseLaw,
    "Allen (1983) Maintaining Knowledge about Temporal Intervals, CACM 26(11):832-843, Table 1"
);

/// GPS ↔ TAI conversion: the fixed −19-second offset specified by
/// the GPS Interface Control Document IS-GPS-200; the round-trip
/// is the identity (within floating-point tolerance).
pub struct GpsTaiConversion;

impl Axiom for GpsTaiConversion {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let test_times = [0.0, 1000.0, 1e6, 1.7e9];
        let gps = to_rich(TimeConcept::GPS);
        let tai = to_rich(TimeConcept::TAI);
        for &t_gps in &test_times {
            let t_tai = time_system::convert(t_gps, gps, tai).unwrap();
            if (t_tai - (t_gps + 19.0)).abs() > 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            let t_gps2 = time_system::convert(t_tai, tai, gps).unwrap();
            if (t_gps2 - t_gps).abs() > 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "GpsTaiConversion",
        "GPS = TAI - 19 s (fixed offset, IS-GPS-200)",
        "IS-GPS-200: Global Positioning System Interface Control Document"
    );
}

pr4xis::register_axiom!(
    GpsTaiConversion,
    "IS-GPS-200: Global Positioning System Interface Control Document"
);

/// TT = TAI + 32.184 s — the fixed offset specified by IAU 2000
/// Resolution B1.9 defining Terrestrial Time on the geoid.
pub struct TtTaiConversion;

impl Axiom for TtTaiConversion {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let test_times = [0.0, 1000.0, 1e6];
        let tai = to_rich(TimeConcept::TAI);
        let tt = to_rich(TimeConcept::TT);
        for &t_tai in &test_times {
            let t_tt = time_system::convert(t_tai, tai, tt).unwrap();
            if (t_tt - (t_tai + 32.184)).abs() > 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "TtTaiConversion",
        "TT = TAI + 32.184 s (IAU 2000 Resolution B1.9)",
        "IAU 2000 Resolution B1.9: Definition of Terrestrial Time"
    );
}

pr4xis::register_axiom!(
    TtTaiConversion,
    "IAU 2000 Resolution B1.9: Definition of Terrestrial Time"
);

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
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<TimeCategory>();
    }

    #[test]
    fn ontology_validates() {
        TimeOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn seven_time_systems() {
        assert_eq!(TimeConcept::variants().len(), 7);
    }

    #[test]
    fn leap_seconds_classification() {
        let q = HasLeapSeconds;
        assert_eq!(q.get(&TimeConcept::UTC), Some(true));
        assert_eq!(q.get(&TimeConcept::Unix), Some(true));
        assert_eq!(q.get(&TimeConcept::TAI), Some(false));
        assert_eq!(q.get(&TimeConcept::GPS), Some(false));
    }

    #[test]
    fn continuity_classification() {
        let q = IsContinuous;
        for c in [
            TimeConcept::TAI,
            TimeConcept::GPS,
            TimeConcept::TT,
            TimeConcept::TCB,
            TimeConcept::MET,
        ] {
            assert_eq!(q.get(&c), Some(true), "{:?} should be continuous", c);
        }
        for c in [TimeConcept::UTC, TimeConcept::Unix] {
            assert_eq!(q.get(&c), Some(false), "{:?} should be discontinuous", c);
        }
    }

    #[test]
    fn metric_axioms_hold() {
        assert!(TotalOrder.verify().is_ok());
        assert!(DurationNonNegativity.verify().is_ok());
        assert!(DurationIdentity.verify().is_ok());
        assert!(DurationAntisymmetry.verify().is_ok());
        assert!(DurationAdditivity.verify().is_ok());
    }

    #[test]
    fn allen_axioms_hold() {
        assert!(AllenExhaustive.verify().is_ok());
        assert!(AllenInverseLaw.verify().is_ok());
    }

    #[test]
    fn conversion_axioms_hold() {
        assert!(GpsTaiConversion.verify().is_ok());
        assert!(TtTaiConversion.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = TimeConcept> {
        proptest::sample::select(TimeConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_leap_seconds_total(c in arb_concept()) {
            prop_assert!(HasLeapSeconds.get(&c).is_some());
        }

        #[test]
        fn prop_continuity_total(c in arb_concept()) {
            prop_assert!(IsContinuous.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            use pr4xis::category::{Arrow, Category};
            for m in TimeCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in TimeOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }
    }
}
