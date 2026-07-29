//! Euclidean geometry ontology — Hilbert's primitive notions plus
//! derived objects, with metric-space, vector-space, and Hilbert-style
//! axioms.
//!
//! Hilbert's three primitive notions (Point, Line, Plane) are extended
//! with the derived objects of school geometry: Ray, Segment, Angle,
//! Triangle, Circle, Sphere, and the linear-algebra primitive Vector.
//! The category's morphisms follow Hilbert's relation groups: incidence,
//! betweenness, congruence, parallelism, perpendicularity, plus the
//! lattice-theoretic containment relation and the constructive
//! "is defined from" relation.
//!
//! # Literature
//!
//! - **Hilbert (1899)** *Grundlagen der Geometrie* — the primitive
//!   notions Point/Line/Plane and the four axiom groups (Incidence,
//!   Order/Betweenness, Congruence, Parallels).
//! - **Avigad, Dean & Mumma (2009)** "A Formal System for Euclid's
//!   Elements", *Review of Symbolic Logic* 2(4):700–768 — formalisation
//!   approach for elementary Euclidean geometry.
//! - **Kahan** *Axioms for Fields and Vector Spaces* — the eight
//!   vector-space axioms used in the Vector-related claims below.

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::geometry::point::Point3;
use crate::formal::math::geometry::projection;
use crate::formal::math::geometry::shape::Triangle;
use crate::formal::math::geometry::vector::Vec3;
use core::f64::consts::PI;

pr4xis::ontology! {
    name: "EuclideanGeometry",
    source: "Hilbert (1899) Grundlagen der Geometrie; Avigad, Dean & Mumma (2009) A Formal System for Euclid's Elements, Review of Symbolic Logic 2(4):700-768; Kahan, Axioms for Fields and Vector Spaces",

    concepts: [
        // Hilbert's primitive notions (Grundlagen §1).
        Point, Line, Plane,
        // Derived linear objects (Euclid's Elements Book I; Hilbert §3).
        Ray, Segment, Vector,
        // Derived planar / spatial objects (Euclid Book I and Book XI).
        Angle, Triangle, Circle, Sphere,
    ],

    labels: {
        Point: ("en", "Point",
            "Hilbert (1899) §1: a primitive notion, one of the three undefined objects of geometry; characterised only by the axioms it satisfies."),
        Line: ("en", "Line",
            "Hilbert (1899) §1: a primitive notion — the second of the three undefined geometric objects, characterised by the incidence and order axioms."),
        Plane: ("en", "Plane",
            "Hilbert (1899) §1: a primitive notion — the third of the three undefined geometric objects; planes are individuated by the points and lines incident to them."),
        Ray: ("en", "Ray",
            "Euclid, Elements Book I Definition 4 (informally); a half-line — a point together with one of the two ordered directions along a line through it."),
        Segment: ("en", "Segment",
            "Hilbert (1899) Group III: the portion of a line between two distinct points; subject to the congruence axioms."),
        Vector: ("en", "Vector",
            "Kahan, Axioms for Fields and Vector Spaces: an element of a vector space — a directed displacement subject to the eight vector-space axioms (closure, associativity, identity, inverse, scalar compatibility, etc.)."),
        Angle: ("en", "Angle",
            "Hilbert (1899) Group III: the figure formed by two rays sharing an endpoint; subject to the congruence axioms for angles."),
        Triangle: ("en", "Triangle",
            "Euclid, Elements Book I Definition 19: a rectilineal figure contained by three straight lines — a triple of non-collinear points and the segments connecting them."),
        Circle: ("en", "Circle",
            "Euclid, Elements Book I Definition 15: a plane figure contained by one line such that all straight lines drawn from a single interior point to it are equal."),
        Sphere: ("en", "Sphere",
            "Euclid, Elements Book XI Definition 14: a solid figure described by the revolution of a semicircle about its diameter — the 3D analogue of the circle."),
    },

    edges: [
        // Hilbert Group I — Incidence. Points lie on lines and planes;
        // lines lie in planes. Hilbert (1899) §3 axioms I.1-I.8.
        (Point, Line, Incidence),
        (Point, Plane, Incidence),
        (Line, Plane, Incidence),
        (Point, Segment, Incidence),
        (Point, Circle, Incidence),
        (Point, Sphere, Incidence),

        // Hilbert Group II — Order / Betweenness (axioms II.1-II.4).
        // Of three collinear points, one lies between the other two.
        (Point, Point, Betweenness),

        // Hilbert Group III — Congruence (axioms III.1-III.5).
        // Segments, angles, and triangles bear a congruence relation
        // to other figures of the same kind.
        (Segment, Segment, Congruence),
        (Angle, Angle, Congruence),
        (Triangle, Triangle, Congruence),

        // Containment (lattice-theoretic; complements incidence).
        // A plane contains its incident lines and points; a line
        // contains its incident points. Modelled distinctly from
        // incidence to preserve the directional reading.
        (Plane, Line, Containment),
        (Plane, Point, Containment),
        (Line, Point, Containment),

        // Hilbert Group IV — Parallels (Playfair's axiom, Hilbert §6).
        // Coplanar lines that do not intersect.
        (Line, Line, Parallelism),
        (Plane, Plane, Parallelism),

        // Perpendicularity — derived from Hilbert congruence axioms
        // (Avigad et al. 2009 §4): two lines / planes meet at a right
        // angle when one of the four angles formed is congruent to its
        // supplement.
        (Line, Line, Perpendicularity),
        (Plane, Plane, Perpendicularity),
        (Line, Plane, Perpendicularity),

        // Construction — Euclid's Elements proceeds by constructing
        // derived objects from primitives (e.g., a triangle is the
        // construction of three segments between three points).
        (Point, Triangle, Construction),
        (Point, Circle, Construction),
        (Point, Sphere, Construction),
    ],

    composed: [
        // Hilbert's transitivity of incidence: a point on a line in a
        // plane is itself in the plane (Avigad et al. 2009 §3).
        (Point, Plane),
    ],
}

