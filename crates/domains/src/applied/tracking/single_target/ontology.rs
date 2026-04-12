use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Target kinematic state components.
///
/// Source: Bar-Shalom, Li & Kirubarajan (2001), Chapter 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
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

define_dense_category! {
    pub TargetStateCategory {
        entity: TargetStateComponent,
        relation: StateDerivative,
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

pub struct SingleTargetOntology;

impl Ontology for SingleTargetOntology {
    type Cat = TargetStateCategory;
    type Qual = ComponentDimension;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(VelocityDerivesFromPosition)]
    }
}
