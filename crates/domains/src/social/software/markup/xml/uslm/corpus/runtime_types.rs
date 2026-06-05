//! USLM runtime aggregate types — the domain-projected typed
//! structures populated by the lens walker from parsed USLM XML.
//!
//! These are not the conceptual USLM ontology — that lives via XSD
//! ontology projection at [`crate::formal::meta::xsd`]. They are the
//! runtime aggregates: `<notes>` + child `<note>` elements gathered
//! into a [`UsCodeNotesBlock`]; structured fields pulled from a
//! `<sourceCredit>` into [`UsCodeSourceCredit`]; etc.
//!
//! Pending the upstream xsd-parser-types serde-derives PR, these
//! Rust structs hold the parsed data at runtime. When that lands,
//! field-level types will delegate to xsd-parser-generated output;
//! the corpus-level [`super::UsCode`] aggregate will remain.

#[allow(unused_imports)]
use alloc::{boxed::Box, string::String, vec::Vec};

use super::kinds::{
    ContainerKind, InlineKind, SubdivisionKind, UsCodeAmendmentKind, UsCodeNoteKind,
    UsCodeTableCellKind,
};

/// A whole U.S. Code title (e.g. Title 18, Title 49) parsed from
/// USLM XML.
///
/// USLM publishes one of these per LRC release point. The
/// title is exposed two ways:
///
/// 1. **Flat `sections` list** — every `<section>` in document
///    order. O(N) iteration, O(N) lookup by identifier.
/// 2. **Nested `hierarchy` tree** — the published structure with
///    its intermediate containers (Subtitle, Part, Subpart,
///    Chapter, Subchapter). Each leaf is a Section; each interior
///    node is a [`UsCodeContainer`]. Required for navigation
///    queries like "what chapter contains § 1514A" or "list every
///    section in Part I of Title 18".
///
/// Different USC titles use different hierarchy depths — Title 18
/// is Title > Part > Chapter > Section, Title 49 is Title >
/// Subtitle > Part > Chapter > Subchapter > Section. The
/// [`HierarchyNode`] enum keeps the model uniform across titles.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeTitle {
    /// USLM identifier for the title, e.g. `/us/usc/t18`.
    pub identifier: String,
    /// Title number (parsed from `<num value="18">`).
    pub number: u32,
    /// `<heading>` text, e.g. "CRIMES AND CRIMINAL PROCEDURE".
    pub heading: String,
    /// Every `<section>` element in the title, flat document order.
    pub sections: Vec<UsCodeSection>,
    /// The published nested hierarchy — Subtitle/Part/Subpart/
    /// Chapter/Subchapter containers with Sections as leaves.
    /// Walking this tree DFS yields exactly `sections` in order.
    pub hierarchy: Vec<HierarchyNode>,
    /// Title-level editorial notes blocks (`<notes>` wrappers
    /// directly under `<title>`).
    pub notes_blocks: Vec<UsCodeNotesBlock>,
    /// Bare `<note>` elements directly under `<title>` (not
    /// inside a `<notes>` wrapper). USLM allows both forms.
    pub bare_notes: Vec<UsCodeNote>,
    /// Title-level `<header>` elements (typically zero or one;
    /// some titles carry one with the authoritative title heading).
    pub headers: Vec<UsCodeHeader>,
    /// Title-level `<signature>` blocks. Rare in codified USC;
    /// more common in bills / amendments / public laws.
    pub signatures: Vec<UsCodeSignature>,
    /// Title-level `<meta>` block, parsed against the Dublin Core
    /// Metadata Element Set (DCMI Metadata Terms, ISO 15836-1:2017).
    /// `None` when the title's USLM source omits the meta block.
    pub meta: Option<UsCodeMeta>,
    /// Title-level `<toc>` blocks. LRC pl-119-90 publishes one
    /// per-title TOC plus one per-chapter TOC. The title-level
    /// TOCs land here; chapter/part TOCs sit on their respective
    /// containers in [`UsCodeContainer::tocs`].
    pub tocs: Vec<UsCodeToc>,
    /// Every XHTML `<table>` anywhere in the title, in document
    /// order. Includes tables in section bodies, editorial notes,
    /// and statutory notes. LRC pl-119-90 Title 18 ships ~17
    /// tables, mostly "TableOfDisposition" entries cross-referencing
    /// former statute sections.
    pub tables: Vec<UsCodeTable>,
    /// Slice U4 (the document-wrapper backbone): the EXACT ordered child
    /// sequence of the `<uscDoc>` root element as a semantic mixed-content tree
    /// (W3C XML 1.0 §3.2.2) — `<meta>` then `<main>` (→ `<title>` → its `<num>` /
    /// `<heading>` / title-level notes / `<toc>` / the `<chapter>` hierarchy
    /// containers → `<section>`s). Every node is a named [`UsCodeContentNode`]
    /// ([`UsCodeContentNode::Generic`] keyed by qualified name for the elements
    /// the typed projections above do not model as their own kind — `<meta>`,
    /// `<main>`, `<title>`, `<chapter>`, `<toc>`, …; `<ref>` / `<date>` / inline
    /// ornaments / `<p>` as their typed variants), with `#PCDATA` captured
    /// VERBATIM — NOT an opaque Infoset blob. This is the backbone-faithful source
    /// of truth from which [`write_uslm`](super::super::lens::writer::write_uslm)
    /// regenerates the whole `<uscDoc>` document; the flat `sections` /
    /// `hierarchy` / `meta` / `tocs` / … projections are its derived views.
    ///
    /// `None` for a bare-`<section>` slice document (no `<uscDoc>` wrapper), which
    /// `write_uslm` regenerates from the single-section path instead.
    pub uscdoc_mixed: Option<UsCodeMixed>,
}

impl UsCodeTitle {
    /// Find a section by its USLM identifier (e.g.
    /// `/us/usc/t18/s1514A`). Returns `None` if absent.
    pub fn section(&self, identifier: &str) -> Option<&UsCodeSection> {
        self.sections.iter().find(|s| s.identifier == identifier)
    }

    /// Iterate every [`UsCodeContainer`] in the hierarchy
    /// (Subtitle, Part, Subpart, Chapter, Subchapter), at any
    /// depth, in DFS order.
    pub fn containers(&self) -> Vec<&UsCodeContainer> {
        let mut out = Vec::new();
        for node in &self.hierarchy {
            walk_containers(node, &mut out);
        }
        out
    }

    /// Containers of a specific kind, in DFS order.
    pub fn containers_of_kind(&self, kind: ContainerKind) -> Vec<&UsCodeContainer> {
        self.containers()
            .into_iter()
            .filter(|c| c.kind == kind)
            .collect()
    }
}

fn walk_containers<'a>(node: &'a HierarchyNode, out: &mut Vec<&'a UsCodeContainer>) {
    if let HierarchyNode::Container(c) = node {
        out.push(c);
        for child in &c.children {
            walk_containers(child, out);
        }
    }
}

/// A node in the published title hierarchy — either an
/// intermediate container (Subtitle/Part/Subpart/Chapter/
/// Subchapter) or a leaf [`UsCodeSection`].
///
/// Both variants are boxed so the enum size remains small (one
/// pointer) regardless of how many fields the underlying value
/// types accumulate as the ontology grows.
// `HierarchyNode` and `UsCodeContainer` form a mutual cycle
// (`UsCodeContainer::children: Vec<HierarchyNode>`). The rkyv derive needs the
// recursive bound cycle broken with `#[rkyv(omit_bounds)]` on the back-edge
// field (`UsCodeContainer::children`) plus the manual non-recursive container
// bounds the omitted derive would otherwise supply — the canonical rkyv 0.8
// recursive-type pattern (rkyv `examples/json_like_schema.rs`), mirroring
// `OwnedUscSubdivision` in `corpus::prx`. `HierarchyNode` itself carries no
// recursive bound of its own (its boxed members are concrete `Archive` types),
// so it takes the plain derive.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum HierarchyNode {
    Container(Box<UsCodeContainer>),
    Section(Box<UsCodeSection>),
}