/// Quality: topological dimension of each geometric primitive.
///
/// Hilbert (1899) treats the primitive notions as abstract (no
/// intrinsic dimension), but downstream uses (Avigad et al. 2009 §2)
/// assign each kind the standard topological dimension. Angle is
/// scalar-valued (dimension 0) by convention.
#[derive(Debug, Clone)]
pub struct GeometricDimension;

impl Quality for GeometricDimension {
    type Individual = EuclideanGeometryConcept;
    type Value = usize;

    fn get(&self, prim: &EuclideanGeometryConcept) -> Option<usize> {
        use EuclideanGeometryConcept as E;
        Some(match prim {
            E::Point => 0,
            E::Line | E::Ray | E::Segment | E::Vector => 1,
            E::Plane | E::Triangle | E::Circle => 2,
            E::Sphere => 2, // 2D manifold embedded in 3D
            E::Angle => 0,  // scalar measure
        })
    }
}

/// Quality: degrees of freedom in 3D Euclidean space.
///
/// Standard parameter counts for each kind of figure embedded in R³ —
/// Avigad et al. (2009) §2 follows the same convention.
#[derive(Debug, Clone)]
pub struct DegreesOfFreedom;

impl Quality for DegreesOfFreedom {
    type Individual = EuclideanGeometryConcept;
    type Value = usize;

    fn get(&self, prim: &EuclideanGeometryConcept) -> Option<usize> {
        use EuclideanGeometryConcept as E;
        Some(match prim {
            E::Point => 3,    // x, y, z
            E::Line => 4,     // point + direction (4 DOF in R³)
            E::Ray => 5,      // origin + direction
            E::Segment => 6,  // two endpoints
            E::Plane => 3,    // normal + offset
            E::Angle => 1,    // single scalar
            E::Triangle => 9, // three vertices
            E::Circle => 4,   // centre + radius (in a plane)
            E::Sphere => 4,   // centre + radius
            E::Vector => 3,   // x, y, z
        })
    }
}

impl Ontology for EuclideanGeometryOntology {
    type Cat = EuclideanGeometryCategory;
    type Qual = GeometricDimension;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(MetricNonNegativity));
        axioms.push(Box::new(MetricIdentity));
        axioms.push(Box::new(MetricSymmetry));
        axioms.push(Box::new(TriangleInequality));
        axioms.push(Box::new(TriangleAngleSum));
        axioms.push(Box::new(PythagoreanTheorem));
        axioms.push(Box::new(VectorAdditionCommutativity));
        axioms.push(Box::new(VectorAdditionAssociativity));
        axioms.push(Box::new(DotProductCommutativity));
        axioms.push(Box::new(CrossProductAnticommutativity));
        axioms.push(Box::new(CrossProductPerpendicularity));
        axioms.push(Box::new(ProjectionIdempotent));
        axioms.push(Box::new(BetweennessSymmetry));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Domain axioms — metric space, vector space, inner-product, Hilbert
// ---------------------------------------------------------------------------

/// Metric-space axiom (M1): d(a, b) ≥ 0 for all a, b.
///
/// Verified by enumeration over canonical 3D points; the geometric
/// definition of distance via `Point3::distance_to` is non-negative
/// by construction (Euclidean norm of a difference).
pub struct MetricNonNegativity;

