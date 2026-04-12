use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::science::math::linear_algebra::matrix::Matrix;
use crate::science::math::linear_algebra::positive_definite;
use crate::science::math::linear_algebra::vector_space::Vector;

use crate::technology::sensor_fusion::fusion::engine::{FusionAction, FusionState, apply_fusion};
use crate::technology::sensor_fusion::state::estimate::StateEstimate;

// ---------------------------------------------------------------------------
// Entity: fusion phases (JDL Data Fusion Process Model)
// ---------------------------------------------------------------------------

/// Phases in the sensor fusion lifecycle.
///
/// Source: US DoD JDL (1999). "Data Fusion Lexicon."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FusionPhase {
    /// Filter initialized, waiting for first measurement.
    Initialized,
    /// Time update (prediction) completed.
    Predicted,
    /// Measurement update completed.
    Updated,
    /// Filter detected divergence (covariance not PSD).
    Diverged,
    /// Filter reset to initial conditions.
    Reset,
}

impl Entity for FusionPhase {
    fn variants() -> Vec<Self> {
        vec![
            Self::Initialized,
            Self::Predicted,
            Self::Updated,
            Self::Diverged,
            Self::Reset,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhaseTransition {
    pub from: FusionPhase,
    pub to: FusionPhase,
}

impl Relationship for PhaseTransition {
    type Object = FusionPhase;
    fn source(&self) -> FusionPhase {
        self.from
    }
    fn target(&self) -> FusionPhase {
        self.to
    }
}

/// Fusion phase category: the predict/update cycle.
pub struct FusionCategory;

impl Category for FusionCategory {
    type Object = FusionPhase;
    type Morphism = PhaseTransition;

    fn identity(obj: &FusionPhase) -> PhaseTransition {
        PhaseTransition {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &PhaseTransition, g: &PhaseTransition) -> Option<PhaseTransition> {
        if f.to != g.from {
            return None;
        }
        Some(PhaseTransition {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<PhaseTransition> {
        use FusionPhase::*;
        let mut m = Vec::new();

        for phase in FusionPhase::variants() {
            m.push(FusionCategory::identity(&phase));
        }

        // Valid transitions: the predict/update cycle
        m.push(PhaseTransition {
            from: Initialized,
            to: Predicted,
        });
        m.push(PhaseTransition {
            from: Predicted,
            to: Updated,
        });
        m.push(PhaseTransition {
            from: Updated,
            to: Predicted,
        }); // cycle
        m.push(PhaseTransition {
            from: Predicted,
            to: Diverged,
        });
        m.push(PhaseTransition {
            from: Updated,
            to: Diverged,
        });
        m.push(PhaseTransition {
            from: Diverged,
            to: Reset,
        });
        m.push(PhaseTransition {
            from: Reset,
            to: Initialized,
        });

        // Transitive closure
        m.push(PhaseTransition {
            from: Initialized,
            to: Updated,
        });
        m.push(PhaseTransition {
            from: Updated,
            to: Updated,
        });
        m.push(PhaseTransition {
            from: Updated,
            to: Diverged,
        });
        m.push(PhaseTransition {
            from: Initialized,
            to: Diverged,
        });
        m.push(PhaseTransition {
            from: Predicted,
            to: Predicted,
        });
        m.push(PhaseTransition {
            from: Diverged,
            to: Initialized,
        });
        m.push(PhaseTransition {
            from: Reset,
            to: Predicted,
        });
        // Full reachability chains
        for &from in &FusionPhase::variants() {
            for &to in &FusionPhase::variants() {
                let t = PhaseTransition { from, to };
                if !m.contains(&t) {
                    m.push(t);
                }
            }
        }

        m
    }
}

// ---------------------------------------------------------------------------
// Quality
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PhaseDescription;

impl Quality for PhaseDescription {
    type Individual = FusionPhase;
    type Value = &'static str;

    fn get(&self, phase: &FusionPhase) -> Option<&'static str> {
        Some(match phase {
            FusionPhase::Initialized => "filter initialized, awaiting data",
            FusionPhase::Predicted => "time update complete, state propagated forward",
            FusionPhase::Updated => "measurement incorporated, estimate refined",
            FusionPhase::Diverged => "filter diverged, covariance not PSD",
            FusionPhase::Reset => "filter reset to initial conditions",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms — including DETERMINISM
// ---------------------------------------------------------------------------

/// DETERMINISM AXIOM: the fusion engine is a pure function.
///
/// Given identical initial state and identical action sequence,
/// the output is ALWAYS bit-for-bit identical. No randomness,
/// no hidden state, no platform-dependent behavior.
///
/// This is the foundational property for safety-critical systems.
/// If the engine is not deterministic, it cannot be certified.
pub struct Determinism;

impl Axiom for Determinism {
    fn description(&self) -> &str {
        "fusion engine is deterministic: same inputs always produce same outputs"
    }

    fn holds(&self) -> bool {
        // Test with multiple state/action combinations
        let test_cases = determinism_test_cases();
        for (state, actions) in &test_cases {
            let result1 = run_sequence(state, actions);
            let result2 = run_sequence(state, actions);
            // Bit-for-bit identical
            if result1.estimate.state.data != result2.estimate.state.data {
                return false;
            }
            if result1.estimate.covariance.data != result2.estimate.covariance.data {
                return false;
            }
        }
        true
    }
}

/// Predict step never decreases uncertainty (information theory).
///
/// Without new information (measurement), uncertainty can only grow.
/// This is analogous to the second law of thermodynamics.
pub struct PredictIncreasesUncertainty;

impl Axiom for PredictIncreasesUncertainty {
    fn description(&self) -> &str {
        "prediction step never decreases uncertainty (no free information)"
    }

    fn holds(&self) -> bool {
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
                return false;
            }
        }
        true
    }
}

/// Update step never increases uncertainty (information gain from observation).
pub struct UpdateReducesUncertainty;

impl Axiom for UpdateReducesUncertainty {
    fn description(&self) -> &str {
        "measurement update never increases uncertainty (information gain)"
    }

    fn holds(&self) -> bool {
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
                return false;
            }
        }
        true
    }
}

/// Covariance remains PSD through all operations.
pub struct CovarianceInvariant;

impl Axiom for CovarianceInvariant {
    fn description(&self) -> &str {
        "covariance remains positive semi-definite through predict and update"
    }

    fn holds(&self) -> bool {
        for (state, actions) in &determinism_test_cases() {
            let result = run_sequence(state, actions);
            if !positive_definite::is_positive_semidefinite(&result.estimate.covariance) {
                return false;
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The sensor fusion ontology.
///
/// Founded on:
///   - Kalman, R.E. (1960). "A New Approach to Linear Filtering and Prediction Problems."
///   - Maybeck, P.S. (1979). *Stochastic Models, Estimation, and Control.*
///   - Bar-Shalom et al. (2001). *Estimation with Applications to Tracking and Navigation.*
///   - US DoD JDL (1999). "Data Fusion Lexicon." (fusion levels)
///
/// Key property: DETERMINISM — the engine is a pure function.
pub struct FusionOntology;

impl Ontology for FusionOntology {
    type Cat = FusionCategory;
    type Qual = PhaseDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(Determinism),
            Box::new(PredictIncreasesUncertainty),
            Box::new(UpdateReducesUncertainty),
            Box::new(CovarianceInvariant),
        ]
    }
}

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
