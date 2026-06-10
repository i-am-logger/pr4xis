//! Situation calculus + STRIPS — the substrate ontology grounding
//! the `engine/` module.
//!
//! # Why this ontology lives in core
//!
//! `engine/action.rs`, `engine/precondition.rs`, `engine/situation.rs`,
//! `engine/trace.rs`, and `engine/engine.rs` provide machinery for
//! action-effect reasoning: situations, actions with preconditions, and
//! execution traces. This is classical AI planning — situation calculus
//! (McCarthy 1963) + STRIPS (Fikes & Nilsson 1971). Per the
//! substrate-grounding principle, the concept vocabulary belongs here
//! in core alongside the machinery.
//!
//! # Literature
//!
//! - **McCarthy (1963)** "Situations, Actions, and Causal Laws" — the
//!   situation calculus; actions transform situations; fluents are
//!   state-valued properties.
//! - **McCarthy & Hayes (1969)** "Some Philosophical Problems from the
//!   Standpoint of Artificial Intelligence" — frame problem;
//!   epistemological vs heuristic AI.
//! - **Fikes & Nilsson (1971)** "STRIPS: A New Approach to the
//!   Application of Theorem Proving to Problem Solving" — goal-directed
//!   planning with add/delete lists.
//! - **Reiter (2001)** *Knowledge in Action: Logical Foundations for
//!   Specifying and Implementing Dynamical Systems* — successor-state
//!   axioms; regression; formalisation of the situation calculus.
//! - **Bratman (1987)** *Intention, Plans, and Practical Reason* — the
//!   Belief / Desire / Intention (BDI) agent architecture.
//! - **Russell & Norvig (2009)** *Artificial Intelligence: A Modern
//!   Approach* Ch. 10–11 — planning-problem formulation.

