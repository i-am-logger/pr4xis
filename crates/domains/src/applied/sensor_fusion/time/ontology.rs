//! Sensor time — synchronisation-strategy ontology for multi-sensor fusion.
//!
//! When sensors operate at different rates, measurements must be aligned
//! to a common time before fusion. Three strategies, ordered by the
//! amount of information they use: nearest-neighbour (one measurement),
//! linear interpolation (two bracketing measurements), and extrapolation
//! (one measurement plus a dynamics model). Extrapolation has unbounded
//! error growth and is dangerous; the ontology marks it accordingly.
//!
//! # Literature
//!
//! - **Bar-Shalom, Li & Kirubarajan (2001)** *Estimation with
//!   Applications to Tracking and Navigation*, Ch. 6 — "Tracking with
//!   Multiple Sensors". §6.2 covers temporal alignment and the
//!   linear-interpolation error bound `T²/8 · a_max`.
//! - **Groves (2013)** *Principles of GNSS, Inertial, and Multisensor
//!   Integrated Navigation Systems*, 2nd ed., §17.2.4 — "Time
//!   synchronization" — the three canonical alignment strategies in
//!   integrated navigation.

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "SensorTime",
    source: "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §6.2; Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. §17.2.4",

    concepts: [
        // The three Groves (2013) §17.2.4 canonical alignment strategies,
        // ordered by error-bound tightness.
        NearestNeighbor,
        LinearInterpolation,
        Extrapolation,
    ],

    labels: {
        NearestNeighbor: ("en", "Nearest neighbour",
            "Groves (2013) §17.2.4: align to the measurement closest in time to the target. Error bounded by T/2 · max_rate."),
        LinearInterpolation: ("en", "Linear interpolation",
            "Bar-Shalom (2001) §6.2.3: interpolate between two bracketing measurements. Error bounded by T²/8 · a_max."),
        Extrapolation: ("en", "Extrapolation",
            "Bar-Shalom (2001) §6.2.4: project the latest measurement forward using a dynamics model. Error grows unboundedly with extrapolation distance."),
    },

    opposes: [
        // Bounded (NearestNeighbor / LinearInterpolation) vs unbounded
        // (Extrapolation) — the safety boundary in §6.2.
        (NearestNeighbor, Extrapolation),
        (Extrapolation, NearestNeighbor),
        (LinearInterpolation, Extrapolation),
        (Extrapolation, LinearInterpolation),
    ],
}

/// Quality: whether a synchronisation strategy has a bounded error per
/// Bar-Shalom (2001) §6.2 / Groves (2013) §17.2.4.
#[derive(Debug, Clone)]
pub struct ErrorBoundedness;

impl Quality for ErrorBoundedness {
    type Individual = SensorTimeConcept;
    type Value = bool;

    fn get(&self, strategy: &SensorTimeConcept) -> Option<bool> {
        Some(match strategy {
            SensorTimeConcept::NearestNeighbor => true,
            SensorTimeConcept::LinearInterpolation => true,
            SensorTimeConcept::Extrapolation => false,
        })
    }
}

impl Ontology for SensorTimeOntology {
    type Cat = SensorTimeCategory;
    type Qual = ErrorBoundedness;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(NearestNeighborBounded));
        axioms.push(Box::new(InterpolationBounded));
        axioms.push(Box::new(ExtrapolationUnbounded));
        axioms
    }
}

/// Axiom: nearest-neighbour synchronisation error is bounded by `T/2 ·
/// max_rate`, where `T = 1/f` is the measurement period.
///
/// Groves (2013) §17.2.4 — the maximum lag between the target time and
/// the closest available measurement is at most half a sample period.
pub struct NearestNeighborBounded;

impl Axiom for NearestNeighborBounded {
    fn verify(&self) -> Verdict {
        if ErrorBoundedness.get(&SensorTimeConcept::NearestNeighbor) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NearestNeighborBounded",
        "nearest-neighbour sync error bounded by T/2 · max_rate",
        "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. §17.2.4"
    );
}

pr4xis::register_axiom!(
    NearestNeighborBounded,
    "Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems 2nd ed. §17.2.4"
);

/// Axiom: linear-interpolation error is bounded by `T²/8 · a_max` where
/// `a_max` is the maximum second derivative of the signal.
///
/// Bar-Shalom, Li & Kirubarajan (2001) §6.2.3 — the worst-case linear
/// interpolation error for a twice-differentiable signal is
/// `T²/8 · max|f''|`.
pub struct InterpolationBounded;

impl Axiom for InterpolationBounded {
    fn verify(&self) -> Verdict {
        if ErrorBoundedness.get(&SensorTimeConcept::LinearInterpolation) == Some(true) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "InterpolationBounded",
        "linear interpolation error is bounded by T²/8 · a_max",
        "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §6.2.3"
    );
}

pr4xis::register_axiom!(
    InterpolationBounded,
    "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §6.2.3"
);

/// Axiom: extrapolation error grows without bound — no new measurement
/// information is incorporated, so the error compounds with distance.
///
/// Bar-Shalom (2001) §6.2.4 — extrapolation is prediction by the
/// dynamics model alone; the error scales at least linearly with the
/// extrapolation horizon and is unbounded as the horizon grows.
pub struct ExtrapolationUnbounded;

impl Axiom for ExtrapolationUnbounded {
    fn verify(&self) -> Verdict {
        if ErrorBoundedness.get(&SensorTimeConcept::Extrapolation) == Some(false) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ExtrapolationUnbounded",
        "extrapolation error grows without bound (no new information)",
        "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §6.2.4"
    );
}

pr4xis::register_axiom!(
    ExtrapolationUnbounded,
    "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §6.2.4"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<SensorTimeCategory>();
    }

    #[test]
    fn ontology_validates() {
        SensorTimeOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn three_sync_strategies() {
        assert_eq!(SensorTimeConcept::variants().len(), 3);
    }

    #[test]
    fn extrapolation_opposes_bounded_strategies() {
        let opp: Vec<_> = SensorTimeCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == SensorTimeRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(
            SensorTimeConcept::NearestNeighbor,
            SensorTimeConcept::Extrapolation
        )));
        assert!(opp.contains(&(
            SensorTimeConcept::LinearInterpolation,
            SensorTimeConcept::Extrapolation
        )));
    }

    #[test]
    fn error_boundedness_total() {
        for s in SensorTimeConcept::variants() {
            assert!(ErrorBoundedness.get(&s).is_some());
        }
    }

    #[test]
    fn nearest_neighbor_bounded_axiom() {
        assert!(NearestNeighborBounded.verify().is_ok());
    }

    #[test]
    fn interpolation_bounded_axiom() {
        assert!(InterpolationBounded.verify().is_ok());
    }

    #[test]
    fn extrapolation_unbounded_axiom() {
        assert!(ExtrapolationUnbounded.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = SensorTimeConcept> {
        proptest::sample::select(SensorTimeConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in SensorTimeCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in SensorTimeOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }

        #[test]
        fn prop_error_boundedness_total(c in arb_concept()) {
            prop_assert!(ErrorBoundedness.get(&c).is_some());
        }

        #[test]
        fn prop_only_extrapolation_unbounded(c in arb_concept()) {
            // Only the Extrapolation variant returns false.
            let bounded = ErrorBoundedness.get(&c).unwrap();
            let is_extrapolation = matches!(c, SensorTimeConcept::Extrapolation);
            prop_assert_eq!(bounded, !is_extrapolation);
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = SensorTimeCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == SensorTimeRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }
    }
}
