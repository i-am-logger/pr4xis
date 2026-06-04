//! USLM structural writer (slice U1) — the XML-tree-level inverse of
//! [`read_uslm_title`], the USC analogue of the
//! WN-LMF [`write_wordnet_document`](crate::social::software::markup::xml::lmf::writer::write_wordnet_document).
//!
//! [`write_uslm`] folds a typed [`UsCodeTitle`] back onto the generic XML
//! ontology ([`XmlDocument`] / [`XmlElement`] / [`XmlNode`]), regenerating the
//! element BACKBONE — element names, pre-order, and the EXACT ordered
//! mixed-content child sequence (text-run ↔ inline-element interleaving) — so
//! that, paired with the generic SourceSyntax complement, it closes a
//! byte-exact graph-faithful `put` (Foster, Greenwald, Moore, Pierce & Schmitt
//! 2007 ACM TOPLAS 29(3) §2.2).
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
//! # Coverage (slice U1)
//!
//! This slice covers the FLAT-section content model the LRC USC titles use
//! for prose sections: a `<section>` of `<num>` + `<heading>` + `<content>`
//! (with a block `<p>`) + `<sourceCredit>` — the last being TRUE MIXED
//! CONTENT (`<ref>` / `<date>` interleaved with literal punctuation). The
//! covered text-bearing families (`<num>`, `<heading>`, `<content>` / `<p>`,
//! `<sourceCredit>` with `<ref>` / `<date>`) regenerate from the
//! [`UsCodeMixed`] semantic trees the reader captured. The remaining USLM
//! vocabulary (subdivisions, notes, tables, the rest of the ~50-element set)
//! is the next slice; a section that exercises an uncovered family surfaces as
//! a backbone divergence in the gate, never a silent drop.
//!
//! # Child order
//!
//! [`diff_element`](crate::social::software::markup::xml::parser::source_syntax)
//! matches children strictly POSITIONALLY (no reorder species), so this writer
//! emits the section's children in EXACT source order. For the covered flat
//! sections that order is the canonical USLM content order
//! `num, heading, content, sourceCredit` (LRC USLM XML User Guide § "Section
//! Structure") — reconstructed from the typed model directly, with NO
//! `ChildOrder` residue added to the generic complement. The gate over the
//! real Title 1 sections proves that order is faithful for the covered family.
//!
//! # Citations
//!
//! - **Foster, J. N.; Greenwald, M. B.; Moore, J. T.; Pierce, B. C.; Schmitt,
//!   A. (2007)** — ACM TOPLAS 29(3) Article 17 §2.2 (well-behaved lens `put`).
//! - **Cowan, J.; Tobin, R. (2004)** — XML Information Set (the items the
//!   typed model carries; attribute order / white-space are not among them).
//! - **Bray et al. (2008)** — XML 1.0 Fifth Edition §3.2.2 (Mixed Content).
//! - **U.S. House Office of the Law Revision Counsel** — *USLM XML User Guide
//!   and Schema*. <https://uscode.house.gov/uslm/>.

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec, vec::Vec};

use super::super::corpus::{
    InlineKind, UsCodeContentNode, UsCodeMixed, UsCodeSection, UsCodeSourceCredit, UsCodeTitle,
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
    /// The typed title carried a structure slice U1 does not regenerate yet
    /// (a subdivision tree, a notes block, a table, …). Carries the family
    /// name so the gate names the next slice.
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
                "write_uslm slice U1 does not yet regenerate the <{family}> family \
                 (section {section}); that is the next slice"
            ),
        }
    }
}

impl std::error::Error for UslmWriteError {}

/// Write a [`UsCodeTitle`] back to a USLM [`XmlDocument`] — the XML-tree-level
/// inverse of [`read_uslm_title`].
///
/// Slice U1 targets the SECTION-SLICE shape `read_uslm_title` produces for a
/// bare `<section>` document (root = the section): the title's single
/// `sections` entry is regenerated as the document root. The canonical
/// `version="1.0" encoding="UTF-8"` prolog mirrors the reader's XML
/// declaration handling.
///
/// Returns [`UslmWriteError::UncoveredFamily`] when the section carries a
/// structure slice U1 does not regenerate — the honest-partial boundary that
/// keeps the backbone diff fail-closed instead of dropping content.
pub fn write_uslm(title: &UsCodeTitle) -> Result<XmlDocument, UslmWriteError> {
    // Slice U1 covers the section-slice document shape: exactly one section,
    // emitted as the root. (A full `<uscDoc>`/`<title>` wrapper is a later
    // slice — its meta/main backbone is not yet regenerated.)
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
/// Emits the canonical USLM flat-section child order
/// (`num, heading, content, sourceCredit`) the covered Title 1 prose sections
/// carry. A section with any structure slice U1 does not regenerate
/// (subdivisions, notes, continuations, def/marker/amendment markup) returns
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

    // <content>…</content> — regenerated from its mixed tree (carries the
    // block <p> in position). Emitted only when the source carried one.
    if let Some(content) = &section.content_mixed {
        children.push(XmlNode::Element(mixed_element("content", content)));
    }

    // <sourceCredit>…</sourceCredit> — TRUE MIXED CONTENT regenerated from its
    // exact ordered tree (literal punctuation interleaved with <ref>/<date>).
    for credit in &section.source_credits {
        children.push(XmlNode::Element(source_credit_element(credit)));
    }

    Ok(element("section", section_attrs(section), children))
}

