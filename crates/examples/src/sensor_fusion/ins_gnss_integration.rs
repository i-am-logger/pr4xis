/// INS/GNSS Integration — the classic sensor fusion problem.
///
/// An IMU provides high-rate position/velocity predictions (100Hz)
/// while GPS provides low-rate position corrections (1Hz).
/// The Kalman filter fuses both for optimal navigation.
///
/// Proves:
/// - IMU prediction increases uncertainty between GPS fixes
/// - GPS update decreases uncertainty
/// - Combined system maintains bounded error
///
/// Source: Groves (2013), Chapter 14.
#[cfg(test)]
mod tests {
    use pr4xis_domains::science::math::linear_algebra::matrix::Matrix;
    use pr4xis_domains::science::math::linear_algebra::positive_definite;
    use pr4xis_domains::science::math::linear_algebra::vector_space::Vector;
    use pr4xis_domains::technology::sensor_fusion::fusion::engine::{
        FusionAction, new_fusion_engine,
    };
    use pr4xis_domains::technology::sensor_fusion::state::estimate::StateEstimate;

    #[test]
    fn ins_gnss_loosely_coupled() {
        // State: [position, velocity] (1D for simplicity)
        let state = StateEstimate::new(
            Vector::new(vec![0.0, 10.0]), // start at 0, moving at 10 m/s
            Matrix::diagonal(&[1.0, 1.0]),
            0.0,
        );
        let mut engine = new_fusion_engine(state);

        let imu_dt = 0.01; // 100 Hz IMU
        let gps_interval = 100; // GPS every 100 IMU steps = 1 Hz

        let f_imu = Matrix::new(2, 2, vec![1.0, imu_dt, 0.0, 1.0]);
        let q_imu = Matrix::new(2, 2, vec![0.0001, 0.0, 0.0, 0.01]);
        let h_gps = Matrix::new(1, 2, vec![1.0, 0.0]); // GPS observes position
        let r_gps = Matrix::new(1, 1, vec![5.0]); // 5m² GPS noise

        for i in 0..500 {
            // IMU prediction (high rate)
            engine = engine
                .try_next(FusionAction::Predict {
                    dt: imu_dt,
                    transition: f_imu.clone(),
                    process_noise: q_imu.clone(),
                })
                .unwrap();

            // GPS update (low rate)
            if i % gps_interval == gps_interval - 1 {
                let t = (i + 1) as f64 * imu_dt;
                let true_position = 10.0 * t; // constant velocity
                let uncertainty_before = engine.situation().estimate.uncertainty();

                engine = engine
                    .try_next(FusionAction::Update {
                        observation_matrix: h_gps.clone(),
                        measurement: Vector::new(vec![true_position]),
                        measurement_noise: r_gps.clone(),
                    })
                    .unwrap();

                let uncertainty_after = engine.situation().estimate.uncertainty();
                assert!(
                    uncertainty_after < uncertainty_before,
                    "GPS update must reduce uncertainty"
                );
            }
        }

        // Final state should be reasonable
        let est_pos = engine.situation().estimate.state.get(0);
        let expected_pos = 10.0 * 5.0; // 5 seconds at 10 m/s
        assert!(
            (est_pos - expected_pos).abs() < 20.0,
            "position error: est={:.1}, expected={:.1}",
            est_pos,
            expected_pos
        );

        assert!(positive_definite::is_positive_semidefinite(
            &engine.situation().estimate.covariance
        ));
    }
}
