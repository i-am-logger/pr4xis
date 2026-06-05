//! RDF/XML structural writer — the byte-exact graph-faithful `put` for an
//! OWL/RDF source, the OWL leaf of #186's graph-faithful tier (the sibling of
//! the WN-LMF [`lmf::writer`](crate::social::software::markup::xml::lmf::writer)
//! and the USLM `uslm::writer`).
//!
//! # The problem the WordNet writer does not have
//!
//! The WN-LMF graph-faithful writer regenerates its element backbone from the
//! **typed [`WordNet`] ontology**, which is rich enough to reproduce every
//! `<LexicalEntry>`/`<Synset>` element. The OWL [`OwlOntology`] typed view is
//! NOT: the W3C OWL 2 RDF Mapping (Patel-Schneider & Motik 2012) is a *lossy*
//! projection — it keeps the classes/properties/individuals/restrictions it
//! recognises and drops every other triple (versioning, imports, blank-node
//! annotation, the raw `rdf:first`/`rdf:rest` list cells, …). Empirically the
//! typed view drops 24–1187 triples per bundled vocab (see
//! [`super::lens`]). So the OWL backbone cannot be regenerated from
//! [`OwlOntology`] alone.
//!
//! # What IS faithful: the RDF/XML serialization structure
//!
//! An RDF/XML document serialises an RDF graph by *striping* it into a
//! document-ordered sequence of **node elements** (`<rdf:Description>` /
//! typed-node form), each carrying a document-ordered sequence of **property
//! elements** (`<rdfs:label>`, `<owl:inverseOf>`, …). That striping — which
//! triples land in which `<rdf:Description>` block, in what source order, with
//! which prefix on each element QName, and which property elements are empty
//! (an `rdf:resource`/`rdf:nodeID` reference) versus leaf-text literals — is
//! the **concrete syntax of the serialization**, NOT the graph's meaning (the
//! same graph serialises many ways; RDF 1.1 §3.6 graph isomorphism). It is
//! exactly the residue the typed ontology must not carry, and exactly what a
//! structural writer needs to reproduce the element backbone.
//!
//! [`RdfXmlStructure`] captures that striping as a TYPED, structured projection
//! (NOT a stored [`XmlDocument`] DOM, NOT raw bytes): per node element its QName
//! and its ordered property elements; per property element its QName and whether
//! it is empty or carries leaf text (the literal lexical form). Attribute
//! *values* are NOT in this structure — they are the generic
//! [`AttributeOverrides`] residue (the source `rdf:about`/`rdf:nodeID`/
//! `rdf:resource`/`rdf:datatype`/`xml:lang` sequences), because XML attribute
//! order/coverage is concrete-syntax (Cowan & Tobin 2004 §2.3), not Infoset.
//!
//! [`write_owl_document`] folds [`RdfXmlStructure`] back to an [`XmlDocument`]
//! whose element backbone equals the source's; the generic residue machinery
//! ([`diff_content_whitespace`] / [`reapply_regenerated_complement`] /
//! [`serialize_document_exact`], the same byte kernel proven over the 89 MB
//! WordNet corpus and the USC titles) then closes the byte-exact loop.
//!
//! # Layering — what is new vs reused
//!
//! Everything below the structural fold is REUSED VERBATIM:
//! - [`parse_document_capturing`] — the exact Infoset DOM + [`SyntaxDecisions`];
//! - [`diff_content_whitespace`] — inter-element white-space + attribute
//!   overrides (the source `rdf:about`/`rdf:resource`/`rdf:nodeID`/`rdf:datatype`
//!   /`xml:lang` reference forms, captured exactly);
//! - [`reapply_regenerated_complement`] + [`serialize_document_exact`] — the
//!   byte kernel (DOCTYPE, root namespaces + their declaration order, the
//!   multi-line `<rdf:RDF>` intra-tag layout, the §4.6 entity-reference form,
//!   the §2.11 EOL form, the prolog/epilog `Misc*`).
//!
//! Only [`RdfXmlStructure`] (the structured RDF/XML striping projection) and the
//! [`capture_owl_complement`] / [`reconstruct_owl_rdfxml_source`] glue are new,
//! mirroring [`capture_wn_complement`] / [`reconstruct_wn_lmf_source`].
//!
//! # Citations
//!
//! - **Gandon & Schreiber (eds.) (2014)** *RDF 1.1 XML Syntax*, W3C
//!   Recommendation 2014-02-25 — §2.10 node elements / §2.13 typed nodes /
//!   §3.4 property elements (the striping this structure projects).
//! - **Patel-Schneider & Motik (eds.) (2012)** *OWL 2 Mapping to RDF Graphs*,
//!   W3C Recommendation 2012-12-11 — the lossy typed mapping the structure
//!   complements.
//! - **Cyganiak, Wood & Lanthaler (eds.) (2014)** *RDF 1.1 Concepts*, §3.6
//!   graph isomorphism (the same graph, many serializations).
//! - **Cowan & Tobin (2004)** *XML Information Set* §2.3 — attribute order is
//!   not an Infoset item, hence concrete-syntax residue.
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** *ACM TOPLAS* 29(3)
//!   §2.2 — the well-behaved lens whose graph-faithful `put` this realises.
//!
//! [`WordNet`]: crate::social::software::markup::xml::lmf::ontology::WordNet
//! [`OwlOntology`]: super::ontology::OwlOntology
//! [`capture_wn_complement`]: crate::social::software::markup::xml::lmf::writer::capture_wn_complement
//! [`reconstruct_wn_lmf_source`]: crate::social::software::markup::xml::lmf::writer::reconstruct_wn_lmf_source

