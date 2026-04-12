use crate::science::math::linear_algebra::matrix::Matrix;
use crate::science::math::linear_algebra::positive_definite;
use crate::science::math::linear_algebra::vector_space::Vector;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::technology::sensor_fusion::fusion::engine::{FusionAction, new_fusion_engine};
use crate::technology::sensor_fusion::fusion::ontology::*;
use crate::technology::sensor_fusion::state::estimate::StateEstimate;

// ---------------------------------------------------------------------------
// Ontology validation — includes DETERMINISM axiom
// ---------------------------------------------------------------------------

#[test]
fn fusion_category_laws() {
    check_category_laws::<FusionCategory>().unwrap();
}

#[test]
fn fusion_ontology_validates() {
    FusionOntology::validate().unwrap();
}

#[test]
fn axiom_determinism() {
    assert!(Determinism.holds());
}

#[test]
fn axiom_predict_increases_uncertainty() {
    assert!(PredictIncreasesUncertainty.holds());
}

#[test]
fn axiom_update_reduces_uncertainty() {
    assert!(UpdateReducesUncertainty.holds());
}

#[test]
fn axiom_covariance_invariant() {
    assert!(CovarianceInvariant.holds());
}

// ---------------------------------------------------------------------------
// Engine proofs
// ---------------------------------------------------------------------------

/// 1D constant position Kalman filter: the simplest possible fusion.
///
/// State: [position]. One sensor measures position with noise.
/// Predict: F=I (static), Q=small.
/// Update: H=[1], z=measurement, R=sensor_noise.
fn simple_1d_filter() -> (StateEstimate, Matrix, Matrix, Matrix, Matrix) {
    let x0 = Vector::new(vec![0.0]);
    let p0 = Matrix::new(1, 1, vec![100.0]); // large initial uncertainty
    let initial = StateEstimate::new(x0, p0, 0.0);

    let f = Matrix::identity(1); // static model
    let q = Matrix::new(1, 1, vec![0.01]); // small process noise
    let h = Matrix::new(1, 1, vec![1.0]); // direct observation
    let r = Matrix::new(1, 1, vec![1.0]); // measurement noise

    (initial, f, q, h, r)
}

#[test]
fn fusion_engine_predict_increases_uncertainty() {
    let (initial, f, q, _h, _r) = simple_1d_filter();
    let uncertainty_before = initial.uncertainty();

    let engine = new_fusion_engine(initial);
    let engine = engine
        .try_next(FusionAction::Predict {
            dt: 1.0,
            transition: f,
            process_noise: q,
        })
        .unwrap();

    let uncertainty_after = engine.situation().estimate.uncertainty();
    assert!(
        uncertainty_after >= uncertainty_before,
        "predict must increase uncertainty: {} >= {}",
        uncertainty_after,
        uncertainty_before
    );
}

#[test]
fn fusion_engine_update_reduces_uncertainty() {
    let (initial, f, q, h, r) = simple_1d_filter();

    let engine = new_fusion_engine(initial);

    // Predict first
    let engine = engine
        .try_next(FusionAction::Predict {
            dt: 1.0,
            transition: f,
            process_noise: q,
        })
        .unwrap();

    let uncertainty_before = engine.situation().estimate.uncertainty();

    // Update with measurement at position 5.0
    let engine = engine
        .try_next(FusionAction::Update {
            observation_matrix: h,
            measurement: Vector::new(vec![5.0]),
            measurement_noise: r,
        })
        .unwrap();

    let uncertainty_after = engine.situation().estimate.uncertainty();
    assert!(
        uncertainty_after < uncertainty_before,
        "update must reduce uncertainty: {} < {}",
        uncertainty_after,
        uncertainty_before
    );
}

#[test]
fn fusion_engine_covariance_stays_psd() {
    let (initial, f, q, h, r) = simple_1d_filter();

    let mut engine = new_fusion_engine(initial);

    // Run 10 predict/update cycles
    for i in 0..10 {
        engine = engine
            .try_next(FusionAction::Predict {
                dt: 1.0,
                transition: f.clone(),
                process_noise: q.clone(),
            })
            .unwrap();

        engine = engine
            .try_next(FusionAction::Update {
                observation_matrix: h.clone(),
                measurement: Vector::new(vec![i as f64 * 0.5]),
                measurement_noise: r.clone(),
            })
            .unwrap();

        assert!(
            positive_definite::is_positive_semidefinite(&engine.situation().estimate.covariance),
            "covariance must remain PSD at step {}",
            i
        );
    }
}

