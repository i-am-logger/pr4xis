use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::science::math::rotation::dcm::Dcm;
use crate::science::math::rotation::quaternion::Quaternion;

/// The representations of rotation — objects in the rotation category.
///
/// Each representation captures the same SO(3) element through a different
/// mathematical formalism. The morphisms between them are the conversion
/// functions (which preserve the rotation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RotationRepr {
    /// Unit quaternion (4 parameters, 1 constraint)
    Quaternion,
    /// Direction Cosine Matrix (9 parameters, 6 constraints)
    DCM,
    /// Euler angles with sequence (3 parameters, gimbal lock singularity)
    Euler,
    /// Axis-angle (4 parameters, 1 constraint)
    AxisAngle,
}

impl Entity for RotationRepr {
    fn variants() -> Vec<Self> {
        vec![Self::Quaternion, Self::DCM, Self::Euler, Self::AxisAngle]
    }
}

/// A conversion between rotation representations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReprConversion {
    pub from: RotationRepr,
    pub to: RotationRepr,
}

impl Relationship for ReprConversion {
    type Object = RotationRepr;

    fn source(&self) -> RotationRepr {
        self.from
    }

    fn target(&self) -> RotationRepr {
        self.to
    }
}

/// The rotation representation category.
///
/// Objects: rotation representations (quaternion, DCM, Euler, axis-angle).
/// Morphisms: conversions between any pair (indiscrete category).
pub struct RotationCategory;

impl Category for RotationCategory {
    type Object = RotationRepr;
    type Morphism = ReprConversion;

    fn identity(obj: &RotationRepr) -> ReprConversion {
        ReprConversion {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &ReprConversion, g: &ReprConversion) -> Option<ReprConversion> {
        if f.to != g.from {
            return None;
        }
        Some(ReprConversion {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<ReprConversion> {
        let reprs = RotationRepr::variants();
        reprs
            .iter()
            .flat_map(|&from| reprs.iter().map(move |&to| ReprConversion { from, to }))
            .collect()
    }
}

/// Quality: number of parameters in the representation.
#[derive(Debug, Clone)]
pub struct ParameterCount;

impl Quality for ParameterCount {
    type Individual = RotationRepr;
    type Value = usize;

    fn get(&self, repr: &RotationRepr) -> Option<usize> {
        Some(match repr {
            RotationRepr::Quaternion => 4,
            RotationRepr::DCM => 9,
            RotationRepr::Euler => 3,
            RotationRepr::AxisAngle => 4,
        })
    }
}

/// Quality: whether the representation has singularities.
#[derive(Debug, Clone)]
pub struct HasSingularity;

impl Quality for HasSingularity {
    type Individual = RotationRepr;
    type Value = bool;

    fn get(&self, repr: &RotationRepr) -> Option<bool> {
        Some(match repr {
            RotationRepr::Quaternion => false,
            RotationRepr::DCM => false,
            RotationRepr::Euler => true,     // gimbal lock
            RotationRepr::AxisAngle => true, // undefined axis at zero angle
        })
    }
}

// --- SO(3) Group Axioms (parameterized by RotationCategory) ---

/// Axiom: quaternion composition preserves unit norm (closure in SO(3)).
pub struct UnitNormClosure;

impl Axiom for UnitNormClosure {
    fn description(&self) -> &str {
        "quaternion multiplication preserves unit norm (SO(3) closure)"
    }

    fn holds(&self) -> bool {
        let rotations = canonical_rotations();
        for a in &rotations {
            for b in &rotations {
                let c = a.multiply(b);
                if (c.norm() - 1.0).abs() > 1e-10 {
                    return false;
                }
            }
        }
        true
    }
}

/// Axiom: rotation composition is associative.
pub struct Associativity;

impl Axiom for Associativity {
    fn description(&self) -> &str {
        "rotation composition is associative: (a*b)*c = a*(b*c)"
    }

    fn holds(&self) -> bool {
        let rotations = canonical_rotations();
        for a in &rotations {
            for b in &rotations {
                for c in &rotations {
                    let ab_c = a.multiply(b).multiply(c);
                    let a_bc = a.multiply(&b.multiply(c));
                    if ab_c != a_bc {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Axiom: identity rotation is the neutral element.
pub struct IdentityElement;

impl Axiom for IdentityElement {
    fn description(&self) -> &str {
        "identity quaternion is the neutral element: q*I = I*q = q"
    }

    fn holds(&self) -> bool {
        let id = Quaternion::identity();
        for q in &canonical_rotations() {
            if q.multiply(&id) != *q || id.multiply(q) != *q {
                return false;
            }
        }
        true
    }
}

/// Axiom: every rotation has an inverse such that q * q^{-1} = I.
pub struct InverseExists;

impl Axiom for InverseExists {
    fn description(&self) -> &str {
        "every rotation has an inverse: q * q^{-1} = identity"
    }

    fn holds(&self) -> bool {
        let id = Quaternion::identity();
        for q in &canonical_rotations() {
            if q.multiply(&q.inverse()) != id {
                return false;
            }
        }
        true
    }
}

/// Axiom: DCM from quaternion is a proper rotation (R^T R = I, det = +1).
pub struct DcmOrthogonality;

impl Axiom for DcmOrthogonality {
    fn description(&self) -> &str {
        "quaternion-to-DCM produces proper rotation: R^T R = I, det(R) = +1"
    }

    fn holds(&self) -> bool {
        for q in &canonical_rotations() {
            let dcm = Dcm::from_quaternion(q);
            if !dcm.is_proper_rotation(1e-10) {
                return false;
            }
        }
        true
    }
}

/// Axiom: quaternion -> DCM -> quaternion roundtrip preserves the rotation.
pub struct QuaternionDcmRoundtrip;

impl Axiom for QuaternionDcmRoundtrip {
    fn description(&self) -> &str {
        "quaternion -> DCM -> quaternion roundtrip preserves rotation"
    }

    fn holds(&self) -> bool {
        for q in &canonical_rotations() {
            let dcm = Dcm::from_quaternion(q);
            let q2 = dcm.to_quaternion();
            if *q != q2 {
                return false;
            }
        }
        true
    }
}

/// The rotation ontology — SO(3) as a praxis ontology.
pub struct RotationOntology;

impl Ontology for RotationOntology {
    type Cat = RotationCategory;
    type Qual = ParameterCount;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(UnitNormClosure),
            Box::new(Associativity),
            Box::new(IdentityElement),
            Box::new(InverseExists),
            Box::new(DcmOrthogonality),
            Box::new(QuaternionDcmRoundtrip),
        ]
    }
}

/// Canonical rotations for axiom verification.
fn canonical_rotations() -> Vec<Quaternion> {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
    vec![
        Quaternion::identity(),
        Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2),
        Quaternion::from_axis_angle([0.0, 1.0, 0.0], FRAC_PI_2),
        Quaternion::from_axis_angle([0.0, 0.0, 1.0], FRAC_PI_2),
        Quaternion::from_axis_angle([1.0, 0.0, 0.0], PI),
        Quaternion::from_axis_angle([0.0, 1.0, 0.0], PI),
        Quaternion::from_axis_angle([0.0, 0.0, 1.0], PI),
        Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_4),
        Quaternion::from_axis_angle([0.0, 1.0, 0.0], FRAC_PI_4),
        Quaternion::from_axis_angle([0.0, 0.0, 1.0], FRAC_PI_4),
        {
            let s = 1.0 / 3.0_f64.sqrt();
            Quaternion::from_axis_angle([s, s, s], 2.0 * PI / 3.0)
        },
    ]
}
