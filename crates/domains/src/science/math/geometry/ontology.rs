use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::science::math::geometry::point::Point3;
use crate::science::math::geometry::projection;
use crate::science::math::geometry::shape::Triangle;
use crate::science::math::geometry::vector::Vec3;
use std::f64::consts::PI;

// ---------------------------------------------------------------------------
// Entity: geometric primitive types (Hilbert's primitive notions + extensions)
// ---------------------------------------------------------------------------

/// Classification of geometric objects in Euclidean geometry.
///
/// Hilbert's primitive notions: Point, Line, Plane.
/// Extended with derived objects: Segment, Ray, Angle, Triangle, Circle, Sphere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeometricPrimitive {
    Point,
    Line,
    Ray,
    Segment,
    Plane,
    Angle,
    Triangle,
    Circle,
    Sphere,
    Vector,
}

impl Entity for GeometricPrimitive {
    fn variants() -> Vec<Self> {
        vec![
            Self::Point,
            Self::Line,
            Self::Ray,
            Self::Segment,
            Self::Plane,
            Self::Angle,
            Self::Triangle,
            Self::Circle,
            Self::Sphere,
            Self::Vector,
        ]
    }
}

// ---------------------------------------------------------------------------
// Relationship: geometric relations (Hilbert's primitive relations)
// ---------------------------------------------------------------------------

/// Geometric relations between primitives.
///
/// Hilbert defines: Incidence, Betweenness, Congruence.
/// Extended with: Containment, Parallelism, Perpendicularity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeometricRelation {
    pub from: GeometricPrimitive,
    pub to: GeometricPrimitive,
    pub kind: RelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationKind {
    /// Category identity morphism.
    Identity,
    /// Hilbert Group I: a point lies on a line/plane, a line lies in a plane.
    Incidence,
    /// Hilbert Group II: a point lies between two others on a line.
    Betweenness,
    /// Hilbert Group III: segments or angles are congruent.
    Congruence,
    /// Containment: one object contains another (plane contains line).
    Containment,
    /// Hilbert Group IV: lines that do not intersect.
    Parallelism,
    /// Lines/planes meeting at right angles.
    Perpendicularity,
    /// Object is defined from another (triangle from points).
    Construction,
}

impl Relationship for GeometricRelation {
    type Object = GeometricPrimitive;

    fn source(&self) -> GeometricPrimitive {
        self.from
    }