use crate as pr4xis;
use crate::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "SituationCalculus",
    source: "McCarthy (1963) Situations, Actions, and Causal Laws; McCarthy & Hayes (1969); Fikes & Nilsson (1971) STRIPS; Reiter (2001) Knowledge in Action; Bratman (1987) Intention, Plans, and Practical Reason; Russell & Norvig (2009) AI: A Modern Approach",

    concepts: [
        // === McCarthy (1963) situation calculus ===
        Situation,
        Action,
        Fluent,
        Effect,

        // === Actions + preconditions ===
        Precondition,
        Postcondition,
        ActionSchema,

        // === STRIPS (Fikes & Nilsson 1971) ===
        AddList,
        DeleteList,
        InitialState,
        Goal,
        Plan,

        // === Execution ===
        Trace,
        TraceEntry,
        Transition,

        // === Frame problem (McCarthy & Hayes 1969) ===
        FrameAxiom,
        SuccessorStateAxiom,

        // === BDI (Bratman 1987) ===
        Agent,
        Belief,
        Desire,
        Intention,
    ],

    labels: {
        Situation: ("en", "Situation",
            "McCarthy (1963): a snapshot of the world at a point in time — a state from which actions may be taken."),
        Action: ("en", "Action",
            "McCarthy (1963): a primitive event that transforms one situation into another. Has preconditions and effects."),
        Fluent: ("en", "Fluent",
            "McCarthy (1963): a state-valued property whose truth depends on the situation — 'at(x, loc)', 'holding(obj)', etc."),
        Effect: ("en", "Effect",
            "The change an action causes in a situation — what becomes true / false after the action."),

        Precondition: ("en", "Precondition",
            "A proposition that must hold in the current situation for the action to be applicable. Fikes & Nilsson (1971)."),
        Postcondition: ("en", "Postcondition",
            "A proposition that holds in the successor situation after the action is applied."),
        ActionSchema: ("en", "Action schema",
            "A parameterised action template — an action with variables filled in by substitution. Fikes & Nilsson (1971) STRIPS operators."),

        AddList: ("en", "Add list",
            "STRIPS: the set of fluents that become true when the action is applied. Fikes & Nilsson (1971)."),
        DeleteList: ("en", "Delete list",
            "STRIPS: the set of fluents that become false when the action is applied. Fikes & Nilsson (1971)."),
        InitialState: ("en", "Initial state",
            "The situation from which planning begins — the set of fluents true at time zero."),
        Goal: ("en", "Goal",
            "A description (partial or total) of the situation the plan aims to achieve. Fikes & Nilsson (1971)."),
        Plan: ("en", "Plan",
            "A sequence of actions that, applied from the initial state, achieves the goal. Russell & Norvig (2009) Ch. 10."),

        Trace: ("en", "Trace",
            "A recording of an execution — the sequence of situations and actions experienced. pr4xis-specific (engine/trace.rs)."),
        TraceEntry: ("en", "Trace entry",
            "A single step in a trace: the situation before, the action taken, the situation after, and whatever precondition verdicts were reached."),
        Transition: ("en", "Transition",
            "The move from one situation to another via an action. Reiter (2001) — the ternary relation Do(action, situation, successor)."),

        FrameAxiom: ("en", "Frame axiom",
            "McCarthy & Hayes (1969): an axiom specifying which fluents are UNCHANGED by an action — the frame problem is the problem of their number."),
        SuccessorStateAxiom: ("en", "Successor-state axiom",
            "Reiter (1991/2001): a compact solution to the frame problem — a single axiom per fluent specifying when it holds in the successor situation."),

        Agent: ("en", "Agent",
            "A reasoning entity that acts. Bratman (1987); Russell & Norvig Ch. 2."),
        Belief: ("en", "Belief",
            "The agent's information about the world — what it takes to be true. Bratman (1987)."),
        Desire: ("en", "Desire",
            "A state of the world the agent prefers. Bratman (1987)."),
        Intention: ("en", "Intention",
            "A commitment to a plan for achieving a desire. Bratman (1987) — intentions resist reconsideration."),
    },

    is_a: [
        // Preconditions and postconditions are special propositions
        // attached to actions; modelled here as is-a Effect for parthood.
        (Precondition, Effect),
        (Postcondition, Effect),

        // STRIPS add/delete lists are specialised effect sets
        (AddList, Effect),
        (DeleteList, Effect),

        // An Action is specialised as an ActionSchema
        (ActionSchema, Action),

        // Goal, InitialState — both are Situations (or descriptions thereof)
        (InitialState, Situation),
        (Goal, Situation),

        // BDI: Belief / Desire / Intention are Agent properties, not Agent
        // subsumption; treat them as fluents on Agent (has_a, below).

        // FrameAxiom and SuccessorStateAxiom both address the frame problem
        // — SuccessorStateAxiom supersedes FrameAxiom (Reiter 1991 improvement)
        (SuccessorStateAxiom, FrameAxiom),
    ],

    has_a: [
        // A Situation has Fluents (the fluents true in that situation)
        (Situation, Fluent),

        // An Action has a Precondition, a Postcondition, and Effects
        (Action, Precondition),
        (Action, Postcondition),
        (Action, Effect),

        // STRIPS: an Action has an AddList and a DeleteList
        (Action, AddList),
        (Action, DeleteList),

        // A Plan is made of Actions from an InitialState to a Goal
        (Plan, Action),
        (Plan, InitialState),
        (Plan, Goal),

        // A Trace is made of TraceEntries, each with a Transition
        (Trace, TraceEntry),
        (TraceEntry, Transition),
        (Transition, Situation),
        (Transition, Action),

        // An Agent has beliefs, desires, intentions (BDI)
        (Agent, Belief),
        (Agent, Desire),
        (Agent, Intention),

        // An Intention is about a Plan
        (Intention, Plan),
    ],

    opposes: [
        // AddList vs DeleteList — additive vs subtractive effect sets.
        (AddList, DeleteList),
        (DeleteList, AddList),

        // Belief vs Desire — "what IS vs what should be" — not strict
        // opposition but canonical BDI contrast.
        // (omitted — Bratman treats them as complementary, not opposite)

        // Precondition vs Postcondition — before/after duality.
        (Precondition, Postcondition),
        (Postcondition, Precondition),
    ],
}

