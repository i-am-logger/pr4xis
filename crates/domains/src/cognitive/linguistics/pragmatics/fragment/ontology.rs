// Fragment Resolution — non-sentential utterances in dialogue.
//
// Most dialogue utterances are not full sentences. "Yes", "the red one",
// "tomorrow?", "John" — these fragments are interpretable only in context.
// This ontology models how fragments are resolved to full propositions
// using the QUD (Question Under Discussion) and dialogue context.
//
// Source: Fernandez & Ginzburg (2002, 2006); Ginzburg "The Interactive Stance" (2012);
//         Schlangen (2003); Purver et al. (2006)

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Concepts in the Fragment Resolution ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum FragmentConcept {
    /// A non-sentential utterance — syntactically incomplete.
    Fragment,
    /// The resolved full proposition derived from fragment + context.
    ResolvedContent,
    /// The Question Under Discussion that licenses the fragment.
    QUD,
    /// The dialogue context providing resolution material.
    DialogueContext,

    // === Fragment types (Fernandez & Ginzburg 2002) ===
    /// Short answer to a question: "John" → "John left".
    ShortAnswer,
    /// Acknowledgment/confirmation: "Yes" / "OK".
    Affirmation,
    /// Rejection: "No" / "Not really".
    Rejection,
    /// Sluice — bare wh-word: "Who?" / "Where?".
    Sluice,
    /// Clarification ellipsis: "John?" (did you say John?).
    ClarificationEllipsis,
    /// Propositional fragment completing a prior partial: "tomorrow".
    Completion,
    /// Correction of a prior utterance: "no, TUESDAY".
    Correction,
    /// Plain acknowledgment token: "uh-huh", "right".
    AcknowledgmentToken,
}

define_ontology! {
    /// Fragment Resolution — interpreting non-sentential utterances.
    pub FragmentOntology for FragmentCategory {
        concepts: FragmentConcept,
        relation: FragmentRelation,

        being: AbstractObject,
        source: "Fernandez & Ginzburg (2002, 2006); Ginzburg (2012); Schlangen (2003)",

        is_a: FragmentTaxonomy [
            (ShortAnswer, Fragment),
            (Affirmation, Fragment),
            (Rejection, Fragment),
            (Sluice, Fragment),
            (ClarificationEllipsis, Fragment),
            (Completion, Fragment),
            (Correction, Fragment),
            (AcknowledgmentToken, Fragment),
        ],

        opposes: FragmentOpposition [
            (Affirmation, Rejection),
            (Fragment, ResolvedContent),
        ],
    }
}

/// Whether a concept is a fragment type vs structural.
#[derive(Debug, Clone)]
pub struct IsFragmentType;

impl Quality for IsFragmentType {
    type Individual = FragmentConcept;
    type Value = bool;

    fn get(&self, individual: &FragmentConcept) -> Option<bool> {
        Some(!matches!(
            individual,
            FragmentConcept::Fragment
                | FragmentConcept::ResolvedContent
                | FragmentConcept::QUD
                | FragmentConcept::DialogueContext
        ))
    }
}

/// All fragment types are classified (Fernandez & Ginzburg 2002 taxonomy).
#[derive(Debug)]
pub struct AllFragmentsClassified;

impl Axiom for AllFragmentsClassified {
    fn description(&self) -> &str {
        "every fragment type is-a Fragment (Fernandez & Ginzburg 2002)"
    }
    fn holds(&self) -> bool {
        use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
        let rels = FragmentTaxonomy::relations();
        let types = [
            FragmentConcept::ShortAnswer,
            FragmentConcept::Affirmation,
            FragmentConcept::Rejection,
            FragmentConcept::Sluice,
            FragmentConcept::ClarificationEllipsis,
            FragmentConcept::Completion,
            FragmentConcept::Correction,
            FragmentConcept::AcknowledgmentToken,
        ];
        types.iter().all(|t| {
            rels.iter()
                .any(|(child, parent)| child == t && *parent == FragmentConcept::Fragment)
        })
    }
}

impl Ontology for FragmentOntology {
    type Cat = FragmentCategory;
    type Qual = IsFragmentType;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        FragmentOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(AllFragmentsClassified)]
    }
}