#[test]
fn fusion_engine_state_converges_to_measurement() {
    let (initial, f, q, h, r) = simple_1d_filter();

    let mut engine = new_fusion_engine(initial);
    let true_position = 5.0;

    // Feed repeated measurements at the true position
    for _ in 0..20 {
        engine = engine
            .try_next(FusionAction::Predict {
                dt: 1.0,
                transition: f.clone(),
                process_noise: q.clone(),
            })
            .unwrap();

        engine = engine
            .try_next(FusionAction::Update {
                observation_matrix: h.clone(),
                measurement: Vector::new(vec![true_position]),
                measurement_noise: r.clone(),
            })
            .unwrap();
    }

    let estimated = engine.situation().estimate.state.get(0);
    assert!(
        (estimated - true_position).abs() < 0.5,
        "state should converge to true position: {} vs {}",
        estimated,
        true_position
    );
}

#[test]
fn fusion_engine_negative_dt_rejected() {
    let (initial, f, q, _h, _r) = simple_1d_filter();
    let engine = new_fusion_engine(initial);

    let result = engine.try_next(FusionAction::Predict {
        dt: -1.0,
        transition: f,
        process_noise: q,
    });

    assert!(result.is_err(), "negative dt should be rejected");
}

#[test]
fn fusion_engine_dimension_mismatch_rejected() {
    let (initial, _f, _q, _h, _r) = simple_1d_filter();
    let engine = new_fusion_engine(initial);

    // Try to update with 2D measurement on 1D state
    let result = engine.try_next(FusionAction::Update {
        observation_matrix: Matrix::new(2, 2, vec![1.0, 0.0, 0.0, 1.0]),
        measurement: Vector::new(vec![1.0, 2.0]),
        measurement_noise: Matrix::identity(2),
    });

    assert!(result.is_err(), "dimension mismatch should be rejected");
}

#[test]
fn fusion_engine_back_forward() {
    let (initial, f, q, h, r) = simple_1d_filter();

    let engine = new_fusion_engine(initial);
    let engine = engine
        .try_next(FusionAction::Predict {
            dt: 1.0,
            transition: f.clone(),
            process_noise: q.clone(),
        })
        .unwrap();

    let state_after_predict = engine.situation().clone();

    let engine = engine
        .try_next(FusionAction::Update {
            observation_matrix: h,
            measurement: Vector::new(vec![3.0]),
            measurement_noise: r,
        })
        .unwrap();

    // Go back to after-predict state
    let engine = engine.back().unwrap();
    assert_eq!(engine.situation(), &state_after_predict);

    // Go forward again
    let engine = engine.forward().unwrap();
    assert_ne!(engine.situation(), &state_after_predict); // should be after update
}

#[test]
fn fusion_engine_trace_records_all_steps() {
    let (initial, f, q, h, r) = simple_1d_filter();
    let engine = new_fusion_engine(initial);

    let engine = engine
        .try_next(FusionAction::Predict {
            dt: 1.0,
            transition: f,
            process_noise: q,
        })
        .unwrap();

    let engine = engine
        .try_next(FusionAction::Update {
            observation_matrix: h,
            measurement: Vector::new(vec![1.0]),
            measurement_noise: r,
        })
        .unwrap();

    assert_eq!(engine.trace().entries().len(), 2);
    assert!(engine.trace().entries()[0].success);
    assert!(engine.trace().entries()[1].success);
}