#[allow(unused_imports)]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::social::software::markup::xml::ontology::{XmlDocument, XmlElement, XmlName, XmlNode};
use crate::social::software::markup::xml::parser::grammar::{
    XmlParseError, parse_document_capturing,
};
use crate::social::software::markup::xml::parser::serializer::serialize_document_exact;
use crate::social::software::markup::xml::parser::source_syntax::{
    DocumentResidue, RegeneratedComplement, RegeneratedComplementError, SyntaxDecisions,
    diff_content_whitespace, reapply_regenerated_complement,
};

// =============================================================================
// RdfXmlStructure — the structured RDF/XML striping projection.
// =============================================================================

/// The content shape of one property element (RDF/XML §3.4 `propertyElt`): the
/// typed ontology does not carry HOW the object was serialised, so the residue
/// records it.
///
/// - [`Empty`](Self::Empty) — an empty element `<rdfs:subPropertyOf
///   rdf:resource="…"/>` (or `rdf:nodeID`): the object is an IRI/blank-node
///   reference carried entirely in the (overridden) attributes, no child nodes.
/// - [`Text`](Self::Text) — a literal property element `<rdfs:label>gives
///   support to</rdfs:label>`: a single leaf-text child whose value is the
///   literal's lexical form (the only #PCDATA the typed model would carry as a
///   `Keep` child).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum PropertyContent {
    /// An empty property element — `rdf:resource`/`rdf:nodeID` reference form.
    Empty,
    /// A literal property element carrying one leaf-text run (the lexical form).
    Text(String),
}

/// One property element of a node block (RDF/XML §3.4 `propertyElt`).
///
/// Carries its qualified element name (the predicate's prefix + local — a
/// serialization choice the IRI alone cannot recover) and its
/// [`PropertyContent`]. Attribute values (`rdf:resource`/`rdf:nodeID`/
/// `rdf:datatype`/`xml:lang`) are NOT here — they are the generic
/// [`AttributeOverrides`](crate::social::software::markup::xml::parser::source_syntax::AttributeOverrides)
/// residue, slot-aligned by element pre-order index.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct RdfPropertyElement {
    /// The property element's qualified name (`rdfs:label`, `owl:inverseOf`, …).
    pub name: XmlName,
    /// Empty (reference) or a leaf-text literal.
    pub content: PropertyContent,
}

/// One node element of the RDF/XML serialization (RDF/XML §2.10 node element):
/// a `<rdf:Description>` (or typed-node) block plus its ordered property
/// elements.
///
/// The block's *identity* (its `rdf:about`/`rdf:nodeID` subject) lives in the
/// overridden attributes, not here — a blank node serialised as two separate
/// `<rdf:Description rdf:nodeID="N…">` blocks (cito has 9 such nodeIDs, each
/// defined once and referenced once) is TWO node blocks in document order, so
/// the backbone count is preserved (a triple-grouped view would wrongly merge
/// them).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct RdfNodeBlock {
    /// The node element's qualified name (`rdf:Description`, or a typed-node
    /// QName like `owl:Class` — cito is all `rdf:Description`).
    pub name: XmlName,
    /// The block's property elements in source document order.
    pub properties: Vec<RdfPropertyElement>,
}

/// The structured RDF/XML serialization striping of an OWL source — the typed,
/// content-addressed projection [`write_owl_document`] regenerates the element
/// backbone from.
///
/// This is the OWL realisation of #186's graph-faithful concrete-syntax
/// residue: NOT the lossy typed [`OwlOntology`] (which drops triples), NOT a
/// stored [`XmlDocument`] DOM (no attributes, no white-space — those are
/// generic residue), NOT raw bytes. It is the *serialization structure*: the
/// document-ordered node-block / property-element backbone plus per-property
/// leaf text, captured so a structural writer reproduces the source's exact
/// element tree.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct RdfXmlStructure {
    /// The root element's qualified name (`rdf:RDF` for every bundled vocab).
    pub root_name: XmlName,
    /// The node blocks in document order.
    pub node_blocks: Vec<RdfNodeBlock>,
}

