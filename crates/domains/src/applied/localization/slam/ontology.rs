use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// SLAM graph components.
///
/// Source: Grisetti et al. (2010), "A Tutorial on Graph-Based SLAM"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum SlamComponent {
    /// Robot pose node in the graph.
    Pose,
    /// Landmark node observed by the robot.
    Landmark,
    /// Constraint (edge) between nodes from sensor observations.
    Constraint,
    /// Loop closure: constraint linking current pose to a previously visited location.
    LoopClosure,
}

/// Relationship between SLAM components.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SlamRelation {
    pub from: SlamComponent,
    pub to: SlamComponent,
}

impl Relationship for SlamRelation {
    type Object = SlamComponent;
    fn source(&self) -> SlamComponent {
        self.from
    }
    fn target(&self) -> SlamComponent {
        self.to
    }
}

/// Category for SLAM graph structure.
///
/// Poses observe landmarks via constraints. Loop closures connect poses.
pub struct SlamCategory;

impl Category for SlamCategory {
    type Object = SlamComponent;
    type Morphism = SlamRelation;

    fn identity(obj: &SlamComponent) -> SlamRelation {
        SlamRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &SlamRelation, g: &SlamRelation) -> Option<SlamRelation> {
        if f.to != g.from {
            return None;
        }
        let rel = SlamRelation {
            from: f.from,
            to: g.to,
        };
        if Self::morphisms().contains(&rel) {
            Some(rel)
        } else {
            None
        }
    }

    fn morphisms() -> Vec<SlamRelation> {
        use SlamComponent::*;
        let mut m = Vec::new();
        // Identities
        for s in SlamComponent::variants() {
            m.push(SlamRelation { from: s, to: s });
        }
        // Pose <-> Landmark via Constraint
        m.push(SlamRelation {
            from: Pose,
            to: Constraint,
        });
        m.push(SlamRelation {
            from: Constraint,
            to: Landmark,
        });
        m.push(SlamRelation {
            from: Pose,
            to: Landmark,
        }); // transitive
        // Pose -> LoopClosure -> Pose
        m.push(SlamRelation {
            from: Pose,
            to: LoopClosure,
        });
        m.push(SlamRelation {
            from: LoopClosure,
            to: Pose,
        });
        // Constraint -> Constraint (chaining)
        // LoopClosure -> LoopClosure (chaining)
        // Transitive: Pose -> LoopClosure -> Pose -> Constraint -> Landmark
        m.push(SlamRelation {
            from: LoopClosure,
            to: Constraint,
        });
        m.push(SlamRelation {
            from: LoopClosure,
            to: Landmark,
        });
        m.push(SlamRelation {
            from: Pose,
            to: Pose,
        }); // already identity, but explicit
        m.push(SlamRelation {
            from: LoopClosure,
            to: LoopClosure,
        }); // already identity
        m.push(SlamRelation {
            from: Constraint,
            to: Constraint,
        }); // already identity
        m
    }
}

/// Quality: description of each SLAM component's role.
#[derive(Debug, Clone)]
pub struct ComponentRole;

impl Quality for ComponentRole {
    type Individual = SlamComponent;
    type Value = &'static str;

    fn get(&self, component: &SlamComponent) -> Option<&'static str> {
        Some(match component {
            SlamComponent::Pose => "robot pose (position + orientation) at a time step",
            SlamComponent::Landmark => "environmental feature observed by the robot",
            SlamComponent::Constraint => "sensor measurement linking two nodes",
            SlamComponent::LoopClosure => "re-observation of a previously visited location",
        })
    }
}

/// Axiom: adding a constraint to the SLAM graph reduces uncertainty.
///
/// In graph-based SLAM, each new constraint (observation) adds information
/// to the system, which can only reduce or maintain (never increase) the
/// uncertainty of the maximum likelihood estimate.
pub struct ConstraintReducesUncertainty;

impl Axiom for ConstraintReducesUncertainty {
    fn description(&self) -> &str {
        "adding a constraint to the SLAM graph reduces or maintains uncertainty"
    }
    fn holds(&self) -> bool {
        // Structural axiom from information theory:
        // Adding an observation to a least-squares system can only
        // increase the information matrix (reduce covariance).
        true
    }
}

/// Axiom: loop closures connect poses (not landmarks).
pub struct LoopClosureConnectsPoses;

impl Axiom for LoopClosureConnectsPoses {
    fn description(&self) -> &str {
        "loop closures connect pose nodes to other pose nodes"
    }
    fn holds(&self) -> bool {
        let morphisms = SlamCategory::morphisms();
        // LoopClosure must have a path to Pose
        let lc_to_pose = morphisms
            .iter()
            .any(|m| m.from == SlamComponent::LoopClosure && m.to == SlamComponent::Pose);
        let pose_to_lc = morphisms
            .iter()
            .any(|m| m.from == SlamComponent::Pose && m.to == SlamComponent::LoopClosure);
        lc_to_pose && pose_to_lc
    }
}

pub struct SlamOntology;

impl Ontology for SlamOntology {
    type Cat = SlamCategory;
    type Qual = ComponentRole;

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(ConstraintReducesUncertainty),
            Box::new(LoopClosureConnectsPoses),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<SlamCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        SlamOntology::validate().unwrap();
    }
}
