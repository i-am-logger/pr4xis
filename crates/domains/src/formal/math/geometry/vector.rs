/// Geometric vector in 2D Euclidean space.
///
/// Satisfies vector space axioms (associativity, commutativity,
/// identity, inverse, scalar compatibility, distributivity).
#[derive(Debug, Clone, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

/// Geometric vector in 3D Euclidean space.
#[derive(Debug, Clone, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Euclidean norm (L2).
    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Squared norm.
    pub fn norm_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Unit vector. Returns None for zero vector.
    pub fn normalize(&self) -> Option<Self> {
        let n = self.norm();
        if n < 1e-15 {
            None
        } else {
            Some(Self {
                x: self.x / n,
                y: self.y / n,
            })
        }
    }

    /// Inner product: a · b = |a||b|cos(θ).
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// 2D cross product (z-component of 3D cross product).
    /// Positive if other is counterclockwise from self.
    pub fn cross(&self, other: &Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// Vector addition (axiom: commutativity, associativity).
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    /// Vector subtraction.
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    /// Scalar multiplication (axiom: compatibility, distributivity).
    pub fn scale(&self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
        }
    }

    /// Additive inverse (axiom 4).
    pub fn negate(&self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }

    /// Angle between vectors in [0, π].
    pub fn angle_to(&self, other: &Self) -> f64 {
        let d = self.dot(other) / (self.norm() * other.norm());
        d.clamp(-1.0, 1.0).acos()
    }

    /// Perpendicular (90° counterclockwise).
    pub fn perpendicular(&self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn unit_x() -> Self {
        Self::new(1.0, 0.0, 0.0)
    }
    pub fn unit_y() -> Self {
        Self::new(0.0, 1.0, 0.0)
    }
    pub fn unit_z() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }

    /// Euclidean norm.
    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Squared norm.
    pub fn norm_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Unit vector. Returns None for zero vector.
    pub fn normalize(&self) -> Option<Self> {
        let n = self.norm();
        if n < 1e-15 {
            None
        } else {
            Some(Self {
                x: self.x / n,
                y: self.y / n,
                z: self.z / n,
            })
        }
    }

    /// Inner product.
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product: a × b, perpendicular to both (right-hand rule).
    /// |a × b| = |a||b|sin(θ).
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Vector addition.
    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    /// Vector subtraction.
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    /// Scalar multiplication.
    pub fn scale(&self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    /// Additive inverse.
    pub fn negate(&self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    /// Angle between vectors in [0, π].
    pub fn angle_to(&self, other: &Self) -> f64 {
        let d = self.dot(other) / (self.norm() * other.norm());
        d.clamp(-1.0, 1.0).acos()
    }

    /// Scalar triple product: a · (b × c) = volume of parallelepiped.
    pub fn triple_product(&self, b: &Self, c: &Self) -> f64 {
        self.dot(&b.cross(c))
    }

    /// Parallel test (cross product ≈ zero).
    pub fn is_parallel_to(&self, other: &Self) -> bool {
        self.cross(other).norm() < 1e-10
    }

    /// Perpendicular test (dot product ≈ zero).
    pub fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.dot(other).abs() < 1e-10
    }
}
