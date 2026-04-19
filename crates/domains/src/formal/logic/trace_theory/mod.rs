//! Trace theory — Plotkin / Plotkin-Power / Abramsky-Jung / Hyland-Ong.
//! Operational and game-semantic view of derivations.

pub mod derivation_functor;
pub mod ontology;

pub use derivation_functor::TraceTheoryToDerivation;
pub use ontology::{
    TraceTheoryCategory, TraceTheoryConcept, TraceTheoryOntology, TraceTheoryRelation,
    TraceTradition,
};
