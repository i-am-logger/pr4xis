//! Phase-1 structural-content audit of the USLM lens (Milestone #266).
//!
//! Compares two element histograms over a USLM XML byte stream:
//!
//! 1. **Raw histogram** — every element in the parsed `XmlDocument`,
//!    counted by `(namespace_uri, local_name)`. Built by the
//!    praxis-native `xml::reader::read_xml` (W3C XML 1.0 §3
//!    *Logical Structures*).
//! 2. **Typed histogram** — every node materialised in the lens's
//!    typed view ([`UsCodeTitle`] + its transitive children), counted
//!    by the same `(namespace_uri, local_name)` key. The typed view is
//!    produced by `read_uslm_title` in `leaf_readers`.
//!
//! The *gap* = raw minus typed, per element name. A non-zero gap is
//! the audit signal: it identifies what the typed view fails to
//! capture, before any byte-side-channel ("constant complement") is
//! permitted to paper over the loss.
//!
//! ## Why this is Phase-1 in Milestone #266
//!
//! The OWL graph-faithful refactor at praxis commit `e74fa2c5` removed
//! the byte side-channel from `OwlLens::Target`, reducing PutGet to
//! the categorical idempotence `read ∘ write ∘ read ≡ read`. That
//! refactor was preceded by an analogous structural-content audit
//! that quantified what the OWL typed view lost relative to its
//! source bytes; this module is the USLM equivalent.
//!
//! ## What this module does NOT do
//!
//! - It does NOT byte-compare. Two USLM bytestreams that round-trip
//!   structurally identical can still differ byte-for-byte
//!   (whitespace, attribute order, XML declaration variants). The
//!   byte-level canonical-form question is W3C XML Canonicalization
//!   1.1 (Boyer & Marcy 2008), which lives in the lens's `canonical`
//!   leg, not here.
//! - It does NOT count attributes or text-node characters. Attribute
//!   coverage is a separate audit dimension — typed view may project a
//!   subset of attributes (the `<note topic="...">` projection drops
//!   the `style` attribute, for example). Counting attributes is a
//!   Phase-2 concern when `write_uslm` is wired and we can compare
//!   attribute sets at write time.
//!
//! ## Citations
//!
//! - **Cowan, J.; Tobin, R. (eds.) (2004)** *XML Information Set*,
//!   W3C Recommendation 4 February 2004 (2nd Ed.). Defines the
//!   namespace-URI + local-name pair as the element identity.
//!   <https://www.w3.org/TR/xml-infoset/>.
//! - **Bray, T.; Paoli, J.; Sperberg-McQueen, C.M.; Maler, E.; Yergeau, F.
//!   (eds.) (2008)** *Extensible Markup Language (XML) 1.0 (Fifth Edition)*,
//!   W3C Recommendation 26 November 2008, §3 Logical Structures.
//!   <https://www.w3.org/TR/xml/>.
//! - **U.S. House Office of the Law Revision Counsel** —
//!   *USLM XML User Guide and Schema (USLM-1.0.18.xsd)*.
//!   <https://uscode.house.gov/uslm/>.
//! - **Foster, J. N.; Greenwald, M. B.; Moore, J. T.; Pierce, B. C.;
//!   Schmitt, A. (2007)** "Combinators for Bidirectional Tree
//!   Transformations", *ACM TOPLAS* 29(3) Article 17, §3, Definition 3.2 — the
//!   well-behaved-lens laws the gap report frames.
//! - **Bancilhon, F.; Spyratos, N. (1981)** "Update Semantics of
//!   Relational Views", *ACM TODS* 6(4) — the constant-complement
//!   theorem this audit motivates removing.

#[allow(unused_imports)]
use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec::Vec,
};

use alloc::collections::BTreeMap;

use crate::social::software::markup::xml::ontology::{XmlDocument, XmlElement, XmlNode};

use super::super::corpus::{
    HierarchyNode, UsCodeContainer, UsCodeMeta, UsCodeNote, UsCodeNotesBlock, UsCodeSection,
    UsCodeSubdivision, UsCodeTable, UsCodeTitle, UsCodeToc,
};

