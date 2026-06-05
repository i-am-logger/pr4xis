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
//!
//! # rkyv-serializability under `prx`
//!
//! The graph-faithful WordNet `.prx` envelope carries the captured
//! [`WnSyntaxComplement`](super::super::lmf::writer::WnSyntaxComplement) — which
//! bundles [`DocumentResidue`] + [`RegeneratedComplement`] + [`SyntaxDecisions`]
//! — as the byte-exact reconstruction's second input. So every residue type in
//! this module must be rkyv-serializable under the `prx` feature where the
//! archive consumes it. The derive is CFG-GATED
//! (`#[cfg_attr(feature = "prx", derive(rkyv::Archive, …))]`) because rkyv is an
//! OPTIONAL dependency (`prx = ["dep:rkyv", …]`): present in the `prx`/`fetch`
//! build, absent from the default + wasm32 builds that do not link rkyv. The
//! `BTreeMap`-backed types (`SyntaxDecisions`, `ContentWhitespace`,
//! `AttributeOverrides`) serialize their private `by_index` map through rkyv's
//! `alloc` `BTreeMap` support directly — no Owned mirror is needed.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::super::ontology::{
    XmlAttribute, XmlDoctype, XmlDocument, XmlElement, XmlNamespace, XmlNode,
};

/// The document-level prolog/epilog `Misc*` run the Information Set does NOT
/// carry — the §2.8 production \[27\] `Misc ::= Comment | PI | S` runs the
/// Infoset discards because they sit OUTSIDE the document element (Cowan & Tobin
/// 2004 §2.1 keeps document-level *children* only for the root element, not the
/// surrounding `Misc*`). Captured so byte-exact reconstruction can re-emit them.
///
/// Each field holds the EXACT consumed `Misc*` substring VERBATIM — including any
/// §2.6 \[16\] processing instruction or §2.5 \[15\] comment, not only §2.3 \[3\]
/// `S` white-space — after the parser's §2.11 end-of-line normalization
/// `\r\n`/`\r` → `\n` (the only transform applied before the grammar descent).
/// Empty means "no `Misc*` here":
///
/// - `after_xml_decl` — `Misc*` consumed AFTER the XML declaration `<?xml …?>`
///   and BEFORE the DOCTYPE (or, when there is no DOCTYPE, before the root
///   element). This is where every USC USLM title's `<?xml-stylesheet …?>`
///   processing instruction (§2.6 \[16\] `PI`) lives — captured here in position.
/// - `after_doctype` — `Misc*` consumed AFTER the DOCTYPE `<!DOCTYPE …>` and
///   BEFORE the root element. Always empty when the document has no DOCTYPE.
/// - `after_root` — `Misc*` consumed AFTER the root element's end-tag (the §2.1
///   production \[1\] `document ::= prolog element Misc*` trailing `Misc*`).
///
/// # Additivity (no regression for prolog-PI-free documents)
///
/// For a pure-`S` `Misc*` — the WN-LMF case: XMLDecl, `S`, DOCTYPE, `S`, root,
/// `S` — the verbatim run IS its leading white-space, so a document carrying no
/// prolog/epilog PI or comment is byte-identical to the previous white-space-only
/// capture. The PI/comment bytes are added ONLY when the source actually carries
/// them (USC), so WordNet is unaffected.
///
/// # §2.11 end-of-line form
///
/// The captured run carries `#xA` line endings because §2.11 normalization runs
/// before the grammar descent. A source that wrote `#xD#xA` (CRLF) — as the
/// on-disk USC title does at its two prolog `?>` boundaries — appears here as
/// `#xA`. The original CR is recovered by the SEPARATE generic
/// [`EndOfLineForm`] residue (keyed by normalized-stream offset, document-wide),
/// re-expanded over the FINISHED serialized bytes, so a CRLF anywhere — prolog
/// `Misc*`, an attribute value, char-data — round-trips. That residue is empty
/// for a pure-`#xA` source (WordNet), so this run is unaffected for LF-only input.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct PrologDecisions {
    /// `Misc*` after the XML declaration, before the DOCTYPE-or-root (the
    /// `<?xml-stylesheet …?>` PI position for USC titles).
    pub after_xml_decl: String,
    /// `Misc*` after the DOCTYPE, before the root element (empty when no
    /// DOCTYPE).
    pub after_doctype: String,
    /// `Misc*` after the root element's end-tag (the epilog `Misc*`).
    pub after_root: String,
}

impl PrologDecisions {
    /// `true` when no prolog/epilog `Misc*` was captured — the canonical
    /// case, for which the byte-exact serializer emits nothing extra.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.after_xml_decl.is_empty()
            && self.after_doctype.is_empty()
            && self.after_root.is_empty()
    }
}

