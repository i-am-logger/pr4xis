#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::geometry::angle::Angle;
use crate::formal::math::geometry::point::Point3;
use core::f64::consts::PI;

/// Triangle defined by three vertices.
///
/// The triangle is the fundamental shape in Euclidean geometry.
/// Hilbert's axiom III.6 (SAS congruence) concerns triangles.
#[derive(Debug, Clone)]
pub struct Triangle {
    pub a: Point3,
    pub b: Point3,
    pub c: Point3,
}

/// Circle defined by center and radius.
#[derive(Debug, Clone)]
pub struct Circle {
    pub center: Point3,
    pub radius: f64,
}

/// Sphere defined by center and radius.
#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
}

impl Triangle {
    pub fn new(a: Point3, b: Point3, c: Point3) -> Self {
        Self { a, b, c }
    }

    /// Side lengths (a = BC, b = CA, c = AB — opposite vertex convention).
    pub fn side_lengths(&self) -> (f64, f64, f64) {
        (
            self.b.distance_to(&self.c),
            self.c.distance_to(&self.a),
            self.a.distance_to(&self.b),
        )
    }

    /// Perimeter.
    pub fn perimeter(&self) -> f64 {
        let (a, b, c) = self.side_lengths();
        a + b + c
    }

    /// Area via cross product: |AB × AC| / 2.
    pub fn area(&self) -> f64 {
        let ab = self.a.vector_to(&self.b);
        let ac = self.a.vector_to(&self.c);
        ab.cross(&ac).norm() / 2.0
    }

    /// Interior angles at each vertex using the law of cosines.
    /// Returns (angle_A, angle_B, angle_C).
    pub fn angles(&self) -> (Angle, Angle, Angle) {
        let (a, b, c) = self.side_lengths();
        let cos_a = (b * b + c * c - a * a) / (2.0 * b * c);
        let cos_b = (a * a + c * c - b * b) / (2.0 * a * c);
        let cos_c = (a * a + b * b - c * c) / (2.0 * a * b);
        (
            Angle::from_radians(cos_a.clamp(-1.0, 1.0).acos()),
            Angle::from_radians(cos_b.clamp(-1.0, 1.0).acos()),
            Angle::from_radians(cos_c.clamp(-1.0, 1.0).acos()),
        )
    }

    /// Sum of interior angles (Euclidean theorem: always π).
    pub fn angle_sum(&self) -> f64 {
        let (a, b, c) = self.angles();
        a.radians() + b.radians() + c.radians()
    }

    /// Is this a right triangle? (one angle = π/2).
    pub fn is_right(&self) -> bool {
        let (a, b, c) = self.angles();
        a.is_right() || b.is_right() || c.is_right()
    }

    /// Is the triangle degenerate? (zero area = collinear points).
    pub fn is_degenerate(&self) -> bool {
        self.area() < 1e-10
    }

    /// Satisfies triangle inequality: each side < sum of other two.
    pub fn satisfies_triangle_inequality(&self) -> bool {
        let (a, b, c) = self.side_lengths();
        a < b + c && b < a + c && c < a + b
    }

    /// Centroid (center of mass).
    pub fn centroid(&self) -> Point3 {
        Point3::new(
            (self.a.x + self.b.x + self.c.x) / 3.0,
            (self.a.y + self.b.y + self.c.y) / 3.0,
            (self.a.z + self.b.z + self.c.z) / 3.0,
        )
    }

    /// Congruence test via SSS (Hilbert III: three sides equal).
    pub fn is_congruent_to(&self, other: &Self) -> bool {
        let (a1, b1, c1) = self.side_lengths();
        let (a2, b2, c2) = other.side_lengths();
        let mut s1 = [a1, b1, c1];
        let mut s2 = [a2, b2, c2];
        s1.sort_by(|a, b| a.partial_cmp(b).unwrap());
        s2.sort_by(|a, b| a.partial_cmp(b).unwrap());
        (s1[0] - s2[0]).abs() < 1e-10
            && (s1[1] - s2[1]).abs() < 1e-10
            && (s1[2] - s2[2]).abs() < 1e-10
    }
}

impl Circle {
    pub fn new(center: Point3, radius: f64) -> Self {
        Self { center, radius }
    }

    /// Circumference: 2πr.
    pub fn circumference(&self) -> f64 {
        2.0 * PI * self.radius
    }

    /// Area: πr².
    pub fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }

    /// Does the point lie on the circle?
    pub fn contains_point(&self, p: &Point3) -> bool {
        (self.center.distance_to(p) - self.radius).abs() < 1e-10
    }

    /// Is the point inside the circle?
    pub fn is_interior(&self, p: &Point3) -> bool {
        self.center.distance_to(p) < self.radius
    }
}

impl Sphere {
    pub fn new(center: Point3, radius: f64) -> Self {
        Self { center, radius }
    }

    /// Surface area: 4πr².
    pub fn surface_area(&self) -> f64 {
        4.0 * PI * self.radius * self.radius
    }

    /// Volume: (4/3)πr³.
    pub fn volume(&self) -> f64 {
        (4.0 / 3.0) * PI * self.radius * self.radius * self.radius
    }

    /// Does the point lie on the sphere?
    pub fn contains_point(&self, p: &Point3) -> bool {
        (self.center.distance_to(p) - self.radius).abs() < 1e-10
    }
}
