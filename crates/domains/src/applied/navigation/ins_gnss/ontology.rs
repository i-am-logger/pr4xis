//! INS/GNSS integration — coupling levels for inertial-GNSS fusion.
//!
//! This ontology covers the coupling levels (Loosely, Tightly, Deeply).
//! The operational state of an INS/GNSS system (Navigation, Coasting,
//! GnssReacquired, Initializing) lives in the sibling `state` module.
//!
//! Source: Groves (2013) Chapters 14-17, Titterton & Weston (2004) Chapter 13.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

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

/// Quality: Coupling bandwidth — how fast corrections propagate.
#[derive(Debug, Clone)]
pub struct CouplingBandwidth;

impl Quality for CouplingBandwidth {
    type Individual = InsGnssConcept;
    type Value = &'static str;

    fn get(&self, level: &InsGnssConcept) -> Option<&'static str> {
        Some(match level {
            InsGnssConcept::Coupling => "depends on coupling level",
            InsGnssConcept::LooselyCoupled => "1-10 Hz GNSS update rate",
            InsGnssConcept::TightlyCoupled => "1-10 Hz, uses raw pseudoranges",
            InsGnssConcept::DeeplyCoupled => "100+ Hz, INS aids GNSS tracking loops",
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
        let bias_mps2 = bias_mg * 1e-3 * 9.80665;
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
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<InsGnssCategory>();
    }

    #[test]
    fn ontology_validates() {
        InsGnssOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