/// Histogram of XML element occurrences keyed by `(namespace_uri,
/// local_name)`.
///
/// `namespace_uri` is `None` for elements that appeared without any
/// in-scope default-namespace binding; otherwise it carries the
/// in-scope URI (USLM = `"http://xml.house.gov/schemas/uslm/1.0"`,
/// Dublin Core = `"http://purl.org/dc/elements/1.1/"`, etc.).
///
/// The histogram is `BTreeMap`-backed so report outputs are stable in
/// alphabetical order — necessary for the structural-content audit's
/// deterministic emit (W3C XML Infoset §1: the namespace-URI +
/// local-name pair is the element identity).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ElementHistogram {
    counts: BTreeMap<(Option<String>, String), usize>,
}

impl ElementHistogram {
    /// Build an empty histogram.
    #[must_use]
    pub fn new() -> Self {
        Self {
            counts: BTreeMap::new(),
        }
    }

    /// Increment the count for one element.
    pub fn bump(&mut self, namespace: Option<&str>, local: &str) {
        let key = (namespace.map(ToString::to_string), local.to_string());
        *self.counts.entry(key).or_insert(0) += 1;
    }

    /// All entries in stable order.
    pub fn entries(&self) -> impl Iterator<Item = (&(Option<String>, String), &usize)> {
        self.counts.iter()
    }

    /// Count for one element name.
    #[must_use]
    pub fn get(&self, namespace: Option<&str>, local: &str) -> usize {
        let key = (namespace.map(ToString::to_string), local.to_string());
        self.counts.get(&key).copied().unwrap_or(0)
    }

    /// Sum of all counts.
    #[must_use]
    pub fn total(&self) -> usize {
        self.counts.values().sum()
    }

    /// Distinct element-name count.
    #[must_use]
    pub fn distinct(&self) -> usize {
        self.counts.len()
    }
}

/// One row of the audit diff: raw count vs typed count for a single
/// element name, with the `gap` field being `raw - typed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapRow {
    pub namespace: Option<String>,
    pub local: String,
    pub raw: usize,
    pub typed: usize,
    pub gap: i64,
}

/// Audit result over one USLM XML byte stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralAudit {
    /// Histogram from the raw XML tree.
    pub raw: ElementHistogram,
    /// Histogram from the typed view ([`UsCodeTitle`]).
    pub typed: ElementHistogram,
    /// Per-element gap rows in alphabetical order.
    pub gaps: Vec<GapRow>,
}

impl StructuralAudit {
    /// Element names with a non-zero gap (raw count > typed count).
    /// These are the structural drops the typed view fails to capture.
    pub fn dropped_elements(&self) -> impl Iterator<Item = &GapRow> {
        self.gaps.iter().filter(|g| g.gap > 0)
    }

    /// Total dropped element count across every element name.
    #[must_use]
    pub fn total_dropped(&self) -> i64 {
        self.dropped_elements().map(|g| g.gap).sum()
    }
}

// ---------------------------------------------------------------------------
// Raw XML histogram — walk the parsed XmlDocument tree.
// ---------------------------------------------------------------------------

/// In-scope namespace bindings per W3C XML Namespaces 1.0 §6.
///
/// Tracks the default-namespace URI (from `xmlns="…"`) and the
/// prefix→URI map (from `xmlns:p="…"`) accumulated on the way down
/// the tree. A prefixed element's namespace URI is the URI bound to
/// its prefix in the nearest enclosing scope; an unprefixed element
/// is in the default namespace (or none, if no default is declared).
#[derive(Debug, Clone, Default)]
struct NsCtx {
    default_uri: Option<String>,
    prefix_bindings: Vec<(String, String)>,
}

impl NsCtx {
    fn enter(&self, elem: &XmlElement) -> Self {
        let mut next = self.clone();
        for ns in &elem.namespaces {
            match &ns.prefix {
                None => next.default_uri = Some(ns.uri.clone()),
                Some(p) => next.prefix_bindings.push((p.clone(), ns.uri.clone())),
            }
        }
        next
    }

