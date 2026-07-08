//! Canonical-codec ontology — the deterministic DAG-CBOR encoding the
//! content address is taken over, made a first-class praxis ontology with
//! runnable axioms (North Star W3 slice 1;
//! `feedback_praxis_as_compiler_self_describing`).
//!
//! The codec ([`pr4xis_runtime::codec`]) is the one piece of substrate
//! machinery whose determinism / round-trip / totality were proven only by
//! untagged `assert!`s inside the runtime crate — named in no `ontology!`,
//! outside the constitution partition. This module brings it INTO the
//! partition: it declares the codec's concepts and proves three genuinely
//! uncovered, machine-checkable facts about it as `verify()` predicates that
//! run against the real [`pr4xis_runtime::codec`] functions.
//!
//! - [`ontology`] — the concepts (`CanonicalEncoding`, `ContentAddress`,
//!   `CodecRoundTrip`, `DecodeTotality`) and the kinded `ContentAddress
//!   --depends-on--> CanonicalEncoding` dependency morphism. Always compiled;
//!   the [`pr4xis_runtime`] dependency is unconditional (no `feature = "prx"`
//!   gate — unlike [`super::ontology_archive`], whose realisation lives behind
//!   the `.prx` feature).
//! - [`axioms`] — the three runnable predicates: `CanonicalEncodingDeterministic`
//!   (value-stable bytes), `CodecRoundTrip` (`decode ∘ encode = id`), and
//!   `DecodeRefusesAdversarialLength` (fail-closed on untrusted input).

pub mod axioms;
pub mod ontology;
