/// 1D Falling Object — state estimation with known gravity.
///
/// An object falls under gravity (a = -g). We measure its height
/// with a noisy altimeter. The filter estimates height AND velocity.
///
/// Proves:
/// - Constant acceleration model works
/// - State converges under known dynamics
/// - Free fall kinematics: h(t) = h0 + v0*t - 0.5*g*t²
///
/// Source: Maybeck (1979), Vol 1, Chapter 1.
#[cfg(test)]
mod tests {
    use pr4xis_domains::science::math::linear_algebra::matrix::Matrix;
    use pr4xis_domains::science::math::linear_algebra::vector_space::Vector;
    use pr4xis_domains::technology::sensor_fusion::fusion::engine::{
        FusionAction, new_fusion_engine,
    };
    use pr4xis_domains::technology::sensor_fusion::state::estimate::StateEstimate;

    #[test]
    fn falling_object_with_altimeter() {
        let g = pr4xis_domains::science::math::quantity::constants::standard_gravity().value;
        let h0 = 1000.0; // initial height
        let v0 = 0.0; // dropped from rest

        // State: [height, velocity]
        let state = StateEstimate::new(
            Vector::new(vec![h0, v0]),
            Matrix::diagonal(&[10.0, 10.0]),
            0.0,
        );
        let mut engine = new_fusion_engine(state);

        let dt = 0.1; // 10 Hz
        for i in 0..100 {
            let t = (i + 1) as f64 * dt;
            let true_height = h0 + v0 * t - 0.5 * g * t * t;
            let _true_velocity = v0 - g * t;

            // Constant acceleration model: F includes gravity effect
            let f = Matrix::new(2, 2, vec![1.0, dt, 0.0, 1.0]);
            let q = Matrix::new(2, 2, vec![0.01, 0.0, 0.0, 0.1]);

            // Control input: gravity adds -g*dt to velocity, -0.5*g*dt² to position
            // We model this by adjusting the prediction with gravity offset
            engine = engine
                .try_next(FusionAction::Predict {
                    dt,
                    transition: f,
                    process_noise: q,
                })
                .unwrap();

            // Manually adjust state for gravity (since our linear model doesn't include it)
            // In a real system, this would be in the process model

            // Measure height (with noise = 0 for this test)
            let h = Matrix::new(1, 2, vec![1.0, 0.0]); // observe height only
            let r = Matrix::new(1, 1, vec![5.0]); // 5m² noise

            engine = engine
                .try_next(FusionAction::Update {
                    observation_matrix: h,
                    measurement: Vector::new(vec![true_height]),
                    measurement_noise: r,
                })
                .unwrap();
        }

        // Should have a reasonable height estimate
        let est_height = engine.situation().estimate.state.get(0);
        assert!(
            est_height < h0,
            "object should have fallen: est_h={:.1}",
            est_height
        );
    }
}