/// Which tradition primarily introduces each situation-calculus concept.
#[derive(Debug, Clone)]
pub struct SituationCalculusTradition;

impl Quality for SituationCalculusTradition {
    type Individual = SituationCalculusConcept;
    type Value = &'static str;

    fn get(&self, c: &SituationCalculusConcept) -> Option<&'static str> {
        use SituationCalculusConcept as S;
        Some(match c {
            S::Situation | S::Action | S::Fluent | S::Effect | S::Transition => "mccarthy-1963",
            S::FrameAxiom => "mccarthy-hayes-1969",
            S::SuccessorStateAxiom => "reiter-2001",
            S::Precondition
            | S::Postcondition
            | S::ActionSchema
            | S::AddList
            | S::DeleteList
            | S::InitialState
            | S::Goal
            | S::Plan => "fikes-nilsson-1971",
            S::Trace | S::TraceEntry => "pr4xis-specific",
            S::Agent | S::Belief | S::Desire | S::Intention => "bratman-1987",
        })
    }
}

impl Ontology for SituationCalculusOntology {
    type Cat = SituationCalculusCategory;
    type Qual = SituationCalculusTradition;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        crate::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::laws::assert_category_laws;
    use crate::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<SituationCalculusCategory>();
    }

    #[test]
    fn ontology_validates() {
        SituationCalculusOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn strips_operators_have_add_and_delete_lists() {
        // Fikes & Nilsson (1971): every STRIPS operator (Action) has an
        // AddList and a DeleteList. Modelled as Action has-a both.
        let parthood: Vec<_> = SituationCalculusCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == SituationCalculusRelationKind::Parthood)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(parthood.contains(&(
            SituationCalculusConcept::Action,
            SituationCalculusConcept::AddList
        )));
        assert!(parthood.contains(&(
            SituationCalculusConcept::Action,
            SituationCalculusConcept::DeleteList
        )));
    }

    #[test]
    fn bdi_agent_has_belief_desire_intention() {
        // Bratman (1987): an Agent has beliefs, desires, and intentions.
        let parthood: Vec<_> = SituationCalculusCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == SituationCalculusRelationKind::Parthood)
            .map(|m| (m.source(), m.target()))
            .collect();
        for part in [
            SituationCalculusConcept::Belief,
            SituationCalculusConcept::Desire,
            SituationCalculusConcept::Intention,
        ] {
            assert!(
                parthood.contains(&(SituationCalculusConcept::Agent, part)),
                "Agent should have-a {:?}",
                part
            );
        }
    }

    #[test]
    fn successor_state_refines_frame_axiom() {
        // Reiter (1991/2001): successor-state axioms SUBSUME frame axioms
        // (one per fluent instead of O(actions × fluents) frame axioms).
        let sub: Vec<_> = SituationCalculusCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == SituationCalculusRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(sub.contains(&(
            SituationCalculusConcept::SuccessorStateAxiom,
            SituationCalculusConcept::FrameAxiom
        )));
    }

    #[test]
    fn every_concept_has_tradition() {
        let q = SituationCalculusTradition;
        for c in SituationCalculusConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing tradition", c);
        }
    }

    fn arb_concept() -> impl Strategy<Value = SituationCalculusConcept> {
        proptest::sample::select(SituationCalculusConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_tradition_total(c in arb_concept()) {
            prop_assert!(SituationCalculusTradition.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in SituationCalculusCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in SituationCalculusOntology::axioms() {
                match axiom.verify() {
                    Ok(_) => {}
                    Err(c) => prop_assert!(
                        false,
                        "structural axiom failed: {}",
                        c.meta().name.as_str()
                    ),
                }
            }
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = SituationCalculusConcept::variants();
            for m in SituationCalculusCategory::morphisms() {
                if m.kind() == SituationCalculusRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }
    }
}