/// A hierarchical container between [`UsCodeTitle`] and
/// [`UsCodeSection`]. USLM has five such kinds with the same
/// structural shape (identifier, num, heading, children) and
/// different semantic roles tracked by [`ContainerKind`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "prx", rkyv(serialize_bounds(
    __S: rkyv::ser::Writer + rkyv::ser::Allocator,
    __S::Error: rkyv::rancor::Source,
)))]
#[cfg_attr(feature = "prx", rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source)))]
#[cfg_attr(feature = "prx", rkyv(bytecheck(bounds(
    __C: rkyv::validation::ArchiveContext,
    __C::Error: rkyv::rancor::Source,
))))]
pub struct UsCodeContainer {
    pub kind: ContainerKind,
    /// USLM identifier, e.g. `/us/usc/t18/ptI`.
    pub identifier: String,
    /// `<num>` value, e.g. `"I"`, `"1"`, `"A"`.
    pub num: String,
    /// `<heading>` plain text.
    pub heading: String,
    /// Nested children — further containers or leaf sections. `omit_bounds`
    /// breaks the `HierarchyNode` ↔ `UsCodeContainer` recursive bound cycle.
    #[cfg_attr(feature = "prx", rkyv(omit_bounds))]
    pub children: Vec<HierarchyNode>,
    /// Container-level editorial notes blocks (chapter-level
    /// short titles, amendment history, etc.).
    pub notes_blocks: Vec<UsCodeNotesBlock>,
    /// Bare `<note>` children of this container (not inside a
    /// `<notes>` wrapper).
    pub bare_notes: Vec<UsCodeNote>,
    /// `<toc>` blocks at this container's level. Chapters typically
    /// carry a TOC of their own sections; subchapters/subparts
    /// likewise. See [`UsCodeToc`].
    pub tocs: Vec<UsCodeToc>,
}

// ---------------------------------------------------------------------------
// Tier-2: Notes, SourceCredit, Continuation, Proviso, Header (M4.δ.5)
//
// USLM tracks editorial and provenance material as siblings of
// the navigational hierarchy. Per LRC USLM Schema § "Editorial
// Structure" these are NOT part of the published statutory
// hierarchy — they're metadata about it.
// ---------------------------------------------------------------------------

/// USLM `<notes>` block — wrapper grouping editorial notes
/// attached to a Title, Container, or Section.
///
/// The `block_type` attribute distinguishes USLM-defined kinds:
/// `"uscNote"` (the dominant in-corpus value), `"statutoryNote"`,
/// etc. See [`UsCodeNote`]'s `topic` field for finer-grained semantic kinds.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeNotesBlock {
    pub block_type: Option<String>,
    pub identifier: Option<String>,
    pub heading: Option<String>,
    pub notes: Vec<UsCodeNote>,
    /// The `<notes>` block's OWN direct `<heading>` semantic mixed tree
    /// (slice U3) — present only when the block carries a heading element
    /// directly under `<notes>` (before its `<note>` children), as opposed
    /// to the per-`<note>` headings the LRC Title 1 corpus uses. `None`
    /// when the block has no direct heading (the dominant in-corpus form).
    /// The backbone-faithful source of truth for the block heading; the
    /// `heading` `String` projection is derived from it.
    pub heading_mixed: Option<UsCodeMixed>,
}

/// A single USLM `<note>` element.
///
/// The `topic` attribute is the LRC's editorial-semantic
/// discriminator: `"amendments"`, `"enacting"`, `"shortTitle"`,
/// `"miscellaneous"`, `"dispositionOfSections"`, `"editorialNotes"`,
/// etc. Praxis treats it as an open enumeration — known values
/// can be promoted to typed variants in M4.δ.12.
///
/// The `role` attribute (when present) describes layout role
/// (`"crossHeading"` etc.); semantically informative but not
/// part of the note's body.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeNote {
    pub topic: Option<String>,
    pub role: Option<String>,
    /// `type` attribute. The standard value is `"footnote"` (per
    /// USLM schema); other values may appear. Stored verbatim;
    /// classification into [`UsCodeNoteKind`] uses both `topic` and
    /// `note_type` per the LRC User Guide § 6.2.
    pub note_type: Option<String>,
    pub identifier: Option<String>,
    pub heading: Option<String>,
    /// Body text, flattened from `<p>` and other children.
    /// Inline structure (italic, references) is preserved as text
    /// for now; M4.δ.7 lifts it to `Vec<UsCodeInlineRun>`.
    pub body: String,
    /// Cross-references collected from the note's body. `<ref>`
    /// elements with `href` attribute; `idref` (footnote backlink)
    /// refs are filtered out per the same rule as in section
    /// bodies.
    pub refs: Vec<UsCodeRef>,
    /// `<quotedContent>` blocks inside this note — statutory text
    /// from the amending act being quoted.
    pub quoted_contents: Vec<UsCodeQuotedContent>,
    /// `<date>` elements anywhere in the note body. Captured as
    /// typed values per ISO 8601.
    pub dates: Vec<UsCodeDate>,
    /// The `<note>` element's EXACT ordered child sequence (slice U3) — the
    /// note body as TRUE MIXED CONTENT (W3C XML 1.0 §3.2.2). A `<note>`
    /// interleaves block-level children (`<heading>`, `<p>`, and — in other
    /// titles — `<num>` / `<table>` / `<signature>` / `<quotedContent>`)
    /// each of which is itself mixed content (a `<heading>` carries `<b>`; a
    /// `<p>` interleaves literal text with `<ref>` / `<date>` / `<i>`). The
    /// ordered tree is the backbone-faithful source of truth from which the
    /// `heading` / `body` / `refs` / `dates` projections are derived, and the
    /// writer regenerates the `<note>`'s child sequence from it node-for-node
    /// (the `<heading>` lands as a [`UsCodeContentNode::Generic`] keyed by its
    /// name, a `<p>` as a [`UsCodeContentNode::Para`], etc. — semantic named
    /// nodes, never an opaque exact-bytes blob).
    pub body_mixed: UsCodeMixed,
}

impl UsCodeNote {
    /// Classify this note per the LRC USLM User Guide § 6.2
    /// topic vocabulary. Derived — `topic` is the source of truth.
    pub fn kind(&self) -> UsCodeNoteKind {
        UsCodeNoteKind::parse(self.topic.as_deref(), self.note_type.as_deref())
    }
}

/// USLM `<sourceCredit>` — Pub. L. provenance for a section,
/// citing the originating act and Stat. citation.
///
/// Example body: `"(Pub. L. 107-204, title VIII, § 806, July 30,
/// 2002, 116 Stat. 804.)"`. The `<ref>` children inside resolve to
/// the originating public-law URN (`/us/pl/107/204/...`) and the
/// Stat. URN (`/us/stat/116/804`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeSourceCredit {
    pub identifier: Option<String>,
    /// Whitespace-collapsed plain-text projection of `mixed` (DERIVED).
    pub text: String,
    /// Cross-references — DERIVED from `mixed` (every `<ref href>` in
    /// document order).
    pub refs: Vec<UsCodeRef>,
    /// `<date>` elements inside the credit (act dates, amendment
    /// dates). Captured as typed ISO 8601 values. DERIVED from `mixed`.
    pub dates: Vec<UsCodeDate>,
    /// The `<sourceCredit>` semantic mixed-content tree (slice U1) —
    /// the EXACT ordered sequence of literal punctuation (`"("`,
    /// `", "`, `"; "`, `".)"`) interleaved with `<ref>` / `<date>`
    /// children (W3C XML 1.0 §3.2.2). The backbone-faithful source of
    /// truth; `text` / `refs` / `dates` are its lossy projections, and
    /// the writer regenerates `<sourceCredit>`'s child sequence from it.
    pub mixed: UsCodeMixed,
}