    /// Resolve the namespace URI for `elem` per W3C XML Namespaces 1.0
    /// §6.2. Prefixed elements bind to their prefix's URI (most recent
    /// binding wins); unprefixed elements inherit the default.
    fn resolve(&self, elem: &XmlElement) -> Option<String> {
        if let Some(prefix) = &elem.name.prefix {
            // Most-recently-pushed binding wins.
            return self
                .prefix_bindings
                .iter()
                .rev()
                .find(|(p, _)| p == prefix)
                .map(|(_, u)| u.clone())
                .or_else(|| {
                    // Fall back to the element's own `namespace` field
                    // if the reader populated it directly.
                    elem.namespace
                        .as_ref()
                        .filter(|n| n.prefix.as_deref() == Some(prefix.as_str()))
                        .map(|n| n.uri.clone())
                });
        }
        // Unprefixed element — default namespace.
        self.default_uri
            .clone()
            .or_else(|| elem.namespace.as_ref().map(|n| n.uri.clone()))
    }
}

/// Walk `doc.root` and produce the [`ElementHistogram`] of every
/// element in document order. Each element contributes one count;
/// element identity is `(in-scope namespace URI, local name)` per W3C
/// XML Infoset §1.
fn histogram_from_xml(doc: &XmlDocument) -> ElementHistogram {
    let mut hist = ElementHistogram::new();
    walk_raw(&doc.root, &NsCtx::default(), &mut hist);
    hist
}

fn walk_raw(elem: &XmlElement, ctx: &NsCtx, hist: &mut ElementHistogram) {
    // Element's URI is resolved against the *outer* scope before any
    // xmlns declarations on the element itself take effect (W3C XML
    // Namespaces 1.0 §6.1 — the element's own xmlns declarations
    // apply to its content, not to itself).
    //
    // BUT in practice the default-namespace declaration is exempt:
    // an unprefixed element with an `xmlns="..."` declaration on
    // itself IS in the namespace it declares. The reader populates
    // `elem.namespace.uri` accordingly; use it for the unprefixed
    // case if the outer scope had nothing.
    let outer_ns = ctx.resolve(elem);
    let element_self_default = elem
        .namespaces
        .iter()
        .find(|n| n.prefix.is_none())
        .map(|n| n.uri.clone());
    let ns = if elem.name.prefix.is_none() && outer_ns.is_none() {
        element_self_default.or(outer_ns)
    } else if elem.name.prefix.is_none() && element_self_default.is_some() {
        // Element with its own default namespace declaration —
        // that declaration is in scope for the element itself.
        element_self_default
    } else {
        outer_ns
    };
    hist.bump(ns.as_deref(), &elem.name.local);
    let child_ctx = ctx.enter(elem);
    for child in &elem.children {
        if let XmlNode::Element(e) = child {
            walk_raw(e, &child_ctx, hist);
        }
    }
}

// ---------------------------------------------------------------------------
// Typed histogram — walk the UsCodeTitle materialised by read_uslm_title.
//
// The typed projection materialises ONE node per typed value. Each
// `UsCodeSection` increments `(USLM, "section")`; each
// `UsCodeSubdivision` increments the kind's element name (per the
// XSD-grounded SubdivisionKind taxonomy); each `UsCodeRef` increments
// `(USLM, "ref")`; etc. This count gives us *what survived the
// typed-view projection*.
//
// The mapping from typed value to `(namespace, local_name)` is the
// projection of LRC USLM User Guide §V's element vocabulary onto the
// loaded XSD's `<xsd:element>` declarations (W3C XSD 1.1 Part 1 §3.3).
// ---------------------------------------------------------------------------

const USLM_NS: &str = "http://xml.house.gov/schemas/uslm/1.0";

