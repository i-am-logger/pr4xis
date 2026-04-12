use crate::formal::math::geometry::point::{Point2, Point3};

/// Euclidean distance in R² (L2 norm).
///
/// Satisfies the metric space axioms: non-negativity, identity of
/// indiscernibles, symmetry, triangle inequality.
pub fn euclidean_2d(a: &Point2, b: &Point2) -> f64 {
    a.distance_to(b)
}

/// Euclidean distance in R³ (L2 norm).
pub fn euclidean_3d(a: &Point3, b: &Point3) -> f64 {
    a.distance_to(b)
}

/// Manhattan distance in R² (L1 norm).
pub fn manhattan_2d(a: &Point2, b: &Point2) -> f64 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

/// Manhattan distance in R³ (L1 norm).
pub fn manhattan_3d(a: &Point3, b: &Point3) -> f64 {
    (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()
}

/// Chebyshev distance in R³ (L∞ norm).
pub fn chebyshev_3d(a: &Point3, b: &Point3) -> f64 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    let dz = (a.z - b.z).abs();
    dx.max(dy).max(dz)
}

/// Squared Euclidean distance (avoids sqrt, still useful for comparisons).
pub fn squared_euclidean_3d(a: &Point3, b: &Point3) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    dx * dx + dy * dy + dz * dz
}
