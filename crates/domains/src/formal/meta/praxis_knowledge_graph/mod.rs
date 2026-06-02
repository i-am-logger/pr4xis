//! Praxis knowledge-graph ontology (#272) — a content-addressed graph-SLICE
//! primitive: select a subgraph, emit it as a deterministic content-addressed
//! binary, load it through a fail-closed gate, re-bind behavioural nodes by
//! name. (The *whole* graph is just the default selection; the
//! request-for-missing negotiation that would let a peer LEARN what it lacks is
//! a separate, deferred layer — not this primitive.)
//!
//! - [`ontology`] declares the 32 concepts (structural-knowledge nodes,
//!   their pair-ontology bindings, selection/slicing, and the
//!   content-addressed storage substratum).
//! - [`functor`] is the fully-faithful `ArchiveIntoGraph` embedding of
//!   [`OntologyArchiveStorage`](super::ontology_archive) — created
//!   alongside, never a rewrite (`feedback_evolution_via_functor`).
//! - [`axioms`] are the runnable axioms (the functor's full-and-faithful
//!   laws, the eight re-exported archive axioms, the two re-bind axioms, and
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

// The whole-graph GraphSnapshot machinery (#271 effort B): select a slice,
// content-address it as a Merkle DAG, rehydrate it through the fail-closed
// admit gate, re-binding behavioural nodes. Generalises the selection BFS +
// reuses the `.prx` primitives; adds NO new ontology edges (the functor stays
// fully faithful). Gated on `feature = "prx"` like the axioms it unlocks.
#[cfg(feature = "prx")]
pub mod snapshot;