// ---------------------------------------------------------------------------
// Property-based proofs — the fusion engine must be DETERMINISTIC
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    /// Generate a random positive definite 1x1 matrix (scalar variance).

    /// Generate a random 1D state estimate.
    fn arb_state_1d() -> impl Strategy<Value = StateEstimate> {
        (-100.0..100.0_f64, 0.01..100.0_f64).prop_map(|(x, p)| {
            StateEstimate::new(Vector::new(vec![x]), Matrix::new(1, 1, vec![p]), 0.0)
        })
    }

    /// Generate a random positive definite 2x2 matrix.
    fn arb_pd_2x2() -> impl Strategy<Value = Matrix> {
        (1.0..10.0_f64, -0.5..0.5_f64, 1.0..10.0_f64).prop_map(|(a, b, d)| {
            // M^T M + εI guarantees PD
            Matrix::new(2, 2, vec![a + 0.1, b, b, d + 0.1])
        })
    }

    /// Generate a random 2D state estimate.
    fn arb_state_2d() -> impl Strategy<Value = StateEstimate> {
        (-100.0..100.0_f64, -100.0..100.0_f64, arb_pd_2x2())
            .prop_map(|(x1, x2, p)| StateEstimate::new(Vector::new(vec![x1, x2]), p, 0.0))
    }

    // --- DETERMINISM: same inputs → same outputs, always ---

    proptest! {
        #[test]
        fn fusion_is_deterministic_predict(
            state in arb_state_1d(),
            dt in 0.001..10.0_f64,
            q_val in 0.001..1.0_f64,
        ) {
            let f = Matrix::identity(1);
            let q = Matrix::new(1, 1, vec![q_val]);

            let engine1 = new_fusion_engine(state.clone());
            let engine2 = new_fusion_engine(state);

            let action = FusionAction::Predict {
                dt,
                transition: f.clone(),
                process_noise: q.clone(),
            };

            let r1 = engine1.try_next(action.clone()).unwrap();
            let r2 = engine2.try_next(action).unwrap();

            // Exact equality — deterministic means bit-for-bit identical
            prop_assert!(r1.situation().estimate.state.data == r2.situation().estimate.state.data);
            prop_assert!(r1.situation().estimate.covariance.data == r2.situation().estimate.covariance.data);
        }

        #[test]
        fn fusion_is_deterministic_update(
            state in arb_state_1d(),
            z_val in -100.0..100.0_f64,
            r_val in 0.01..10.0_f64,
        ) {
            let h = Matrix::new(1, 1, vec![1.0]);
            let z = Vector::new(vec![z_val]);
            let r = Matrix::new(1, 1, vec![r_val]);

            let engine1 = new_fusion_engine(state.clone());
            let engine2 = new_fusion_engine(state);

            let action = FusionAction::Update {
                observation_matrix: h.clone(),
                measurement: z.clone(),
                measurement_noise: r.clone(),
            };

            let r1 = engine1.try_next(action.clone()).unwrap();
            let r2 = engine2.try_next(action).unwrap();

            prop_assert!(r1.situation().estimate.state.data == r2.situation().estimate.state.data);
            prop_assert!(r1.situation().estimate.covariance.data == r2.situation().estimate.covariance.data);
        }

        #[test]
        fn fusion_is_deterministic_full_cycle(
            state in arb_state_1d(),
            dt in 0.01..1.0_f64,
            z_val in -50.0..50.0_f64,
        ) {
            let f = Matrix::identity(1);
            let q = Matrix::new(1, 1, vec![0.01]);
            let h = Matrix::new(1, 1, vec![1.0]);
            let r = Matrix::new(1, 1, vec![1.0]);

            let mut e1 = new_fusion_engine(state.clone());
            let mut e2 = new_fusion_engine(state);

            // 5 predict/update cycles — results must be identical
            for i in 0..5 {
                let predict = FusionAction::Predict {
                    dt,
                    transition: f.clone(),
                    process_noise: q.clone(),
                };
                e1 = e1.try_next(predict.clone()).unwrap();
                e2 = e2.try_next(predict).unwrap();

                let update = FusionAction::Update {
                    observation_matrix: h.clone(),
                    measurement: Vector::new(vec![z_val + i as f64]),
                    measurement_noise: r.clone(),
                };
                e1 = e1.try_next(update.clone()).unwrap();
                e2 = e2.try_next(update).unwrap();
            }

            prop_assert!(e1.situation().estimate.state.data == e2.situation().estimate.state.data);
        }

        // --- PREDICT: uncertainty always increases ---

        #[test]
        fn predict_never_decreases_uncertainty(
            state in arb_state_1d(),
            dt in 0.0..10.0_f64,
            q_val in 0.0..1.0_f64,
        ) {
            let f = Matrix::identity(1);
            let q = Matrix::new(1, 1, vec![q_val]);
            let before = state.uncertainty();

            let engine = new_fusion_engine(state);
            let engine = engine.try_next(FusionAction::Predict {
                dt,
                transition: f,
                process_noise: q,
            }).unwrap();

            let after = engine.situation().estimate.uncertainty();
            prop_assert!(after >= before - 1e-10,
                "predict must not decrease uncertainty: {} -> {}", before, after);
        }

        // --- UPDATE: uncertainty never increases ---

        #[test]
        fn update_never_increases_uncertainty(
            state in arb_state_1d(),
            z_val in -100.0..100.0_f64,
            r_val in 0.01..100.0_f64,
        ) {
            let h = Matrix::new(1, 1, vec![1.0]);
            let z = Vector::new(vec![z_val]);
            let r = Matrix::new(1, 1, vec![r_val]);
            let before = state.uncertainty();

            let engine = new_fusion_engine(state);
            let engine = engine.try_next(FusionAction::Update {
                observation_matrix: h,
                measurement: z,
                measurement_noise: r,
            }).unwrap();

            let after = engine.situation().estimate.uncertainty();
            prop_assert!(after <= before + 1e-10,
                "update must not increase uncertainty: {} -> {}", before, after);
        }

        // --- COVARIANCE STAYS PSD THROUGH RANDOM SEQUENCES ---

        #[test]
        fn covariance_stays_psd_random_sequence(
            state in arb_state_1d(),
            measurements in proptest::collection::vec(-50.0..50.0_f64, 1..20),
        ) {
            let f = Matrix::identity(1);
            let q = Matrix::new(1, 1, vec![0.1]);
            let h = Matrix::new(1, 1, vec![1.0]);
            let r = Matrix::new(1, 1, vec![1.0]);

            let mut engine = new_fusion_engine(state);

            for z_val in &measurements {
                engine = engine.try_next(FusionAction::Predict {
                    dt: 1.0,
                    transition: f.clone(),
                    process_noise: q.clone(),
                }).unwrap();

                engine = engine.try_next(FusionAction::Update {
                    observation_matrix: h.clone(),
                    measurement: Vector::new(vec![*z_val]),
                    measurement_noise: r.clone(),
                }).unwrap();

                prop_assert!(
                    positive_definite::is_positive_semidefinite(
                        &engine.situation().estimate.covariance
                    ),
                    "covariance must stay PSD"
                );
            }
        }

        // --- 2D: PSD preservation in higher dimensions ---

        #[test]
        fn covariance_stays_psd_2d(
            state in arb_state_2d(),
            z1 in -50.0..50.0_f64,
            z2 in -50.0..50.0_f64,
        ) {
            let f = Matrix::identity(2);
            let q = Matrix::new(2, 2, vec![0.1, 0.0, 0.0, 0.1]);
            let h = Matrix::identity(2);
            let r = Matrix::new(2, 2, vec![1.0, 0.0, 0.0, 1.0]);

            let engine = new_fusion_engine(state);

            let engine = engine.try_next(FusionAction::Predict {
                dt: 1.0,
                transition: f,
                process_noise: q,
            }).unwrap();

            let engine = engine.try_next(FusionAction::Update {
                observation_matrix: h,
                measurement: Vector::new(vec![z1, z2]),
                measurement_noise: r,
            }).unwrap();

            prop_assert!(
                positive_definite::is_positive_semidefinite(
                    &engine.situation().estimate.covariance
                ),
                "2D covariance must stay PSD"
            );
        }

        // --- NEGATIVE DT ALWAYS REJECTED ---

        #[test]
        fn negative_dt_always_rejected(
            state in arb_state_1d(),
            dt in -100.0..-0.001_f64,
        ) {
            let engine = new_fusion_engine(state);
            let result = engine.try_next(FusionAction::Predict {
                dt,
                transition: Matrix::identity(1),
                process_noise: Matrix::new(1, 1, vec![0.1]),
            });
            prop_assert!(result.is_err());
        }

        // --- STATE CONVERGES with repeated measurements ---

        #[test]
        fn state_converges_to_measurement(
            true_val in -50.0..50.0_f64,
            initial_guess in -100.0..100.0_f64,
        ) {
            let state = StateEstimate::new(
                Vector::new(vec![initial_guess]),
                Matrix::new(1, 1, vec![100.0]),
                0.0,
            );
            let f = Matrix::identity(1);
            let q = Matrix::new(1, 1, vec![0.01]);
            let h = Matrix::new(1, 1, vec![1.0]);
            let r = Matrix::new(1, 1, vec![1.0]);

            let mut engine = new_fusion_engine(state);

            for _ in 0..50 {
                engine = engine.try_next(FusionAction::Predict {
                    dt: 1.0,
                    transition: f.clone(),
                    process_noise: q.clone(),
                }).unwrap();

                engine = engine.try_next(FusionAction::Update {
                    observation_matrix: h.clone(),
                    measurement: Vector::new(vec![true_val]),
                    measurement_noise: r.clone(),
                }).unwrap();
            }

            let estimated = engine.situation().estimate.state.get(0);
            prop_assert!((estimated - true_val).abs() < 1.0,
                "should converge: estimated={}, true={}", estimated, true_val);
        }

        // --- BACK/FORWARD preserves exact state ---

        #[test]
        fn back_forward_preserves_state(
            state in arb_state_1d(),
            z_val in -50.0..50.0_f64,
        ) {
            let engine = new_fusion_engine(state);

            let engine = engine.try_next(FusionAction::Predict {
                dt: 1.0,
                transition: Matrix::identity(1),
                process_noise: Matrix::new(1, 1, vec![0.1]),
            }).unwrap();

            let state_after_predict = engine.situation().estimate.state.data.clone();

            let engine = engine.try_next(FusionAction::Update {
                observation_matrix: Matrix::new(1, 1, vec![1.0]),
                measurement: Vector::new(vec![z_val]),
                measurement_noise: Matrix::new(1, 1, vec![1.0]),
            }).unwrap();

            let state_after_update = engine.situation().estimate.state.data.clone();

            // Back to predict state
            let engine = engine.back().unwrap();
            prop_assert!(engine.situation().estimate.state.data == state_after_predict);

            // Forward to update state
            let engine = engine.forward().unwrap();
            prop_assert!(engine.situation().estimate.state.data == state_after_update);
        }
    }
}

