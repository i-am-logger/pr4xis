use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::navigation::ahrs::engine::*;
use crate::applied::navigation::ahrs::ontology::*;

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

#[test]
fn ahrs_filter_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<AhrsFilterTaxonomy>>().unwrap();
}

#[test]
fn attitude_state_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<AttitudeStateTaxonomy>>().unwrap();
}

#[test]
fn ahrs_ontology_validates() {
    AhrsOntology::validate().unwrap();
}

#[test]
fn attitude_state_taxonomy_is_dag() {
    assert!(AttitudeStateTaxonomyIsDAG.holds());
}

#[test]
fn gravity_gives_level_attitude_axiom() {
    assert!(GravityGivesLevelAttitude.holds());
}

#[test]
fn magnetometer_gives_heading_axiom() {
    assert!(MagnetometerGivesHeading.holds());
}

#[test]
fn gyro_integration_drifts_axiom() {
    assert!(GyroIntegrationDrifts.holds());
}

// ---------------------------------------------------------------------------
// Engine tests
// ---------------------------------------------------------------------------

#[test]
fn zero_gyro_preserves_attitude() {
    let sit = AhrsSituation {
        attitude: AttitudeEstimate::new(0.1, 0.2, 0.3),
        alpha: 0.98,
        step: 0,
        total_time: 0.0,
    };
    let next = apply_ahrs(
        &sit,
        &AhrsAction::GyroUpdate {
            angular_rate: [0.0, 0.0, 0.0],
            dt: 0.01,
        },
    )
    .unwrap();
    // With zero angular rate, attitude should not change much
    // (small drift toward zero from (1-alpha) blending)
    assert!((next.attitude.roll - sit.attitude.roll).abs() < 0.01);
    assert!((next.attitude.pitch - sit.attitude.pitch).abs() < 0.01);
    assert!((next.attitude.yaw - sit.attitude.yaw).abs() < 0.01);
}

#[test]
fn accel_at_rest_gives_level() {
    let sit = AhrsSituation {
        attitude: AttitudeEstimate::new(0.5, 0.5, 0.0), // start tilted
        alpha: 0.0,                                     // trust accel completely
        step: 0,
        total_time: 0.0,
    };
    let g = 9.80665;
    let next = apply_ahrs(
        &sit,
        &AhrsAction::AccelCorrection {
            accel: [0.0, 0.0, -g], // level frame, accel reads [0, 0, -g]
        },
    )
    .unwrap();
    // With alpha=0, should snap to accel-derived attitude (level)
    assert!(
        next.attitude.roll.abs() < 0.01,
        "roll should be ~0, got {}",
        next.attitude.roll
    );
    assert!(
        next.attitude.pitch.abs() < 0.01,
        "pitch should be ~0, got {}",
        next.attitude.pitch
    );
}

#[test]
fn mag_north_gives_zero_heading() {
    let sit = AhrsSituation {
        attitude: AttitudeEstimate::new(0.0, 0.0, 1.0), // start with non-zero yaw
        alpha: 0.0,                                     // trust mag completely
        step: 0,
        total_time: 0.0,
    };
    let next = apply_ahrs(
        &sit,
        &AhrsAction::MagCorrection {
            mag: [20.0e-6, 0.0, -40.0e-6], // horizontal field pointing north
        },
    )
    .unwrap();
    // With alpha=0, should snap to mag-derived heading (north = 0)
    assert!(
        next.attitude.yaw.abs() < 0.01,
        "yaw should be ~0, got {}",
        next.attitude.yaw
    );
}

#[test]
fn negative_dt_rejected() {
    let sit = AhrsSituation {
        attitude: AttitudeEstimate::zero(),
        alpha: 0.98,
        step: 0,
        total_time: 0.0,
    };
    let result = apply_ahrs(
        &sit,
        &AhrsAction::GyroUpdate {
            angular_rate: [0.0, 0.0, 0.1],
            dt: -0.01,
        },
    );
    assert!(result.is_err());
}

