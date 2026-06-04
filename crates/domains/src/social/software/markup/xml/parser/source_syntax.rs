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
use alloc::string::String;

/// The document-level prolog/epilog white-space the Information Set does NOT
/// carry — the §2.8 production \[27\] `Misc` `S` runs the Infoset discards
/// because they sit OUTSIDE the document element (Cowan & Tobin 2004 §2.1 keeps
/// document-level *children* only for the root element, not the surrounding
/// white-space). Captured so byte-exact reconstruction can re-emit them.
///
/// Each field holds the EXACT consumed substring (after the parser's §2.11
/// end-of-line normalization `\r\n`/`\r` → `\n`, which is the only transform
/// applied before the grammar descent). Empty means "no white-space here":
///
/// - `after_xml_decl` — `S` consumed AFTER the XML declaration `<?xml …?>` and
///   BEFORE the DOCTYPE (or, when there is no DOCTYPE, before the root element).
/// - `after_doctype` — `S` consumed AFTER the DOCTYPE `<!DOCTYPE …>` and BEFORE
///   the root element. Always empty when the document has no DOCTYPE.
/// - `after_root` — `S` consumed AFTER the root element's end-tag (the §2.1
///   production \[1\] `document ::= prolog element Misc*` trailing `Misc*`).
///
/// # Limitation (prolog/epilog Comment / PI)
///
/// A `Misc` item may also be a Comment or PI (§2.8 \[27\]
/// `Misc ::= Comment | PI | S`). Those are DROPPED from the Infoset at the
/// document level by the existing `parse_misc_star` handling, so they are not
/// byte-exactly reconstructable regardless. These fields therefore capture ONLY
/// the white-space run found at each position when it is pure `S`; if a Comment
/// or PI interrupts the run the capture covers the white-space up to that item,
/// and the dropped item makes a full byte-exact round-trip impossible (a
/// separate, later concrete-syntax slice). The realistic WN-LMF case is pure
/// white-space between the XML declaration, the DOCTYPE, and the root element.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrologDecisions {
    /// `S` after the XML declaration, before the DOCTYPE-or-root.
    pub after_xml_decl: String,
    /// `S` after the DOCTYPE, before the root element (empty when no DOCTYPE).
    pub after_doctype: String,
    /// `S` after the root element's end-tag (the epilog `Misc*`).
    pub after_root: String,
}

impl PrologDecisions {
    /// `true` when no prolog/epilog white-space was captured — the canonical
    /// case, for which the byte-exact serializer emits nothing extra.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.after_xml_decl.is_empty()
            && self.after_doctype.is_empty()
            && self.after_root.is_empty()
    }
}

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
    /// Document-level prolog/epilog white-space (§2.8 \[27\] `Misc` `S`) — the
    /// residue outside the document element, separate from the per-element
    /// `by_index` decisions. Default (all-empty) for a canonical document.
    prolog: PrologDecisions,
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

    /// Record the document-level prolog/epilog white-space.
    pub fn set_prolog(&mut self, prolog: PrologDecisions) {
        self.prolog = prolog;
    }

    /// The document-level prolog/epilog white-space (§2.8 \[27\] `Misc` `S`).
    #[must_use]
    pub fn prolog(&self) -> &PrologDecisions {
        &self.prolog
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

impl CaptureCtx {
    /// Record the document-level prolog/epilog white-space (§2.8 \[27\] `Misc`
    /// `S`) the reader consumed outside the document element.
    pub fn record_prolog(&mut self, prolog: PrologDecisions) {
        self.decisions.set_prolog(prolog);
    }
}
