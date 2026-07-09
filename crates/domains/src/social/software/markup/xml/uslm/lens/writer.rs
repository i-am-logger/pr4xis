//! USLM structural writer (slices U1 + U2) — the XML-tree-level inverse of
//! [`read_uslm_title`], the USC analogue of the
//! WN-LMF [`write_wordnet_document`](crate::social::software::markup::xml::lmf::writer::write_wordnet_document).
//!
//! [`write_uslm`] folds a typed [`UsCodeTitle`] back onto the generic XML
//! ontology ([`XmlDocument`] / [`XmlElement`] / [`XmlNode`]), regenerating the
//! element BACKBONE — element names, pre-order, and the EXACT ordered
//! mixed-content child sequence (text-run ↔ inline-element interleaving) — so
//! that, paired with the generic SourceSyntax complement, it closes a
//! byte-exact graph-faithful `put` (Foster, Greenwald, Moore, Pierce & Schmitt
//! 2007 ACM TOPLAS 29(3) §3, Definition 3.2).
//!
//! # Layering (L1 of the serialized reverse lens)
//!
//! This is the structural fold only. It regenerates the elements + leaf text
//! the typed model carries; the BYTE residue the W3C Information Set (Cowan &
//! Tobin 2004) does not carry — attribute ORDER/coverage, the
//! `<?xml-stylesheet?>` prolog PI, the `<!DOCTYPE>`, namespace declarations,
//! and the §2.4 \[14\] inter-element white-space (indentation) — is NOT this
//! writer's job. It belongs in the generic complement (the
//! [`DocumentResidue`] and [`RegeneratedComplement`]), captured by
//! [`diff_content_whitespace`].
//! What this writer MUST get right is the element backbone and child sequence
//! (which Text vs element, in exact source order) — otherwise the backbone
//! diff fails closed.
//!
//! # Coverage (slices U1 + U2)
//!
//! **U1** covers the FLAT-section content model: a `<section>` of `<num>` +
//! `<heading>` + `<content>` (with a block `<p>`) + `<sourceCredit>` — the
//! last being TRUE MIXED CONTENT (`<ref>` / `<date>` interleaved with literal
//! punctuation).
//!
//! **U2** adds the USLM SUBDIVISION backbone most USC sections actually carry:
//! the `<subsection>` / `<paragraph>` / `<subparagraph>` / `<clause>` /
//! `<subclause>` / `<item>` / `<subitem>` recursion. Each subdivision
//! regenerates its own `<num>` + optional `<heading>` / `<chapeau>` + EITHER a
//! leaf `<content>` OR nested child subdivisions, all from the
//! [`UsCodeMixed`] semantic trees the reader now captures on
//! [`UsCodeSubdivision`] (the §-level chapeau is covered too). The element
//! name comes from [`SubdivisionKind::tag`] — the inverse of the reader's
//! XSD-grounded `from_xsd_element` dispatch — never a call-site literal.
//!
//! **U3** adds the USLM NOTES backbone almost every published § carries: the
//! `<notes>` container and its `<note>` children (the editorial / statutory /
//! cross-reference note blocks that follow `<sourceCredit>`). A `<note>` body
//! is TRUE MIXED CONTENT — an ordered sequence of block children (`<heading>`,
//! `<p>`, and in other titles `<num>` / `<table>` / `<signature>` /
//! `<quotedContent>`) each itself mixed content (a `<heading>` carries `<b>`; a
//! `<p>` interleaves literal text with `<ref>` / `<date>` / `<i>`). The whole
//! note body regenerates from the [`UsCodeNote::body_mixed`] semantic tree the
//! reader now captures — node-for-node, in source order — exactly as the
//! `<sourceCredit>` / `<content>` mixed trees do (U1 / U2). The notes follow
//! `<sourceCredit>` at the § level (LRC USLM XML User Guide §V).
//!
//! The remaining USLM vocabulary (tables as a typed family, def / marker /
//! amendment markup, the `<continuation>` flush-text family — absent from
//! Title 1, so left fail-closed rather than written un-proven — and the full
//! `<uscDoc>` wrapper) is the next slice; a section or subdivision that
//! exercises an uncovered family surfaces as a
//! [`UslmWriteError::UncoveredFamily`] (named by family) rather than a silent
//! drop — fail-closed at the exact boundary.
//!
//! # Child order
//!
//! [`diff_element`](crate::social::software::markup::xml::parser::source_syntax)
//! matches children strictly POSITIONALLY (no reorder species), so this writer
//! emits each element's children in EXACT source order. That order is the
//! canonical USLM level order `num, heading?, chapeau?, (content | child
//! levels)`, with `<sourceCredit>` last at the § level (LRC USLM XML User Guide
//! §V "Level Structure") — reconstructed from the typed model directly, with NO
//! `ChildOrder` residue added to the generic complement. The gates over the
//! real Title 1 §§ 8 and 201 (the latter exercising the
//! `<subsection>` → `<paragraph>` recursion) prove that order is faithful for
//! the covered families.
//!
//! # Citations
//!
//! - **Foster, J. N.; Greenwald, M. B.; Moore, J. T.; Pierce, B. C.; Schmitt,
//!   A. (2007)** — ACM TOPLAS 29(3) Article 17 §3, Definition 3.2 (well-behaved lens `put`).
//! - **Cowan, J.; Tobin, R. (2004)** — XML Information Set (the items the
//!   typed model carries; attribute order / white-space are not among them).
//! - **Bray et al. (2008)** — XML 1.0 Fifth Edition §3.2.2 (Mixed Content).
//! - **U.S. House Office of the Law Revision Counsel** — *USLM XML User Guide
//!   and Schema*. <https://uscode.house.gov/uslm/>.

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec, vec::Vec};

use super::super::corpus::{
    InlineKind, SubdivisionKind, UsCodeContentNode, UsCodeMixed, UsCodeNote, UsCodeNotesBlock,
    UsCodeSection, UsCodeSourceCredit, UsCodeSubdivision, UsCodeTitle,
};
use super::read_uslm_title;
use crate::social::software::markup::xml::ontology::{
    XmlAttribute, XmlDocument, XmlElement, XmlName, XmlNode,
};
use crate::social::software::markup::xml::parser::grammar::{
    XmlParseError, parse_document_capturing,
};
use crate::social::software::markup::xml::parser::serializer::serialize_document_exact;
use crate::social::software::markup::xml::parser::source_syntax::{
    DocumentResidue, RegeneratedComplement, RegeneratedComplementError, SyntaxDecisions,
    diff_content_whitespace, reapply_regenerated_complement,
};

/// Errors a USLM structural write can surface. A family the slice does not yet
/// regenerate is reported (never silently dropped), so the byte-exact gate
/// fails LOUD at the exact uncovered element rather than fabricating bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UslmWriteError {
    /// The typed title carried a structure the writer does not regenerate yet
    /// (a notes block, a table, a `<def>` / `<marker>` / amendment markup, the
    /// `<uscDoc>` wrapper, …). Carries the family name so the gate names the
    /// next slice. (Slices U1 + U2 cover the §/subdivision text backbone.)
    UncoveredFamily {
        /// The uncovered USLM element family (e.g. `"subsection"`, `"notes"`).
        family: String,
        /// The section identifier where it was found, for diagnosis.
        section: String,
    },
}

impl core::fmt::Display for UslmWriteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UncoveredFamily { family, section } => write!(
                f,
                "write_uslm does not yet regenerate the <{family}> family \
                 (at {section}); that is the next slice"
            ),
        }
    }
}

impl std::error::Error for UslmWriteError {}

/// Write a [`UsCodeTitle`] back to a USLM [`XmlDocument`] — the XML-tree-level
/// inverse of [`read_uslm_title`].
///
/// Targets the SECTION-SLICE shape `read_uslm_title` produces for a bare
/// `<section>` document (root = the section): the title's single `sections`
/// entry is regenerated as the document root, recursing through its
/// subdivisions (slice U2). The canonical `version="1.0" encoding="UTF-8"`
/// prolog mirrors the reader's XML declaration handling.
///
/// Returns [`UslmWriteError::UncoveredFamily`] when the section carries a
/// structure the writer does not regenerate — the honest-partial boundary that
/// keeps the backbone diff fail-closed instead of dropping content.
pub fn write_uslm(title: &UsCodeTitle) -> Result<XmlDocument, UslmWriteError> {
    // SLICE U4 — the full `<uscDoc>` document wrapper. When the reader captured
    // the `<uscDoc>` root's backbone (`uscdoc_mixed`), regenerate the WHOLE
    // document from it: the `<meta>` block, `<main>` → `<title>` → its `<num>` /
    // `<heading>` / title-level notes / `<toc>` / the `<chapter>` hierarchy
    // containers grouping the multi-section list. Every node regenerates from the
    // semantic mixed tree node-for-node — the same machinery the §-level mixed
    // content uses (`mixed_element` / `content_node`), so the document is
    // backbone-faithful end-to-end. The root start-tag's interleaved
    // `xmlns`/attribute sequence and the inter-element white-space are restored by
    // the generic complement (`AttributeOverrides` + `start_tag_order` +
    // `ContentWhitespace`), so the regenerated `<uscDoc>` carries no attributes
    // here.
    if let Some(uscdoc) = &title.uscdoc_mixed {
        let root = element("uscDoc", Vec::new(), mixed_children(uscdoc));
        return Ok(XmlDocument {
            version: "1.0".to_string(),
            encoding: Some("UTF-8".to_string()),
            doctype: None,
            root,
        });
    }

    // The bare-`<section>` slice document shape (no `<uscDoc>` wrapper): exactly
    // one section, emitted as the root, regenerated from its typed model (slices
    // U1–U3).
    let [section] = title.sections.as_slice() else {
        return Err(UslmWriteError::UncoveredFamily {
            family: "title".to_string(),
            section: title.identifier.clone(),
        });
    };
    let root = section_element(section)?;
    Ok(XmlDocument {
        version: "1.0".to_string(),
        encoding: Some("UTF-8".to_string()),
        doctype: None,
        root,
    })
}

/// Regenerate a `<section>` element backbone from a [`UsCodeSection`].
///
/// Emits the canonical USLM level child order
/// `num, heading, chapeau?, (content | subdivisions...), sourceCredit*,
/// notes*` — a flat prose § carries `content`; a branch § carries
/// `<subsection>` children (slice U2), recursing via [`subdivision_element`];
/// the `<notes>` blocks (slice U3) follow `<sourceCredit>`. A section with a
/// structure the writer does not regenerate (a `<continuation>` flush, a
/// `<def>` / `<marker>` / amendment markup, the `<uscDoc>` wrapper) returns
/// [`UslmWriteError::UncoveredFamily`] so the gate fails closed at that family.
fn section_element(section: &UsCodeSection) -> Result<XmlElement, UslmWriteError> {
    reject_uncovered(section)?;

    let mut children: Vec<XmlNode> = Vec::new();

    // <num value="…">§ N.</num> — the `value` attribute plus the VISIBLE text
    // leaf the reader captured verbatim into `num_text`.
    children.push(XmlNode::Element(num_element(
        &section.num,
        &section.num_text,
    )));

    // <heading>…</heading> — regenerated from the semantic mixed tree.
    children.push(XmlNode::Element(mixed_element(
        "heading",
        &section.heading_mixed,
    )));

    // <chapeau>…</chapeau> — a §-level branch's introductory phrase before its
    // subsections; emitted only when the source carried one (most flat sections
    // do not). Same canonical position (after heading, before children) as the
    // subdivision-level chapeau.
    if let Some(chapeau) = &section.chapeau_mixed {
        children.push(XmlNode::Element(mixed_element("chapeau", chapeau)));
    }

    // <content>…</content> — regenerated from its mixed tree (carries the
    // block <p> in position). Emitted only when the source carried one. A USLM
    // level is EITHER a leaf (carries `<content>`) OR a branch (carries
    // `<subsection>` children); the LRC pl-119-90 sections never carry both, so
    // these two emit in their canonical source positions without interleaving.
    if let Some(content) = &section.content_mixed {
        children.push(XmlNode::Element(mixed_element("content", content)));
    }

    // <subsection>… — the SUBDIVISION BACKBONE (slice U2). Each child
    // subdivision regenerates recursively in source order (num, heading?,
    // chapeau?, then its own children or content), emitted between the §'s
    // leaf content and its <sourceCredit> per the canonical USLM level order
    // (LRC USLM XML User Guide §V).
    for child in &section.children {
        children.push(XmlNode::Element(subdivision_element(child)?));
    }

    // <sourceCredit>…</sourceCredit> — TRUE MIXED CONTENT regenerated from its
    // exact ordered tree (literal punctuation interleaved with <ref>/<date>).
    for credit in &section.source_credits {
        children.push(XmlNode::Element(source_credit_element(credit)));
    }

    // <notes>… — the NOTES BACKBONE (slice U3). The editorial / statutory /
    // cross-reference note blocks follow <sourceCredit> at the § level (LRC
    // USLM XML User Guide §V). Each <notes> container regenerates its <note>
    // children from their semantic body trees, node-for-node.
    for block in &section.notes_blocks {
        children.push(XmlNode::Element(notes_block_element(block, section)?));
    }

    // Bare <note> children (a <note> directly under <section>, not inside a
    // <notes> wrapper — USLM allows both forms; absent from LRC Title 1 §§ but
    // covered here so a section carrying one regenerates rather than failing
    // closed). Same per-note backbone as inside a <notes> container.
    for note in &section.bare_notes {
        children.push(XmlNode::Element(note_element(note)));
    }

    Ok(element("section", section_attrs(section), children))
}

