use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::formal::math::geometry::ontology::*;

// ---------------------------------------------------------------------------
// Category law verification
// ---------------------------------------------------------------------------

#[test]
fn geometry_category_laws() {
    check_category_laws::<GeometryCategory>().unwrap();
}

#[test]
fn geometry_ontology_validates() {
    EuclideanGeometryOntology::validate().unwrap();
}

// ---------------------------------------------------------------------------
// Metric space axioms (individual proofs)
// ---------------------------------------------------------------------------

#[test]
fn metric_non_negativity() {
    assert!(MetricNonNegativity.holds());
}

#[test]
fn metric_identity_of_indiscernibles() {
    assert!(MetricIdentity.holds());
}

#[test]
fn metric_symmetry() {
    assert!(MetricSymmetry.holds());
}

#[test]
fn metric_triangle_inequality() {
    assert!(TriangleInequality.holds());
}

// ---------------------------------------------------------------------------
// Euclidean theorems
// ---------------------------------------------------------------------------

#[test]
fn triangle_angle_sum_is_pi() {
    assert!(TriangleAngleSum.holds());
}

#[test]
fn pythagorean_theorem() {
    assert!(PythagoreanTheorem.holds());
}

// ---------------------------------------------------------------------------
// Vector space axioms
// ---------------------------------------------------------------------------

#[test]
fn vector_addition_is_commutative() {
    assert!(VectorAdditionCommutativity.holds());
}

#[test]
fn vector_addition_is_associative() {
    assert!(VectorAdditionAssociativity.holds());
}

// ---------------------------------------------------------------------------
// Inner/cross product laws
// ---------------------------------------------------------------------------

#[test]
fn dot_product_is_commutative() {
    assert!(DotProductCommutativity.holds());
}

#[test]
fn cross_product_is_anticommutative() {
    assert!(CrossProductAnticommutativity.holds());
}

#[test]
fn cross_product_is_perpendicular() {
    assert!(CrossProductPerpendicularity.holds());
}

// ---------------------------------------------------------------------------
// Projection
// ---------------------------------------------------------------------------

#[test]
fn projection_is_idempotent() {
    assert!(ProjectionIdempotent.holds());
}

// ---------------------------------------------------------------------------
// Hilbert betweenness
// ---------------------------------------------------------------------------

#[test]
fn betweenness_is_symmetric() {
    assert!(BetweennessSymmetry.holds());
}

// ---------------------------------------------------------------------------
// Property-based proofs (proptest)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// H10: Plane::project_point with zero normal does not panic
// ---------------------------------------------------------------------------

#[test]
fn plane_project_point_zero_normal_no_panic() {
    use crate::formal::math::geometry::plane::Plane;
    use crate::formal::math::geometry::point::Point3;
    use crate::formal::math::geometry::vector::Vec3;

    let plane = Plane::from_point_normal(Point3::new(0.0, 0.0, 0.0), Vec3::zero());
    let p = Point3::new(1.0, 2.0, 3.0);
    // Should not panic; returns the original point for degenerate plane
    let projected = plane.project_point(&p);
    assert!(
        (projected.x - p.x).abs() < 1e-12
            && (projected.y - p.y).abs() < 1e-12
            && (projected.z - p.z).abs() < 1e-12,
        "degenerate plane should return point unchanged"
    );
}

#[cfg(test)]
mod proptest_proofs {
    use crate::formal::math::geometry::distance;
    use crate::formal::math::geometry::point::{Point2, Point3};
    use crate::formal::math::geometry::projection;
    use crate::formal::math::geometry::shape::Triangle;
    use crate::formal::math::geometry::vector::Vec3;
    use core::f64::consts::PI;
    use proptest::prelude::*;

    fn arb_point3() -> impl Strategy<Value = Point3> {
        (-100.0..100.0_f64, -100.0..100.0_f64, -100.0..100.0_f64)
            .prop_map(|(x, y, z)| Point3::new(x, y, z))
    }

    fn arb_vec3() -> impl Strategy<Value = Vec3> {
        (-100.0..100.0_f64, -100.0..100.0_f64, -100.0..100.0_f64)
            .prop_map(|(x, y, z)| Vec3::new(x, y, z))
    }

    fn arb_nonzero_vec3() -> impl Strategy<Value = Vec3> {
        arb_vec3().prop_filter("non-zero", |v| v.norm() > 1e-6)
    }

    fn arb_point2() -> impl Strategy<Value = Point2> {
        (-100.0..100.0_f64, -100.0..100.0_f64).prop_map(|(x, y)| Point2::new(x, y))
    }

    // --- Metric axioms ---

