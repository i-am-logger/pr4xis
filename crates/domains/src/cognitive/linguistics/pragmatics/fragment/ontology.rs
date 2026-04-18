//! Fragment Resolution — non-sentential utterances in dialogue.
//!
//! Most dialogue utterances are not full sentences. "Yes", "the red one",
//! "tomorrow?", "John" — these fragments are interpretable only in context.
//! This ontology models how fragments are resolved to full propositions
//! using the QUD (Question Under Discussion) and dialogue context.
//!
//! Source: Fernandez & Ginzburg (2002, 2006); Ginzburg "The Interactive Stance" (2012);
//!         Schlangen (2003); Purver et al. (2006)

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Fragment",
    source: "Fernandez & Ginzburg (2002, 2006); Ginzburg (2012); Schlangen (2003)",
    being: AbstractObject,

    concepts: [
        Fragment,
        ResolvedContent,
        QUD,
        DialogueContext,
        ShortAnswer,
        Affirmation,
        Rejection,
        Sluice,
        ClarificationEllipsis,
        Completion,
        Correction,
        AcknowledgmentToken,
    ],

    labels: {
        Fragment: ("en", "Fragment", "A non-sentential utterance — syntactically incomplete."),
        ResolvedContent: ("en", "Resolved content", "The resolved full proposition derived from fragment + context."),
        QUD: ("en", "Question Under Discussion", "The QUD that licenses the fragment."),
        DialogueContext: ("en", "Dialogue context", "The dialogue context providing resolution material."),
        ShortAnswer: ("en", "Short answer", "Short answer to a question: 'John' → 'John left'."),
        Affirmation: ("en", "Affirmation", "Acknowledgment/confirmation: 'Yes' / 'OK'."),
        Rejection: ("en", "Rejection", "Rejection: 'No' / 'Not really'."),
        Sluice: ("en", "Sluice", "Bare wh-word: 'Who?' / 'Where?'."),
        ClarificationEllipsis: ("en", "Clarification ellipsis", "'John?' (did you say John?)."),
        Completion: ("en", "Completion", "Propositional fragment completing a prior partial: 'tomorrow'."),
        Correction: ("en", "Correction", "Correction of a prior utterance: 'no, TUESDAY'."),
        AcknowledgmentToken: ("en", "Acknowledgment token", "Plain acknowledgment token: 'uh-huh', 'right'."),
    },

    is_a: [
        (ShortAnswer, Fragment),
        (Affirmation, Fragment),
        (Rejection, Fragment),
        (Sluice, Fragment),
        (ClarificationEllipsis, Fragment),
        (Completion, Fragment),
        (Correction, Fragment),
        (AcknowledgmentToken, Fragment),
    ],

    opposes: [
        (Affirmation, Rejection),
        (Fragment, ResolvedContent),
    ],
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
pr4xis::register_axiom!(
    AllFragmentsClassified,
    "Fernandez & Ginzburg (2002, 2006); Ginzburg \"The Interactive Stance\" (2012);"
);

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
