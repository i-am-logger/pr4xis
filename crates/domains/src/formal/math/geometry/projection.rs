use crate::formal::math::geometry::line::Line3;
use crate::formal::math::geometry::plane::Plane;
use crate::formal::math::geometry::point::Point3;
use crate::formal::math::geometry::vector::Vec3;

/// Orthogonal projection of a vector onto another vector.
///
/// proj_b(a) = (a · b / b · b) * b
///
/// Projection is idempotent: proj(proj(a)) = proj(a).
pub fn project_vector_onto_vector(a: &Vec3, b: &Vec3) -> Vec3 {
    let scale = a.dot(b) / b.dot(b);
    b.scale(scale)
}

/// Component of a perpendicular to b (rejection).
///
/// a = proj_b(a) + rej_b(a)
pub fn reject_vector_from_vector(a: &Vec3, b: &Vec3) -> Vec3 {
    let proj = project_vector_onto_vector(a, b);
    a.sub(&proj)
}

/// Project a point onto a line (closest point on line).
pub fn project_point_onto_line(p: &Point3, line: &Line3) -> Point3 {
    let v = line.point.vector_to(p);
    let t = v.dot(&line.direction) / line.direction.dot(&line.direction);
    line.at(t)
}

/// Project a point onto a plane (closest point on plane).
pub fn project_point_onto_plane(p: &Point3, plane: &Plane) -> Point3 {
    plane.project_point(p)
}

/// Distance from a point to its projection on a line.
pub fn point_line_distance(p: &Point3, line: &Line3) -> f64 {
    let proj = project_point_onto_line(p, line);
    p.distance_to(&proj)
}
