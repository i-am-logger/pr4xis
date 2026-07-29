#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::laws::assert_category_laws;
use pr4xis::logic::Axiom;
use pr4xis::ontology::Ontology;

use crate::applied::space::attitude::engine::*;
use crate::applied::space::attitude::kinematics::*;
use crate::applied::space::attitude::ontology::*;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::natural::physics::kinematics::angular_velocity::AngularVelocity;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn attitude_category_laws() {
    assert_category_laws::<AttitudeCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn attitude_ontology_validates() {
    AttitudeOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn quaternion_unit_norm_holds() {
    assert!(QuaternionUnitNorm.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn star_tracker_most_accurate_holds() {
    assert!(StarTrackerMostAccurate.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn identity_quaternion_has_unit_norm() {
    let q = Quaternion::identity();
    assert!((q.norm().value - 1.0).abs() < 1e-12);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn quaternion_multiplication_associative() {
    let q1 = Quaternion::new(1.0, 0.1, 0.2, 0.3);
    let q2 = Quaternion::new(0.5, 0.3, 0.1, 0.2);
    let q3 = Quaternion::new(0.7, 0.2, 0.4, 0.1);

    let left = q1.multiply(&q2).multiply(&q3);
    let right = q1.multiply(&q2.multiply(&q3));
    assert!((left.w() - right.w()).abs() < 1e-10);
    assert!((left.x() - right.x()).abs() < 1e-10);
    assert!((left.y() - right.y()).abs() < 1e-10);
    assert!((left.z() - right.z()).abs() < 1e-10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn quaternion_conjugate_gives_identity() {
    let q = Quaternion::new(1.0, 0.1, 0.2, 0.3);
    let result = q.multiply(&q.conjugate());
    assert!((result.w() - 1.0).abs() < 1e-10);
    assert!(result.x().abs() < 1e-10);
    assert!(result.y().abs() < 1e-10);
    assert!(result.z().abs() < 1e-10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn zero_angular_velocity_preserves_attitude() {
    let q = Quaternion::new(1.0, 0.1, 0.2, 0.3);
    let omega = Vector::new(vec![0.0, 0.0, 0.0]);
    let q_new = propagate_attitude(&q, &omega, 1.0);
    assert!((q_new.w() - q.w()).abs() < 1e-10);
    assert!((q_new.x() - q.x()).abs() < 1e-10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn attitude_state_propagation() {
    let state = AttitudeState {
        quaternion: Quaternion::identity(),
        angular_velocity: AngularVelocity::new(0.01, 0.0, 0.0), // slow rotation about x
    };
    let propagated = state.propagate(1.0);
    // Should have rotated slightly
    assert!(propagated.quaternion.w() < 1.0);
    assert!((propagated.quaternion.norm().value - 1.0).abs() < 1e-10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn angle_between_orthogonal_vectors() {
    let a = Vector::new(vec![1.0, 0.0, 0.0]);
    let b = Vector::new(vec![0.0, 1.0, 0.0]);
    let angle = angle_between(&a, &b).value;
    assert!((angle - core::f64::consts::FRAC_PI_2).abs() < 1e-10);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn quaternion_norm_preserved_after_normalization(
            q0 in -10.0..10.0_f64,
            q1 in -10.0..10.0_f64,
            q2 in -10.0..10.0_f64,
            q3 in -10.0..10.0_f64
        ) {
            // Skip near-zero quaternions
            if q0*q0 + q1*q1 + q2*q2 + q3*q3 > 1e-10 {
                let q = Quaternion::new(q0, q1, q2, q3);
                prop_assert!((q.norm().value - 1.0).abs() < 1e-10,
                    "quaternion norm should be 1 after normalization, got {}", q.norm().value);
            }
        }

        #[test]
        fn conjugate_product_is_identity(
            q0 in -10.0..10.0_f64,
            q1 in -10.0..10.0_f64,
            q2 in -10.0..10.0_f64,
            q3 in -10.0..10.0_f64
        ) {
            if q0*q0 + q1*q1 + q2*q2 + q3*q3 > 1e-10 {
                let q = Quaternion::new(q0, q1, q2, q3);
                let result = q.multiply(&q.conjugate());
                prop_assert!((result.w() - 1.0).abs() < 1e-8,
                    "q * q_conj should give scalar ~1, got {}", result.w());
                prop_assert!(result.x().abs() < 1e-8);
                prop_assert!(result.y().abs() < 1e-8);
                prop_assert!(result.z().abs() < 1e-8);
            }
        }
    }

    pr4xis::register_praxis_value!(quaternion_norm_preserved_after_normalization, Verifiable);
    pr4xis::register_praxis_value!(conjugate_product_is_identity, Verifiable);
}
