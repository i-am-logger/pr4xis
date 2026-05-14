//! Provenance — the substrate ontology grounding `ontology/meta.rs`.
//!
//! Names the W3C PROV-O (2013) concepts (Artifact, Activity, Agent,
//! Source, Citation, …) that core's `Provenance.citation` and
//! `Provenance.module_path` fields realise.

pub mod ontology;

pub use ontology::{IsProvOCore, ProvenanceCategory, ProvenanceConcept, ProvenanceOntology};