/// Regenerate one subdivision element (`<subsection>` / `<paragraph>` /
/// `<subparagraph>` / `<clause>` / `<subclause>` / `<item>` / `<subitem>`)
/// from a [`UsCodeSubdivision`] — the slice-U2 recursion.
///
/// Emits the canonical USLM subdivision child order — `num`, then the optional
/// `heading` / `chapeau` mixed trees, then EITHER the nested child
/// subdivisions (a branch) OR the leaf `<content>` (LRC USLM XML User Guide §V
/// "Level Structure"). The element NAME is the subdivision kind's canonical
/// USLM tag ([`SubdivisionKind::tag`], the inverse of the XSD-grounded
/// `SubdivisionKind::from_xsd_element` the reader dispatched on). A subdivision
/// carrying a family slice U2 does not regenerate (def / marker / amendment
/// markup) fails closed via [`reject_uncovered_subdivision`].
fn subdivision_element(sub: &UsCodeSubdivision) -> Result<XmlElement, UslmWriteError> {
    reject_uncovered_subdivision(sub)?;

    let mut children: Vec<XmlNode> = Vec::new();

    // <num value="…">text</num> — the `value` attribute plus the visible text
    // leaf (e.g. `"(a)"`, `"“(1)"`) the reader captured verbatim.
    children.push(XmlNode::Element(num_element(&sub.num, &sub.num_text)));

    // <heading>…</heading> — only when the source carried one (most paragraphs
    // and deeper levels do not). Regenerated from its semantic mixed tree (a
    // subdivision heading carries `<inline class="small-caps">…</inline>`).
    if let Some(heading) = &sub.heading_mixed {
        children.push(XmlNode::Element(mixed_element("heading", heading)));
    }

    // <chapeau>…</chapeau> — the introductory phrase a branch subdivision uses
    // before its enumerated children; emitted only when present.
    if let Some(chapeau) = &sub.chapeau_mixed {
        children.push(XmlNode::Element(mixed_element("chapeau", chapeau)));
    }

    // EITHER the leaf <content> OR the nested child subdivisions — a USLM level
    // is a leaf or a branch, never both (LRC USLM XML User Guide §V).
    if let Some(content) = &sub.content_mixed {
        children.push(XmlNode::Element(mixed_element("content", content)));
    }
    for nested in &sub.children {
        children.push(XmlNode::Element(subdivision_element(nested)?));
    }

    Ok(element(
        subdivision_tag(sub.kind),
        subdivision_attrs(sub),
        children,
    ))
}

/// The canonical USLM tag name for a subdivision kind — the inverse of the
/// reader's XSD-grounded `SubdivisionKind::from_xsd_element`. Reads the name
/// off the typed enum's [`SubdivisionKind::tag`] (the same vocabulary the
/// loaded USLM XSD declares as `substitutionGroup="level"` members), never a
/// hand-coded literal at the call site.
fn subdivision_tag(kind: SubdivisionKind) -> &'static str {
    kind.tag()
}

/// The subdivision's start-tag attributes the typed model carries — only
/// `identifier`, and only when present (the synthetic-section subdivisions
/// inside notes carry no `identifier`). As with the section, the EXACT source
/// attribute sequence (`style` / `class` / `id` / `identifier` order and the
/// metadata attrs the typed model drops) is restored by the generic
/// `AttributeOverrides` complement, so only PRESENCE matters here.
fn subdivision_attrs(sub: &UsCodeSubdivision) -> Vec<XmlAttribute> {
    if sub.identifier.is_empty() {
        Vec::new()
    } else {
        vec![attr("identifier", &sub.identifier)]
    }
}

/// Fail closed when a subdivision carries a family slice U2 does not regenerate
/// yet — mirroring [`reject_uncovered`] at the §-level. Slice U2 covers the
/// subdivision backbone (`num` / `heading` / `chapeau` / `content` + nested
/// child subdivisions); `<def>` / `<marker>` / `<ins>` / `<del>` inside a
/// subdivision are the next slice and surface by family name so the gate fails
/// LOUD rather than emitting a short backbone.
fn reject_uncovered_subdivision(sub: &UsCodeSubdivision) -> Result<(), UslmWriteError> {
    let uncovered = if !sub.def_blocks.is_empty() {
        Some("def")
    } else if !sub.markers.is_empty() {
        Some("marker")
    } else if !sub.amendments.is_empty() {
        Some("ins")
    } else {
        None
    };
    match uncovered {
        Some(family) => Err(UslmWriteError::UncoveredFamily {
            family: family.to_string(),
            section: sub.identifier.clone(),
        }),
        None => Ok(()),
    }
}

/// Fail closed when the section carries a §-level structure family the writer
/// does not regenerate yet. Each is reported by family name so the gate names
/// exactly what the next slice must cover, rather than the writer silently
/// emitting a shorter backbone (which the positional diff would then reject
/// downstream with a less specific message).
fn reject_uncovered(section: &UsCodeSection) -> Result<(), UslmWriteError> {
    // Slice U2 covers the subdivision backbone (`section.children`) + the
    // §-level chapeau; slice U3 covers the `<notes>` / `<note>` backbone
    // (`section.notes_blocks` / `section.bare_notes`) — both regenerate from
    // their semantic mixed trees. The still-uncovered §-level families
    // (`<continuation>` flush text — absent from Title 1, so written un-proven
    // would be dishonest — plus `<def>` / `<marker>` / `<ins>`) remain
    // fail-closed below.
    let uncovered = if !section.continuations.is_empty() {
        Some("continuation")
    } else if !section.def_blocks.is_empty() {
        Some("def")
    } else if !section.markers.is_empty() {
        Some("marker")
    } else if !section.amendments.is_empty() {
        Some("ins")
    } else {
        None
    };
    match uncovered {
        Some(family) => Err(UslmWriteError::UncoveredFamily {
            family: family.to_string(),
            section: section.identifier.clone(),
        }),
        None => Ok(()),
    }
}

/// The section's start-tag attributes the typed model carries — `identifier`
/// (always) and `id` when present. The EXACT source attribute sequence (the
/// `style`/`id`/`identifier` order, plus any attribute the typed model drops)
/// is restored by the generic `AttributeOverrides` complement, so only
/// PRESENCE matters here for the backbone diff.
fn section_attrs(section: &UsCodeSection) -> Vec<XmlAttribute> {
    vec![attr("identifier", &section.identifier)]
}

/// `<num value="…">text</num>`. The text leaf is emitted only when non-empty,
/// matching the source (a childless `<num>` regenerates childless).
fn num_element(value: &str, text: &str) -> XmlElement {
    let children = if text.is_empty() {
        Vec::new()
    } else {
        vec![XmlNode::Text(text.to_string())]
    };
    element("num", vec![attr("value", value)], children)
}

/// Regenerate a `<sourceCredit>` element from its semantic mixed tree — the
/// exact ordered `#PCDATA` ↔ `<ref>`/`<date>` interleaving.
fn source_credit_element(credit: &UsCodeSourceCredit) -> XmlElement {
    let attrs = match &credit.identifier {
        Some(id) => vec![attr("id", id)],
        None => Vec::new(),
    };
    element("sourceCredit", attrs, mixed_children(&credit.mixed))
}

/// Regenerate a `<notes>` container element (slice U3) — its optional direct
/// `<heading>` (built from `heading_mixed`, in position before the notes) plus
/// its `<note>` children in source order. The typed model carries the
/// `type`/`id` attributes (`block_type`/`identifier`); the EXACT source
/// attribute sequence — including the `style` attr the model drops — is
/// restored by the generic `AttributeOverrides` complement, so only presence
/// matters here.
///
/// Fails closed when the block carries a direct `<heading>` String projection
/// but NO captured `heading_mixed` tree (a shape the reader could not lift to a
/// backbone-faithful tree) — the honest-partial boundary that keeps the byte
/// diff fail-closed rather than emitting a heading-less container.
fn notes_block_element(
    block: &UsCodeNotesBlock,
    section: &UsCodeSection,
) -> Result<XmlElement, UslmWriteError> {
    let mut children: Vec<XmlNode> = Vec::new();

    // The block's OWN direct <heading> (rare — absent from LRC Title 1, whose
    // headings are per-<note>). When the legacy String projection is present
    // the backbone-faithful mixed tree MUST be too, else the writer would emit
    // a heading-less container that the positional diff could not reconcile.
    match (&block.heading, &block.heading_mixed) {
        (_, Some(heading)) => children.push(XmlNode::Element(mixed_element("heading", heading))),
        (Some(_), None) => {
            return Err(UslmWriteError::UncoveredFamily {
                family: "notes-heading".to_string(),
                section: section.identifier.clone(),
            });
        }
        (None, None) => {}
    }

    for note in &block.notes {
        children.push(XmlNode::Element(note_element(note)));
    }

    Ok(element("notes", notes_block_attrs(block), children))
}

/// Regenerate one `<note>` element (slice U3) from a [`UsCodeNote`]. Its body
/// is the EXACT ordered child sequence of [`UsCodeNote::body_mixed`] — the
/// `<heading>` / `<p>` / … block children regenerated node-for-node from the
/// semantic mixed tree (a `<heading>` lands as a `Generic` node carrying its
/// `<b>` ornament; a `<p>` as a `Para` carrying its interleaved `<ref>` /
/// `<date>`). The typed model carries `topic`/`role`/`type`/`id`; the exact
/// source attribute sequence (including the `style` attr the model drops, and
/// the order) is restored by the generic complement.
///
/// Total over the note family: the mixed-content machinery carries every
/// note-body element (`heading`, `p`, and in other titles `num` / `table` /
/// `signature` / `quotedContent`) as a typed-or-`Generic` node, so a note never
/// needs a per-note `UncoveredFamily` — any divergence surfaces as a byte-exact
/// gate failure at the exact node, not a silent short body.
fn note_element(note: &UsCodeNote) -> XmlElement {
    element("note", note_attrs(note), mixed_children(&note.body_mixed))
}

/// The `<notes>` container's start-tag attributes the typed model carries —
/// `type` (`block_type`) and `id` (`identifier`), each only when present. The
/// EXACT source attribute sequence (the `style`/`type`/`id` order plus the
/// metadata attrs the model drops) is restored by the generic
/// `AttributeOverrides` complement, so only presence matters here.
fn notes_block_attrs(block: &UsCodeNotesBlock) -> Vec<XmlAttribute> {
    let mut attrs = Vec::new();
    if let Some(t) = &block.block_type {
        attrs.push(attr("type", t));
    }
    if let Some(id) = &block.identifier {
        attrs.push(attr("id", id));
    }
    attrs
}

