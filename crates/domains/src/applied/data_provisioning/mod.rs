//! Data provisioning — the applied subsystem for managing external data
//! dependencies.
//!
//! Composes several pr4xis ontologies into a uniform fetch + content-type-
//! polymorphic decoder chain:
//!
//! - `formal/meta/artifact_identity/` — every `DataSource` declares its
//!   identity via one of the four families (ContentHash, CryptographicSignature,
//!   PersistentIdentifier, SelfDescribingMetadata)
//! - `formal/information/storage/` — the local `DataCache` is a `Store`;
//!   materialization is the existing `Materialize` morphism
//! - `formal/information/provenance/` — every fetch is a `prov:Activity`
//!   recorded against the data source
//! - `formal/meta/staging/` — a fetch is an instance of the Futamura
//!   `freeze: Dynamic → Static` functor; the URL is dynamic, the local
//!   file is static, staging level = 1
//!
//! The content-type polymorphism is the key extensibility axis: the fetch
//! pipeline is uniform (download bytes, verify identity, write to disk),
//! while the decoder chain is content-type-specific. The initial PR wires
//! `XmlLmf` to the existing `xml_reader → lmf::reader → English::from_wordnet`
//! pipeline. Future PRs add `Pdf`, `Video`, `Audio`, etc. as new decoder
//! chains without touching the fetch path.
//!
//! Sources:
//! - Dolstra 2006 (via artifact_identity) — fixed-output derivations
//! - Wilkinson et al. 2016 — FAIR Guiding Principles, F1 + A1 + R1
//! - W3C PROV-O — the provenance model (via `formal/information/provenance/`)
//! - Global WordNet Association WN-LMF 1.3 — the upstream schema for
//!   WordNet, the initial and only registered data source

#[cfg(feature = "std")]
pub mod chat_lexicons;
pub mod corpus_parse_signature;
pub mod decoders;
/// The `[compact_defines_signatures]` grammar-staleness fingerprint —
/// requires `std` (it reads workspace-relative `.rs` source files), like
/// [`chat_lexicons`].
#[cfg(feature = "std")]
pub mod defines_grammar_signature;
pub mod lexicon_provenance;
pub mod lockfile;
pub mod ontology;
/// The ONE canonical `.prx` envelope codec (LEB128 framing + raw-source and
/// registry envelope grammars), shared byte-for-byte between the runtime load
/// path (`raw_source_prx`, `registry_prx`) and the build script, which
/// `#[path]`-includes it. `pub` (not `pub(crate)`) because its codec symbols are
/// re-exported as public API by `raw_source_prx` / `registry_prx` (as they were
/// before extraction) and several are referenced only from `#[cfg(test)]` code.
pub mod prx_envelope;
pub mod raw_source_prx;
pub mod registry;
pub mod registry_prx;
pub mod source_role;
/// The derived definitional lexicon naming the U.S. Code titles a deployment
/// HOLDS — composed at load time from the registry's descriptions and the
/// materialized corpus, so it shares [`chat_lexicons`]'s `std` gate (it rides
/// the same WN-LMF bridge).
#[cfg(feature = "std")]
pub mod usc_title_lexicon;

#[cfg(feature = "fetch")]
pub mod fetch;

#[cfg(test)]
mod tests;
