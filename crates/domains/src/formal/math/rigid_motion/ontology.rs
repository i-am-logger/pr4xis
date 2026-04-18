use crate::formal::math::rotation::ontology::RotationCategory;
use crate::formal::math::rotation::quaternion::Quaternion;
use pr4xis::ontology::{Axiom, Ontology};

use crate::formal::math::rigid_motion::pose::Pose;

/// Axiom: SE(3) composition is associative.
pub struct Associativity;

impl Axiom for Associativity {
    fn description(&self) -> &str {
        "SE(3) composition is associative: (A*B)*C = A*(B*C)"
    }

    fn holds(&self) -> bool {
        let poses = canonical_poses();
        for a in &poses {
            for b in &poses {
                for c in &poses {
                    let ab_c = a.compose(b).compose(c);
                    let a_bc = a.compose(&b.compose(c));
                    if ab_c != a_bc {
                        return false;
                    }
                }
            }
        }
        true
    }
}
pr4xis::register_axiom!(Associativity);

/// Axiom: identity pose is the neutral element.
pub struct IdentityElement;

impl Axiom for IdentityElement {
    fn description(&self) -> &str {
        "identity pose is the neutral element"
    }

    fn holds(&self) -> bool {
        let id = Pose::identity();
        for p in &canonical_poses() {
            if p.compose(&id) != *p || id.compose(p) != *p {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(IdentityElement);

/// Axiom: every pose has an inverse such that T * T^{-1} = I.
pub struct InverseExists;

impl Axiom for InverseExists {
    fn description(&self) -> &str {
        "every SE(3) element has an inverse: T * T^{-1} = identity"
    }

    fn holds(&self) -> bool {
        let id = Pose::identity();
        for p in &canonical_poses() {
            if p.compose(&p.inverse()) != id {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(InverseExists);

/// Axiom: composing poses then transforming equals sequential transforms.
pub struct CompositionConsistency;

impl Axiom for CompositionConsistency {
    fn description(&self) -> &str {
        "composing poses then transforming equals sequential transforms"
    }

    fn holds(&self) -> bool {
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
                        return false;
                    }
                }
            }
        }
        true
    }
}
pr4xis::register_axiom!(CompositionConsistency);

/// The rigid motion ontology — SE(3) group axioms.
///
/// Uses RotationCategory as the underlying category since SE(3)
/// extends SO(3) with translations.
pub struct RigidMotionOntology;

impl Ontology for RigidMotionOntology {
    type Cat = RotationCategory;
    type Qual = crate::formal::math::rotation::ontology::ParameterCount;

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(Associativity),
            Box::new(IdentityElement),
            Box::new(InverseExists),
            Box::new(CompositionConsistency),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ontology_validates() {
        RigidMotionOntology::validate().unwrap();
    }
}

/// Canonical poses for axiom verification.
fn canonical_poses() -> Vec<Pose> {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};
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