/// The `<note>`'s start-tag attributes the typed model carries —
/// `topic` / `role` / `type` (`note_type`) / `id` (`identifier`), each only
/// when present. As with every USLM element, the EXACT source attribute
/// sequence (order + the `style` / `class` attrs the model drops) is restored
/// by the generic complement, so only presence matters here.
fn note_attrs(note: &UsCodeNote) -> Vec<XmlAttribute> {
    let mut attrs = Vec::new();
    if let Some(t) = &note.topic {
        attrs.push(attr("topic", t));
    }
    if let Some(r) = &note.role {
        attrs.push(attr("role", r));
    }
    if let Some(t) = &note.note_type {
        attrs.push(attr("type", t));
    }
    if let Some(id) = &note.identifier {
        attrs.push(attr("id", id));
    }
    attrs
}

/// An element named `name` whose children are the regenerated mixed-content
/// sequence of `mixed` (no start-tag attributes the typed model carries; the
/// source attribute sequence is restored by the complement).
fn mixed_element(name: &str, mixed: &UsCodeMixed) -> XmlElement {
    element(name, Vec::new(), mixed_children(mixed))
}

/// Regenerate the EXACT ordered child sequence of a [`UsCodeMixed`] — every
/// `#PCDATA` run as a [`XmlNode::Text`] (VERBATIM), every typed element node
/// as its named [`XmlNode::Element`] with its own regenerated children, in
/// source order. This is the load-bearing step: which child is Text vs element
/// and in what order is exactly what the positional backbone diff checks.
fn mixed_children(mixed: &UsCodeMixed) -> Vec<XmlNode> {
    mixed.nodes.iter().map(content_node).collect()
}

/// Regenerate one [`UsCodeContentNode`] into its XML node.
fn content_node(node: &UsCodeContentNode) -> XmlNode {
    match node {
        UsCodeContentNode::Text(t) => XmlNode::Text(t.clone()),
        UsCodeContentNode::Ref { attrs, children } => {
            XmlNode::Element(content_element("ref", attrs, children))
        }
        UsCodeContentNode::Date { attrs, children } => {
            XmlNode::Element(content_element("date", attrs, children))
        }
        UsCodeContentNode::Inline {
            kind,
            attrs,
            children,
        } => XmlNode::Element(content_element(inline_tag(*kind), attrs, children)),
        UsCodeContentNode::Para { attrs, children } => {
            XmlNode::Element(content_element("p", attrs, children))
        }
        UsCodeContentNode::Generic {
            name,
            attrs,
            children,
        } => XmlNode::Element(content_element(name, attrs, children)),
    }
}

/// Build a mixed-content child element: its source attributes (in order) plus
/// its own regenerated mixed children. The attribute byte sequence is restored
/// by the complement, so the typed `(name, value)` pairs need only be present.
///
/// `name` is a QUALIFIED name — a `Generic` node carrying a prefixed element
/// (`<dc:title>`, `<dcterms:created>` in the `<meta>` block) keeps its prefix, so
/// the name is split into [`XmlName::with_prefix`] to match the source DOM's
/// `(prefix, local)` exactly (the positional backbone diff compares full names).
fn content_element(
    name: &str,
    attrs: &[super::super::corpus::UsCodeContentAttr],
    children: &[UsCodeContentNode],
) -> XmlElement {
    let attributes = attrs.iter().map(|a| attr(&a.name, &a.value)).collect();
    let kids = children.iter().map(content_node).collect();
    qualified_element(name, attributes, kids)
}

/// The USLM/XHTML tag name for an inline ornament kind — the inverse of
/// [`InlineKind::parse`]. `PlainText` is not an element (it never reaches this
/// path: plain text is a [`UsCodeContentNode::Text`] node), so it maps to the
/// generic `<inline>` defensively.
fn inline_tag(kind: InlineKind) -> &'static str {
    match kind {
        InlineKind::Inline | InlineKind::PlainText => "inline",
        InlineKind::Italic => "i",
        InlineKind::Bold => "b",
        InlineKind::Superscript => "sup",
        InlineKind::Subscript => "sub",
        InlineKind::Span => "span",
        InlineKind::Anchor => "a",
    }
}

// ── small XML-ontology constructors (mirrors lmf/writer.rs) ──────────────────

/// Build a namespace-free [`XmlElement`] from an UNPREFIXED local name. The
/// typed USC model carries no namespace declarations of its own — the root's
/// `xmlns` set is restored by the generic [`DocumentResidue`] complement.
fn element(name: &str, attributes: Vec<XmlAttribute>, children: Vec<XmlNode>) -> XmlElement {
    XmlElement {
        name: XmlName::new(name),
        namespace: None,
        namespaces: Vec::new(),
        attributes,
        children,
    }
}

/// Build an [`XmlElement`] from a QUALIFIED name — a prefixed name (`dc:title`,
/// `dcterms:created`, `xsi:foo`) splits into [`XmlName::with_prefix`] so the
/// regenerated element's `(prefix, local)` matches the source DOM exactly (the
/// positional backbone diff compares full names, not just the local part); an
/// unprefixed name uses [`XmlName::new`] verbatim. Mirrors [`attr`]'s name
/// handling.
fn qualified_element(
    name: &str,
    attributes: Vec<XmlAttribute>,
    children: Vec<XmlNode>,
) -> XmlElement {
    let xml_name = match name.split_once(':') {
        Some((prefix, local)) => XmlName::with_prefix(prefix, local),
        None => XmlName::new(name),
    };
    XmlElement {
        name: xml_name,
        namespace: None,
        namespaces: Vec::new(),
        attributes,
        children,
    }
}

/// An [`XmlAttribute`] whose name may carry a prefix (`xml:lang`, `dc:source`).
/// A prefixed name splits into [`XmlName::with_prefix`] so a re-read
/// reconstructs the same `prefix`/`local`; an unprefixed name uses
/// [`XmlName::new`] verbatim.
fn attr(name: &str, value: &str) -> XmlAttribute {
    let xml_name = match name.split_once(':') {
        Some((prefix, local)) => XmlName::with_prefix(prefix, local),
        None => XmlName::new(name),
    };
    XmlAttribute {
        name: xml_name,
        value: value.to_string(),
    }
}

// ── Graph-faithful reconstruction — USLM source bytes from model + complement ──
//
// [`write_uslm`] regenerates a STRUCTURAL element tree from the typed
// [`UsCodeTitle`] model. By itself that tree is LOSSY against the real upstream
// USLM source: the W3C Information Set (Cowan & Tobin 2004) the typed model
// captures does not carry the byte-affecting residue — the document
// `<?xml-stylesheet?>`/`<!DOCTYPE>` prolog, the section root's `xmlns`
// declarations, the EXACT start-tag attribute order/coverage (`style`/`id`/
// `identifier` order; the `class`/`role` attrs the typed model drops), the §4.6
// predefined-entity-reference form, and the §2.4 \[14\] inter-element
// white-space (indentation). Those belong in the concrete-syntax COMPLEMENT,
// never in the ontology (the same ontology written two ways keeps one content
// address).
//
// [`UslmSyntaxComplement`] bundles that residue — the GENERIC XML-family
// [`DocumentResidue`] + [`RegeneratedComplement`] and the per-element
// [`SyntaxDecisions`] the byte-exact serializer honours — captured EXACTLY as
// the WN-LMF path captures its [`WnSyntaxComplement`]
// (`parse_document_capturing` for the DOM + decisions, `write_uslm` for the
// regenerated tree, `diff_content_whitespace` for the regenerated-tree residue,
// [`DocumentResidue`] read off the exact DOM). [`capture_uslm_complement`]
// derives it from a real section fragment; [`reconstruct_uslm_source`] re-applies
// it to `write_uslm`'s regenerated tree and serializes byte-exact.
//
// The reconstruction is `uslm::UsCodeTitle ontology + captured complement →
// exact source bytes` — the USC realisation of the serialized reverse lens'
// graph-faithful `put` (Foster, Greenwald, Moore, Pierce & Schmitt 2007, ACM
// TOPLAS 29(3) §3, Definition 3.2). The parser-level residue machinery is generic XML-family
// (no USLM vocabulary); only this glue is USLM-specific.
//
// This is still the WRITER-FIRST, NO-ENVELOPE slice: the complement is the
// byte-exact GATE's second input, NOT yet wired into an rkyv `.prx` envelope (no
// `praxis.lock`/rkyv-struct change). The `prx`-gated derives the generic residue
// types already carry travel for free when the envelope slice arrives.

/// The concrete-syntax COMPLEMENT for one USLM section fragment: everything the
/// typed [`UsCodeTitle`] model + [`write_uslm`]'s regenerated tree do NOT carry,
/// captured so [`reconstruct_uslm_source`] reproduces the exact source bytes.
///
/// The exact USC analogue of the WN-LMF
/// [`WnSyntaxComplement`](crate::social::software::markup::xml::lmf::writer::WnSyntaxComplement):
/// the same three generic XML-family residue classes (document/root residue,
/// regenerated-tree residue, per-element byte decisions), differing only in that
/// the typed model is a [`UsCodeTitle`], not a `WordNet`. All three fields are
/// generic XML-family residue carrying the same `prx`-gated rkyv derive; this
/// bundle is the only USLM-specific piece.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct UslmSyntaxComplement {
    /// Document/root-level residue: the `<!DOCTYPE>` (§2.8 \[28\]) and the
    /// section root's namespace declarations (Bray, Hollander, Layman & Tobin
    /// 2009 §3) the typed model drops.
    pub document_residue: DocumentResidue,
    /// The regenerated-tree residue: the §2.4 \[14\] inter-element white-space
    /// the structural writer's tree lacks AND the exact source start-tag
    /// attribute sequences it reordered or under-populated (the `<section>`
    /// `style`/`id`/`identifier` order; the metadata attrs the typed model does
    /// not carry — attribute order/coverage being concrete-syntax, not Infoset,
    /// per Cowan & Tobin 2004 §2.3). Keyed by pre-order element index.
    pub regenerated: RegeneratedComplement,
    /// The per-element concrete-syntax decisions the byte-exact serializer
    /// honours — intra-tag white-space layout (§3.1 \[40\]/\[44\]), §4.6
    /// predefined-entity-reference form, the empty-element form (§3.1), and the
    /// prolog/epilog white-space (§2.8 \[27\]). Captured by
    /// `parse_document_capturing` directly from the source.
    pub syntax_decisions: SyntaxDecisions,
}

/// Failure modes of the USLM graph-faithful reconstruction. Fail-closed: every
/// path returns a typed error (no `unwrap`/`expect`), so the byte-exact gate
/// fails LOUD at the exact cause rather than fabricating bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UslmReconstructError {
    /// The source bytes did not parse as well-formed XML (§2.1) — the grammar
    /// parser rejected them. Carries the underlying [`XmlParseError`].
    Parse(XmlParseError),
    /// [`read_uslm_title`] could not build the typed view from the fragment
    /// (not a recognised USLM section/title shape). Carries the read error's
    /// rendered message.
    Read(String),
    /// [`write_uslm`] could not regenerate the section backbone — the fragment
    /// carries a structure family the writer does not yet cover (a notes block,
    /// a `<def>` / `<marker>`, the `<uscDoc>` wrapper, …). Carries the
    /// [`UslmWriteError`] naming the next slice; this is the honest-partial
    /// boundary, never a silent short tree.
    Write(UslmWriteError),
    /// `write_uslm`'s regenerated tree was not element-backbone-equal to the
    /// captured source DOM (a dropped/added/reordered/renamed element or
    /// leaf-text). The residue cannot be a pure white-space/decl complement;
    /// carries the exact divergence.
    Complement(RegeneratedComplementError),
}

impl core::fmt::Display for UslmReconstructError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "USLM source parse: {e}"),
            Self::Read(m) => write!(f, "USLM typed read: {m}"),
            Self::Write(e) => write!(f, "USLM structural write: {e}"),
            Self::Complement(e) => write!(f, "USLM regenerated-tree complement: {e}"),
        }
    }
}

impl std::error::Error for UslmReconstructError {}

