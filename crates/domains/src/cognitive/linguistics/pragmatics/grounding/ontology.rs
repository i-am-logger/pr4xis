//! Conversation Grounding — how participants establish mutual understanding.
//!
//! Grounding is the process by which conversational participants ensure
//! their contributions are understood. Every utterance must be grounded
//! before the conversation can advance — this is what distinguishes
//! dialogue from monologue.
//!
//! Clark & Schaefer (1989): contributions have two phases — presentation
//! and acceptance. The contribution is grounded when accepted.
//!
//! Clark "Using Language" (1996): grounding is a joint activity with
//! grounding criteria (evidence of understanding) that vary by medium.
//!
//! Traum (1994): computational grounding acts — acknowledge, continue,
//! initiate, repair, request clarification.
//!
//! Ginzburg "The Interactive Stance" (2012) KoS: information states,
//! dialogue gameboards, QUD (Questions Under Discussion) stack.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Grounding",
    source: "Clark & Schaefer (1989); Clark (1996); Traum (1994); Ginzburg (2012)",

    concepts: [
        CommonGround,
        Grounding,
        GroundingAct,
        Presentation,
        Acceptance,
        Contribution,
        Acknowledgment,
        Continuation,
        Initiation,
        Repair,
        ClarificationRequest,
        GroundingCriterion,
        Evidence,
        InfoState,
        DialogueGameBoard,
        LatestMove,
        Pending,
        MaxQUD,
        Commitment,
    ],

    labels: {
        CommonGround: ("en", "Common ground", "Shared knowledge/beliefs between participants (Stalnaker 2002)."),
        Grounding: ("en", "Grounding", "The process of establishing mutual understanding."),
        GroundingAct: ("en", "Grounding act", "An act that advances or maintains the grounding process."),
        Presentation: ("en", "Presentation", "Phase 1: speaker presents new content."),
        Acceptance: ("en", "Acceptance", "Phase 2: addressee signals understanding."),
        Contribution: ("en", "Contribution", "A unit of discourse that has been jointly accepted."),
        Acknowledgment: ("en", "Acknowledgment", "Explicit signal of understanding (nod, 'uh-huh', paraphrase)."),
        Continuation: ("en", "Continuation", "Implicit grounding by continuing the conversation."),
        Initiation: ("en", "Initiation", "Starting a new contribution."),
        Repair: ("en", "Repair", "Third-party or self-repair of a misunderstanding."),
        ClarificationRequest: ("en", "Clarification request", "Requesting clarification of a previous utterance."),
        GroundingCriterion: ("en", "Grounding criterion", "The standard of evidence required for grounding."),
        Evidence: ("en", "Evidence", "Evidence that the addressee understood (verbal, gestural, etc.)."),
        InfoState: ("en", "Info state", "The participant's private + shared information state."),
        DialogueGameBoard: ("en", "Dialogue game board", "The public record of the dialogue state (QUD, moves, commitments)."),
        LatestMove: ("en", "Latest move", "The most recent move in the dialogue."),
        Pending: ("en", "Pending", "Content awaiting grounding (not yet integrated into common ground)."),
        MaxQUD: ("en", "Max QUD", "The current Question Under Discussion driving the dialogue."),
        Commitment: ("en", "Commitment", "A participant's commitment to a proposition."),
    },

    is_a: [
        (Acknowledgment, GroundingAct),
        (Continuation, GroundingAct),
        (Initiation, GroundingAct),
        (Repair, GroundingAct),
        (ClarificationRequest, GroundingAct),
        (Presentation, Grounding),
        (Acceptance, Grounding),
        (GroundingCriterion, Grounding),
        (Evidence, Grounding),
    ],

    has_a: [
        (CommonGround, Contribution),
        (CommonGround, Commitment),
        (InfoState, DialogueGameBoard),
        (DialogueGameBoard, LatestMove),
        (DialogueGameBoard, Pending),
        (DialogueGameBoard, MaxQUD),
    ],

    causes: [
        (Presentation, Acceptance),
        (Presentation, Repair),
        (ClarificationRequest, Repair),
        (Acceptance, Contribution),
        (GroundingAct, InfoState),
    ],

    opposes: [
        (Presentation, Acceptance),
        (Acceptance, Repair),
        (Pending, Contribution),
    ],
}

/// Whether a concept is from the Clark/Traum model vs Ginzburg KoS.
#[derive(Debug, Clone)]
pub struct IsKosFramework;

impl Quality for IsKosFramework {
    type Individual = GroundingConcept;
    type Value = bool;

    fn get(&self, individual: &GroundingConcept) -> Option<bool> {
        Some(matches!(
            individual,
            GroundingConcept::InfoState
                | GroundingConcept::DialogueGameBoard
                | GroundingConcept::LatestMove
                | GroundingConcept::Pending
                | GroundingConcept::MaxQUD
        ))
    }
}

/// All grounding acts are classified in the taxonomy (Traum 1994).
#[derive(Debug)]
pub struct AllActsClassified;

impl Axiom for AllActsClassified {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let subs: Vec<_> = GroundingCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == GroundingRelationKind::Subsumption)
            .collect();
        let acts = [
            GroundingConcept::Acknowledgment,
            GroundingConcept::Continuation,
            GroundingConcept::Initiation,
            GroundingConcept::Repair,
            GroundingConcept::ClarificationRequest,
        ];
        if acts.iter().all(|act| {
            subs.iter()
                .any(|m| m.source() == *act && m.target() == GroundingConcept::GroundingAct)
        }) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AllActsClassified",
        "every grounding act is-a GroundingAct (Traum 1994 taxonomy)",
        "Traum (1994) A Computational Theory of Grounding in Natural Language Conversation"
    );
}
pr4xis::register_axiom!(
    AllActsClassified,
    "Traum (1994) A Computational Theory of Grounding in Natural Language Conversation"
);

/// Presentation causes either Acceptance or Repair (Clark & Schaefer 1989).
#[derive(Debug)]
pub struct PresentationHasConsequence;

impl Axiom for PresentationHasConsequence {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let causal: Vec<_> = GroundingCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == GroundingRelationKind::Causation)
            .collect();
        let causes_accept = causal.iter().any(|m| {
            m.source() == GroundingConcept::Presentation
                && m.target() == GroundingConcept::Acceptance
        });
        let causes_repair = causal.iter().any(|m| {
            m.source() == GroundingConcept::Presentation && m.target() == GroundingConcept::Repair
        });
        if causes_accept && causes_repair {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PresentationHasConsequence",
        "Presentation causes Acceptance or Repair (Clark & Schaefer 1989)",
        "Clark & Schaefer (1989) Contributing to Discourse, Cognitive Science 13"
    );
}
pr4xis::register_axiom!(
    PresentationHasConsequence,
    "Clark & Schaefer (1989) Contributing to Discourse, Cognitive Science 13"
);

impl Ontology for GroundingOntology {
    type Cat = GroundingCategory;
    type Qual = IsKosFramework;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(AllActsClassified));
        axioms.push(Box::new(PresentationHasConsequence));
        axioms
    }
}
