use pr4xis::category::laws::assert_category_laws;
use pr4xis::logic::Axiom;
use pr4xis::ontology::Ontology;

use crate::applied::sensor_fusion::frame::boresight::Boresight;
use crate::applied::sensor_fusion::frame::lever_arm::LeverArm;
use crate::applied::sensor_fusion::frame::ontology::*;
use crate::applied::sensor_fusion::frame::reference::ReferenceFrame;
use crate::applied::sensor_fusion::frame::transform::FrameTransform;

use crate::formal::math::geometry::vector::Vec3;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::rotation::quaternion::Quaternion;

// ---------------------------------------------------------------------------
// Category law validation
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn frame_category_laws() {
    assert_category_laws::<FrameCategory>();
}

// ---------------------------------------------------------------------------
// Ontology validation
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn frame_ontology_validates() {
    FrameOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn axiom_transforms_compose_associatively() {
    assert!(TransformsComposeAssociatively.verify().is_ok());
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn axiom_identity_exists() {
    assert!(IdentityExists.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_transforms_invertible() {
    assert!(TransformsInvertible.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_all_frames_right_handed() {
    assert!(AllFramesRightHanded.verify().is_ok());
}

// ---------------------------------------------------------------------------
// Boresight tests
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn boresight_compose_chain() {
    let q1 = Quaternion::from_axis_angle(&Vector::new(vec![1.0, 0.0, 0.0]), 0.01);
    let q2 = Quaternion::from_axis_angle(&Vector::new(vec![0.0, 1.0, 0.0]), 0.02);
    let b1 = Boresight::new(ReferenceFrame::IMU, ReferenceFrame::Body, q1.clone(), 0.9);
    let b2 = Boresight::new(
        ReferenceFrame::Body,
        ReferenceFrame::Camera,
        q2.clone(),
        0.8,
    );
    let composed = b1.compose(&b2).unwrap();
    assert_eq!(composed.from_sensor, ReferenceFrame::IMU);
    assert_eq!(composed.to_sensor, ReferenceFrame::Camera);
    // Composed quality is min of components
    assert!((composed.calibration_quality - 0.8).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// Lever arm tests
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn lever_arm_velocity_correction_orthogonal() {
    // Pure Z rotation with lever arm along X => velocity along Y
    let la = LeverArm::new(
        ReferenceFrame::IMU,
        ReferenceFrame::GNSS,
        Vec3::new(2.0, 0.0, 0.0),
        ReferenceFrame::Body,
    );
    let omega = Vector::new(vec![0.0, 0.0, 0.5]); // 0.5 rad/s around Z
    let v = la.velocity_correction(&omega);
    assert!((v.get(0)).abs() < 1e-10);
    assert!((v.get(1) - 1.0).abs() < 1e-10); // 0.5 * 2.0 = 1.0
    assert!((v.get(2)).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// Property-based proofs
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use pr4xis::category::Category;
    use proptest::prelude::*;

    fn arb_frame() -> impl Strategy<Value = ReferenceFrame> {
        prop_oneof![
            Just(ReferenceFrame::ECEF),
            Just(ReferenceFrame::ECI),
            Just(ReferenceFrame::NED),
            Just(ReferenceFrame::ENU),
            Just(ReferenceFrame::Body),
            Just(ReferenceFrame::IMU),
            Just(ReferenceFrame::Camera),
            Just(ReferenceFrame::LiDAR),
            Just(ReferenceFrame::Radar),
            Just(ReferenceFrame::GNSS),
        ]
    }

    proptest! {
        /// Identity composed with any morphism yields that morphism.
        #[test]
        fn identity_is_neutral(a in arb_frame(), b in arb_frame()) {
            let id = FrameCategory::identity(&a);
            let f = FrameTransform::new(a, b);
            let composed = FrameCategory::compose(&id, &f).unwrap();
            prop_assert_eq!(composed, f);
        }

        /// Composition is associative for arbitrary frame triples.
        #[test]
        fn composition_associative(a in arb_frame(), b in arb_frame(), c in arb_frame(), d in arb_frame()) {
            let f = FrameTransform::new(a, b);
            let g = FrameTransform::new(b, c);
            let h = FrameTransform::new(c, d);

            let fg = FrameCategory::compose(&f, &g).unwrap();
            let fgh_left = FrameCategory::compose(&fg, &h).unwrap();

            let gh = FrameCategory::compose(&g, &h).unwrap();
            let fgh_right = FrameCategory::compose(&f, &gh).unwrap();

            prop_assert_eq!(fgh_left, fgh_right);
        }

        /// Boresight inverse composed with original yields identity rotation.
        #[test]
        fn boresight_inverse_identity(
            angle in -0.1..0.1_f64,
        ) {
            let q = Quaternion::from_axis_angle(&Vector::new(vec![0.0, 0.0, 1.0]), angle);
            let b = Boresight::new(ReferenceFrame::IMU, ReferenceFrame::Body, q, 0.9);
            let b_inv = b.inverse();
            let composed = b.compose(&b_inv).unwrap();
            prop_assert!(composed.magnitude().value < 1e-6,
                "inverse should yield identity, got magnitude {}", composed.magnitude().value);
        }

        /// Lever arm inverse-inverse is original offset.
        #[test]
        fn lever_arm_double_inverse(
            x in -10.0..10.0_f64,
            y in -10.0..10.0_f64,
            z in -10.0..10.0_f64,
        ) {
            let la = LeverArm::new(
                ReferenceFrame::IMU,
                ReferenceFrame::GNSS,
                Vec3::new(x, y, z),
                ReferenceFrame::Body,
            );
            let la2 = la.inverse().inverse();
            prop_assert!((la2.offset.x - la.offset.x).abs() < 1e-10);
            prop_assert!((la2.offset.y - la.offset.y).abs() < 1e-10);
            prop_assert!((la2.offset.z - la.offset.z).abs() < 1e-10);
        }
    }

    pr4xis::register_praxis_value!(identity_is_neutral, Deterministic);
    pr4xis::register_praxis_value!(composition_associative, Deterministic);
    pr4xis::register_praxis_value!(boresight_inverse_identity, Deterministic);
    pr4xis::register_praxis_value!(lever_arm_double_inverse, Deterministic);
}
