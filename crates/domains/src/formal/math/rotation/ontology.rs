//! The rotation representation category — representations of SO(3).
//!
//! Source: Hamilton (1844); Shoemake (1985).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::rotation::dcm::Dcm;
use crate::formal::math::rotation::quaternion::Quaternion;

pr4xis::ontology! {
    name: "Rotation",
    source: "Hamilton (1844); Shoemake (1985)",

    concepts: [Quaternion, DCM, Euler, AxisAngle],

    labels: {
        Quaternion: ("en", "Quaternion", "Unit quaternion (4 parameters, 1 constraint)."),
        DCM: ("en", "Direction Cosine Matrix", "Direction Cosine Matrix (9 parameters, 6 constraints)."),
        Euler: ("en", "Euler angles", "Euler angles with sequence (3 parameters, gimbal lock singularity)."),
        AxisAngle: ("en", "Axis-angle", "Axis-angle (4 parameters, 1 constraint)."),
    },
}

/// Quality: number of parameters in the representation.
#[derive(Debug, Clone)]
pub struct ParameterCount;

impl Quality for ParameterCount {
    type Individual = RotationConcept;
    type Value = usize;

    fn get(&self, repr: &RotationConcept) -> Option<usize> {
        Some(match repr {
            RotationConcept::Quaternion => 4,
            RotationConcept::DCM => 9,
            RotationConcept::Euler => 3,
            RotationConcept::AxisAngle => 4,
        })
    }
}

/// Quality: whether the representation has singularities.
#[derive(Debug, Clone)]
pub struct HasSingularity;

impl Quality for HasSingularity {
    type Individual = RotationConcept;
    type Value = bool;

    fn get(&self, repr: &RotationConcept) -> Option<bool> {
        Some(match repr {
            RotationConcept::Quaternion => false,
            RotationConcept::DCM => false,
            RotationConcept::Euler => true,
            RotationConcept::AxisAngle => true,
        })
    }
}

/// Axiom: quaternion composition preserves unit norm (closure in SO(3)).
pub struct UnitNormClosure;

impl Axiom for UnitNormClosure {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let rotations = canonical_rotations();
        for a in &rotations {
            for b in &rotations {
                let c = a.multiply(b);
                if (c.norm() - 1.0).abs() > 1e-10 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "UnitNormClosure",
        "quaternion multiplication preserves unit norm (SO(3) closure)",
        "Hamilton (1844); Shoemake (1985)."
    );
}
pr4xis::register_axiom!(UnitNormClosure, "Hamilton (1844); Shoemake (1985).");

/// Axiom: rotation composition is associative.
pub struct Associativity;

impl Axiom for Associativity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let rotations = canonical_rotations();
        for a in &rotations {
            for b in &rotations {
                for c in &rotations {
                    let ab_c = a.multiply(b).multiply(c);
                    let a_bc = a.multiply(&b.multiply(c));
                    if ab_c != a_bc {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "Associativity",
        "rotation composition is associative: (a*b)*c = a*(b*c)",
        "Hamilton (1844); Shoemake (1985)."
    );
}
pr4xis::register_axiom!(Associativity, "Hamilton (1844); Shoemake (1985).");

/// Axiom: identity rotation is the neutral element.
pub struct IdentityElement;

impl Axiom for IdentityElement {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let id = Quaternion::identity();
        for q in &canonical_rotations() {
            if q.multiply(&id) != *q || id.multiply(q) != *q {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "IdentityElement",
        "identity quaternion is the neutral element: q*I = I*q = q",
        "Hamilton (1844); Shoemake (1985)."
    );
}
pr4xis::register_axiom!(IdentityElement, "Hamilton (1844); Shoemake (1985).");

/// Axiom: every rotation has an inverse such that q * q^{-1} = I.
pub struct InverseExists;

impl Axiom for InverseExists {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let id = Quaternion::identity();
        for q in &canonical_rotations() {
            if q.multiply(&q.inverse()) != id {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "InverseExists",
        "every rotation has an inverse: q * q^{-1} = identity",
        "Hamilton (1844); Shoemake (1985)."
    );
}
pr4xis::register_axiom!(InverseExists, "Hamilton (1844); Shoemake (1985).");

/// Axiom: DCM from quaternion is a proper rotation (R^T R = I, det = +1).
pub struct DcmOrthogonality;

impl Axiom for DcmOrthogonality {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for q in &canonical_rotations() {
            let dcm = Dcm::from_quaternion(q);
            if !dcm.is_proper_rotation(1e-10) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "DcmOrthogonality",
        "quaternion-to-DCM produces proper rotation: R^T R = I, det(R) = +1",
        "Hamilton (1844); Shoemake (1985)."
    );
}
pr4xis::register_axiom!(DcmOrthogonality, "Hamilton (1844); Shoemake (1985).");

/// Axiom: quaternion -> DCM -> quaternion roundtrip preserves the rotation.
pub struct QuaternionDcmRoundtrip;

impl Axiom for QuaternionDcmRoundtrip {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for q in &canonical_rotations() {
            let dcm = Dcm::from_quaternion(q);
            let q2 = dcm.to_quaternion();
            if *q != q2 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "QuaternionDcmRoundtrip",
        "quaternion -> DCM -> quaternion roundtrip preserves rotation",
        "Hamilton (1844); Shoemake (1985)."
    );
}
pr4xis::register_axiom!(QuaternionDcmRoundtrip, "Hamilton (1844); Shoemake (1985).");

impl Ontology for RotationOntology {
    type Cat = RotationCategory;
    type Qual = ParameterCount;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(UnitNormClosure));
        axioms.push(Box::new(Associativity));
        axioms.push(Box::new(IdentityElement));
        axioms.push(Box::new(InverseExists));
        axioms.push(Box::new(DcmOrthogonality));
        axioms.push(Box::new(QuaternionDcmRoundtrip));
        axioms
    }
}

/// Canonical rotations for axiom verification.
fn canonical_rotations() -> Vec<Quaternion> {
    use core::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        pr4xis::category::laws::assert_category_laws::<RotationCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        RotationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
