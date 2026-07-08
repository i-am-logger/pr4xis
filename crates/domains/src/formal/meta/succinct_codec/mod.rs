//! Succinct-codec ontology — the compact bit-packed `.prx` wire format the
//! `.cprx.gz` English corpus and the registry ship in, made a first-class praxis
//! ontology with runnable axioms (North Star W3 slice 3;
//! `feedback_praxis_as_compiler_self_describing`).
//!
//! The succinct codec (the bit-packing kernel
//! [`markup::xml::succinct`](crate::social::software::markup::xml::succinct) and
//! [`OwnedCodegenData::to_succinct`](crate::social::software::markup::xml::owl::prx::OwnedCodegenData::to_succinct)/`from_succinct`)
//! is the COMPACT wire form — bit-packed columns, gap-coded monotone offsets, a
//! front-coded string dictionary — whose round-trip and compaction were proven
//! by exactly one on-disk integration test, named in no `ontology!`, outside the
//! constitution partition. This module brings it in: it declares the codec's
//! concepts and proves three genuinely uncovered, non-tautological,
//! machine-checkable facts about it as `verify()` predicates that run against
//! the real codec.
//!
//! - [`ontology`] — the concepts (`SuccinctEncoding`, `BitPackedColumn`,
//!   `MonotoneGapColumn`, `FrontCodedDictionary`, `SuccinctRoundTrip`) and their
//!   kinded mereology/dependency morphisms.
//! - [`axioms`] — the three runnable predicates: `SuccinctCodecRoundTrip`
//!   (`from_succinct ∘ to_succinct = id`), `MonotoneOffsetsCompact` (gap coding
//!   is lossless AND strictly smaller than absolute packing), and
//!   `FrontCodingSharesPrefixes` (front coding is order-independent AND strictly
//!   smaller than the plain dictionary on shared prefixes).
//!
//! Gated on `feature = "prx"`: the codec this ontology describes only exists
//! under `prx` (both `markup::xml::succinct` and `owl::prx` are `#[cfg(feature =
//! "prx")]`), so the ontology that self-describes it is gated with it — unlike
//! [`super::canonical_codec`], whose DAG-CBOR codec is unconditionally compiled.

pub mod axioms;
pub mod ontology;