/// USLM `<continuation>` — text continuation across a
/// subdivision boundary. Used when a section's body text continues
/// past an enumerated paragraph back to the parent's flow.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeContinuation {
    pub body: String,
}

/// USLM `<proviso>` — a "provided that…" clause embedded in a
/// section's body. Distinct from the regular body text because
/// it's a conditional / exception qualifier per long-standing
/// statutory drafting convention.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeProviso {
    pub body: String,
}

/// A `<table>` block per W3C HTML 4.01 §11 "Tables" (the underlying
/// table model that XHTML 1.0 transcribes — XHTML 1.0 itself has no
/// §9). USLM embeds XHTML-namespaced tables when statutory text needs
/// tabular layout (e.g. Table of Disposition entries, conversion
/// tables). Tables live in the XHTML namespace, not USLM's —
/// discrimination is on the namespace URI per W3C XML Namespaces 1.0
/// §6.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeTable {
    /// `id` attribute.
    pub identifier: Option<String>,
    /// `class` attribute. LRC uses semantic classes like
    /// `"TableOfDisposition"` — preserved verbatim for downstream
    /// classification.
    pub class: Option<String>,
    /// Header rows (from `<thead>`).
    pub header_rows: Vec<UsCodeTableRow>,
    /// Body rows (from `<tbody>` or direct children of `<table>`).
    pub body_rows: Vec<UsCodeTableRow>,
}

/// One `<tr>` row of a [`UsCodeTable`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeTableRow {
    /// `class` attribute, if any.
    pub class: Option<String>,
    /// Cells in left-to-right order — both `<th>` and `<td>` per
    /// HTML 4.01 §11.2.6. Cell kind discriminated by [`UsCodeTableCellKind`].
    pub cells: Vec<UsCodeTableCell>,
}

/// One `<th>` or `<td>` cell of a [`UsCodeTableRow`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeTableCell {
    /// Header (`<th>`) vs data (`<td>`).
    pub kind: UsCodeTableCellKind,
    /// Visible flat-text content (whitespace collapsed).
    pub text: String,
    /// `colspan` attribute, if any (default 1 per HTML 4.01 §11.2.6).
    pub colspan: Option<u32>,
    /// `rowspan` attribute, if any (default 1).
    pub rowspan: Option<u32>,
}

/// USLM `<toc>` block — a table-of-contents listing per LRC USLM
/// User Guide § "Table of Contents". A TOC carries a sequence of
/// `<tocItem>`s; each item points (via inline `<ref>`s) at a
/// navigable element in the corpus (part, chapter, section).
///
/// The TOC's `role` attribute discriminates layout variants — LRC
/// uses `"threeColumnTOC"` for Part/Heading/Section three-column
/// layout. The role is preserved verbatim for downstream renderers.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeToc {
    /// `id` attribute on `<toc>`, if any.
    pub identifier: Option<String>,
    /// `role` attribute. Values observed in LRC pl-119-90:
    /// `"threeColumnTOC"`. Other USLM documents may use different
    /// roles; the field is preserved verbatim.
    pub role: Option<String>,
    /// TOC entries in document order. Each `<tocItem>` becomes a
    /// [`UsCodeTocItem`].
    pub items: Vec<UsCodeTocItem>,
}

/// One row in a [`UsCodeToc`] — `<tocItem>` per LRC USLM Schema.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeTocItem {
    /// USLM identifier of the *target* (not the TOC item itself).
    /// Derived from the first `<ref href="...">` inside the item if
    /// present; `None` if the item has no cross-reference.
    pub target: Option<String>,
    /// Visible text projection — every column's text concatenated
    /// with `\t` separators, mirroring how a tabular TOC would
    /// render. Useful for downstream text search / navigation.
    pub text: String,
    /// Cross-references collected from this item's body. Typically
    /// one or two: an identifier `<ref>` for the part/chapter and a
    /// section-number `<ref>` for the leftmost statute.
    pub refs: Vec<UsCodeRef>,
}

/// USLM `<meta>` block — title-level Dublin Core metadata.
///
/// Cited per DCMI Metadata Terms (Dublin Core Metadata Initiative,
/// ISO 15836-1:2017). The four elements consistently populated by
/// LRC release pl-119-90 (across Titles 18 and 49) are `dc:title`,
/// `dc:type`, `dc:publisher`, `dc:creator`. Other Dublin Core
/// elements (`dc:date`, `dc:identifier`, `dc:language`, etc.) are
/// defined by the spec and could be present in other USLM source
/// documents (bills, public laws); this type carries them as
/// `Option<String>` so an incomplete meta block doesn't fail.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeMeta {
    /// `<dc:title>`. Typically `"Title 18"`, `"Title 49"`, etc.
    pub title: Option<String>,
    /// `<dc:type>`. LRC's USLM uses `"USCTitle"` for USC titles,
    /// `"USCBill"` for bills, etc.
    pub doc_type: Option<String>,
    /// `<dc:publisher>`. Always `"OLRC"` (Office of the Law
    /// Revision Counsel) for LRC-published USC titles.
    pub publisher: Option<String>,
    /// `<dc:creator>`. The software identifier that produced the
    /// USLM, e.g. `"USCConverter 1.7.2"`.
    pub creator: Option<String>,
    /// `<dc:date>` (DCMI Metadata Terms). Free-form date string —
    /// the LRC publishes this when present, often in ISO-8601 form.
    pub date: Option<String>,
    /// `<dc:identifier>`. The canonical identifier for the document,
    /// often a USLM URN (e.g. `/us/usc/t18`).
    pub identifier: Option<String>,
    /// `<dc:language>`. RFC 5646 / BCP 47 language tag — `"en"` for
    /// the published USC.
    pub language: Option<String>,
    /// `<dc:format>`. Typically a MIME-type indicator.
    pub format: Option<String>,
    /// `<dc:rights>`. License / rights-statement text.
    pub rights: Option<String>,
    /// `<dc:source>`. Origin of the document (Pub. L. citation,
    /// etc.).
    pub source: Option<String>,
    /// Dublin Core elements present in the source but not covered
    /// by the typed fields above — `(local_name, body_text)` pairs
    /// in document order. Surfacing rather than silently dropping
    /// preserves data the LRC might add in future releases and lets
    /// callers detect a mismatch between the spec they expect and
    /// the spec they got. The DCMI Metadata Element Set is open-
    /// ended (DCMI Terms 2024-09-16); routing the full vocabulary
    /// to typed fields would require loading the DC schema, which
    /// is deferred.
    pub other_dc: Vec<(String, String)>,
    /// USLM-native `<docNumber>` element body. LRC uses this for
    /// the title number ("18" for Title 18) and other doc-level
    /// numeric identifiers in non-USC documents.
    pub doc_number: Option<String>,
    /// USLM-native `<docPublicationName>` element body. LRC uses
    /// `"Online@<release-point>"` (e.g. `"Online@119-90"`).
    pub doc_publication_name: Option<String>,
    /// USLM-native `<property role="...">` elements. Each carries a
    /// semantic role and a body value (typically `"yes"`/`"no"`).
    /// The canonical role observed in LRC pl-119-90 is
    /// `"is-positive-law"`. Use [`UsCodeMeta::is_positive_law`] to
    /// project that property.
    pub properties: Vec<UsCodeMetaProperty>,
    /// DCMI Terms `<dcterms:created>` body — typically an ISO 8601
    /// timestamp of when the USLM was generated.
    pub dcterms_created: Option<String>,
    /// DCMI Terms `<dcterms:modified>` body — last-modified
    /// timestamp.
    pub dcterms_modified: Option<String>,
    /// DCMI Terms elements not covered by typed fields above. Same
    /// shape and rationale as [`UsCodeMeta::other_dc`].
    pub dcterms_other: Vec<(String, String)>,
}