/// The W3C XML 1.0 Fifth Edition §2.11 \[2.11\] "End-of-Line Handling" FORM the
/// XML processor erases on input — the line-ending bytes the §2.11 normalization
/// collapsed to a single `#xA` before the grammar descent ever ran.
///
/// §2.11 mandates: *"the XML processor MUST behave as if it normalized all line
/// breaks … on input, before parsing, by translating both the two-character
/// sequence #xD #xA and any #xD that is not followed by #xA to a single #xA
/// character."* So `#xD#xA` (CRLF) and a lone `#xD` (CR) both vanish into `#xA`,
/// and every downstream production — and therefore every other concrete-syntax
/// residue ([`PrologDecisions`], [`IntraTagWhitespace`], the leaf `Text` runs) —
/// sees only `#xA`. This residue records, per collapsed line break, the original
/// FORM so the byte-exact serializer can put the `#xD` back.
///
/// # Keying: the LF ORDINAL (robust against re-escaping)
///
/// Recorded as `(lf_ordinal, EolKind)` pairs in ascending ordinal order.
/// `lf_ordinal` is the 0-based index of the produced `#xA` among ALL `#xA` bytes
/// in the §2.11-normalized stream (counting the line breaks the form did NOT
/// touch — a source literal `#xA` — too). The byte-exact serializer emits every
/// `#xA` LITERALLY and in order (char-data does not escape `#xA`; the prolog/epilog
/// `Misc*` and inter-element `S` runs are verbatim), so the k-th `#xA` byte in the
/// serialized output is the SAME line break as the k-th `#xA` in the normalized
/// stream — even though their BYTE OFFSETS differ (a §4.6 `&`/`<`/`>` escape, or
/// a `&amp;`-vs-`&` re-expansion, shifts byte offsets but never adds or removes an
/// `#xA`). Keying by ordinal rather than byte offset is therefore robust against
/// every other escaping decision, and FULLY GENERIC: a CRLF in the document body
/// re-expands identically to one in the prolog.
///
/// # Out of scope: an `#xA` INSIDE an attribute value
///
/// The one place the serializer does NOT emit `#xA` literally is an attribute
/// value — §3.3.3 attribute-value normalization escapes a literal `#xA` to
/// `&#xA;` (Boyer & Marcy 2008 C14N 1.1 §3.5). A line break written inside an
/// attribute value is thus a SEPARATE §3.3.3 residue this byte kernel does not
/// model — independent of the CRLF form, since even a pure-`#xA` literal newline
/// in an attribute already fails to round-trip. Such a break would shift the LF
/// ordinal, so it is out of this slice's scope; the real corpora (USC prolog
/// CRLFs, WordNet pure-LF) never write one.
///
/// # Additivity (a pure-`#xA` source records NOTHING)
///
/// A source with no `#xD` at all collapses no line break, so `eols` is empty and
/// [`Self::is_empty`] holds. The serializer's re-expansion pass is then a no-op
/// and the output is byte-identical to the pre-§2.11-form serializer — so a
/// pure-LF corpus (the Open English WordNet 2025 89 MB source, every
/// `reverse_lens` fixture) is wholly unaffected.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct EndOfLineForm {
    /// `(lf_ordinal, kind)` for every §2.11-collapsed line break, in ascending
    /// `lf_ordinal` order — the 0-based index of the produced `#xA` among all
    /// `#xA` bytes in the normalized stream, and whether the source wrote `#xD#xA`
    /// (CRLF) or a lone `#xD` (CR). Empty for a pure-`#xA` source.
    pub eols: Vec<(usize, EolKind)>,
}

impl EndOfLineForm {
    /// `true` when the source collapsed no line break — a pure-`#xA` document,
    /// for which the byte-exact serializer's re-expansion is a no-op.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.eols.is_empty()
    }
}

/// Which §2.11 \[2.11\] source form collapsed to the recorded `#xA` — a typed
/// enum (not a bare flag) so the byte-exact re-expansion dispatches on the closed
/// §2.11 set and an out-of-set form is unrepresentable. The third §2.11 case,
/// a literal `#xA` already, records no [`EndOfLineForm`] entry at all (there is
/// nothing to put back).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum EolKind {
    /// The source wrote the two-character sequence `#xD#xA` (CRLF) — the
    /// re-expansion inserts a `#xD` before the `#xA`.
    Crlf,
    /// The source wrote a lone `#xD` (CR not followed by `#xA`) — the
    /// re-expansion replaces the `#xA` with `#xD`.
    Cr,
}

/// The empty-element form decision (W3C XML 1.0 §3.1): `<a/>` (empty-element
/// tag) versus `<a></a>` (start- plus end-tag) — the same Information Set,
/// distinct bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum EmptyForm {
    /// `<a/>`.
    SelfClosing,
    /// `<a></a>`.
    Explicit,
}

/// One token of a start-tag's `(S Attribute)*` sequence in EXACT source order —
/// either a namespace declaration (`xmlns` / `xmlns:prefix`, Bray, Hollander,
/// Layman & Tobin 2009 §3) or a regular §3.1 \[41\] `Attribute`. Carries the
/// full token so the byte-exact serializer can re-emit the start-tag tokens in
/// the order the source wrote them, even when an `xmlns` declaration is
/// INTERLEAVED with non-`xmlns` attributes (the USC `<uscDoc>` root, which writes
/// `xsi:schemaLocation` / `xml:lang` / `identifier` BEFORE its `xmlns` decls).
///
/// XML attribute order is NOT an Information-Set item (Cowan & Tobin 2004 §2.3
/// defines the attributes as an unordered SET), so this is concrete-syntax
/// residue — captured in the [`SyntaxDecisions`] complement, never on the Infoset
/// [`XmlElement`] (which keeps the `namespaces` / `attributes` SETS, unordered
/// relative to each other). It is recorded ONLY when the source order is
/// non-canonical — i.e. an `xmlns` declaration does not strictly precede every
/// regular attribute — so a canonically-ordered tag (every element WordNet
/// writes, and most USC elements) records nothing and is byte-identical to the
/// previous ns-then-attr emit.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum StartTagToken {
    /// An `xmlns` / `xmlns:prefix` namespace declaration (§3).
    Namespace(XmlNamespace),
    /// A regular §3.1 \[41\] `Attribute`.
    Attribute(XmlAttribute),
}

