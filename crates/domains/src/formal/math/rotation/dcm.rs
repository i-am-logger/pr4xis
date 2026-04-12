use crate::formal::math::rotation::quaternion::Quaternion;

/// 3x3 Direction Cosine Matrix — element of SO(3).
///
/// A proper orthogonal matrix: R^T R = I, det(R) = +1.
#[derive(Debug, Clone)]
pub struct Dcm {
    pub m: [[f64; 3]; 3],
}

impl Dcm {
    /// Identity DCM.
    pub fn identity() -> Self {
        Self {
            m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// Construct from a 3x3 array.
    pub fn from_array(m: [[f64; 3]; 3]) -> Self {
        Self { m }
    }

    /// Transpose (which is the inverse for orthogonal matrices).
    pub fn transpose(&self) -> Self {
        Self {
            m: [
                [self.m[0][0], self.m[1][0], self.m[2][0]],
                [self.m[0][1], self.m[1][1], self.m[2][1]],
                [self.m[0][2], self.m[1][2], self.m[2][2]],
            ],
        }
    }

    /// Group inverse (transpose for SO(3)).
    pub fn inverse(&self) -> Self {
        self.transpose()
    }

    /// Matrix multiply: self * other.
    pub fn multiply(&self, other: &Self) -> Self {
        let mut result = [[0.0; 3]; 3];
        for (i, row) in result.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                for k in 0..3 {
                    *cell += self.m[i][k] * other.m[k][j];
                }
            }
        }
        Self { m: result }
    }

    /// Group operation: apply self first, then other.
    pub fn compose(&self, other: &Self) -> Self {
        other.multiply(self)
    }

    /// Rotate a vector: v' = R * v.
    pub fn rotate_vector(&self, v: [f64; 3]) -> [f64; 3] {
        [
            self.m[0][0] * v[0] + self.m[0][1] * v[1] + self.m[0][2] * v[2],
            self.m[1][0] * v[0] + self.m[1][1] * v[1] + self.m[1][2] * v[2],
            self.m[2][0] * v[0] + self.m[2][1] * v[1] + self.m[2][2] * v[2],
        ]
    }

    /// Determinant.
    pub fn determinant(&self) -> f64 {
        let m = &self.m;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }

    /// Check orthogonality: R^T R ≈ I within tolerance.
    pub fn is_orthogonal(&self, tol: f64) -> bool {
        let rtr = self.transpose().multiply(self);
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                if (rtr.m[i][j] - expected).abs() > tol {
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
        Quaternion::from_dcm(&self.m)
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
    /// Create from DCM (Shepperd's method). Result is normalized.
    pub fn from_dcm(r: &[[f64; 3]; 3]) -> Self {
        let trace = r[0][0] + r[1][1] + r[2][2];
        let raw = if trace > 0.0 {
            let s = 0.5 / (trace + 1.0).sqrt();
            (
                0.25 / s,
                (r[2][1] - r[1][2]) * s,
                (r[0][2] - r[2][0]) * s,
                (r[1][0] - r[0][1]) * s,
            )
        } else if r[0][0] > r[1][1] && r[0][0] > r[2][2] {
            let s = 2.0 * (1.0 + r[0][0] - r[1][1] - r[2][2]).sqrt();
            (
                (r[2][1] - r[1][2]) / s,
                0.25 * s,
                (r[0][1] + r[1][0]) / s,
                (r[0][2] + r[2][0]) / s,
            )
        } else if r[1][1] > r[2][2] {
            let s = 2.0 * (1.0 + r[1][1] - r[0][0] - r[2][2]).sqrt();
            (
                (r[0][2] - r[2][0]) / s,
                (r[0][1] + r[1][0]) / s,
                0.25 * s,
                (r[1][2] + r[2][1]) / s,
            )
        } else {
            let s = 2.0 * (1.0 + r[2][2] - r[0][0] - r[1][1]).sqrt();
            (
                (r[1][0] - r[0][1]) / s,
                (r[0][2] + r[2][0]) / s,
                (r[1][2] + r[2][1]) / s,
                0.25 * s,
            )
        };
        Self::new(raw.0, raw.1, raw.2, raw.3)
    }
}
