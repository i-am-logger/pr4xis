//! Proof theory — Gentzen / Prawitz / Troelstra-Schwichtenberg / Girard-Lafont-Taylor.

pub mod derivation_functor;
pub mod ontology;
pub mod trace_theory_functor;

pub use derivation_functor::ProofTheoryToDerivation;
pub use ontology::{
    ProofTheoryCategory, ProofTheoryConcept, ProofTheoryOntology, ProofTheoryRelation,
    ProofTheoryTradition,
};
pub use trace_theory_functor::ProofTheoryToTraceTheory;
