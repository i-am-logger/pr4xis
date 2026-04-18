//! SLAM graph components.
//!
//! Source: Grisetti et al. (2010), "A Tutorial on Graph-Based SLAM"

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Slam",
    source: "Grisetti et al. (2010); Durrant-Whyte & Bailey (2006)",
    being: Process,

    concepts: [Pose, Landmark, Constraint, LoopClosure],

    labels: {
        Pose: ("en", "Pose", "Robot pose node in the graph."),
        Landmark: ("en", "Landmark", "Landmark node observed by the robot."),
        Constraint: ("en", "Constraint", "Constraint (edge) between nodes from sensor observations."),
        LoopClosure: ("en", "Loop closure", "Loop closure: constraint linking current pose to a previously visited location."),
    },

    edges: [
        (Pose, Constraint, Observes),
        (Constraint, Landmark, Links),
        (Pose, LoopClosure, Closes),
        (LoopClosure, Pose, Closes),
        (LoopClosure, Constraint, Observes),
    ],
}

/// Quality: description of each SLAM component's role.
#[derive(Debug, Clone)]
pub struct ComponentRole;

impl Quality for ComponentRole {
    type Individual = SlamConcept;
    type Value = &'static str;

    fn get(&self, component: &SlamConcept) -> Option<&'static str> {
        Some(match component {
            SlamConcept::Pose => "robot pose (position + orientation) at a time step",
            SlamConcept::Landmark => "environmental feature observed by the robot",
            SlamConcept::Constraint => "sensor measurement linking two nodes",
            SlamConcept::LoopClosure => "re-observation of a previously visited location",
        })
    }
}

/// Axiom: adding a constraint to the SLAM graph reduces uncertainty.
pub struct ConstraintReducesUncertainty;

impl Axiom for ConstraintReducesUncertainty {
    fn description(&self) -> &str {
        "adding a constraint to the SLAM graph reduces or maintains uncertainty"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(
    ConstraintReducesUncertainty,
    "Grisetti et al. (2010), \"A Tutorial on Graph-Based SLAM\""
);

/// Axiom: loop closures connect poses (not landmarks).
pub struct LoopClosureConnectsPoses;

impl Axiom for LoopClosureConnectsPoses {
    fn description(&self) -> &str {
        "loop closures connect pose nodes to other pose nodes"
    }
    fn holds(&self) -> bool {
        let morphisms = SlamCategory::morphisms();
        let lc_to_pose = morphisms
            .iter()
            .any(|m| m.from == SlamConcept::LoopClosure && m.to == SlamConcept::Pose);
        let pose_to_lc = morphisms
            .iter()
            .any(|m| m.from == SlamConcept::Pose && m.to == SlamConcept::LoopClosure);
        lc_to_pose && pose_to_lc
    }
}
pr4xis::register_axiom!(
    LoopClosureConnectsPoses,
    "Grisetti et al. (2010), \"A Tutorial on Graph-Based SLAM\""
);

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
