use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

// Systems thinking ontology — the science of wholes, relationships, and patterns.
//
// A system is a set of interacting components that together exhibit behavior
// that the components individually do not. This ontology formalizes the
// core concepts of systems thinking and cybernetics as a category.
//
// References:
// - Ludwig von Bertalanffy, General System Theory (1968)
// - Norbert Wiener, Cybernetics (1948)
// - W. Ross Ashby, An Introduction to Cybernetics (1956)
// - Stafford Beer, Brain of the Firm (1972)
// - Donella Meadows, Thinking in Systems (2008)

/// Core concepts of systems thinking.
///
/// These are the fundamental building blocks that every system exhibits.
/// A traffic intersection, a chess game, a conversation, an economy —
/// all are systems composed of these concepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemConcept {
    /// A component or element within the system.
    /// Traffic: a signal. Chess: a piece. Economy: a firm.
    Component,

    /// A connection or interaction between components.
    /// Traffic: conflict between directions. Chess: attack relationship.
    Interaction,

    /// The state of the system at a point in time.
    /// Traffic: current signal configuration. Chess: board position.
    State,

    /// A transition that changes the system's state.
    /// Traffic: signal advance. Chess: a move.
    Transition,

    /// A rule that constrains which transitions are valid.
    /// Traffic: safety check (no conflicting greens). Chess: legal move rules.
    Constraint,

    /// A feedback loop — output influences future input.
    /// Traffic: congestion → longer green → reduced congestion.
    Feedback,

    /// The tendency to maintain stable state despite perturbation.
    /// Traffic: green wave timing. Economy: price equilibrium.
    Homeostasis,

    /// A property of the whole that parts don't have individually.
    /// Traffic: flow rate. Language: meaning. Economy: GDP.
    Emergence,

    /// The boundary between system and environment.
    /// Traffic: the intersection perimeter. Chess: the board edge.
    Boundary,

    /// The observer or controller of the system.
    /// Traffic: the signal controller. Cybernetics: the regulator.
    Controller,
}

impl Entity for SystemConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Component,
            Self::Interaction,
            Self::State,
            Self::Transition,
            Self::Constraint,
            Self::Feedback,
            Self::Homeostasis,
            Self::Emergence,
            Self::Boundary,
            Self::Controller,
        ]
    }
}

/// Relationships between system concepts.
///
/// These encode how the concepts of systems thinking relate to each other.
/// The morphisms ARE the theory — they state what systems thinking claims
/// about how systems work.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SystemRelation {
    pub from: SystemConcept,
    pub to: SystemConcept,
    pub kind: SystemRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemRelationKind {
    Identity,
    /// Components compose into state.
    ComposesInto,
    /// Transitions change state.
    Changes,
    /// Constraints govern transitions.
    Governs,
    /// Feedback connects output to input.
    FeedsBack,
    /// Homeostasis stabilizes state via feedback.
    Stabilizes,
    /// Emergence arises from interactions.
    ArisesFrom,
    /// Controller regulates via constraints.
    Regulates,
    /// Boundary separates system from environment.
    Separates,
    /// Composed relationship (transitive).
    Composed,
}

impl Relationship for SystemRelation {
    type Object = SystemConcept;
    fn source(&self) -> SystemConcept {
        self.from
    }
    fn target(&self) -> SystemConcept {
        self.to
    }
}

/// The systems thinking category.
///
/// This IS the formal structure of systems thinking.
/// If the category laws hold, then systems thinking is
/// mathematically consistent as a theory.
pub struct SystemsCategory;

impl Category for SystemsCategory {
    type Object = SystemConcept;
    type Morphism = SystemRelation;

    fn identity(obj: &SystemConcept) -> SystemRelation {
        SystemRelation {
            from: *obj,
            to: *obj,
            kind: SystemRelationKind::Identity,
        }
    }