/// The §2.3 \[3\] `S` (white-space) runs INSIDE a start-tag (`STag` / the
/// `EmptyElemTag` prefix) the Information Set discards — W3C XML 1.0 Fifth
/// Edition §3.1 productions \[40\] `STag ::= '<' Name (S Attribute)* S? '>'` and
/// \[44\] `EmptyElemTag ::= '<' Name (S Attribute)* S? '/>'`. The Infoset
/// carries the attribute *set* and the element name, but not the white-space
/// LAYOUT between them — the real Open English WordNet 2025 corpus indents each
/// attribute onto its own line, so the runs are non-trivial and must be
/// captured to round-trip byte-for-byte.
///
/// The runs are recorded in attribute-like SOURCE ORDER — one entry per
/// `(S Attribute)` group (every `xmlns`/`xmlns:prefix` declaration AND every
/// regular attribute), matching the order the byte-exact serializer re-emits
/// them. Production \[41\] `Attribute ::= Name Eq AttValue` with \[25\]
/// `Eq ::= S? '=' S?` contributes the two optional `S?` runs straddling the
/// `=`. Every field empty (the canonical single-space-separated single-line
/// tag) emits nothing extra, so canonical tags are unaffected.
///
/// # xmlns/attribute co-location
///
/// By default the serializer emits all `namespaces` (the `xmlns` decls) BEFORE
/// all regular `attributes`, and the per-slot runs key by that ns-then-attr emit
/// order. When the source INTERLEAVES the two — the USC `<uscDoc>` root writes
/// `xsi:schemaLocation` / `xml:lang` / `identifier` BEFORE its `xmlns` decls —
/// the element's [`NodeDecisions::start_tag_order`] carries the exact ordered
/// [`StartTagToken`] sequence, and the byte-exact serializer keys the per-slot
/// `before_attr` / `around_eq` runs by THAT order instead. So the runs always
/// land on the right items. A canonically-ordered tag records no
/// `start_tag_order` (the OEWN-2025 corpus never co-locates), and the ns-then-attr
/// default is used unchanged — the interleave handling is purely additive.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct IntraTagWhitespace {
    /// The §3.1 \[40\]/\[44\] `S` run BEFORE each `(S Attribute)` group, one
    /// entry per attribute-like item in source order (the leading `S` of the
    /// `(S Attribute)*` repetition; never empty in well-formed input since a `S`
    /// is mandatory between the name/previous-attribute and the next attribute).
    pub before_attr: Vec<String>,
    /// The §3.1 \[25\] `Eq ::= S? '=' S?` white-space straddling each
    /// attribute's `=` — `(name->eq, eq->value)` — one entry per attribute-like
    /// item, in the same source order as [`Self::before_attr`]. Empty strings
    /// for the canonical `name="value"` (no space around `=`).
    pub around_eq: Vec<(String, String)>,
    /// The §3.1 \[40\]/\[44\] trailing `S?` BEFORE the closing `>` or `/>`.
    /// Empty for a tag whose last token abuts the close.
    pub before_close: String,
}

impl IntraTagWhitespace {
    /// `true` when the captured layout is exactly the CANONICAL one the
    /// byte-exact serializer emits with NO decision recorded: a single `#x20`
    /// before each attribute (the leading `S` of `(S Attribute)`), no `S` around
    /// any `=` (`Eq ::= S? '=' S?` with both `S?` empty), and no trailing `S?`
    /// before the close. A canonical single-line tag therefore records no
    /// `IntraTagWhitespace` decision (the writer's default reproduces it); only a
    /// departure — the real corpus's multi-line attribute indentation — is
    /// recorded.
    #[must_use]
    pub fn is_canonical(&self) -> bool {
        self.before_attr.iter().all(|s| s == " ")
            && self
                .around_eq
                .iter()
                .all(|(a, b)| a.is_empty() && b.is_empty())
            && self.before_close.is_empty()
    }
}

/// Which character of a RESOLVED string was written in the source as a §4.6
/// predefined entity reference (`&amp; &lt; &gt; &apos; &quot;`) rather than as
/// the literal character — W3C XML 1.0 Fifth Edition §4.6 "Predefined Entities".
///
/// The parser RESOLVES every predefined reference to its single literal
/// character (§4.4.5 "Included in Literal" for attribute values, §4.4.2
/// "Included" for content), so the Infoset DOM holds the bare char and the
/// canonical serializer re-escapes only the C14N-required minimum (`& < >` in
/// char-data; `& < "` in attribute values). A source `&apos;` becomes a literal
/// `'` in the DOM and would re-emit as a bare `'` — a byte mismatch. This
/// records the entity NAME so the byte-exact escaper re-emits the reference at
/// the exact resolved-string CHAR INDEX it occupied.
///
/// Keyed by char index into the RESOLVED string because the serializer iterates
/// the value with `str::chars()`; a source byte offset would be wrong against
/// the resolved char stream (references collapse 4-6 source bytes to one char,
/// and adjacent multibyte chars — curly quotes U+2018/U+2019 in real
/// definitions — shift byte offsets but not char indices).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct EntityReferenceForm {
    /// `(char_index, entity_name)` pairs in ascending `char_index` order — the
    /// resolved-string position and the §4.6 predefined entity name (`amp`,
    /// `lt`, `gt`, `apos`, `quot`) the source wrote there. Multiple entries per
    /// string are allowed (e.g. `Dhu&apos;l-Qa&apos;dah` records two `apos`).
    pub refs: Vec<(usize, EntityName)>,
}