// =============================================================================
// Structural projection — exact DOM → RdfXmlStructure.
// =============================================================================

/// Failure projecting a source DOM to an [`RdfXmlStructure`] — a shape the
/// flat-striping structural writer does not model (a NESTED node element inside
/// a property element, i.e. a non-flat RDF/XML serialization, or non-leaf mixed
/// content). cito (and the SPAR vocab family) serialise every node — named and
/// blank — as a FLAT top-level `<rdf:Description>` under `<rdf:RDF>`, with no
/// `parseType` and no nested typed nodes, so the flat projection is exact. A
/// vocab that nests (a striped sub-node, `parseType="Resource"`/`"Collection"`
/// inline content) is honestly rejected here (the byte-exact tier is earned
/// per-source; the rejected source rides the floor), never silently flattened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RdfXmlStructureError {
    /// The source bytes did not parse as well-formed XML (§2.1).
    Parse(XmlParseError),
    /// A node block contained a non-flat shape: a child of the root that is not
    /// an element node, or a property element with element children (a nested
    /// node element / inline `parseType` content) the flat writer cannot
    /// reproduce. Carries the offending element's qualified name.
    NonFlat { element: String },
    /// A property element carried more than one text run or mixed text and
    /// element content — not the single-leaf-literal shape the structure models.
    MixedContent { element: String },
}

