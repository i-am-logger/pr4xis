#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::sensor_fusion::frame::reference::ReferenceFrame;

/// Physical offset between sensor reference points.
///
/// A lever arm is the 3D translation vector from one sensor's reference point
/// to another's, expressed in a given coordinate frame. This is distinct from
/// boresight (which is rotational alignment).
///
/// Lever arms are measured during installation and are critical for
/// multi-sensor fusion — a 10 cm lever arm error at 100 m/s introduces
/// ~0.1 m position error per second.
///
/// Source: Groves (2013), Section 14.2.1 — "Lever arm compensation."
///         Titterton & Weston (2004), Section 13.3 — "Installation errors."
#[derive(Debug, Clone, PartialEq)]
pub struct LeverArm {
    /// The sensor reference point we are measuring FROM.
    pub from_sensor: ReferenceFrame,
    /// The sensor reference point we are measuring TO.
    pub to_sensor: ReferenceFrame,
    /// Offset vector [x, y, z] in meters.
    pub offset: [f64; 3],
    /// The frame in which the offset vector is expressed.
    pub expressed_in: ReferenceFrame,
}

impl LeverArm {
    /// Create a new lever arm.
    pub fn new(
        from_sensor: ReferenceFrame,
        to_sensor: ReferenceFrame,
        offset: [f64; 3],
        expressed_in: ReferenceFrame,
    ) -> Self {
        Self {
            from_sensor,
            to_sensor,
            offset,
            expressed_in,
        }
    }

    /// Zero lever arm — sensors are co-located.
    pub fn zero(from_sensor: ReferenceFrame, to_sensor: ReferenceFrame) -> Self {
        Self {
            from_sensor,
            to_sensor,
            offset: [0.0, 0.0, 0.0],
            expressed_in: from_sensor,
        }
    }

    /// Inverse lever arm: swap from/to and negate offset.
    ///
    /// If the offset from A to B is [x, y, z], then from B to A is [-x, -y, -z]
    /// (in the same expression frame).
    pub fn inverse(&self) -> Self {
        Self {
            from_sensor: self.to_sensor,
            to_sensor: self.from_sensor,
            offset: [-self.offset[0], -self.offset[1], -self.offset[2]],
            expressed_in: self.expressed_in,
        }
    }

    /// Magnitude of the lever arm in meters (Euclidean norm).
    pub fn magnitude(&self) -> f64 {
        let [x, y, z] = self.offset;
        (x * x + y * y + z * z).sqrt()
    }

    /// Velocity correction for a rotating body.
    ///
    /// If the body rotates with angular velocity omega (rad/s),
    /// the lever arm induces a velocity correction: v_corr = omega x lever_arm.
    ///
    /// Source: Groves (2013), Eq. 14.14.
    pub fn velocity_correction(&self, omega: [f64; 3]) -> [f64; 3] {
        let [ox, oy, oz] = omega;
        let [lx, ly, lz] = self.offset;
        [oy * lz - oz * ly, oz * lx - ox * lz, ox * ly - oy * lx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_lever_arm_has_zero_magnitude() {
        let la = LeverArm::zero(ReferenceFrame::IMU, ReferenceFrame::GNSS);
        assert!(la.magnitude() < 1e-10);
    }

    #[test]
    fn inverse_negates_offset() {
        let la = LeverArm::new(
            ReferenceFrame::IMU,
            ReferenceFrame::GNSS,
            [1.0, 2.0, 3.0],
            ReferenceFrame::Body,
        );
        let inv = la.inverse();
        assert_eq!(inv.from_sensor, ReferenceFrame::GNSS);
        assert_eq!(inv.to_sensor, ReferenceFrame::IMU);
        for i in 0..3 {
            assert!((inv.offset[i] + la.offset[i]).abs() < 1e-10);
        }
    }

    #[test]
    fn magnitude_is_euclidean_norm() {
        let la = LeverArm::new(
            ReferenceFrame::IMU,
            ReferenceFrame::GNSS,
            [3.0, 4.0, 0.0],
            ReferenceFrame::Body,
        );
        assert!((la.magnitude() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn inverse_inverse_is_original() {
        let la = LeverArm::new(
            ReferenceFrame::IMU,
            ReferenceFrame::GNSS,
            [1.5, -0.3, 0.7],
            ReferenceFrame::Body,
        );
        let la2 = la.inverse().inverse();
        assert_eq!(la, la2);
    }

    #[test]
    fn velocity_correction_cross_product() {
        // omega = [0, 0, 1] rad/s, lever_arm = [1, 0, 0] m
        // v_corr = omega x lever = [0, 0, 1] x [1, 0, 0] = [0, 1, 0]
        let la = LeverArm::new(
            ReferenceFrame::IMU,
            ReferenceFrame::GNSS,
            [1.0, 0.0, 0.0],
            ReferenceFrame::Body,
        );
        let v = la.velocity_correction([0.0, 0.0, 1.0]);
        assert!((v[0]).abs() < 1e-10);
        assert!((v[1] - 1.0).abs() < 1e-10);
        assert!((v[2]).abs() < 1e-10);
    }
}