impl From<XmlParseError> for UslmReconstructError {
    fn from(e: XmlParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<UslmWriteError> for UslmReconstructError {
    fn from(e: UslmWriteError) -> Self {
        Self::Write(e)
    }
}

impl From<RegeneratedComplementError> for UslmReconstructError {
    fn from(e: RegeneratedComplementError) -> Self {
        Self::Complement(e)
    }
}

/// Capture the typed [`UsCodeTitle`] model AND the concrete-syntax
/// [`UslmSyntaxComplement`] from a USLM section fragment, such that
/// [`reconstruct_uslm_source`] reproduces the source bytes byte-for-byte.
///
/// Threaded EXACTLY like the WN-LMF
/// [`capture_wn_complement`](crate::social::software::markup::xml::lmf::writer::capture_wn_complement):
///
/// 1. **The exact DOM + decisions** — `parse_document_capturing(frag)` parses
///    the fragment through the W3C XML 1.0 grammar, yielding the full
///    Information-Set DOM (with the `<!DOCTYPE>`, the root namespaces, and the
///    inter-element white-space `Text` children) and the [`SyntaxDecisions`] the
///    byte-exact serializer honours.
/// 2. **The typed model** — [`read_uslm_title`] extracts the same elements /
///    attrs / leaf-text the grammar parser does (it keys on element names), so
///    the model the writer regenerates from is faithful to the exact DOM's
///    backbone.
/// 3. **The regenerated-tree complement** — `write_uslm(&title)` regenerates the
///    structural tree the typed model alone produces; [`diff_content_whitespace`]
///    extracts the inter-element white-space + attribute residue that tree
///    LACKS, and the `<!DOCTYPE>` + root namespaces are read straight off the
///    exact DOM.
///
/// Fails closed (typed [`UslmReconstructError`], no `unwrap`/`expect`) if the
/// fragment is not well-formed XML, not a recognised USLM shape, carries an
/// uncovered family ([`write_uslm`] returns [`UslmWriteError`]), or the
/// regenerated tree is not element-backbone-equal to the captured DOM.
pub fn capture_uslm_complement(
    frag: &str,
) -> Result<(UsCodeTitle, UslmSyntaxComplement), UslmReconstructError> {
    // (1) The exact DOM + the per-element/prolog decisions the byte-exact
    // serializer honours.
    let (exact_dom, syntax_decisions) = parse_document_capturing(frag.as_bytes())?;

    // (2) The typed model. `read_uslm_title` keys on element names, so the model
    // the writer regenerates from is faithful to `exact_dom`'s backbone.
    let title =
        read_uslm_title(frag).map_err(|e| UslmReconstructError::Read(alloc::format!("{e}")))?;

    // (3) The regenerated tree the typed model alone produces, and the residue it
    // lacks relative to the exact DOM — inter-element white-space AND the exact
    // source attribute sequences (order + metadata attrs the model drops).
    let regenerated_tree = write_uslm(&title)?;
    let regenerated = diff_content_whitespace(&exact_dom, &regenerated_tree)?;

    // The document/root-level residue read straight off the exact DOM: the
    // `<!DOCTYPE>` and the root element's namespace declarations.
    let document_residue = DocumentResidue {
        doctype: exact_dom.doctype.clone(),
        root_namespaces: exact_dom.root.namespaces.clone(),
        // The XML declaration's version + encoding form, captured so the
        // byte-exact writer reproduces the source declaration rather than the
        // structural writer's default. Additive (USLM declares the same form the
        // structural writer emits, so a re-pin is unnecessary).
        xml_version: Some(exact_dom.version.clone()),
        xml_encoding: Some(exact_dom.encoding.clone()),
    };

    Ok((
        title,
        UslmSyntaxComplement {
            document_residue,
            regenerated,
            syntax_decisions,
        },
    ))
}

/// Reconstruct the exact USLM source bytes from the typed [`UsCodeTitle`] model
/// and its captured [`UslmSyntaxComplement`] — the graph-faithful `put`.
///
/// 1. `write_uslm(title)` regenerates the structural element tree the typed
///    model produces (no white-space, no DOCTYPE, no namespaces);
/// 2. [`reapply_regenerated_complement`] merges the residue back in — sets the
///    `<!DOCTYPE>` and root namespaces, overrides each element's start-tag
///    attribute sequence with the exact source one, splices the inter-element
///    white-space `Text` children at their captured pre-order positions —
///    reproducing the exact captured DOM;
/// 3. [`serialize_document_exact`] emits that DOM PLUS the [`SyntaxDecisions`]
///    byte-for-byte.
///
/// When `title` and `complement` are the pair [`capture_uslm_complement`]
/// returned for a fragment `s`, the result equals `s.as_bytes()` exactly. Fails
/// closed (typed error, no `unwrap`/`expect`) on a structural divergence — never
/// fabricates bytes.
pub fn reconstruct_uslm_source(
    title: &UsCodeTitle,
    complement: &UslmSyntaxComplement,
) -> Result<Vec<u8>, UslmReconstructError> {
    let mut tree = write_uslm(title)?;
    reapply_regenerated_complement(
        &mut tree,
        &complement.document_residue,
        &complement.regenerated,
    )?;
    Ok(serialize_document_exact(
        &tree,
        &complement.syntax_decisions,
    ))
}

// =============================================================================
// UslmGraphFaithfulLens — the byte-exact graph-faithful lens, and the
// registration that flips the PROVEN `usc_title_1` off the universal floor in
// the completeness meter.
//
// The USC analogue of the WN-LMF
// [`WordNetLmfLens`](crate::social::software::markup::xml::lmf::lens::WordNetLmfLens):
// the source regenerates from the typed [`UsCodeTitle`] ontology PLUS a content-
// addressed concrete-syntax complement ([`UslmSyntaxComplement`]), held to the
// strict byte-exact PutGet law (Foster, Greenwald, Moore, Pierce & Schmitt 2007
// ACM TOPLAS 29(3) §3, Definition 3.2; Mac Lane 1998 §IV.4 equivalence-of-categories counit at
// byte identity), with NO stored raw blob.
//
// - `get : &[u8] → (UsCodeTitle, UslmSyntaxComplement)` — [`capture_uslm_complement`]:
//   parse the source through the USLM grammar, yielding the typed ontology AND
//   the byte-affecting residue. Fails closed on malformed input, an uncovered
//   family, or a structural-writer divergence.
// - `put : &(UsCodeTitle, UslmSyntaxComplement) → Vec<u8>` —
//   [`reconstruct_uslm_source`]: re-apply the complement to the structural
//   writer's regenerated tree and serialize byte-exact.
// - `canonical` — the IDENTITY: a byte-exact lens guarantees the source IS its
//   own canonical form (`put(get(b)) == b`); the byte-exact harness path never
//   calls it (it compares raw bytes via `assert_byte_exact_law` and signs the raw
//   bytes via `[byte_exact_signatures]`). Provided only for trait totality.
//
// Registering this lens with `FIDELITY = ByteExactGraphFaithful` makes the
// completeness meter
// ([`crate::formal::meta::well_behaved_lens::completeness`]) DECLARE any title
// BOUND to it graph-faithful and drop that title's `write_uslm` gap; when the
// title is provisioned on disk the harness MEASURES the achieved tier by running
// the byte-exact law, and the anti-lie cross-check confirms declared == achieved.
// WHICH titles bind here vs the floor `UslmXmlLens` is the registry's concern
// (the `register_lens!` calls in `lens/mod.rs` and below), NOT this type's: a
// title rides the floor only while it carries no graph-faithful registration —
// e.g. the largest titles, held off the always-run byte-exact gate for the CI
// per-test budget (the writer is title-agnostic, so it reconstructs them too).
// =============================================================================

/// The USLM byte-exact graph-faithful lens: `bytes ↔ (UsCodeTitle ontology +
/// concrete-syntax complement)`. Declares
/// [`RoundTripFidelity::ByteExactGraphFaithful`](crate::formal::meta::well_behaved_lens::RoundTripFidelity::ByteExactGraphFaithful) — the USC sibling of the WN-LMF
/// `WordNetLmfLens`.
#[derive(Debug)]
pub struct UslmGraphFaithfulLens;

/// The graph-faithful target: the typed [`UsCodeTitle`] ontology paired with the
/// concrete-syntax [`UslmSyntaxComplement`] the byte-exact `put` re-applies.
/// `get` produces this pair; `put` consumes it to regenerate the source bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct UslmGraphFaithfulView {
    /// The typed USLM title ontology — the graph the source regenerates from.
    pub title: UsCodeTitle,
    /// The concrete-syntax residue the typed ontology does not carry.
    pub complement: UslmSyntaxComplement,
}

/// Error from the USLM graph-faithful lens: a UTF-8 decode failure (the source is
/// not valid UTF-8 text) or a [`UslmReconstructError`] from the
/// capture/reconstruct pair.
#[derive(Debug)]
pub enum UslmGraphFaithfulLensError {
    /// The source bytes are not valid UTF-8 (USLM is XML text).
    NotUtf8(String),
    /// The graph-faithful capture or reconstruction failed (a parse error, an
    /// unrecognised USLM shape, an uncovered family, or a backbone divergence).
    Reconstruct(UslmReconstructError),
}

impl core::fmt::Display for UslmGraphFaithfulLensError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotUtf8(e) => write!(f, "USLM source is not UTF-8: {e}"),
            Self::Reconstruct(e) => write!(f, "USLM graph-faithful round-trip: {e}"),
        }
    }
}

impl std::error::Error for UslmGraphFaithfulLensError {}

impl From<UslmReconstructError> for UslmGraphFaithfulLensError {
    fn from(e: UslmReconstructError) -> Self {
        Self::Reconstruct(e)
    }
}

impl crate::formal::meta::well_behaved_lens::WellBehavedLens for UslmGraphFaithfulLens {
    type Target = UslmGraphFaithfulView;
    type Error = UslmGraphFaithfulLensError;

    /// The graph-faithful tier — held to the strict byte-exact PutGet law.
    const FIDELITY: crate::formal::meta::well_behaved_lens::RoundTripFidelity =
        crate::formal::meta::well_behaved_lens::RoundTripFidelity::ByteExactGraphFaithful;

    /// `get` — capture the typed ontology AND the concrete-syntax complement
    /// from the source ([`capture_uslm_complement`]).
    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        let text = core::str::from_utf8(bytes)
            .map_err(|e| UslmGraphFaithfulLensError::NotUtf8(alloc::format!("{e}")))?;
        let (title, complement) = capture_uslm_complement(text)?;
        Ok(UslmGraphFaithfulView { title, complement })
    }

    /// `put` — regenerate the source bytes from the graph + complement
    /// ([`reconstruct_uslm_source`]), NO stored raw blob.
    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(reconstruct_uslm_source(&target.title, &target.complement)?)
    }

    /// `canonical` — the IDENTITY for a byte-exact lens: the source is its own
    /// canonical form (`put(get(b)) == b`). The byte-exact harness path never
    /// calls this (it compares raw bytes); it is here only for trait totality.
    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        Ok(bytes.to_vec())
    }
}

// Bind `UslmGraphFaithfulLens` to the registered, PROVEN `usc_title_1@pl-119-90`
// source. The harness runs the byte-exact law (because FIDELITY is
// ByteExactGraphFaithful) and verifies the raw-bytes signature against
// `[byte_exact_signatures]` in praxis.lock. The completeness meter reads the
// FIDELITY const to declare the title graph-faithful and drop its `write_uslm`
// gap. Native only — linkme's distributed slice is unsupported on wasm32 (the
// harness is a native CI/audit tool), mirroring every other `register_lens!`.
crate::register_lens!(
    USC_TITLE_1_GRAPH_FAITHFUL_LENS,
    "usc_title_1",
    "pl-119-90",
    UslmGraphFaithfulLens
);

