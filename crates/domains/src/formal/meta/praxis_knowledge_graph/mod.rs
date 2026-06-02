//! Praxis knowledge-graph ontology (#272 / PR 1) — praxis as a whole-graph
//! wire protocol.
//!
//! - [`ontology`] declares the 32 concepts (structural-knowledge nodes,
//!   their pair-ontology bindings, selection/slicing, and the
//!   content-addressed storage substratum).
//! - [`functor`] is the fully-faithful `ArchiveIntoGraph` embedding of
//!   [`OntologyArchiveStorage`](super::ontology_archive) — created
//!   alongside, never a rewrite (`feedback_evolution_via_functor`).
//! - [`axioms`] are the runnable axioms (the functor's full-and-faithful
//!   laws, the seven re-exported archive axioms, the two re-bind axioms, and
//!   the lens / selection / pair-round-trip axioms; the snapshot / attestation
//!   axioms are deferred there, not stubbed).
//!
//! The behavioural axioms exercise the `.prx` realisation and the
//! registries, so [`axioms`] and [`functor`] are gated on `feature = "prx"`;
//! the concept declaration in [`ontology`] is format-agnostic and always
//! compiles.

pub mod ontology;

#[cfg(feature = "prx")]
pub mod axioms;

#[cfg(feature = "prx")]
pub mod functor;