    fn compose(f: &SystemRelation, g: &SystemRelation) -> Option<SystemRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == SystemRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == SystemRelationKind::Identity {
            return Some(f.clone());
        }
        Some(SystemRelation {
            from: f.from,
            to: g.to,
            kind: SystemRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<SystemRelation> {
        use SystemConcept::*;
        use SystemRelationKind::*;

        let mut m = Vec::new();

        // Identity for each concept
        for c in SystemConcept::variants() {
            m.push(SystemRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // Components compose into State
        m.push(SystemRelation {
            from: Component,
            to: State,
            kind: ComposesInto,
        });
        // Interactions compose into State
        m.push(SystemRelation {
            from: Interaction,
            to: State,
            kind: ComposesInto,
        });
        // Transitions change State
        m.push(SystemRelation {
            from: Transition,
            to: State,
            kind: Changes,
        });
        // Constraints govern Transitions
        m.push(SystemRelation {
            from: Constraint,
            to: Transition,
            kind: Governs,
        });
        // Feedback connects State back to Transition (circular!)
        m.push(SystemRelation {
            from: State,
            to: Feedback,
            kind: FeedsBack,
        });
        m.push(SystemRelation {
            from: Feedback,
            to: Transition,
            kind: FeedsBack,
        });
        // Homeostasis stabilizes State via Feedback
        m.push(SystemRelation {
            from: Homeostasis,
            to: State,
            kind: Stabilizes,
        });
        m.push(SystemRelation {
            from: Feedback,
            to: Homeostasis,
            kind: Stabilizes,
        });
        // Emergence arises from Interactions
        m.push(SystemRelation {
            from: Interaction,
            to: Emergence,
            kind: ArisesFrom,
        });
        // Controller regulates via Constraints
        m.push(SystemRelation {
            from: Controller,
            to: Constraint,
            kind: Regulates,
        });
        // Boundary separates system
        m.push(SystemRelation {
            from: Boundary,
            to: Component,
            kind: Separates,
        });
        // Transition modifies Components (a signal advance changes the signal)
        m.push(SystemRelation {
            from: Transition,
            to: Component,
            kind: Changes,
        });
        // Feedback informs Controller (Ashby: the regulator receives information)
        m.push(SystemRelation {
            from: Feedback,
            to: Controller,
            kind: FeedsBack,
        });

        // The full cybernetic loop:
        // State → Feedback → Controller → Constraint → Transition → Component → State
        //
        // This means EVERYTHING is reachable from State — which is
        // the defining property of a system: interconnectedness.

        // Transitive compositions for closure
        // State → Feedback → Transition
        m.push(SystemRelation {
            from: State,
            to: Transition,
            kind: Composed,
        });
        // State → Feedback → Homeostasis
        m.push(SystemRelation {
            from: State,
            to: Homeostasis,
            kind: Composed,
        });
        // State → Feedback → Controller
        m.push(SystemRelation {
            from: State,
            to: Controller,
            kind: Composed,
        });
        // State → ... → Constraint
        m.push(SystemRelation {
            from: State,
            to: Constraint,
            kind: Composed,
        });
        // State → ... → Component
        m.push(SystemRelation {
            from: State,
            to: Component,
            kind: Composed,
        });
        // State → ... → Interaction (Component participates in Interaction)
        m.push(SystemRelation {
            from: State,
            to: Interaction,
            kind: Composed,
        });
        // State → ... → Emergence
        m.push(SystemRelation {
            from: State,
            to: Emergence,
            kind: Composed,
        });
        // State → ... → Boundary
        m.push(SystemRelation {
            from: State,
            to: Boundary,
            kind: Composed,
        });
        // Feedback → Homeostasis → State
        m.push(SystemRelation {
            from: Feedback,
            to: State,
            kind: Composed,
        });
        // Controller → Constraint → Transition
        m.push(SystemRelation {
            from: Controller,
            to: Transition,
            kind: Composed,
        });
        // Controller → ... → State
        m.push(SystemRelation {
            from: Controller,
            to: State,
            kind: Composed,
        });
        // Controller → ... → Component
        m.push(SystemRelation {
            from: Controller,
            to: Component,
            kind: Composed,
        });
        // Constraint → Transition → State
        m.push(SystemRelation {
            from: Constraint,
            to: State,
            kind: Composed,
        });
        // Constraint → Transition → Component
        m.push(SystemRelation {
            from: Constraint,
            to: Component,
            kind: Composed,
        });
        // Transition → Component → State
        m.push(SystemRelation {
            from: Transition,
            to: Component,
            kind: Composed,
        });
        // Component → State → Feedback
        m.push(SystemRelation {
            from: Component,
            to: Feedback,
            kind: Composed,
        });
        // Interaction → State (via ComposesInto, already direct above)
        // Interaction → Feedback
        m.push(SystemRelation {
            from: Interaction,
            to: Feedback,
            kind: Composed,
        });
        // Boundary → Component → State
        m.push(SystemRelation {
            from: Boundary,
            to: State,
            kind: Composed,
        });

        // Self-composed morphisms (roundtrips produce Composed(X,X))
        for c in SystemConcept::variants() {
            m.push(SystemRelation {
                from: c,
                to: c,
                kind: Composed,
            });
        }

        m
    }
}
