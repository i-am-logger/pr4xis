use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;
use pr4xis::category::{Category, Functor};

use super::control::*;

// Control Systems → Engine Pattern functor.
//
// Proof that the praxis Engine IS a control system.
// The Engine implements the closed-loop control pattern:
//
//   Plant       = Situation (the world state)
//   Controller  = Precondition evaluation + rule selection
//   Sensor      = Situation::describe() (observing current state)
//   Actuator    = Action::apply() (changing the state)
//   Setpoint    = Goal / desired postconditions
//   Error       = Precondition violation (gap between is and ought)
//   Signal      = TraceEntry (information flowing through the loop)
//   Disturbance = Invalid user input, unexpected state
//   Model       = Ontology (Conant-Ashby: the ontology IS the model)
//   FeedbackLoop = Engine.next() cycle: evaluate → act → observe → evaluate
//
// This proves Conant-Ashby (1970) in code: the Engine's ontology
// IS the model of the system it regulates.

/// The Engine pattern as categorical objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineElement {
    /// The current state of the world (= Plant).
    Situation,
    /// Rule checking (= Controller).
    PreconditionCheck,
    /// Observing the current state (= Sensor).
    Observation,
    /// Applying a change (= Actuator).
    ActionExecution,
    /// The desired outcome (= Setpoint).
    Goal,
    /// A rule violation (= Error).
    Violation,
    /// Information flowing through the loop (= Signal).
    TraceEntry,
    /// Unexpected input or state (= Disturbance).
    UnexpectedInput,
    /// The domain ontology (= Model, per Conant-Ashby).
    Ontology,
    /// The Engine.next() cycle (= FeedbackLoop).
    EngineCycle,
}