    proptest! {
        #[test]
        fn distance_is_non_negative(a in arb_point3(), b in arb_point3()) {
            prop_assert!(a.distance_to(&b) >= 0.0);
        }

        #[test]
        fn distance_is_symmetric(a in arb_point3(), b in arb_point3()) {
            let ab = a.distance_to(&b);
            let ba = b.distance_to(&a);
            prop_assert!((ab - ba).abs() < 1e-12);
        }

        #[test]
        fn distance_triangle_inequality(
            a in arb_point3(),
            b in arb_point3(),
            c in arb_point3(),
        ) {
            let ac = a.distance_to(&c);
            let ab = a.distance_to(&b);
            let bc = b.distance_to(&c);
            prop_assert!(ac <= ab + bc + 1e-10);
        }

        #[test]
        fn distance_to_self_is_zero(a in arb_point3()) {
            prop_assert!(a.distance_to(&a) < 1e-15);
        }

        // --- Vector space axioms ---

        #[test]
        fn addition_commutativity(u in arb_vec3(), v in arb_vec3()) {
            let uv = u.add(&v);
            let vu = v.add(&u);
            prop_assert!((uv.x - vu.x).abs() < 1e-12);
            prop_assert!((uv.y - vu.y).abs() < 1e-12);
            prop_assert!((uv.z - vu.z).abs() < 1e-12);
        }

        #[test]
        fn addition_associativity(u in arb_vec3(), v in arb_vec3(), w in arb_vec3()) {
            let lhs = u.add(&v).add(&w);
            let rhs = u.add(&v.add(&w));
            prop_assert!((lhs.x - rhs.x).abs() < 1e-10);
            prop_assert!((lhs.y - rhs.y).abs() < 1e-10);
            prop_assert!((lhs.z - rhs.z).abs() < 1e-10);
        }

        #[test]
        fn additive_identity(v in arb_vec3()) {
            let sum = v.add(&Vec3::zero());
            prop_assert!((sum.x - v.x).abs() < 1e-15);
            prop_assert!((sum.y - v.y).abs() < 1e-15);
            prop_assert!((sum.z - v.z).abs() < 1e-15);
        }

        #[test]
        fn additive_inverse(v in arb_vec3()) {
            let sum = v.add(&v.negate());
            prop_assert!(sum.norm() < 1e-12);
        }

        #[test]
        fn scalar_multiplication_identity(v in arb_vec3()) {
            let scaled = v.scale(1.0);
            prop_assert!((scaled.x - v.x).abs() < 1e-15);
            prop_assert!((scaled.y - v.y).abs() < 1e-15);
            prop_assert!((scaled.z - v.z).abs() < 1e-15);
        }

        #[test]
        fn scalar_multiplication_compatibility(
            v in arb_vec3(),
            a in -10.0..10.0_f64,
            b in -10.0..10.0_f64,
        ) {
            let lhs = v.scale(a * b);
            let rhs = v.scale(a).scale(b);
            prop_assert!((lhs.x - rhs.x).abs() < 1e-8);
            prop_assert!((lhs.y - rhs.y).abs() < 1e-8);
            prop_assert!((lhs.z - rhs.z).abs() < 1e-8);
        }

        #[test]
        fn scalar_distributivity_over_vectors(
            u in arb_vec3(),
            v in arb_vec3(),
            a in -10.0..10.0_f64,
        ) {
            let lhs = u.add(&v).scale(a);
            let rhs = u.scale(a).add(&v.scale(a));
            prop_assert!((lhs.x - rhs.x).abs() < 1e-8);
            prop_assert!((lhs.y - rhs.y).abs() < 1e-8);
            prop_assert!((lhs.z - rhs.z).abs() < 1e-8);
        }

        #[test]
        fn scalar_distributivity_over_field(
            v in arb_vec3(),
            a in -10.0..10.0_f64,
            b in -10.0..10.0_f64,
        ) {
            let lhs = v.scale(a + b);
            let rhs = v.scale(a).add(&v.scale(b));
            prop_assert!((lhs.x - rhs.x).abs() < 1e-8);
            prop_assert!((lhs.y - rhs.y).abs() < 1e-8);
            prop_assert!((lhs.z - rhs.z).abs() < 1e-8);
        }

        // --- Inner product / cross product ---

        #[test]
        fn dot_product_commutative(a in arb_vec3(), b in arb_vec3()) {
            prop_assert!((a.dot(&b) - b.dot(&a)).abs() < 1e-10);
        }

        #[test]
        fn cross_product_anticommutative(a in arb_vec3(), b in arb_vec3()) {
            let ab = a.cross(&b);
            let ba_neg = b.cross(&a).negate();
            prop_assert!((ab.x - ba_neg.x).abs() < 1e-10);
            prop_assert!((ab.y - ba_neg.y).abs() < 1e-10);
            prop_assert!((ab.z - ba_neg.z).abs() < 1e-10);
        }

        #[test]
        fn cross_product_perpendicular_to_both(a in arb_nonzero_vec3(), b in arb_nonzero_vec3()) {
            let cross = a.cross(&b);
            prop_assert!(cross.dot(&a).abs() < 1e-8,
                "cross.a = {}", cross.dot(&a));
            prop_assert!(cross.dot(&b).abs() < 1e-8,
                "cross.b = {}", cross.dot(&b));
        }

        #[test]
        fn lagrange_identity(a in arb_vec3(), b in arb_vec3()) {
            // |a × b|² = |a|²|b|² - (a·b)²
            let cross_sq = a.cross(&b).norm_squared();
            let rhs = a.norm_squared() * b.norm_squared() - a.dot(&b) * a.dot(&b);
            prop_assert!((cross_sq - rhs).abs() < 1e-6,
                "lhs={}, rhs={}", cross_sq, rhs);
        }

        #[test]
        fn rotation_preserves_norm(v in arb_nonzero_vec3()) {
            let unit = v.normalize().unwrap();
            prop_assert!((unit.norm() - 1.0).abs() < 1e-12);
        }

        // --- Projection ---

        #[test]
        fn projection_is_idempotent(a in arb_vec3(), b in arb_nonzero_vec3()) {
            let p1 = projection::project_vector_onto_vector(&a, &b);
            let p2 = projection::project_vector_onto_vector(&p1, &b);
            prop_assert!((p1.x - p2.x).abs() < 1e-8);
            prop_assert!((p1.y - p2.y).abs() < 1e-8);
            prop_assert!((p1.z - p2.z).abs() < 1e-8);
        }

        #[test]
        fn projection_plus_rejection_equals_original(a in arb_vec3(), b in arb_nonzero_vec3()) {
            let proj = projection::project_vector_onto_vector(&a, &b);
            let rej = projection::reject_vector_from_vector(&a, &b);
            let sum = proj.add(&rej);
            prop_assert!((sum.x - a.x).abs() < 1e-8);
            prop_assert!((sum.y - a.y).abs() < 1e-8);
            prop_assert!((sum.z - a.z).abs() < 1e-8);
        }

        #[test]
        fn rejection_is_perpendicular_to_basis(a in arb_vec3(), b in arb_nonzero_vec3()) {
            let rej = projection::reject_vector_from_vector(&a, &b);
            prop_assert!(rej.dot(&b).abs() < 1e-8);
        }

        // --- Triangle ---

        #[test]
        fn triangle_angle_sum_is_pi(
            ax in -10.0..10.0_f64, ay in -10.0..10.0_f64,
            bx in -10.0..10.0_f64, by in -10.0..10.0_f64,
            cx in -10.0..10.0_f64, cy in -10.0..10.0_f64,
        ) {
            let t = Triangle::new(
                Point3::new(ax, ay, 0.0),
                Point3::new(bx, by, 0.0),
                Point3::new(cx, cy, 0.0),
            );
            if !t.is_degenerate() {
                prop_assert!((t.angle_sum() - PI).abs() < 1e-8,
                    "angle sum = {}", t.angle_sum());
            }
        }

        #[test]
        fn triangle_inequality_holds_for_sides(
            ax in -10.0..10.0_f64, ay in -10.0..10.0_f64,
            bx in -10.0..10.0_f64, by in -10.0..10.0_f64,
            cx in -10.0..10.0_f64, cy in -10.0..10.0_f64,
        ) {
            let t = Triangle::new(
                Point3::new(ax, ay, 0.0),
                Point3::new(bx, by, 0.0),
                Point3::new(cx, cy, 0.0),
            );
            if !t.is_degenerate() {
                prop_assert!(t.satisfies_triangle_inequality());
            }
        }

        // --- Congruence (Hilbert III) ---

        #[test]
        fn congruence_is_reflexive(
            ax in -10.0..10.0_f64, ay in -10.0..10.0_f64,
            bx in -10.0..10.0_f64, by in -10.0..10.0_f64,
            cx in -10.0..10.0_f64, cy in -10.0..10.0_f64,
        ) {
            let t = Triangle::new(
                Point3::new(ax, ay, 0.0),
                Point3::new(bx, by, 0.0),
                Point3::new(cx, cy, 0.0),
            );
            prop_assert!(t.is_congruent_to(&t));
        }

        // --- 2D metric ---

        #[test]
        fn distance_2d_is_non_negative(a in arb_point2(), b in arb_point2()) {
            prop_assert!(a.distance_to(&b) >= 0.0);
        }

        #[test]
        fn distance_2d_is_symmetric(a in arb_point2(), b in arb_point2()) {
            prop_assert!((a.distance_to(&b) - b.distance_to(&a)).abs() < 1e-12);
        }

        #[test]
        fn manhattan_satisfies_triangle_inequality(
            a in arb_point3(),
            b in arb_point3(),
            c in arb_point3(),
        ) {
            let ac = distance::manhattan_3d(&a, &c);
            let ab = distance::manhattan_3d(&a, &b);
            let bc = distance::manhattan_3d(&b, &c);
            prop_assert!(ac <= ab + bc + 1e-10);
        }
    }
}