impl core::fmt::Display for RdfXmlStructureError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "RDF/XML source parse: {e}"),
            Self::NonFlat { element } => write!(
                f,
                "non-flat RDF/XML serialization at <{element}> — the flat structural \
                 writer models only top-level rdf:Description blocks with leaf property \
                 elements (this source rides the raw-bytes floor)"
            ),
            Self::MixedContent { element } => write!(
                f,
                "mixed content at property element <{element}> — the structure models a \
                 single leaf-text literal or an empty reference element only"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RdfXmlStructureError {}

impl From<XmlParseError> for RdfXmlStructureError {
    fn from(e: XmlParseError) -> Self {
        Self::Parse(e)
    }
}

/// Project a parsed RDF/XML [`XmlDocument`] to the structured
/// [`RdfXmlStructure`] striping. Fails closed on any non-flat shape (a nested
/// node element, inline `parseType` content, mixed property content) so the
/// byte-exact tier is never claimed for a source the flat writer cannot
/// regenerate.
fn project_rdfxml_structure(doc: &XmlDocument) -> Result<RdfXmlStructure, RdfXmlStructureError> {
    let root = &doc.root;
    let mut node_blocks = Vec::new();
    for child in &root.children {
        match child {
            XmlNode::Element(node_el) => {
                node_blocks.push(project_node_block(node_el)?);
            }
            // Inter-element white-space `Text` between node blocks is generic
            // residue (the content-white-space complement), not structure — skip
            // it here. A NON-white-space text run directly under <rdf:RDF> is not
            // a flat RDF/XML shape.
            XmlNode::Text(t) if t.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')) => {}
            XmlNode::Text(_) => {
                return Err(RdfXmlStructureError::NonFlat {
                    element: root.name.qualified(),
                });
            }
            // Comments / PIs / CDATA directly under the root are not the flat
            // node-block shape (cito carries none).
            _ => {
                return Err(RdfXmlStructureError::NonFlat {
                    element: root.name.qualified(),
                });
            }
        }
    }
    Ok(RdfXmlStructure {
        root_name: root.name.clone(),
        node_blocks,
    })
}

/// Project one node element (`<rdf:Description …>`) to an [`RdfNodeBlock`]. Each
/// element child must be a leaf property element (RDF/XML §3.4); a nested node
/// element or inline `parseType` content is non-flat and rejected.
fn project_node_block(node_el: &XmlElement) -> Result<RdfNodeBlock, RdfXmlStructureError> {
    let mut properties = Vec::new();
    for child in &node_el.children {
        match child {
            XmlNode::Element(prop_el) => {
                properties.push(project_property_element(prop_el)?);
            }
            // Inter-element white-space between property elements — generic
            // residue, skip.
            XmlNode::Text(t) if t.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')) => {}
            // Non-white-space char data directly inside a node element is not the
            // flat striped shape.
            XmlNode::Text(_) => {
                return Err(RdfXmlStructureError::NonFlat {
                    element: node_el.name.qualified(),
                });
            }
            _ => {
                return Err(RdfXmlStructureError::NonFlat {
                    element: node_el.name.qualified(),
                });
            }
        }
    }
    Ok(RdfNodeBlock {
        name: node_el.name.clone(),
        properties,
    })
}

/// Project one property element to an [`RdfPropertyElement`]. Empty ⇒
/// [`PropertyContent::Empty`] (a reference element); a single leaf-text child ⇒
/// [`PropertyContent::Text`] (a literal). Anything else (element children — a
/// nested node element; multiple text runs — mixed content) is rejected.
fn project_property_element(
    prop_el: &XmlElement,
) -> Result<RdfPropertyElement, RdfXmlStructureError> {
    // Partition the children: collect non-white-space leaf text; reject any
    // element child (a nested node element / inline content the flat writer does
    // not reproduce).
    let mut texts: Vec<&String> = Vec::new();
    for child in &prop_el.children {
        match child {
            XmlNode::Text(t) | XmlNode::CData(t) => texts.push(t),
            XmlNode::Element(_) => {
                return Err(RdfXmlStructureError::NonFlat {
                    element: prop_el.name.qualified(),
                });
            }
            // Comments / PIs inside a property element are not the leaf-literal
            // shape (cito carries none).
            _ => {
                return Err(RdfXmlStructureError::MixedContent {
                    element: prop_el.name.qualified(),
                });
            }
        }
    }
    let content = match texts.as_slice() {
        [] => PropertyContent::Empty,
        [single] => PropertyContent::Text((*single).clone()),
        // More than one text run on a leaf — the grammar coalesces adjacent text,
        // so this would be mixed content the structure does not model.
        _ => {
            return Err(RdfXmlStructureError::MixedContent {
                element: prop_el.name.qualified(),
            });
        }
    };
    Ok(RdfPropertyElement {
        name: prop_el.name.clone(),
        content,
    })
}

// =============================================================================
// Structural writer — RdfXmlStructure → XmlDocument (the element backbone).
// =============================================================================

/// Build an [`XmlElement`] with no attributes and no namespace declarations —
/// those come from the generic [`AttributeOverrides`] /
/// [`DocumentResidue`] residue. The structural writer emits only the element
/// NAME and CHILDREN (the backbone); the byte kernel restores everything else.
fn bare_element(name: &XmlName, children: Vec<XmlNode>) -> XmlElement {
    XmlElement {
        name: name.clone(),
        namespace: None,
        namespaces: Vec::new(),
        attributes: Vec::new(),
        children,
    }
}

/// Regenerate the RDF/XML element backbone from an [`RdfXmlStructure`] — the
/// structural fold, the OWL analogue of
/// [`write_wordnet_document`](crate::social::software::markup::xml::lmf::writer::write_wordnet_document).
///
/// Emits `<{root}>` (no namespaces — the [`DocumentResidue`] restores them),
/// then one `<{node block name}>` element per node block, each with one
/// `<{property name}>` element per property (empty for a reference, a single
/// leaf-text child for a literal). The result is element-backbone-equal to the
/// source DOM: same elements, same pre-order, same names, same leaf text. The
/// `version="1.0" encoding="UTF-8"` prolog matches the bundled vocab declaration
/// (the byte kernel reproduces the exact declaration bytes).
pub fn write_owl_document(structure: &RdfXmlStructure) -> XmlDocument {
    let mut root_children: Vec<XmlNode> = Vec::with_capacity(structure.node_blocks.len());
    for block in &structure.node_blocks {
        let mut block_children: Vec<XmlNode> = Vec::with_capacity(block.properties.len());
        for prop in &block.properties {
            let prop_children = match &prop.content {
                PropertyContent::Empty => Vec::new(),
                PropertyContent::Text(text) => vec![XmlNode::Text(text.clone())],
            };
            block_children.push(XmlNode::Element(bare_element(&prop.name, prop_children)));
        }
        root_children.push(XmlNode::Element(bare_element(&block.name, block_children)));
    }
    let root = bare_element(&structure.root_name, root_children);
    XmlDocument {
        version: "1.0".to_string(),
        encoding: Some("UTF-8".to_string()),
        doctype: None,
        root,
    }
}

// =============================================================================
// OwlSyntaxComplement — the concrete-syntax residue bundle.
// =============================================================================

/// The concrete-syntax COMPLEMENT for one OWL/RDF source: everything the typed
/// [`OwlOntology`] graph + [`write_owl_document`]'s regenerated backbone do NOT
/// carry, captured so [`reconstruct_owl_rdfxml_source`] reproduces the exact
/// source bytes.
///
/// Four residue layers, each grounded in the same W3C residue classes the
/// WordNet leaf uses — see the module-level note. The `structure` field is the
/// only OWL-specific piece (the RDF/XML striping); the other three are generic
/// XML-family residue reused verbatim from the parser.
///
/// rkyv-serializable under `prx` (CFG-GATED): the graph-faithful OWL `.prx`
/// envelope carries this complement beside the typed [`OwlOntology`] graph so
/// the byte-exact `put` runs from the archive alone.
///
/// [`OwlOntology`]: super::ontology::OwlOntology
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct OwlSyntaxComplement {
    /// The RDF/XML serialization STRIPING — the document-ordered node-block /
    /// property-element backbone + per-property leaf text the typed graph does
    /// not carry. The OWL-specific residue; everything else is generic.
    pub structure: RdfXmlStructure,
    /// Document/root-level residue: the `<!DOCTYPE>` (§2.8) and the root
    /// element's namespace declarations + their declaration order (Bray,
    /// Hollander, Layman & Tobin 2009 §3 — cito's four `xmlns:*` on `<rdf:RDF>`).
    pub document_residue: DocumentResidue,
    /// The regenerated-tree residue: the §2.4 inter-element white-space (the
    /// indentation) AND the EXACT source attribute sequences the structural
    /// writer left empty — the `rdf:about`/`rdf:nodeID`/`rdf:resource`/
    /// `rdf:datatype`/`xml:lang` reference forms in source order, keyed by
    /// pre-order element index. Generic residue.
    pub regenerated: RegeneratedComplement,
    /// The per-element concrete-syntax decisions the byte-exact serializer
    /// honours — the multi-line `<rdf:RDF>` intra-tag layout (§3.1), the §4.6
    /// entity-reference form, the empty-element form, the prolog/epilog `Misc*`
    /// (§2.8), the §2.11 EOL form. Captured by `parse_document_capturing`.
    pub syntax_decisions: SyntaxDecisions,
}

/// Failure modes of the RDF/XML graph-faithful reconstruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwlReconstructError {
    /// The source did not parse as well-formed XML (§2.1).
    Parse(XmlParseError),
    /// The source is not the flat RDF/XML shape the structural writer models
    /// (a nested node element, inline `parseType` content, mixed content).
    Structure(RdfXmlStructureError),
    /// [`write_owl_document`]'s regenerated tree was not element-backbone-equal
    /// to the captured source DOM (a dropped/added/reordered/renamed
    /// element/leaf-text). Carries the exact divergence. This is the
    /// fail-closed guard: a structural defect surfaces here, never as a
    /// fabricated complement.
    Complement(RegeneratedComplementError),
}

