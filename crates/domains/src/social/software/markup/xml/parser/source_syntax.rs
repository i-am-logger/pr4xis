//! The SourceSyntax COMPLEMENT — the concrete-syntax decisions the W3C
//! Information Set (Cowan & Tobin 2004) does not carry, kept SEPARATE from the
//! Infoset DOM so that byte-exact reconstruction is possible without storing raw
//! source.
//!
//! `serialize_document` (the canonical `put`) normalizes away byte-affecting
//! choices that W3C XML Canonicalization 1.1 (Boyer & Marcy 2008) erases — a
//! childless element always self-closes, entity escaping is the canonical set.
//! `serialize_document_exact` is its byte-exact sibling: it reproduces the
//! ORIGINAL bytes from the Infoset DOM PLUS the recorded [`SyntaxDecisions`].
//! This module holds the residue types so BOTH the reader (`grammar`, which
//! CAPTURES the decisions) and the writer (`serializer`, which HONOURS them) can
//! share one definition without depending on each other.
//!
//! # The pre-order element index keying
//!
//! Each element's decisions are keyed by its 0-based PRE-ORDER element index —
//! the order a depth-first walk ENTERS elements (root = 0, then each child in
//! document order, descending before advancing to a sibling). Both the reader's
//! capture counter and the writer's emit counter increment at element ENTRY, so
//! the two walks agree index-for-index. This is simpler and more robust than a
//! child-path key: it is a single `usize`, it survives entity-inclusion
//! splicing (the included element still occupies one pre-order slot), and it
//! needs no path bookkeeping.
//!
//! # Citations
//!
//! - **Bray et al. (2008)** XML 1.0 Fifth Edition §3.1 — the empty-element form
//!   decision (`EmptyElemTag` `<a/>` versus `STag content ETag` `<a></a>`).
//! - **Cowan & Tobin (2004)** XML Information Set — the Infoset items the DOM
//!   carries; the empty-element form is NOT one of them, hence this complement.
//! - **Foster et al. (2007)** ACM TOPLAS 29(3) — the well-behaved lens whose
//!   byte-exact `put` takes the DOM and this complement as its two inputs.

use alloc::collections::BTreeMap;

/// The empty-element form decision (W3C XML 1.0 §3.1): `<a/>` (empty-element
/// tag) versus `<a></a>` (start- plus end-tag) — the same Information Set,
/// distinct bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyForm {
    /// `<a/>`.
    SelfClosing,
    /// `<a></a>`.
    Explicit,
}

/// The concrete-syntax decisions for ONE node — the SourceSyntax residue the
/// Information Set DOM does not carry, needed to reproduce its exact bytes.
/// Today only the empty-element form; further decisions (entity-reference form,
/// attribute-value escaping, white-space) extend this struct.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeDecisions {
    /// The empty-element form — only meaningful for a childless element.
    pub empty_form: Option<EmptyForm>,
}

/// A document's concrete-syntax decisions, keyed by 0-based PRE-ORDER element
/// index (the order a depth-first walk ENTERS elements; the root element is
/// index `0`). The byte-exact serializer threads a matching pre-order counter
/// and looks up each element's decisions as it walks.
///
/// This is the SourceSyntax COMPLEMENT, kept SEPARATE from the Infoset DOM: the
/// decisions live in the per-source `.prx` envelope, never in the ontology or
/// its content-address (so the same ontology serialized two ways keeps one
/// root). It is the byte-exact `put`'s second input, beside the DOM.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SyntaxDecisions {
    by_index: BTreeMap<usize, NodeDecisions>,
}

impl SyntaxDecisions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the concrete-syntax decisions for the element at pre-order
    /// `index`.
    pub fn set(&mut self, index: usize, decisions: NodeDecisions) {
        self.by_index.insert(index, decisions);
    }

    /// The concrete-syntax decisions recorded for the element at pre-order
    /// `index`, if any.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&NodeDecisions> {
        self.by_index.get(&index)
    }
}

/// The reader-side capture state for the serialized reverse lens: a pre-order
/// element counter paired with the [`SyntaxDecisions`] being accumulated.
///
/// `parse_document_capturing` threads `&mut Option<CaptureCtx>` through the
/// element/content descent. `parse_element` takes the current `counter` value
/// as the element's pre-order index AT ENTRY (then increments), so a child's
/// index is always greater than its parent's and increases in document order —
/// the exact walk the byte-exact serializer replays. A non-canonical decision
/// (today: an explicit-empty `<a></a>`) is recorded against that index.
#[derive(Debug, Clone, Default)]
pub struct CaptureCtx {
    /// Next pre-order element index to assign (incremented at each element's
    /// entry).
    pub counter: usize,
    /// The decisions captured so far.
    pub decisions: SyntaxDecisions,
}