/// One of the five W3C XML 1.0 §4.6 predefined entity names. A typed enum (not a
/// bare `String`) so the escaper dispatches on the closed §4.6 set rather than
/// re-parsing a name, and an out-of-set value is unrepresentable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum EntityName {
    /// `&amp;` → `&`.
    Amp,
    /// `&lt;` → `<`.
    Lt,
    /// `&gt;` → `>`.
    Gt,
    /// `&apos;` → `'`.
    Apos,
    /// `&quot;` → `"`.
    Quot,
}

impl EntityName {
    /// The literal character this §4.6 predefined entity resolves to.
    #[must_use]
    pub fn resolved_char(self) -> char {
        match self {
            Self::Amp => '&',
            Self::Lt => '<',
            Self::Gt => '>',
            Self::Apos => '\'',
            Self::Quot => '"',
        }
    }

    /// The reference text `&name;` the source wrote for this entity.
    #[must_use]
    pub fn reference(self) -> &'static str {
        match self {
            Self::Amp => "&amp;",
            Self::Lt => "&lt;",
            Self::Gt => "&gt;",
            Self::Apos => "&apos;",
            Self::Quot => "&quot;",
        }
    }

    /// The §4.6 predefined entity whose resolved character is `ch`, if any. The
    /// inverse of [`Self::resolved_char`]; `None` for a char no predefined
    /// entity resolves to.
    #[must_use]
    pub fn for_resolved_char(ch: char) -> Option<Self> {
        match ch {
            '&' => Some(Self::Amp),
            '<' => Some(Self::Lt),
            '>' => Some(Self::Gt),
            '\'' => Some(Self::Apos),
            '"' => Some(Self::Quot),
            _ => None,
        }
    }
}

/// The concrete-syntax decisions for ONE node — the SourceSyntax residue the
/// Information Set DOM does not carry, needed to reproduce its exact bytes.
///
/// Three kinds of residue, each a separate concrete-syntax decision:
/// - the empty-element form (§3.1 `<a/>` vs `<a></a>`);
/// - the intra-tag white-space layout (§3.1 \[40\]/\[44\] `S` runs inside the
///   start-tag — the multi-line attribute indentation of the real corpus);
/// - the predefined-entity-reference form (§4.6) the source used in attribute
///   values and char-data, recorded so the escaper re-emits the reference
///   instead of the resolved literal character.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct NodeDecisions {
    /// The empty-element form — only meaningful for a childless element.
    pub empty_form: Option<EmptyForm>,
    /// The §3.1 \[40\]/\[44\] start-tag white-space layout (multi-line attribute
    /// indentation). `None` for a canonical single-line tag.
    pub intra_tag_whitespace: Option<IntraTagWhitespace>,
    /// The §4.6 predefined-entity-reference form used in this element's
    /// attribute VALUES, keyed by attribute index (the position in the element's
    /// emitted attribute-like sequence — `namespaces` then `attributes`, the
    /// serializer's write order). Absent key = that value held no resolved
    /// reference.
    pub attr_entity_refs: BTreeMap<usize, EntityReferenceForm>,
    /// The §4.6 predefined-entity-reference form used in this element's CHAR
    /// DATA, keyed by the ordinal of the `Text` child among the element's
    /// children (0-based child position). Absent key = that text node held no
    /// resolved reference.
    pub text_entity_refs: BTreeMap<usize, EntityReferenceForm>,
    /// The EXACT ordered start-tag token sequence ([`StartTagToken`]) — recorded
    /// ONLY when the source INTERLEAVES an `xmlns` declaration with non-`xmlns`
    /// attributes (the USC `<uscDoc>` root). When present, the byte-exact
    /// serializer emits these tokens in this order, and keys the per-slot
    /// intra-tag white-space / attribute-value entity-reference runs by it, rather
    /// than by the default `namespaces`-then-`attributes` emit order. `None` for a
    /// canonically-ordered start-tag (every WordNet element, most USC elements),
    /// so the default emit is unchanged — the field is purely additive.
    pub start_tag_order: Option<Vec<StartTagToken>>,
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
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct SyntaxDecisions {
    by_index: BTreeMap<usize, NodeDecisions>,
    /// Document-level prolog/epilog white-space (§2.8 \[27\] `Misc` `S`) — the
    /// residue outside the document element, separate from the per-element
    /// `by_index` decisions. Default (all-empty) for a canonical document.
    prolog: PrologDecisions,
    /// Document-wide §2.11 \[2.11\] end-of-line form (which collapsed `#xA`s came
    /// from a source `#xD#xA`/`#xD`) — keyed by normalized-stream offset, NOT by
    /// element, because a line break is concrete-syntax that falls in any
    /// production. Default (empty) for a pure-`#xA` document.
    eol_form: EndOfLineForm,
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

    /// Record the document-wide §2.11 \[2.11\] end-of-line form (the collapsed
    /// `#xD#xA`/`#xD` source line breaks the byte-exact serializer re-expands).
    pub fn set_eol_form(&mut self, eol_form: EndOfLineForm) {
        self.eol_form = eol_form;
    }

    /// The document-wide §2.11 \[2.11\] end-of-line form (empty for a pure-`#xA`
    /// source).
    #[must_use]
    pub fn eol_form(&self) -> &EndOfLineForm {
        &self.eol_form
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

    /// Record the document-wide §2.11 \[2.11\] end-of-line form the §2.11
    /// normalization erased before the grammar descent.
    pub fn record_eol_form(&mut self, eol_form: EndOfLineForm) {
        self.decisions.set_eol_form(eol_form);
    }
}

// ── The regenerated-DOM complement — residue a STRUCTURAL writer's tree lacks ──
//
// `serialize_document_exact` reproduces the source bytes from the EXACT Infoset
// DOM (the one `parse_document_capturing` builds) plus the [`SyntaxDecisions`]
// above. A structural writer (e.g. WordNet's `write_wordnet_document`) instead
// regenerates a FRESH element tree from a typed model. That regenerated tree is
// equal to the captured DOM on the element backbone — same elements, in the same
// pre-order, with the same names/attributes/leaf-text — but it LACKS three
// classes of byte-affecting residue the Information Set never required the typed
// model to carry:
//
//   1. the document `<!DOCTYPE …>` (§2.8 \[28\] `doctypedecl`);
//   2. the root element's namespace declarations (Bray, Hollander, Layman &
//      Tobin 2009 §3 — `xmlns` / `xmlns:prefix`);
//   3. the §2.4 \[14\] `CharData` runs of pure white-space that sit BETWEEN
//      element children (the indentation the §2.10 "White Space Handling"
//      note calls insignificant) — present in the captured DOM as
//      [`XmlNode::Text`] children, absent from a structural writer's tree.
//
// [`DocumentResidue`] carries (1) + (2); [`ContentWhitespace`] carries (3). Both
// are GENERIC XML-family residue (no format vocabulary), keyed — like the
// per-element [`SyntaxDecisions`] — by the robust pre-order ELEMENT index, which
// a structural writer reproduces identically because inter-element white-space
// occupies no element slot. [`reapply_regenerated_complement`] merges the residue
// back into a regenerated tree so [`serialize_document_exact`] can close the
// byte-exact loop.

/// The document/root-level residue a structural writer's regenerated
/// [`XmlDocument`] does not carry: the `<!DOCTYPE>` (§2.8 \[28\]) and the root
/// element's namespace declarations (Bray, Hollander, Layman & Tobin 2009 §3).
///
/// Both are Information-Set items the typed model is free to drop — a typed
/// lexicon/vocabulary models the document's CONTENT, not its DTD binding or its
/// namespace prefixes — so a regenerated tree emits neither. Captured once
/// (document-level) and re-applied by [`reapply_regenerated_complement`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct DocumentResidue {
    /// The `<!DOCTYPE …>` declaration the source carried (§2.8 \[28\]
    /// `doctypedecl`). `None` when the source had no DOCTYPE.
    pub doctype: Option<XmlDoctype>,
    /// The root element's namespace declarations in document order (Bray,
    /// Hollander, Layman & Tobin 2009 §3). Empty when the root declares none.
    pub root_namespaces: Vec<XmlNamespace>,
}

