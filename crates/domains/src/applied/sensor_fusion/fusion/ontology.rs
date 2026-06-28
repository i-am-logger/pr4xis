//! Sensor fusion — Kalman filter lifecycle and engine determinism.
//!
//! Models the predict/update cycle of a Bayesian state-estimation engine
//! as the phases an estimator passes through, plus the foundational
//! engine properties (determinism, PSD-preservation, monotone
//! uncertainty change per direction) that safety-critical certification
//! depends on.
//!
//! # Literature
//!
//! - **Kalman (1960)** "A New Approach to Linear Filtering and Prediction
//!   Problems", *Trans. ASME J. Basic Engineering* 82(D) — the Kalman
//!   filter; the predict/update recursion; the canonical equations
//!   relating prior, measurement, gain, and posterior.
//! - **Maybeck (1979)** *Stochastic Models, Estimation, and Control*,
//!   Vol. 1 — the textbook treatment of the Kalman recursion and its
//!   covariance invariants.
//! - **Bar-Shalom, Li & Kirubarajan (2001)** *Estimation with
//!   Applications to Tracking and Navigation* — the multi-sensor /
//!   multi-target fusion framework.
//! - **US DoD JDL (1999)** *Data Fusion Lexicon* — fusion-process model
//!   levels and phase vocabulary.

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::positive_definite;
use crate::formal::math::linear_algebra::vector_space::Vector;

use crate::applied::sensor_fusion::fusion::engine::{FusionAction, FusionState, apply_fusion};
use crate::applied::sensor_fusion::state::estimate::StateEstimate;

pr4xis::ontology! {
    name: "Fusion",
    source: "Kalman (1960) A New Approach to Linear Filtering and Prediction Problems, Trans. ASME J. Basic Engineering 82(D); Maybeck (1979) Stochastic Models, Estimation, and Control Vol. 1; Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation; US DoD JDL (1999) Data Fusion Lexicon",

    concepts: [
        // Kalman (1960) / Bar-Shalom (2001) filter lifecycle phases.
        Initialized,
        Predicted,
        Updated,
        Diverged,
        Reset,
    ],

    labels: {
        Initialized: ("en", "Initialized",
            "Kalman (1960): the filter has its initial state and covariance and is waiting for the first input."),
        Predicted: ("en", "Predicted",
            "Kalman (1960): the time-update step has been applied — state propagated forward by the dynamics model."),
        Updated: ("en", "Updated",
            "Kalman (1960): a measurement-update step has incorporated a new observation, refining the state estimate."),
        Diverged: ("en", "Diverged",
            "Maybeck (1979): the filter has lost integrity — covariance failed positive-semidefiniteness or innovation grew unboundedly."),
        Reset: ("en", "Reset",
            "Bar-Shalom (2001): the filter has been restored to its initial conditions after divergence detection."),
    },

    opposes: [
        // Predicted (uncertainty grows) vs Updated (uncertainty shrinks).
        (Predicted, Updated),
        (Updated, Predicted),
        // Diverged (failure) vs Updated (success).
        (Diverged, Updated),
        (Updated, Diverged),
    ],
}

/// Quality: a one-line description of each filter phase.
#[derive(Debug, Clone)]
pub struct PhaseDescription;

impl Quality for PhaseDescription {
    type Individual = FusionConcept;
    type Value = &'static str;

    fn get(&self, phase: &FusionConcept) -> Option<&'static str> {
        Some(match phase {
            FusionConcept::Initialized => "filter initialized, awaiting data",
            FusionConcept::Predicted => "time update complete, state propagated forward",
            FusionConcept::Updated => "measurement incorporated, estimate refined",
            FusionConcept::Diverged => "filter diverged, covariance not PSD",
            FusionConcept::Reset => "filter reset to initial conditions",
        })
    }
}

impl Ontology for FusionOntology {
    type Cat = FusionCategory;
    type Qual = PhaseDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(Determinism));
        axioms.push(Box::new(PredictIncreasesUncertainty));
        axioms.push(Box::new(UpdateReducesUncertainty));
        axioms.push(Box::new(CovarianceInvariant));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Axiom: the fusion engine is a deterministic, pure function — identical
