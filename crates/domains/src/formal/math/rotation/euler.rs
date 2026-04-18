#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::rotation::quaternion::Quaternion;

/// Euler angle sequence convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EulerSequence {
    /// ZYX (yaw-pitch-roll) — aerospace/navigation convention
    ZYX,
    /// XYZ — robotics convention
    XYZ,
    /// ZXZ — classical mechanics (precession-nutation-spin)
    ZXZ,
}

/// Euler angles with explicit sequence convention.
#[derive(Debug, Clone, PartialEq)]
pub struct EulerAngles {
    pub first: f64,
    pub second: f64,
    pub third: f64,
    pub sequence: EulerSequence,
}

impl EulerAngles {
    /// Convert ZYX Euler angles (yaw, pitch, roll) to quaternion.
    pub fn to_quaternion(&self) -> Quaternion {
        match self.sequence {
            EulerSequence::ZYX => Quaternion::from_euler_321(self.first, self.second, self.third),
            EulerSequence::XYZ => {
                // R = Rz(third) * Ry(second) * Rx(first)
                let qx = Quaternion::from_axis_angle([1.0, 0.0, 0.0], self.first);
                let qy = Quaternion::from_axis_angle([0.0, 1.0, 0.0], self.second);
                let qz = Quaternion::from_axis_angle([0.0, 0.0, 1.0], self.third);
                qx.compose(&qy).compose(&qz)
            }
            EulerSequence::ZXZ => {
                let q1 = Quaternion::from_axis_angle([0.0, 0.0, 1.0], self.first);
                let q2 = Quaternion::from_axis_angle([1.0, 0.0, 0.0], self.second);
                let q3 = Quaternion::from_axis_angle([0.0, 0.0, 1.0], self.third);
                q1.compose(&q2).compose(&q3)
            }
        }
    }
}