impl DocumentResidue {
    /// `true` when there is no document/root residue — a regenerated tree
    /// already matches the source at the document and root level.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.doctype.is_none() && self.root_namespaces.is_empty()
    }
}

/// One slot in an element's reconstructed child sequence — the plan that splices
/// a structural writer's regenerated children back together with the source's
/// inter-element white-space.
///
/// `Keep` consumes the next child from the regenerated element in order (an
/// element child, or the single leaf `#PCDATA` text node the typed model DID
/// carry); `InsertText` injects a §2.4 \[14\] `CharData` run the regenerated
/// tree lacked (the inter-element white-space). Replaying the slots yields the
/// captured DOM's exact child sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum ChildSlot {
    /// Take the next child from the regenerated element, unchanged.
    Keep,
    /// Insert this literal text run (the source's inter-element white-space)
    /// the regenerated tree did not produce.
    InsertText(String),
}

/// The inter-element white-space (§2.4 \[14\] `CharData` runs between element
/// children) a structural writer's regenerated tree lacks, keyed by the parent
/// element's pre-order index. Each value is the [`ChildSlot`] template that
/// re-threads the regenerated element's own children with the source's text
/// runs.
///
/// Only elements whose source child sequence DIFFERS from the regenerated one
/// (i.e. carried inter-element white-space) get an entry; a leaf element whose
/// children the typed model reproduced exactly is absent (its reconstruction is
/// the identity). This is the §2.10 insignificant-white-space residue, captured
/// verbatim so reconstruction is byte-exact without a per-depth indentation
/// model.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct ContentWhitespace {
    by_index: BTreeMap<usize, Vec<ChildSlot>>,
}

impl ContentWhitespace {
    /// `true` when no element carried inter-element white-space.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_index.is_empty()
    }

    /// The number of elements that carried inter-element white-space.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_index.len()
    }
}

/// The exact source start-tag attribute sequence (namespace declarations then
/// regular attributes, IN SOURCE ORDER) for elements whose attributes a
/// structural writer's regenerated tree did NOT reproduce identically — keyed by
/// the parent element's pre-order index.
///
/// XML attribute *order* is NOT an Information-Set item — Cowan & Tobin 2004 §2.3
/// defines an element's attributes as an unordered SET — so a structural writer is
/// free to (and does) emit them in a different order, or to omit a §2.3 \[41\]
/// `Attribute` whose value the typed model does not carry (a `dc:type` role tag,
/// an `Example`'s `dc:source` provenance). Both are concrete-syntax residue: the
/// SERIALIZATION of the attribute set, not the set's meaning. Captured verbatim —
/// the exact `(namespaces, attributes)` the source wrote — so reconstruction
/// re-emits the source's bytes without the typed model having to carry every
/// metadata attribute (the ontology stays clean).
///
/// The captured sequence is the same attribute-like SOURCE ORDER the per-element
/// [`SyntaxDecisions`] keyed its intra-tag-white-space and §4.6 entity-reference
/// slots by, so overwriting the regenerated element's attributes with this exact
/// sequence keeps those decisions slot-aligned with the byte-exact serializer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct AttributeOverrides {
    by_index: BTreeMap<usize, ElementAttributes>,
}