#[test]
fn zero_accel_rejected() {
    let sit = AhrsSituation {
        attitude: AttitudeEstimate::zero(),
        alpha: 0.98,
        step: 0,
        total_time: 0.0,
    };
    let result = apply_ahrs(
        &sit,
        &AhrsAction::AccelCorrection {
            accel: [0.0, 0.0, 0.0],
        },
    );
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Proptest
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn gyro_update_is_deterministic(
            roll in -3.14..3.14_f64,
            pitch in -1.5..1.5_f64,
            yaw in -3.14..3.14_f64,
            wx in -1.0..1.0_f64,
            wy in -1.0..1.0_f64,
            wz in -1.0..1.0_f64,
            dt in 0.001..0.1_f64,
        ) {
            let sit = AhrsSituation {
                attitude: AttitudeEstimate::new(roll, pitch, yaw),
                alpha: 0.98,
                step: 0,
                total_time: 0.0,
            };
            let action = AhrsAction::GyroUpdate {
                angular_rate: [wx, wy, wz],
                dt,
            };
            let r1 = apply_ahrs(&sit, &action).unwrap();
            let r2 = apply_ahrs(&sit, &action).unwrap();
            prop_assert!((r1.attitude.roll - r2.attitude.roll).abs() < 1e-15);
            prop_assert!((r1.attitude.pitch - r2.attitude.pitch).abs() < 1e-15);
            prop_assert!((r1.attitude.yaw - r2.attitude.yaw).abs() < 1e-15);
        }

        #[test]
        fn time_monotonically_increases(
            dt in 0.001..0.1_f64,
        ) {
            let sit = AhrsSituation {
                attitude: AttitudeEstimate::zero(),
                alpha: 0.98,
                step: 0,
                total_time: 0.0,
            };
            let next = apply_ahrs(&sit, &AhrsAction::GyroUpdate {
                angular_rate: [0.0, 0.0, 0.1],
                dt,
            }).unwrap();
            prop_assert!(next.total_time > sit.total_time);
            prop_assert_eq!(next.step, sit.step + 1);
        }

        /// C3: gyro update must NOT double-blend — pure integration only.
        /// With alpha < 1, the old code would decay toward zero; the fix keeps pure integration.
        #[test]
        fn gyro_update_pure_integration_no_alpha_blend(
            roll in -3.14..3.14_f64,
            pitch in -1.5..1.5_f64,
            yaw in -3.14..3.14_f64,
            wx in -1.0..1.0_f64,
            wy in -1.0..1.0_f64,
            wz in -1.0..1.0_f64,
            dt in 0.001..0.1_f64,
            alpha in 0.5..0.99_f64,
        ) {
            let sit = AhrsSituation {
                attitude: AttitudeEstimate::new(roll, pitch, yaw),
                alpha,
                step: 0,
                total_time: 0.0,
            };
            let next = apply_ahrs(&sit, &AhrsAction::GyroUpdate {
                angular_rate: [wx, wy, wz],
                dt,
            }).unwrap();

            // Pure integration: new = old + rate*dt
            let expected_roll = roll + wx * dt;
            let expected_pitch = pitch + wy * dt;
            let expected_yaw = yaw + wz * dt;

            prop_assert!(
                (next.attitude.roll - expected_roll).abs() < 1e-12,
                "gyro roll: got {} expected {}", next.attitude.roll, expected_roll
            );
            prop_assert!(
                (next.attitude.pitch - expected_pitch).abs() < 1e-12,
                "gyro pitch: got {} expected {}", next.attitude.pitch, expected_pitch
            );
            prop_assert!(
                (next.attitude.yaw - expected_yaw).abs() < 1e-12,
                "gyro yaw: got {} expected {}", next.attitude.yaw, expected_yaw
            );
        }

        #[test]
        fn accel_correction_does_not_change_yaw(
            yaw in -3.14..3.14_f64,
            ax in -2.0..2.0_f64,
            ay in -2.0..2.0_f64,
        ) {
            let g = 9.80665;
            let sit = AhrsSituation {
                attitude: AttitudeEstimate::new(0.0, 0.0, yaw),
                alpha: 0.5,
                step: 0,
                total_time: 0.0,
            };
            let result = apply_ahrs(&sit, &AhrsAction::AccelCorrection {
                accel: [ax, ay, -g],
            });
            if let Ok(next) = result {
                // Accel correction should only affect roll and pitch, not yaw
                prop_assert!((next.attitude.yaw - sit.attitude.yaw).abs() < 1e-12,
                    "yaw changed from {} to {}", sit.attitude.yaw, next.attitude.yaw);
            }
        }
    }
}
