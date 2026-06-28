//! Grounding — the process of establishing mutual understanding as a
//! finite state machine over discourse-unit lifecycle states.
//!
//! Objects are grounding states; morphisms are grounding acts that
//! transition between them. The "clean path" is
//! `Start → Initiated → Acknowledged → Grounded`; repair routes back
//! through `RepairRequested → Initiated`; cancellation goes to `Dead`.
//!
//! # Literature
//!
//! - **Clark & Schaefer (1989)** "Contributing to Discourse", *Cognitive
//!   Science* 13(2):259-294 — contribution = presentation + acceptance;
//!   levels of grounding evidence.
//! - **Clark (1996)** *Using Language*, Cambridge UP — Ch. 8 grounding
//!   in conversation; mutual belief criterion for the addressee's
//!   understanding.
//! - **Traum (1994)** *A Computational Theory of Grounding in Natural
//!   Language Conversation*, TR 545, University of Rochester — the
//!   seven grounding acts (Initiate, Continue, Acknowledge, Repair,
//!   RequestRepair, RequestAck, Cancel) as transitions over a
//!   discourse-unit lifecycle automaton.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "DialogueGrounding",
    source: "Clark & Schaefer (1989) Contributing to Discourse, Cognitive Science 13(2):259-294; Clark (1996) Using Language Ch. 8; Traum (1994) A Computational Theory of Grounding in Natural Language Conversation, TR 545, U. Rochester",

    concepts: [
        Start,
        Initiated,
        RepairRequested,
        Acknowledged,
        Grounded,
        Dead,
    ],

    labels: {
        Start: ("en", "Start",
            "Traum (1994): initial state - no discourse unit initiated yet."),
        Initiated: ("en", "Initiated",
            "Traum (1994): content presented by one participant; awaiting acknowledgment from the other."),
        RepairRequested: ("en", "Repair requested",
            "Traum (1994): the addressee has signalled non-understanding and requested clarification."),
        Acknowledged: ("en", "Acknowledged",
            "Clark (1996) Ch. 8: addressee has signalled sufficient understanding for current purposes; grounding criteria being established."),
        Grounded: ("en", "Grounded",
            "Clark (1996) Ch. 8: terminal success state - mutual belief that the content is understood sufficiently for current purposes."),
        Dead: ("en", "Dead",
            "Traum (1994): terminal failure state - the discourse unit was cancelled without grounding."),
    },

    edges: [
        // S → Initiated (start a new contribution). Traum (1994) Initiate.
        (Start, Initiated, Initiate),
        // Initiated → Initiated (extend from same speaker). Continue.
        (Initiated, Initiated, Continue),
        // Initiated → Acknowledged (addressee signals understanding).
        (Initiated, Acknowledged, Acknowledge),
        // Initiated → RepairRequested (addressee signals non-understanding).
        (Initiated, RepairRequested, RequestRepair),
        // Initiated → Initiated (self-repair by presenter).
        (Initiated, Initiated, Repair),
        // Initiated → Dead (cancel without grounding).
        (Initiated, Dead, Cancel),
        // RepairRequested → Initiated (presenter repairs).
        (RepairRequested, Initiated, Repair),
        // RepairRequested → Dead (give up).
        (RepairRequested, Dead, Cancel),
        // Acknowledged → Grounded (mutual belief established).
        (Acknowledged, Grounded, Acknowledge),
        // Acknowledged → Initiated (re-opened for correction).
        (Acknowledged, Initiated, Repair),
        // Initiated → Initiated (request ack — Traum 1994 RequestAck).
        (Initiated, Initiated, RequestAck),
    ],
}

/// Quality: whether a state is terminal in the grounding lifecycle.
/// Clark (1996) Ch. 8: Grounded is terminal-success; Traum (1994): Dead
/// is terminal-failure. All other states are non-terminal.
#[derive(Debug, Clone)]
pub struct IsTerminal;

impl Quality for IsTerminal {
    type Individual = DialogueGroundingConcept;
    type Value = bool;

    fn get(&self, c: &DialogueGroundingConcept) -> Option<bool> {
        use DialogueGroundingConcept as G;
        Some(matches!(c, G::Grounded | G::Dead))
    }
}

impl Ontology for DialogueGroundingOntology {
    type Cat = DialogueGroundingCategory;
    type Qual = IsTerminal;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<DialogueGroundingCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        DialogueGroundingOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn six_states() {
        assert_eq!(DialogueGroundingConcept::variants().len(), 6);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn clean_path_exists() {
        // Clark (1996) Ch. 8 + Traum (1994):
        // Start → Initiated → Acknowledged → Grounded.
        let m = DialogueGroundingCategory::morphisms();
        use DialogueGroundingConcept as G;
        assert!(m.iter().any(|r| r.source() == G::Start
            && r.target() == G::Initiated
            && r.kind() == DialogueGroundingRelationKind::Initiate));
        assert!(m.iter().any(|r| r.source() == G::Initiated
            && r.target() == G::Acknowledged
            && r.kind() == DialogueGroundingRelationKind::Acknowledge));
        assert!(m.iter().any(|r| r.source() == G::Acknowledged
            && r.target() == G::Grounded
            && r.kind() == DialogueGroundingRelationKind::Acknowledge));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn repair_path_exists() {
        // Traum (1994): Initiated → RepairRequested → Initiated.
        let m = DialogueGroundingCategory::morphisms();
        use DialogueGroundingConcept as G;
        assert!(m.iter().any(|r| r.source() == G::Initiated
            && r.target() == G::RepairRequested
            && r.kind() == DialogueGroundingRelationKind::RequestRepair));
        assert!(m.iter().any(|r| r.source() == G::RepairRequested
            && r.target() == G::Initiated
            && r.kind() == DialogueGroundingRelationKind::Repair));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cancel_always_leads_to_dead() {
        let m = DialogueGroundingCategory::morphisms();
        for cancel in m
            .iter()
            .filter(|r| r.kind() == DialogueGroundingRelationKind::Cancel)
        {
            assert_eq!(cancel.target(), DialogueGroundingConcept::Dead);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn terminal_states_marked() {
        assert_eq!(
            IsTerminal.get(&DialogueGroundingConcept::Grounded),
            Some(true)
        );
        assert_eq!(IsTerminal.get(&DialogueGroundingConcept::Dead), Some(true));
        assert_eq!(
            IsTerminal.get(&DialogueGroundingConcept::Initiated),
            Some(false)
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn grounded_has_no_non_identity_exits() {
        // Clark (1996): Grounded is the terminal success state.
        let m = DialogueGroundingCategory::morphisms();
        let exits: Vec<_> = m
            .iter()
            .filter(|r| {
                r.source() == DialogueGroundingConcept::Grounded
                    && r.target() != DialogueGroundingConcept::Grounded
                    && r.kind() != DialogueGroundingRelationKind::Identity
            })
            .collect();
        assert!(
            exits.is_empty(),
            "Grounded has non-identity exits: {:?}",
            exits
                .iter()
                .map(|m| (m.source(), m.target(), m.kind()))
                .collect::<Vec<_>>()
        );
    }

    fn arb_concept() -> impl Strategy<Value = DialogueGroundingConcept> {
        proptest::sample::select(DialogueGroundingConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in DialogueGroundingCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in DialogueGroundingOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_terminal_total(c in arb_concept()) {
            prop_assert!(IsTerminal.get(&c).is_some());
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
    pr4xis::register_praxis_value!(prop_terminal_total, Verifiable);
}