/// One element's exact source attribute sequence — its namespace declarations and
/// regular attributes in source order, the pair the byte-exact serializer emits.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct ElementAttributes {
    /// The element's `xmlns` / `xmlns:prefix` declarations in source order (Bray,
    /// Hollander, Layman & Tobin 2009 §3).
    pub namespaces: Vec<XmlNamespace>,
    /// The element's §2.3 \[41\] `Attribute`s in source order.
    pub attributes: Vec<XmlAttribute>,
}

impl AttributeOverrides {
    /// `true` when every element's attributes were reproduced identically by the
    /// structural writer (no override needed).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_index.is_empty()
    }

    /// The number of elements whose attribute sequence had to be overridden.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_index.len()
    }

    /// Mutable iterator over each overridden element's exact source attribute
    /// sequence. Used by the byte-exact corruption meta-tests to mutate a
    /// captured attribute (e.g. a blank-node `rdf:nodeID`) and prove the
    /// reconstruction diverges — the gate's teeth.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut ElementAttributes> {
        self.by_index.values_mut()
    }
}

/// The whole regenerated-tree complement: the inter-element white-space and the
/// exact source attribute sequences a structural writer's tree does not reproduce.
/// Returned by [`diff_content_whitespace`] and consumed by
/// [`reapply_regenerated_complement`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct RegeneratedComplement {
    /// The §2.4 \[14\] inter-element white-space the regenerated tree lacks.
    pub content_whitespace: ContentWhitespace,
    /// The exact source attribute sequences the regenerated tree reordered or
    /// under-populated.
    pub attribute_overrides: AttributeOverrides,
}

/// Failure when diffing a structural writer's regenerated tree against the
/// captured source DOM, or when re-applying the residue. A mismatch means the
/// regenerated tree is NOT element-backbone-equal to the source — the structural
/// writer dropped, reordered, or altered an element/attribute/leaf-text the
/// source carried — so the residue cannot be a pure white-space/decl complement.
/// Surfaced (never papered over) so the byte-exact gate fails LOUD at the exact
/// divergence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegeneratedComplementError {
    /// At pre-order element `index`, the regenerated element's name differs from
    /// the source's — the backbones diverge here.
    ElementMismatch {
        /// Pre-order element index where the divergence was found.
        index: usize,
        /// The source element's qualified name.
        source: String,
        /// The regenerated element's qualified name.
        regenerated: String,
    },
    /// At pre-order element `index`, a [`ChildSlot::Keep`] had no regenerated
    /// child to consume (the regenerated element has FEWER element/leaf-text
    /// children than the source's non-white-space children).
    KeepUnderflow {
        /// Pre-order element index where the underflow occurred.
        index: usize,
    },
    /// After replaying an element's [`ChildSlot`] template, the regenerated
    /// element had children left over (MORE than the source's non-white-space
    /// children) — the structural writer emitted content the source lacked.
    KeepOverflow {
        /// Pre-order element index where the overflow occurred.
        index: usize,
        /// How many regenerated children were left unconsumed.
        remaining: usize,
    },
    /// The captured [`ContentWhitespace`] references a pre-order index the
    /// regenerated walk never reached — the trees have different element counts.
    DanglingIndex {
        /// The pre-order index present in the residue but absent from the tree.
        index: usize,
    },
    /// At pre-order element `index`, a NON-white-space source `Text` run had no
    /// regenerated counterpart — the structural writer dropped genuine character
    /// data. Re-inserting it as white-space residue would let real `#PCDATA`
    /// masquerade as concrete-syntax white-space (XML 1.0 §2.3 \[3\] `S` is only
    /// `#x20 | #x9 | #xD | #xA`, distinct from character data), so the diff fails
    /// closed instead.
    UnmatchedContentText {
        /// Pre-order element index where the dropped character data was found.
        index: usize,
    },
}

impl core::fmt::Display for RegeneratedComplementError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ElementMismatch {
                index,
                source,
                regenerated,
            } => write!(
                f,
                "regenerated tree diverges from source at pre-order element {index}: \
                 source <{source}> vs regenerated <{regenerated}>"
            ),
            Self::KeepUnderflow { index } => write!(
                f,
                "regenerated element at pre-order index {index} has fewer children than \
                 the source (a structural writer dropped content)"
            ),
            Self::KeepOverflow { index, remaining } => write!(
                f,
                "regenerated element at pre-order index {index} has {remaining} extra \
                 child(ren) the source lacked (a structural writer added content)"
            ),
            Self::DanglingIndex { index } => write!(
                f,
                "content-white-space residue references pre-order element {index} that \
                 the regenerated tree never reached (different element counts)"
            ),
            Self::UnmatchedContentText { index } => write!(
                f,
                "non-white-space source text at pre-order element {index} had no \
                 regenerated counterpart (a structural writer dropped #PCDATA content)"
            ),
        }
    }
}

