#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::rotation::ontology::RotationCategory;
use crate::formal::math::rotation::quaternion::Quaternion;
use pr4xis::ontology::{Axiom, Ontology};

use crate::formal::math::rigid_motion::pose::Pose;

/// Axiom: SE(3) composition is associative.
pub struct Associativity;

impl Axiom for Associativity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let poses = canonical_poses();
        for a in &poses {
            for b in &poses {
                for c in &poses {
                    let ab_c = a.compose(b).compose(c);
                    let a_bc = a.compose(&b.compose(c));
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
        "SE(3) composition is associative: (A*B)*C = A*(B*C)",
        "Murray, Li & Sastry (1994) A Mathematical Introduction to Robotic Manipulation; Lynch & Park (2017) Modern Robotics"
    );
}
pr4xis::register_axiom!(
    Associativity,
    "Murray, Li & Sastry (1994) A Mathematical Introduction to Robotic Manipulation; Lynch & Park (2017) Modern Robotics"
);

/// Axiom: identity pose is the neutral element.
pub struct IdentityElement;

impl Axiom for IdentityElement {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let id = Pose::identity();
        for p in &canonical_poses() {
            if p.compose(&id) != *p || id.compose(p) != *p {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "IdentityElement",
        "identity pose is the neutral element",
        "Murray, Li & Sastry (1994) A Mathematical Introduction to Robotic Manipulation; Lynch & Park (2017) Modern Robotics"
    );
}
pr4xis::register_axiom!(
    IdentityElement,
    "Murray, Li & Sastry (1994) A Mathematical Introduction to Robotic Manipulation; Lynch & Park (2017) Modern Robotics"
);

/// Axiom: every pose has an inverse such that T * T^{-1} = I.
pub struct InverseExists;

impl Axiom for InverseExists {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let id = Pose::identity();
        for p in &canonical_poses() {
            if p.compose(&p.inverse()) != id {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "InverseExists",
        "every SE(3) element has an inverse: T * T^{-1} = identity",
        "Murray, Li & Sastry (1994) A Mathematical Introduction to Robotic Manipulation; Lynch & Park (2017) Modern Robotics"
    );
}
pr4xis::register_axiom!(
    InverseExists,
    "Murray, Li & Sastry (1994) A Mathematical Introduction to Robotic Manipulation; Lynch & Park (2017) Modern Robotics"
);

/// Axiom: composing poses then transforming equals sequential transforms.
pub struct CompositionConsistency;

impl Axiom for CompositionConsistency {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let poses = canonical_poses();
        let test_points = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 2.0, 3.0],
        ];
        for a in &poses {
            for b in &poses {
                let ab = a.compose(b);
                for p in &test_points {
                    let direct = ab.transform_point(*p);
                    let sequential = b.transform_point(a.transform_point(*p));
                    let tol = 1e-9;
                    if (direct[0] - sequential[0]).abs() > tol
                        || (direct[1] - sequential[1]).abs() > tol
                        || (direct[2] - sequential[2]).abs() > tol
                    {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "CompositionConsistency",
        "composing poses then transforming equals sequential transforms",
        "Murray, Li & Sastry (1994) A Mathematical Introduction to Robotic Manipulation; Lynch & Park (2017) Modern Robotics"
    );
}
pr4xis::register_axiom!(
    CompositionConsistency,
    "Murray, Li & Sastry (1994) A Mathematical Introduction to Robotic Manipulation; Lynch & Park (2017) Modern Robotics"
);

/// The rigid motion ontology — SE(3) group axioms.
///
/// Uses RotationCategory as the underlying category since SE(3)
/// extends SO(3) with translations.
pub struct RigidMotionOntology;

impl Ontology for RigidMotionOntology {
    type Cat = RotationCategory;
    type Qual = crate::formal::math::rotation::ontology::ParameterCount;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(Associativity),
            Box::new(IdentityElement),
            Box::new(InverseExists),
            Box::new(CompositionConsistency),
        ]
    }
}

/// Canonical poses for axiom verification.
fn canonical_poses() -> Vec<Pose> {
    use core::f64::consts::{FRAC_PI_2, FRAC_PI_4};
    vec![
        Pose::identity(),
        Pose::from_translation([1.0, 0.0, 0.0]),
        Pose::from_translation([0.0, 2.0, 0.0]),
        Pose::from_translation([0.0, 0.0, 3.0]),
        Pose::from_translation([1.0, 2.0, 3.0]),
        Pose::from_rotation(Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_2)),
        Pose::from_rotation(Quaternion::from_axis_angle([0.0, 1.0, 0.0], FRAC_PI_4)),
        Pose {
            rotation: Quaternion::from_axis_angle([0.0, 0.0, 1.0], FRAC_PI_2),
            translation: [1.0, 2.0, 3.0],
        },
        Pose {
            rotation: Quaternion::from_axis_angle([1.0, 0.0, 0.0], FRAC_PI_4),
            translation: [-1.0, 0.5, 2.0],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        RigidMotionOntology::validate().unwrap();
    }
}
