use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::rotation::ontology::{
    Associativity, DcmOrthogonality, IdentityElement, InverseExists, QuaternionDcmRoundtrip,
    RotationCategory, RotationOntology, UnitNormClosure,
};

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn rotation_category_laws() {
    assert_category_laws::<RotationCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn rotation_ontology_validates() {
    RotationOntology::validate().unwrap();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn so3_closure() {
    assert!(UnitNormClosure.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn so3_associativity() {
    assert!(Associativity.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn so3_identity() {
    assert!(IdentityElement.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn so3_inverse() {
    assert!(InverseExists.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dcm_is_proper_rotation() {
    assert!(DcmOrthogonality.verify().is_ok());
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn quaternion_dcm_roundtrip() {
    assert!(QuaternionDcmRoundtrip.verify().is_ok());
}

// ---------------------------------------------------------------------------
// H1: Zero quaternion returns identity instead of NaN
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Honest)]
#[test]
fn zero_quaternion_returns_identity() {
    let q = crate::formal::math::rotation::quaternion::Quaternion::new(0.0, 0.0, 0.0, 0.0);
    let id = crate::formal::math::rotation::quaternion::Quaternion::identity();
    assert_eq!(q, id, "zero quaternion should return identity");
    assert!(!q.w().is_nan(), "w should not be NaN");
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn zero_quaternion_normalize_returns_identity() {
    let zero_q = crate::formal::math::rotation::quaternion::Quaternion::new(0.0, 0.0, 0.0, 0.0);
    let normalized = zero_q.normalize();
    let id = crate::formal::math::rotation::quaternion::Quaternion::identity();
    assert_eq!(normalized, id);
    assert!(!normalized.w().is_nan());
}

#[cfg(test)]
mod proptest_proofs {
    use crate::formal::math::linear_algebra::vector_space::Vector;
    use crate::formal::math::rotation::dcm::Dcm;
    use crate::formal::math::rotation::quaternion::Quaternion;
    use proptest::prelude::*;

    fn arb_quaternion() -> impl Strategy<Value = Quaternion> {
        (-1.0..1.0_f64, -1.0..1.0_f64, -1.0..1.0_f64, -1.0..1.0_f64)
            .prop_filter("non-zero", |(w, x, y, z)| {
                w * w + x * x + y * y + z * z > 1e-6
            })
            .prop_map(|(w, x, y, z)| Quaternion::new(w, x, y, z))
    }

    proptest! {
        #[test]
        fn unit_norm_preserved_under_composition(
            a in arb_quaternion(),
            b in arb_quaternion(),
        ) {
            let c = a.multiply(&b);
            prop_assert!((c.norm() - 1.0).abs() < 1e-10,
                "norm was {}", c.norm());
        }

        #[test]
        fn composition_is_associative(
            a in arb_quaternion(),
            b in arb_quaternion(),
            c in arb_quaternion(),
        ) {
            let ab_c = a.multiply(&b).multiply(&c);
            let a_bc = a.multiply(&b.multiply(&c));
            prop_assert!(ab_c == a_bc);
        }

        #[test]
        fn inverse_yields_identity(q in arb_quaternion()) {
            let product = q.multiply(&q.inverse());
            prop_assert!(product == Quaternion::identity());
        }

        #[test]
        fn dcm_is_orthogonal(q in arb_quaternion()) {
            let dcm = Dcm::from_quaternion(&q);
            prop_assert!(dcm.is_orthogonal(1e-10));
        }

        #[test]
        fn dcm_determinant_is_one(q in arb_quaternion()) {
            let dcm = Dcm::from_quaternion(&q);
            prop_assert!((dcm.determinant() - 1.0).abs() < 1e-10);
        }

        #[test]
        fn quaternion_dcm_roundtrip_preserves_rotation(q in arb_quaternion()) {
            let dcm = Dcm::from_quaternion(&q);
            let q2 = dcm.to_quaternion();
            prop_assert!(q == q2);
        }

        #[test]
        fn rotate_vector_preserves_norm(
            q in arb_quaternion(),
            vx in -100.0..100.0_f64,
            vy in -100.0..100.0_f64,
            vz in -100.0..100.0_f64,
        ) {
            let v = Vector::new(vec![vx, vy, vz]);
            let v2 = q.rotate_vector(&v);
            let norm_before = (v.get(0)*v.get(0) + v.get(1)*v.get(1) + v.get(2)*v.get(2)).sqrt();
            let norm_after = (v2.get(0)*v2.get(0) + v2.get(1)*v2.get(1) + v2.get(2)*v2.get(2)).sqrt();
            prop_assert!((norm_before - norm_after).abs() < 1e-9);
        }

        #[test]
        fn dcm_and_quaternion_rotate_same(
            q in arb_quaternion(),
            vx in -100.0..100.0_f64,
            vy in -100.0..100.0_f64,
            vz in -100.0..100.0_f64,
        ) {
            let v = Vector::new(vec![vx, vy, vz]);
            let v_quat = q.rotate_vector(&v);
            let dcm = Dcm::from_quaternion(&q);
            let v_dcm = dcm.rotate_vector(&v);
            prop_assert!((v_quat.get(0) - v_dcm.get(0)).abs() < 1e-9);
            prop_assert!((v_quat.get(1) - v_dcm.get(1)).abs() < 1e-9);
            prop_assert!((v_quat.get(2) - v_dcm.get(2)).abs() < 1e-9);
        }
    }

    pr4xis::register_praxis_value!(unit_norm_preserved_under_composition, Verifiable);
    pr4xis::register_praxis_value!(composition_is_associative, Verifiable);
    pr4xis::register_praxis_value!(inverse_yields_identity, Verifiable);
    pr4xis::register_praxis_value!(dcm_is_orthogonal, Verifiable);
    pr4xis::register_praxis_value!(dcm_determinant_is_one, Verifiable);
    pr4xis::register_praxis_value!(quaternion_dcm_roundtrip_preserves_rotation, Deterministic);
    pr4xis::register_praxis_value!(rotate_vector_preserves_norm, Verifiable);
    pr4xis::register_praxis_value!(dcm_and_quaternion_rotate_same, Verifiable);
}

// ---------------------------------------------------------------------------
// Meta-axiom: the Quaternion TYPE enforces the unit-norm invariant.
//
// Every construction path must produce a quaternion with |q| = 1.
// This is the UnknownKnown gap closed by making fields private:
// the ontology claims unit-norm, now the type guarantees it.
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn type_enforces_unit_norm_new() {
    // Zero input must produce unit quaternion (identity)
    let q1 = crate::formal::math::rotation::quaternion::Quaternion::new(0.0, 0.0, 0.0, 0.0);
    assert!(
        (q1.norm() - 1.0).abs() < 1e-10,
        "zero input must produce unit quaternion, got norm={}",
        q1.norm()
    );

    // Unnormalized input must be normalized
    let q2 = crate::formal::math::rotation::quaternion::Quaternion::new(100.0, 0.0, 0.0, 0.0);
    assert!(
        (q2.norm() - 1.0).abs() < 1e-10,
        "unnormalized input must be normalized, got norm={}",
        q2.norm()
    );

    // Arbitrary large components
    let q3 = crate::formal::math::rotation::quaternion::Quaternion::new(3.0, 4.0, 5.0, 6.0);
    assert!(
        (q3.norm() - 1.0).abs() < 1e-10,
        "arbitrary input must be normalized, got norm={}",
        q3.norm()
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn type_enforces_unit_norm_from_axis_angle() {
    // Unit axis
    let q1 = crate::formal::math::rotation::quaternion::Quaternion::from_axis_angle(
        &Vector::new(vec![1.0, 0.0, 0.0]),
        1.0,
    );
    assert!(
        (q1.norm() - 1.0).abs() < 1e-10,
        "unit axis must produce unit quaternion"
    );

    // Non-unit axis (the gap that was previously open)
    let q2 = crate::formal::math::rotation::quaternion::Quaternion::from_axis_angle(
        &Vector::new(vec![5.0, 0.0, 0.0]),
        1.0,
    );
    assert!(
        (q2.norm() - 1.0).abs() < 1e-10,
        "non-unit axis must still produce unit quaternion, got norm={}",
        q2.norm()
    );

    // All-component non-unit axis
    let q3 = crate::formal::math::rotation::quaternion::Quaternion::from_axis_angle(
        &Vector::new(vec![3.0, 4.0, 5.0]),
        2.5,
    );
    assert!(
        (q3.norm() - 1.0).abs() < 1e-10,
        "arbitrary axis must produce unit quaternion, got norm={}",
        q3.norm()
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn type_enforces_unit_norm_from_euler_321() {
    let q = crate::formal::math::rotation::quaternion::Quaternion::from_euler_321(1.0, 0.5, 0.3);
    assert!(
        (q.norm() - 1.0).abs() < 1e-10,
        "Euler construction must produce unit quaternion"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn type_enforces_unit_norm_from_dcm() {
    // Identity DCM
    let id = Matrix::new(3, 3, vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    let q = crate::formal::math::rotation::quaternion::Quaternion::from_matrix(&id);
    assert!(
        (q.norm() - 1.0).abs() < 1e-10,
        "from_dcm(identity) must produce unit quaternion"
    );

    // 90-degree rotation about z
    let r = Matrix::new(3, 3, vec![0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    let q2 = crate::formal::math::rotation::quaternion::Quaternion::from_matrix(&r);
    assert!(
        (q2.norm() - 1.0).abs() < 1e-10,
        "from_dcm(Rz90) must produce unit quaternion"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn type_enforces_unit_norm_identity() {
    let q = crate::formal::math::rotation::quaternion::Quaternion::identity();
    assert!(
        (q.norm() - 1.0).abs() < 1e-15,
        "identity must be exactly unit"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn accessors_return_components() {
    let q = crate::formal::math::rotation::quaternion::Quaternion::new(1.0, 2.0, 3.0, 4.0);
    // After normalization the components should be scaled
    let n = (1.0_f64 + 4.0 + 9.0 + 16.0).sqrt();
    assert!((q.w() - 1.0 / n).abs() < 1e-12);
    assert!((q.x() - 2.0 / n).abs() < 1e-12);
    assert!((q.y() - 3.0 / n).abs() < 1e-12);
    assert!((q.z() - 4.0 / n).abs() < 1e-12);
}
