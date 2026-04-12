use crate::formal::math::geometry::point::Point3;
use crate::formal::math::geometry::vector::Vec3;

/// Plane in 3D: defined by a point and normal vector.
///
/// Hilbert I.4: three non-collinear points determine a unique plane.
/// Equation: n · (p - point) = 0, or ax + by + cz = d.
#[derive(Debug, Clone)]
pub struct Plane {
    pub point: Point3,
    pub normal: Vec3,
}

impl Plane {
    /// From point and normal vector.
    pub fn from_point_normal(point: Point3, normal: Vec3) -> Self {
        Self { point, normal }
    }

    /// From three non-collinear points (Hilbert I.4).
    /// Returns None if points are collinear.
    pub fn from_three_points(a: &Point3, b: &Point3, c: &Point3) -> Option<Self> {
        let ab = a.vector_to(b);
        let ac = a.vector_to(c);
        let normal = ab.cross(&ac);
        if normal.norm() < 1e-10 {
            return None; // collinear
        }
        Some(Self {
            point: a.clone(),
            normal,
        })
    }

    /// Signed distance from a point to the plane.
    /// Positive = same side as normal, negative = opposite.
    pub fn signed_distance(&self, p: &Point3) -> f64 {
        let v = self.point.vector_to(p);
        v.dot(&self.normal) / self.normal.norm()
    }

    /// Absolute distance from a point to the plane.
    pub fn distance_to_point(&self, p: &Point3) -> f64 {
        self.signed_distance(p).abs()
    }

    /// Does the point lie on the plane? (Hilbert I.6: incidence).
    pub fn contains_point(&self, p: &Point3) -> bool {
        self.distance_to_point(p) < 1e-10
    }

    /// Are two planes parallel?
    pub fn is_parallel_to(&self, other: &Self) -> bool {
        self.normal.is_parallel_to(&other.normal)
    }

    /// Are two planes perpendicular?
    pub fn is_perpendicular_to(&self, other: &Self) -> bool {
        self.normal.is_perpendicular_to(&other.normal)
    }

    /// Project a point onto the plane (closest point on plane).
    /// If the plane has a degenerate (zero) normal, returns the point unchanged.
    pub fn project_point(&self, p: &Point3) -> Point3 {
        let d = self.signed_distance(p);
        let n_unit = match self.normal.normalize() {
            Some(n) => n,
            None => return p.clone(),
        };
        p.translate(&n_unit.scale(-d))
    }
}
