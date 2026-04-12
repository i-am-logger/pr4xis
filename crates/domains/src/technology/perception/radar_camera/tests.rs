use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::technology::perception::radar_camera::engine::*;
use crate::technology::perception::radar_camera::ontology::*;

#[test]
fn radar_camera_category_laws() {
    check_category_laws::<RadarCameraCategory>().unwrap();
}

#[test]
fn radar_camera_ontology_validates() {
    RadarCameraOntology::validate().unwrap();
}

#[test]
fn both_modalities_required_holds() {
    assert!(BothModalitiesRequired.holds());
}

#[test]
fn fused_output_is_terminal_holds() {
    assert!(FusedOutputIsTerminal.holds());
}

#[test]
fn azimuth_to_image_center() {
    // Azimuth 0 should map to image center
    let x = radar_azimuth_to_image_x(0.0, 640.0, std::f64::consts::PI);
    assert!((x - 320.0).abs() < 1e-6);
}

#[test]
fn radar_camera_association() {
    let frame = AlignedFrame {
        radar_targets: vec![RadarTarget {
            range: 50.0,
            doppler: -10.0,
            azimuth: 0.0,
            rcs: 5.0,
        }],
        camera_objects: vec![CameraObject {
            x_min: 300.0,
            y_min: 100.0,
            x_max: 400.0,
            y_max: 300.0,
            class_label: "car",
            confidence: 0.95,
        }],
        time_offset_s: 0.0,
    };
    let fused = associate_radar_camera(&frame, 640.0, std::f64::consts::PI);
    assert_eq!(fused.len(), 1);
    assert_eq!(fused[0].class_label, "car");
    assert!((fused[0].range - 50.0).abs() < 1e-12);
}

// ---------------------------------------------------------------------------
// H8: NaN confidence does not panic in associate_radar_camera
// ---------------------------------------------------------------------------

#[test]
fn associate_radar_camera_nan_confidence_no_panic() {
    let frame = AlignedFrame {
        radar_targets: vec![RadarTarget {
            range: 50.0,
            doppler: -10.0,
            azimuth: 0.0,
            rcs: 5.0,
        }],
        camera_objects: vec![
            CameraObject {
                x_min: 300.0,
                y_min: 100.0,
                x_max: 400.0,
                y_max: 300.0,
                class_label: "car",
                confidence: f64::NAN,
            },
            CameraObject {
                x_min: 300.0,
                y_min: 100.0,
                x_max: 400.0,
                y_max: 300.0,
                class_label: "truck",
                confidence: 0.8,
            },
        ],
        time_offset_s: 0.0,
    };
    // Should not panic
    let fused = associate_radar_camera(&frame, 640.0, std::f64::consts::PI);
    assert_eq!(fused.len(), 1);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn azimuth_maps_within_image(
            azimuth in -1.0..1.0_f64,
            width in 100.0..2000.0_f64
        ) {
            let fov = std::f64::consts::PI;
            let x = radar_azimuth_to_image_x(azimuth, width, fov);
            // Azimuth in [-fov/2, fov/2] should map within [0, width]
            if azimuth.abs() <= fov / 2.0 {
                prop_assert!(x >= 0.0 && x <= width,
                    "azimuth {} mapped to x={}, should be in [0, {}]", azimuth, x, width);
            }
        }

        #[test]
        fn association_never_panics(
            range in 1.0..1000.0_f64,
            doppler in -50.0..50.0_f64,
            azimuth in -0.5..0.5_f64
        ) {
            let frame = AlignedFrame {
                radar_targets: vec![RadarTarget {
                    range, doppler, azimuth, rcs: 5.0,
                }],
                camera_objects: vec![],
                time_offset_s: 0.0,
            };
            let fused = associate_radar_camera(&frame, 640.0, std::f64::consts::PI);
            prop_assert!(fused.is_empty(), "no camera objects means no fused detections");
        }
    }
}
