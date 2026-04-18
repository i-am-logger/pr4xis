#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::perception::lidar_camera::calibration::{
    CameraIntrinsics, ExtrinsicCalibration,
};
use crate::applied::perception::lidar_camera::engine::*;
use crate::applied::perception::lidar_camera::ontology::*;

#[test]
fn lidar_camera_category_laws() {
    check_category_laws::<LidarCameraCategory>().unwrap();
}

#[test]
fn lidar_camera_ontology_validates() {
    LidarCameraOntology::validate().unwrap();
}

#[test]
fn projection_preserves_ordering_holds() {
    assert!(ProjectionPreservesOrdering.holds());
}

#[test]
fn pipeline_is_sequential_holds() {
    assert!(PipelineIsSequential.holds());
}

#[test]
fn identity_calibration_preserves_point() {
    let cal = ExtrinsicCalibration::identity();
    let result = cal.transform_point([1.0, 2.0, 3.0]);
    assert!((result[0] - 1.0).abs() < 1e-12);
    assert!((result[1] - 2.0).abs() < 1e-12);
    assert!((result[2] - 3.0).abs() < 1e-12);
}

#[test]
fn camera_projection_behind_camera_returns_none() {
    let intrinsic = CameraIntrinsics {
        fx: 500.0,
        fy: 500.0,
        cx: 320.0,
        cy: 240.0,
    };
    assert!(intrinsic.project([1.0, 1.0, -1.0]).is_none());
    assert!(intrinsic.project([1.0, 1.0, 0.0]).is_none());
}

#[test]
fn project_lidar_points_filters_behind_camera() {
    let extrinsic = ExtrinsicCalibration::identity();
    let intrinsic = CameraIntrinsics {
        fx: 500.0,
        fy: 500.0,
        cx: 320.0,
        cy: 240.0,
    };
    let points = vec![
        LidarPoint {
            x: 0.0,
            y: 0.0,
            z: 5.0,
            intensity: 1.0,
        },
        LidarPoint {
            x: 0.0,
            y: 0.0,
            z: -1.0,
            intensity: 1.0,
        },
    ];
    let projected = project_lidar_points(&points, &extrinsic, &intrinsic);
    assert_eq!(projected.len(), 1);
    assert!((projected[0].depth - 5.0).abs() < 1e-12);
}

#[test]
fn association_matches_points_in_bbox() {
    let projected = vec![
        ProjectedPoint {
            u: 100.0,
            v: 100.0,
            depth: 5.0,
            intensity: 1.0,
        },
        ProjectedPoint {
            u: 500.0,
            v: 500.0,
            depth: 10.0,
            intensity: 1.0,
        },
    ];
    let detections = vec![CameraDetection {
        x_min: 50.0,
        y_min: 50.0,
        x_max: 200.0,
        y_max: 200.0,
        class_id: 0,
        confidence: 0.9,
    }];
    let assoc = associate_points_to_detections(&projected, &detections);
    assert_eq!(assoc.len(), 1);
    assert_eq!(assoc[0].1, vec![0]); // only first point in bbox
}

// ---------------------------------------------------------------------------
// H8: NaN depths do not panic in fuse_detections
// ---------------------------------------------------------------------------

#[test]
fn fuse_detections_nan_depth_no_panic() {
    let detections = vec![CameraDetection {
        x_min: 0.0,
        y_min: 0.0,
        x_max: 1000.0,
        y_max: 1000.0,
        class_id: 0,
        confidence: 0.9,
    }];
    let projected = vec![
        ProjectedPoint {
            u: 100.0,
            v: 100.0,
            depth: f64::NAN,
            intensity: 1.0,
        },
        ProjectedPoint {
            u: 200.0,
            v: 200.0,
            depth: 5.0,
            intensity: 1.0,
        },
    ];
    let assoc = associate_points_to_detections(&projected, &detections);
    // Should not panic even with NaN depth
    let fused = fuse_detections(&detections, &projected, &assoc);
    assert_eq!(fused.len(), 1);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn identity_calibration_is_identity(x in -100.0..100.0_f64, y in -100.0..100.0_f64, z in -100.0..100.0_f64) {
            let cal = ExtrinsicCalibration::identity();
            let result = cal.transform_point([x, y, z]);
            prop_assert!((result[0] - x).abs() < 1e-12);
            prop_assert!((result[1] - y).abs() < 1e-12);
            prop_assert!((result[2] - z).abs() < 1e-12);
        }

        #[test]
        fn projection_depth_preserved(
            x in -10.0..10.0_f64,
            y in -10.0..10.0_f64,
            z in 0.1..100.0_f64
        ) {
            let intrinsic = CameraIntrinsics { fx: 500.0, fy: 500.0, cx: 320.0, cy: 240.0 };
            let result = intrinsic.project([x, y, z]);
            prop_assert!(result.is_some(), "point in front of camera should project");
        }
    }
}