// SLICE U7 — flip the smaller positive-law titles that ride the SAME generic
// `uscdoc_mixed` document-wrapper path Title 1 proved (U4) onto the byte-exact
// graph-faithful lens. They exercise USLM families ABSENT from Title 1 — the
// `<continuation>` flush-text family and the XHTML `<table>` family — yet
// regenerate byte-exact, because the U4 mixed-content backbone carries every
// such element as a generic node and the L3 byte kernel restores their residue
// (prolog PI, §2.11 CRLFs, §4.6 `&amp;` predefined-entity form, attribute order,
// inter-element white-space). No new typed family was needed; the proof is the
// `flipped_titles_reconstruct_byte_exact` gate + the all-sources round-trip
// integration test, and each title's `[byte_exact_signatures]` pin equals its
// `[hashes]` content address (registry.rs enforces sig == raw_hash).
//
// These four titles are all ≤ 16 MB, so the always-run `ci_gate_passes` harness
// reconstructs them within the strict nextest `ci` budget. The GIANT titles
// (usc_title_5/15/42/49, 19–113 MB) ride the SAME lens in SLICE U8 below; their
// > 16 MB reconstruction is deferred by the CI-A oversize split
// (`OVERSIZE_BYTE_EXACT_CAP_BYTES`) to the slow `ci_gate_passes_giants` lane, so
// the fast lane stays under budget. The double-registration lesson — one source,
// one lens, one tier — means each flipped title carries ONLY this graph-faithful
// lens; its previous floor `UslmXmlLens` registration in `lens/mod.rs` is removed.
crate::register_lens!(
    USC_TITLE_28_GRAPH_FAITHFUL_LENS,
    "usc_title_28",
    "pl-119-90",
    UslmGraphFaithfulLens
);
crate::register_lens!(
    USC_TITLE_18_GRAPH_FAITHFUL_LENS,
    "usc_title_18",
    "pl-119-90",
    UslmGraphFaithfulLens
);
crate::register_lens!(
    USC_TITLE_29_GRAPH_FAITHFUL_LENS,
    "usc_title_29",
    "pl-119-90",
    UslmGraphFaithfulLens
);
crate::register_lens!(
    USC_TITLE_50_GRAPH_FAITHFUL_LENS,
    "usc_title_50",
    "pl-119-90",
    UslmGraphFaithfulLens
);

