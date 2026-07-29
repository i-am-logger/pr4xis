//! ConceptNet as typed Rust data — the shape [`super::reader::read_conceptnet`]
//! populates from the committed, WordNet-lemma-crosswalk-filtered TSV.
//!
//! Mirrors [`crate::cognitive::linguistics::verbnet::ontology`]'s plain
//! struct-family shape: an instance-data loader, not a `pr4xis::ontology!`
//! category (VerbNet has neither — see that module's doc for why: no schema-
//! driven codegen exists anywhere in this codebase, and this is bounded
//! third-party corpus data, not a domain reasoning category).
//!
//! Reference:
//! - Speer, R., Chin, J. & Havasi, C. (2017). "ConceptNet 5.5: An Open
//!   Multilingual Graph of General Knowledge". Proceedings of AAAI 2017.

#[allow(unused_imports)]
use alloc::{string::String, vec::Vec};

/// One ConceptNet assertion, already filtered to the case both endpoints
/// resolve to a loaded WordNet lemma (see
/// [`crate::applied::data_provisioning::decoders`] module doc's raw-source
/// path and this ontology's `regenerate` module for the filter). Fields carry
/// the raw assertion data unchanged (a pure format-conversion, per the
/// `AssociativeConceptTable` taxonomy leaf's license note) — no relation-type
/// interpretation happens here; that's [`super::store::ConceptNetStore`]'s
/// job, and it deliberately stays coarse (see that module's doc).
#[derive(Debug, Clone, PartialEq)]
pub struct ConceptNetEdge {
    /// The ConceptNet relation URI with the `/r/` prefix stripped (e.g.
    /// `"RelatedTo"`, `"IsA"`, `"UsedFor"`) — carried as provenance, not
    /// type-dispatched: the corroboration mechanism maps every relation
    /// generically onto the existing `Association` relation kind (see
    /// `formal::relations::ontology::Association`'s own citation, SKOS
    /// `related`) rather than a fine-grained per-relation mapping.
    pub relation: String,
    /// The start concept's normalized lemma (see
    /// [`super::store::normalize_lemma`] — lowercase, spaces and hyphens
    /// folded to `_`, matching ConceptNet's own `/c/en/…` URI convention).
    pub start_lemma: String,
    /// The end concept's normalized lemma, same convention.
    pub end_lemma: String,
    /// The assertion's weight field, carried through unchanged (ConceptNet's
    /// per-assertion confidence — sourced from the JSON metadata blob's
    /// `"weight"` key; typically 1.0 for a single OMCS mention, higher for
    /// multiply-attested or curated-dataset assertions).
    pub weight: f32,
}

/// The full loaded, filtered ConceptNet assertion set.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConceptNet {
    pub edges: Vec<ConceptNetEdge>,
}
