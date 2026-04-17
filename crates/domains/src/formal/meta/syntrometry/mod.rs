//! Syntrometry — Heim's syntrometric logic, Phase 1.
//!
//! Encodes the distinction primitives, syntrometric structures, and
//! mereological primitive from Heim's *Syntrometrische Maximentelezentrik*
//! (modernised per the 2025 category-theoretic reformulation) plus a
//! verified functor to a small pr4xis-core substrate ontology. The functor
//! laws turn the long-standing lineage claim — "pr4xis instantiates Heim's
//! syntrometric structure" — into a tested theorem rather than a prose
//! assertion.
//!
//! Phase 2 (deferred): telecenters, transzendenzstufen, and maximes — the
//! teleological concepts that map to pr4xis's goal-directed planning
//! architecture.

pub mod adjunction;
pub mod lineage_functor;
pub mod meta_ontology_functor;
pub mod ontology;
#[cfg(test)]
mod proptests;
pub mod substrate;
pub mod substrate_functor;
