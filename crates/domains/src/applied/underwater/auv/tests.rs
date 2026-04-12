use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::underwater::auv::engine::*;
use crate::applied::underwater::auv::ontology::*;

#[test]
fn auv_category_laws() {
    check_category_laws::<AuvCategory>().unwrap();
}

#[test]
fn auv_ontology_validates() {
    AuvOntology::validate().unwrap();
}

#[test]
fn depth_non_negative_holds() {
    assert!(DepthNonNegative.holds());
}

#[test]
fn dvl_requires_bottom_lock_holds() {
    assert!(DvlRequiresBottomLock.holds());
}

#[test]
fn dead_reckoning_straight_north() {
    let state = AuvState {
        north: 0.0,
        east: 0.0,
        depth: 10.0,
        heading: 0.0,
    };
    let dvl = DvlMeasurement {
        forward: 1.0,
        starboard: 0.0,
        downward: 0.0,
        bottom_lock: true,
    };
    let new_state = dead_reckon(&state, &dvl, 0.0, 10.0);
    assert!((new_state.north - 10.0).abs() < 1e-10);
    assert!(new_state.east.abs() < 1e-10);
    assert!((new_state.depth - 10.0).abs() < 1e-10);
}

#[test]
fn dead_reckoning_straight_east() {
    let state = AuvState {
        north: 0.0,
        east: 0.0,
        depth: 10.0,
        heading: std::f64::consts::FRAC_PI_2, // heading east
    };
    let dvl = DvlMeasurement {
        forward: 2.0,
        starboard: 0.0,
        downward: 0.0,
        bottom_lock: true,
    };
    let new_state = dead_reckon(&state, &dvl, std::f64::consts::FRAC_PI_2, 5.0);
    assert!(new_state.north.abs() < 1e-10);
    assert!((new_state.east - 10.0).abs() < 1e-10);
}

#[test]
fn distance_2d_basic() {
    let a = AuvState {
        north: 0.0,
        east: 0.0,
        depth: 0.0,
        heading: 0.0,
    };
    let b = AuvState {
        north: 3.0,
        east: 4.0,
        depth: 0.0,
        heading: 0.0,
    };
    assert!((distance_2d(&a, &b) - 5.0).abs() < 1e-10);
}

#[test]
fn distance_3d_basic() {
    let a = AuvState {
        north: 0.0,
        east: 0.0,
        depth: 0.0,
        heading: 0.0,
    };
    let b = AuvState {
        north: 1.0,
        east: 2.0,
        depth: 2.0,
        heading: 0.0,
    };
    assert!((distance_3d(&a, &b) - 3.0).abs() < 1e-10);
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
            heading in 0.0..6.28_f64,
            dt in 0.1..100.0_f64
        ) {
            let state = AuvState { north, east, depth, heading };
            let dvl = DvlMeasurement {
                forward: 0.0, starboard: 0.0, downward: 0.0, bottom_lock: true,
            };
            let new_state = dead_reckon(&state, &dvl, heading, dt);
            prop_assert!((new_state.north - north).abs() < 1e-10);
            prop_assert!((new_state.east - east).abs() < 1e-10);
            prop_assert!((new_state.depth - depth).abs() < 1e-10);
        }

        #[test]
        fn distance_is_non_negative(
            n1 in -100.0..100.0_f64,
            e1 in -100.0..100.0_f64,
            n2 in -100.0..100.0_f64,
            e2 in -100.0..100.0_f64
        ) {
            let a = AuvState { north: n1, east: e1, depth: 0.0, heading: 0.0 };
            let b = AuvState { north: n2, east: e2, depth: 0.0, heading: 0.0 };
            prop_assert!(distance_2d(&a, &b) >= 0.0);
        }
    }
}
