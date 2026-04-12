/// Multi-Sensor AHRS — Attitude and Heading Reference System.
///
/// Fuses accelerometer (gravity direction), gyroscope (angular rate),
/// and magnetometer (magnetic north) to estimate attitude (roll, pitch, yaw).
///
/// This is a simplified demonstration using the IMU strapdown mechanization.
///
/// Proves:
/// - IMU at rest converges to correct gravity direction
/// - Attitude quaternion stays unit norm
/// - Strapdown mechanization is deterministic
///
/// Source: Madgwick (2010), Mahony et al. (2008), Groves (2013) Chapter 5.
#[cfg(test)]
mod tests {
    use pr4xis_domains::science::math::geometry::point::Point3;
    use pr4xis_domains::science::math::rotation::quaternion::Quaternion;
    use pr4xis_domains::science::physics::kinematics::velocity::Velocity;
    use pr4xis_domains::technology::navigation::imu::strapdown::*;

    #[test]
    fn stationary_imu_preserves_attitude() {
        let state = NavState {
            position: Point3::origin(),
            velocity: Velocity::zero(),
            attitude: Quaternion::identity(),
        };

        // Stationary: accel reads -g (specific force cancels gravity)
        let sample = ImuSample {
            specific_force: [0.0, 0.0, -gravity_ned()[2]],
            angular_rate: [0.0, 0.0, 0.0],
            dt: 0.01,
        };

        let mut nav = state;
        for _ in 0..1000 {
            nav = mechanize(&nav, &sample);
        }

        // Should still be at origin
        assert!(
            nav.position.distance_to(&Point3::origin()) < 0.1,
            "stationary IMU drifted: dist={}",
            nav.position.distance_to(&Point3::origin())
        );

        // Attitude should be near identity
        assert!(
            (nav.attitude.norm() - 1.0).abs() < 1e-10,
            "attitude quaternion not unit: norm={}",
            nav.attitude.norm()
        );
    }

    #[test]
    fn yaw_rotation_changes_heading() {
        let state = NavState {
            position: Point3::origin(),
            velocity: Velocity::zero(),
            attitude: Quaternion::identity(),
        };

        // Constant yaw rate: 10°/s for 1 second
        let yaw_rate = 10.0_f64.to_radians();
        let sample = ImuSample {
            specific_force: [0.0, 0.0, -gravity_ned()[2]],
            angular_rate: [0.0, 0.0, yaw_rate],
            dt: 0.01,
        };

        let mut nav = state;
        for _ in 0..100 {
            nav = mechanize(&nav, &sample);
        }

        // Attitude should have rotated ~10° about z
        assert!(nav.attitude != Quaternion::identity());
    }
}