// ---------------------------------------------------------------------------
// C1: Singular innovation covariance returns Err
// ---------------------------------------------------------------------------

#[test]
fn singular_innovation_covariance_returns_err() {
    // Create a state with zero covariance (P=0) and zero measurement noise (R=0).
    // This makes S = H P H^T + R = 0, which is singular.
    let x0 = Vector::new(vec![0.0]);
    let p0 = Matrix::new(1, 1, vec![0.0]); // zero covariance
    let initial = StateEstimate {
        state: x0,
        covariance: p0,
        epoch: 0.0,
        step: 0,
    };
    let state = crate::technology::sensor_fusion::fusion::engine::FusionState {
        estimate: initial,
        sensors_active: 0,
    };

    let action = FusionAction::Update {
        observation_matrix: Matrix::new(1, 1, vec![1.0]),
        measurement: Vector::new(vec![1.0]),
        measurement_noise: Matrix::new(1, 1, vec![0.0]), // zero noise
    };

    let result = crate::technology::sensor_fusion::fusion::engine::apply_fusion(&state, &action);
    assert!(
        result.is_err(),
        "singular S must return Err, got {:?}",
        result
    );
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("singular"),
        "error message should mention singular: {}",
        err_msg
    );
}

// ---------------------------------------------------------------------------
// C2: DimensionConsistency column checks
// ---------------------------------------------------------------------------

