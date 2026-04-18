#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// A point in 2D Euclidean space.
///
/// Hilbert's primitive notion: points are undefined objects
/// satisfying the axioms of incidence, order, and congruence.
#[derive(Debug, Clone, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

/// A point in 3D Euclidean space.
#[derive(Debug, Clone, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn origin() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Euclidean distance (metric axiom d(a,b)).
    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Hilbert's betweenness: is b between self and c?
    /// A * B * C iff d(A,B) + d(B,C) = d(A,C) and B != A and B != C.
    pub fn is_between(&self, b: &Self, c: &Self) -> bool {
        let ab = self.distance_to(b);
        let bc = b.distance_to(c);
        let ac = self.distance_to(c);
        (ab + bc - ac).abs() < 1e-10 && ab > 1e-15 && bc > 1e-15
    }

    /// Midpoint (congruence: divides segment into two congruent parts).
    pub fn midpoint(&self, other: &Self) -> Self {
        Self {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
        }
    }

    /// Vector from self to other.
    pub fn vector_to(&self, other: &Self) -> crate::formal::math::geometry::vector::Vec2 {
        crate::formal::math::geometry::vector::Vec2 {
            x: other.x - self.x,
            y: other.y - self.y,
        }
    }

    /// Translate by a vector.
    pub fn translate(&self, v: &crate::formal::math::geometry::vector::Vec2) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }

    /// Collinearity test for three points (Hilbert I.1: two points determine a line).
    pub fn collinear(a: &Self, b: &Self, c: &Self) -> bool {
        let ab = a.vector_to(b);
        let ac = a.vector_to(c);
        ab.cross(&ac).abs() < 1e-10
    }
}

impl Point3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn origin() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Euclidean distance.
    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Betweenness: is b between self and c?
    pub fn is_between(&self, b: &Self, c: &Self) -> bool {
        let ab = self.distance_to(b);
        let bc = b.distance_to(c);
        let ac = self.distance_to(c);
        (ab + bc - ac).abs() < 1e-10 && ab > 1e-15 && bc > 1e-15
    }

    /// Midpoint.
    pub fn midpoint(&self, other: &Self) -> Self {
        Self {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
            z: (self.z + other.z) / 2.0,
        }
    }

    /// Vector from self to other.
    pub fn vector_to(&self, other: &Self) -> crate::formal::math::geometry::vector::Vec3 {
        crate::formal::math::geometry::vector::Vec3 {
            x: other.x - self.x,
            y: other.y - self.y,
            z: other.z - self.z,
        }
    }

    /// Translate by a vector.
    pub fn translate(&self, v: &crate::formal::math::geometry::vector::Vec3) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
        }
    }

    /// Collinearity: three points on the same line (cross product = 0).
    pub fn collinear(a: &Self, b: &Self, c: &Self) -> bool {
        let ab = a.vector_to(b);
        let ac = a.vector_to(c);
        ab.cross(&ac).norm() < 1e-10
    }

    /// Coplanarity: four points in the same plane (scalar triple product = 0).
    pub fn coplanar(a: &Self, b: &Self, c: &Self, d: &Self) -> bool {
        let ab = a.vector_to(b);
        let ac = a.vector_to(c);
        let ad = a.vector_to(d);
        ab.triple_product(&ac, &ad).abs() < 1e-10
    }
}