/// Diff a structural writer's regenerated tree against the captured source DOM,
/// extracting the [`RegeneratedComplement`] residue: the inter-element
/// white-space the regenerated tree lacks AND the exact source attribute sequence
/// for every element the writer reordered or under-populated. Walks BOTH trees in
/// lockstep pre-order; at each element it (a) records an attribute override when
/// the source and regenerated attribute sequences are not byte-identical, then
/// (b) threads the source element's children against the regenerated element's
/// children, emitting a [`ChildSlot::Keep`] where the two agree on a
/// non-white-space child and a [`ChildSlot::InsertText`] for each source-only
/// `Text` run.
///
/// A non-white-space divergence (different element name, a dropped/added/altered
/// ELEMENT or leaf-text child) is a [`RegeneratedComplementError`] — the element
/// backbones are not equal, so the residue is not a pure white-space/attribute
/// complement. (Attribute order/coverage divergence is NOT an error; it is
/// captured as an override, since attributes are an unordered Infoset set.) The
/// caller fails closed.
///
/// Generic over the XML family: no format vocabulary. The whole-document leg —
/// the DOCTYPE and root namespaces — is the caller's [`DocumentResidue`]; this
/// function diffs the element-content white-space and the start-tag attributes.
pub fn diff_content_whitespace(
    source: &XmlDocument,
    regenerated: &XmlDocument,
) -> Result<RegeneratedComplement, RegeneratedComplementError> {
    let mut content = ContentWhitespace::default();
    let mut attrs = AttributeOverrides::default();
    let mut counter = 0usize;
    diff_element(
        &source.root,
        &regenerated.root,
        &mut counter,
        &mut content.by_index,
        &mut attrs.by_index,
    )?;
    Ok(RegeneratedComplement {
        content_whitespace: content,
        attribute_overrides: attrs,
    })
}

/// Recursive worker for [`diff_content_whitespace`]: diff one element pair,
/// claiming the element's pre-order index AT ENTRY (mirroring `parse_element` and
/// the byte-exact serializer), then descend into matched element children.
fn diff_element(
    source: &XmlElement,
    regenerated: &XmlElement,
    counter: &mut usize,
    ws_by_index: &mut BTreeMap<usize, Vec<ChildSlot>>,
    attr_by_index: &mut BTreeMap<usize, ElementAttributes>,
) -> Result<(), RegeneratedComplementError> {
    let my_index = *counter;
    *counter += 1;

    if source.name != regenerated.name {
        return Err(RegeneratedComplementError::ElementMismatch {
            index: my_index,
            source: source.name.qualified(),
            regenerated: regenerated.name.qualified(),
        });
    }

    // Attribute residue: when the source's attribute sequence (namespaces then
    // attributes, in source order) is not byte-identical to the regenerated one,
    // record the EXACT source sequence as an override. Attribute order/coverage is
    // not an Infoset item (Cowan & Tobin 2004 §2.3), so a structural writer's
    // re-ordering or omission of a metadata attribute is concrete-syntax residue,
    // not a backbone divergence — it is captured, never an error.
    if source.namespaces != regenerated.namespaces || source.attributes != regenerated.attributes {
        attr_by_index.insert(
            my_index,
            ElementAttributes {
                namespaces: source.namespaces.clone(),
                attributes: source.attributes.clone(),
            },
        );
    }

    // Thread the source element's children against the regenerated element's
    // children. A source `Text` node with no regenerated counterpart is
    // inter-element white-space → `InsertText`; every other source child must
    // line up positionally with a regenerated child → `Keep`. White-space-only
    // `Text` is the residue; a non-white-space source `Text` (a leaf `#PCDATA`
    // value the typed model carried) is matched as a `Keep` against the
    // regenerated leaf text.
    let mut slots: Vec<ChildSlot> = Vec::new();
    let mut reg_iter = regenerated.children.iter().peekable();
    let mut carried_ws = false;

    for src_child in &source.children {
        match src_child {
            XmlNode::Text(t) if is_regenerated_text_residue(reg_iter.peek()) => {
                // No regenerated child lines up here (or the next regenerated
                // child is an element, not this text) — this source text run is
                // residue the structural writer dropped. It is inter-element
                // white-space residue ONLY when it is white-space-only (XML 1.0
                // §2.3 [3] S = #x20 | #x9 | #xD | #xA); a non-white-space run here
                // is genuine #PCDATA the typed writer dropped, so fail closed
                // rather than re-insert real content as concrete-syntax residue.
                if !t.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')) {
                    return Err(RegeneratedComplementError::UnmatchedContentText {
                        index: my_index,
                    });
                }
                slots.push(ChildSlot::InsertText(t.clone()));
                carried_ws = true;
            }
            _ => {
                // Consume the matching regenerated child. Element children
                // recurse; leaf text / other nodes are kept as-is (their bytes
                // come from the regenerated tree, which the typed model produced).
                let Some(reg_child) = reg_iter.next() else {
                    return Err(RegeneratedComplementError::KeepUnderflow { index: my_index });
                };
                if let (XmlNode::Element(se), XmlNode::Element(re)) = (src_child, reg_child) {
                    diff_element(se, re, counter, ws_by_index, attr_by_index)?;
                }
                slots.push(ChildSlot::Keep);
            }
        }
    }

    // Any regenerated children left unconsumed = the writer emitted more than the
    // source carried.
    let remaining = reg_iter.count();
    if remaining > 0 {
        return Err(RegeneratedComplementError::KeepOverflow {
            index: my_index,
            remaining,
        });
    }

    // Record the template only when it actually re-introduces white-space; an
    // all-`Keep` template is the identity reconstruction and is dropped.
    if carried_ws {
        ws_by_index.insert(my_index, slots);
    }
    Ok(())
}

