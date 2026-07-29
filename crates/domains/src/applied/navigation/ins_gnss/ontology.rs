//! INS/GNSS integration — coupling levels for inertial-GNSS fusion.
//!
//! This ontology covers the coupling levels (Loosely, Tightly, Deeply).
//! The operational state of an INS/GNSS system (Navigation, Coasting,
//! GnssReacquired, Initializing) lives in the sibling `state` module.
//!
//! Source: Groves (2013) Chapters 14-17, Titterton & Weston (2004) Chapter 13.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

use crate::formal::math::quantity::unit::HERTZ;
use crate::formal::math::quantity::value::{Quantity, QuantityRange};

pr4xis::ontology! {
    name: "InsGnss",
    source: "Groves (2013); Titterton & Weston (2004)",

    concepts: [Coupling, LooselyCoupled, TightlyCoupled, DeeplyCoupled],

    labels: {
        Coupling: ("en", "INS/GNSS coupling", "Abstract INS/GNSS coupling level — root of the taxonomy."),
        LooselyCoupled: ("en", "Loosely coupled", "GNSS provides position/velocity to INS filter."),
        TightlyCoupled: ("en", "Tightly coupled", "GNSS provides raw pseudoranges to INS filter. Works with < 4 satellites."),
        DeeplyCoupled: ("en", "Deeply coupled", "INS aids GNSS tracking loops. Handles weaker signals."),
    },

    is_a: [
        (LooselyCoupled, Coupling),
        (TightlyCoupled, Coupling),
        (DeeplyCoupled, Coupling),
        (TightlyCoupled, LooselyCoupled),
        (DeeplyCoupled, TightlyCoupled),
    ],
}

/// Backward-compat alias for code that predates the rename via the
/// ontology! proc macro. `CouplingLevel` is the legacy name of
/// `InsGnssConcept` and is re-exported here so engine.rs / coupling.rs /
/// tests.rs keep working. Prefer `InsGnssConcept` in new code.
pub type CouplingLevel = InsGnssConcept;

/// Backward-compat alias — `InsGnssState` is now a separate ontology
/// (`super::state::InsGnssStateConcept`). Re-exported here so existing
/// callers resolve without scattered imports.
pub use crate::applied::navigation::ins_gnss::state::InsGnssStateConcept as InsGnssState;

/// Quality: Error state components at each coupling level.
///
/// Source: Groves (2013) Table 14.1.
#[derive(Debug, Clone)]
pub struct ErrorStateDescription;

impl Quality for ErrorStateDescription {
    type Individual = InsGnssConcept;
    type Value = &'static str;

    fn get(&self, level: &InsGnssConcept) -> Option<&'static str> {
        Some(match level {
            InsGnssConcept::Coupling => "position/velocity/attitude errors + sensor biases",
            InsGnssConcept::LooselyCoupled => {
                "15-state: pos(3)+vel(3)+att(3)+gyro_bias(3)+accel_bias(3)"
            }
            InsGnssConcept::TightlyCoupled => "17-state: 15 + clock_bias + clock_drift",
            InsGnssConcept::DeeplyCoupled => "17+ state with tracking loop aiding",
        })
    }
}

/// Quality: Coupling bandwidth — how fast corrections propagate, as a frequency
/// [`QuantityRange`] (Hz), NOT a prose string.
///
/// Loosely and tightly coupled fusion both run at the 1–10 Hz GNSS solution rate
/// (tightly coupled consumes raw pseudoranges rather than a PVT fix, but at the
/// same output cadence); deeply coupled fusion aids the GNSS tracking loops and
/// runs open-ended above 100 Hz, represented here as the half-open interval
/// `[100, ∞) Hz`. `None` for the abstract `Coupling` root — bandwidth depends on
/// the coupling level.
///
/// Source: Groves (2013) Chapters 14–17.
#[derive(Debug, Clone)]
pub struct CouplingBandwidth;

impl Quality for CouplingBandwidth {
    type Individual = InsGnssConcept;
    type Value = QuantityRange;
    const KIND: QualityKind = QualityKind::Physical;

    fn get(&self, level: &InsGnssConcept) -> Option<QuantityRange> {
        let hz = |lo: f64, hi: f64| QuantityRange {
            min: Quantity::from_unit(lo, &HERTZ),
            max: Quantity::from_unit(hi, &HERTZ),
        };
        Some(match level {
            // Abstract root — bandwidth depends on the coupling level.
            InsGnssConcept::Coupling => return None,
            InsGnssConcept::LooselyCoupled => hz(1.0, 10.0),
            // Same 1–10 Hz cadence; consumes raw pseudoranges rather than a PVT fix.
            InsGnssConcept::TightlyCoupled => hz(1.0, 10.0),
            // Open-ended above 100 Hz — INS aids the GNSS tracking loops.
            InsGnssConcept::DeeplyCoupled => hz(100.0, f64::INFINITY),
        })
    }
}

