#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::rotation::quaternion::Quaternion;

/// 3x3 Direction Cosine Matrix — element of SO(3).
///
/// A proper orthogonal matrix: `RᵀR = I`, `det(R) = +1`. Backed by a 3×3
/// [`Matrix`], not a raw `[[f64; 3]; 3]`.
#[derive(Debug, Clone)]
pub struct Dcm {
    pub m: Matrix,
}

impl Dcm {
    /// Identity DCM.
    pub fn identity() -> Self {
        Self {
            m: Matrix::identity(3),
        }
    }

    /// Construct from a 3×3 [`Matrix`].
    pub fn from_matrix(m: Matrix) -> Self {
        Self { m }
    }

    /// Transpose (which is the inverse for orthogonal matrices).
    pub fn transpose(&self) -> Self {
        Self {
            m: self.m.transpose(),
        }
    }

    /// Group inverse (transpose for SO(3)).
    pub fn inverse(&self) -> Self {
        self.transpose()
    }

    /// Matrix multiply: self * other.
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            m: self.m.multiply(&other.m),
        }
    }

    /// Group operation: apply self first, then other.
    pub fn compose(&self, other: &Self) -> Self {
        other.multiply(self)
    }

    /// Rotate a vector: `v' = R · v`.
    pub fn rotate_vector(&self, v: &Vector) -> Vector {
        self.m.multiply_vector(v)
    }

    /// Determinant (3×3).
    pub fn determinant(&self) -> f64 {
        let m = &self.m;
        m.get(0, 0) * (m.get(1, 1) * m.get(2, 2) - m.get(1, 2) * m.get(2, 1))
            - m.get(0, 1) * (m.get(1, 0) * m.get(2, 2) - m.get(1, 2) * m.get(2, 0))
            + m.get(0, 2) * (m.get(1, 0) * m.get(2, 1) - m.get(1, 1) * m.get(2, 0))
    }

    /// Check orthogonality: `RᵀR ≈ I` within tolerance.
    pub fn is_orthogonal(&self, tol: f64) -> bool {
        let rtr = self.transpose().multiply(self);
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                if (rtr.m.get(i, j) - expected).abs() > tol {
                    return false;
                }
            }
        }
        true
    }

    /// Check proper rotation: orthogonal AND det = +1.
    pub fn is_proper_rotation(&self, tol: f64) -> bool {
        self.is_orthogonal(tol) && (self.determinant() - 1.0).abs() < tol
    }

    /// Convert to quaternion.
    pub fn to_quaternion(&self) -> Quaternion {
        Quaternion::from_matrix(&self.m)
    }

    /// Create from quaternion.
    pub fn from_quaternion(q: &Quaternion) -> Self {
        Self { m: q.to_dcm() }
    }
}

/// Create DCM from quaternion.
impl From<&Quaternion> for Dcm {
    fn from(q: &Quaternion) -> Self {
        Self::from_quaternion(q)
    }
}

impl Quaternion {
    /// Create from a 3×3 DCM [`Matrix`] (Shepperd's method). Result is normalized.
    pub fn from_matrix(r: &Matrix) -> Self {
        let (r00, r01, r02) = (r.get(0, 0), r.get(0, 1), r.get(0, 2));
        let (r10, r11, r12) = (r.get(1, 0), r.get(1, 1), r.get(1, 2));
        let (r20, r21, r22) = (r.get(2, 0), r.get(2, 1), r.get(2, 2));
        let trace = r00 + r11 + r22;
        let raw = if trace > 0.0 {
            let s = 0.5 / (trace + 1.0).sqrt();
            (0.25 / s, (r21 - r12) * s, (r02 - r20) * s, (r10 - r01) * s)
        } else if r00 > r11 && r00 > r22 {
            let s = 2.0 * (1.0 + r00 - r11 - r22).sqrt();
            ((r21 - r12) / s, 0.25 * s, (r01 + r10) / s, (r02 + r20) / s)
        } else if r11 > r22 {
            let s = 2.0 * (1.0 + r11 - r00 - r22).sqrt();
            ((r02 - r20) / s, (r01 + r10) / s, 0.25 * s, (r12 + r21) / s)
        } else {
            let s = 2.0 * (1.0 + r22 - r00 - r11).sqrt();
            ((r10 - r01) / s, (r02 + r20) / s, (r12 + r21) / s, 0.25 * s)
        };
        Self::new(raw.0, raw.1, raw.2, raw.3)
    }
}
