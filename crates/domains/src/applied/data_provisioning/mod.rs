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

pub mod decoders;
pub mod ontology;
pub mod registry;

#[cfg(feature = "fetch")]
pub mod fetch;

#[cfg(test)]
mod tests;
