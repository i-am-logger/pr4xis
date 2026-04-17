//! Syntrometry — Heim's syntrometric logic.
//!
//! Encodes the distinction primitives, syntrometric structures, mereology,
//! and teleological concepts from Heim's *Syntrometrische Maximentelezentrik*
//! (modernised per the 2025 category-theoretic reformulation) plus a family
//! of verified functors into pr4xis's substrate and existing meta,
//! composition, staging, and cognitive ontologies. The functor laws turn
//! the long-standing lineage claim — "pr4xis instantiates Heim's
//! syntrometric structure" — into a tested theorem rather than a prose
//! assertion.

pub mod adjunction;
pub mod algebra_functor;
pub mod consciousness_functor;
pub mod dialectics_functor;
pub mod distinction_functor;
pub mod lineage_functor;
pub mod meta_ontology_functor;
pub mod ontology;
#[cfg(test)]
mod proptests;
pub mod staging_functor;
pub mod substrate;
pub mod substrate_functor;
