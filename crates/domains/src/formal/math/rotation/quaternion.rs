use std::fmt;

/// Unit quaternion representing an element of SO(3).
///
/// q = w + xi + yj + zk where |q| = 1.
/// Convention: Hamilton, scalar-first [w, x, y, z].
///
/// Two quaternions q and -q represent the same rotation.
/// This is the double cover of SO(3) by SU(2).
#[derive(Clone)]
pub struct Quaternion {
    w: f64,
    x: f64,
    y: f64,
    z: f64,
}

impl fmt::Debug for Quaternion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Quaternion({:.6}, {:.6}, {:.6}, {:.6})",
            self.w, self.x, self.y, self.z
        )
    }
}

impl Quaternion {
    /// The identity element of the rotation group.
    pub fn identity() -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Construct from components. Normalizes to unit quaternion.
    /// Returns identity if the input is a zero quaternion.
    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        let n = (w * w + x * x + y * y + z * z).sqrt();
        if n < 1e-15 {
            return Self::identity();
        }
        Self {
            w: w / n,
            x: x / n,
            y: y / n,
            z: z / n,
        }
    }

    /// Construct from axis-angle. Normalizes the axis internally; `angle` in radians.
    pub fn from_axis_angle(axis: [f64; 3], angle: f64) -> Self {
        let an = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        let (ax, ay, az) = if an < 1e-15 {
            (0.0, 0.0, 1.0) // degenerate axis: treat as identity-axis
        } else {
            (axis[0] / an, axis[1] / an, axis[2] / an)
        };
        let half = angle / 2.0;
        let s = half.sin();
        Self {
            w: half.cos(),
            x: ax * s,
            y: ay * s,
            z: az * s,
        }
    }

    /// Construct from Euler angles (3-2-1 / ZYX: yaw, pitch, roll). Radians.
    pub fn from_euler_321(yaw: f64, pitch: f64, roll: f64) -> Self {
        let (sr, cr) = (roll / 2.0).sin_cos();
        let (sp, cp) = (pitch / 2.0).sin_cos();
        let (sy, cy) = (yaw / 2.0).sin_cos();
        Self {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }

    /// Squared norm.
    pub fn norm_squared(&self) -> f64 {
        self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Norm (1.0 for unit quaternion).
    pub fn norm(&self) -> f64 {
        self.norm_squared().sqrt()
    }

    /// Normalize to unit quaternion. Returns identity if the norm is near zero.
    pub fn normalize(&self) -> Self {
        let n = self.norm();
        if n < 1e-15 {
            return Self::identity();
        }
        Self {
            w: self.w / n,
            x: self.x / n,
            y: self.y / n,
            z: self.z / n,
        }
    }

    /// Conjugate: q* = w - xi - yj - zk. For unit quaternions, q* = q^{-1}.
    pub fn conjugate(&self) -> Self {
        Self {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    /// Group inverse. For unit quaternions this is the conjugate.
    pub fn inverse(&self) -> Self {
        self.conjugate()
    }

    /// Hamilton product: self * other.
    pub fn multiply(&self, other: &Self) -> Self {
        Self {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }

    /// Group operation: compose rotations. Apply self first, then other.
    /// R_result = R_other * R_self  =>  q_result = q_other * q_self.
    pub fn compose(&self, other: &Self) -> Self {
        other.multiply(self)
    }

    /// Rotate a vector: v' = q v q*.
    pub fn rotate_vector(&self, v: [f64; 3]) -> [f64; 3] {
        // Optimized Rodriguez formula avoids constructing pure quaternion
        let t = [
            2.0 * (self.y * v[2] - self.z * v[1]),
            2.0 * (self.z * v[0] - self.x * v[2]),
            2.0 * (self.x * v[1] - self.y * v[0]),
        ];
        [
            v[0] + self.w * t[0] + self.y * t[2] - self.z * t[1],
            v[1] + self.w * t[1] + self.z * t[0] - self.x * t[2],
            v[2] + self.w * t[2] + self.x * t[1] - self.y * t[0],
        ]
    }

    /// Convert to 3x3 Direction Cosine Matrix.
    pub fn to_dcm(&self) -> [[f64; 3]; 3] {
        let (w, x, y, z) = (self.w, self.x, self.y, self.z);
        [
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y - w * z),
                2.0 * (x * z + w * y),
            ],
            [
                2.0 * (x * y + w * z),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z - w * x),
            ],
            [
                2.0 * (x * z - w * y),
                2.0 * (y * z + w * x),
                1.0 - 2.0 * (x * x + y * y),
            ],
        ]
    }

    /// Extract axis-angle. Returns (unit axis, angle in radians).
    /// For identity rotation, returns ([0,0,1], 0.0).
    pub fn to_axis_angle(&self) -> ([f64; 3], f64) {
        let angle = 2.0 * self.w.clamp(-1.0, 1.0).acos();
        let s = (1.0 - self.w * self.w).sqrt();
        if s < 1e-12 {
            ([0.0, 0.0, 1.0], 0.0)
        } else {
            ([self.x / s, self.y / s, self.z / s], angle)
        }
    }
}

/// Rotation-equality: q and -q are the same rotation.
impl PartialEq for Quaternion {
    fn eq(&self, other: &Self) -> bool {
        const TOL: f64 = 1e-10;
        let dot = self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z;
        (dot.abs() - 1.0).abs() < TOL
    }
}

impl Eq for Quaternion {}

// ---------------------------------------------------------------------------
// Read-only accessors — the unit-norm invariant is enforced by keeping
// fields private and only constructing through normalizing constructors.
// ---------------------------------------------------------------------------

impl Quaternion {
    /// Scalar component.
    pub fn w(&self) -> f64 {
        self.w
    }

    /// Vector i-component.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Vector j-component.
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Vector k-component.
    pub fn z(&self) -> f64 {
        self.z
    }
}