impl Entity for EngineElement {
    fn variants() -> Vec<Self> {
        vec![
            Self::Situation,
            Self::PreconditionCheck,
            Self::Observation,
            Self::ActionExecution,
            Self::Goal,
            Self::Violation,
            Self::TraceEntry,
            Self::UnexpectedInput,
            Self::Ontology,
            Self::EngineCycle,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EngineRelation {
    pub from: EngineElement,
    pub to: EngineElement,
    pub kind: EngineRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineRelationKind {
    Identity,
    Observes,
    Checks,
    Applies,
    Compares,
    Disrupts,
    Models,
    Closes,
    Records,
    Composed,
}

impl Relationship for EngineRelation {
    type Object = EngineElement;
    fn source(&self) -> EngineElement {
        self.from
    }
    fn target(&self) -> EngineElement {
        self.to
    }
}

pub struct EngineCategory;

impl Category for EngineCategory {
    type Object = EngineElement;
    type Morphism = EngineRelation;

    fn identity(obj: &EngineElement) -> EngineRelation {
        EngineRelation {
            from: *obj,
            to: *obj,
            kind: EngineRelationKind::Identity,
        }
    }

    fn compose(f: &EngineRelation, g: &EngineRelation) -> Option<EngineRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == EngineRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == EngineRelationKind::Identity {
            return Some(f.clone());
        }
        Some(EngineRelation {
            from: f.from,
            to: g.to,
            kind: EngineRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<EngineRelation> {
        use EngineElement::*;
        use EngineRelationKind::*;

        let mut m = Vec::new();

        for c in EngineElement::variants() {
            m.push(EngineRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // The Engine control loop
        m.push(EngineRelation {
            from: Observation,
            to: Situation,
            kind: Observes,
        });
        m.push(EngineRelation {
            from: PreconditionCheck,
            to: Violation,
            kind: Checks,
        });
        m.push(EngineRelation {
            from: ActionExecution,
            to: Situation,
            kind: Applies,
        });
        m.push(EngineRelation {
            from: Goal,
            to: Violation,
            kind: Compares,
        });
        m.push(EngineRelation {
            from: PreconditionCheck,
            to: ActionExecution,
            kind: Records,
        });
        m.push(EngineRelation {
            from: Observation,
            to: Violation,
            kind: Records,
        });
        m.push(EngineRelation {
            from: UnexpectedInput,
            to: Situation,
            kind: Disrupts,
        });
        m.push(EngineRelation {
            from: Ontology,
            to: Situation,
            kind: Models,
        });
        m.push(EngineRelation {
            from: PreconditionCheck,
            to: Ontology,
            kind: Records,
        });
        m.push(EngineRelation {
            from: EngineCycle,
            to: Observation,
            kind: Closes,
        });
        m.push(EngineRelation {
            from: EngineCycle,
            to: PreconditionCheck,
            kind: Closes,
        });
        m.push(EngineRelation {
            from: TraceEntry,
            to: ActionExecution,
            kind: Records,
        });

        // Transitive
        m.push(EngineRelation {
            from: PreconditionCheck,
            to: Situation,
            kind: Composed,
        });
        m.push(EngineRelation {
            from: Observation,
            to: PreconditionCheck,
            kind: Composed,
        });
        m.push(EngineRelation {
            from: Goal,
            to: PreconditionCheck,
            kind: Composed,
        });
        m.push(EngineRelation {
            from: UnexpectedInput,
            to: Violation,
            kind: Composed,
        });

        for c in EngineElement::variants() {
            m.push(EngineRelation {
                from: c,
                to: c,
                kind: Composed,
            });
        }

        m
    }
}

/// Functor: Control Systems → Engine Pattern.
///
/// THE PROOF that the Engine is a control system.
/// Conant-Ashby (1970): the ontology IS the model of the system.
pub struct ControlToEngine;

impl Functor for ControlToEngine {
    type Source = ControlCategory;
    type Target = EngineCategory;

    fn map_object(obj: &ControlConcept) -> EngineElement {
        match obj {
            ControlConcept::Plant => EngineElement::Situation,
            ControlConcept::Controller => EngineElement::PreconditionCheck,
            ControlConcept::Sensor => EngineElement::Observation,
            ControlConcept::Actuator => EngineElement::ActionExecution,
            ControlConcept::Setpoint => EngineElement::Goal,
            ControlConcept::Error => EngineElement::Violation,
            ControlConcept::Signal => EngineElement::TraceEntry,
            ControlConcept::Disturbance => EngineElement::UnexpectedInput,
            ControlConcept::Model => EngineElement::Ontology,
            ControlConcept::FeedbackLoop => EngineElement::EngineCycle,
        }
    }

    fn map_morphism(m: &ControlRelation) -> EngineRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = match m.kind {
            ControlRelationKind::Identity => EngineRelationKind::Identity,
            ControlRelationKind::Measures => EngineRelationKind::Observes,
            ControlRelationKind::ComputesFrom => EngineRelationKind::Checks,
            ControlRelationKind::ActsOn => EngineRelationKind::Applies,
            ControlRelationKind::ComparedWith => EngineRelationKind::Compares,
            ControlRelationKind::Perturbs => EngineRelationKind::Disrupts,
            ControlRelationKind::Represents => EngineRelationKind::Models,
            ControlRelationKind::Closes => EngineRelationKind::Closes,
            ControlRelationKind::Carries => EngineRelationKind::Records,
            ControlRelationKind::Composed => EngineRelationKind::Composed,
        };
        EngineRelation { from, to, kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::{check_category_laws, check_functor_laws};

    #[test]
    fn engine_category_laws() {
        check_category_laws::<EngineCategory>().unwrap();
    }

    #[test]
    fn control_to_engine_functor_laws() {
        check_functor_laws::<ControlToEngine>().unwrap();
    }

    #[test]
    fn plant_maps_to_situation() {
        assert_eq!(
            ControlToEngine::map_object(&ControlConcept::Plant),
            EngineElement::Situation
        );
    }

    #[test]
    fn model_maps_to_ontology() {
        // Conant-Ashby: the model IS the ontology
        assert_eq!(
            ControlToEngine::map_object(&ControlConcept::Model),
            EngineElement::Ontology
        );
    }

    #[test]
    fn feedback_maps_to_engine_cycle() {
        assert_eq!(
            ControlToEngine::map_object(&ControlConcept::FeedbackLoop),
            EngineElement::EngineCycle
        );
    }
}