impl core::fmt::Display for OwlReconstructError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "RDF/XML source parse: {e}"),
            Self::Structure(e) => write!(f, "RDF/XML structure projection: {e}"),
            Self::Complement(e) => write!(f, "RDF/XML regenerated-tree complement: {e}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OwlReconstructError {}

impl From<XmlParseError> for OwlReconstructError {
    fn from(e: XmlParseError) -> Self {
        Self::Parse(e)
    }
}

impl From<RdfXmlStructureError> for OwlReconstructError {
    fn from(e: RdfXmlStructureError) -> Self {
        match e {
            RdfXmlStructureError::Parse(p) => Self::Parse(p),
            other => Self::Structure(other),
        }
    }
}

impl From<RegeneratedComplementError> for OwlReconstructError {
    fn from(e: RegeneratedComplementError) -> Self {
        Self::Complement(e)
    }
}

/// Capture the typed [`OwlOntology`] graph AND the concrete-syntax
/// [`OwlSyntaxComplement`] from an OWL/RDF source, such that
/// [`reconstruct_owl_rdfxml_source`] reproduces the source bytes byte-for-byte.
///
/// The OWL analogue of
/// [`capture_wn_complement`](crate::social::software::markup::xml::lmf::writer::capture_wn_complement),
/// with two byte-exact-grounded halves:
///
/// 1. **The exact DOM + decisions** — `parse_document_capturing` yields the full
///    Information Set DOM (with the root `xmlns:*`, the inter-element
///    white-space, the multi-line `<rdf:RDF>` layout) and the
///    [`SyntaxDecisions`] the byte-exact serializer honours.
/// 2. **The structural projection + regenerated-tree complement** —
///    [`project_rdfxml_structure`] derives the structured striping;
///    [`write_owl_document`] regenerates the backbone from it; and
///    [`diff_content_whitespace`] extracts the inter-element white-space the
///    regenerated tree lacks PLUS the exact source attribute sequences the
///    structural writer left empty (the `rdf:about`/`rdf:resource`/`rdf:nodeID`/
///    `rdf:datatype`/`xml:lang` forms). The `<!DOCTYPE>` + root namespaces are
///    read straight off the exact DOM.
///
/// Returns the typed [`OwlOntology`] (the reasoning graph) and the complement.
/// Fails closed if the source is not well-formed XML, not the flat RDF/XML
/// shape, or if the regenerated tree is not element-backbone-equal to the DOM.
pub fn capture_owl_complement(
    source: &str,
) -> Result<(super::ontology::OwlOntology, OwlSyntaxComplement), OwlReconstructError> {
    // (1) The exact DOM + per-element/prolog decisions.
    let (exact_dom, syntax_decisions) = parse_document_capturing(source.as_bytes())?;

    // The typed reasoning graph — the same `read_owl` the rest of the OWL stack
    // projects from. (Carried in the .prx envelope as the navigable ontology.)
    let ont = super::reader::read_owl(source).map_err(|e| {
        OwlReconstructError::Parse(XmlParseError::Syntax {
            position: 0,
            expected: "well-formed OWL/RDF document".into(),
            found: format!("{e}"),
        })
    })?;

    // (2a) The structured RDF/XML striping projection (the OWL-specific residue).
    let structure = project_rdfxml_structure(&exact_dom)?;

    // (2b) The regenerated backbone + the residue it lacks (inter-element
    // white-space AND exact source attribute sequences).
    let regenerated_tree = write_owl_document(&structure);
    let regenerated = diff_content_whitespace(&exact_dom, &regenerated_tree)?;

    // The document/root-level residue read straight off the exact DOM.
    let document_residue = DocumentResidue {
        doctype: exact_dom.doctype.clone(),
        root_namespaces: exact_dom.root.namespaces.clone(),
    };

    Ok((
        ont,
        OwlSyntaxComplement {
            structure,
            document_residue,
            regenerated,
            syntax_decisions,
        },
    ))
}

/// Reconstruct the exact OWL/RDF source bytes from the captured
/// [`OwlSyntaxComplement`] — the graph-faithful `put`.
///
/// The OWL analogue of
/// [`reconstruct_wn_lmf_source`](crate::social::software::markup::xml::lmf::writer::reconstruct_wn_lmf_source):
///
/// 1. [`write_owl_document`] regenerates the structural element backbone from
///    the structure (no white-space, no attributes, no DOCTYPE, no namespaces);
/// 2. [`reapply_regenerated_complement`] merges the residue back in — sets the
///    `<!DOCTYPE>` and root namespaces, overwrites each element's start-tag
///    attribute sequence with the exact source one, splices the inter-element
///    white-space — reproducing the exact captured DOM;
/// 3. [`serialize_document_exact`] emits that DOM PLUS the [`SyntaxDecisions`]
///    byte-for-byte.
///
/// When `complement` is what [`capture_owl_complement`] returned for a source
/// `s`, the result equals `s.as_bytes()` exactly. Fails closed on a structural
/// divergence (never fabricates bytes). The typed graph is NOT consulted here —
/// the byte-exact reconstruction rides the `structure` striping; the graph is
/// the navigable reasoning view the envelope also carries.
pub fn reconstruct_owl_rdfxml_source(
    complement: &OwlSyntaxComplement,
) -> Result<Vec<u8>, OwlReconstructError> {
    let mut tree = write_owl_document(&complement.structure);
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

    /// A minimal flat RDF/XML fragment in the cito shape: an `<rdf:RDF>` root
    /// with the four namespace declarations multi-line indented, two
    /// `<rdf:Description>` blocks (one `rdf:about` named, one `rdf:nodeID`
    /// blank), reference property elements (`rdf:resource`/`rdf:nodeID`), a
    /// `xml:lang` literal, and an `rdf:datatype` literal — every residue species
    /// cito exercises, two-space indented, trailing newline. The cheap proof
    /// that `reconstruct(capture(s)) == s` BYTE-FOR-BYTE.
    const FLAT_RDFXML: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<rdf:RDF\n\
   xmlns:dc=\"http://purl.org/dc/elements/1.1/\"\n\
   xmlns:owl=\"http://www.w3.org/2002/07/owl#\"\n\
   xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n\
   xmlns:rdfs=\"http://www.w3.org/2000/01/rdf-schema#\"\n\
>\n\
  <rdf:Description rdf:about=\"http://purl.org/spar/cito/givesSupportTo\">\n\
    <owl:inverseOf rdf:resource=\"http://purl.org/spar/cito/obtainsSupportFrom\"/>\n\
    <rdfs:label xml:lang=\"en\">gives support to</rdfs:label>\n\
    <rdfs:subClassOf rdf:nodeID=\"N0af6042b85424b339de02002660681c0\"/>\n\
    <rdf:type rdf:resource=\"http://www.w3.org/2002/07/owl#ObjectProperty\"/>\n\
  </rdf:Description>\n\
  <rdf:Description rdf:nodeID=\"N0af6042b85424b339de02002660681c0\">\n\
    <owl:cardinality rdf:datatype=\"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\">1</owl:cardinality>\n\
  </rdf:Description>\n\
</rdf:RDF>\n";

    /// The cheap byte-exact gate: capture the typed graph + complement from a
    /// flat cito-shaped fragment, reconstruct, assert the bytes match exactly.
    /// Exercises the structured striping, the four root namespaces + their
    /// multi-line layout, the inter-element white-space, the attribute overrides
    /// (about/nodeID/resource/datatype/lang), and the trailing newline.
    #[test]
    fn reconstruct_flat_rdfxml_byte_exact() {
        let (_ont, complement) = capture_owl_complement(FLAT_RDFXML).expect("capture");
        // The residue is genuinely present (a vacuous round-trip would lie).
        assert!(
            !complement.document_residue.root_namespaces.is_empty(),
            "root xmlns:* captured into the complement"
        );
        assert!(
            !complement.regenerated.content_whitespace.is_empty(),
            "inter-element white-space captured into the complement"
        );
        assert!(
            !complement.regenerated.attribute_overrides.is_empty(),
            "exact source attribute sequences (about/nodeID/resource/datatype/lang) captured"
        );
        // The structure carries the striping: two node blocks in document order
        // (the named one then the blank one — NOT merged).
        assert_eq!(
            complement.structure.node_blocks.len(),
            2,
            "two node blocks (named + blank), document order, not merged"
        );
        let out = reconstruct_owl_rdfxml_source(&complement).expect("reconstruct");
        assert_eq!(
            core::str::from_utf8(&out).unwrap(),
            FLAT_RDFXML,
            "reconstruct_owl_rdfxml_source(capture_owl_complement(s)) must equal s byte-for-byte"
        );
    }

    /// Fail-closed: a NESTED node element inside a property element (a striped
    /// non-flat RDF/XML serialization) is honestly rejected by the structure
    /// projection rather than silently flattened — the byte-exact tier is earned
    /// per-source, a non-flat source rides the floor.
    #[test]
    fn capture_rejects_nonflat_serialization() {
        let nested = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\" \
xmlns:owl=\"http://www.w3.org/2002/07/owl#\">\
<rdf:Description rdf:about=\"http://example.org/s\">\
<owl:equivalentClass><owl:Class rdf:about=\"http://example.org/o\"/></owl:equivalentClass>\
</rdf:Description></rdf:RDF>";
        let err = capture_owl_complement(nested)
            .expect_err("a nested node element must fail closed, not fake a flat structure");
        assert!(
            matches!(err, OwlReconstructError::Structure(_)),
            "expected a Structure (non-flat) error, got {err:?}"
        );
    }

    /// Lowercase-hex SHA-256 (NIST FIPS 180-4 §6.2) of `bytes` — the corpus
    /// gate's headline hash, computed directly via the crate's `sha2`
    /// dependency.
    fn sha256_hex(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(bytes);
        let mut s = String::with_capacity(64);
        for byte in digest {
            s.push_str(&format!("{byte:02x}"));
        }
        s
    }

    /// THE HARD GATE: the REAL on-disk `crates/domains/data/ontologies/
    /// cito-2.8.1.owl` (83 836 bytes) reconstructs BYTE-FOR-BYTE from the typed
    /// [`OwlOntology`] graph + the captured concrete-syntax complement — with NO
    /// stored raw blob, NO stored DOM. `capture_owl_complement(src)` then
    /// `reconstruct_owl_rdfxml_source(&complement)` must equal the source bytes
    /// exactly, and SHA-256 to the pinned `[hashes]` cito content address. This
    /// is the only non-vacuous proof that CiTO is now graph-faithful — praxis's
    /// FIRST byte-exact OWL vocabulary.
    ///
    /// Bundled in-tree (the SPAR canonical publication), so unlike the 89 MB
    /// WordNet corpus there is no graceful skip — a missing file is a defect.
    #[test]
    fn cito_reconstruct_byte_exact_over_real_source() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/cito-2.8.1.owl"
        );
        let source = std::fs::read(path).expect("bundled cito-2.8.1.owl must exist");
        let src = core::str::from_utf8(&source).expect("CiTO is UTF-8");

        let (ont, complement) = capture_owl_complement(src).expect("capture cito complement");

        // The capture is genuinely non-vacuous: the typed graph carries CiTO's
        // classes/properties, and the complement carries the real residue.
        assert!(
            !ont.properties.is_empty(),
            "typed OWL graph carries CiTO object properties (capture is non-vacuous)"
        );
        assert_eq!(
            complement.document_residue.root_namespaces.len(),
            4,
            "the four <rdf:RDF> xmlns:* declarations captured (dc/owl/rdf/rdfs)"
        );
        assert_eq!(
            complement.structure.node_blocks.len(),
            131,
            "131 <rdf:Description> node blocks projected in document order"
        );
        assert!(
            !complement.regenerated.content_whitespace.is_empty(),
            "inter-element indentation captured"
        );
        assert!(
            !complement.regenerated.attribute_overrides.is_empty(),
            "exact source attribute sequences captured"
        );

        let out = reconstruct_owl_rdfxml_source(&complement).expect("reconstruct cito");

        // Byte-for-byte. Report the EXACT first byte-diff for an honest failure.
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
        let hash = sha256_hex(&out);
        assert_eq!(
            hash,
            sha256_hex(&source),
            "reconstructed CiTO must hash-equal the source"
        );
        assert_eq!(
            hash, "48b31bbb36f2a81efb9a65bd8334be6b41b7c00dd54dc4f8a061e69607d66a84",
            "reconstructed CiTO must hash to the pinned praxis.lock [hashes] cito@2.8.1 source sha256"
        );
    }

    /// CORRUPTION META-TEST — the gate has teeth. Capturing the real CiTO,
    /// then corrupting the captured complement (a triple's leaf text, a
    /// blank-node nodeID attribute, a namespace-prefix declaration) MUST make
    /// the reconstruction DIVERGE from the source — proving the byte-exact
    /// assertion is non-vacuous (it would catch a real drift), not a tautology.
    ///
    /// Mirrors the USC subdivision corruption meta-test: corrupt each residue
    /// species the gate depends on and assert the reconstruction no longer
    /// equals the source.
    #[test]
    fn cito_corruption_diverges_red() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/ontologies/cito-2.8.1.owl"
        );
        let source = std::fs::read(path).expect("bundled cito-2.8.1.owl must exist");
        let src = core::str::from_utf8(&source).expect("CiTO is UTF-8");
        let (_ont, base) = capture_owl_complement(src).expect("capture cito");

        // Sanity: the unmodified complement DOES reconstruct exactly (so the
        // divergences below are caused by the corruption, not a pre-existing gap).
        assert_eq!(
            reconstruct_owl_rdfxml_source(&base).expect("reconstruct base"),
            source,
            "baseline reconstruction must be byte-exact before corruption"
        );

        // (1) Corrupt a TRIPLE: flip a property element's leaf text in the
        // structure striping. The reconstruction must diverge.
        {
            let mut c = base.clone();
            let mut flipped = false;
            'outer: for block in &mut c.structure.node_blocks {
                for prop in &mut block.properties {
                    if let PropertyContent::Text(t) = &mut prop.content
                        && !t.is_empty()
                    {
                        t.push_str("__CORRUPT__");
                        flipped = true;
                        break 'outer;
                    }
                }
            }
            assert!(flipped, "found a leaf-text property to corrupt");
            let out = reconstruct_owl_rdfxml_source(&c).expect("reconstruct corrupt-triple");
            assert_ne!(
                out, source,
                "corrupting a triple's leaf text must make reconstruction DIVERGE (gate has teeth)"
            );
        }

        // (2) Corrupt a BLANK NODE: mangle a captured `rdf:nodeID` attribute
        // value in the attribute overrides. The reconstruction must diverge.
        {
            let mut c = base.clone();
            let mut flipped = false;
            for over in c.regenerated.attribute_overrides.values_mut() {
                for attr in &mut over.attributes {
                    if attr.name.local == "nodeID" {
                        attr.value.push_str("DEADBEEF");
                        flipped = true;
                        break;
                    }
                }
                if flipped {
                    break;
                }
            }
            assert!(flipped, "found an rdf:nodeID attribute to corrupt");
            let out = reconstruct_owl_rdfxml_source(&c).expect("reconstruct corrupt-nodeID");
            assert_ne!(
                out, source,
                "corrupting a blank-node rdf:nodeID must make reconstruction DIVERGE"
            );
        }

        // (3) Corrupt a PREFIX: mangle the prefix of a captured root namespace
        // declaration. cito's four `<rdf:RDF>` `xmlns:*` declarations are
        // captured as the root element's (index-0) attribute override (the
        // structural writer emits a bare root, so the whole namespace sequence is
        // an override), which is the path that drives the start-tag bytes. The
        // reconstruction must diverge.
        {
            let mut c = base.clone();
            let mut flipped = false;
            for over in c.regenerated.attribute_overrides.values_mut() {
                if let Some(ns) = over.namespaces.first_mut() {
                    ns.prefix = Some("ZZ".to_string());
                    flipped = true;
                    break;
                }
            }
            assert!(flipped, "found a root xmlns:* prefix to corrupt");
            let out = reconstruct_owl_rdfxml_source(&c).expect("reconstruct corrupt-prefix");
            assert_ne!(
                out, source,
                "corrupting a namespace-prefix declaration must make reconstruction DIVERGE"
            );
        }
    }
}
