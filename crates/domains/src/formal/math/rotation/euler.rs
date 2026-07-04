#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::angle::Angle;
use crate::formal::math::linear_algebra::vector_space::Vector;
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
///
/// The three angles are typed [`Angle`]s (elements of the circle group), so
/// radians and degrees can never be confused and no scalar can stand in.
#[derive(Debug, Clone, PartialEq)]
pub struct EulerAngles {
    pub first: Angle,
    pub second: Angle,
    pub third: Angle,
    pub sequence: EulerSequence,
}

impl EulerAngles {
    /// Construct from three angles given in **radians**.
    pub fn from_radians(first: f64, second: f64, third: f64, sequence: EulerSequence) -> Self {
        Self {
            first: Angle::from_radians(first),
            second: Angle::from_radians(second),
            third: Angle::from_radians(third),
            sequence,
        }
    }

    /// Construct from three angles given in **degrees**.
    pub fn from_degrees(first: f64, second: f64, third: f64, sequence: EulerSequence) -> Self {
        Self {
            first: Angle::from_degrees(first),
            second: Angle::from_degrees(second),
            third: Angle::from_degrees(third),
            sequence,
        }
    }

    /// Convert ZYX Euler angles (yaw, pitch, roll) to quaternion.
    pub fn to_quaternion(&self) -> Quaternion {
        let (a, b, c) = (
            self.first.radians(),
            self.second.radians(),
            self.third.radians(),
        );
        match self.sequence {
            EulerSequence::ZYX => Quaternion::from_euler_321(a, b, c),
            EulerSequence::XYZ => {
                // R = Rz(third) * Ry(second) * Rx(first)
                let qx = Quaternion::from_axis_angle(&Vector::new(vec![1.0, 0.0, 0.0]), a);
                let qy = Quaternion::from_axis_angle(&Vector::new(vec![0.0, 1.0, 0.0]), b);
                let qz = Quaternion::from_axis_angle(&Vector::new(vec![0.0, 0.0, 1.0]), c);
                qx.compose(&qy).compose(&qz)
            }
            EulerSequence::ZXZ => {
                let q1 = Quaternion::from_axis_angle(&Vector::new(vec![0.0, 0.0, 1.0]), a);
                let q2 = Quaternion::from_axis_angle(&Vector::new(vec![1.0, 0.0, 0.0]), b);
                let q3 = Quaternion::from_axis_angle(&Vector::new(vec![0.0, 0.0, 1.0]), c);
                q1.compose(&q2).compose(&q3)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn zyx_euler_matches_direct_321() {
        let e = EulerAngles::from_radians(0.3, -0.2, 0.5, EulerSequence::ZYX);
        let direct = Quaternion::from_euler_321(0.3, -0.2, 0.5);
        assert!(e.to_quaternion() == direct);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn angles_are_typed_not_confusable() {
        // Degrees and radians are set by distinct constructors and never mixed.
        let e = EulerAngles::from_degrees(90.0, 0.0, 0.0, EulerSequence::XYZ);
        assert!((e.first.radians() - core::f64::consts::FRAC_PI_2).abs() < 1e-12);
        assert!((e.first.degrees() - 90.0).abs() < 1e-12);
    }
}