/// A `<property role="...">value</property>` declaration inside
/// `<meta>`. LRC uses this for legally-significant doc-level facts
/// (e.g. positive-law status). Per LRC USLM User Guide § "Metadata".
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeMetaProperty {
    /// `role` attribute. Observed values: `"is-positive-law"`.
    /// Other values may appear in non-USC documents.
    pub role: Option<String>,
    /// Element body text.
    pub value: String,
}

impl UsCodeMeta {
    /// Whether the title is enacted as positive law per
    /// `<property role="is-positive-law">`. Legally significant:
    /// positive-law titles ARE the statute; non-positive-law titles
    /// are LRC-published compilations whose authoritative source is
    /// the underlying public laws (per 1 U.S.C. § 204 (a)/(b)).
    ///
    /// Returns `None` if the title's USLM source doesn't declare
    /// the property.
    pub fn is_positive_law(&self) -> Option<bool> {
        for p in &self.properties {
            if p.role.as_deref() == Some("is-positive-law") {
                return Some(
                    p.value.eq_ignore_ascii_case("yes") || p.value.eq_ignore_ascii_case("true"),
                );
            }
        }
        None
    }
}

/// USLM `<header>` — a title-page header element. Typically
/// carries the title's authoritative heading block and is a
/// sibling of `<main>` under `<uscDoc>`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeHeader {
    pub text: String,
}

// ---------------------------------------------------------------------------
// Tier-3: QuotedContent, Date, Signature (M4.δ.6)
//
// LRC USLM Schema § "Quoted Content" + § "Inline Typed Values".
// QuotedContent is load-bearing for amendment history — it carries
// statutory text being inserted, replaced, or deleted by an
// amending act. Sections appearing INSIDE quotedContent are
// citations / quoted excerpts, not part of the title's published
// hierarchy; we model them as [`UsCodeSectionRef`].
// ---------------------------------------------------------------------------

/// USLM `<quotedContent origin="...">` — statutory text quoted
/// from an amending act, typically inside a `<note>`. The `origin`
/// attribute is a URN pointing at the law being quoted (e.g.
/// `/us/pl/107/204/s806`).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeQuotedContent {
    /// `origin="..."` — URN of the act being quoted.
    pub origin: Option<String>,
    /// Plain-text body of the quote.
    pub body: String,
    /// `<section>` elements inside the quote — citations to
    /// sections, NOT real published sections. The structural
    /// distinction matters: a UsCodeSectionRef appears inside a
    /// UsCodeQuotedContent body, while a UsCodeSection lives in
    /// the title's published hierarchy.
    pub section_refs: Vec<UsCodeSectionRef>,
    /// `<ref href="...">` cross-references within the quote.
    pub refs: Vec<UsCodeRef>,
}

/// USLM `<section>` element appearing inside `<quotedContent>` —
/// a citation to a section, not a real published section.
///
/// Distinct from [`UsCodeSection`] which represents a real
/// hierarchical entity in the title. The two share local-name
/// (`section`) but live in different ontological roles per the
/// USLM Schema's quoted-content semantics.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeSectionRef {
    pub identifier: Option<String>,
    pub num: String,
    pub heading: Option<String>,
    pub body: String,
}

/// USLM `<date date="YYYY-MM-DD">visible text</date>` — a typed
/// date value. Used pervasively in source credits, amendment
/// history, effective-date notes.
///
/// Per ISO 8601 the `date` attribute holds the canonical date
/// string; the element body holds the human-readable form (e.g.
/// "July 30, 2002").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeDate {
    pub iso: String,
    pub text: String,
}

/// USLM `<signature>` — a signature block, typically containing
/// one or more `<name>` elements identifying the signatory and
/// their role.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeSignature {
    pub names: Vec<UsCodeName>,
}

/// USLM `<name>` — a person or entity name, typically inside a
/// `<signature>` block.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeName {
    pub text: String,
}

// ---------------------------------------------------------------------------
// Tier-4: Inline markup (M4.δ.7)
//
// USLM inherits the inline-markup family from HTML/XHTML:
// `<inline class="...">`, `<i>`, `<b>`, `<sup>`, `<sub>`, `<span>`,
// `<a>`. These mark up runs of text within headings, chapeaux,
// content, notes, and quotedContent. Praxis preserves the typed
// markup so downstream consumers can distinguish e.g. small-caps
// statutory headings from plain headings, italic terms-of-art from
// plain terms.
//
// Coexists with the existing `String` fields (plain-text
// projection) — every text field that supports inline markup
// also has a parallel `*_runs: Vec<UsCodeInlineRun>` field. The
// runs are the typed structure; the String is its flattening for
// callers that don't care about ornaments.
//
// Citation: LRC USLM Schema § "Inline Elements"; W3C XHTML 1.0
// inline-element model.
// ---------------------------------------------------------------------------

/// A single inline-markup run within a text-bearing element.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeInlineRun {
    pub kind: InlineKind,
    pub text: String,
    /// CSS-style class on `<inline>` / `<span>` (e.g.
    /// `"small-caps"`, `"centered"`). Other inline kinds have
    /// class only when the source XML carries one.
    pub class: Option<String>,
    /// `<a href="...">` and `<ref href="...">` carry an href.
    /// For other inline kinds it's `None`.
    pub href: Option<String>,
}

// ---------------------------------------------------------------------------
// Semantic mixed-content tree (slice U1 — the graph-faithful backbone model).
//
// USLM text-bearing elements have TRUE MIXED CONTENT per W3C XML 1.0 Fifth
// Edition §3.2.2 (Mixed Content): a `<sourceCredit>` / `<p>` / `<heading>`
// interleaves literal `#PCDATA` runs (the punctuation `"("`, `", "`, `".)"`
// that joins citations) with inline child elements (`<ref>`, `<date>`,
// `<inline>`). The ORDER of that interleaving is load-bearing — it is the
// statutory sentence, not a set — so a flat `Vec<UsCodeInlineRun>` (which
// collapses whitespace, drops empty-text runs, and gathers `<ref>`s out of
// position into a side list) cannot regenerate the element backbone.
//
// [`UsCodeContentNode`] is the backbone-faithful replacement: a semantic
// ordered tree whose [`UsCodeContentNode::Text`] holds GENUINE `#PCDATA` runs
// VERBATIM (no whitespace collapse) and whose element variants
// (`Ref` / `Date` / `Inline` / `Para`) carry typed attributes plus their own
// ordered children. It is NOT a DOM-in-disguise: there is no opaque
// exact-bytes child vector — only semantic nodes, with exact-bytes strings
// reserved strictly for `#PCDATA`. The byte residue the W3C Information Set
// (Cowan & Tobin 2004) does not carry — attribute order, inter-element
// white-space layout, entity form — lives in the GENERIC SourceSyntax
// complement, never per node.
//
// The flat `*_runs` / plain-text / `refs` projections are KEPT and DERIVED
// from this tree (see [`UsCodeContentNode::collect_inline_runs`],
// [`UsCodeContentNode::plain_text`], [`UsCodeContentNode::collect_refs`]) so
// every Stratum-B (`from_uslm_titles_owned`) and downstream consumer keeps
// compiling and passing — the tree is the new source of truth, the flat views
// are its lossy shadows.
//
// Citation: W3C XML 1.0 Fifth Edition §3.2.2 (Mixed Content); LRC USLM XML
// User Guide § "Inline Elements"; U.S. House Office of the Law Revision
// Counsel, USLM-1.0.18.xsd `<xsd:element name="ref"/>` / `"date"` / `"inline"`.
// ---------------------------------------------------------------------------

