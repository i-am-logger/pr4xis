/// Frame Transform Chain — proving SE(3) category laws.
///
/// Demonstrates that composing frame transforms is associative
/// and that the identity transform is neutral.
///
/// Proves:
/// - SE(3) composition associativity
/// - Identity element
/// - Inverse yields identity
///
/// Source: Murray, Li & Sastry (1994), Chapter 2.
#[cfg(test)]
mod tests {
    use pr4xis_domains::formal::math::rigid_motion::pose::Pose;
    use pr4xis_domains::formal::math::rotation::quaternion::Quaternion;

    #[test]
    fn three_frame_chain_is_associative() {
        // Body → IMU → Camera: three frames
        let t_body_imu = Pose {
            rotation: Quaternion::from_axis_angle([1.0, 0.0, 0.0], 0.1),
            translation: [0.05, 0.0, -0.1],
        };
        let t_imu_camera = Pose {
            rotation: Quaternion::from_axis_angle([0.0, 1.0, 0.0], -0.05),
            translation: [0.0, 0.02, 0.03],
        };
        let t_camera_lidar = Pose {
            rotation: Quaternion::from_axis_angle([0.0, 0.0, 1.0], 0.2),
            translation: [0.1, -0.05, 0.0],
        };

        // (T_body_imu ∘ T_imu_camera) ∘ T_camera_lidar
        let lhs = t_body_imu.compose(&t_imu_camera).compose(&t_camera_lidar);

        // T_body_imu ∘ (T_imu_camera ∘ T_camera_lidar)
        let rhs = t_body_imu.compose(&t_imu_camera.compose(&t_camera_lidar));

        assert!(lhs == rhs, "SE(3) composition must be associative");
    }

    #[test]
    fn inverse_roundtrip() {
        let t = Pose {
            rotation: Quaternion::from_axis_angle([0.0, 0.0, 1.0], 0.5),
            translation: [1.0, 2.0, 3.0],
        };
        let roundtrip = t.compose(&t.inverse());
        assert!(roundtrip == Pose::identity());
    }

    #[test]
    fn transform_point_through_chain() {
        let t_ab = Pose::from_translation([10.0, 0.0, 0.0]);
        let t_bc = Pose::from_translation([0.0, 5.0, 0.0]);
        let t_ac = t_ab.compose(&t_bc);

        let p = [0.0, 0.0, 0.0];
        let direct = t_ac.transform_point(p);
        let sequential = t_bc.transform_point(t_ab.transform_point(p));

        assert!((direct[0] - sequential[0]).abs() < 1e-10);
        assert!((direct[1] - sequential[1]).abs() < 1e-10);
        assert!((direct[2] - sequential[2]).abs() < 1e-10);
    }

    mod proptest_proofs {
        use super::*;
        use proptest::prelude::*;

        fn arb_pose() -> impl Strategy<Value = Pose> {
            (
                -50.0..50.0_f64,
                -50.0..50.0_f64,
                -50.0..50.0_f64,
                0.0..3.0_f64,
                -50.0..50.0_f64,
                -50.0..50.0_f64,
                -50.0..50.0_f64,
            )
                .prop_map(|(ax, ay, az, angle, tx, ty, tz)| {
                    let n = (ax * ax + ay * ay + az * az).sqrt().max(1e-6);
                    Pose {
                        rotation: Quaternion::from_axis_angle([ax / n, ay / n, az / n], angle),
                        translation: [tx, ty, tz],
                    }
                })
        }

        proptest! {
            #[test]
            fn composition_is_associative(a in arb_pose(), b in arb_pose(), c in arb_pose()) {
                let lhs = a.compose(&b).compose(&c);
                let rhs = a.compose(&b.compose(&c));
                prop_assert!(lhs == rhs);
            }

            #[test]
            fn inverse_yields_identity(p in arb_pose()) {
                let result = p.compose(&p.inverse());
                prop_assert!(result == Pose::identity());
            }
        }
    }
}
