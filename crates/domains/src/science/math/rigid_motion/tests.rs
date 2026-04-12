use pr4xis::ontology::{Axiom, Ontology};

use crate::science::math::rigid_motion::ontology::{
    Associativity, CompositionConsistency, IdentityElement, InverseExists, RigidMotionOntology,
};

#[test]
fn rigid_motion_ontology_validates() {
    RigidMotionOntology::validate().unwrap();
}

#[test]
fn se3_associativity() {
    assert!(Associativity.holds());
}

#[test]
fn se3_identity() {
    assert!(IdentityElement.holds());
}

#[test]
fn se3_inverse() {
    assert!(InverseExists.holds());
}

#[test]
fn se3_composition_consistency() {
    assert!(CompositionConsistency.holds());
}

#[cfg(test)]
mod proptest_proofs {
    use crate::science::math::rigid_motion::pose::Pose;
    use crate::science::math::rotation::quaternion::Quaternion;
    use proptest::prelude::*;

    fn arb_quaternion() -> impl Strategy<Value = Quaternion> {
        (-1.0..1.0_f64, -1.0..1.0_f64, -1.0..1.0_f64, -1.0..1.0_f64)
            .prop_filter("non-zero", |(w, x, y, z)| {
                w * w + x * x + y * y + z * z > 1e-6
            })
            .prop_map(|(w, x, y, z)| Quaternion::new(w, x, y, z))
    }

    fn arb_pose() -> impl Strategy<Value = Pose> {
        (
            arb_quaternion(),
            -100.0..100.0_f64,
            -100.0..100.0_f64,
            -100.0..100.0_f64,
        )
            .prop_map(|(q, tx, ty, tz)| Pose {
                rotation: q,
                translation: [tx, ty, tz],
            })
    }

    proptest! {
        #[test]
        fn composition_is_associative(
            a in arb_pose(),
            b in arb_pose(),
            c in arb_pose(),
        ) {
            let ab_c = a.compose(&b).compose(&c);
            let a_bc = a.compose(&b.compose(&c));
            prop_assert!(ab_c == a_bc);
        }

        #[test]
        fn inverse_yields_identity(p in arb_pose()) {
            let result = p.compose(&p.inverse());
            prop_assert!(result == Pose::identity());
        }

        #[test]
        fn transform_point_preserves_distance_under_pure_rotation(
            q in arb_quaternion(),
            px in -100.0..100.0_f64,
            py in -100.0..100.0_f64,
            pz in -100.0..100.0_f64,
        ) {
            let pose = Pose::from_rotation(q);
            let p = [px, py, pz];
            let p2 = pose.transform_point(p);
            let d1 = (p[0]*p[0] + p[1]*p[1] + p[2]*p[2]).sqrt();
            let d2 = (p2[0]*p2[0] + p2[1]*p2[1] + p2[2]*p2[2]).sqrt();
            prop_assert!((d1 - d2).abs() < 1e-9);
        }
    }
}
