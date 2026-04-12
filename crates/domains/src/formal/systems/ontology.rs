use pr4xis::category::Entity;
use pr4xis::define_category;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
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

define_category! {
    /// The systems thinking category.
    ///
    /// This IS the formal structure of systems thinking.
    /// If the category laws hold, then systems thinking is
    /// mathematically consistent as a theory.
    pub SystemsCategory {
        entity: SystemConcept,
        relation: SystemRelation,
        kind: SystemRelationKind,
        kinds: [
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
        ],
        edges: [
            // Components compose into State
            (Component, State, ComposesInto),
            // Interactions compose into State
            (Interaction, State, ComposesInto),
            // Transitions change State
            (Transition, State, Changes),
            // Constraints govern Transitions
            (Constraint, Transition, Governs),
            // Feedback connects State back to Transition (circular!)
            (State, Feedback, FeedsBack),
            (Feedback, Transition, FeedsBack),
            // Homeostasis stabilizes State via Feedback
            (Homeostasis, State, Stabilizes),
            (Feedback, Homeostasis, Stabilizes),
            // Emergence arises from Interactions
            (Interaction, Emergence, ArisesFrom),
            // Controller regulates via Constraints
            (Controller, Constraint, Regulates),
            // Boundary separates system
            (Boundary, Component, Separates),
            // Transition modifies Components (a signal advance changes the signal)
            (Transition, Component, Changes),
            // Feedback informs Controller (Ashby: the regulator receives information)
            (Feedback, Controller, FeedsBack),
        ],
        composed: [
            // The full cybernetic loop:
            // State → Feedback → Controller → Constraint → Transition → Component → State

            // State → Feedback → Transition
            (State, Transition),
            // State → Feedback → Homeostasis
            (State, Homeostasis),
            // State → Feedback → Controller
            (State, Controller),
            // State → ... → Constraint
            (State, Constraint),
            // State → ... → Component
            (State, Component),
            // State → ... → Interaction (Component participates in Interaction)
            (State, Interaction),
            // State → ... → Emergence
            (State, Emergence),
            // State → ... → Boundary
            (State, Boundary),
            // Feedback → Homeostasis → State
            (Feedback, State),
            // Controller → Constraint → Transition
            (Controller, Transition),
            // Controller → ... → State
            (Controller, State),
            // Controller → ... → Component
            (Controller, Component),
            // Constraint → Transition → State
            (Constraint, State),
            // Constraint → Transition → Component
            (Constraint, Component),
            // Transition → Component → State
            (Transition, Component),
            // Component → State → Feedback
            (Component, Feedback),
            // Interaction → Feedback
            (Interaction, Feedback),
            // Boundary → Component → State
            (Boundary, State),
        ],
    }
}