/// Direct subsumption query: is there an `is_a` edge from `child` to `parent`?
fn is_a(child: InsGnssConcept, parent: InsGnssConcept) -> bool {
    InsGnssCategory::morphisms().iter().any(|m| {
        m.kind() == InsGnssRelationKind::Subsumption && m.source() == child && m.target() == parent
    })
}

/// Coasting degrades: without GNSS, INS position error grows quadratically.
///
/// Source: Groves (2013) Eq. 14.1.
pub struct CoastingDegrades;

impl Axiom for CoastingDegrades {
    fn verify(&self) -> Verdict {
        let bias_mg = 1.0_f64;
        // milli-g of bias → m/s² via the cited standard-gravity constant (BIPM),
        // never a raw 9.80665 literal.
        let bias_mps2 =
            bias_mg * 1e-3 * crate::formal::math::quantity::constants::standard_gravity().value;
        let t1 = 30.0_f64;
        let t2 = 60.0_f64;
        let error_t1 = 0.5 * bias_mps2 * t1 * t1;
        let error_t2 = 0.5 * bias_mps2 * t2 * t2;
        let ratio = error_t2 / error_t1;
        if (ratio - 4.0).abs() < 0.01 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CoastingDegrades",
        "without GNSS, INS position error grows quadratically (bias -> t^2 error)",
        "Groves (2013) Eq. 14.1"
    );
}
pr4xis::register_axiom!(CoastingDegrades, "Groves (2013) Eq. 14.1");

/// GNSS measurement update reduces position uncertainty.
///
/// Source: Brown & Hwang (2012), Chapter 5.
pub struct GnssUpdateReducesError;

impl Axiom for GnssUpdateReducesError {
    fn verify(&self) -> Verdict {
        let p_prior = 100.0;
        let r = 25.0;
        let p_post = p_prior * r / (p_prior + r);
        if p_post < p_prior {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GnssUpdateReducesError",
        "GNSS measurement update decreases position uncertainty",
        "Brown & Hwang (2012), Chapter 5"
    );
}
pr4xis::register_axiom!(GnssUpdateReducesError, "Brown & Hwang (2012), Chapter 5");

/// Tighter coupling provides better performance in degraded GNSS.
///
/// Source: Groves (2013) Section 14.5.
pub struct TighterCouplingBetter;

impl Axiom for TighterCouplingBetter {
    fn verify(&self) -> Verdict {
        if is_a(
            InsGnssConcept::TightlyCoupled,
            InsGnssConcept::LooselyCoupled,
        ) && is_a(
            InsGnssConcept::DeeplyCoupled,
            InsGnssConcept::TightlyCoupled,
        ) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TighterCouplingBetter",
        "tighter coupling provides better performance in degraded GNSS",
        "Groves (2013) Section 14.5"
    );
}
pr4xis::register_axiom!(TighterCouplingBetter, "Groves (2013) Section 14.5");

impl Ontology for InsGnssOntology {
    type Cat = InsGnssCategory;
    type Qual = ErrorStateDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(CoastingDegrades));
        axioms.push(Box::new(GnssUpdateReducesError));
        axioms.push(Box::new(TighterCouplingBetter));
        axioms.push(Box::new(GnssFixNeverWorsensVelocity));
        axioms
    }
}

/// Axiom: a GNSS position fix never worsens the velocity estimate.
///
/// The velocity error is corrected through the position–velocity coupling by a
/// factor `√(1 − ρ²·K) ≤ 1` (see [`PosVelCoupling`](crate::applied::navigation::ins_gnss::coupling::PosVelCoupling)),
/// so the posterior velocity error can never exceed the prior — a GNSS update is
/// honest, it cannot inject velocity error. Verified over a grid of prior errors,
/// coupling regimes, and Kalman gains. This is the property the former inline
/// `velocity_error * 0.8` / `* 0.5` gains asserted only by fiat.
pub struct GnssFixNeverWorsensVelocity;

impl Axiom for GnssFixNeverWorsensVelocity {
    fn verify(&self) -> Verdict {
        use crate::applied::navigation::ins_gnss::coupling::PosVelCoupling;
        let couplings = [PosVelCoupling::nominal(), PosVelCoupling::reacquisition()];
        let ok = couplings.iter().all(|c| {
            [0.1_f64, 1.0, 10.0].iter().all(|&v| {
                [0.0_f64, 0.3, 0.7, 1.0]
                    .iter()
                    .all(|&k| c.velocity_error_after_fix(v, k).value <= v + 1e-12)
            })
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GnssFixNeverWorsensVelocity",
        "a GNSS position fix corrects velocity error by a factor <= 1 through the pos-vel coupling (never increases it)",
        "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation, 2nd ed., §14.3.3"
    );
}
pr4xis::register_axiom!(
    GnssFixNeverWorsensVelocity,
    "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation, 2nd ed., §14.3.3"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<InsGnssCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        InsGnssOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
