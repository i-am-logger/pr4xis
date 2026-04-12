use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::space::attitude::engine::*;
use crate::applied::space::attitude::kinematics::*;
use crate::applied::space::attitude::ontology::*;

#[test]
fn attitude_category_laws() {
    check_category_laws::<AttitudeCategory>().unwrap();
}

#[test]
fn attitude_ontology_validates() {
    AttitudeOntology::validate().unwrap();
}

#[test]
fn quaternion_unit_norm_holds() {
    assert!(QuaternionUnitNorm.holds());
}

#[test]
fn star_tracker_most_accurate_holds() {
    assert!(StarTrackerMostAccurate.holds());
}

#[test]
fn identity_quaternion_has_unit_norm() {
    let q = Quaternion::identity();
    assert!((q.norm() - 1.0).abs() < 1e-12);
}

#[test]
fn quaternion_multiplication_associative() {
    let q1 = Quaternion::new(1.0, 0.1, 0.2, 0.3);
    let q2 = Quaternion::new(0.5, 0.3, 0.1, 0.2);
    let q3 = Quaternion::new(0.7, 0.2, 0.4, 0.1);

    let left = q1.multiply(&q2).multiply(&q3);
    let right = q1.multiply(&q2.multiply(&q3));
    assert!((left.q0 - right.q0).abs() < 1e-10);
    assert!((left.q1 - right.q1).abs() < 1e-10);
    assert!((left.q2 - right.q2).abs() < 1e-10);
    assert!((left.q3 - right.q3).abs() < 1e-10);
}

#[test]
fn quaternion_conjugate_gives_identity() {
    let q = Quaternion::new(1.0, 0.1, 0.2, 0.3);
    let result = q.multiply(&q.conjugate());
    assert!((result.q0 - 1.0).abs() < 1e-10);
    assert!(result.q1.abs() < 1e-10);
    assert!(result.q2.abs() < 1e-10);
    assert!(result.q3.abs() < 1e-10);
}

#[test]
fn zero_angular_velocity_preserves_attitude() {
    let q = Quaternion::new(1.0, 0.1, 0.2, 0.3);
    let omega = [0.0, 0.0, 0.0];
    let q_new = propagate_attitude(&q, &omega, 1.0);
    assert!((q_new.q0 - q.q0).abs() < 1e-10);
    assert!((q_new.q1 - q.q1).abs() < 1e-10);
}

#[test]
fn attitude_state_propagation() {
    let state = AttitudeState {
        quaternion: Quaternion::identity(),
        angular_velocity: [0.01, 0.0, 0.0], // slow rotation about x
    };
    let propagated = state.propagate(1.0);
    // Should have rotated slightly
    assert!(propagated.quaternion.q0 < 1.0);
    assert!((propagated.quaternion.norm() - 1.0).abs() < 1e-10);
}

#[test]
fn angle_between_orthogonal_vectors() {
    let a = [1.0, 0.0, 0.0];
    let b = [0.0, 1.0, 0.0];
    let angle = angle_between(&a, &b);
    assert!((angle - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
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
                prop_assert!((q.norm() - 1.0).abs() < 1e-10,
                    "quaternion norm should be 1 after normalization, got {}", q.norm());
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
                prop_assert!((result.q0 - 1.0).abs() < 1e-8,
                    "q * q_conj should give scalar ~1, got {}", result.q0);
                prop_assert!(result.q1.abs() < 1e-8);
                prop_assert!(result.q2.abs() < 1e-8);
                prop_assert!(result.q3.abs() < 1e-8);
            }
        }
    }
}
