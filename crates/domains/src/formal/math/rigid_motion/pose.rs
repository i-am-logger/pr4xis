#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::rotation::quaternion::Quaternion;

/// Rigid body transformation in SE(3): rotation + translation.
///
/// Transforms a point from source frame to target frame:
/// `p_target = R * p_source + t`. The translation is a typed [`Vector`], not a
/// raw `[f64; 3]`.
#[derive(Debug, Clone)]
pub struct Pose {
    pub rotation: Quaternion,
    /// Translation vector (3-D).
    pub translation: Vector,
}

impl Pose {
    /// Identity pose (no rotation, no translation).
    pub fn identity() -> Self {
        Self {
            rotation: Quaternion::identity(),
            translation: Vector::zeros(3),
        }
    }

    /// From rotation only (zero translation).
    pub fn from_rotation(rotation: Quaternion) -> Self {
        Self {
            rotation,
            translation: Vector::zeros(3),
        }
    }

    /// From translation only (identity rotation).
    pub fn from_translation(translation: Vector) -> Self {
        Self {
            rotation: Quaternion::identity(),
            translation,
        }
    }

    /// Group operation: self (A->B) followed by other (B->C) = result (A->C).
    ///
    /// `R_AC = R_BC * R_AB`, `t_AC = R_BC * t_AB + t_BC`.
    pub fn compose(&self, other: &Self) -> Self {
        let rotated_t = other.rotation.rotate_vector(&self.translation);
        Self {
            rotation: self.rotation.compose(&other.rotation),
            translation: rotated_t.add(&other.translation),
        }
    }

    /// Group inverse: if self is A->B, result is B->A.
    ///
    /// `T^{-1} = (R^T, -R^T * t)`.
    pub fn inverse(&self) -> Self {
        let r_inv = self.rotation.inverse();
        let t_inv = r_inv.rotate_vector(&self.translation);
        Self {
            rotation: r_inv,
            translation: t_inv.scale(-1.0),
        }
    }

    /// Transform a point from source to target frame.
    pub fn transform_point(&self, point: &Vector) -> Vector {
        self.rotation.rotate_vector(point).add(&self.translation)
    }

    /// 4x4 homogeneous transformation [`Matrix`].
    pub fn to_homogeneous(&self) -> Matrix {
        let r = self.rotation.to_dcm();
        let t = &self.translation;
        Matrix::new(
            4,
            4,
            vec![
                r.get(0, 0),
                r.get(0, 1),
                r.get(0, 2),
                t.get(0), //
                r.get(1, 0),
                r.get(1, 1),
                r.get(1, 2),
                t.get(1), //
                r.get(2, 0),
                r.get(2, 1),
                r.get(2, 2),
                t.get(2), //
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        )
    }
}

impl PartialEq for Pose {
    fn eq(&self, other: &Self) -> bool {
        const TOL: f64 = 1e-9;
        self.rotation == other.rotation
            && self.translation.dim() == other.translation.dim()
            && (0..self.translation.dim())
                .all(|i| (self.translation.get(i) - other.translation.get(i)).abs() < TOL)
    }
}

impl Eq for Pose {}