#[test]
fn dimension_consistency_rejects_non_square_process_noise() {
    let (initial, f, _q, _h, _r) = simple_1d_filter();
    let engine = new_fusion_engine(initial);

    // Process noise with wrong column count: 1x2 instead of 1x1
    let bad_q = Matrix::new(1, 2, vec![0.01, 0.0]);
    let result = engine.try_next(FusionAction::Predict {
        dt: 1.0,
        transition: f,
        process_noise: bad_q,
    });
    assert!(
        result.is_err(),
        "non-square process noise should be rejected"
    );
}

#[test]
fn dimension_consistency_rejects_non_square_measurement_noise() {
    let (initial, _f, _q, h, _r) = simple_1d_filter();
    let engine = new_fusion_engine(initial);

    // Measurement noise with wrong column count: 1x2 instead of 1x1
    let bad_r = Matrix::new(1, 2, vec![1.0, 0.0]);
    let result = engine.try_next(FusionAction::Update {
        observation_matrix: h,
        measurement: Vector::new(vec![5.0]),
        measurement_noise: bad_r,
    });
    assert!(
        result.is_err(),
        "non-square measurement noise should be rejected"
    );
}

// ---------------------------------------------------------------------------
// H11: solve_spd failure propagation test
// ---------------------------------------------------------------------------