impl Axiom for MetricNonNegativity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for a in &canonical_points_3d() {
            for b in &canonical_points_3d() {
                if a.distance_to(b).value < -1e-15 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MetricNonNegativity",
        "metric axiom M1: d(a,b) >= 0 (non-negativity)",
        "Hilbert (1899) Grundlagen der Geometrie; standard metric-space formulation (Frechet 1906)"
    );
}

pr4xis::register_axiom!(
    MetricNonNegativity,
    "Hilbert (1899) Grundlagen der Geometrie; standard metric-space formulation (Frechet 1906)"
);

/// Metric-space axiom (M2): d(a, b) = 0 iff a = b — identity of
/// indiscernibles.
pub struct MetricIdentity;

impl Axiom for MetricIdentity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let pts = canonical_points_3d();
        for a in &pts {
            if a.distance_to(a).value > 1e-15 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            for b in &pts {
                if a != b && a.distance_to(b).value < 1e-15 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MetricIdentity",
        "metric axiom M2: d(a,b) = 0 iff a = b (identity of indiscernibles)",
        "Hilbert (1899) Grundlagen der Geometrie; standard metric-space formulation (Frechet 1906)"
    );
}

pr4xis::register_axiom!(
    MetricIdentity,
    "Hilbert (1899) Grundlagen der Geometrie; standard metric-space formulation (Frechet 1906)"
);

/// Metric-space axiom (M3): d(a, b) = d(b, a) — symmetry.
pub struct MetricSymmetry;

