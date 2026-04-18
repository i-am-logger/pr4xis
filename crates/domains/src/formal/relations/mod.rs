//! Relations — canonical binary relation types (issue #152).
//!
//! Enumerates the ten canonical binary relation types used across pr4xis
//! ontologies (Subsumption, Parthood, Causation, Opposition, Similarity,
//! Precedence, Equivalence, Specialisation, Dependence, Association) plus
//! the seven Tarski-algebraic structural properties each relation type
//! satisfies.
//!
//! Foundation for kind-parameterised structural axioms in
//! `pr4xis::ontology::reasoning::structural`. Edge kind names emitted by
//! `pr4xis::ontology!` (e.g. `Subsumption`) match concept names here by
//! convention — the Relations ontology is the authoritative catalog.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

pub mod dialectics_functor;
pub mod ontology;

pub use dialectics_functor::RelationsToDialectics;
pub use ontology::{
    RelationProperty, RelationsCategory, RelationsConcept, RelationsOntology, RelationsRelation,
    RelationsRelationKind,
};