/// Public re-export of the USLM namespace URI for tests in sibling
/// modules. Per W3C XML Namespaces 1.0 §6 element identity is
/// (namespace URI, local name); tests that assert on the histogram
/// need the URI half.
#[cfg(test)]
pub(super) const USLM_NS_FOR_TEST: &str = USLM_NS;

/// Build the typed histogram by walking the [`UsCodeTitle`] tree and
/// emitting one count per materialised node.
fn histogram_from_typed(title: &UsCodeTitle) -> ElementHistogram {
    let mut hist = ElementHistogram::new();
    bump_uslm(&mut hist, "uscDoc"); // implicit root the lens reads through
    bump_uslm(&mut hist, "main"); // implicit wrapper
    bump_uslm(&mut hist, "title");
    // Title-level `<num>` + `<heading>` (the lens reads these
    // attribute-bearing leaves; count one per).
    bump_uslm(&mut hist, "num");
    bump_uslm(&mut hist, "heading");
    for node in &title.hierarchy {
        bump_hierarchy(&mut hist, node);
    }
    for nb in &title.notes_blocks {
        bump_notes_block(&mut hist, nb);
    }
    for n in &title.bare_notes {
        bump_note(&mut hist, n);
    }
    for _h in &title.headers {
        bump_uslm(&mut hist, "header");
    }
    for s in &title.signatures {
        bump_uslm(&mut hist, "signature");
        for _n in &s.names {
            bump_uslm(&mut hist, "name");
        }
    }
    if let Some(meta) = &title.meta {
        bump_meta(&mut hist, meta);
    }
    for t in &title.tocs {
        bump_toc(&mut hist, t);
    }
    for tab in &title.tables {
        bump_table(&mut hist, tab);
    }
    hist
}

fn bump_uslm(hist: &mut ElementHistogram, local: &str) {
    hist.bump(Some(USLM_NS), local);
}

fn bump_hierarchy(hist: &mut ElementHistogram, node: &HierarchyNode) {
    match node {
        HierarchyNode::Section(s) => bump_section(hist, s),
        HierarchyNode::Container(c) => bump_container(hist, c),
    }
}

fn bump_container(hist: &mut ElementHistogram, c: &UsCodeContainer) {
    // ContainerKind names: subtitle/part/subpart/chapter/subchapter/
    // division/article/subarticle/etc. — projected from the loaded
    // USLM XSD's substitutionGroup="level" members per the LRC USLM
    // User Guide §V hierarchy.
    bump_uslm(hist, container_kind_name(c));
    bump_uslm(hist, "num");
    bump_uslm(hist, "heading");
    for child in &c.children {
        bump_hierarchy(hist, child);
    }
    for nb in &c.notes_blocks {
        bump_notes_block(hist, nb);
    }
    for n in &c.bare_notes {
        bump_note(hist, n);
    }
    for t in &c.tocs {
        bump_toc(hist, t);
    }
}

fn container_kind_name(c: &UsCodeContainer) -> &'static str {
    use crate::social::software::markup::xml::uslm::corpus::ContainerKind;
    match c.kind {
        ContainerKind::Subtitle => "subtitle",
        ContainerKind::Part => "part",
        ContainerKind::Subpart => "subpart",
        ContainerKind::Chapter => "chapter",
        ContainerKind::Subchapter => "subchapter",
    }
}

