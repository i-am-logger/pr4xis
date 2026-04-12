/// Extrinsic calibration parameters for LiDAR-camera fusion.
///
/// Represents the rigid body transformation from LiDAR frame to camera frame.
///
/// Source: Zhang & Pless (2004), "Extrinsic Calibration of a Camera and Laser Range Finder"
#[derive(Debug, Clone)]
pub struct ExtrinsicCalibration {
    /// Rotation matrix (3x3, row-major) from LiDAR to camera frame.
    pub rotation: [f64; 9],
    /// Translation vector (3,) from LiDAR to camera frame, in meters.
    pub translation: [f64; 3],
}

impl ExtrinsicCalibration {
    /// Create a new extrinsic calibration.
    pub fn new(rotation: [f64; 9], translation: [f64; 3]) -> Self {
        Self {
            rotation,
            translation,
        }
    }

    /// Identity calibration (LiDAR and camera frames coincide).
    pub fn identity() -> Self {
        Self {
            rotation: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
            translation: [0.0, 0.0, 0.0],
        }
    }

    /// Transform a 3D point from LiDAR frame to camera frame.
    pub fn transform_point(&self, point: [f64; 3]) -> [f64; 3] {
        let r = &self.rotation;
        let t = &self.translation;
        [
            r[0] * point[0] + r[1] * point[1] + r[2] * point[2] + t[0],
            r[3] * point[0] + r[4] * point[1] + r[5] * point[2] + t[1],
            r[6] * point[0] + r[7] * point[1] + r[8] * point[2] + t[2],
        ]
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
    /// Returns None if the point is behind the camera (z <= 0).
    pub fn project(&self, point: [f64; 3]) -> Option<[f64; 2]> {
        if point[2] <= 0.0 {
            return None;
        }
        Some([
            self.fx * point[0] / point[2] + self.cx,
            self.fy * point[1] / point[2] + self.cy,
        ])
    }
}
