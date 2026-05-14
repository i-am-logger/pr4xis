//! Model theory — Tarski / Chang-Keisler / Hodges / Gödel.
//! Semantic entailment and satisfaction.

pub mod derivation_functor;
pub mod ontology;

pub use derivation_functor::ModelTheoryToDerivation;
pub use ontology::{
    ModelTheoryCategory, ModelTheoryConcept, ModelTheoryOntology, ModelTheoryRelation,
    ModelTheoryTradition,
};