/// Whether the next regenerated child (if any) means the current source `Text`
/// node is white-space residue: the regenerated tree has NO text node here — its
/// next pending child is an element (or it is exhausted). A regenerated `Text`
/// node lined up at this position is a leaf `#PCDATA` value the typed model
/// carried, so the source `Text` is NOT residue and falls through to `Keep`.
fn is_regenerated_text_residue(next_regenerated: Option<&&XmlNode>) -> bool {
    !matches!(next_regenerated, Some(XmlNode::Text(_)))
}

/// Re-apply the [`DocumentResidue`] and [`RegeneratedComplement`] to a structural
/// writer's regenerated [`XmlDocument`], reconstructing the captured source DOM
/// so [`serialize_document_exact`](super::serializer::serialize_document_exact)
/// can close the byte-exact loop:
///
/// 1. set the document `<!DOCTYPE>` and root namespace declarations
///    ([`DocumentResidue`]);
/// 2. for every element with an override, replace its start-tag attribute
///    sequence with the exact source one ([`AttributeOverrides`]);
/// 3. splice the inter-element white-space `Text` children back into every
///    element at its captured pre-order index ([`ContentWhitespace`]).
///
/// Fails closed on any structural divergence ([`RegeneratedComplementError`]) —
/// it never fabricates or drops content to force a fit.
pub fn reapply_regenerated_complement(
    regenerated: &mut XmlDocument,
    document_residue: &DocumentResidue,
    complement: &RegeneratedComplement,
) -> Result<(), RegeneratedComplementError> {
    // (1) document/root-level residue.
    regenerated.doctype = document_residue.doctype.clone();
    regenerated.root.namespaces = document_residue.root_namespaces.clone();
    // Keep the legacy single-slot `namespace` representative consistent with the
    // restored declarations (it mirrors the first declaration, as the parser and
    // canonical writer do).
    regenerated.root.namespace = document_residue.root_namespaces.first().cloned();

    // (2)+(3) per-element attribute overrides + inter-element white-space, in one
    // pre-order walk that claims indices exactly as the diff and serializer do.
    let ws = &complement.content_whitespace.by_index;
    let attrs = &complement.attribute_overrides.by_index;
    let mut counter = 0usize;
    let mut ws_applied = 0usize;
    let mut attr_applied = 0usize;
    apply_element(
        &mut regenerated.root,
        &mut counter,
        ws,
        attrs,
        &mut ws_applied,
        &mut attr_applied,
    )?;
    // A residue index the walk never reached means the trees disagree on element
    // count. `counter` holds the total reached; the first residue index `>=
    // counter` (ascending `BTreeMap` keys) is the dangling one.
    if ws_applied != ws.len()
        && let Some((&index, _)) = ws.range(counter..).next()
    {
        return Err(RegeneratedComplementError::DanglingIndex { index });
    }
    if attr_applied != attrs.len()
        && let Some((&index, _)) = attrs.range(counter..).next()
    {
        return Err(RegeneratedComplementError::DanglingIndex { index });
    }
    Ok(())
}

/// Recursive worker for [`reapply_regenerated_complement`]: at one element apply
/// its attribute override (if any) and splice its inter-element white-space (if
/// any), claiming its pre-order index AT ENTRY (mirroring the diff and the
/// serializer), then descend into element children.
fn apply_element(
    element: &mut XmlElement,
    counter: &mut usize,
    ws_by_index: &BTreeMap<usize, Vec<ChildSlot>>,
    attr_by_index: &BTreeMap<usize, ElementAttributes>,
    ws_applied: &mut usize,
    attr_applied: &mut usize,
) -> Result<(), RegeneratedComplementError> {
    let my_index = *counter;
    *counter += 1;

    // Attribute override: replace the regenerated element's start-tag attribute
    // sequence with the exact source one. The byte-exact serializer emits
    // `namespaces` then `attributes` in vec order, so this restores both the
    // source ORDER and any metadata attribute the typed model did not carry.
    if let Some(over) = attr_by_index.get(&my_index) {
        *attr_applied += 1;
        element.namespaces = over.namespaces.clone();
        element.namespace = over.namespaces.first().cloned();
        element.attributes = over.attributes.clone();
    }

    if let Some(slots) = ws_by_index.get(&my_index) {
        *ws_applied += 1;
        let original = core::mem::take(&mut element.children);
        let mut kept = original.into_iter();
        let mut rebuilt: Vec<XmlNode> = Vec::with_capacity(slots.len());
        for slot in slots {
            match slot {
                ChildSlot::Keep => {
                    let Some(child) = kept.next() else {
                        return Err(RegeneratedComplementError::KeepUnderflow { index: my_index });
                    };
                    rebuilt.push(child);
                }
                ChildSlot::InsertText(t) => rebuilt.push(XmlNode::Text(t.clone())),
            }
        }
        let remaining = kept.count();
        if remaining > 0 {
            return Err(RegeneratedComplementError::KeepOverflow {
                index: my_index,
                remaining,
            });
        }
        element.children = rebuilt;
    }

    // Descend into element children in document order — claiming pre-order
    // indices for them exactly as the diff did.
    for child in &mut element.children {
        if let XmlNode::Element(el) = child {
            apply_element(
                el,
                counter,
                ws_by_index,
                attr_by_index,
                ws_applied,
                attr_applied,
            )?;
        }
    }
    Ok(())
}
