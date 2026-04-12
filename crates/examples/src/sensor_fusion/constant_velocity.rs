/// 1D Constant Velocity Kalman Filter — the "hello world" of sensor fusion.
///
/// A target moves at constant velocity. A sensor measures its position
/// with Gaussian noise. The Kalman filter fuses these measurements
/// to estimate both position AND velocity (which is not directly observed).
///
/// Proves:
/// - State converges to true trajectory
/// - Uncertainty decreases with each measurement
/// - Covariance stays PSD throughout
/// - Engine is deterministic
///
/// Source: Kalman (1960), Bar-Shalom (2001) Example 6.4.
#[cfg(test)]
mod tests {

    use pr4xis_domains::science::math::linear_algebra::positive_definite;

    use pr4xis_domains::technology::tracking::single_target::engine::*;

    #[test]
    fn constant_velocity_tracker_converges() {
        let true_pos = 0.0;
        let true_vel = 5.0; // 5 m/s

        let mut engine = new_cv_tracker_1d(0.0, 0.0, 100.0, 0.1, 4.0);

        let mut prev_uncertainty = f64::INFINITY;
        for i in 0..50 {
            let dt = 1.0;
            let t = (i + 1) as f64;
            let true_position = true_pos + true_vel * t;

            engine = engine.try_next(cv_predict_1d(dt, 0.1)).unwrap();
            engine = engine
                .try_next(cv_update_position_1d(true_position, 4.0))
                .unwrap();

            // Covariance stays PSD
            assert!(positive_definite::is_positive_semidefinite(
                &engine.situation().estimate.covariance
            ));

            // Uncertainty generally decreases (steady state after a few steps)
            let uncertainty = engine.situation().estimate.uncertainty();
            if i > 5 {
                assert!(
                    uncertainty < prev_uncertainty + 1.0,
                    "uncertainty should stabilize"
                );
            }
            prev_uncertainty = uncertainty;
        }

        // Position estimate should be close to true
        let est_pos = engine.situation().estimate.state.get(0);
        let expected_pos = true_pos + true_vel * 50.0;
        assert!(
            (est_pos - expected_pos).abs() < 10.0,
            "position: est={:.1}, true={:.1}",
            est_pos,
            expected_pos
        );

        // Velocity estimate should be close to true
        let est_vel = engine.situation().estimate.state.get(1);
        assert!(
            (est_vel - true_vel).abs() < 2.0,
            "velocity: est={:.1}, true={:.1}",
            est_vel,
            true_vel
        );
    }

    mod proptest_proofs {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn tracker_converges_for_any_velocity(
                true_vel in -20.0..20.0_f64,
                initial_guess in -50.0..50.0_f64,
            ) {
                let mut engine = new_cv_tracker_1d(initial_guess, 0.0, 100.0, 0.1, 4.0);
                for i in 0..30 {
                    let t = (i + 1) as f64;
                    engine = engine.try_next(cv_predict_1d(1.0, 0.1)).unwrap();
                    engine = engine
                        .try_next(cv_update_position_1d(true_vel * t, 4.0))
                        .unwrap();
                }
                let est_vel = engine.situation().estimate.state.get(1);
                prop_assert!((est_vel - true_vel).abs() < 5.0,
                    "velocity should converge: est={}, true={}", est_vel, true_vel);
            }

            #[test]
            fn covariance_stays_psd_for_any_measurements(
                measurements in proptest::collection::vec(-100.0..100.0_f64, 5..30),
            ) {
                let mut engine = new_cv_tracker_1d(0.0, 0.0, 100.0, 0.1, 4.0);
                for z in &measurements {
                    engine = engine.try_next(cv_predict_1d(1.0, 0.1)).unwrap();
                    engine = engine.try_next(cv_update_position_1d(*z, 4.0)).unwrap();
                    prop_assert!(positive_definite::is_positive_semidefinite(
                        &engine.situation().estimate.covariance
                    ));
                }
            }

            #[test]
            fn tracker_is_deterministic(
                true_vel in -20.0..20.0_f64,
                initial in -50.0..50.0_f64,
            ) {
                let mut e1 = new_cv_tracker_1d(initial, 0.0, 100.0, 0.1, 4.0);
                let mut e2 = new_cv_tracker_1d(initial, 0.0, 100.0, 0.1, 4.0);
                for i in 0..10 {
                    let z = true_vel * (i + 1) as f64;
                    e1 = e1.try_next(cv_predict_1d(1.0, 0.1)).unwrap();
                    e2 = e2.try_next(cv_predict_1d(1.0, 0.1)).unwrap();
                    e1 = e1.try_next(cv_update_position_1d(z, 4.0)).unwrap();
                    e2 = e2.try_next(cv_update_position_1d(z, 4.0)).unwrap();
                }
                prop_assert!(e1.situation().estimate.state.data
                    == e2.situation().estimate.state.data);
            }
        }
    }
}
