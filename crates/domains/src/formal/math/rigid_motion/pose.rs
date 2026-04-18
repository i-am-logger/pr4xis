#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::rotation::quaternion::Quaternion;

/// Rigid body transformation in SE(3): rotation + translation.
///
/// Transforms a point from source frame to target frame:
/// p_target = R * p_source + t
#[derive(Debug, Clone)]
pub struct Pose {
    pub rotation: Quaternion,
    pub translation: [f64; 3],
}

impl Pose {
    /// Identity pose (no rotation, no translation).
    pub fn identity() -> Self {
        Self {
            rotation: Quaternion::identity(),
            translation: [0.0; 3],
        }
    }

    /// From rotation only (zero translation).
    pub fn from_rotation(rotation: Quaternion) -> Self {
        Self {
            rotation,
            translation: [0.0; 3],
        }
    }

    /// From translation only (identity rotation).
    pub fn from_translation(translation: [f64; 3]) -> Self {
        Self {
            rotation: Quaternion::identity(),
            translation,
        }
    }

    /// Group operation: self (A->B) followed by other (B->C) = result (A->C).
    ///
    /// R_AC = R_BC * R_AB
    /// t_AC = R_BC * t_AB + t_BC
    pub fn compose(&self, other: &Self) -> Self {
        let rotated_t = other.rotation.rotate_vector(self.translation);
        Self {
            rotation: self.rotation.compose(&other.rotation),
            translation: [
                rotated_t[0] + other.translation[0],
                rotated_t[1] + other.translation[1],
                rotated_t[2] + other.translation[2],
            ],
        }
    }

    /// Group inverse: if self is A->B, result is B->A.
    ///
    /// T^{-1} = (R^T, -R^T * t)
    pub fn inverse(&self) -> Self {
        let r_inv = self.rotation.inverse();
        let t_inv = r_inv.rotate_vector(self.translation);
        Self {
            rotation: r_inv,
            translation: [-t_inv[0], -t_inv[1], -t_inv[2]],
        }
    }

    /// Transform a point from source to target frame.
    pub fn transform_point(&self, point: [f64; 3]) -> [f64; 3] {
        let rotated = self.rotation.rotate_vector(point);
        [
            rotated[0] + self.translation[0],
            rotated[1] + self.translation[1],
            rotated[2] + self.translation[2],
        ]
    }

    /// 4x4 homogeneous transformation matrix.
    pub fn to_homogeneous(&self) -> [[f64; 4]; 4] {
        let r = self.rotation.to_dcm();
        let t = self.translation;
        [
            [r[0][0], r[0][1], r[0][2], t[0]],
            [r[1][0], r[1][1], r[1][2], t[1]],
            [r[2][0], r[2][1], r[2][2], t[2]],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}

impl PartialEq for Pose {
    fn eq(&self, other: &Self) -> bool {
        const TOL: f64 = 1e-9;
        self.rotation == other.rotation
            && (self.translation[0] - other.translation[0]).abs() < TOL
            && (self.translation[1] - other.translation[1]).abs() < TOL
            && (self.translation[2] - other.translation[2]).abs() < TOL
    }
}

impl Eq for Pose {}
