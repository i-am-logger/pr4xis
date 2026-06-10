//! Systems-thinking ontology — components, interactions, states,
//! transitions, constraints, feedback, homeostasis, emergence,
//! boundary, controller. The categorical structure of any system.
//!
//! A system is a set of interacting components that together exhibit
//! behaviour the components individually do not. The ten concepts
//! below are the building blocks every system exhibits — a traffic
//! intersection, a chess game, a conversation, an economy.
//!
//! # Literature
//!
//! - **von Bertalanffy (1968)** *General System Theory: Foundations,
//!   Development, Applications*, George Braziller — the founding text
//!   of general systems theory.
//! - **Wiener (1948)** *Cybernetics: Or Control and Communication in
//!   the Animal and the Machine*, MIT Press — control + communication
//!   as the cybernetic loop.
//! - **Ashby (1956)** *An Introduction to Cybernetics*, Chapman & Hall
//!   — Law of Requisite Variety; the regulator must have at least the
//!   variety of the disturbance.
//! - **Beer (1972)** *Brain of the Firm*, John Wiley & Sons — the
//!   Viable System Model.
//! - **Meadows (2008)** *Thinking in Systems: A Primer*, Chelsea Green
//!   — feedback loops, homeostasis, emergence in everyday systems.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "System",
    source: "von Bertalanffy (1968) General System Theory; Wiener (1948) Cybernetics; Ashby (1956) An Introduction to Cybernetics; Beer (1972) Brain of the Firm; Meadows (2008) Thinking in Systems",

    concepts: [
        Component,
        Interaction,
        State,
        Transition,
        Constraint,
        Feedback,
        Homeostasis,
        Emergence,
        Boundary,
        Controller,
    ],

    labels: {
        Component: ("en", "Component",
            "von Bertalanffy (1968) Ch. 1: an element within the system; a traffic signal, a chess piece, a firm in an economy."),
        Interaction: ("en", "Interaction",
            "von Bertalanffy (1968) Ch. 1: a connection between components; the relational glue that makes a set of parts into a system."),
        State: ("en", "State",
            "Ashby (1956) §2: the configuration of the system at a point in time - a complete description of all relevant component values."),
        Transition: ("en", "Transition",
            "Ashby (1956) §2: a change of state; the dynamics of the system as it evolves through its state space."),
        Constraint: ("en", "Constraint",
            "Ashby (1956) §7: a rule restricting which transitions are valid - safety rules in a traffic light, legal-move rules in chess."),
        Feedback: ("en", "Feedback",
            "Wiener (1948) Ch. 4: a return path connecting output back to input; positive feedback amplifies, negative feedback regulates."),
        Homeostasis: ("en", "Homeostasis",
            "Ashby (1956) §11: the tendency to maintain a stable state despite perturbation; achieved via negative-feedback regulation."),
        Emergence: ("en", "Emergence",
            "von Bertalanffy (1968) Ch. 3: a property of the whole that the parts individually do not possess - flow rate, language meaning, GDP."),
        Boundary: ("en", "Boundary",
            "Meadows (2008) Ch. 5: the demarcation between system and environment - the intersection perimeter, the chessboard edge, the firm's organisational boundary."),
        Controller: ("en", "Controller",
            "Ashby (1956) §10: the regulator that observes the system and acts to keep it within desired bounds; the cybernetic loop's decision-maker."),
    },

    edges: [
        (Component, State, ComposesInto),
        (Interaction, State, ComposesInto),
        (Transition, State, Changes),
        (Constraint, Transition, Governs),
        (State, Feedback, FeedsBack),
        (Feedback, Transition, FeedsBack),
        (Homeostasis, State, Stabilizes),
        (Feedback, Homeostasis, Stabilizes),
        (Interaction, Emergence, ArisesFrom),
        (Controller, Constraint, Regulates),
        (Boundary, Component, Separates),
        (Transition, Component, Changes),
        (Feedback, Controller, FeedsBack),
    ],
}

/// Quality: whether each concept is a node in the cybernetic
/// feedback loop (Ashby 1956 §10). State, Feedback, Controller,
/// Constraint, Transition, Homeostasis form the loop;
/// Component / Interaction / Boundary / Emergence are structural.
#[derive(Debug, Clone)]
pub struct IsCyberneticLoop;

impl Quality for IsCyberneticLoop {
    type Individual = SystemConcept;
    type Value = bool;

    fn get(&self, individual: &SystemConcept) -> Option<bool> {
        Some(matches!(
            individual,
            SystemConcept::State
                | SystemConcept::Feedback
                | SystemConcept::Controller
                | SystemConcept::Constraint
                | SystemConcept::Transition
                | SystemConcept::Homeostasis
        ))
    }
}

impl Ontology for SystemOntology {
    type Cat = SystemCategory;
    type Qual = IsCyberneticLoop;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<SystemCategory>();
    }

    #[test]
    fn ontology_validates() {
        SystemOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn ten_system_concepts() {
        assert_eq!(SystemConcept::variants().len(), 10);
    }

    #[test]
    fn cybernetic_loop_classification() {
        let q = IsCyberneticLoop;
        for c in [
            SystemConcept::State,
            SystemConcept::Feedback,
            SystemConcept::Controller,
            SystemConcept::Constraint,
            SystemConcept::Transition,
            SystemConcept::Homeostasis,
        ] {
            assert_eq!(q.get(&c), Some(true), "{:?} should be in loop", c);
        }
        for c in [
            SystemConcept::Component,
            SystemConcept::Interaction,
            SystemConcept::Boundary,
            SystemConcept::Emergence,
        ] {
            assert_eq!(q.get(&c), Some(false), "{:?} should be structural", c);
        }
    }

    fn arb_concept() -> impl Strategy<Value = SystemConcept> {
        proptest::sample::select(SystemConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_cybernetic_loop_total(c in arb_concept()) {
            prop_assert!(IsCyberneticLoop.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            use pr4xis::category::{Arrow, Category};
            for m in SystemCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in SystemOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }
    }
}
