#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::perception::lidar_camera::calibration::{
    CameraIntrinsics, ExtrinsicCalibration,
};

/// A 3D LiDAR point with intensity.
#[derive(Debug, Clone)]
pub struct LidarPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub intensity: f64,
}

/// A 2D camera detection (bounding box).
#[derive(Debug, Clone)]
pub struct CameraDetection {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
    pub class_id: usize,
    pub confidence: f64,
}

/// A projected LiDAR point in the camera image plane.
#[derive(Debug, Clone)]
pub struct ProjectedPoint {
    pub u: f64,
    pub v: f64,
    pub depth: f64,
    pub intensity: f64,
}

/// Project LiDAR points into the camera frame.
pub fn project_lidar_points(
    points: &[LidarPoint],
    extrinsic: &ExtrinsicCalibration,
    intrinsic: &CameraIntrinsics,
) -> Vec<ProjectedPoint> {
    points
        .iter()
        .filter_map(|p| {
            let cam_pt = extrinsic.transform_point([p.x, p.y, p.z]);
            let depth = cam_pt[2];
            intrinsic.project(cam_pt).map(|[u, v]| ProjectedPoint {
                u,
                v,
                depth,
                intensity: p.intensity,
            })
        })
        .collect()
}

/// Associate projected LiDAR points with camera detections.
///
/// A point is associated with a detection if it falls within the bounding box.
pub fn associate_points_to_detections(
    projected: &[ProjectedPoint],
    detections: &[CameraDetection],
) -> Vec<(usize, Vec<usize>)> {
    detections
        .iter()
        .enumerate()
        .map(|(det_idx, det)| {
            let associated: Vec<usize> = projected
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    p.u >= det.x_min && p.u <= det.x_max && p.v >= det.y_min && p.v <= det.y_max
                })
                .map(|(idx, _)| idx)
                .collect();
            (det_idx, associated)
        })
        .collect()
}

/// Fused detection: camera detection enriched with LiDAR depth.
#[derive(Debug, Clone)]
pub struct FusedDetection {
    pub class_id: usize,
    pub confidence: f64,
    pub median_depth: f64,
    pub num_lidar_points: usize,
}

/// Compute fused detections from associations.
pub fn fuse_detections(
    detections: &[CameraDetection],
    projected: &[ProjectedPoint],
    associations: &[(usize, Vec<usize>)],
) -> Vec<FusedDetection> {
    associations
        .iter()
        .filter_map(|(det_idx, point_indices)| {
            if point_indices.is_empty() {
                return None;
            }
            let det = &detections[*det_idx];
            let mut depths: Vec<f64> = point_indices.iter().map(|&i| projected[i].depth).collect();
            depths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
            let median_depth = depths[depths.len() / 2];
            Some(FusedDetection {
                class_id: det.class_id,
                confidence: det.confidence,
                median_depth,
                num_lidar_points: point_indices.len(),
            })
        })
        .collect()
}