/// Fail closed when the section carries a structure family slice U1 does not
/// regenerate yet. Each is reported by family name so the gate names exactly
/// what the next slice must cover, rather than the writer silently emitting a
/// shorter backbone (which the positional diff would then reject downstream
/// with a less specific message).
fn reject_uncovered(section: &UsCodeSection) -> Result<(), UslmWriteError> {
    let uncovered = if !section.children.is_empty() {
        Some("subsection")
    } else if section.chapeau_mixed.is_some() {
        Some("chapeau")
    } else if !section.notes_blocks.is_empty() {
        Some("notes")
    } else if !section.bare_notes.is_empty() {
        Some("note")
    } else if !section.continuations.is_empty() {
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
fn content_element(
    name: &str,
    attrs: &[super::super::corpus::UsCodeContentAttr],
    children: &[UsCodeContentNode],
) -> XmlElement {
    let attributes = attrs.iter().map(|a| attr(&a.name, &a.value)).collect();
    let kids = children.iter().map(content_node).collect();
    element(name, attributes, kids)
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

/// Build a namespace-free [`XmlElement`]. The typed USC model carries no
/// namespace declarations of its own — the root's `xmlns` set is restored by
/// the generic [`DocumentResidue`] complement.
fn element(name: &str, attributes: Vec<XmlAttribute>, children: Vec<XmlNode>) -> XmlElement {
    XmlElement {
        name: XmlName::new(name),
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
// TOPLAS 29(3) §2.2). The parser-level residue machinery is generic XML-family
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
    /// carries a structure family slice U1 does not yet cover (a subsection
    /// tree, a notes block, …). Carries the [`UslmWriteError`] naming the next
    /// slice; this is the honest-partial boundary, never a silent short tree.
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

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Lowercase-hex SHA-256 (NIST FIPS 180-4 §6.2) of `bytes` — mirrors the
    /// WN-LMF corpus gate's headline hash, computed directly via the crate's
    /// `sha2` dependency (no `prx`-feature coupling).
    fn sha256_hex(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(bytes);
        let mut s = String::with_capacity(64);
        for byte in digest {
            s.push_str(&alloc::format!("{byte:02x}"));
        }
        s
    }

    /// THE BYTE-EXACT GATE over one real section fragment, mirroring the WN-LMF
    /// `reconstruct_*_byte_exact` gates: `capture_uslm_complement(frag)` then
    /// `reconstruct_uslm_source(&title, &complement)` must equal `frag` BYTE-FOR
    /// -BYTE (SHA-256-pinned). This proves `write_uslm` FAITHFULNESS — a
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
        // SHA-256 equality is the headline assertion. `out == source` already
        // implies it; the explicit pin guards against a silent corpus swap.
        assert_eq!(
            sha256_hex(&out),
            sha256_hex(source),
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
    #[test]
    fn real_title1_s2_reconstruct_is_byte_exact() {
        let Some(frag) = real_title1_section("/us/usc/t1/s2") else {
            return; // corpus not provisioned — skip gracefully
        };
        assert_byte_exact_gate(&frag);
    }

    /// Second real section so the byte-exact gate is not single-instance.
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

    /// An uncovered family fails CLOSED (honest-partial): a section carrying
    /// a subsection subtree (the next slice) returns
    /// [`UslmWriteError::UncoveredFamily`] rather than emitting a short
    /// backbone the diff would silently mismatch.
    #[test]
    fn uncovered_subsection_fails_closed() {
        const WITH_SUBSECTION: &str = "<section identifier=\"/us/usc/t1/s7\">\
<num value=\"7\">§ 7.</num><heading>Marriage</heading>\
<subsection identifier=\"/us/usc/t1/s7/a\"><num value=\"a\">(a)</num>\
<content>For the purposes of any Federal law.</content></subsection></section>";
        let title = read_uslm_title(WITH_SUBSECTION).expect("read");
        let err = write_uslm(&title).expect_err("subsection is uncovered in slice U1");
        assert!(
            matches!(err, UslmWriteError::UncoveredFamily { ref family, .. } if family == "subsection"),
            "got {err:?}"
        );
    }
}
