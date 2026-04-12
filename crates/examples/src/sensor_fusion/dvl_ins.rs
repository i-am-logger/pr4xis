/// DVL-Aided INS — underwater navigation.
///
/// A Doppler Velocity Log measures bottom-track velocity.
/// Combined with IMU, this provides position estimation
/// in GPS-denied underwater environments.
///
/// This demonstrates the fusion engine used for underwater navigation.
///
/// Source: Kinsey et al. (2006), "A survey of underwater vehicle navigation."
///         Groves (2013), Chapter 16.
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
    fn dvl_aided_ins_position_estimation() {
        // State: [x, vx] (1D forward motion)
        // DVL measures velocity directly: H = [0, 1]
        let state = StateEstimate::new(
            Vector::new(vec![0.0, 2.0]), // start at 0, moving at 2 m/s
            Matrix::diagonal(&[100.0, 10.0]),
            0.0,
        );
        let mut engine = new_fusion_engine(state);

        let dt = 1.0;
        let f = Matrix::new(2, 2, vec![1.0, dt, 0.0, 1.0]); // constant velocity
        let q = Matrix::new(2, 2, vec![0.1, 0.0, 0.0, 0.01]);

        // DVL observes velocity (not position!)
        let h_dvl = Matrix::new(1, 2, vec![0.0, 1.0]);
        let r_dvl = Matrix::new(1, 1, vec![0.01]); // DVL is very accurate

        let true_velocity = 2.0;

        for _ in 0..30 {
            engine = engine
                .try_next(FusionAction::Predict {
                    dt,
                    transition: f.clone(),
                    process_noise: q.clone(),
                })
                .unwrap();

            engine = engine
                .try_next(FusionAction::Update {
                    observation_matrix: h_dvl.clone(),
                    measurement: Vector::new(vec![true_velocity]),
                    measurement_noise: r_dvl.clone(),
                })
                .unwrap();
        }

        // Velocity should converge to true value
        let est_vel = engine.situation().estimate.state.get(1);
        assert!(
            (est_vel - true_velocity).abs() < 0.5,
            "velocity: est={:.2}, true={:.2}",
            est_vel,
            true_velocity
        );

        // Position should be approximately 2 m/s * 30 s = 60 m
        let est_pos = engine.situation().estimate.state.get(0);
        assert!(
            (est_pos - 60.0).abs() < 20.0,
            "position: est={:.1}, expected ~60",
            est_pos
        );

        assert!(positive_definite::is_positive_semidefinite(
            &engine.situation().estimate.covariance
        ));
    }
}
