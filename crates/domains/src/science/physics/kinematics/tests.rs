use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::science::physics::kinematics::ontology::*;

#[test]
fn kinematics_category_laws() {
    check_category_laws::<KinematicsCategory>().unwrap();
}

#[test]
fn kinematics_ontology_validates() {
    KinematicsOntology::validate().unwrap();
}

#[test]
fn velocity_is_derivative_of_position() {
    assert!(VelocityIsDerivativeOfPosition.holds());
}

#[test]
fn acceleration_is_derivative_of_velocity() {
    assert!(AccelerationIsDerivativeOfVelocity.holds());
}

#[test]
fn constant_velocity_propagation() {
    assert!(ConstantVelocityPropagation.holds());
}

#[test]
fn constant_acceleration_propagation() {
    assert!(ConstantAccelerationPropagation.holds());
}

#[test]
fn velocity_update_under_acceleration() {
    assert!(VelocityUpdateUnderAcceleration.holds());
}

#[test]
fn static_model_invariance() {
    assert!(StaticModelInvariance.holds());
}

#[test]
fn speed_non_negative() {
    assert!(SpeedNonNegative.holds());
}

#[test]
fn velocity_addition_commutative() {
    assert!(VelocityAdditionCommutative.holds());
}

#[cfg(test)]
mod proptest_proofs {
    use crate::science::math::geometry::point::Point3;
    use crate::science::physics::kinematics::acceleration::Acceleration;
    use crate::science::physics::kinematics::motion_model::{self, MotionModelType};
    use crate::science::physics::kinematics::trajectory::KinematicState;
    use crate::science::physics::kinematics::velocity::Velocity;
    use proptest::prelude::*;

    fn arb_velocity() -> impl Strategy<Value = Velocity> {
        (-100.0..100.0_f64, -100.0..100.0_f64, -100.0..100.0_f64)
            .prop_map(|(vx, vy, vz)| Velocity::new(vx, vy, vz))
    }

    fn arb_acceleration() -> impl Strategy<Value = Acceleration> {
        (-10.0..10.0_f64, -10.0..10.0_f64, -10.0..10.0_f64)
            .prop_map(|(ax, ay, az)| Acceleration::new(ax, ay, az))
    }

    fn arb_state() -> impl Strategy<Value = KinematicState> {
        (
            -100.0..100.0_f64,
            -100.0..100.0_f64,
            -100.0..100.0_f64,
            arb_velocity(),
            arb_acceleration(),
        )
            .prop_map(|(px, py, pz, v, a)| KinematicState {
                position: Point3::new(px, py, pz),
                velocity: v,
                acceleration: a,
            })
    }

    proptest! {
        #[test]
        fn speed_is_non_negative(v in arb_velocity()) {
            prop_assert!(v.speed() >= 0.0);
        }

        #[test]
        fn velocity_addition_is_commutative(v1 in arb_velocity(), v2 in arb_velocity()) {
            let a = v1.add(&v2);
            let b = v2.add(&v1);
            prop_assert!((a.vx - b.vx).abs() < 1e-10);
            prop_assert!((a.vy - b.vy).abs() < 1e-10);
            prop_assert!((a.vz - b.vz).abs() < 1e-10);
        }

        #[test]
        fn static_model_preserves_position(
            state in arb_state(),
            dt in 0.1..100.0_f64,
        ) {
            let next = motion_model::propagate(&state, dt, MotionModelType::Static);
            prop_assert!(next.position == state.position);
        }

        #[test]
        fn constant_velocity_displacement_is_linear(
            v in arb_velocity(),
            dt in 0.01..10.0_f64,
        ) {
            let state = KinematicState {
                position: Point3::origin(),
                velocity: v.clone(),
                acceleration: Acceleration::zero(),
            };
            let next = motion_model::propagate(&state, dt, MotionModelType::ConstantVelocity);
            prop_assert!((next.position.x - v.vx * dt).abs() < 1e-8);
            prop_assert!((next.position.y - v.vy * dt).abs() < 1e-8);
            prop_assert!((next.position.z - v.vz * dt).abs() < 1e-8);
        }

        #[test]
        fn constant_velocity_preserves_speed(
            state in arb_state(),
            dt in 0.01..10.0_f64,
        ) {
            let next = motion_model::propagate(&state, dt, MotionModelType::ConstantVelocity);
            prop_assert!((next.velocity.speed() - state.velocity.speed()).abs() < 1e-10);
        }

        #[test]
        fn acceleration_changes_velocity_linearly(
            a in arb_acceleration(),
            dt in 0.01..10.0_f64,
        ) {
            let state = KinematicState {
                position: Point3::origin(),
                velocity: Velocity::zero(),
                acceleration: a.clone(),
            };
            let next = state.propagate(dt);
            // v(dt) = a * dt
            prop_assert!((next.velocity.vx - a.ax * dt).abs() < 1e-8);
            prop_assert!((next.velocity.vy - a.ay * dt).abs() < 1e-8);
            prop_assert!((next.velocity.vz - a.az * dt).abs() < 1e-8);
        }

        #[test]
        fn free_fall_position(dt in 0.01..5.0_f64) {
            // Starting at rest, falling under gravity
            let state = KinematicState {
                position: Point3::new(0.0, 0.0, 1000.0),
                velocity: Velocity::zero(),
                acceleration: Acceleration::gravity(),
            };
            let next = state.propagate(dt);
            let g = crate::science::math::quantity::constants::standard_gravity().value;
            let expected_z = 1000.0 + 0.5 * (-g) * dt * dt;
            prop_assert!((next.position.z - expected_z).abs() < 1e-6,
                "z={}, expected={}", next.position.z, expected_z);
        }

        #[test]
        fn propagate_zero_dt_is_identity(state in arb_state()) {
            let next = state.propagate(0.0);
            prop_assert!((next.position.x - state.position.x).abs() < 1e-12);
            prop_assert!((next.position.y - state.position.y).abs() < 1e-12);
            prop_assert!((next.position.z - state.position.z).abs() < 1e-12);
            prop_assert!((next.velocity.vx - state.velocity.vx).abs() < 1e-12);
            prop_assert!((next.velocity.vy - state.velocity.vy).abs() < 1e-12);
            prop_assert!((next.velocity.vz - state.velocity.vz).abs() < 1e-12);
        }
    }
}