impl Axiom for MetricSymmetry {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let pts = canonical_points_3d();
        for a in &pts {
            for b in &pts {
                if (a.distance_to(b).value - b.distance_to(a).value).abs() > 1e-15 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MetricSymmetry",
        "metric axiom M3: d(a,b) = d(b,a) (symmetry)",
        "Hilbert (1899) Grundlagen der Geometrie; standard metric-space formulation (Frechet 1906)"
    );
}

pr4xis::register_axiom!(
    MetricSymmetry,
    "Hilbert (1899) Grundlagen der Geometrie; standard metric-space formulation (Frechet 1906)"
);

/// Metric-space axiom (M4): d(a, c) ≤ d(a, b) + d(b, c) — the triangle
/// inequality. Equivalent (in Euclidean space) to Hilbert Group III
/// congruence axioms reading distance from segment congruence.
pub struct TriangleInequality;

impl Axiom for TriangleInequality {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let pts = canonical_points_3d();
        for a in &pts {
            for b in &pts {
                for c in &pts {
                    if a.distance_to(c).value
                        > a.distance_to(b).value + b.distance_to(c).value + 1e-10
                    {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "TriangleInequality",
        "metric axiom M4: d(a,c) <= d(a,b) + d(b,c)",
        "Hilbert (1899) Grundlagen der Geometrie; standard metric-space formulation (Frechet 1906)"
    );
}

pr4xis::register_axiom!(
    TriangleInequality,
    "Hilbert (1899) Grundlagen der Geometrie; standard metric-space formulation (Frechet 1906)"
);

/// Euclidean theorem (Euclid, Elements Book I Proposition 32):
/// the three interior angles of a triangle sum to π (two right angles).
/// This is equivalent in Euclidean geometry to Hilbert's parallel
/// postulate (Hilbert Group IV).
pub struct TriangleAngleSum;

impl Axiom for TriangleAngleSum {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for t in &canonical_triangles() {
            if t.is_degenerate() {
                continue;
            }
            if (t.angle_sum() - PI).abs() > 1e-9 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "TriangleAngleSum",
        "Euclid Elements I.32: interior angles of a triangle sum to pi",
        "Euclid, Elements Book I Proposition 32; Hilbert (1899) Group IV parallel postulate"
    );
}

pr4xis::register_axiom!(
    TriangleAngleSum,
    "Euclid, Elements Book I Proposition 32; Hilbert (1899) Group IV parallel postulate"
);

/// Euclidean theorem (Euclid, Elements Book I Proposition 47): in a
/// right triangle with legs a, b and hypotenuse c, a² + b² = c².
pub struct PythagoreanTheorem;

impl Axiom for PythagoreanTheorem {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let right_triangles = vec![
            Triangle::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(3.0, 0.0, 0.0),
                Point3::new(0.0, 4.0, 0.0),
            ),
            Triangle::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(5.0, 0.0, 0.0),
                Point3::new(0.0, 12.0, 0.0),
            ),
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
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "PythagoreanTheorem",
        "Euclid Elements I.47: a^2 + b^2 = c^2 for right triangles",
        "Euclid, Elements Book I Proposition 47"
    );
}

pr4xis::register_axiom!(PythagoreanTheorem, "Euclid, Elements Book I Proposition 47");

/// Vector-space axiom (V2): u + v = v + u — commutativity of vector
/// addition (Kahan, *Axioms for Fields and Vector Spaces*).
pub struct VectorAdditionCommutativity;

impl Axiom for VectorAdditionCommutativity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let vecs = canonical_vectors_3d();
        for u in &vecs {
            for v in &vecs {
                let uv = u.add(v);
                let vu = v.add(u);
                if (uv.x - vu.x).abs() > 1e-15
                    || (uv.y - vu.y).abs() > 1e-15
                    || (uv.z - vu.z).abs() > 1e-15
                {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "VectorAdditionCommutativity",
        "vector-space axiom V2: u + v = v + u (commutativity of addition)",
        "Kahan, Axioms for Fields and Vector Spaces"
    );
}

pr4xis::register_axiom!(
    VectorAdditionCommutativity,
    "Kahan, Axioms for Fields and Vector Spaces"
);

/// Vector-space axiom (V1): (u + v) + w = u + (v + w) — associativity
/// of vector addition.
pub struct VectorAdditionAssociativity;

impl Axiom for VectorAdditionAssociativity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
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
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "VectorAdditionAssociativity",
        "vector-space axiom V1: (u+v)+w = u+(v+w) (associativity of addition)",
        "Kahan, Axioms for Fields and Vector Spaces"
    );
}

pr4xis::register_axiom!(
    VectorAdditionAssociativity,
    "Kahan, Axioms for Fields and Vector Spaces"
);

/// Inner-product property: a · b = b · a — commutativity of the
/// standard real-valued dot product on R³.
pub struct DotProductCommutativity;

impl Axiom for DotProductCommutativity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let vecs = canonical_vectors_3d();
        for a in &vecs {
            for b in &vecs {
                if (a.dot(b) - b.dot(a)).abs() > 1e-15 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DotProductCommutativity",
        "inner-product symmetry on R^3: a . b = b . a",
        "Kahan, Axioms for Fields and Vector Spaces (inner-product axioms)"
    );
}

pr4xis::register_axiom!(
    DotProductCommutativity,
    "Kahan, Axioms for Fields and Vector Spaces (inner-product axioms)"
);

/// Cross-product property: a × b = −(b × a) — anticommutativity of
/// the standard cross product on R³.
pub struct CrossProductAnticommutativity;

impl Axiom for CrossProductAnticommutativity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let vecs = canonical_vectors_3d();
        for a in &vecs {
            for b in &vecs {
                let ab = a.cross(b);
                let ba = b.cross(a).negate();
                if (ab.x - ba.x).abs() > 1e-15
                    || (ab.y - ba.y).abs() > 1e-15
                    || (ab.z - ba.z).abs() > 1e-15
                {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "CrossProductAnticommutativity",
        "cross product is anticommutative: a x b = -(b x a)",
        "Kahan, Axioms for Fields and Vector Spaces"
    );
}

pr4xis::register_axiom!(
    CrossProductAnticommutativity,
    "Kahan, Axioms for Fields and Vector Spaces"
);

/// Cross-product property: (a × b) · a = 0 — the cross product is
/// orthogonal to both factors.
pub struct CrossProductPerpendicularity;

impl Axiom for CrossProductPerpendicularity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let vecs = canonical_vectors_3d();
        for a in &vecs {
            for b in &vecs {
                let cross = a.cross(b);
                if cross.dot(a).abs() > 1e-10 || cross.dot(b).abs() > 1e-10 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "CrossProductPerpendicularity",
        "cross product is perpendicular to both inputs: (a x b) . a = 0",
        "Kahan, Axioms for Fields and Vector Spaces"
    );
}

pr4xis::register_axiom!(
    CrossProductPerpendicularity,
    "Kahan, Axioms for Fields and Vector Spaces"
);

/// Projection property: proj_b(proj_b(a)) = proj_b(a) — projection
/// onto a direction is idempotent (orthogonal projection is a
/// projection operator P with P² = P).
pub struct ProjectionIdempotent;

impl Axiom for ProjectionIdempotent {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let vecs = canonical_vectors_3d();
        for a in &vecs {
            for b in &vecs {
                if b.norm().value < 1e-15 {
                    continue;
                }
                let p1 = projection::project_vector_onto_vector(a, b);
                let p2 = projection::project_vector_onto_vector(&p1, b);
                if (p1.x - p2.x).abs() > 1e-10
                    || (p1.y - p2.y).abs() > 1e-10
                    || (p1.z - p2.z).abs() > 1e-10
                {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ProjectionIdempotent",
        "vector projection is idempotent: proj_b(proj_b(a)) = proj_b(a)",
        "Kahan, Axioms for Fields and Vector Spaces (orthogonal projection)"
    );
}

pr4xis::register_axiom!(
    ProjectionIdempotent,
    "Kahan, Axioms for Fields and Vector Spaces (orthogonal projection)"
);

/// Hilbert Group II axiom (Order/Betweenness): if B lies between A
/// and C, then B also lies between C and A — betweenness is symmetric
/// in the outer two points. Hilbert (1899) axiom II.1.
pub struct BetweennessSymmetry;

impl Axiom for BetweennessSymmetry {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(1.0, 0.0, 0.0);
        let c = Point3::new(2.0, 0.0, 0.0);
        if a.is_between(&b, &c) == c.is_between(&b, &a) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BetweennessSymmetry",
        "Hilbert II.1: if B is between A and C, then B is between C and A",
        "Hilbert (1899) Grundlagen der Geometrie, Group II (Order) axiom II.1"
    );
}

pr4xis::register_axiom!(
    BetweennessSymmetry,
    "Hilbert (1899) Grundlagen der Geometrie, Group II (Order) axiom II.1"
);

// ---------------------------------------------------------------------------
// Canonical test data — small representative sets for axiom enumeration.
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
        Triangle::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(3.0, 0.0, 0.0),
            Point3::new(0.0, 4.0, 0.0),
        ),
        Triangle::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.5, 0.866_025_403_784_438_6, 0.0),
        ),
        Triangle::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            Point3::new(1.0, 3.0, 0.0),
        ),
        Triangle::new(
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 2.0, 0.0),
            Point3::new(0.0, 0.0, 3.0),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<EuclideanGeometryCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        EuclideanGeometryOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ten_geometric_concepts() {
        // Hilbert's three primitives + Ray/Segment/Vector + Angle/Triangle/Circle/Sphere.
        assert_eq!(EuclideanGeometryConcept::variants().len(), 10);
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn dimension_quality_total() {
        let q = GeometricDimension;
        for c in EuclideanGeometryConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing dimension", c);
        }
    }

    #[pr4xis::praxis_value(Explainable, Verifiable)]
    #[test]
    fn dof_quality_total() {
        let q = DegreesOfFreedom;
        for c in EuclideanGeometryConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing DOF", c);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn hilbert_incidence_present() {
        // Hilbert Group I: Point on Line, Point on Plane, Line in Plane.
        let inc: Vec<_> = EuclideanGeometryCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == EuclideanGeometryRelationKind::Incidence)
            .map(|m| (m.source(), m.target()))
            .collect();
        use EuclideanGeometryConcept as E;
        assert!(inc.contains(&(E::Point, E::Line)));
        assert!(inc.contains(&(E::Point, E::Plane)));
        assert!(inc.contains(&(E::Line, E::Plane)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn metric_axioms_hold() {
        assert!(MetricNonNegativity.verify().is_ok());
        assert!(MetricIdentity.verify().is_ok());
        assert!(MetricSymmetry.verify().is_ok());
        assert!(TriangleInequality.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn euclidean_theorems_hold() {
        assert!(TriangleAngleSum.verify().is_ok());
        assert!(PythagoreanTheorem.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn vector_space_axioms_hold() {
        assert!(VectorAdditionCommutativity.verify().is_ok());
        assert!(VectorAdditionAssociativity.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn inner_and_cross_product_laws_hold() {
        assert!(DotProductCommutativity.verify().is_ok());
        assert!(CrossProductAnticommutativity.verify().is_ok());
        assert!(CrossProductPerpendicularity.verify().is_ok());
    }

    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn projection_idempotent_holds() {
        assert!(ProjectionIdempotent.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn betweenness_symmetry_holds() {
        assert!(BetweennessSymmetry.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = EuclideanGeometryConcept> {
        proptest::sample::select(EuclideanGeometryConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_dimension_total(c in arb_concept()) {
            prop_assert!(GeometricDimension.get(&c).is_some());
        }

        #[test]
        fn prop_dof_total(c in arb_concept()) {
            prop_assert!(DegreesOfFreedom.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in EuclideanGeometryCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in EuclideanGeometryOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_dimension_total, Explainable, Verifiable);
    pr4xis::register_praxis_value!(prop_dof_total, Explainable, Verifiable);
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable, Honest);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