/// One node of a USLM mixed-content sequence (W3C XML 1.0 §3.2.2). Either a
/// genuine `#PCDATA` text run captured VERBATIM, or a typed inline child
/// element carrying its own ordered children.
///
/// This is a SEMANTIC tree, not an Infoset blob: `Text` is the only
/// exact-bytes leaf and it holds *character data only*; every element kind is
/// a named, typed variant. White-space layout, attribute order, and the
/// `<!DOCTYPE>` / namespaces are byte residue carried by the generic
/// SourceSyntax complement, not here.
// `UsCodeContentNode` is recursive (`children: Vec<UsCodeContentNode>` in every
// element variant), so the rkyv derive needs `#[rkyv(omit_bounds)]` on each
// recursive field to break the `Self: Archive` bound cycle, plus the manual
// non-recursive container bounds the omitted derive would otherwise supply — the
// canonical rkyv 0.8 recursive-type pattern (rkyv `examples/json_like_schema.rs`),
// mirroring `OwnedUscSubdivision` in `corpus::prx`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "prx", rkyv(serialize_bounds(
    __S: rkyv::ser::Writer + rkyv::ser::Allocator,
    __S::Error: rkyv::rancor::Source,
)))]
#[cfg_attr(feature = "prx", rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source)))]
#[cfg_attr(feature = "prx", rkyv(bytecheck(bounds(
    __C: rkyv::validation::ArchiveContext,
    __C::Error: rkyv::rancor::Source,
))))]
pub enum UsCodeContentNode {
    /// A genuine `#PCDATA` run (W3C XML 1.0 §2.4 \[14\] `CharData`), captured
    /// VERBATIM — no whitespace collapse, no trim — so the writer reproduces
    /// the exact text leaf the source carried (the `"("`, `", "`, `".)"`
    /// punctuation between citations is part of this).
    Text(String),
    /// A `<ref href="…">…</ref>` cross-reference (USLM citation-graph edge).
    /// `attrs` are the source attributes in order (only `href` is
    /// semantically projected to [`UsCodeRef`]); `children` is the ref's own
    /// mixed content (its visible text, possibly itself inline-marked).
    Ref {
        attrs: Vec<UsCodeContentAttr>,
        #[cfg_attr(feature = "prx", rkyv(omit_bounds))]
        children: Vec<UsCodeContentNode>,
    },
    /// A `<date date="YYYY-MM-DD">…</date>` typed-value element.
    Date {
        attrs: Vec<UsCodeContentAttr>,
        #[cfg_attr(feature = "prx", rkyv(omit_bounds))]
        children: Vec<UsCodeContentNode>,
    },
    /// A USLM/XHTML inline ornament (`<inline>` / `<i>` / `<b>` / `<sup>` /
    /// `<sub>` / `<span>` / `<a>`) — [`InlineKind`] names which. `children`
    /// is its mixed content.
    Inline {
        kind: InlineKind,
        attrs: Vec<UsCodeContentAttr>,
        #[cfg_attr(feature = "prx", rkyv(omit_bounds))]
        children: Vec<UsCodeContentNode>,
    },
    /// A block-level `<p>` paragraph inside a `<content>` / `<chapeau>`.
    /// `children` is its mixed content.
    Para {
        attrs: Vec<UsCodeContentAttr>,
        #[cfg_attr(feature = "prx", rkyv(omit_bounds))]
        children: Vec<UsCodeContentNode>,
    },
    /// Any other element the slice does not yet model as its own typed kind —
    /// carried with its exact local NAME so the backbone writer reproduces it
    /// faithfully (and no text is lost). This is a SEMANTIC named node, NOT an
    /// opaque exact-bytes blob: its `children` are themselves
    /// [`UsCodeContentNode`]s. Widening the typed vocabulary (promoting more
    /// of USLM's ~50-element inline set out of `Generic`) is the next slice.
    Generic {
        /// The element's local name (e.g. an unmodeled inline ornament).
        name: String,
        attrs: Vec<UsCodeContentAttr>,
        #[cfg_attr(feature = "prx", rkyv(omit_bounds))]
        children: Vec<UsCodeContentNode>,
    },
}

/// One source attribute on a [`UsCodeContentNode`] element, captured in source
/// order as a `(qualified-name, value)` pair. The qualified name keeps any
/// prefix (`xml:lang`) so the writer reproduces it; the generic
/// `AttributeOverrides` complement carries the EXACT byte sequence, so this
/// only needs to be present, not byte-perfect, for the backbone diff.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeContentAttr {
    /// Qualified attribute name (e.g. `href`, `date`, `class`, `xml:lang`).
    pub name: String,
    /// Attribute value, verbatim.
    pub value: String,
}

impl UsCodeContentNode {
    /// The element's first attribute whose qualified name is `key`, if any.
    #[must_use]
    pub fn attr(&self, key: &str) -> Option<&str> {
        let attrs = match self {
            Self::Text(_) => return None,
            Self::Ref { attrs, .. }
            | Self::Date { attrs, .. }
            | Self::Inline { attrs, .. }
            | Self::Para { attrs, .. }
            | Self::Generic { attrs, .. } => attrs,
        };
        attrs
            .iter()
            .find(|a| a.name == key)
            .map(|a| a.value.as_str())
    }

    /// The element's children (empty for a [`Self::Text`] leaf).
    #[must_use]
    pub fn children(&self) -> &[UsCodeContentNode] {
        match self {
            Self::Text(_) => &[],
            Self::Ref { children, .. }
            | Self::Date { children, .. }
            | Self::Inline { children, .. }
            | Self::Para { children, .. }
            | Self::Generic { children, .. } => children,
        }
    }

    /// Append every descendant `#PCDATA` run to `buf` (pre-order) — the
    /// un-normalized concatenation. The DERIVED plain-text / `*_runs`
    /// projections normalize on top of this.
    pub fn push_raw_text(&self, buf: &mut String) {
        match self {
            Self::Text(t) => buf.push_str(t),
            other => {
                for child in other.children() {
                    child.push_raw_text(buf);
                }
            }
        }
    }

    /// Append every descendant `#PCDATA` run that belongs to the element's
    /// PROSE to `buf` (pre-order), SKIPPING editorial footnote annotation —
    /// the typed `<note type="footnote">` the LRC nests inside a text-bearing
    /// element (e.g. a `<heading>`) plus the superscript `<ref
    /// class="footnoteRef">` marker that points at it.
    ///
    /// This is the discriminator the flat [`UsCodeMixed::plain_text`]
    /// projection lacks: `plain_text` flattens the whole mixed tree, so the
    /// footnote's own sentence ("Section catchline was not amended…") leaks
    /// into the heading string and a reader of the prose sees the editor's
    /// note as if it were part of the title. The typed model already
    /// DISTINGUISHES these nodes (the footnote is a
    /// [`Self::Generic`]`{ name: "note" }` carrying `type="footnote"`; the
    /// marker is a [`Self::Ref`] carrying `class="footnoteRef"` — both per the
    /// LRC USLM XML User Guide § "Notes" / XHTML footnote-reference idiom), so
    /// the prose projection just declines to descend into them.
    ///
    /// CONSERVATIVE by construction: only those two annotation shapes are
    /// skipped. A genuine `<ref href="…">` cross-reference in the prose is
    /// kept (it has no `class="footnoteRef"`); a non-footnote `<note>` (e.g.
    /// `type="uscNote"`) is kept (its `type` is not `"footnote"`); every other
    /// node recurses exactly as [`Self::push_raw_text`] would.
    pub fn push_prose_text(&self, buf: &mut String) {
        match self {
            Self::Text(t) => buf.push_str(t),
            // The editorial footnote the LRC embeds inside a text-bearing
            // element: a `<note type="footnote">`. Skip its WHOLE subtree —
            // its `<num>` marker and its sentence are annotation, not prose.
            Self::Generic { name, .. }
                if name == "note" && self.attr("type") == Some("footnote") => {}
            // The superscript marker that points at that footnote: a `<ref
            // class="footnoteRef">`. Skip its subtree (the bare marker digit).
            Self::Ref { .. } if self.attr("class") == Some("footnoteRef") => {}
            other => {
                for child in other.children() {
                    child.push_prose_text(buf);
                }
            }
        }
    }
}

