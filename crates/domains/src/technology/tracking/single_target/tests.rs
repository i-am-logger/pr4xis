use crate::science::math::linear_algebra::positive_definite;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::Ontology;

use crate::technology::tracking::single_target::engine::*;
use crate::technology::tracking::single_target::motion_model;
use crate::technology::tracking::single_target::ontology::*;

#[test]
fn target_state_category_laws() {
    check_category_laws::<TargetStateCategory>().unwrap();
}

#[test]
fn single_target_ontology_validates() {
    SingleTargetOntology::validate().unwrap();
}

#[test]
fn cv_tracker_converges_to_true_position() {
    let mut engine = new_cv_tracker_1d(0.0, 0.0, 100.0, 0.1, 1.0);
    let true_pos = 10.0;
    let true_vel = 2.0;

    for i in 0..30 {
        let dt = 1.0;
        let t = i as f64;
        engine = engine.try_next(cv_predict_1d(dt, 0.1)).unwrap();
        engine = engine
            .try_next(cv_update_position_1d(true_pos + true_vel * t, 1.0))
            .unwrap();
    }

    let est_pos = engine.situation().estimate.state.get(0);
    // Should be near true_pos + true_vel * 29
    let expected = true_pos + true_vel * 29.0;
    assert!(
        (est_pos - expected).abs() < 5.0,
        "should converge: est={}, expected={}",
        est_pos,
        expected
    );
}

#[test]
fn cv_tracker_covariance_stays_psd() {
    let mut engine = new_cv_tracker_1d(0.0, 0.0, 100.0, 0.1, 1.0);

    for i in 0..20 {
        engine = engine.try_next(cv_predict_1d(1.0, 0.1)).unwrap();
        engine = engine
            .try_next(cv_update_position_1d(i as f64 * 0.5, 1.0))
            .unwrap();
        assert!(positive_definite::is_positive_semidefinite(
            &engine.situation().estimate.covariance
        ));
    }
}

#[test]
fn cv_motion_model_identity_at_zero_dt() {
    let (f, q) = motion_model::constant_velocity_1d(0.0, 0.1);
    // F at dt=0 should be identity
    assert!((f.get(0, 0) - 1.0).abs() < 1e-12);
    assert!((f.get(0, 1) - 0.0).abs() < 1e-12);
    assert!((f.get(1, 0) - 0.0).abs() < 1e-12);
    assert!((f.get(1, 1) - 1.0).abs() < 1e-12);
    // Q at dt=0 should be zero
    assert!(q.get(0, 0).abs() < 1e-12);
}

#[test]
fn cv_motion_model_process_noise_is_symmetric() {
    let (_, q) = motion_model::constant_velocity_1d(1.0, 0.5);
    assert!(q.is_symmetric(1e-12));
}

#[test]
fn ca_motion_model_process_noise_is_symmetric() {
    let (_, q) = motion_model::constant_acceleration_1d(1.0, 0.5);
    assert!(q.is_symmetric(1e-12));
}

#[test]
fn cv_2d_model_is_block_diagonal() {
    let (f, q) = motion_model::constant_velocity_2d(1.0, 0.5);
    // Cross-block terms should be zero
    assert!(f.get(0, 2).abs() < 1e-12);
    assert!(f.get(0, 3).abs() < 1e-12);
    assert!(f.get(2, 0).abs() < 1e-12);
    assert!(q.get(0, 2).abs() < 1e-12);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn cv_model_process_noise_symmetric(dt in 0.01..10.0_f64, q in 0.001..10.0_f64) {
            let (_, qm) = motion_model::constant_velocity_1d(dt, q);
            prop_assert!(qm.is_symmetric(1e-12));
        }

        #[test]
        fn ca_model_process_noise_symmetric(dt in 0.01..10.0_f64, q in 0.001..10.0_f64) {
            let (_, qm) = motion_model::constant_acceleration_1d(dt, q);
            prop_assert!(qm.is_symmetric(1e-12));
        }

        #[test]
        fn cv_model_transition_at_zero_dt_is_identity(q in 0.001..10.0_f64) {
            let (f, _) = motion_model::constant_velocity_1d(0.0, q);
            prop_assert!((f.get(0, 0) - 1.0).abs() < 1e-12);
            prop_assert!((f.get(1, 1) - 1.0).abs() < 1e-12);
            prop_assert!(f.get(0, 1).abs() < 1e-12);
            prop_assert!(f.get(1, 0).abs() < 1e-12);
        }

        #[test]
        fn tracker_is_deterministic(
            initial in -50.0..50.0_f64,
            meas in -50.0..50.0_f64,
        ) {
            let e1 = new_cv_tracker_1d(initial, 0.0, 100.0, 0.1, 1.0);
            let e2 = new_cv_tracker_1d(initial, 0.0, 100.0, 0.1, 1.0);

            let e1 = e1.try_next(cv_predict_1d(1.0, 0.1)).unwrap();
            let e2 = e2.try_next(cv_predict_1d(1.0, 0.1)).unwrap();

            let e1 = e1.try_next(cv_update_position_1d(meas, 1.0)).unwrap();
            let e2 = e2.try_next(cv_update_position_1d(meas, 1.0)).unwrap();

            prop_assert!(e1.situation().estimate.state.data == e2.situation().estimate.state.data);
        }
    }
}
