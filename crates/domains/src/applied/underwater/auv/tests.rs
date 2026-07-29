use pr4xis::category::laws::assert_category_laws;
use pr4xis::logic::Axiom;
use pr4xis::ontology::Ontology;

use crate::applied::underwater::auv::engine::*;
use crate::applied::underwater::auv::ontology::*;
use crate::formal::math::angle::Angle;
use crate::formal::math::geometry::point::Point3;
use crate::formal::math::temporal::duration::Duration;
use crate::natural::physics::kinematics::velocity::Velocity;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn auv_category_laws() {
    assert_category_laws::<AuvCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn auv_ontology_validates() {
    AuvOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn depth_non_negative_holds() {
    assert!(DepthNonNegative.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dvl_requires_bottom_lock_holds() {
    assert!(DvlRequiresBottomLock.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dead_reckoning_straight_north() {
    let state = AuvState {
        position: Point3::new(0.0, 0.0, 10.0),
        heading: Angle::from_radians(0.0),
    };
    let dvl = DvlMeasurement {
        velocity: Velocity::new(1.0, 0.0, 0.0),
        bottom_lock: true,
    };
    let new_state = dead_reckon(
        &state,
        &dvl,
        Angle::from_radians(0.0),
        Duration::from_seconds(10.0),
    );
    assert!((new_state.position.x - 10.0).abs() < 1e-10);
    assert!(new_state.position.y.abs() < 1e-10);
    assert!((new_state.position.z - 10.0).abs() < 1e-10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dead_reckoning_straight_east() {
    let state = AuvState {
        position: Point3::new(0.0, 0.0, 10.0),
        heading: Angle::from_radians(core::f64::consts::FRAC_PI_2), // heading east
    };
    let dvl = DvlMeasurement {
        velocity: Velocity::new(2.0, 0.0, 0.0),
        bottom_lock: true,
    };
    let new_state = dead_reckon(
        &state,
        &dvl,
        Angle::from_radians(core::f64::consts::FRAC_PI_2),
        Duration::from_seconds(5.0),
    );
    assert!(new_state.position.x.abs() < 1e-10);
    assert!((new_state.position.y - 10.0).abs() < 1e-10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn distance_2d_basic() {
    let a = AuvState {
        position: Point3::new(0.0, 0.0, 0.0),
        heading: Angle::from_radians(0.0),
    };
    let b = AuvState {
        position: Point3::new(3.0, 4.0, 0.0),
        heading: Angle::from_radians(0.0),
    };
    assert!((distance_2d(&a, &b).value - 5.0).abs() < 1e-10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn distance_3d_basic() {
    let a = AuvState {
        position: Point3::new(0.0, 0.0, 0.0),
        heading: Angle::from_radians(0.0),
    };
    let b = AuvState {
        position: Point3::new(1.0, 2.0, 2.0),
        heading: Angle::from_radians(0.0),
    };
    assert!((distance_3d(&a, &b).value - 3.0).abs() < 1e-10);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn zero_velocity_preserves_position(
            north in -1000.0..1000.0_f64,
            east in -1000.0..1000.0_f64,
            depth in 0.0..1000.0_f64,
            heading in 0.0..core::f64::consts::TAU,
            dt in 0.1..100.0_f64
        ) {
            let state = AuvState {
                position: Point3::new(north, east, depth),
                heading: Angle::from_radians(heading),
            };
            let dvl = DvlMeasurement {
                velocity: Velocity::new(0.0, 0.0, 0.0), bottom_lock: true,
            };
            let new_state = dead_reckon(
                &state,
                &dvl,
                Angle::from_radians(heading),
                Duration::from_seconds(dt),
            );
            prop_assert!((new_state.position.x - north).abs() < 1e-10);
            prop_assert!((new_state.position.y - east).abs() < 1e-10);
            prop_assert!((new_state.position.z - depth).abs() < 1e-10);
        }

        #[test]
        fn distance_is_non_negative(
            n1 in -100.0..100.0_f64,
            e1 in -100.0..100.0_f64,
            n2 in -100.0..100.0_f64,
            e2 in -100.0..100.0_f64
        ) {
            let a = AuvState {
                position: Point3::new(n1, e1, 0.0),
                heading: Angle::from_radians(0.0),
            };
            let b = AuvState {
                position: Point3::new(n2, e2, 0.0),
                heading: Angle::from_radians(0.0),
            };
            prop_assert!(distance_2d(&a, &b).value >= 0.0);
        }
    }

    pr4xis::register_praxis_value!(zero_velocity_preserves_position, Verifiable);
    pr4xis::register_praxis_value!(distance_is_non_negative, Verifiable);
}
