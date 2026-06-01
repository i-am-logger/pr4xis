//! Ontology-archive storage — the praxis ontology + runnable axioms for
//! content-addressed Merkle-DAG storage that the `.prx` machinery
//! realises (M4.ι.0 / task #175).
//!
//! - [`ontology`] — the concepts (`ContentAddressableNode`, `MerkleDag`,
//!   `BinaryEnvelope`, `SourcePin`, `LoadGate`, …) and their structure;
//!   always compiled, format-agnostic.
//! - [`axioms`] — the runnable `verify()` predicates that exercise the
//!   real realisation; gated on `feature = "prx"` (where the first
//!   realisation lives). USC (#271) becomes a second consumer of the
//!   same ontology.

pub mod ontology;

#[cfg(feature = "prx")]
pub mod axioms;
