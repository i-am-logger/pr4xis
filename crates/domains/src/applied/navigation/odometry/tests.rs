use pr4xis::category::laws::assert_category_laws;
use pr4xis::logic::Axiom;
use pr4xis::ontology::Ontology;

use crate::applied::navigation::odometry::engine::*;
use crate::applied::navigation::odometry::ontology::*;
use crate::formal::math::angle::Angle;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::duration::Duration;

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn odometry_category_laws() {
    assert_category_laws::<OdometryCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn odometry_ontology_validates() {
    OdometryOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn drift_is_unbounded_axiom() {
    assert!(DriftIsUnbounded.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn relative_motion_only_axiom() {
    assert!(RelativeMotionOnly.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn slip_corrupts_wheel_odometry_axiom() {
    assert!(SlipCorruptsWheelOdometry.verify().is_ok());
}

// ---------------------------------------------------------------------------
// Engine tests
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn stationary_robot_stays_put() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: Quantity::from_unit(0.0, &unit::METER_PER_SECOND),
        distance_traveled: Quantity::from_unit(0.0, &unit::METER),
        estimated_error: Quantity::from_unit(0.0, &unit::METER),
        drift_rate: 0.02,
        step: 0,
    };
    let next = apply_odometry(
        &sit,
        &OdometryAction::DriveForward {
            velocity: Quantity::from_unit(0.0, &unit::METER_PER_SECOND),
            heading_rate: Quantity::from_unit(0.0, &unit::RADIAN_PER_SECOND),
            dt: Duration::from_seconds(1.0),
        },
    )
    .unwrap();
    assert!((next.pose.position.x).abs() < 1e-10);
    assert!((next.pose.position.y).abs() < 1e-10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn drive_forward_moves_in_heading_direction() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: Quantity::from_unit(0.0, &unit::METER_PER_SECOND),
        distance_traveled: Quantity::from_unit(0.0, &unit::METER),
        estimated_error: Quantity::from_unit(0.0, &unit::METER),
        drift_rate: 0.02,
        step: 0,
    };
    let next = apply_odometry(
        &sit,
        &OdometryAction::DriveForward {
            velocity: Quantity::from_unit(1.0, &unit::METER_PER_SECOND),
            heading_rate: Quantity::from_unit(0.0, &unit::RADIAN_PER_SECOND),
            dt: Duration::from_seconds(10.0),
        },
    )
    .unwrap();
    // Heading 0 means forward along x
    assert!(
        (next.pose.position.x - 10.0).abs() < 0.01,
        "x = {}",
        next.pose.position.x
    );
    assert!(
        next.pose.position.y.abs() < 0.01,
        "y = {}",
        next.pose.position.y
    );
    assert!((next.distance_traveled.value - 10.0).abs() < 0.01);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn wheel_tick_straight_line() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: Quantity::from_unit(0.0, &unit::METER_PER_SECOND),
        distance_traveled: Quantity::from_unit(0.0, &unit::METER),
        estimated_error: Quantity::from_unit(0.0, &unit::METER),
        drift_rate: 0.02,
        step: 0,
    };
    // Both wheels move same distance = straight line
    let next = apply_odometry(
        &sit,
        &OdometryAction::WheelTick {
            left: Quantity::from_unit(1.0, &unit::METER),
            right: Quantity::from_unit(1.0, &unit::METER),
            wheel_base: Quantity::from_unit(0.5, &unit::METER),
        },
    )
    .unwrap();
    assert!(
        (next.pose.position.x - 1.0).abs() < 0.01,
        "x = {}",
        next.pose.position.x
    );
    assert!(
        next.pose.position.y.abs() < 0.01,
        "y = {}",
        next.pose.position.y
    );
    assert!(
        next.pose.heading.radians().abs() < 0.01,
        "heading = {}",
        next.pose.heading.radians()
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn wheel_tick_turn_in_place() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: Quantity::from_unit(0.0, &unit::METER_PER_SECOND),
        distance_traveled: Quantity::from_unit(0.0, &unit::METER),
        estimated_error: Quantity::from_unit(0.0, &unit::METER),
        drift_rate: 0.02,
        step: 0,
    };
    // Wheels move equal and opposite = turn in place
    let wheel_base = 0.5;
    let next = apply_odometry(
        &sit,
        &OdometryAction::WheelTick {
            left: Quantity::from_unit(-0.25, &unit::METER),
            right: Quantity::from_unit(0.25, &unit::METER),
            wheel_base: Quantity::from_unit(wheel_base, &unit::METER),
        },
    )
    .unwrap();
    // Should rotate but not translate much
    let expected_dtheta = 0.5 / wheel_base; // 1.0 rad
    assert!(
        (next.pose.heading.radians() - expected_dtheta).abs() < 0.1,
        "heading = {}",
        next.pose.heading.radians()
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn error_grows_with_distance() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: Quantity::from_unit(0.0, &unit::METER_PER_SECOND),
        distance_traveled: Quantity::from_unit(0.0, &unit::METER),
        estimated_error: Quantity::from_unit(0.0, &unit::METER),
        drift_rate: 0.02,
        step: 0,
    };
    let next = apply_odometry(
        &sit,
        &OdometryAction::DriveForward {
            velocity: Quantity::from_unit(1.0, &unit::METER_PER_SECOND),
            heading_rate: Quantity::from_unit(0.0, &unit::RADIAN_PER_SECOND),
            dt: Duration::from_seconds(100.0),
        },
    )
    .unwrap();
    assert!(
        next.estimated_error.value > 0.0,
        "error should grow: {}",
        next.estimated_error.value
    );
    assert!(
        (next.estimated_error.value - 0.02 * 100.0).abs() < 0.01,
        "error should be ~2.0m: {}",
        next.estimated_error.value
    );
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn negative_dt_rejected() {
    let sit = OdometrySituation {
        pose: OdometryPose::origin(),
        velocity: Quantity::from_unit(0.0, &unit::METER_PER_SECOND),
        distance_traveled: Quantity::from_unit(0.0, &unit::METER),
        estimated_error: Quantity::from_unit(0.0, &unit::METER),
        drift_rate: 0.02,
        step: 0,
    };
    let result = apply_odometry(
        &sit,
        &OdometryAction::DriveForward {
            velocity: Quantity::from_unit(1.0, &unit::METER_PER_SECOND),
            heading_rate: Quantity::from_unit(0.0, &unit::RADIAN_PER_SECOND),
            dt: Duration::from_seconds(-1.0),
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
                velocity: Quantity::from_unit(0.0, &unit::METER_PER_SECOND),
                distance_traveled: Quantity::from_unit(50.0, &unit::METER),
                estimated_error: Quantity::from_unit(1.0, &unit::METER),
                drift_rate: 0.02,
                step: 0,
            };
            let next = apply_odometry(&sit, &OdometryAction::DriveForward {
                velocity: Quantity::from_unit(v, &unit::METER_PER_SECOND),
                heading_rate: Quantity::from_unit(0.0, &unit::RADIAN_PER_SECOND),
                dt: Duration::from_seconds(dt),
            }).unwrap();
            prop_assert!(next.distance_traveled >= sit.distance_traveled,
                "distance should not decrease: {:?} vs {:?}",
                next.distance_traveled, sit.distance_traveled);
        }

        #[test]
        fn error_never_decreases(
            v in 0.1..10.0_f64,
            dt in 0.01..10.0_f64,
        ) {
            let sit = OdometrySituation {
                pose: OdometryPose::origin(),
                velocity: Quantity::from_unit(0.0, &unit::METER_PER_SECOND),
                distance_traveled: Quantity::from_unit(10.0, &unit::METER),
                estimated_error: Quantity::from_unit(0.2, &unit::METER),
                drift_rate: 0.02,
                step: 0,
            };
            let next = apply_odometry(&sit, &OdometryAction::DriveForward {
                velocity: Quantity::from_unit(v, &unit::METER_PER_SECOND),
                heading_rate: Quantity::from_unit(0.0, &unit::RADIAN_PER_SECOND),
                dt: Duration::from_seconds(dt),
            }).unwrap();
            prop_assert!(next.estimated_error.value >= sit.estimated_error.value - 1e-10,
                "error should not decrease: {} vs {}",
                next.estimated_error.value, sit.estimated_error.value);
        }

        #[test]
        fn dead_reckoning_is_deterministic(
            v in -5.0..5.0_f64,
            w in -1.0..1.0_f64,
            dt in 0.01..1.0_f64,
        ) {
            let sit = OdometrySituation {
                pose: OdometryPose::new(1.0, 2.0, Angle::from_radians(0.5)),
                velocity: Quantity::from_unit(0.0, &unit::METER_PER_SECOND),
                distance_traveled: Quantity::from_unit(0.0, &unit::METER),
                estimated_error: Quantity::from_unit(0.0, &unit::METER),
                drift_rate: 0.02,
                step: 0,
            };
            let action = OdometryAction::DriveForward {
                velocity: Quantity::from_unit(v, &unit::METER_PER_SECOND),
                heading_rate: Quantity::from_unit(w, &unit::RADIAN_PER_SECOND),
                dt: Duration::from_seconds(dt),
            };
            let r1 = apply_odometry(&sit, &action).unwrap();
            let r2 = apply_odometry(&sit, &action).unwrap();
            prop_assert!((r1.pose.position.x - r2.pose.position.x).abs() < 1e-15);
            prop_assert!((r1.pose.position.y - r2.pose.position.y).abs() < 1e-15);
            prop_assert!((r1.pose.heading.radians() - r2.pose.heading.radians()).abs() < 1e-15);
        }
    }

    pr4xis::register_praxis_value!(distance_traveled_monotonically_increases, Verifiable);
    pr4xis::register_praxis_value!(error_never_decreases, Verifiable);
    pr4xis::register_praxis_value!(dead_reckoning_is_deterministic, Deterministic);
}