#[test]
fn solve_spd_failure_propagates_to_engine() {
    // S = HPH^T + R. If P=0 and R=0, then S=0, which is singular.
    // This directly tests that the Kalman gain computation propagates the error.
    let x0 = Vector::new(vec![0.0]);
    let p0 = Matrix::new(1, 1, vec![0.0]); // zero covariance
    let initial = StateEstimate {
        state: x0,
        covariance: p0,
        epoch: 0.0,
        step: 0,
    };
    let state = crate::technology::sensor_fusion::fusion::engine::FusionState {
        estimate: initial,
        sensors_active: 0,
    };

    let action = FusionAction::Update {
        observation_matrix: Matrix::new(1, 1, vec![1.0]),
        measurement: Vector::new(vec![1.0]),
        measurement_noise: Matrix::new(1, 1, vec![0.0]), // zero noise
    };

    let result = crate::technology::sensor_fusion::fusion::engine::apply_fusion(&state, &action);
    assert!(
        result.is_err(),
        "singular innovation covariance (S=0) should return Err"
    );
}

// ---------------------------------------------------------------------------
// H12: Non-identity (constant velocity) transition matrix proptest
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_cv_model {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn fusion_with_cv_model(
            dt in 0.01..1.0_f64,
            pos in -50.0..50.0_f64,
            vel in -10.0..10.0_f64,
            q_scale in 0.01..1.0_f64,
        ) {
            // Constant velocity model: state = [position, velocity]
            // F = [[1, dt], [0, 1]]
            let f = Matrix::new(2, 2, vec![1.0, dt, 0.0, 1.0]);
            let q = Matrix::new(2, 2, vec![q_scale * dt, 0.0, 0.0, q_scale]);
            let p0 = Matrix::new(2, 2, vec![10.0, 0.0, 0.0, 10.0]);
            let state = StateEstimate::new(Vector::new(vec![pos, vel]), p0, 0.0);

            let engine = new_fusion_engine(state);
            let engine = engine.try_next(FusionAction::Predict {
                dt,
                transition: f,
                process_noise: q,
            }).unwrap();

            // Position should advance by vel*dt
            let predicted_pos = engine.situation().estimate.state.get(0);
            let expected_pos = pos + vel * dt;
            prop_assert!(
                (predicted_pos - expected_pos).abs() < 1e-10,
                "CV model: predicted={}, expected={}",
                predicted_pos, expected_pos
            );

            // Velocity should remain unchanged
            let predicted_vel = engine.situation().estimate.state.get(1);
            prop_assert!(
                (predicted_vel - vel).abs() < 1e-10,
                "CV model: velocity predicted={}, expected={}",
                predicted_vel, vel
            );

            // Covariance must remain PSD
            prop_assert!(
                positive_definite::is_positive_semidefinite(
                    &engine.situation().estimate.covariance
                ),
                "covariance must stay PSD with CV model"
            );
        }
    }
}
