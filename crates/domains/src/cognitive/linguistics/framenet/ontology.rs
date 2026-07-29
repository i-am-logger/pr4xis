//! FrameNet as typed Rust data — the shape [`super::reader::read_framenet`]
//! populates from the committed, extracted TSV.
//!
//! Mirrors [`crate::cognitive::linguistics::conceptnet::ontology`]'s plain
//! struct-family shape: an instance-data loader, not a `pr4xis::ontology!`
//! category (same reasoning as VerbNet/ConceptNet — no schema-driven codegen
//! exists anywhere in this codebase, and this is bounded third-party corpus
//! data, not a domain reasoning category).
//!
//! Reference:
//! - Baker, C. F., Fillmore, C. J. & Lowe, J. B. (1998). "The Berkeley
//!   FrameNet Project." Proceedings of COLING-ACL 1998.
//! - Ruppenhofer, J., Ellsworth, M., Petruck, M. R. L., Johnson, C. R. &
//!   Scheffczyk, J. (2016). *FrameNet II: Extended Theory and Practice*.
//!   ICSI. The 9 frame-to-frame relation types this ontology's `relation`
//!   field carries: Inheritance, Using, Subframe, Perspective_on,
//!   Causative_of, Inchoative_of, Precedes, Metaphor, See_also.

#[allow(unused_imports)]
use alloc::{string::String, vec::Vec};

use crate::social::software::markup::xml::lmf::LmfPos;

/// One FrameNet lexical unit — a lemma+POS's membership in a semantic
/// frame (e.g. `cause.v` evokes `Causation`). Only OPEN-CLASS lexical
/// units survive the regen filter (`framenet::regenerate`'s module doc) —
/// closed-class FrameNet POS tags (PREP/NUM/INTJ/ART/C/SCON) have no
/// WordNet `ConceptId` to ever corroborate against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameNetLexicalUnit {
    pub lemma: String,
    pub pos: LmfPos,
    pub frame: String,
}

/// One frame-to-frame relation edge (Ruppenhofer et al. 2016) — carried as
/// raw provenance, not type-dispatched: like ConceptNet's `relation` field,
/// every FrameNet relation type is mapped GENERICALLY onto the existing
/// `Association` relation kind by [`super::store::FrameNetStore`], never a
/// fine-grained per-relation-type distinction (Inheritance is genuinely
/// hierarchical while the other 8 are lateral, but treating them
/// differently without a specific, tested reason would repeat exactly the
/// unprincipled per-source special-casing VerbNet's own regression already
/// proved dangerous).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameNetRelation {
    pub relation: String,
    pub sub_frame: String,
    pub super_frame: String,
}

/// The full loaded FrameNet lexical-unit + frame-relation data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrameNet {
    pub lexical_units: Vec<FrameNetLexicalUnit>,
    pub relations: Vec<FrameNetRelation>,
}
