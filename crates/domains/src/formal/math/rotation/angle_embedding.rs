//! The embedding **SO(2) ↪ SO(3)** — an [`Angle`] as a rotation about a fixed
//! axis, connecting the circle-group [`Angle`] ontology to the [`Quaternion`]
//! rotation ontology by a group homomorphism.
//!
//! # Literature
//!
//! - Rotations about a fixed oriented axis form a **one-parameter subgroup of
//!   the 3-D rotation group SO(3), isomorphic to the circle group SO(2)** (a
//!   maximal torus). Hence angle addition maps to rotation composition — the
//!   map is a group homomorphism. Stillwell (2008), *Naive Lie Theory*, §2–3;
//!   *3-D rotation group* (one-parameter subgroups / maximal tori).
//! - The quaternion realisation `q = cos(θ/2) + sin(θ/2)·n̂` is Hamilton (1844);
//!   Shoemake (1985).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::Axiom;

use crate::formal::math::angle::Angle;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::rotation::quaternion::Quaternion;

/// The one-parameter subgroup **SO(2) ↪ SO(3)**: the rotation by `angle` about a
/// fixed `axis`, as a unit [`Quaternion`].
///
/// Rotations about a common axis form a one-parameter subgroup of SO(3)
/// isomorphic to the circle group SO(2), so this map is a group homomorphism:
/// angle addition ↦ quaternion composition (see [`AngleAdditionIsRotationComposition`]).
pub fn rotation_about(angle: &Angle, axis: &Vector) -> Quaternion {
    Quaternion::from_axis_angle(axis, angle.radians())
}

/// Axiom: the zero angle maps to the identity rotation — the homomorphism sends
/// the group identity to the group identity.
pub struct AngleEmbeddingPreservesIdentity;

impl Axiom for AngleEmbeddingPreservesIdentity {
    fn verify(&self) -> Verdict {
        let axes = [
            Vector::new(vec![0.0, 0.0, 1.0]),
            Vector::new(vec![1.0, 2.0, 2.0]),
        ];
        let ok = axes
            .iter()
            .all(|axis| rotation_about(&Angle::ZERO, axis) == Quaternion::identity());
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AngleEmbeddingPreservesIdentity",
        "the SO(2)↪SO(3) embedding sends the zero angle to the identity rotation",
        "One-parameter subgroups of SO(3) (Stillwell 2008, Naive Lie Theory §2)"
    );
}
pr4xis::register_axiom!(
    AngleEmbeddingPreservesIdentity,
    "One-parameter subgroups of SO(3) (Stillwell 2008, Naive Lie Theory §2)"
);

/// Axiom: angle addition maps to quaternion composition about a common axis —
/// the SO(2)↪SO(3) map is a group homomorphism:
/// `rotation_about(a + b, n̂) = rotation_about(a, n̂) ∘ rotation_about(b, n̂)`.
pub struct AngleAdditionIsRotationComposition;

impl Axiom for AngleAdditionIsRotationComposition {
    fn verify(&self) -> Verdict {
        let axis = Vector::new(vec![1.0, 2.0, 2.0]); // normalized inside from_axis_angle
        let fixtures = [(0.3, 0.5), (1.2, -0.7), (2.0, 2.5), (-1.0, -1.5)];
        let ok = fixtures.iter().all(|&(a, b)| {
            let (ra, rb) = (Angle::from_radians(a), Angle::from_radians(b));
            let sum = rotation_about(&ra.add(&rb), &axis);
            let composed = rotation_about(&ra, &axis).compose(&rotation_about(&rb, &axis));
            // Quaternion PartialEq is rotation-equality (q ≡ -q), so this is the
            // homomorphism law in SO(3), double-cover aware.
            sum == composed
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AngleAdditionIsRotationComposition",
        "rotation_about(a+b, n̂) = rotation_about(a, n̂) ∘ rotation_about(b, n̂): the SO(2)↪SO(3) embedding is a group homomorphism",
        "One-parameter subgroups of SO(3) are isomorphic to SO(2) (Stillwell 2008, Naive Lie Theory §2–3)"
    );
}
pr4xis::register_axiom!(
    AngleAdditionIsRotationComposition,
    "One-parameter subgroups of SO(3) are isomorphic to SO(2) (Stillwell 2008, Naive Lie Theory §2–3)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use core::f64::consts::FRAC_PI_2;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn embedding_axioms_hold() {
        assert!(AngleEmbeddingPreservesIdentity.verify().is_ok());
        assert!(AngleAdditionIsRotationComposition.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn right_angle_about_z_rotates_x_to_y() {
        // A +90° rotation about +z takes the x-axis to the y-axis.
        let q = rotation_about(
            &Angle::from_radians(FRAC_PI_2),
            &Vector::new(vec![0.0, 0.0, 1.0]),
        );
        let rotated = q.rotate_vector(&Vector::new(vec![1.0, 0.0, 0.0]));
        assert!((rotated.get(0)).abs() < 1e-9);
        assert!((rotated.get(1) - 1.0).abs() < 1e-9);
    }

    /// Gap-analysis note: the embedding is faithful as a map of *rotations*, but
    /// the round-trip angle → quaternion → angle is faithful only modulo the
    /// SU(2)→SO(3) double cover (a full turn, 2π, maps to the identity). This
    /// test documents that the rotation is preserved even where the raw angle
    /// wraps.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn full_turn_maps_to_identity_rotation() {
        let axis = Vector::new(vec![0.3, -0.4, 0.5]);
        let full = rotation_about(&Angle::from_turns(1.0), &axis);
        assert!(full == Quaternion::identity());
    }
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    fn any_axis() -> impl Strategy<Value = Vector> {
        (0.1f64..3.0, -3.0f64..3.0, -3.0f64..3.0).prop_map(|(x, y, z)| Vector::new(vec![x, y, z]))
    }

    proptest! {
        /// The SO(2)↪SO(3) embedding is a homomorphism for every angle pair and axis.
        #[test]
        fn embedding_is_homomorphism(
            a in -50.0f64..50.0, b in -50.0f64..50.0, axis in any_axis(),
        ) {
            let (ra, rb) = (Angle::from_radians(a), Angle::from_radians(b));
            let sum = rotation_about(&ra.add(&rb), &axis);
            let composed = rotation_about(&ra, &axis).compose(&rotation_about(&rb, &axis));
            prop_assert!(sum == composed);
        }

        /// Rotating by an angle about an axis agrees with the quaternion's own action.
        #[test]
        fn embedding_rotates_consistently(theta in -PI..PI, axis in any_axis()) {
            let q = rotation_about(&Angle::from_radians(theta), &axis);
            // A point on the axis is fixed by the rotation.
            let fixed = q.rotate_vector(&axis);
            prop_assert!((fixed.get(0) - axis.get(0)).abs() < 1e-6);
            prop_assert!((fixed.get(1) - axis.get(1)).abs() < 1e-6);
            prop_assert!((fixed.get(2) - axis.get(2)).abs() < 1e-6);
        }
    }

    use core::f64::consts::PI;

    pr4xis::register_praxis_value!(embedding_is_homomorphism, Verifiable);
    pr4xis::register_praxis_value!(embedding_rotates_consistently, Verifiable);
}