    fn target(&self) -> GeometricPrimitive {
        self.to
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

/// The Euclidean geometry category.
///
/// Objects: geometric primitive types.
/// Morphisms: geometric relations between them.
pub struct GeometryCategory;

impl Category for GeometryCategory {
    type Object = GeometricPrimitive;
    type Morphism = GeometricRelation;

    fn identity(obj: &GeometricPrimitive) -> GeometricRelation {
        GeometricRelation {
            from: *obj,
            to: *obj,
            kind: RelationKind::Identity,
        }
    }

    fn compose(f: &GeometricRelation, g: &GeometricRelation) -> Option<GeometricRelation> {
        if f.to != g.from {
            return None;
        }
        // Identity law: compose(f, id) = f, compose(id, g) = g
        let kind = match (f.kind, g.kind) {
            (RelationKind::Identity, k) => k,
            (k, RelationKind::Identity) => k,
            (_, k) => k,
        };
        Some(GeometricRelation {
            from: f.from,
            to: g.to,
            kind,
        })
    }

    fn morphisms() -> Vec<GeometricRelation> {
        use GeometricPrimitive::*;
        use RelationKind::*;

        let mut m = Vec::new();

        // Identity for all
        for obj in GeometricPrimitive::variants() {
            m.push(Self::identity(&obj));
        }

        // Hilbert I: Incidence relations
        m.push(GeometricRelation {
            from: Point,
            to: Line,
            kind: Incidence,
        });
        m.push(GeometricRelation {
            from: Point,
            to: Plane,
            kind: Incidence,
        });
        m.push(GeometricRelation {
            from: Line,
            to: Plane,
            kind: Incidence,
        });
        m.push(GeometricRelation {
            from: Point,
            to: Segment,
            kind: Incidence,
        });
        m.push(GeometricRelation {
            from: Point,
            to: Circle,
            kind: Incidence,
        });
        m.push(GeometricRelation {
            from: Point,
            to: Sphere,
            kind: Incidence,
        });

        // Hilbert II: Betweenness
        m.push(GeometricRelation {
            from: Point,
            to: Point,
            kind: Betweenness,
        });

        // Hilbert III: Congruence
        m.push(GeometricRelation {
            from: Segment,
            to: Segment,
            kind: Congruence,
        });
        m.push(GeometricRelation {
            from: Angle,
            to: Angle,
            kind: Congruence,
        });
        m.push(GeometricRelation {
            from: Triangle,
            to: Triangle,
            kind: Congruence,
        });

        // Containment
        m.push(GeometricRelation {
            from: Plane,
            to: Line,
            kind: Containment,
        });
        m.push(GeometricRelation {
            from: Plane,
            to: Point,
            kind: Containment,
        });
        m.push(GeometricRelation {
            from: Line,
            to: Point,
            kind: Containment,
        });

        // Hilbert IV: Parallelism
        m.push(GeometricRelation {
            from: Line,
            to: Line,
            kind: Parallelism,
        });
        m.push(GeometricRelation {
            from: Plane,
            to: Plane,
            kind: Parallelism,
        });

        // Perpendicularity
        m.push(GeometricRelation {
            from: Line,
            to: Line,
            kind: Perpendicularity,
        });
        m.push(GeometricRelation {
            from: Plane,
            to: Plane,
            kind: Perpendicularity,
        });
        m.push(GeometricRelation {
            from: Line,
            to: Plane,
            kind: Perpendicularity,
        });

        // Construction (derived objects from primitives)
        m.push(GeometricRelation {
            from: Point,
            to: Triangle,
            kind: Construction,
        });
        m.push(GeometricRelation {
            from: Point,
            to: Circle,
            kind: Construction,
        });
        m.push(GeometricRelation {
            from: Point,
            to: Sphere,
            kind: Construction,
        });

        // Transitive closure: Point → Line → Plane
        m.push(GeometricRelation {
            from: Point,
            to: Plane,
            kind: Containment,
        });

        m
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: dimension of the geometric object.
#[derive(Debug, Clone)]
pub struct Dimension;

impl Quality for Dimension {
    type Individual = GeometricPrimitive;
    type Value = usize;

    fn get(&self, prim: &GeometricPrimitive) -> Option<usize> {
        Some(match prim {
            GeometricPrimitive::Point => 0,
            GeometricPrimitive::Line | GeometricPrimitive::Ray | GeometricPrimitive::Segment => 1,
            GeometricPrimitive::Plane
            | GeometricPrimitive::Triangle
            | GeometricPrimitive::Circle => 2,
            GeometricPrimitive::Sphere => 2, // 2D manifold in 3D
            GeometricPrimitive::Angle => 0,  // scalar measure
            GeometricPrimitive::Vector => 1,
        })
    }
}

/// Quality: degrees of freedom.
#[derive(Debug, Clone)]
pub struct DegreesOfFreedom;

impl Quality for DegreesOfFreedom {
    type Individual = GeometricPrimitive;
    type Value = usize;

    fn get(&self, prim: &GeometricPrimitive) -> Option<usize> {
        Some(match prim {
            GeometricPrimitive::Point => 3,    // x, y, z
            GeometricPrimitive::Line => 4,     // point + direction (4 DOF in R³)
            GeometricPrimitive::Ray => 5,      // origin + direction
            GeometricPrimitive::Segment => 6,  // two endpoints
            GeometricPrimitive::Plane => 3,    // normal + offset
            GeometricPrimitive::Angle => 1,    // single scalar
            GeometricPrimitive::Triangle => 9, // three vertices
            GeometricPrimitive::Circle => 4,   // center + radius (in a plane)
            GeometricPrimitive::Sphere => 4,   // center + radius
            GeometricPrimitive::Vector => 3,   // x, y, z
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms — Hilbert's groups + metric space + vector space
// ---------------------------------------------------------------------------

/// Hilbert I / Metric: distance is non-negative. d(a,b) ≥ 0.
pub struct MetricNonNegativity;

impl Axiom for MetricNonNegativity {
    fn description(&self) -> &str {
        "metric axiom: d(a,b) >= 0 (non-negativity)"
    }

    fn holds(&self) -> bool {
        for a in &canonical_points_3d() {
            for b in &canonical_points_3d() {
                if a.distance_to(b) < -1e-15 {
                    return false;
                }
            }
        }
        true
    }
}

/// Metric: d(a,b) = 0 iff a = b (identity of indiscernibles).
pub struct MetricIdentity;

impl Axiom for MetricIdentity {
    fn description(&self) -> &str {
        "metric axiom: d(a,b) = 0 iff a = b (identity of indiscernibles)"
    }

    fn holds(&self) -> bool {
        let pts = canonical_points_3d();
        for a in &pts {
            // d(a,a) = 0
            if a.distance_to(a) > 1e-15 {
                return false;
            }
            // d(a,b) > 0 for a != b
            for b in &pts {
                if a != b && a.distance_to(b) < 1e-15 {
                    return false;
                }
            }
        }
        true
    }
}

/// Metric: d(a,b) = d(b,a) (symmetry).
pub struct MetricSymmetry;

impl Axiom for MetricSymmetry {
    fn description(&self) -> &str {
        "metric axiom: d(a,b) = d(b,a) (symmetry)"
    }

    fn holds(&self) -> bool {
        let pts = canonical_points_3d();
        for a in &pts {
            for b in &pts {
                if (a.distance_to(b) - b.distance_to(a)).abs() > 1e-15 {
                    return false;
                }
            }
        }
        true
    }
}

/// Metric: d(a,c) ≤ d(a,b) + d(b,c) (triangle inequality).
pub struct TriangleInequality;

impl Axiom for TriangleInequality {
    fn description(&self) -> &str {
        "metric axiom: d(a,c) <= d(a,b) + d(b,c) (triangle inequality)"
    }

    fn holds(&self) -> bool {
        let pts = canonical_points_3d();
        for a in &pts {
            for b in &pts {
                for c in &pts {
                    if a.distance_to(c) > a.distance_to(b) + b.distance_to(c) + 1e-10 {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Euclidean theorem: sum of interior angles of a triangle = π.
pub struct TriangleAngleSum;

impl Axiom for TriangleAngleSum {
    fn description(&self) -> &str {
        "Euclidean theorem: triangle interior angles sum to pi"
    }

    fn holds(&self) -> bool {
        for t in &canonical_triangles() {
            if t.is_degenerate() {
                continue;
            }
            if (t.angle_sum() - PI).abs() > 1e-9 {
                return false;
            }
        }
        true
    }
}

/// Pythagorean theorem: for right triangle with legs a,b and hypotenuse c: a² + b² = c².
pub struct PythagoreanTheorem;

impl Axiom for PythagoreanTheorem {
    fn description(&self) -> &str {
        "Pythagorean theorem: a^2 + b^2 = c^2 for right triangles"
    }

    fn holds(&self) -> bool {
        // Construct known right triangles and verify
        let right_triangles = vec![
            // 3-4-5
            Triangle::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(3.0, 0.0, 0.0),
                Point3::new(0.0, 4.0, 0.0),
            ),
            // 5-12-13
            Triangle::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(5.0, 0.0, 0.0),
                Point3::new(0.0, 12.0, 0.0),
            ),
            // 1-1-√2
            Triangle::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(1.0, 0.0, 0.0),
                Point3::new(0.0, 1.0, 0.0),
            ),
        ];

        for t in &right_triangles {
            let (a, b, c) = t.side_lengths();
            let mut sides = [a, b, c];
            sides.sort_by(|x, y| x.partial_cmp(y).unwrap());
            let (leg1, leg2, hyp) = (sides[0], sides[1], sides[2]);
            if (leg1 * leg1 + leg2 * leg2 - hyp * hyp).abs() > 1e-9 {
                return false;
            }
        }
        true
    }
}

/// Vector space axiom: addition is commutative. u + v = v + u.
pub struct VectorAdditionCommutativity;

impl Axiom for VectorAdditionCommutativity {
    fn description(&self) -> &str {
        "vector space axiom 2: u + v = v + u (commutativity)"
    }

    fn holds(&self) -> bool {
        let vecs = canonical_vectors_3d();
        for u in &vecs {
            for v in &vecs {
                let uv = u.add(v);
                let vu = v.add(u);
                if (uv.x - vu.x).abs() > 1e-15
                    || (uv.y - vu.y).abs() > 1e-15
                    || (uv.z - vu.z).abs() > 1e-15
                {
                    return false;
                }
            }
        }
        true
    }
}

/// Vector space axiom: addition is associative. (u+v)+w = u+(v+w).
pub struct VectorAdditionAssociativity;

impl Axiom for VectorAdditionAssociativity {
    fn description(&self) -> &str {
        "vector space axiom 1: (u+v)+w = u+(v+w) (associativity)"
    }

    fn holds(&self) -> bool {
        let vecs = canonical_vectors_3d();
        for u in &vecs {
            for v in &vecs {
                for w in &vecs {
                    let lhs = u.add(v).add(w);
                    let rhs = u.add(&v.add(w));
                    if (lhs.x - rhs.x).abs() > 1e-12
                        || (lhs.y - rhs.y).abs() > 1e-12
                        || (lhs.z - rhs.z).abs() > 1e-12
                    {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Cross product is anticommutative: a × b = -(b × a).
pub struct CrossProductAnticommutativity;

impl Axiom for CrossProductAnticommutativity {
    fn description(&self) -> &str {
        "cross product is anticommutative: a x b = -(b x a)"
    }

    fn holds(&self) -> bool {
        let vecs = canonical_vectors_3d();
        for a in &vecs {
            for b in &vecs {
                let ab = a.cross(b);
                let ba = b.cross(a).negate();
                if (ab.x - ba.x).abs() > 1e-15
                    || (ab.y - ba.y).abs() > 1e-15
                    || (ab.z - ba.z).abs() > 1e-15
                {
                    return false;
                }
            }
        }
        true
    }
}

/// Cross product perpendicular to both inputs: (a×b)·a = 0, (a×b)·b = 0.
pub struct CrossProductPerpendicularity;

impl Axiom for CrossProductPerpendicularity {
    fn description(&self) -> &str {
        "cross product is perpendicular to both inputs: (a x b) . a = 0"
    }

    fn holds(&self) -> bool {
        let vecs = canonical_vectors_3d();
        for a in &vecs {
            for b in &vecs {
                let cross = a.cross(b);
                if cross.dot(a).abs() > 1e-10 || cross.dot(b).abs() > 1e-10 {
                    return false;
                }
            }
        }
        true
    }
}

/// Dot product is commutative: a · b = b · a.
pub struct DotProductCommutativity;

impl Axiom for DotProductCommutativity {
    fn description(&self) -> &str {
        "inner product is commutative: a . b = b . a"
    }

    fn holds(&self) -> bool {
        let vecs = canonical_vectors_3d();
        for a in &vecs {
            for b in &vecs {
                if (a.dot(b) - b.dot(a)).abs() > 1e-15 {
                    return false;
                }
            }
        }
        true
    }
}

/// Projection is idempotent: proj(proj(a, b), b) = proj(a, b).
pub struct ProjectionIdempotent;

impl Axiom for ProjectionIdempotent {
    fn description(&self) -> &str {
        "vector projection is idempotent: proj(proj(a,b),b) = proj(a,b)"
    }

    fn holds(&self) -> bool {
        let vecs = canonical_vectors_3d();
        for a in &vecs {
            for b in &vecs {
                if b.norm() < 1e-15 {
                    continue;
                }
                let p1 = projection::project_vector_onto_vector(a, b);
                let p2 = projection::project_vector_onto_vector(&p1, b);
                if (p1.x - p2.x).abs() > 1e-10
                    || (p1.y - p2.y).abs() > 1e-10
                    || (p1.z - p2.z).abs() > 1e-10
                {
                    return false;
                }
            }
        }
        true
    }
}

/// Hilbert II.1: betweenness is symmetric. If B is between A and C, B is between C and A.
pub struct BetweennessSymmetry;

impl Axiom for BetweennessSymmetry {
    fn description(&self) -> &str {
        "Hilbert II.1: if B is between A and C, then B is between C and A"
    }

    fn holds(&self) -> bool {
        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(1.0, 0.0, 0.0);
        let c = Point3::new(2.0, 0.0, 0.0);
        a.is_between(&b, &c) == c.is_between(&b, &a)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The Euclidean geometry ontology.
///
/// Founded on Hilbert's axiom system (Grundlagen der Geometrie, 1899)
/// with metric space axioms and vector space axioms for R³.
pub struct EuclideanGeometryOntology;

impl Ontology for EuclideanGeometryOntology {
    type Cat = GeometryCategory;
    type Qual = Dimension;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            // Metric space axioms
            Box::new(MetricNonNegativity),
            Box::new(MetricIdentity),
            Box::new(MetricSymmetry),
            Box::new(TriangleInequality),
            // Euclidean theorems
            Box::new(TriangleAngleSum),
            Box::new(PythagoreanTheorem),
            // Vector space axioms
            Box::new(VectorAdditionCommutativity),
            Box::new(VectorAdditionAssociativity),
            // Inner/cross product laws
            Box::new(DotProductCommutativity),
            Box::new(CrossProductAnticommutativity),
            Box::new(CrossProductPerpendicularity),
            // Projection
            Box::new(ProjectionIdempotent),
            // Hilbert betweenness
            Box::new(BetweennessSymmetry),
        ]
    }
}

// ---------------------------------------------------------------------------
// Canonical test data
// ---------------------------------------------------------------------------

fn canonical_points_3d() -> Vec<Point3> {
    vec![
        Point3::origin(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(1.0, 2.0, 3.0),
        Point3::new(-1.0, 0.5, -2.0),
        Point3::new(3.0, 4.0, 0.0),
        Point3::new(0.0, 5.0, 12.0),
    ]
}

fn canonical_vectors_3d() -> Vec<Vec3> {
    vec![
        Vec3::zero(),
        Vec3::unit_x(),
        Vec3::unit_y(),
        Vec3::unit_z(),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(1.0, 2.0, 3.0),
        Vec3::new(-1.0, 0.5, -2.0),
        Vec3::new(3.0, 4.0, 5.0),
    ]
}

fn canonical_triangles() -> Vec<Triangle> {
    vec![
        // Right triangle 3-4-5
        Triangle::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(3.0, 0.0, 0.0),
            Point3::new(0.0, 4.0, 0.0),
        ),
        // Equilateral
        Triangle::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 0.866_025_403_784_438_6, 0.0),
        ),
        // Isoceles
        Triangle::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(1.0, 3.0, 0.0),
        ),
        // Scalene in 3D
        Triangle::new(
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(0.0, 0.0, 3.0),
        ),
    ]
}