fn bump_section(hist: &mut ElementHistogram, s: &UsCodeSection) {
    bump_uslm(hist, "section");
    bump_uslm(hist, "num");
    if s.num_footnote.is_some() {
        // The footnote inside `<num>` is a typed `<note type="footnote">`
        // per the LRC's duplicate-number disambiguation convention.
        bump_uslm(hist, "note");
    }
    bump_uslm(hist, "heading");
    if s.chapeau.is_some() {
        bump_uslm(hist, "chapeau");
    }
    if s.content.is_some() {
        bump_uslm(hist, "content");
    }
    for child in &s.children {
        bump_subdivision(hist, child);
    }
    for r in &s.refs {
        let _ = r;
        bump_uslm(hist, "ref");
    }
    for nb in &s.notes_blocks {
        bump_notes_block(hist, nb);
    }
    for n in &s.bare_notes {
        bump_note(hist, n);
    }
    for sc in &s.source_credits {
        let _ = sc;
        bump_uslm(hist, "sourceCredit");
        for r in &sc.refs {
            let _ = r;
            bump_uslm(hist, "ref");
        }
        for d in &sc.dates {
            let _ = d;
            bump_uslm(hist, "date");
        }
    }
    for _c in &s.continuations {
        bump_uslm(hist, "continuation");
    }
    for db in &s.def_blocks {
        bump_uslm(hist, "def");
        for _t in &db.terms {
            bump_uslm(hist, "term");
        }
    }
    for _m in &s.markers {
        bump_uslm(hist, "marker");
    }
    for a in &s.amendments {
        use crate::social::software::markup::xml::uslm::corpus::UsCodeAmendmentKind;
        bump_uslm(
            hist,
            match a.kind {
                UsCodeAmendmentKind::Insertion => "ins",
                UsCodeAmendmentKind::Deletion => "del",
            },
        );
    }
}

fn bump_subdivision(hist: &mut ElementHistogram, s: &UsCodeSubdivision) {
    use crate::social::software::markup::xml::uslm::corpus::SubdivisionKind;
    bump_uslm(
        hist,
        match s.kind {
            SubdivisionKind::Subsection => "subsection",
            SubdivisionKind::Paragraph => "paragraph",
            SubdivisionKind::Subparagraph => "subparagraph",
            SubdivisionKind::Clause => "clause",
            SubdivisionKind::Subclause => "subclause",
            SubdivisionKind::Item => "item",
            SubdivisionKind::Subitem => "subitem",
        },
    );
    bump_uslm(hist, "num");
    if s.heading.is_some() {
        bump_uslm(hist, "heading");
    }
    if s.chapeau.is_some() {
        bump_uslm(hist, "chapeau");
    }
    if s.content.is_some() {
        bump_uslm(hist, "content");
    }
    for child in &s.children {
        bump_subdivision(hist, child);
    }
    for r in &s.refs {
        let _ = r;
        bump_uslm(hist, "ref");
    }
    for db in &s.def_blocks {
        bump_uslm(hist, "def");
        for _t in &db.terms {
            bump_uslm(hist, "term");
        }
    }
    for _m in &s.markers {
        bump_uslm(hist, "marker");
    }
    for a in &s.amendments {
        use crate::social::software::markup::xml::uslm::corpus::UsCodeAmendmentKind;
        bump_uslm(
            hist,
            match a.kind {
                UsCodeAmendmentKind::Insertion => "ins",
                UsCodeAmendmentKind::Deletion => "del",
            },
        );
    }
}

fn bump_notes_block(hist: &mut ElementHistogram, nb: &UsCodeNotesBlock) {
    bump_uslm(hist, "notes");
    if nb.heading.is_some() {
        bump_uslm(hist, "heading");
    }
    for n in &nb.notes {
        bump_note(hist, n);
    }
}

fn bump_note(hist: &mut ElementHistogram, n: &UsCodeNote) {
    bump_uslm(hist, "note");
    if n.heading.is_some() {
        bump_uslm(hist, "heading");
    }
    for r in &n.refs {
        let _ = r;
        bump_uslm(hist, "ref");
    }
    for qc in &n.quoted_contents {
        bump_uslm(hist, "quotedContent");
        for r in &qc.refs {
            let _ = r;
            bump_uslm(hist, "ref");
        }
        for sr in &qc.section_refs {
            let _ = sr;
            bump_uslm(hist, "section");
        }
    }
    for d in &n.dates {
        let _ = d;
        bump_uslm(hist, "date");
    }
}

