use crate::applied::sensor_fusion::frame::reference::ReferenceFrame;
use crate::formal::math::rotation::quaternion::Quaternion;

/// Angular misalignment between sensor measurement axes.
///
/// Boresight error is the small-angle rotation that aligns one sensor's
/// measurement axis with another's. It arises from imperfect mechanical
/// mounting and is calibrated during installation.
///
/// The misalignment is represented as a unit quaternion (small rotation).
/// For well-calibrated systems, the rotation angle is typically < 1 degree.
///
/// Source: Groves (2013), Section 14.2.3 — "Boresight alignment calibration."
///         Bar-Shalom et al. (2001), Section 6.7 — "Sensor alignment."
#[derive(Debug, Clone)]
pub struct Boresight {
    /// The sensor frame we are transforming FROM.
    pub from_sensor: ReferenceFrame,
    /// The sensor frame we are transforming TO.
    pub to_sensor: ReferenceFrame,
    /// The angular misalignment as a unit quaternion.
    pub misalignment: Quaternion,
    /// Calibration quality indicator (0.0 = uncalibrated, 1.0 = perfect).
    pub calibration_quality: f64,
}

impl Boresight {
    /// Create a new boresight calibration.
    pub fn new(
        from_sensor: ReferenceFrame,
        to_sensor: ReferenceFrame,
        misalignment: Quaternion,
        calibration_quality: f64,
    ) -> Self {
        Self {
            from_sensor,
            to_sensor,
            misalignment,
            calibration_quality: calibration_quality.clamp(0.0, 1.0),
        }
    }

    /// Create a zero (identity) boresight — sensors are perfectly aligned.
    pub fn identity(from_sensor: ReferenceFrame, to_sensor: ReferenceFrame) -> Self {
        Self {
            from_sensor,
            to_sensor,
            misalignment: Quaternion::identity(),
            calibration_quality: 1.0,
        }
    }

    /// Apply boresight correction to a measurement vector expressed in `from_sensor`.
    /// Returns the vector expressed in `to_sensor`.
    pub fn apply(&self, v: [f64; 3]) -> [f64; 3] {
        self.misalignment.rotate_vector(v)
    }

    /// Inverse boresight: swap from/to and invert the rotation.
    pub fn inverse(&self) -> Self {
        Self {
            from_sensor: self.to_sensor,
            to_sensor: self.from_sensor,
            misalignment: self.misalignment.inverse(),
            calibration_quality: self.calibration_quality,
        }
    }

    /// Compose two boresights: self followed by other.
    /// self: A -> B, other: B -> C, result: A -> C.
    /// Returns None if the frames don't chain (self.to != other.from).
    pub fn compose(&self, other: &Self) -> Option<Self> {
        if self.to_sensor != other.from_sensor {
            return None;
        }
        Some(Self {
            from_sensor: self.from_sensor,
            to_sensor: other.to_sensor,
            misalignment: self.misalignment.compose(&other.misalignment),
            calibration_quality: self.calibration_quality.min(other.calibration_quality),
        })
    }

    /// Magnitude of the boresight error in radians.
    pub fn magnitude(&self) -> f64 {
        let (_, angle) = self.misalignment.to_axis_angle();
        angle
    }

    /// Is this boresight within tolerance (radians)?
    pub fn is_within_tolerance(&self, tolerance_rad: f64) -> bool {
        self.magnitude() <= tolerance_rad
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_boresight_has_zero_magnitude() {
        let b = Boresight::identity(ReferenceFrame::IMU, ReferenceFrame::Body);
        assert!(b.magnitude() < 1e-10);
    }

    #[test]
    fn apply_identity_preserves_vector() {
        let b = Boresight::identity(ReferenceFrame::IMU, ReferenceFrame::Body);
        let v = [1.0, 2.0, 3.0];
        let result = b.apply(v);
        for i in 0..3 {
            assert!((result[i] - v[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn inverse_compose_gives_identity() {
        let q = Quaternion::from_axis_angle([0.0, 0.0, 1.0], 0.01); // 0.01 rad ~ 0.57 deg
        let b = Boresight::new(ReferenceFrame::IMU, ReferenceFrame::Body, q, 0.9);
        let b_inv = b.inverse();
        let composed = b.compose(&b_inv).unwrap();
        assert!(composed.magnitude() < 1e-10);
    }

    #[test]
    fn compose_requires_matching_frames() {
        let b1 = Boresight::identity(ReferenceFrame::IMU, ReferenceFrame::Body);
        let b2 = Boresight::identity(ReferenceFrame::Camera, ReferenceFrame::LiDAR);
        assert!(b1.compose(&b2).is_none());
    }

    #[test]
    fn magnitude_matches_axis_angle() {
        let angle = 0.05; // ~2.86 degrees
        let q = Quaternion::from_axis_angle([1.0, 0.0, 0.0], angle);
        let b = Boresight::new(ReferenceFrame::IMU, ReferenceFrame::Body, q, 0.8);
        assert!((b.magnitude() - angle).abs() < 1e-10);
    }

    #[test]
    fn calibration_quality_clamped() {
        let b = Boresight::new(
            ReferenceFrame::IMU,
            ReferenceFrame::Body,
            Quaternion::identity(),
            1.5,
        );
        assert!((b.calibration_quality - 1.0).abs() < 1e-10);
    }
}
