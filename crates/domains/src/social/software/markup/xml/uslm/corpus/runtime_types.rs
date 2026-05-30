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
#[derive(Debug, Clone, PartialEq)]
pub enum HierarchyNode {
    Container(Box<UsCodeContainer>),
    Section(Box<UsCodeSection>),
}

/// A hierarchical container between [`UsCodeTitle`] and
/// [`UsCodeSection`]. USLM has five such kinds with the same
/// structural shape (identifier, num, heading, children) and
/// different semantic roles tracked by [`ContainerKind`].
#[derive(Debug, Clone, PartialEq)]
pub struct UsCodeContainer {
    pub kind: ContainerKind,
    /// USLM identifier, e.g. `/us/usc/t18/ptI`.
    pub identifier: String,
    /// `<num>` value, e.g. `"I"`, `"1"`, `"A"`.
    pub num: String,
    /// `<heading>` plain text.
    pub heading: String,
    /// Nested children — further containers or leaf sections.
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
pub struct UsCodeNotesBlock {
    pub block_type: Option<String>,
    pub identifier: Option<String>,
    pub heading: Option<String>,
    pub notes: Vec<UsCodeNote>,
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
pub struct UsCodeSourceCredit {
    pub identifier: Option<String>,
    pub text: String,
    pub refs: Vec<UsCodeRef>,
    /// `<date>` elements inside the credit (act dates, amendment
    /// dates). Captured as typed ISO 8601 values.
    pub dates: Vec<UsCodeDate>,
}

/// USLM `<continuation>` — text continuation across a
/// subdivision boundary. Used when a section's body text continues
/// past an enumerated paragraph back to the parent's flow.
#[derive(Debug, Clone, PartialEq)]
pub struct UsCodeContinuation {
    pub body: String,
}

/// USLM `<proviso>` — a "provided that…" clause embedded in a
/// section's body. Distinct from the regular body text because
/// it's a conditional / exception qualifier per long-standing
/// statutory drafting convention.
#[derive(Debug, Clone, PartialEq)]
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
pub struct UsCodeTableRow {
    /// `class` attribute, if any.
    pub class: Option<String>,
    /// Cells in left-to-right order — both `<th>` and `<td>` per
    /// HTML 4.01 §11.2.6. Cell kind discriminated by [`UsCodeTableCellKind`].
    pub cells: Vec<UsCodeTableCell>,
}

/// One `<th>` or `<td>` cell of a [`UsCodeTableRow`].
#[derive(Debug, Clone, PartialEq)]
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
pub struct UsCodeDate {
    pub iso: String,
    pub text: String,
}

/// USLM `<signature>` — a signature block, typically containing
/// one or more `<name>` elements identifying the signatory and
/// their role.
#[derive(Debug, Clone, PartialEq)]
pub struct UsCodeSignature {
    pub names: Vec<UsCodeName>,
}

/// USLM `<name>` — a person or entity name, typically inside a
/// `<signature>` block.
#[derive(Debug, Clone, PartialEq)]
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
pub struct UsCodeMarker {
    /// The marker's `name` attribute — its fragment identifier.
    /// Required by the schema; empty string if missing in source.
    pub name: String,
    /// The marker's `class` attribute, if any (categorizes the
    /// marker's purpose: "anchor", "label", layout class, etc.).
    pub class: Option<String>,
}

/// One § of a U.S. Code title.
#[derive(Debug, Clone, PartialEq)]
pub struct UsCodeSection {
    /// USLM identifier, e.g. `/us/usc/t18/s1514A`.
    pub identifier: String,
    /// The `<num>` value, e.g. `"1514A"`.
    pub num: String,
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
    /// Flat-text projection of `heading_runs`.
    pub heading: String,
    /// Typed inline-markup runs from `<heading>` — preserves
    /// small-caps, italic, and other ornaments.
    pub heading_runs: Vec<UsCodeInlineRun>,
    /// `<chapeau>` if the § opens with introductory text before
    /// nested subdivisions. Flat-text projection of `chapeau_runs`.
    pub chapeau: Option<String>,
    pub chapeau_runs: Vec<UsCodeInlineRun>,
    /// `<content>` if the § is a flat (no-subdivision) section.
    /// Flat-text projection of `content_runs`.
    pub content: Option<String>,
    pub content_runs: Vec<UsCodeInlineRun>,
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
#[derive(Debug, Clone, PartialEq)]
pub struct UsCodeSubdivision {
    /// USLM identifier, e.g. `/us/usc/t18/s1514A/a/1/A`.
    pub identifier: String,
    /// The `<num>` value, e.g. `"a"`, `"1"`, `"A"`, `"i"`.
    pub num: String,
    /// Which USLM hierarchy level this subdivision sits at.
    pub kind: SubdivisionKind,
    /// `<heading>` text if any. Flat-text projection of `heading_runs`.
    pub heading: Option<String>,
    pub heading_runs: Vec<UsCodeInlineRun>,
    /// `<chapeau>` if this subdivision introduces children.
    /// Flat-text projection of `chapeau_runs`.
    pub chapeau: Option<String>,
    pub chapeau_runs: Vec<UsCodeInlineRun>,
    /// `<content>` if this subdivision is a leaf. Flat-text
    /// projection of `content_runs`.
    pub content: Option<String>,
    pub content_runs: Vec<UsCodeInlineRun>,
    /// Nested children — for a subsection these are paragraphs;
    /// for a paragraph, subparagraphs; etc.
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
