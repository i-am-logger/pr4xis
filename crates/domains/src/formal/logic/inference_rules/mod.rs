//! Inference rules — canonical catalog.
//! Hilbert-Ackermann / Gentzen / Peirce / Carnap.

pub mod ontology;
pub mod proof_theory_functor;

pub use ontology::{
    InferenceRulesCategory, InferenceRulesConcept, InferenceRulesOntology, InferenceRulesRelation,
    RuleMode, RuleOrigin,
};
pub use proof_theory_functor::InferenceRulesToProofTheory;
