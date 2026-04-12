use crate::formal::math::geometry::point::{Point2, Point3};
use crate::formal::math::geometry::vector::{Vec2, Vec3};

/// Line in 2D: defined by a point and direction (Hilbert I.1: two points determine a line).
#[derive(Debug, Clone)]
pub struct Line2 {
    pub point: Point2,
    pub direction: Vec2,
}

/// Line in 3D: point + direction.
#[derive(Debug, Clone)]
pub struct Line3 {
    pub point: Point3,
    pub direction: Vec3,
}

/// Line segment: bounded by two endpoints (Hilbert II: betweenness).
#[derive(Debug, Clone)]
pub struct Segment3 {
    pub start: Point3,
    pub end: Point3,
}

/// Ray: half-line starting at a point in a direction.
#[derive(Debug, Clone)]
pub struct Ray3 {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Line2 {
    /// From two points (Hilbert I.1).
    pub fn from_points(a: &Point2, b: &Point2) -> Self {
        Self {
            point: a.clone(),
            direction: a.vector_to(b),
        }
    }

    /// Point at parameter t: p(t) = point + t * direction.
    pub fn at(&self, t: f64) -> Point2 {
        self.point.translate(&self.direction.scale(t))
    }

    /// Distance from a point to this line.
    pub fn distance_to_point(&self, p: &Point2) -> f64 {
        let v = self.point.vector_to(p);
        v.cross(&self.direction).abs() / self.direction.norm()
    }

    /// Are two lines parallel? (Hilbert IV.1 relates to this).
    pub fn is_parallel_to(&self, other: &Self) -> bool {
        self.direction.cross(&other.direction).abs() < 1e-10
    }
}

impl Line3 {
    /// From two points.
    pub fn from_points(a: &Point3, b: &Point3) -> Self {
        Self {
            point: a.clone(),
            direction: a.vector_to(b),
        }
    }

    /// Point at parameter t.
    pub fn at(&self, t: f64) -> Point3 {
        self.point.translate(&self.direction.scale(t))
    }

    /// Distance from a point to this line.
    pub fn distance_to_point(&self, p: &Point3) -> f64 {
        let v = self.point.vector_to(p);
        v.cross(&self.direction).norm() / self.direction.norm()
    }

    /// Are two lines parallel?
    pub fn is_parallel_to(&self, other: &Self) -> bool {
        self.direction.is_parallel_to(&other.direction)
    }
}

impl Segment3 {
    pub fn new(start: Point3, end: Point3) -> Self {
        Self { start, end }
    }

    /// Length of the segment (congruence: segments with equal length are congruent).
    pub fn length(&self) -> f64 {
        self.start.distance_to(&self.end)
    }

    /// Midpoint.
    pub fn midpoint(&self) -> Point3 {
        self.start.midpoint(&self.end)
    }

    /// Direction vector.
    pub fn direction(&self) -> Vec3 {
        self.start.vector_to(&self.end)
    }

    /// Congruence test (Hilbert III.2): two segments are congruent iff same length.
    pub fn is_congruent_to(&self, other: &Self) -> bool {
        (self.length() - other.length()).abs() < 1e-10
    }
}

impl Ray3 {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// Point at parameter t >= 0.
    pub fn at(&self, t: f64) -> Point3 {
        self.origin.translate(&self.direction.scale(t))
    }
}