// SLICE U8 — flip the GIANT positive-law titles (usc_title_5/15/42/49, 19–113 MB)
// onto the SAME title-agnostic `UslmGraphFaithfulLens`. No new writer family: the
// giants ride the identical generic `uscDoc` mixed-content backbone (U4) + L3 byte
// kernel the smaller titles proved, so the structural writer reconstructs them
// byte-exact too. They were held on the floor ONLY for the always-run harness
// budget; the CI-A oversize split now defers their reconstruction to the slow
// `ci_gate_passes_giants` lane, so registering them no longer burdens the fast
// lane — and EVERY bundled `.prx`-consumer USC title is now byte-exact
// graph-faithful (17/17 sources overall). Each title's `[byte_exact_signatures]`
// pin equals its `[hashes]` content address (registry.rs enforces sig == raw_hash);
// the proof is `ci_gate_passes_giants` + the all-sources source round-trip test,
// whose coverage assertion requires every provisioned graph-faithful source to
// reconstruct byte-exact.
crate::register_lens!(
    USC_TITLE_5_GRAPH_FAITHFUL_LENS,
    "usc_title_5",
    "pl-119-90",
    UslmGraphFaithfulLens
);
crate::register_lens!(
    USC_TITLE_15_GRAPH_FAITHFUL_LENS,
    "usc_title_15",
    "pl-119-90",
    UslmGraphFaithfulLens
);
crate::register_lens!(
    USC_TITLE_42_GRAPH_FAITHFUL_LENS,
    "usc_title_42",
    "pl-119-90",
    UslmGraphFaithfulLens
);
crate::register_lens!(
    USC_TITLE_49_GRAPH_FAITHFUL_LENS,
    "usc_title_49",
    "pl-119-90",
    UslmGraphFaithfulLens
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis_runtime::address::ContentAddress;

    /// Workspace root — grandparent of `crates/domains` (mirrors the WordNet
    /// corpus gate's `workspace_root_for_test`).
    fn workspace_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }

    /// The minimal whole-document XML declaration the writer emits and the
    /// byte-exact serializer always prepends (W3C XML 1.0 §2.8 \[23\] `XMLDecl`).
    /// A USLM document is served WITH this declaration; the writer regenerates a
    /// whole document, so the byte-exact fragment is a whole mini-document
    /// (declaration + the sliced `<section>`), not a bare mid-file element.
    ///
    /// NOTE on the slice harness: the section bytes are sliced verbatim from the
    /// mid-file source (no transcription); only this canonical declaration is
    /// prepended to make a well-formed document with the prolog the writer
    /// produces. The byte-exact assertion then covers the ENTIRE mini-document.
    /// (XML-declaration presence/absence is itself concrete-syntax the current
    /// byte-exact kernel does not yet represent — `serialize_document_exact`
    /// always emits one — so a declaration-LESS bare-element fragment is an
    /// uncovered species for a LATER kernel slice, fenced out of this writer
    /// slice by anchoring to the whole-document form.)
    const XML_DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>";

    /// Slice the EXACT `<section …identifier="{urn}">…</section>` substring out
    /// of the on-disk Title 1 USLM source (no transcription, genuine bytes) and
    /// wrap it in the canonical [`XML_DECL`] so it is a whole mini-document with
    /// the prolog the writer emits. `None` when the corpus file is absent
    /// (graceful skip) or the section is not found.
    fn real_title1_section(urn: &str) -> Option<String> {
        let path = workspace_root()
            .join("crates/domains/data/legal/uscode/usc_title_1/usc_title_1-pl-119-90.xml");
        let xml = std::fs::read_to_string(&path).ok()?;
        // Find the `<section ` whose attributes include `identifier="{urn}"`.
        let needle = alloc::format!("identifier=\"{urn}\"");
        let id_pos = xml.find(&needle)?;
        let start = xml[..id_pos].rfind("<section")?;
        let end_tag = "</section>";
        let end_rel = xml[start..].find(end_tag)? + end_tag.len();
        let section = &xml[start..start + end_rel];
        Some(alloc::format!("{XML_DECL}{section}"))
    }

    /// THE BYTE-EXACT GATE over one real section fragment, mirroring the WN-LMF
    /// `reconstruct_*_byte_exact` gates: `capture_uslm_complement(frag)` then
    /// `reconstruct_uslm_source(&title, &complement)` must equal `frag` BYTE-FOR
    /// -BYTE (content-address-pinned). This proves `write_uslm` FAITHFULNESS — a
    /// corrupted text leaf or a reversed mixed sequence diverges the
    /// reconstructed bytes (the meta-test below proves it goes RED on
    /// corruption), unlike the old `diff_content_whitespace(...).is_ok()` gate,
    /// which matched children by node-type position + element name and never
    /// compared the regenerated text value.
    ///
    /// On failure it reports the EXACT first byte-diff (an honest, bounded
    /// 80-byte window — never a full dump) so an uncaptured concrete-syntax
    /// species in the section fragment names itself.
    fn assert_byte_exact_gate(frag: &str) {
        let (title, complement) = match capture_uslm_complement(frag) {
            Ok(pair) => pair,
            Err(e) => panic!("capture_uslm_complement failed (RED): {e}"),
        };
        // The residue is GENUINELY present — a vacuous (empty-complement)
        // round-trip would be a lie. A real sliced section carries at least the
        // §2.4 inter-element white-space around its `<num>`/`<heading>`/
        // `<content>`/`<sourceCredit>` children.
        assert!(
            !complement.regenerated.content_whitespace.is_empty()
                || !complement.regenerated.attribute_overrides.is_empty(),
            "the section fragment must carry genuine regenerated-tree residue \
             (inter-element white-space and/or attribute overrides)"
        );
        let out = match reconstruct_uslm_source(&title, &complement) {
            Ok(bytes) => bytes,
            Err(e) => panic!("reconstruct_uslm_source failed (RED): {e}"),
        };
        let source = frag.as_bytes();
        if out != source {
            let first = out
                .iter()
                .zip(source.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(source.len().min(out.len()));
            let lo = first.saturating_sub(80);
            let hi_out = (first + 80).min(out.len());
            let hi_src = (first + 80).min(source.len());
            panic!(
                "byte mismatch at offset {first} (out.len()={}, source.len()={})\n  \
                 expected: {:?}\n  got:      {:?}",
                out.len(),
                source.len(),
                String::from_utf8_lossy(&source[lo..hi_src]),
                String::from_utf8_lossy(&out[lo..hi_out]),
            );
        }
        // Content-digest equality is the headline assertion. `out == source` already
        // implies it; the explicit pin guards against a silent corpus swap.
        assert_eq!(
            ContentAddress::of(&out).to_hex(),
            ContentAddress::of(source).to_hex(),
            "reconstructed section must hash-equal the source fragment"
        );
    }

    /// HARD BYTE-EXACT GATE: the real Title 1 § 2 — including its true
    /// mixed-content `<sourceCredit>` — reconstructs BYTE-FOR-BYTE from the
    /// typed [`UsCodeTitle`] + the captured [`UslmSyntaxComplement`]. This is
    /// the `write_uslm` FAITHFULNESS proof: the writer must reproduce every
    /// element AND every `#PCDATA` text value in exact order, not merely a
    /// backbone the old diff-Ok gate could reconcile while the regenerated text
    /// was corrupt.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn real_title1_s2_reconstruct_is_byte_exact() {
        let Some(frag) = real_title1_section("/us/usc/t1/s2") else {
            return; // corpus not provisioned — skip gracefully
        };
        assert_byte_exact_gate(&frag);
    }

    /// Second real section so the byte-exact gate is not single-instance.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn real_title1_s4_reconstruct_is_byte_exact() {
        let Some(frag) = real_title1_section("/us/usc/t1/s4") else {
            return;
        };
        assert_byte_exact_gate(&frag);
    }

    /// META-TEST (the gate has TEETH): capture the real § 2 fragment, then
    /// CORRUPT the typed [`UsCodeMixed`] BEFORE reconstruct — here by reversing
    /// the `<sourceCredit>`'s interleaved mixed nodes — and assert the byte-exact
    /// reconstruction NO LONGER equals the source. This is the proof the old
    /// `diff_content_whitespace(...).is_ok()` gate LACKED: an empirical mutation
    /// (reversing the sourceCredit Text/Ref/Text/Ref/Text sequence, replacing
    /// every #PCDATA with "XXXX") left both diff-Ok gates GREEN, because the
    /// diff's `Keep` branch consumes a regenerated `Text` without comparing its
    /// value and records attributes as source-side overrides, never comparing
    /// them. The byte-exact gate above catches exactly that class of writer/model
    /// corruption — proven RED here.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn corrupted_mixed_breaks_byte_exact_gate() {
        let Some(frag) = real_title1_section("/us/usc/t1/s2") else {
            return;
        };
        let (mut title, complement) =
            capture_uslm_complement(&frag).expect("capture the real § 2 fragment");

        // Sanity: the UNCORRUPTED pair reconstructs byte-exact, so the failure
        // below is attributable to the corruption, not a pre-existing defect.
        let clean = reconstruct_uslm_source(&title, &complement).expect("clean reconstruct");
        assert_eq!(
            clean,
            frag.as_bytes(),
            "the uncorrupted capture must reconstruct byte-exact (control)"
        );

        // CORRUPT the typed model: rewrite a #PCDATA Text leaf of the first
        // sourceCredit's mixed sequence to a different value. This leaves the
        // element BACKBONE (which child is Text vs <ref>, in what order) IDENTICAL
        // — so `reapply_regenerated_complement`'s pre-order walk still succeeds —
        // but a FAITHFUL writer must now emit different bytes for that text run.
        // The exact mutation class the old diff-Ok gate missed: it consumed the
        // regenerated `Text` as a `Keep` without ever comparing its value.
        let credit = &mut title.sections[0].source_credits[0];
        let corrupted_a_text = credit
            .mixed
            .nodes
            .iter_mut()
            .find_map(|n| match n {
                UsCodeContentNode::Text(t) => {
                    *t = alloc::format!("{t}-CORRUPTED");
                    Some(())
                }
                _ => None,
            })
            .is_some();
        assert!(
            corrupted_a_text,
            "the § 2 sourceCredit must carry a #PCDATA Text leaf to corrupt"
        );

        let corrupted = reconstruct_uslm_source(&title, &complement)
            .expect("reconstruct still runs on a corrupted-but-backbone-valid model");
        assert_ne!(
            corrupted,
            frag.as_bytes(),
            "a corrupted #PCDATA Text value MUST diverge the byte-exact \
             reconstruction — the gate has teeth (this is what the old diff-Ok \
             gate failed to catch)"
        );
    }

    /// READER-MODEL CHECK (NOT a `write_uslm` faithfulness proof). The
    /// mixed-content `<sourceCredit>` is GENUINELY present and ordered in the
    /// typed view `read_uslm_title` produces — a vacuous read (empty credit)
    /// would be a lie. This asserts the READER's model, and passed UNCHANGED
    /// while `write_uslm` emitted corrupted text (the adversary mutation), so it
    /// is explicitly NOT a writer-faithfulness gate; the byte-exact gates above
    /// are. Kept because the covered family really exercises interleaved text ↔
    /// element children, which the writer must then reproduce.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reader_model_sourcecredit_mixed_content_is_genuinely_interleaved() {
        let Some(frag) = real_title1_section("/us/usc/t1/s2") else {
            return;
        };
        let title = read_uslm_title(&frag).expect("read");
        let section = &title.sections[0];
        let credit = &section.source_credits[0];
        // (, ref, ", ", ref, .) → 5 nodes: Text, Ref, Text, Ref, Text.
        assert_eq!(
            credit.mixed.nodes.len(),
            5,
            "sourceCredit interleaves literal punctuation with two <ref>s"
        );
        assert!(
            matches!(credit.mixed.nodes[0], UsCodeContentNode::Text(ref t) if t == "("),
            "opens with the literal '(' #PCDATA run"
        );
        assert!(
            matches!(credit.mixed.nodes[1], UsCodeContentNode::Ref { .. }),
            "second node is the first <ref> citation"
        );
        assert!(
            matches!(credit.mixed.nodes[2], UsCodeContentNode::Text(ref t) if t == ", "),
            "literal ', ' punctuation between the two citations"
        );
        assert!(
            matches!(credit.mixed.nodes[4], UsCodeContentNode::Text(ref t) if t == ".)"),
            "closes with the literal '.)' #PCDATA run"
        );
        // The DERIVED flat views still hold (downstream stays working).
        assert_eq!(credit.refs.len(), 2, "two refs derived from the mixed tree");
        assert_eq!(
            credit.refs[0].href, "/us/act/1947-07-30/ch388",
            "first ref href derived"
        );
    }

    /// READER-MODEL CHECK (NOT a `write_uslm` faithfulness proof). The
    /// `<content>` carries its block `<p>` IN POSITION as a semantic `Para` node
    /// in the typed view `read_uslm_title` produces — proving `<content>` is not
    /// flattened to a bare string in the READER's model. The surrounding
    /// inter-element white-space (`\n` around the `<p>`) is pure §2.3 \[3\] `S`,
    /// which `read_xml` drops from the typed tree and the GENERIC SourceSyntax
    /// complement carries instead (it re-splices it on reconstruction) — so the
    /// semantic tree holds only the `<p>` node, no per-node whitespace blob.
    ///
    /// Like the sourceCredit reader-model check, this asserts the READER's model
    /// and passed unchanged while the writer emitted corrupted text; the
    /// byte-exact gates above are the writer-faithfulness proof. The writer emits
    /// a whitespace-free `<content><p>…</p></content>`; reconstruction re-splices
    /// the source's `\n` runs as `InsertText` white-space residue.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reader_model_content_block_paragraph_is_positional() {
        let Some(frag) = real_title1_section("/us/usc/t1/s2") else {
            return;
        };
        let title = read_uslm_title(&frag).expect("read");
        let content = title.sections[0]
            .content_mixed
            .as_ref()
            .expect("content present");
        // Exactly the `<p>` Para node — the whitespace siblings are complement
        // residue, not tree nodes (no DOM-in-disguise).
        assert_eq!(content.nodes.len(), 1, "the block <p>, whitespace elided");
        assert!(matches!(content.nodes[0], UsCodeContentNode::Para { .. }));
        // The `<p>` itself carries its prose as a verbatim #PCDATA Text leaf.
        let UsCodeContentNode::Para { children, .. } = &content.nodes[0] else {
            panic!("expected the content's single node to be the block <p> Para");
        };
        assert!(
            matches!(children.first(), Some(UsCodeContentNode::Text(t)) if t.starts_with("The word")),
            "the <p>'s #PCDATA is captured verbatim as a Text leaf"
        );
        // The DERIVED plain-text projection still flattens to the prose.
        assert!(
            title.sections[0]
                .content
                .as_deref()
                .is_some_and(|c| c.starts_with("The word")),
            "flat content projection still derived"
        );
    }

    /// HARD BYTE-EXACT GATE (slice U2): the real Title 1 § 8 — three
    /// `<subsection>`s, each a `<num>` + `<content>` leaf, plus the §'s true
    /// mixed-content `<sourceCredit>` — reconstructs BYTE-FOR-BYTE from the
    /// typed [`UsCodeTitle`] + captured [`UslmSyntaxComplement`]. This proves
    /// the SUBDIVISION backbone (the `<subsection>` recursion U2 adds) is
    /// faithful on a real published section: every subsection `<num>` /
    /// `<content>` regenerates in exact source order.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn real_title1_s8_reconstruct_is_byte_exact() {
        let Some(frag) = real_title1_section("/us/usc/t1/s8") else {
            return; // corpus not provisioned — skip gracefully
        };
        assert_byte_exact_gate(&frag);
    }

    /// Slice the EXACT § 201 `<section>…</section>` substring (a synthetic
    /// note-§ that carries NO `identifier` attribute on `<section>`, so the
    /// URN-keyed [`real_title1_section`] can't find it) by anchoring on its
    /// unique `“SEC. 201.` num text, then wrap it in [`XML_DECL`]. `None` when
    /// the corpus file is absent (graceful skip).
    ///
    /// § 201 is the byte-exact gate for the FULL subdivision recursion U2
    /// targets — a `<subsection>` carrying `<num>` + `<heading>` (with an
    /// `<inline class="small-caps">` ornament) + `<chapeau>` (with an
    /// interleaved `<date>`) + nested `<paragraph>` children, alongside a leaf
    /// `<subsection>` of `<num>` + `<heading>` + `<content>`. The
    /// subsection→paragraph nesting + the heading/chapeau mixed trees on a
    /// SUBDIVISION are exactly what this slice adds.
    fn real_title1_section_201() -> Option<String> {
        let path = workspace_root()
            .join("crates/domains/data/legal/uscode/usc_title_1/usc_title_1-pl-119-90.xml");
        let xml = std::fs::read_to_string(&path).ok()?;
        let num_pos = xml.find("\u{201c}SEC. 201.")?;
        let start = xml[..num_pos].rfind("<section")?;
        let end_tag = "</section>";
        let end_rel = xml[start..].find(end_tag)? + end_tag.len();
        let section = &xml[start..start + end_rel];
        Some(alloc::format!("{XML_DECL}{section}"))
    }

    /// The § 201 byte-exact gate proper — see `real_title1_s201_*` doc above.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn real_title1_s201_subdivision_recursion_is_byte_exact() {
        let Some(frag) = real_title1_section_201() else {
            return; // corpus not provisioned — skip gracefully
        };
        // Sanity: the fragment really is the subsection→paragraph recursion
        // this slice targets (else a corpus change silently weakened the gate).
        assert!(
            frag.contains("<subsection") && frag.contains("<paragraph"),
            "the § 201 slice must carry the subsection→paragraph recursion"
        );
        assert_byte_exact_gate(&frag);
    }

    /// META-TEST (slice U2 has TEETH at the SUBDIVISION level): capture the real
    /// § 201 fragment, then corrupt a #PCDATA Text leaf INSIDE a nested
    /// `<paragraph>`'s `<content>` (deep in the subdivision recursion) and
    /// assert the byte-exact reconstruction NO LONGER equals the source. This
    /// is the U2 analogue of `corrupted_mixed_breaks_byte_exact_gate`: it
    /// proves the new subdivision writer reproduces the EXACT text of a deep
    /// leaf, not merely a backbone the positional diff could reconcile.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn corrupted_subdivision_content_breaks_byte_exact_gate() {
        let Some(frag) = real_title1_section_201() else {
            return;
        };
        let (mut title, complement) =
            capture_uslm_complement(&frag).expect("capture the real § 201 fragment");

        // Control: the uncorrupted capture reconstructs byte-exact.
        let clean = reconstruct_uslm_source(&title, &complement).expect("clean reconstruct");
        assert_eq!(
            clean,
            frag.as_bytes(),
            "the uncorrupted § 201 capture must reconstruct byte-exact (control)"
        );

        // Corrupt the FIRST nested paragraph's content #PCDATA — a leaf two
        // levels deep in the subdivision tree (section → subsection(a) →
        // paragraph(1) → content). The element BACKBONE stays identical, so
        // `reapply_regenerated_complement`'s pre-order walk still succeeds, but
        // a faithful writer must now emit different bytes for that text run.
        let subsection_a = &mut title.sections[0].children[0];
        let paragraph_1 = subsection_a
            .children
            .first_mut()
            .expect("§ 201(a) must carry at least one nested <paragraph>");
        let content = paragraph_1
            .content_mixed
            .as_mut()
            .expect("the nested <paragraph> must carry a <content> leaf");
        let corrupted_a_text = content
            .nodes
            .iter_mut()
            .find_map(|n| match n {
                UsCodeContentNode::Text(t) => {
                    *t = alloc::format!("{t}-CORRUPTED");
                    Some(())
                }
                _ => None,
            })
            .is_some();
        assert!(
            corrupted_a_text,
            "the nested <paragraph>'s <content> must carry a #PCDATA Text leaf"
        );

        let corrupted = reconstruct_uslm_source(&title, &complement)
            .expect("reconstruct still runs on a corrupted-but-backbone-valid model");
        assert_ne!(
            corrupted,
            frag.as_bytes(),
            "a corrupted DEEP-subdivision #PCDATA value MUST diverge the \
             byte-exact reconstruction — the U2 subdivision gate has teeth"
        );
    }

    /// READER + WRITER CHECK (slice U2): a section carrying a real `<subsection>`
    /// subtree no longer fails closed — it regenerates. This is the U1→U2
    /// transition: the family that was `UncoveredFamily { "subsection" }` in
    /// slice U1 now writes without error.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn covered_subsection_regenerates() {
        const WITH_SUBSECTION: &str = "<section identifier=\"/us/usc/t1/s7\">\
<num value=\"7\">§ 7.</num><heading>Marriage</heading>\
<subsection identifier=\"/us/usc/t1/s7/a\"><num value=\"a\">(a)</num>\
<content>For the purposes of any Federal law.</content></subsection></section>";
        let title = read_uslm_title(WITH_SUBSECTION).expect("read");
        let doc = write_uslm(&title).expect("subsection is covered in slice U2");
        // The regenerated root is the <section>; its third child (after num,
        // heading) is the <subsection>, whose own children are num + content.
        let subsection = doc.root.children.iter().find_map(|n| match n {
            XmlNode::Element(e) if e.name.local == "subsection" => Some(e),
            _ => None,
        });
        let subsection = subsection.expect("regenerated tree carries the <subsection>");
        let inner: Vec<&str> = subsection
            .children
            .iter()
            .filter_map(|n| match n {
                XmlNode::Element(e) => Some(e.name.local.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            inner,
            ["num", "content"],
            "subsection backbone = num, content"
        );
    }

    /// HARD BYTE-EXACT GATE (slice U3): the real Title 1 § 210 — a flat-prose §
    /// (`<num>` + `<heading>` + `<content>` + `<sourceCredit>`) FOLLOWED BY a
    /// `<notes type="uscNote">` container whose two `<note>` children carry
    /// mixed-content bodies (a crossHeading note of `<heading><b>…</b></heading>`
    /// only, and a cross-reference note of `<heading>` + a `<p>` interleaving
    /// literal text with a `<ref>`) — reconstructs BYTE-FOR-BYTE from the typed
    /// [`UsCodeTitle`] + captured [`UslmSyntaxComplement`]. This proves the
    /// NOTES backbone (the `<notes>`/`<note>` family U3 adds) is faithful on a
    /// real published section: each note body regenerates node-for-node in
    /// source order from [`UsCodeNote::body_mixed`].
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn real_title1_s210_with_notes_reconstruct_is_byte_exact() {
        let Some(frag) = real_title1_section("/us/usc/t1/s210") else {
            return; // corpus not provisioned — skip gracefully
        };
        // Sanity: the fragment really is the notes family this slice targets
        // (else a corpus change silently weakened the gate).
        assert!(
            frag.contains("<notes") && frag.contains("<note "),
            "the § 210 slice must carry the <notes>/<note> backbone"
        );
        assert_byte_exact_gate(&frag);
    }

    /// Second real notes-bearing section so the U3 byte-exact gate is not
    /// single-instance: the real Title 1 § 105, whose `<notes>` carries FOUR
    /// `<note>`s across two crossHeading banners (`Editorial Notes`, `Statutory
    /// Notes and Related Subsidiaries`) and two substantive notes
    /// (`Amendments`, `Effective Date of 1974 Amendment`) — the latter a `<p>`
    /// interleaving several `<ref>` and `<date>` children. Reconstructs
    /// BYTE-FOR-BYTE, proving the per-note ordered backbone over a multi-note
    /// block.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn real_title1_s105_with_notes_reconstruct_is_byte_exact() {
        let Some(frag) = real_title1_section("/us/usc/t1/s105") else {
            return; // corpus not provisioned — skip gracefully
        };
        assert!(
            frag.contains("<notes") && frag.contains("topic=\"amendments\""),
            "the § 105 slice must carry the multi-note <notes> backbone"
        );
        assert_byte_exact_gate(&frag);
    }

    /// META-TEST (slice U3 has TEETH at the NOTE level): capture the real § 210
    /// fragment, then CORRUPT a #PCDATA Text leaf INSIDE a `<note>`'s body
    /// (here the cross-reference note's `<p>`) and assert the byte-exact
    /// reconstruction NO LONGER equals the source. This is the U3 analogue of
    /// `corrupted_mixed_breaks_byte_exact_gate` / the U2 subdivision meta-test:
    /// it proves the new notes writer reproduces the EXACT text of a note-body
    /// leaf, not merely a backbone the positional diff could reconcile.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn corrupted_note_content_breaks_byte_exact_gate() {
        let Some(frag) = real_title1_section("/us/usc/t1/s210") else {
            return;
        };
        let (mut title, complement) =
            capture_uslm_complement(&frag).expect("capture the real § 210 fragment");

        // Control: the uncorrupted capture reconstructs byte-exact.
        let clean = reconstruct_uslm_source(&title, &complement).expect("clean reconstruct");
        assert_eq!(
            clean,
            frag.as_bytes(),
            "the uncorrupted § 210 capture must reconstruct byte-exact (control)"
        );

        // Corrupt a #PCDATA Text leaf somewhere in the notes block's note
        // bodies (the cross-reference note's `<p>` carries literal prose around
        // its `<ref>`). The element BACKBONE stays identical, so
        // `reapply_regenerated_complement`'s pre-order walk still succeeds, but
        // a faithful writer must now emit different bytes for that text run.
        let block = title.sections[0]
            .notes_blocks
            .first_mut()
            .expect("§ 210 must carry a <notes> block");
        let mut corrupted_a_text = false;
        for note in &mut block.notes {
            if corrupt_first_text_in(&mut note.body_mixed.nodes) {
                corrupted_a_text = true;
                break;
            }
        }
        assert!(
            corrupted_a_text,
            "the § 210 notes block must carry a #PCDATA Text leaf in a note body"
        );

        let corrupted = reconstruct_uslm_source(&title, &complement)
            .expect("reconstruct still runs on a corrupted-but-backbone-valid model");
        assert_ne!(
            corrupted,
            frag.as_bytes(),
            "a corrupted note-body #PCDATA value MUST diverge the byte-exact \
             reconstruction — the U3 notes gate has teeth"
        );
    }

    /// Corrupt the FIRST descendant `#PCDATA` [`UsCodeContentNode::Text`] leaf
    /// in a mixed-content node list (pre-order), rewriting it to a different
    /// value. Returns whether a leaf was found and mutated. Used by the U3
    /// note-body corruption meta-test; the note body's text leaves sit one or
    /// two levels deep (inside the `<heading>` / `<p>` block children), so the
    /// walk recurses through element nodes.
    fn corrupt_first_text_in(nodes: &mut [UsCodeContentNode]) -> bool {
        for node in nodes {
            match node {
                UsCodeContentNode::Text(t) => {
                    *t = alloc::format!("{t}-CORRUPTED");
                    return true;
                }
                UsCodeContentNode::Ref { children, .. }
                | UsCodeContentNode::Date { children, .. }
                | UsCodeContentNode::Inline { children, .. }
                | UsCodeContentNode::Para { children, .. }
                | UsCodeContentNode::Generic { children, .. } => {
                    if corrupt_first_text_in(children) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// READER-MODEL CHECK (slice U3, NOT a `write_uslm` faithfulness proof). The
    /// `<note>` body is GENUINELY captured as an ordered mixed-content tree in
    /// the typed view `read_uslm_title` produces — a vacuous read (empty body)
    /// would be a lie. The real § 210 cross-reference note's body is the ordered
    /// sequence `<heading>` then `<p>`, with the `<p>` interleaving literal prose
    /// and a `<ref>`. Asserts the READER's model; the byte-exact gates above are
    /// the writer-faithfulness proof. The DERIVED flat views still hold.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reader_model_note_body_is_genuinely_mixed_content() {
        let Some(frag) = real_title1_section("/us/usc/t1/s210") else {
            return;
        };
        let title = read_uslm_title(&frag).expect("read");
        let block = title.sections[0]
            .notes_blocks
            .first()
            .expect("§ 210 carries a <notes> block");
        // The cross-reference note (topic="crossReferences") carries a heading
        // + a paragraph; its body_mixed holds those two block nodes in order.
        let xref = block
            .notes
            .iter()
            .find(|n| n.topic.as_deref() == Some("crossReferences"))
            .expect("§ 210 notes include the crossReferences note");
        let block_kids: Vec<&str> = xref
            .body_mixed
            .nodes
            .iter()
            .filter_map(|n| match n {
                UsCodeContentNode::Generic { name, .. } => Some(name.as_str()),
                UsCodeContentNode::Para { .. } => Some("p"),
                _ => None,
            })
            .collect();
        assert_eq!(
            block_kids,
            ["heading", "p"],
            "the crossReferences note body = <heading> then <p>, in order"
        );
        // The <p> is a Para whose mixed children interleave literal text with a
        // <ref> citation edge (the backbone the writer reproduces).
        let para = xref
            .body_mixed
            .nodes
            .iter()
            .find_map(|n| match n {
                UsCodeContentNode::Para { children, .. } => Some(children),
                _ => None,
            })
            .expect("the crossReferences note carries a <p> body");
        assert!(
            para.iter()
                .any(|n| matches!(n, UsCodeContentNode::Ref { .. })),
            "the note <p> interleaves a <ref> citation in position"
        );
        assert!(
            para.iter()
                .any(|n| matches!(n, UsCodeContentNode::Text(t) if !t.trim().is_empty())),
            "the note <p> interleaves literal #PCDATA around the <ref>"
        );
        // The DERIVED flat projections still hold (downstream stays working).
        assert!(
            xref.heading.as_deref() == Some("Cross References"),
            "flat note heading still derived: {:?}",
            xref.heading
        );
        assert!(
            !xref.refs.is_empty(),
            "flat note refs still derived from the body"
        );
    }

    /// READER + WRITER CHECK (slice U3): a section carrying a `<notes>` block no
    /// longer fails closed — it regenerates. This is the U2→U3 transition: the
    /// family that was `UncoveredFamily { "notes" }` in slices U1 + U2 now
    /// writes without error, emitting the `<notes>`/`<note>` backbone in source
    /// order after `<sourceCredit>`.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn covered_notes_regenerates() {
        const WITH_NOTES: &str = "<section identifier=\"/us/usc/t1/s7\">\
<num value=\"7\">§ 7.</num><heading>Marriage</heading>\
<subsection identifier=\"/us/usc/t1/s7/a\"><num value=\"a\">(a)</num>\
<content>For the purposes of any Federal law.</content></subsection>\
<notes type=\"uscNote\"><note topic=\"amendments\"><heading>Amendments</heading>\
<p>Some editorial note.</p></note></notes>\
</section>";
        let title = read_uslm_title(WITH_NOTES).expect("read");
        let doc = write_uslm(&title).expect("notes is covered in slice U3");
        // The regenerated root carries the <notes> container after the
        // <subsection>; its single <note> child regenerates heading + p.
        let notes = doc
            .root
            .children
            .iter()
            .find_map(|n| match n {
                XmlNode::Element(e) if e.name.local == "notes" => Some(e),
                _ => None,
            })
            .expect("regenerated tree carries the <notes> container");
        let note = notes
            .children
            .iter()
            .find_map(|n| match n {
                XmlNode::Element(e) if e.name.local == "note" => Some(e),
                _ => None,
            })
            .expect("the <notes> container carries a <note>");
        let inner: Vec<&str> = note
            .children
            .iter()
            .filter_map(|n| match n {
                XmlNode::Element(e) => Some(e.name.local.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            inner,
            ["heading", "p"],
            "note backbone = heading, p (in source order)"
        );
    }

    /// An uncovered family STILL fails CLOSED (honest-partial): a section
    /// carrying a `<continuation>` flush — a family slices U1–U3 do NOT
    /// regenerate (and which is ABSENT from LRC Title 1, so writing it
    /// un-proven would be dishonest) — returns
    /// [`UslmWriteError::UncoveredFamily`] rather than emitting a short backbone
    /// the diff would silently mismatch. Proves the fail-closed boundary still
    /// has teeth now that notes are covered.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn uncovered_continuation_fails_closed() {
        const WITH_CONTINUATION: &str = "<section identifier=\"/us/usc/t1/s7\">\
<num value=\"7\">§ 7.</num><heading>Marriage</heading>\
<subsection identifier=\"/us/usc/t1/s7/a\"><num value=\"a\">(a)</num>\
<content>For the purposes of any Federal law.</content></subsection>\
<continuation>and shall remain in force.</continuation>\
</section>";
        let title = read_uslm_title(WITH_CONTINUATION).expect("read");
        let err = write_uslm(&title).expect_err("continuation is uncovered in slices U1–U3");
        assert!(
            matches!(err, UslmWriteError::UncoveredFamily { ref family, .. } if family == "continuation"),
            "got {err:?}"
        );
    }

    // ── SLICE U4: the full `<uscDoc>` document wrapper ───────────────────────

    /// The on-disk Title 1 USLM corpus path.
    fn real_title1_path() -> std::path::PathBuf {
        workspace_root()
            .join("crates/domains/data/legal/uscode/usc_title_1/usc_title_1-pl-119-90.xml")
    }

    /// The LITERAL on-disk Title 1 USLM file — the raw bytes EXACTLY as published,
    /// CRLFs included (the file carries two `#xD#xA` line endings in the prolog,
    /// at the `?>` boundaries of the XML declaration and the `<?xml-stylesheet?>`
    /// PI). `None` when the corpus file is absent (graceful skip) or not UTF-8.
    ///
    /// Slice U5 captures the W3C XML 1.0 §2.11 \[2.11\] end-of-line FORM, so the
    /// literal file — CRLFs and all — now reconstructs byte-for-byte; this is the
    /// honest whole-document the byte-exact gate runs over.
    fn real_title1_literal() -> Option<String> {
        String::from_utf8(std::fs::read(real_title1_path()).ok()?).ok()
    }

    /// The real on-disk Title 1 USLM file with its §2.11 \[2.11\] end-of-line
    /// normalisation applied (`\r\n` → `\n`). `None` when the corpus file is
    /// absent (graceful skip).
    ///
    /// Used by the wrapper-skeleton and backbone-corruption tests that assert on
    /// the parsed STRUCTURE rather than the literal bytes — the §2.11 EOL form is
    /// orthogonal to those, so the LF-normalised view keeps them stable. The
    /// LITERAL byte-exact gate uses [`real_title1_literal`].
    fn real_title1_lf_normalized() -> Option<String> {
        Some(
            real_title1_literal()?
                .replace("\r\n", "\n")
                .replace('\r', "\n"),
        )
    }

    /// HARD BYTE-EXACT GATE (slices U4 + U5): the WHOLE real Title 1 `<uscDoc>`
    /// document — `<?xml-stylesheet?>` prolog PI, the interleaved-attribute
    /// `<uscDoc>` root, the `<meta>` block, `<main>` → `<title>` (its `<num>` /
    /// `<heading>` / title-level `<note>`s / `<toc>` / the three `<chapter>`
    /// hierarchy containers grouping every `<section>`) — reconstructs
    /// BYTE-FOR-BYTE from `capture_uslm_complement` then `reconstruct_uslm_source`,
    /// over the LITERAL on-disk file — CRLFs INCLUDED (the two prolog `#xD#xA`
    /// line endings).
    ///
    /// U4 proved the document-wrapper BACKBONE over the §2.11-LF-normalised file;
    /// U5 closes the last 2-byte gap by capturing the W3C XML 1.0 §2.11 \[2.11\]
    /// end-of-line FORM, so the gate now runs over the raw published bytes with NO
    /// carve-out. It exercises every U5 path that matters together: the
    /// `<?xml-stylesheet?>` PI via the prolog-`Misc*` capture, the interleaved
    /// `<uscDoc>` start-tag via `start_tag_order`, every `<meta>` / `<main>` /
    /// `<title>` / `<chapter>` / `<section>` element via the semantic mixed-content
    /// backbone, AND the prolog CRLFs via the generic [`EndOfLineForm`] residue.
    ///
    /// On failure it reports the EXACT first byte-diff (a bounded 80-byte window)
    /// so an uncaptured concrete-syntax species or uncovered family names itself.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn real_title1_full_uscdoc_reconstruct_is_byte_exact() {
        let Some(frag) = real_title1_literal() else {
            return; // corpus not provisioned — skip gracefully
        };
        // Sanity: this really is the full `<uscDoc>` document (the wrapper this
        // slice targets), not a section slice — else a corpus change silently
        // weakened the gate.
        assert!(
            frag.contains("<?xml-stylesheet")
                && frag.contains("<uscDoc ")
                && frag.contains("<meta>")
                && frag.contains("<main>")
                && frag.contains("<chapter"),
            "the U4+U5 gate must run over the full <uscDoc> document wrapper"
        );
        // …and that it genuinely carries the prolog CRLF the U5 EOL form must put
        // back — else a corpus re-export to pure LF would silently make the gate
        // vacuous over the §2.11 form (the byte kernel's whole point this slice).
        assert!(
            frag.contains("?>\r\n"),
            "the U5 literal gate must run over a file that genuinely carries CRLF \
             (the §2.11 end-of-line form this slice captures)"
        );
        assert_byte_exact_gate(&frag);
    }

    /// The wrapper regenerates the `<uscDoc>` / `<meta>` / `<main>` / `<title>`
    /// SKELETON from the captured backbone — proving the document-wrapper shape
    /// (not just a single section) is reconstructed. Asserts the regenerated tree
    /// has the `<uscDoc>` root whose first two element children are `<meta>` then
    /// `<main>`, and that `<main>` carries a `<title>` whose first two element
    /// children are `<num>` then `<heading>`.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn full_uscdoc_wrapper_skeleton_regenerates() {
        let Some(frag) = real_title1_lf_normalized() else {
            return;
        };
        let title = read_uslm_title(&frag).expect("read full uscDoc");
        let doc = write_uslm(&title).expect("the full <uscDoc> wrapper is covered in slice U4");
        assert_eq!(doc.root.name.local, "uscDoc", "root is <uscDoc>");
        let root_kids: Vec<&str> = element_child_names(&doc.root);
        assert_eq!(
            root_kids,
            ["meta", "main"],
            "the <uscDoc> root regenerates <meta> then <main>"
        );
        let main = find_child(&doc.root, "main").expect("<main> regenerated");
        let title_el = find_child(main, "title").expect("<title> regenerated under <main>");
        let title_kids = element_child_names(title_el);
        assert_eq!(
            &title_kids[..2],
            ["num", "heading"],
            "the <title> regenerates <num> then <heading> first"
        );
    }

    /// META-TEST (slice U4 has TEETH at the WRAPPER level): capture the full real
    /// `<uscDoc>` document, then CORRUPT a deep #PCDATA Text leaf in the captured
    /// backbone (the FIRST text leaf anywhere in `uscdoc_mixed`) and assert the
    /// byte-exact reconstruction NO LONGER equals the source. The U4 analogue of
    /// the §/subdivision/note corruption meta-tests: proves the wrapper writer
    /// reproduces the EXACT text of a deep document leaf, not merely a backbone
    /// the positional diff could reconcile.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn corrupted_uscdoc_backbone_breaks_byte_exact_gate() {
        let Some(frag) = real_title1_lf_normalized() else {
            return;
        };
        let (mut title, complement) =
            capture_uslm_complement(&frag).expect("capture the full <uscDoc> document");

        // Control: the uncorrupted capture reconstructs byte-exact.
        let clean = reconstruct_uslm_source(&title, &complement).expect("clean reconstruct");
        assert_eq!(
            clean,
            frag.as_bytes(),
            "the uncorrupted full-<uscDoc> capture must reconstruct byte-exact (control)"
        );

        // Corrupt the first #PCDATA Text leaf in the document backbone. The
        // element BACKBONE stays identical, so the complement's pre-order walk
        // still succeeds, but a faithful writer must now emit different bytes.
        let uscdoc = title
            .uscdoc_mixed
            .as_mut()
            .expect("the full document carries a captured <uscDoc> backbone");
        let corrupted = corrupt_first_text_in(&mut uscdoc.nodes);
        assert!(
            corrupted,
            "the <uscDoc> backbone must carry a #PCDATA Text leaf to corrupt"
        );

        let out = reconstruct_uslm_source(&title, &complement)
            .expect("reconstruct still runs on a corrupted-but-backbone-valid model");
        assert_ne!(
            out,
            frag.as_bytes(),
            "a corrupted document-backbone #PCDATA value MUST diverge the byte-exact \
             reconstruction — the U4 wrapper gate has teeth"
        );
    }

    /// The `<?xml-stylesheet?>` prolog PI is GENUINELY captured into the prolog
    /// `Misc*` residue (a vacuous round-trip that dropped it would be a lie), and
    /// the interleaved `<uscDoc>` start-tag is GENUINELY captured as a
    /// non-canonical `start_tag_order` (proving the co-location handling fires).
    /// Asserts the generic byte-kernel additions are exercised by the real
    /// document, not merely present.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn full_uscdoc_prolog_pi_and_root_colocation_are_captured() {
        let Some(frag) = real_title1_lf_normalized() else {
            return;
        };
        let (_title, complement) =
            capture_uslm_complement(&frag).expect("capture the full <uscDoc> document");
        // (1) The prolog `Misc*` residue carries the `<?xml-stylesheet?>` PI in
        // position (the generic prolog-PI byte-kernel addition).
        assert!(
            complement
                .syntax_decisions
                .prolog()
                .after_xml_decl
                .contains("<?xml-stylesheet"),
            "the <?xml-stylesheet?> prolog PI must be captured in the prolog Misc* residue"
        );
        // (2) The `<uscDoc>` root (pre-order element index 0) records a
        // non-canonical `start_tag_order` — an `xmlns` decl follows the
        // `xsi:schemaLocation`/`xml:lang`/`identifier` attributes, so the
        // co-location complement fired.
        let root_decisions = complement
            .syntax_decisions
            .get(0)
            .expect("the interleaved <uscDoc> root records concrete-syntax decisions");
        assert!(
            root_decisions.start_tag_order.is_some(),
            "the <uscDoc> root's interleaved xmlns/attribute order must be captured \
             as a non-canonical start_tag_order"
        );
    }

    /// The element-child local names of `el`, in document order (white-space
    /// `Text` siblings elided) — a small test helper for the wrapper skeleton
    /// assertions.
    fn element_child_names(el: &XmlElement) -> Vec<&str> {
        el.children
            .iter()
            .filter_map(|n| match n {
                XmlNode::Element(e) => Some(e.name.local.as_str()),
                _ => None,
            })
            .collect()
    }

    /// The first element child of `el` whose local name is `name`, if any.
    fn find_child<'a>(el: &'a XmlElement, name: &str) -> Option<&'a XmlElement> {
        el.children.iter().find_map(|n| match n {
            XmlNode::Element(e) if e.name.local == name => Some(e),
            _ => None,
        })
    }

    /// THE CARVE-OUT IS NOW ZERO (slice U5): the LITERAL on-disk Title 1 file —
    /// CRLFs included — reconstructs byte-for-byte with NO divergence, because the
    /// W3C XML 1.0 §2.11 \[2.11\] end-of-line FORM is now captured. Where U4 had a
    /// 2-byte prolog-CRLF carve-out, U5 closes it: the reconstruction equals the
    /// raw bytes exactly (`out.len() == src.len()`, no first-diff), AND the two
    /// prolog `#xD#xA` line endings are genuinely re-expanded from the
    /// [`EndOfLineForm`] residue (not a fluke of an LF-only file).
    ///
    /// This is the U5 proof at the residue level (the whole-document gate
    /// [`real_title1_full_uscdoc_reconstruct_is_byte_exact`] proves the bytes; this
    /// proves the §2.11 form is the thing that closed the gap — it has teeth).
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn literal_title1_crlf_round_trips_via_eol_form() {
        let Some(src) = real_title1_literal() else {
            return; // corpus not provisioned — skip gracefully
        };
        // The on-disk file genuinely carries CRLF in the prolog (else this gate is
        // vacuous — a corpus re-export to pure LF would make the literal file
        // already byte-exact without any §2.11 form to capture).
        assert!(
            src.contains("?>\r\n"),
            "the on-disk Title 1 file must carry the prolog CRLF this slice round-trips"
        );
        let (title, complement) =
            capture_uslm_complement(&src).expect("capture the literal on-disk file");

        // The §2.11 EOL form genuinely captured the prolog CRLFs — a vacuous
        // round-trip that dropped them would be a lie. Title 1 carries exactly two
        // `#xD#xA` (prolog), no lone `#xD`, so the form lists two `Crlf` entries.
        let eol = complement.syntax_decisions.eol_form();
        assert!(
            !eol.is_empty(),
            "the literal file's prolog CRLFs must be captured in the §2.11 EOL form"
        );
        assert!(
            eol.eols.iter().all(|(_, k)| matches!(
                k,
                crate::social::software::markup::xml::parser::source_syntax::EolKind::Crlf
            )),
            "Title 1's only line breaks are CRLF (the prolog `?>\\r\\n`), no lone CR"
        );

        let out = reconstruct_uslm_source(&title, &complement).expect("reconstruct");
        let sb = src.as_bytes();
        // ZERO gap: the reconstruction is byte-for-byte the literal file, CRLFs and
        // all — no length delta, no first divergence.
        assert_eq!(
            out.len(),
            sb.len(),
            "the literal reconstruction must be exactly the source length (CRLFs put back)"
        );
        assert_eq!(
            out,
            sb.to_vec(),
            "the literal on-disk Title 1 file must reconstruct byte-for-byte (U5: the \
             §2.11 end-of-line carve-out is now zero)"
        );
    }

    // The HARD BYTE-EXACT flipped-title gates (slice U7) — the clean
    // byte-for-byte reconstruct over four mid-size USC titles + its
    // corruption-divergence meta-test — re-capture and re-emit those titles, so
    // they live in the heavy-corpus lane: see
    // `crates/praxis-corpus-tests/tests/usc_flipped_titles.rs`. The
    // `corrupt_first_text_in` helper above is shared with the bare-section
    // corruption meta-tests that STAY in this module, so it remains here (a copy
    // travels with the moved tests).
}