/// initial state and identical action sequence yields bit-for-bit
/// identical output. The bedrock property for safety-critical
/// certification; if an engine is non-deterministic, it cannot be
/// certified.
///
/// Kalman (1960) §3 — the recursion is algebraic, with no stochastic
/// term in the update equations themselves; reproducibility follows
/// directly from the recursion form.
pub struct Determinism;

impl Axiom for Determinism {
    fn verify(&self) -> Verdict {
        let test_cases = determinism_test_cases();
        for (state, actions) in &test_cases {
            let r1 = run_sequence(state, actions);
            let r2 = run_sequence(state, actions);
            if r1.estimate.state.data != r2.estimate.state.data {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            if r1.estimate.covariance.data != r2.estimate.covariance.data {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "Determinism",
        "fusion engine is deterministic: same inputs always produce same outputs",
        "Kalman (1960) A New Approach to Linear Filtering and Prediction Problems, Trans. ASME J. Basic Engineering 82(D) §3"
    );
}

pr4xis::register_axiom!(
    Determinism,
    "Kalman (1960) A New Approach to Linear Filtering and Prediction Problems, Trans. ASME J. Basic Engineering 82(D) §3"
);

/// Axiom: predict-step never decreases uncertainty.
///
/// Maybeck (1979) §4.2 — `P_{k|k-1} = F P_{k-1|k-1} F^T + Q` with `Q ≽ 0`
/// implies `trace(P_{k|k-1}) ≥ trace(F P_{k-1|k-1} F^T)`; for the identity
/// dynamics case the inequality is `P_{k|k-1} ≥ P_{k-1|k-1}` in Loewner
/// order. Information-theoretically: no new measurement, no information
/// gain.
pub struct PredictIncreasesUncertainty;

impl Axiom for PredictIncreasesUncertainty {
    fn verify(&self) -> Verdict {
        for (state, _) in &determinism_test_cases() {
            let before = state.uncertainty();
            let f = Matrix::identity(state.dim());
            let q = Matrix::identity(state.dim()).scale(0.1);
            let fusion_state = FusionState {
                estimate: state.clone(),
                sensors_active: 0,
            };
            let after_state = apply_fusion(
                &fusion_state,
                &FusionAction::Predict {
                    dt: 1.0,
                    transition: f,
                    process_noise: q,
                },
            )
            .unwrap();
            if after_state.estimate.uncertainty() < before - 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "PredictIncreasesUncertainty",
        "prediction step never decreases uncertainty (no free information)",
        "Maybeck (1979) Stochastic Models, Estimation, and Control Vol. 1 §4.2"
    );
}

pr4xis::register_axiom!(
    PredictIncreasesUncertainty,
    "Maybeck (1979) Stochastic Models, Estimation, and Control Vol. 1 §4.2"
);

/// Axiom: measurement-update step never increases uncertainty.
///
/// Maybeck (1979) §4.3 — the Joseph form
/// `P_{k|k} = (I − K H) P_{k|k-1} (I − K H)^T + K R K^T`
/// is positive-semidefinite-less-than `P_{k|k-1}` when `R ≽ 0`. New
/// information can only shrink the posterior covariance in Loewner
/// order; the trace inherits the same monotonicity.
pub struct UpdateReducesUncertainty;

impl Axiom for UpdateReducesUncertainty {
    fn verify(&self) -> Verdict {
        for (state, _) in &determinism_test_cases() {
            let before = state.uncertainty();
            let n = state.dim();
            let h = Matrix::identity(n);
            let r = Matrix::identity(n);
            let z = Vector::zeros(n);
            let fusion_state = FusionState {
                estimate: state.clone(),
                sensors_active: 1,
            };
            let after_state = apply_fusion(
                &fusion_state,
                &FusionAction::Update {
                    observation_matrix: h,
                    measurement: z,
                    measurement_noise: r,
                },
            )
            .unwrap();
            if after_state.estimate.uncertainty() > before + 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "UpdateReducesUncertainty",
        "measurement update never increases uncertainty (information gain)",
        "Maybeck (1979) Stochastic Models, Estimation, and Control Vol. 1 §4.3"
    );
}

pr4xis::register_axiom!(
    UpdateReducesUncertainty,
    "Maybeck (1979) Stochastic Models, Estimation, and Control Vol. 1 §4.3"
);

/// Axiom: the state covariance remains positive-semidefinite through
/// every predict and update step.
///
/// Bar-Shalom, Li & Kirubarajan (2001) §5.2 — the Joseph form is the
/// numerically robust covariance update that preserves PSD by
/// construction; loss of PSD signals divergence.
pub struct CovarianceInvariant;

impl Axiom for CovarianceInvariant {
    fn verify(&self) -> Verdict {
        for (state, actions) in &determinism_test_cases() {
            let result = run_sequence(state, actions);
            if !positive_definite::is_positive_semidefinite(&result.estimate.covariance) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "CovarianceInvariant",
        "covariance remains positive semi-definite through predict and update",
        "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §5.2"
    );
}

pr4xis::register_axiom!(
    CovarianceInvariant,
    "Bar-Shalom, Li & Kirubarajan (2001) Estimation with Applications to Tracking and Navigation §5.2"
);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn run_sequence(initial: &StateEstimate, actions: &[FusionAction]) -> FusionState {
    let mut state = FusionState {
        estimate: initial.clone(),
        sensors_active: 1,
    };
    for action in actions {
        state = apply_fusion(&state, action).unwrap();
    }
    state
}

fn determinism_test_cases() -> Vec<(StateEstimate, Vec<FusionAction>)> {
    let s1 = StateEstimate::new(Vector::new(vec![0.0]), Matrix::new(1, 1, vec![10.0]), 0.0);
    let s2 = StateEstimate::new(
        Vector::new(vec![1.0, 2.0]),
        Matrix::new(2, 2, vec![5.0, 1.0, 1.0, 5.0]),
        0.0,
    );

    let actions_1d: Vec<FusionAction> = vec![
        FusionAction::Predict {
            dt: 1.0,
            transition: Matrix::identity(1),
            process_noise: Matrix::new(1, 1, vec![0.1]),
        },
        FusionAction::Update {
            observation_matrix: Matrix::new(1, 1, vec![1.0]),
            measurement: Vector::new(vec![5.0]),
            measurement_noise: Matrix::new(1, 1, vec![1.0]),
        },
        FusionAction::Predict {
            dt: 0.5,
            transition: Matrix::identity(1),
            process_noise: Matrix::new(1, 1, vec![0.05]),
        },
        FusionAction::Update {
            observation_matrix: Matrix::new(1, 1, vec![1.0]),
            measurement: Vector::new(vec![4.8]),
            measurement_noise: Matrix::new(1, 1, vec![1.0]),
        },
    ];

    let actions_2d: Vec<FusionAction> = vec![
        FusionAction::Predict {
            dt: 1.0,
            transition: Matrix::identity(2),
            process_noise: Matrix::new(2, 2, vec![0.1, 0.0, 0.0, 0.1]),
        },
        FusionAction::Update {
            observation_matrix: Matrix::identity(2),
            measurement: Vector::new(vec![3.0, 4.0]),
            measurement_noise: Matrix::identity(2),
        },
    ];

    vec![(s1, actions_1d), (s2, actions_2d)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<FusionCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        FusionOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn five_fusion_phases() {
        assert_eq!(FusionConcept::variants().len(), 5);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn predict_and_update_oppose() {
        let opp: Vec<_> = FusionCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == FusionRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(FusionConcept::Predicted, FusionConcept::Updated)));
        assert!(opp.contains(&(FusionConcept::Updated, FusionConcept::Predicted)));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn determinism_axiom_holds() {
        assert!(Determinism.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn predict_increases_uncertainty_axiom() {
        assert!(PredictIncreasesUncertainty.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn update_reduces_uncertainty_axiom() {
        assert!(UpdateReducesUncertainty.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn covariance_invariant_axiom() {
        assert!(CovarianceInvariant.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn phase_description_total() {
        for c in FusionConcept::variants() {
            assert!(
                PhaseDescription.get(&c).is_some(),
                "{:?} missing description",
                c
            );
        }
    }

    fn arb_concept() -> impl Strategy<Value = FusionConcept> {
        proptest::sample::select(FusionConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in FusionCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in FusionOntology::axioms() {
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
        fn prop_phase_description_total(c in arb_concept()) {
            prop_assert!(PhaseDescription.get(&c).is_some());
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = FusionCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == FusionRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} → {:?} but not back", a, b);
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_phase_description_total, Verifiable);
    pr4xis::register_praxis_value!(prop_opposition_is_symmetric, Verifiable);
}
