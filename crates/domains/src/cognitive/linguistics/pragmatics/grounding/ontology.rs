// Conversation Grounding — how participants establish mutual understanding.
//
// Grounding is the process by which conversational participants ensure
// their contributions are understood. Every utterance must be grounded
// before the conversation can advance — this is what distinguishes
// dialogue from monologue.
//
// Clark & Schaefer (1989): contributions have two phases — presentation
// and acceptance. The contribution is grounded when accepted.
//
// Clark "Using Language" (1996): grounding is a joint activity with
// grounding criteria (evidence of understanding) that vary by medium.
//
// Traum (1994): computational grounding acts — acknowledge, continue,
// initiate, repair, request clarification.
//
// Ginzburg "The Interactive Stance" (2012) KoS: information states,
// dialogue gameboards, QUD (Questions Under Discussion) stack.

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Concepts in the Conversation Grounding ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum GroundingConcept {
    // === Clark & Schaefer (1989) contribution model ===
    /// Shared knowledge/beliefs between participants (Stalnaker 2002).
    CommonGround,
    /// The process of establishing mutual understanding.
    Grounding,
    /// An act that advances or maintains the grounding process.
    GroundingAct,
    /// Phase 1: speaker presents new content.
    Presentation,
    /// Phase 2: addressee signals understanding.
    Acceptance,
    /// A unit of discourse that has been jointly accepted.
    Contribution,

    // === Traum (1994) grounding acts ===
    /// Explicit signal of understanding (nod, "uh-huh", paraphrase).
    Acknowledgment,
    /// Implicit grounding by continuing the conversation.
    Continuation,
    /// Starting a new contribution.
    Initiation,
    /// Third-party or self-repair of a misunderstanding.
    Repair,
    /// Requesting clarification of a previous utterance.
    ClarificationRequest,

    // === Clark (1996) grounding criteria ===
    /// The standard of evidence required for grounding.
    GroundingCriterion,
    /// Evidence that the addressee understood (verbal, gestural, etc.).
    Evidence,

    // === Ginzburg (2012) KoS framework ===
    /// The participant's private + shared information state.
    InfoState,
    /// The public record of the dialogue state (QUD, moves, commitments).
    DialogueGameBoard,
    /// The most recent move in the dialogue.
    LatestMove,
    /// Content awaiting grounding (not yet integrated into common ground).
    Pending,
    /// The current Question Under Discussion driving the dialogue.
    MaxQUD,
    /// A participant's commitment to a proposition.
    Commitment,
}

define_ontology! {
    /// Conversation Grounding — mutual understanding in dialogue.
    pub GroundingOntology for GroundingCategory {
        concepts: GroundingConcept,
        relation: GroundingRelation,

        being: Process,
        source: "Clark & Schaefer (1989); Clark (1996); Traum (1994); Ginzburg (2012)",

        is_a: GroundingTaxonomy [
            // Grounding acts are specializations
            (Acknowledgment, GroundingAct),
            (Continuation, GroundingAct),
            (Initiation, GroundingAct),
            (Repair, GroundingAct),
            (ClarificationRequest, GroundingAct),
            // Presentation and Acceptance are phases of Grounding
            (Presentation, Grounding),
            (Acceptance, Grounding),
            // Grounding criterion and evidence relate to the process
            (GroundingCriterion, Grounding),
            (Evidence, Grounding),
        ],

        has_a: GroundingMereology [
            // Common ground is composed of grounded contributions
            (CommonGround, Contribution),
            (CommonGround, Commitment),
            // InfoState has a dialogue gameboard
            (InfoState, DialogueGameBoard),
            // Dialogue gameboard has its components
            (DialogueGameBoard, LatestMove),
            (DialogueGameBoard, Pending),
            (DialogueGameBoard, MaxQUD),
        ],

        causes: GroundingCausation for GroundingConcept [
            // Presentation causes Acceptance (or Repair)
            (Presentation, Acceptance),
            (Presentation, Repair),
            (ClarificationRequest, Repair),
            // Acceptance causes update to CommonGround
            (Acceptance, Contribution),
            // Grounding act causes update to InfoState
            (GroundingAct, InfoState),
        ],

        opposes: GroundingOpposition [
            // Presentation vs Acceptance: speaker vs addressee phase
            (Presentation, Acceptance),
            // Acceptance vs Repair: understanding vs misunderstanding
            (Acceptance, Repair),
            // Pending vs Contribution: ungrounded vs grounded
            (Pending, Contribution),
        ],
    }
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
    fn description(&self) -> &str {
        "every grounding act is-a GroundingAct (Traum 1994 taxonomy)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
        let rels = GroundingTaxonomy::relations();
        let acts = [
            GroundingConcept::Acknowledgment,
            GroundingConcept::Continuation,
            GroundingConcept::Initiation,
            GroundingConcept::Repair,
            GroundingConcept::ClarificationRequest,
        ];
        acts.iter().all(|act| {
            rels.iter()
                .any(|(child, parent)| child == act && *parent == GroundingConcept::GroundingAct)
        })
    }
}

/// Presentation causes either Acceptance or Repair (Clark & Schaefer 1989).
#[derive(Debug)]
pub struct PresentationHasConsequence;

impl Axiom for PresentationHasConsequence {
    fn description(&self) -> &str {
        "Presentation causes Acceptance or Repair (Clark & Schaefer 1989)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::causation::CausalDef;
        let rels = GroundingCausation::relations();
        let causes_accept = rels.iter().any(|(cause, effect)| {
            *cause == GroundingConcept::Presentation && *effect == GroundingConcept::Acceptance
        });
        let causes_repair = rels.iter().any(|(cause, effect)| {
            *cause == GroundingConcept::Presentation && *effect == GroundingConcept::Repair
        });
        causes_accept && causes_repair
    }
}

impl Ontology for GroundingOntology {
    type Cat = GroundingCategory;
    type Qual = IsKosFramework;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        GroundingOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(AllActsClassified),
            Box::new(PresentationHasConsequence),
        ]
    }
}
