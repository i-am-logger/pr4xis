use pr4xis::category::Arrow;
use pr4xis::category::entity::{Concept, FinitelyGenerated};
use pr4xis::category::{Category, Functor};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

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

impl Concept for EngineElement {}
impl FinitelyGenerated for EngineElement {
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

impl Arrow for EngineRelation {
    type Object = EngineElement;
    type Kind = EngineRelationKind;
    fn source(&self) -> EngineElement {
        self.from
    }
    fn target(&self) -> EngineElement {
        self.to
    }
    fn kind(&self) -> EngineRelationKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new(format!("{:?}-[{:?}]-{:?}", self.from, self.kind, self.to)),
            description: Label::new(format!(
                "{:?} -[{:?}]-> {:?}",
                self.from, self.kind, self.to
            )),
            citation: Citation::parse_static(
                "Conant & Ashby (1970) Every good regulator of a system must be a model of that system, Int. J. Systems Sci. 1(2)",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
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
        let candidate = EngineRelation {
            from: f.from,
            to: g.to,
            kind: EngineRelationKind::Composed,
        };
        // Partial category (#166): composition is defined only when the
        // composite is itself a declared morphism. `morphisms()` lists the
        // specific `Composed` edges produced by the engine control loop;
        // any other path is undefined per OBO-RO partial-relations.
        if Self::morphisms().contains(&candidate) {
            Some(candidate)
        } else {
            None
        }
    }

    fn morphisms() -> Vec<EngineRelation> {
        use EngineElement::*;
        use EngineRelationKind::*;
        use std::collections::HashSet;

        // Direct kinded edges of the Engine control loop.
        let direct: Vec<(EngineElement, EngineElement, EngineRelationKind)> = vec![
            (Observation, Situation, Observes),
            (PreconditionCheck, Violation, Checks),
            (ActionExecution, Situation, Applies),
            (Goal, Violation, Compares),
            (PreconditionCheck, ActionExecution, Records),
            (Observation, Violation, Records),
            (UnexpectedInput, Situation, Disrupts),
            (Ontology, Situation, Models),
            (PreconditionCheck, Ontology, Records),
            (EngineCycle, Observation, Closes),
            (EngineCycle, PreconditionCheck, Closes),
            (TraceEntry, ActionExecution, Records),
        ];

        let mut m: Vec<EngineRelation> = Vec::new();
        for c in EngineElement::variants() {
            m.push(EngineRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }
        for &(f, t, k) in &direct {
            m.push(EngineRelation {
                from: f,
                to: t,
                kind: k,
            });
        }

        // Transitive closure (Warshall 1962) over the kinded edges,
        // collapsing all reachability into the `Composed` umbrella kind
        // per Mac Lane CWM Ch. I §1 closure axiom.
        let edges: HashSet<(EngineElement, EngineElement)> =
            direct.iter().map(|&(f, t, _)| (f, t)).collect();
        let mut closure = edges.clone();
        loop {
            let mut added = false;
            let snap: Vec<_> = closure.iter().cloned().collect();
            for &(a, b) in &snap {
                for &(b2, c) in &snap {
                    if b == b2 && !closure.contains(&(a, c)) {
                        closure.insert((a, c));
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }
        for (f, t) in closure {
            m.push(EngineRelation {
                from: f,
                to: t,
                kind: Composed,
            });
        }
        // Self-loops under Composed (idempotent reflexive closure).
        for c in EngineElement::variants() {
            let r = EngineRelation {
                from: c,
                to: c,
                kind: Composed,
            };
            if !m.contains(&r) {
                m.push(r);
            }
        }

        m
    }
}

impl pr4xis::category::NamedCategory for EngineCategory {
    fn ontology_name() -> OntologyName {
        OntologyName::new_static("Engine")
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
            // Canonical Relations-ontology kinds (Smith 2005 OBO-RO) —
            // unreachable when source has no edges of these kinds.
            ControlRelationKind::Subsumption
            | ControlRelationKind::Parthood
            | ControlRelationKind::Causation
            | ControlRelationKind::Opposition => EngineRelationKind::Identity,
        };
        EngineRelation { from, to, kind }
    }
}
pr4xis::register_functor!(ControlToEngine);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};

    #[test]
    fn engine_category_laws() {
        assert_category_laws::<EngineCategory>();
    }

    #[test]
    fn control_to_engine_functor_laws() {
        assert_functor_laws::<ControlToEngine>();
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