/// A USLM mixed-content sequence — the ordered child list of one text-bearing
/// element (`<heading>` / `<content>` / `<chapeau>` / `<sourceCredit>` / `<p>`
/// / `<ref>` …). The semantic source of truth from which the flat `*_runs` /
/// plain-text / `refs` views are derived.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeMixed {
    /// The element's children in EXACT source order (W3C XML 1.0 §3.2.2).
    pub nodes: Vec<UsCodeContentNode>,
}

impl UsCodeMixed {
    /// Empty sequence (the element had no children).
    #[must_use]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// `true` when the sequence carries no nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The un-normalized concatenation of every descendant `#PCDATA` run.
    #[must_use]
    pub fn raw_text(&self) -> String {
        let mut buf = String::new();
        for node in &self.nodes {
            node.push_raw_text(&mut buf);
        }
        buf
    }

    /// Whitespace-collapsed, trimmed plain text — the DERIVED projection that
    /// the legacy `heading` / `content` / `chapeau` `String` fields hold
    /// (W3C XML 1.0 §2.10 White Space Handling).
    #[must_use]
    pub fn plain_text(&self) -> String {
        collapse_ws(&self.raw_text())
    }

    /// Whitespace-collapsed, trimmed PROSE text — `plain_text` minus the
    /// editorial footnote annotation the LRC nests inside the element (the
    /// typed `<note type="footnote">` and its `<ref class="footnoteRef">`
    /// marker). See [`UsCodeContentNode::push_prose_text`].
    ///
    /// The lexical-understanding pipeline reads THIS, not `plain_text`: a
    /// heading's prose is its title, and the editor's footnote ("Section
    /// catchline was not amended…") is metadata about the title, not a word IN
    /// the title — so resolving the heading's lemmas against WordNet should
    /// never see "catchline". `plain_text` is deliberately left untouched
    /// (the byte-exact writer + the `heading` flat projection depend on it),
    /// so this is a strictly-narrower SIBLING projection, not a replacement.
    #[must_use]
    pub fn prose_text(&self) -> String {
        let mut buf = String::new();
        for node in &self.nodes {
            node.push_prose_text(&mut buf);
        }
        collapse_ws(&buf)
    }
}

/// Collapse internal whitespace runs to single spaces and trim — the W3C XML
/// 1.0 §2.10 insignificant-white-space normalization the flat plain-text views
/// apply on top of the verbatim mixed-content tree.
fn collapse_ws(s: &str) -> String {
    let trimmed = s.trim();
    let mut out = String::with_capacity(trimmed.len());
    let mut prev_space = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

/// Amendment markup — `<ins>` and `<del>` per LRC USLM User Guide
/// § "Amendment Markup". These elements are populated in USLM
/// sources representing amendments-in-progress (e.g. an enrolled
/// bill modifying existing statutory text); LRC pl-119-90 USC
/// titles, being already-codified, carry zero `<ins>` / `<del>`.
///
/// Stored as a typed kind on a flat-text body so legal-research
/// tooling can ask "show me the text as it stood before vs after
/// this amendment" without re-parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeAmendmentMarkup {
    pub kind: UsCodeAmendmentKind,
    /// Body text of the `<ins>` or `<del>` element (whitespace
    /// collapsed per W3C XML 1.0 §2.10).
    pub text: String,
}

// ---------------------------------------------------------------------------
// Definitional ontology (M4.δ.13 / Tier-5).
//
// USLM distinguishes three definitional elements:
//
//   <def>     — a *definitional clause*: a structural wrapper around
//               one or more `<term>`s and the prose that defines them
//               ("In this section, the following terms apply…").
//   <term>    — a *defined-term mention*: the surface form of a
//               statutory term-of-art being defined or used. Carries
//               the term's canonical text and (optionally) a
//               normative pointer back to its `<def>` host.
//   <marker>  — a *cross-reference anchor*: a named target that
//               other `<ref>`s can link to via fragment identifier.
//               Distinct from `<term>` because it doesn't define
//               anything; it merely names a point in the text.
//
// LRC's USC release pl-119-90 ships these schema elements but they
// are not yet populated in Titles 18 / 49 — defined-terms still
// appear there as `<inline class="small-caps">` ornaments. The
// types here cover the schema element shape so that when LRC
// rolls forward the retro-conversion, the reader handles them
// without further code change.
//
// Citation: LRC, *USLM XML User Guide* (USLM-1.0.15.xsd), § "Lexical
// Elements"; Bluebook § 3.3 ("Sections and Paragraphs") for the standard
// convention that defined terms in statutes are introduced with the
// formula "the term … means …".
// ---------------------------------------------------------------------------

/// A `<def>` block — a definitional clause introducing one or more
/// `<term>` definitions. Per USLM Schema § "Lexical Elements".
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeDefBlock {
    /// USLM `id` if present (allows cross-references to land here).
    pub identifier: Option<String>,
    /// All `<term>` mentions inside this `<def>`. The first term is
    /// conventionally the canonical defined term; subsequent terms
    /// are typically aliases or grammatical variants.
    pub terms: Vec<UsCodeTerm>,
    /// Flat-text projection of the entire `<def>` body (term +
    /// definitional prose). Useful for fuzzy term lookup until the
    /// downstream NLP pipeline can match against the typed run.
    pub body: String,
}

/// A `<term>` mention — the surface form of a statutory defined
/// term being either introduced (inside a `<def>`) or used (inside
/// any text-bearing element). Per USLM Schema § "Lexical Elements".
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeTerm {
    /// The visible term text, e.g. `"covered employee"`. Whitespace
    /// is collapsed per W3C XML 1.0 §2.10.
    pub text: String,
    /// Optional `refers-to` attribute pointing at the host `<def>`
    /// when this `<term>` is a *use* rather than a *definition*. In
    /// USLM the canonical attribute is `refersTo`; we accept both
    /// `refersTo` and `refers-to` per LRC's xPath examples.
    pub refers_to: Option<String>,
}

/// A `<marker>` element — a named cross-reference target placed in
/// the text. `<marker name="foo"/>` is referenced from elsewhere via
/// `<ref href="#foo">`. Per USLM Schema § "Lexical Elements".
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeMarker {
    /// The marker's `name` attribute — its fragment identifier.
    /// Required by the schema; empty string if missing in source.
    pub name: String,
    /// The marker's `class` attribute, if any (categorizes the
    /// marker's purpose: "anchor", "label", layout class, etc.).
    pub class: Option<String>,
}

