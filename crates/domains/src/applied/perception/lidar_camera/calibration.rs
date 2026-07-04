#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

/// Extrinsic calibration parameters for LiDAR-camera fusion.
///
/// Represents the rigid body transformation from LiDAR frame to camera frame.
///
/// Source: Zhang & Pless (2004), "Extrinsic Calibration of a Camera and Laser Range Finder"
#[derive(Debug, Clone)]
pub struct ExtrinsicCalibration {
    /// Rotation matrix (3×3) from LiDAR frame to camera frame.
    pub rotation: Matrix,
    /// Translation vector (3-D) from LiDAR frame to camera frame, in meters.
    pub translation: Vector,
}

impl ExtrinsicCalibration {
    /// Create a new extrinsic calibration.
    ///
    /// `rotation` is the 3×3 LiDAR→camera rotation; `translation` is the 3-D
    /// LiDAR→camera translation in meters.
    pub fn new(rotation: Matrix, translation: Vector) -> Self {
        Self {
            rotation,
            translation,
        }
    }

    /// Identity calibration (LiDAR and camera frames coincide).
    pub fn identity() -> Self {
        Self {
            rotation: Matrix::identity(3),
            translation: Vector::new(vec![0.0, 0.0, 0.0]),
        }
    }

    /// Transform a 3D point from LiDAR frame to camera frame.
    ///
    /// `point` is expressed in the LiDAR frame; the returned point is in the
    /// camera frame (R·p + t).
    pub fn transform_point(&self, point: &Vector) -> Vector {
        self.rotation.multiply_vector(point).add(&self.translation)
    }
}

/// Intrinsic camera parameters (pinhole model).
#[derive(Debug, Clone)]
pub struct CameraIntrinsics {
    /// Focal length in x (pixels).
    pub fx: f64,
    /// Focal length in y (pixels).
    pub fy: f64,
    /// Principal point x (pixels).
    pub cx: f64,
    /// Principal point y (pixels).
    pub cy: f64,
}

impl CameraIntrinsics {
    /// Project a 3D point (in camera frame) to 2D pixel coordinates.
    ///
    /// `point` is expressed in the camera frame; the returned 2-D vector holds
    /// the pixel coordinates (u, v). Returns None if the point is behind the
    /// camera (z <= 0).
    pub fn project(&self, point: &Vector) -> Option<Vector> {
        if point.get(2) <= 0.0 {
            return None;
        }
        Some(Vector::new(vec![
            self.fx * point.get(0) / point.get(2) + self.cx,
            self.fy * point.get(1) / point.get(2) + self.cy,
        ]))
    }
}
