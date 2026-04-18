#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Target kinematic state components.
///
/// Source: Bar-Shalom, Li & Kirubarajan (2001), Chapter 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum TargetStateComponent {
    /// Position (x, y, z).
    Position,
    /// Velocity (vx, vy, vz).
    Velocity,
    /// Acceleration (ax, ay, az).
    Acceleration,
    /// Turn rate (ω) — for maneuvering targets.
    TurnRate,
}

define_ontology! {
    pub SingleTargetOntology for TargetStateCategory {
        entity: TargetStateComponent,
        relation: StateDerivative,
        being: Process,
        source: "Bar-Shalom et al. (2001); Li & Jilkov (2003)",
    }
}

#[derive(Debug, Clone)]
pub struct ComponentDimension;

impl Quality for ComponentDimension {
    type Individual = TargetStateComponent;
    type Value = usize;

    fn get(&self, c: &TargetStateComponent) -> Option<usize> {
        Some(match c {
            TargetStateComponent::Position => 3,
            TargetStateComponent::Velocity => 3,
            TargetStateComponent::Acceleration => 3,
            TargetStateComponent::TurnRate => 1,
        })
    }
}

/// Axiom: velocity is derivative of position (kinematics).
pub struct VelocityDerivesFromPosition;

impl Axiom for VelocityDerivesFromPosition {
    fn description(&self) -> &str {
        "velocity is the time derivative of position"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(VelocityDerivesFromPosition);

impl Ontology for SingleTargetOntology {
    type Cat = TargetStateCategory;
    type Qual = ComponentDimension;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(VelocityDerivesFromPosition)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<TargetStateCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        SingleTargetOntology::validate().unwrap();
    }
}