/// One § of a U.S. Code title.
///
/// `UsCodeSection` carries a `Vec<UsCodeSubdivision>` (the self-recursive type);
/// the recursion lives on `UsCodeSubdivision`, which holds the `omit_bounds` +
/// manual bounds, so the section itself takes the plain derive.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeSection {
    /// USLM identifier, e.g. `/us/usc/t18/s1514A`.
    pub identifier: String,
    /// The `<num>` value, e.g. `"1514A"`.
    pub num: String,
    /// The `<num>` element's VISIBLE text leaf, e.g. `"§ 2."` —
    /// the `#PCDATA` the `value` attribute does NOT carry (`num` is
    /// `"2"`, this is `"§ 2."`). Empty when the source `<num>` is
    /// childless. Captured for slice U1 so the backbone writer
    /// reproduces the `<num>` text node (W3C XML 1.0 §3.2.2); the
    /// reader populates it from the mixed-content walk.
    pub num_text: String,
    /// Cross-reference footnote the LRC embeds inside `<num>` to
    /// disambiguate a duplicated section number — e.g. "Another
    /// section 3598 is set out after this section." `None` for the
    /// overwhelming majority of sections (a unique section number
    /// carries no such footnote).
    ///
    /// This is the LRC's actual disambiguation mechanism when two
    /// distinct sections share one URN: the section number repeats
    /// (and the heading may even repeat verbatim, as at 5 U.S.C.
    /// § 3598), so the only structural discriminator the LRC publishes
    /// is this `<note type="footnote">` inside each `<num>`. Per the
    /// Office of the Law Revision Counsel, *Detailed Guide to the
    /// United States Code Content and Features* ("Editorial
    /// Reclassification" / duplicate-numbering notes). Retaining it
    /// lets the uniqueness axiom recognize a legitimate LRC duplicate
    /// from a genuine parse error without a hand-coded exceptions list.
    pub num_footnote: Option<String>,
    /// `<heading>` plain text, e.g. "Civil action to protect…".
    /// Flat-text projection of `heading_mixed` (DERIVED).
    pub heading: String,
    /// Typed inline-markup runs from `<heading>` — preserves
    /// small-caps, italic, and other ornaments. DERIVED from
    /// `heading_mixed`.
    pub heading_runs: Vec<UsCodeInlineRun>,
    /// `<heading>` semantic mixed-content tree (slice U1) — the EXACT
    /// ordered `#PCDATA` ↔ inline-element sequence (W3C XML 1.0
    /// §3.2.2). The backbone-faithful source of truth; `heading` and
    /// `heading_runs` are its lossy projections.
    pub heading_mixed: UsCodeMixed,
    /// `<chapeau>` if the § opens with introductory text before
    /// nested subdivisions. Flat-text projection of `chapeau_mixed`.
    pub chapeau: Option<String>,
    pub chapeau_runs: Vec<UsCodeInlineRun>,
    /// `<chapeau>` semantic mixed-content tree (slice U1). `None` when
    /// the § has no `<chapeau>`.
    pub chapeau_mixed: Option<UsCodeMixed>,
    /// `<content>` if the § is a flat (no-subdivision) section.
    /// Flat-text projection of `content_mixed`.
    pub content: Option<String>,
    pub content_runs: Vec<UsCodeInlineRun>,
    /// `<content>` semantic mixed-content tree (slice U1) — carries
    /// any block-level `<p>` children plus interleaved text VERBATIM,
    /// so the backbone writer reproduces `<content><p>…</p></content>`
    /// exactly. `None` when the § has no `<content>`.
    pub content_mixed: Option<UsCodeMixed>,
    /// Nested subdivisions — (a)/(b)/(c)… subsections, each of
    /// which may recurse into paragraphs, subparagraphs, etc.
    pub children: Vec<UsCodeSubdivision>,
    /// Cross-references collected from the § (top-level text only;
    /// nested subdivisions hold their own).
    pub refs: Vec<UsCodeRef>,
    /// Section-level editorial notes blocks — amendment history,
    /// effective dates, short titles, dispositions, etc.
    pub notes_blocks: Vec<UsCodeNotesBlock>,
    /// Bare `<note>` children of this section (not inside a
    /// `<notes>` wrapper).
    pub bare_notes: Vec<UsCodeNote>,
    /// Pub. L. provenance for this section (act, statute citation).
    /// Some sections have multiple `<sourceCredit>` entries when
    /// repeatedly amended; the parser collects all.
    pub source_credits: Vec<UsCodeSourceCredit>,
    /// `<continuation>` blocks within the section body — text
    /// that continues across a subdivision boundary back to the
    /// parent's flow.
    pub continuations: Vec<UsCodeContinuation>,
    /// `<def>` blocks declared at the section's top level. Each
    /// block introduces one or more defined terms. Per LRC USLM
    /// User Guide § "Lexical Elements". Currently empty for LRC
    /// release pl-119-90 (which uses small-caps inline ornaments
    /// for defined terms rather than typed `<def>` elements); the
    /// field exists so when LRC rolls forward the retro-conversion
    /// the parser handles them without further code change.
    pub def_blocks: Vec<UsCodeDefBlock>,
    /// `<marker name="...">` anchors declared at the section's top
    /// level. Per LRC USLM User Guide § "Lexical Elements". Same
    /// "currently empty in pl-119-90" caveat as `def_blocks`.
    pub markers: Vec<UsCodeMarker>,
    /// `<ins>` and `<del>` amendment-markup elements. Empty in
    /// retro-converted USC titles; populated in
    /// amendment/bill USLM. See [`UsCodeAmendmentMarkup`].
    pub amendments: Vec<UsCodeAmendmentMarkup>,
}

/// A single subdivision below a §. The recursive `children` field
/// captures USLM's strictly-nested hierarchy: a subsection contains
/// paragraphs, which contain subparagraphs, which contain clauses,
/// and so on.
// `UsCodeSubdivision` is self-recursive (`children: Vec<UsCodeSubdivision>`), so
// the rkyv derive needs `#[rkyv(omit_bounds)]` on the recursive `children` field
// to break the `Self: Archive` bound cycle, plus the manual non-recursive
// container bounds — the canonical rkyv 0.8 recursive-type pattern (rkyv
// `examples/json_like_schema.rs`), mirroring `OwnedUscSubdivision` in `corpus::prx`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "prx", rkyv(serialize_bounds(
    __S: rkyv::ser::Writer + rkyv::ser::Allocator,
    __S::Error: rkyv::rancor::Source,
)))]
#[cfg_attr(feature = "prx", rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source)))]
#[cfg_attr(feature = "prx", rkyv(bytecheck(bounds(
    __C: rkyv::validation::ArchiveContext,
    __C::Error: rkyv::rancor::Source,
))))]
pub struct UsCodeSubdivision {
    /// USLM identifier, e.g. `/us/usc/t18/s1514A/a/1/A`.
    pub identifier: String,
    /// The `<num>` value, e.g. `"a"`, `"1"`, `"A"`, `"i"`.
    pub num: String,
    /// The `<num>` element's VISIBLE text leaf, e.g. `"(a)"` / `"“(1)"`
    /// — the `#PCDATA` the `value` attribute does NOT carry (`num` is
    /// `"a"`, this is `"(a)"`). Empty when the source `<num>` is
    /// childless. Captured for slice U2 so the backbone writer
    /// reproduces the subdivision `<num>` text node (W3C XML 1.0
    /// §3.2.2), exactly as [`UsCodeSection::num_text`] does for the §.
    pub num_text: String,
    /// Which USLM hierarchy level this subdivision sits at.
    pub kind: SubdivisionKind,
    /// `<heading>` text if any. Flat-text projection of `heading_mixed`
    /// (DERIVED). `None` when the subdivision carries no `<heading>`.
    pub heading: Option<String>,
    /// Typed inline-markup runs from `<heading>` — DERIVED from
    /// `heading_mixed`.
    pub heading_runs: Vec<UsCodeInlineRun>,
    /// `<heading>` semantic mixed-content tree (slice U2) — the EXACT
    /// ordered `#PCDATA` ↔ inline-element sequence (W3C XML 1.0
    /// §3.2.2). The backbone-faithful source of truth; `heading` and
    /// `heading_runs` are its lossy projections. `None` when the
    /// subdivision has no `<heading>`.
    pub heading_mixed: Option<UsCodeMixed>,
    /// `<chapeau>` if this subdivision introduces children.
    /// Flat-text projection of `chapeau_mixed` (DERIVED).
    pub chapeau: Option<String>,
    pub chapeau_runs: Vec<UsCodeInlineRun>,
    /// `<chapeau>` semantic mixed-content tree (slice U2). `None` when
    /// the subdivision has no `<chapeau>`.
    pub chapeau_mixed: Option<UsCodeMixed>,
    /// `<content>` if this subdivision is a leaf. Flat-text
    /// projection of `content_mixed` (DERIVED).
    pub content: Option<String>,
    pub content_runs: Vec<UsCodeInlineRun>,
    /// `<content>` semantic mixed-content tree (slice U2). `None` when
    /// the subdivision is a branch (carries children, not content).
    pub content_mixed: Option<UsCodeMixed>,
    /// Nested children — for a subsection these are paragraphs;
    /// for a paragraph, subparagraphs; etc. `omit_bounds` breaks the
    /// `UsCodeSubdivision` self-recursive bound cycle.
    #[cfg_attr(feature = "prx", rkyv(omit_bounds))]
    pub children: Vec<UsCodeSubdivision>,
    /// Cross-references collected from this subdivision's body
    /// text (not from its children — they hold their own).
    pub refs: Vec<UsCodeRef>,
    /// `<def>` blocks declared at this subdivision's top level.
    /// See [`UsCodeSection::def_blocks`].
    pub def_blocks: Vec<UsCodeDefBlock>,
    /// `<marker name="...">` anchors declared at this subdivision's
    /// top level. See [`UsCodeSection::markers`].
    pub markers: Vec<UsCodeMarker>,
    /// `<ins>` / `<del>` amendment markup. See
    /// [`UsCodeSection::amendments`].
    pub amendments: Vec<UsCodeAmendmentMarkup>,
}

