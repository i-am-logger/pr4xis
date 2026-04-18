#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::rotation::quaternion::Quaternion;

/// Axis-angle representation of a rotation.
///
/// A rotation of `angle` radians about a unit vector `axis`.
/// This is the exponential map from so(3) to SO(3).
#[derive(Debug, Clone, PartialEq)]
pub struct AxisAngle {
    /// Unit rotation axis.
    pub axis: [f64; 3],
    /// Rotation angle in radians.
    pub angle: f64,
}

impl AxisAngle {
    /// Identity rotation (zero angle about arbitrary axis).
    pub fn identity() -> Self {
        Self {
            axis: [0.0, 0.0, 1.0],
            angle: 0.0,
        }
    }

    /// Construct and normalize the axis to unit length.
    pub fn new(axis: [f64; 3], angle: f64) -> Self {
        let n = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        if n < 1e-12 {
            Self::identity()
        } else {
            Self {
                axis: [axis[0] / n, axis[1] / n, axis[2] / n],
                angle,
            }
        }
    }

    /// Convert to quaternion.
    pub fn to_quaternion(&self) -> Quaternion {
        Quaternion::from_axis_angle(self.axis, self.angle)
    }

    /// Create from quaternion.
    pub fn from_quaternion(q: &Quaternion) -> Self {
        let (axis, angle) = q.to_axis_angle();
        Self { axis, angle }
    }
}