fn bump_meta(hist: &mut ElementHistogram, m: &UsCodeMeta) {
    const DC_NS: &str = "http://purl.org/dc/elements/1.1/";
    const DCTERMS_NS: &str = "http://purl.org/dc/terms/";
    bump_uslm(hist, "meta");
    if m.title.is_some() {
        hist.bump(Some(DC_NS), "title");
    }
    if m.doc_type.is_some() {
        hist.bump(Some(DC_NS), "type");
    }
    if m.publisher.is_some() {
        hist.bump(Some(DC_NS), "publisher");
    }
    if m.creator.is_some() {
        hist.bump(Some(DC_NS), "creator");
    }
    if m.date.is_some() {
        hist.bump(Some(DC_NS), "date");
    }
    if m.identifier.is_some() {
        hist.bump(Some(DC_NS), "identifier");
    }
    if m.language.is_some() {
        hist.bump(Some(DC_NS), "language");
    }
    if m.format.is_some() {
        hist.bump(Some(DC_NS), "format");
    }
    if m.rights.is_some() {
        hist.bump(Some(DC_NS), "rights");
    }
    if m.source.is_some() {
        hist.bump(Some(DC_NS), "source");
    }
    for (local, _v) in &m.other_dc {
        hist.bump(Some(DC_NS), local);
    }
    if m.doc_number.is_some() {
        bump_uslm(hist, "docNumber");
    }
    if m.doc_publication_name.is_some() {
        bump_uslm(hist, "docPublicationName");
    }
    for _p in &m.properties {
        bump_uslm(hist, "property");
    }
    if m.dcterms_created.is_some() {
        hist.bump(Some(DCTERMS_NS), "created");
    }
    if m.dcterms_modified.is_some() {
        hist.bump(Some(DCTERMS_NS), "modified");
    }
    for (local, _v) in &m.dcterms_other {
        hist.bump(Some(DCTERMS_NS), local);
    }
}

fn bump_toc(hist: &mut ElementHistogram, t: &UsCodeToc) {
    bump_uslm(hist, "toc");
    for item in &t.items {
        bump_uslm(hist, "tocItem");
        // tocItem cells were projected as concatenated `<column>` text;
        // we can't recover the per-column count from the typed text
        // alone, but the typed projection collapses them — count once
        // per item as a structural lower bound.
        for r in &item.refs {
            let _ = r;
            bump_uslm(hist, "ref");
        }
    }
}