/// A `<ref href="...">` cross-reference within USLM text.
///
/// USLM uses these to wire one statute provision to another by
/// stable identifier URN (e.g. `/us/usc/t15/s78` points at the
/// Securities Exchange Act § 78). Their resolution is the
/// foundation of cross-statute reasoning in the legal layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UsCodeRef {
    /// The `href` attribute value — a USLM identifier URN.
    pub href: String,
    /// The visible text of the reference, e.g. `"15 U.S.C. 78"`.
    pub text: String,
}

// ---------------------------------------------------------------------------
// Read errors
// ---------------------------------------------------------------------------

/// Failure modes when reading USLM XML into the typed ontology.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UslmReadError {
    /// Underlying XML parser rejected the input.
    Xml(String),
    /// XML parsed but the document is missing the USLM root
    /// (`<uscDoc>` or `<title>`).
    NoUsCodeRoot,
    /// A USLM `<title>`'s `<num value="N">` couldn't be parsed as
    /// an integer.
    BadTitleNumber { raw: String },
    /// Generic structural anomaly (an element that should have a
    /// required child doesn't).
    Structure(String),
}

impl core::fmt::Display for UslmReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Xml(e) => write!(f, "XML parse error: {e}"),
            Self::NoUsCodeRoot => {
                write!(f, "no <uscDoc> / <title> root found in USLM document")
            }
            Self::BadTitleNumber { raw } => {
                write!(f, "title @num value {raw:?} is not a valid integer")
            }
            Self::Structure(s) => write!(f, "structural anomaly: {s}"),
        }
    }
}

impl std::error::Error for UslmReadError {}

#[cfg(test)]
mod prose_text_tests {
    use super::{UsCodeContentAttr, UsCodeContentNode, UsCodeMixed};
    use alloc::{string::ToString, vec};

    fn attr(name: &str, value: &str) -> UsCodeContentAttr {
        UsCodeContentAttr {
            name: name.to_string(),
            value: value.to_string(),
        }
    }

    fn text(s: &str) -> UsCodeContentNode {
        UsCodeContentNode::Text(s.to_string())
    }

    /// A `<heading>` shaped exactly like the LRC catchline cases
    /// (18 U.S.C. § 1303): prose text, then a `<ref class="footnoteRef">`
    /// superscript marker, then the `<note type="footnote">` whose sentence
    /// carries "catchline".
    fn heading_with_footnote() -> UsCodeMixed {
        UsCodeMixed {
            nodes: vec![
                text(" Postmaster or employee as lottery agent "),
                UsCodeContentNode::Ref {
                    attrs: vec![attr("class", "footnoteRef"), attr("idref", "fn002105")],
                    children: vec![text("1")],
                },
                UsCodeContentNode::Generic {
                    name: "note".to_string(),
                    attrs: vec![attr("type", "footnote"), attr("id", "fn002105")],
                    children: vec![
                        UsCodeContentNode::Generic {
                            name: "num".to_string(),
                            attrs: vec![],
                            children: vec![text("1")],
                        },
                        text(
                            " Section catchline was not amended to conform to change made in the text by ",
                        ),
                        UsCodeContentNode::Ref {
                            attrs: vec![attr("href", "/us/pl/91/375")],
                            children: vec![text("Pub. L. 91–375")],
                        },
                        text("."),
                    ],
                },
            ],
        }
    }

    #[test]
    fn prose_text_excludes_footnote_note() {
        let prose = heading_with_footnote().prose_text();
        assert_eq!(prose, "Postmaster or employee as lottery agent");
        // Neither the footnote's sentence ("catchline") nor the superscript
        // marker digit survives the prose projection.
        assert!(
            !prose.contains("catchline"),
            "prose must not carry the footnote sentence: {prose:?}"
        );
        assert!(
            !prose.contains("Section catchline"),
            "prose must not carry the footnote sentence: {prose:?}"
        );
        // The `<ref class=footnoteRef>` marker leaf ("1") is also gone — the
        // prose ends at "agent", not "agent 1".
        assert!(
            !prose.contains('1'),
            "prose must not carry the footnoteRef marker digit: {prose:?}"
        );
    }

    #[test]
    fn prose_text_keeps_genuine_href_ref() {
        // A genuine `<ref href="…">` cross-reference in the prose (NOT a
        // footnoteRef) is kept — only `class="footnoteRef"` is skipped.
        let mixed = UsCodeMixed {
            nodes: vec![
                text("Civil action — see "),
                UsCodeContentNode::Ref {
                    attrs: vec![attr("href", "/us/usc/t18/s1514A")],
                    children: vec![text("section 1514A")],
                },
            ],
        };
        assert_eq!(mixed.prose_text(), "Civil action — see section 1514A");
    }

    #[test]
    fn prose_text_keeps_non_footnote_note() {
        // A `<note>` whose `type` is NOT "footnote" (e.g. "uscNote") is kept:
        // the skip predicate is `type == "footnote"`, conservatively narrow.
        let mixed = UsCodeMixed {
            nodes: vec![
                text("Definitions "),
                UsCodeContentNode::Generic {
                    name: "note".to_string(),
                    attrs: vec![attr("type", "uscNote")],
                    children: vec![text("kept-note-prose")],
                },
            ],
        };
        assert_eq!(mixed.prose_text(), "Definitions kept-note-prose");
    }

    #[test]
    fn plain_text_still_includes_annotations() {
        // `plain_text` is UNTOUCHED — on the same synthetic heading it STILL
        // flattens the footnote (so U6 byte-exactness + the flat `heading`
        // projection that depends on it are unchanged).
        let plain = heading_with_footnote().plain_text();
        assert!(
            plain.contains("catchline"),
            "plain_text must still flatten the footnote: {plain:?}"
        );
        assert_eq!(
            plain,
            "Postmaster or employee as lottery agent 11 Section catchline \
             was not amended to conform to change made in the text by Pub. L. 91–375."
        );
    }
}
