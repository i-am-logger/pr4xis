use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
use pr4xis::ontology::{Axiom, Ontology};

use crate::technology::navigation::odometry::engine::*;
use crate::technology::navigation::odometry::ontology::*;

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

#[test]
fn odometry_source_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<OdometrySourceTaxonomy>>().unwrap();
}

#[test]
fn odometry_state_taxonomy_category_laws() {
    check_category_laws::<TaxonomyCategory<OdometryStateTaxonomy>>().unwrap();
}

#[test]
fn odometry_ontology_validates() {
    OdometryOntology::validate().unwrap();
}

#[test]
fn odometry_source_taxonomy_is_dag() {
    assert!(OdometrySourceTaxonomyIsDAG.holds());
}

#[test]
fn odometry_state_taxonomy_is_dag() {
    assert!(OdometryStateTaxonomyIsDAG.holds());
}

#[test]
fn drift_is_unbounded_axiom() {
    assert!(DriftIsUnbounded.holds());
}

#[test]
fn relative_motion_only_axiom() {
    assert!(RelativeMotionOnly.holds());
}

#[test]
fn slip_corrupts_wheel_odometry_axiom() {
    assert!(SlipCorruptsWheelOdometry.holds());
}

// ---------------------------------------------------------------------------
// Engine tests
// ---------------------------------------------------------------------------

#[test]
fn stationary_robot_stays_put() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: 0.0,
        distance_traveled: 0.0,
        estimated_error: 0.0,
        drift_rate: 0.02,
        step: 0,
    };
    let next = apply_odometry(
        &sit,
        &OdometryAction::DriveForward {
            velocity: 0.0,
            heading_rate: 0.0,
            dt: 1.0,
        },
    )
    .unwrap();
    assert!((next.pose.x).abs() < 1e-10);
    assert!((next.pose.y).abs() < 1e-10);
}

#[test]
fn drive_forward_moves_in_heading_direction() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: 0.0,
        distance_traveled: 0.0,
        estimated_error: 0.0,
        drift_rate: 0.02,
        step: 0,
    };
    let next = apply_odometry(
        &sit,
        &OdometryAction::DriveForward {
            velocity: 1.0,
            heading_rate: 0.0,
            dt: 10.0,
        },
    )
    .unwrap();
    // Heading 0 means forward along x
    assert!((next.pose.x - 10.0).abs() < 0.01, "x = {}", next.pose.x);
    assert!(next.pose.y.abs() < 0.01, "y = {}", next.pose.y);
    assert!((next.distance_traveled - 10.0).abs() < 0.01);
}

#[test]
fn wheel_tick_straight_line() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: 0.0,
        distance_traveled: 0.0,
        estimated_error: 0.0,
        drift_rate: 0.02,
        step: 0,
    };
    // Both wheels move same distance = straight line
    let next = apply_odometry(
        &sit,
        &OdometryAction::WheelTick {
            left: 1.0,
            right: 1.0,
            wheel_base: 0.5,
        },
    )
    .unwrap();
    assert!((next.pose.x - 1.0).abs() < 0.01, "x = {}", next.pose.x);
    assert!(next.pose.y.abs() < 0.01, "y = {}", next.pose.y);
    assert!(
        next.pose.heading.abs() < 0.01,
        "heading = {}",
        next.pose.heading
    );
}

#[test]
fn wheel_tick_turn_in_place() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: 0.0,
        distance_traveled: 0.0,
        estimated_error: 0.0,
        drift_rate: 0.02,
        step: 0,
    };
    // Wheels move equal and opposite = turn in place
    let wheel_base = 0.5;
    let next = apply_odometry(
        &sit,
        &OdometryAction::WheelTick {
            left: -0.25,
            right: 0.25,
            wheel_base,
        },
    )
    .unwrap();
    // Should rotate but not translate much
    let expected_dtheta = 0.5 / wheel_base; // 1.0 rad
    assert!(
        (next.pose.heading - expected_dtheta).abs() < 0.1,
        "heading = {}",
        next.pose.heading
    );
}

#[test]
fn error_grows_with_distance() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: 0.0,
        distance_traveled: 0.0,
        estimated_error: 0.0,
        drift_rate: 0.02,
        step: 0,
    };
    let next = apply_odometry(
        &sit,
        &OdometryAction::DriveForward {
            velocity: 1.0,
            heading_rate: 0.0,
            dt: 100.0,
        },
    )
    .unwrap();
    assert!(
        next.estimated_error > 0.0,
        "error should grow: {}",
        next.estimated_error
    );
    assert!(
        (next.estimated_error - 0.02 * 100.0).abs() < 0.01,
        "error should be ~2.0m: {}",
        next.estimated_error
    );
}

#[test]
fn negative_dt_rejected() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: 0.0,
        distance_traveled: 0.0,
        estimated_error: 0.0,
        drift_rate: 0.02,
        step: 0,
    };
    let result = apply_odometry(
        &sit,
        &OdometryAction::DriveForward {
            velocity: 1.0,
            heading_rate: 0.0,
            dt: -1.0,
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
        fn distance_traveled_monotonically_increases(
            v in 0.0..10.0_f64,
            dt in 0.01..10.0_f64,
        ) {
            let sit = OdometrySituation {
                pose: OdometryPose::origin(),
                velocity: 0.0,
                distance_traveled: 50.0,
                estimated_error: 1.0,
                drift_rate: 0.02,
                step: 0,
            };
            let next = apply_odometry(&sit, &OdometryAction::DriveForward {
                velocity: v,
                heading_rate: 0.0,
                dt,
            }).unwrap();
            prop_assert!(next.distance_traveled >= sit.distance_traveled,
                "distance should not decrease: {} vs {}",
                next.distance_traveled, sit.distance_traveled);
        }

        #[test]
        fn error_never_decreases(
            v in 0.1..10.0_f64,
            dt in 0.01..10.0_f64,
        ) {
            let sit = OdometrySituation {
                pose: OdometryPose::origin(),
                velocity: 0.0,
                distance_traveled: 10.0,
                estimated_error: 0.2,
                drift_rate: 0.02,
                step: 0,
            };
            let next = apply_odometry(&sit, &OdometryAction::DriveForward {
                velocity: v,
                heading_rate: 0.0,
                dt,
            }).unwrap();
            prop_assert!(next.estimated_error >= sit.estimated_error - 1e-10,
                "error should not decrease: {} vs {}",
                next.estimated_error, sit.estimated_error);
        }

        #[test]
        fn dead_reckoning_is_deterministic(
            v in -5.0..5.0_f64,
            w in -1.0..1.0_f64,
            dt in 0.01..1.0_f64,
        ) {
            let sit = OdometrySituation {
                pose: OdometryPose::new(1.0, 2.0, 0.5),
                velocity: 0.0,
                distance_traveled: 0.0,
                estimated_error: 0.0,
                drift_rate: 0.02,
                step: 0,
            };
            let action = OdometryAction::DriveForward {
                velocity: v,
                heading_rate: w,
                dt,
            };
            let r1 = apply_odometry(&sit, &action).unwrap();
            let r2 = apply_odometry(&sit, &action).unwrap();
            prop_assert!((r1.pose.x - r2.pose.x).abs() < 1e-15);
            prop_assert!((r1.pose.y - r2.pose.y).abs() < 1e-15);
            prop_assert!((r1.pose.heading - r2.pose.heading).abs() < 1e-15);
        }
    }
}