fn bump_table(hist: &mut ElementHistogram, t: &UsCodeTable) {
    bump_uslm(hist, "table");
    if !t.header_rows.is_empty() {
        bump_uslm(hist, "thead");
    }
    if !t.body_rows.is_empty() {
        bump_uslm(hist, "tbody");
    }
    for r in &t.header_rows {
        bump_uslm(hist, "tr");
        for c in &r.cells {
            use crate::social::software::markup::xml::uslm::corpus::UsCodeTableCellKind;
            bump_uslm(
                hist,
                match c.kind {
                    UsCodeTableCellKind::Header => "th",
                    UsCodeTableCellKind::Data => "td",
                },
            );
        }
    }
    for r in &t.body_rows {
        bump_uslm(hist, "tr");
        for c in &r.cells {
            use crate::social::software::markup::xml::uslm::corpus::UsCodeTableCellKind;
            bump_uslm(
                hist,
                match c.kind {
                    UsCodeTableCellKind::Header => "th",
                    UsCodeTableCellKind::Data => "td",
                },
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Audit entry point
// ---------------------------------------------------------------------------

/// Audit the structural-content faithfulness of the USLM typed view
/// against the source XML bytes.
///
/// Per the Phase-1 design above: builds a raw histogram and a typed
/// histogram, then diffs them per element name. Non-zero gap rows
/// identify what the typed view drops.
///
/// Returns an error iff the input bytes can't be parsed as XML or
/// can't be projected to a [`UsCodeTitle`] — both rejections are
/// genuine signals of an upstream problem, not audit failures.
pub fn audit_structural_content(xml_bytes: &[u8]) -> Result<StructuralAudit, AuditError> {
    let s = core::str::from_utf8(xml_bytes).map_err(|e| AuditError::NotUtf8(format!("{e}")))?;
    let doc = crate::social::software::markup::xml::reader::read_xml(s)
        .map_err(|e| AuditError::ParseXml(e.message))?;
    let raw = histogram_from_xml(&doc);

    let title = super::leaf_readers::read_uslm_title(s).map_err(AuditError::ParseUslm)?;
    let typed = histogram_from_typed(&title);

    // Diff every key that appears in either histogram.
    let mut keys: alloc::collections::BTreeSet<(Option<String>, String)> =
        alloc::collections::BTreeSet::new();
    for ((ns, local), _) in raw.entries() {
        keys.insert((ns.clone(), local.clone()));
    }
    for ((ns, local), _) in typed.entries() {
        keys.insert((ns.clone(), local.clone()));
    }
    let mut gaps = Vec::with_capacity(keys.len());
    for (ns, local) in keys {
        let r = raw.get(ns.as_deref(), &local);
        let t = typed.get(ns.as_deref(), &local);
        gaps.push(GapRow {
            namespace: ns,
            local,
            raw: r,
            typed: t,
            gap: r as i64 - t as i64,
        });
    }

    Ok(StructuralAudit { raw, typed, gaps })
}

/// Audit error.
#[derive(Debug)]
pub enum AuditError {
    NotUtf8(String),
    ParseXml(String),
    ParseUslm(crate::social::software::markup::xml::uslm::corpus::UslmReadError),
}

impl core::fmt::Display for AuditError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotUtf8(m) => write!(f, "structural audit: not UTF-8: {m}"),
            Self::ParseXml(m) => write!(f, "structural audit: XML parse: {m}"),
            Self::ParseUslm(e) => write!(f, "structural audit: USLM read: {e}"),
        }
    }
}

impl std::error::Error for AuditError {}

// ---------------------------------------------------------------------------
// uslm_equivalent — structural-equality predicate over UsCodeTitle.
//
// Two UsCodeTitle values are uslm_equivalent iff they project to the
// same typed structure. This is the OWL refactor's `owl_equivalent`
// counterpart — the equality predicate the Phase-3 PutGet law uses
// in place of `==`, so trivial reorderings (e.g. of unordered
// collections) don't break round-trip.
//
// As of Phase 1 the predicate is exactly `PartialEq` — `UsCodeTitle`
// derives PartialEq and every field is order-preserving, so structural
// equality and `==` coincide. The function exists as a named extension
// point: when Phase 2's `write_uslm` introduces canonicalization that
// re-orders e.g. notes alphabetically, the predicate can be relaxed
// to set-equality on the affected fields without churning callers.
// ---------------------------------------------------------------------------

/// Structural equivalence of two USLM titles.
///
/// Today this is `==` on [`UsCodeTitle`]; the named function exists so
/// Phase 2 / Phase 3 can relax to permutation-tolerant equality on
/// unordered fields without churning callers.
#[must_use]
pub fn uslm_equivalent(a: &UsCodeTitle, b: &UsCodeTitle) -> bool {
    a == b
}

// ---------------------------------------------------------------------------
// Audit summary rendering — a deterministic ASCII report for the
// Phase-1 test that runs across every registered USC title.
// ---------------------------------------------------------------------------

/// Render an audit as a stable multi-line ASCII report. Suitable for
/// `eprintln!` in CI logs and for snapshot tests.
#[must_use]
pub fn render_audit(name: &str, audit: &StructuralAudit) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "=== uslm structural audit: {name} ===\n  raw distinct names = {}\n  raw total elements = {}\n  typed distinct names = {}\n  typed total elements = {}\n  total dropped (raw - typed, where raw > typed) = {}\n",
        audit.raw.distinct(),
        audit.raw.total(),
        audit.typed.distinct(),
        audit.typed.total(),
        audit.total_dropped(),
    ));
    out.push_str("  per-element diff:\n");
    for g in &audit.gaps {
        let ns = g.namespace.as_deref().unwrap_or("(none)");
        out.push_str(&format!(
            "    {{{ns}}}{}  raw={}  typed={}  gap={:+}\n",
            g.local, g.raw, g.typed, g.gap
        ));
    }
    out
}
