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
//! it is empty, carries leaf text (the literal lexical form), or carries nested
//! node elements (the striped inline-resource form — `parseType="Collection"`
//! member lists, an inline `owl:unionOf`/`owl:Restriction` block). The structure
//! is RECURSIVE: [`PropertyContent::Nested`] holds further [`RdfNodeBlock`]s, so
//! the exact source nesting (and, via the attribute residue, the exact blank-node
//! label) is captured as typed structure to arbitrary depth — never opaque
//! exact-bytes for a whole element, never a stored DOM. Attribute *values* are
//! NOT in this structure — they are the generic [`AttributeOverrides`] residue
//! (the source `rdf:about`/`rdf:nodeID`/`rdf:resource`/`rdf:datatype`/`xml:lang`
//! sequences), because XML attribute order/coverage is concrete-syntax (Cowan &
//! Tobin 2004 §2.3), not Infoset.
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
//!   §3, Definition 3.2 — the well-behaved lens whose graph-faithful `put` this realises.
//!
//! [`WordNet`]: crate::social::software::markup::xml::lmf::ontology::WordNet
//! [`OwlOntology`]: super::ontology::OwlOntology
//! [`AttributeOverrides`]: crate::social::software::markup::xml::parser::source_syntax::AttributeOverrides
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
/// - [`Nested`](Self::Nested) — an INLINE-RESOURCE property element whose content
///   is one or more nested **node elements** (RDF/XML §2.14 `striped` form): an
///   `<owl:Class>`/`<rdf:Description>`/anonymous-node child block, an
///   `rdf:parseType="Collection"` member list, an `owl:unionOf`/`owl:intersectionOf`
///   inline list, or an inline `owl:Restriction` blank-node block. Each member is
///   itself a recursive [`RdfNodeBlock`], so the structure captures the EXACT
///   source nesting (and, via the generic
///   [`AttributeOverrides`](crate::social::software::markup::xml::parser::source_syntax::AttributeOverrides)
///   residue keyed by
///   pre-order index, the exact blank-node label / `parseType` attribute) as
///   TYPED structure — never opaque exact-bytes, never a stored DOM. The cito
///   (and biro/c4o/doco) SPAR vocabs serialise every node FLAT (each blank node a
///   separate top-level block), so they never use this variant; the striped
///   RDF/XML form (prov_o's `parseType="Collection"`, an inline `owl:Restriction`)
///   does.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
// `PropertyContent` and `RdfNodeBlock` are mutually recursive
// (`PropertyContent::Nested(Vec<RdfNodeBlock>)`, `RdfNodeBlock.properties:
// Vec<RdfPropertyElement>`, `RdfPropertyElement.content: PropertyContent`). rkyv's
// derive would otherwise recurse infinitely computing the
// `Archive`/`Serialize`/`Deserialize` where-bounds (rustc E0275 overflow), so the
// recursive `Nested` field carries `#[rkyv(omit_bounds)]` and this container
// supplies the resolved bounds explicitly — the canonical rkyv 0.8 recursive-type
// pattern, identical to `XmlElement`/`XmlNode`. CFG-GATED on `prx`: the default +
// wasm32 builds (no rkyv) see a plain enum.
#[cfg_attr(
    feature = "prx",
    rkyv(serialize_bounds(
        __S: rkyv::ser::Writer + rkyv::ser::Allocator,
        __S::Error: rkyv::rancor::Source,
    )),
    rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source)),
    rkyv(bytecheck(bounds(
        __C: rkyv::validation::ArchiveContext,
        __C::Error: rkyv::rancor::Source,
    )))
)]
pub enum PropertyContent {
    /// An empty property element — `rdf:resource`/`rdf:nodeID` reference form.
    Empty,
    /// A literal property element carrying one leaf-text run (the lexical form).
    Text(String),
    /// An inline-resource property element carrying one or more nested node
    /// elements in document order (a striped sub-node, a `parseType="Collection"`
    /// member list, an inline `owl:unionOf`/`owl:Restriction` block). Recursive.
    Nested(#[cfg_attr(feature = "prx", rkyv(omit_bounds))] Vec<RdfNodeBlock>),
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
///
/// RECURSIVE: a property element's content may itself be nested node blocks
/// ([`PropertyContent::Nested`]) — the striped RDF/XML form (RDF/XML §2.14). A
/// flat SPAR vocab is a one-level tree (every node a top-level block); a striped
/// vocab (prov_o) nests an inline `owl:Restriction` / `parseType="Collection"`
/// member list under a property element, and this type captures that nesting to
/// arbitrary depth.
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
///
/// [`OwlOntology`]: super::ontology::OwlOntology
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
/// RECURSIVE node-block writer does not model. The projection handles the flat
/// SPAR form (every node a top-level `<rdf:Description>`) AND the striped RDF/XML
/// form (RDF/XML §2.14): nested node elements under a property element
/// (`parseType="Collection"` member lists, inline `owl:Restriction`/`owl:unionOf`
/// blocks). What it still rejects, fail-closed (the source rides the floor):
/// non-element non-white-space content the byte kernel's element-backbone
/// reconstruction cannot reproduce — a comment/PI interspersed with node blocks
/// (the structural writer emits only elements), or MIXED content (text *and*
/// element children, or multiple text runs) in one property element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RdfXmlStructureError {
    /// The source bytes did not parse as well-formed XML (§2.1).
    Parse(XmlParseError),
    /// A non-element, non-white-space node appeared where the structural writer
    /// reproduces only elements: a comment / PI / non-white-space text run
    /// directly among node blocks or inside a node element. The structural fold
    /// emits no comments, so such a source rides the raw-bytes floor. Carries the
    /// containing element's qualified name.
    NonFlat { element: String },
    /// A property element MIXED text and element content (or carried more than
    /// one text run, or a comment/PI/CDATA child) — not the single-leaf-literal,
    /// empty-reference, or pure-nested-resource shape the structure models.
    MixedContent { element: String },
}

impl core::fmt::Display for RdfXmlStructureError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "RDF/XML source parse: {e}"),
            Self::NonFlat { element } => write!(
                f,
                "RDF/XML serialization at <{element}> carries a comment/PI/non-white-space \
                 text run among node blocks — the structural writer reproduces only the \
                 element backbone (this source rides the raw-bytes floor)"
            ),
            Self::MixedContent { element } => write!(
                f,
                "mixed content at property element <{element}> — the structure models a \
                 single leaf-text literal, an empty reference element, or pure nested \
                 node elements (not text-and-element mixed)"
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

/// `true` iff `t` is pure XML white-space (§2.3 `S`) — inter-element
/// indentation, generic residue (the content-white-space complement), not
/// structure.
fn is_whitespace(t: &str) -> bool {
    t.chars().all(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))
}

/// Project a parsed RDF/XML [`XmlDocument`] to the structured
/// [`RdfXmlStructure`] striping. Handles the flat AND the striped (nested) form;
/// fails closed only on a comment/PI/non-white-space text run interspersed with
/// node blocks (the structural writer reproduces only elements), so the
/// byte-exact tier is never claimed for a source the writer cannot regenerate.
fn project_rdfxml_structure(doc: &XmlDocument) -> Result<RdfXmlStructure, RdfXmlStructureError> {
    let root = &doc.root;
    let node_blocks = project_node_block_list(&root.children, &root.name)?;
    Ok(RdfXmlStructure {
        root_name: root.name.clone(),
        node_blocks,
    })
}

/// Project a sequence of children into the node blocks among them. Used for the
/// root's children AND for the nested member list of an inline-resource property
/// element (RDF/XML §2.14 striped form). Inter-element white-space is skipped
/// (generic residue); a comment/PI/non-white-space text run is rejected
/// fail-closed (the structural writer emits only elements). `container` names the
/// element whose children these are, for the error.
fn project_node_block_list(
    children: &[XmlNode],
    container: &XmlName,
) -> Result<Vec<RdfNodeBlock>, RdfXmlStructureError> {
    let mut node_blocks = Vec::new();
    for child in children {
        match child {
            XmlNode::Element(node_el) => node_blocks.push(project_node_block(node_el)?),
            XmlNode::Text(t) if is_whitespace(t) => {}
            // A §2.5 [15] Comment or §2.6 [16] PI interspersed among node blocks
            // (the section-divider comments prov_o writes) is concrete-syntax
            // residue the generic byte kernel re-splices (the `ChildSlot::Insert*`
            // template), NOT structure — skip it here so the backbone projection
            // stays pure elements, exactly as inter-element white-space is skipped.
            XmlNode::Comment(_) | XmlNode::ProcessingInstruction { .. } => {}
            _ => {
                return Err(RdfXmlStructureError::NonFlat {
                    element: container.qualified(),
                });
            }
        }
    }
    Ok(node_blocks)
}

/// Project one node element (`<rdf:Description …>` or a typed node `<owl:Class>`)
/// to an [`RdfNodeBlock`]. Each element child is a property element (RDF/XML
/// §3.4), projected recursively; inter-element white-space is skipped; a
/// comment/PI/non-white-space text run inside a node element is rejected.
fn project_node_block(node_el: &XmlElement) -> Result<RdfNodeBlock, RdfXmlStructureError> {
    let mut properties = Vec::new();
    for child in &node_el.children {
        match child {
            XmlNode::Element(prop_el) => {
                properties.push(project_property_element(prop_el)?);
            }
            XmlNode::Text(t) if is_whitespace(t) => {}
            // Interspersed comment/PI inside a node element — generic residue the
            // byte kernel re-splices, skipped here like inter-element white-space.
            XmlNode::Comment(_) | XmlNode::ProcessingInstruction { .. } => {}
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

/// Project one property element to an [`RdfPropertyElement`], RECURSIVELY:
/// - no children ⇒ [`PropertyContent::Empty`] (a reference element);
/// - exactly one leaf-text child (+ optional surrounding white-space) ⇒
///   [`PropertyContent::Text`] (a literal);
/// - one or more nested NODE elements (+ optional white-space) ⇒
///   [`PropertyContent::Nested`] (the striped inline-resource form — each member
///   projected recursively as an [`RdfNodeBlock`]).
///
/// MIXED text *and* element content, multiple text runs, or a comment/PI/CDATA
/// child is rejected ([`RdfXmlStructureError::MixedContent`]) — the source rides
/// the floor.
fn project_property_element(
    prop_el: &XmlElement,
) -> Result<RdfPropertyElement, RdfXmlStructureError> {
    // First, does this property have ANY element children (the striped
    // inline-resource form)? If not, fall through to the leaf classification,
    // which preserves the flat writer's EXACT text/empty semantics byte-for-byte.
    let has_element_child = prop_el
        .children
        .iter()
        .any(|c| matches!(c, XmlNode::Element(_)));

    if has_element_child {
        // Striped form: collect the nested node elements; surrounding white-space
        // text is generic residue (the byte kernel's content-white-space
        // complement re-splices it), exactly as it does between top-level node
        // blocks. Any NON-white-space text run abutting the elements is MIXED
        // content the structure does not model.
        let mut nested: Vec<RdfNodeBlock> = Vec::new();
        for child in &prop_el.children {
            match child {
                XmlNode::Element(child_el) => nested.push(project_node_block(child_el)?),
                XmlNode::Text(t) if is_whitespace(t) => {}
                // An interspersed comment/PI in the nested member list (olia's
                // commented-out `<!--dcr:datcat …/-->` members) — generic residue
                // the byte kernel re-splices via the `ChildSlot::Insert*` template,
                // skipped here like inter-element white-space (NOT mixed content).
                XmlNode::Comment(_) | XmlNode::ProcessingInstruction { .. } => {}
                _ => {
                    return Err(RdfXmlStructureError::MixedContent {
                        element: prop_el.name.qualified(),
                    });
                }
            }
        }
        return Ok(RdfPropertyElement {
            name: prop_el.name.clone(),
            content: PropertyContent::Nested(nested),
        });
    }

    // Leaf form (no element children) — the original flat classification,
    // unchanged: collect every text/CData run; [] ⇒ Empty, [single] ⇒ Text,
    // anything else (a comment/PI child, multiple coalesced runs) ⇒ MixedContent.
    let mut texts: Vec<&String> = Vec::new();
    for child in &prop_el.children {
        match child {
            XmlNode::Text(t) | XmlNode::CData(t) => texts.push(t),
            // Comments / PIs inside a property element are not the leaf-literal
            // shape (the flat vocabs carry none).
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

/// Emit one node block as a bare [`XmlElement`] (RECURSIVE) — the node's QName
/// with one `<{property name}>` child element per property. A property's content
/// becomes its child nodes: [`Empty`](PropertyContent::Empty) ⇒ none (a
/// reference element); [`Text`](PropertyContent::Text) ⇒ one leaf-text node (a
/// literal); [`Nested`](PropertyContent::Nested) ⇒ the nested node blocks emitted
/// recursively (the striped inline-resource form). No attributes / white-space —
/// the generic residue restores them in lockstep pre-order.
fn write_node_block(block: &RdfNodeBlock) -> XmlElement {
    let mut block_children: Vec<XmlNode> = Vec::with_capacity(block.properties.len());
    for prop in &block.properties {
        let prop_children = match &prop.content {
            PropertyContent::Empty => Vec::new(),
            PropertyContent::Text(text) => vec![XmlNode::Text(text.clone())],
            PropertyContent::Nested(members) => members
                .iter()
                .map(|m| XmlNode::Element(write_node_block(m)))
                .collect(),
        };
        block_children.push(XmlNode::Element(bare_element(&prop.name, prop_children)));
    }
    bare_element(&block.name, block_children)
}

/// Regenerate the RDF/XML element backbone from an [`RdfXmlStructure`] — the
/// structural fold, the OWL analogue of
/// [`write_wordnet_document`](crate::social::software::markup::xml::lmf::writer::write_wordnet_document).
///
/// Emits `<{root}>` (no namespaces — the [`DocumentResidue`] restores them),
/// then RECURSIVELY one `<{node block name}>` element per node block (via
/// `write_node_block`), each with one `<{property name}>` element per property
/// — empty for a reference, a single leaf-text child for a literal, or the nested
/// member blocks for the striped inline-resource form. The result is
/// element-backbone-equal to the source DOM: same elements, same pre-order, same
/// names, same leaf text, same nesting. The `version="1.0" encoding="UTF-8"`
/// prolog matches the bundled flat vocab declarations (cito/biro/c4o/doco — the
/// byte kernel reproduces the exact declaration bytes).
pub fn write_owl_document(structure: &RdfXmlStructure) -> XmlDocument {
    let root_children: Vec<XmlNode> = structure
        .node_blocks
        .iter()
        .map(|block| XmlNode::Element(write_node_block(block)))
        .collect();
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
    /// The source is not a shape the structural writer models — a comment/PI/
    /// non-white-space text run interspersed with node blocks, or mixed
    /// text-and-element property content (the nested striped form IS modelled).
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
///    `project_rdfxml_structure` derives the structured striping;
///    [`write_owl_document`] regenerates the backbone from it; and
///    [`diff_content_whitespace`] extracts the inter-element white-space the
///    regenerated tree lacks PLUS the exact source attribute sequences the
///    structural writer left empty (the `rdf:about`/`rdf:resource`/`rdf:nodeID`/
///    `rdf:datatype`/`xml:lang` forms). The `<!DOCTYPE>` + root namespaces are
///    read straight off the exact DOM.
///
/// Returns the typed [`OwlOntology`] (the reasoning graph) and the complement.
/// Fails closed if the source is not well-formed XML, carries a shape the
/// structural writer does not model (an interspersed comment/PI, mixed property
/// content), or if the regenerated tree is not element-backbone-equal to the DOM.
///
/// [`OwlOntology`]: super::ontology::OwlOntology
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
        // The XML declaration's version + encoding form (prov_o declares
        // `<?xml version="1.0"?>` with NO encoding; cito declares
        // `encoding="UTF-8"`) — captured so the byte-exact writer reproduces the
        // declaration instead of `write_owl_document`'s hardcoded
        // `encoding="UTF-8"`.
        xml_version: Some(exact_dom.version.clone()),
        xml_encoding: Some(exact_dom.encoding.clone()),
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
    use pr4xis_runtime::address::ContentAddress;

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
    #[pr4xis::praxis_value(Deterministic)]
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

    /// A STRIPED RDF/XML fragment exercising every nested shape the recursive
    /// node-block writer regenerates: (a) an inline-resource nested
    /// `<owl:Class>` under `<owl:equivalentClass>`; (b) an
    /// `<owl:unionOf rdf:parseType="Collection">` member list of two nested
    /// `<rdf:Description>` blocks; (c) an inline `<owl:Restriction>` blank-node
    /// block with its own `<owl:onProperty>`/`<owl:someValuesFrom>` leaf
    /// properties — two levels deep. Two-space indented, trailing newline. The
    /// cheap proof that `reconstruct(capture(s)) == s` BYTE-FOR-BYTE over the
    /// striped form (the cito/biro/c4o/doco flat form is the sibling
    /// `FLAT_RDFXML` proof).
    const STRIPED_RDFXML: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<rdf:RDF\n\
   xmlns:owl=\"http://www.w3.org/2002/07/owl#\"\n\
   xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n\
>\n\
  <owl:Class rdf:about=\"http://example.org/A\">\n\
    <owl:equivalentClass>\n\
      <owl:Class rdf:about=\"http://example.org/B\"/>\n\
    </owl:equivalentClass>\n\
    <owl:unionOf rdf:parseType=\"Collection\">\n\
      <rdf:Description rdf:about=\"http://example.org/C\"/>\n\
      <owl:Restriction>\n\
        <owl:onProperty rdf:resource=\"http://example.org/p\"/>\n\
        <owl:someValuesFrom rdf:resource=\"http://example.org/D\"/>\n\
      </owl:Restriction>\n\
    </owl:unionOf>\n\
  </owl:Class>\n\
</rdf:RDF>\n";

    /// The RECURSIVE byte-exact gate: capture the typed graph + complement from a
    /// striped (nested) cito-shaped fragment, reconstruct, assert byte-for-byte.
    /// Exercises [`PropertyContent::Nested`] at two depths, the
    /// `parseType="Collection"` member list, the inline `owl:Restriction`
    /// blank-node block, and the per-element pre-order attribute/white-space
    /// residue across the nesting. This is the proof the new recursion is real,
    /// not dead code.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn reconstruct_striped_rdfxml_byte_exact() {
        let (_ont, complement) = capture_owl_complement(STRIPED_RDFXML).expect("capture striped");
        // The structure carries ONE top-level node block (the named class); its
        // nesting lives inside the property contents, not as sibling blocks.
        assert_eq!(
            complement.structure.node_blocks.len(),
            1,
            "one top-level node block (the striped members are nested, not top-level)"
        );
        // The nesting is genuinely captured as TYPED structure (not flattened,
        // not opaque bytes): the class has an equivalentClass with one nested
        // member and a unionOf with two nested members (one of which is itself a
        // Restriction with two leaf properties).
        let top = &complement.structure.node_blocks[0];
        let union = top
            .properties
            .iter()
            .find(|p| p.name.local == "unionOf")
            .expect("unionOf property present");
        match &union.content {
            PropertyContent::Nested(members) => {
                assert_eq!(members.len(), 2, "the Collection has two members");
                // The second member is the inline Restriction, with two nested
                // leaf properties (onProperty + someValuesFrom).
                assert_eq!(members[1].name.local, "Restriction");
                assert_eq!(
                    members[1].properties.len(),
                    2,
                    "the inline Restriction carries its two leaf properties as recursive structure"
                );
            }
            other => panic!("unionOf must be Nested structure, got {other:?}"),
        }
        let out = reconstruct_owl_rdfxml_source(&complement).expect("reconstruct striped");
        assert_eq!(
            core::str::from_utf8(&out).unwrap(),
            STRIPED_RDFXML,
            "the recursive writer must reconstruct the striped form byte-for-byte"
        );
    }

    /// A COMMENT interspersed among node blocks (the section-divider comments
    /// prov_o/olia write) is now captured as STRUCTURED concrete-syntax residue
    /// (the `ChildSlot::InsertComment` template) and reconstructed BYTE-FOR-BYTE —
    /// the structural writer still emits only the element backbone, the byte
    /// kernel re-splices the comment. NOT a stored DOM: the comment rides the
    /// generic content-white-space residue, exactly like inter-element
    /// indentation. (This is the L3 residue that, together with the DOCTYPE +
    /// numeric/general reference forms, flips prov_o/olia off the floor.)
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn capture_reconstructs_interspersed_comment_byte_exact() {
        let with_comment = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\" \
xmlns:owl=\"http://www.w3.org/2002/07/owl#\">\
<!-- a section comment the structural writer does not emit -->\
<rdf:Description rdf:about=\"http://example.org/s\">\
<rdf:type rdf:resource=\"http://www.w3.org/2002/07/owl#Class\"/>\
</rdf:Description></rdf:RDF>";
        let (_ont, complement) =
            capture_owl_complement(with_comment).expect("interspersed comment is captured residue");
        let out = reconstruct_owl_rdfxml_source(&complement).expect("reconstruct with comment");
        assert_eq!(
            core::str::from_utf8(&out).unwrap(),
            with_comment,
            "an interspersed comment must reconstruct byte-for-byte (captured residue, not dropped)"
        );
    }

    /// THE HARD GATE: the REAL on-disk `crates/domains/data/ontologies/
    /// cito-2.8.1.owl` (83 836 bytes) reconstructs BYTE-FOR-BYTE from the typed
    /// [`OwlOntology`] graph + the captured concrete-syntax complement — with NO
    /// stored raw blob, NO stored DOM. `capture_owl_complement(src)` then
    /// `reconstruct_owl_rdfxml_source(&complement)` must equal the source bytes
    /// exactly, and hash to the pinned `[hashes]` cito content address. This
    /// is the only non-vacuous proof that CiTO is now graph-faithful — praxis's
    /// FIRST byte-exact OWL vocabulary.
    ///
    /// Bundled in-tree (the SPAR canonical publication), so unlike the 89 MB
    /// WordNet corpus there is no graceful skip — a missing file is a defect.
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
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
        let hash = ContentAddress::of(&out).to_hex();
        assert_eq!(
            hash,
            ContentAddress::of(&source).to_hex(),
            "reconstructed CiTO must hash-equal the source"
        );
        assert_eq!(
            hash, "58061b9db3e5c1739f1e17691597f6178bda0fb62d7801d57bee72185d2ebbdd",
            "reconstructed CiTO must hash to the pinned praxis.lock [hashes] cito@2.8.1 source pin"
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
    #[pr4xis::praxis_value(Deterministic)]
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

    // =========================================================================
    // The byte-exact OWL vocab family — biro / c4o / doco.
    //
    // These three SPAR vocabs serialise in the FLAT form (every node — named and
    // blank — a top-level `<rdf:Description>`, no `parseType`, no DOCTYPE, no
    // comments, no numeric character references, no DTD entities), so the
    // structural writer reconstructs each byte-for-byte exactly as it does cito.
    // Each gets the same HARD GATE as cito: a real-source byte-exact reconstruct
    // against the pinned `[hashes]` content address, and a corruption meta-test
    // proving the gate has teeth.
    //
    // prov_o and olia (the STRIPED form) have their OWN gates below — the L3 byte
    // kernel captures the concrete syntax that once blocked them (the verbatim
    // internal-subset DOCTYPE, the §4.1 numeric `&#39;` and general-entity
    // `&rdfs;` reference forms, the interspersed §2.5 comments) as STRUCTURED
    // residue, so `capture_owl_complement` now succeeds and reconstructs each
    // byte-for-byte; `build_envelope` emits the graph-faithful envelope.
    // =========================================================================

    /// Reconstruct one bundled flat OWL vocab byte-for-byte and assert it hashes
    /// to its pinned `[hashes]` content address — the shared HARD GATE body for
    /// biro/c4o/doco (the cito gate inlined the same logic). `expect_blocks` and
    /// `expect_namespaces` make the capture non-vacuity assertion source-specific.
    fn assert_vocab_byte_exact(file: &str, expect_hash: &str, expect_namespaces: usize) {
        let path = format!("{}/data/ontologies/{}", env!("CARGO_MANIFEST_DIR"), file);
        let source = std::fs::read(&path).unwrap_or_else(|_| panic!("bundled {file} must exist"));
        let src = core::str::from_utf8(&source).expect("vocab is UTF-8");

        let (ont, complement) =
            capture_owl_complement(src).unwrap_or_else(|e| panic!("capture {file}: {e}"));

        // Non-vacuous: the typed graph and the residue both carry real content.
        assert!(
            !ont.classes.is_empty() || !ont.properties.is_empty(),
            "{file}: typed OWL graph carries classes/properties (capture is non-vacuous)"
        );
        assert_eq!(
            complement.document_residue.root_namespaces.len(),
            expect_namespaces,
            "{file}: the <rdf:RDF> xmlns:* declarations are captured"
        );
        assert!(
            !complement.regenerated.content_whitespace.is_empty(),
            "{file}: inter-element indentation captured"
        );
        assert!(
            !complement.regenerated.attribute_overrides.is_empty(),
            "{file}: exact source attribute sequences captured"
        );

        let out = reconstruct_owl_rdfxml_source(&complement)
            .unwrap_or_else(|e| panic!("reconstruct {file}: {e}"));

        if out != source {
            let first = out
                .iter()
                .zip(source.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(source.len().min(out.len()));
            let lo = first.saturating_sub(80);
            panic!(
                "{file}: byte mismatch at offset {first} (out.len()={}, source.len()={})\n  \
                 expected: {:?}\n  got:      {:?}",
                out.len(),
                source.len(),
                String::from_utf8_lossy(&source[lo..(first + 80).min(source.len())]),
                String::from_utf8_lossy(&out[lo..(first + 80).min(out.len())]),
            );
        }
        let hash = ContentAddress::of(&out).to_hex();
        assert_eq!(
            hash,
            ContentAddress::of(&source).to_hex(),
            "{file}: reconstructed must hash-equal the source"
        );
        assert_eq!(
            hash, expect_hash,
            "{file}: reconstructed must hash to the pinned praxis.lock [hashes] source pin"
        );
    }

    /// Capture one bundled flat OWL vocab, prove the baseline reconstructs
    /// byte-exact, then corrupt each residue species (a triple's leaf text, a
    /// blank-node `rdf:nodeID`, a namespace prefix) and assert the reconstruction
    /// DIVERGES — the shared CORRUPTION META-TEST for biro/c4o/doco.
    fn assert_vocab_corruption_diverges(file: &str) {
        let path = format!("{}/data/ontologies/{}", env!("CARGO_MANIFEST_DIR"), file);
        let source = std::fs::read(&path).unwrap_or_else(|_| panic!("bundled {file} must exist"));
        let src = core::str::from_utf8(&source).expect("vocab is UTF-8");
        let (_ont, base) =
            capture_owl_complement(src).unwrap_or_else(|e| panic!("capture {file}: {e}"));

        assert_eq!(
            reconstruct_owl_rdfxml_source(&base).expect("reconstruct base"),
            source,
            "{file}: baseline reconstruction must be byte-exact before corruption"
        );

        // (1) Corrupt a TRIPLE leaf text.
        {
            let mut c = base.clone();
            let mut flipped = false;
            'outer: for block in &mut c.structure.node_blocks {
                if corrupt_first_text(&mut block.properties) {
                    flipped = true;
                    break 'outer;
                }
            }
            assert!(flipped, "{file}: found a leaf-text property to corrupt");
            let out = reconstruct_owl_rdfxml_source(&c).expect("reconstruct corrupt-triple");
            assert_ne!(
                out, source,
                "{file}: corrupting a triple's leaf text must DIVERGE"
            );
        }

        // (2) Corrupt a blank-node rdf:nodeID attribute.
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
            assert!(flipped, "{file}: found an rdf:nodeID attribute to corrupt");
            let out = reconstruct_owl_rdfxml_source(&c).expect("reconstruct corrupt-nodeID");
            assert_ne!(
                out, source,
                "{file}: corrupting a blank-node rdf:nodeID must DIVERGE"
            );
        }

        // (3) Corrupt a namespace prefix.
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
            assert!(flipped, "{file}: found a root xmlns:* prefix to corrupt");
            let out = reconstruct_owl_rdfxml_source(&c).expect("reconstruct corrupt-prefix");
            assert_ne!(
                out, source,
                "{file}: corrupting a namespace prefix must DIVERGE"
            );
        }
    }

    /// Push `__CORRUPT__` onto the first non-empty leaf-text literal in a property
    /// list, recursing into nested members. Returns whether it found one.
    fn corrupt_first_text(props: &mut [RdfPropertyElement]) -> bool {
        for prop in props.iter_mut() {
            match &mut prop.content {
                PropertyContent::Text(t) if !t.is_empty() => {
                    t.push_str("__CORRUPT__");
                    return true;
                }
                PropertyContent::Nested(members) => {
                    for m in members.iter_mut() {
                        if corrupt_first_text(&mut m.properties) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// BiRO byte-exact over the real bundled source (4 root namespaces, all-flat).
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn biro_reconstruct_byte_exact_over_real_source() {
        assert_vocab_byte_exact(
            "biro-1.1.1.owl",
            "0ef855c71718304ebda66c9ce16e7a95fe2be1a78ff5c0e6c4f89c346128225e",
            4,
        );
    }

    /// BiRO corruption meta-test — the gate has teeth.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn biro_corruption_diverges_red() {
        assert_vocab_corruption_diverges("biro-1.1.1.owl");
    }

    /// C4O byte-exact over the real bundled source (5 root namespaces incl. swrl).
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn c4o_reconstruct_byte_exact_over_real_source() {
        assert_vocab_byte_exact(
            "c4o-1.2.owl",
            "542459f6b5bb1e529fdede81440dd452a8646e619533e9249f03ffda01a28ca7",
            5,
        );
    }

    /// C4O corruption meta-test.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn c4o_corruption_diverges_red() {
        assert_vocab_corruption_diverges("c4o-1.2.owl");
    }

    /// DoCO byte-exact over the real bundled source (371 node blocks).
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn doco_reconstruct_byte_exact_over_real_source() {
        assert_vocab_byte_exact(
            "doco-1.3.owl",
            "47bef164db1c2ae524cf06f5e9958b5812c250a3ded3c2b38055d1db71345b20",
            4,
        );
    }

    /// DoCO corruption meta-test.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn doco_corruption_diverges_red() {
        assert_vocab_corruption_diverges("doco-1.3.owl");
    }

    // =========================================================================
    // The STRIPED OWL vocab family — prov_o / olia (the L3 byte kernel slice).
    //
    // These two are the STRIPED RDF/XML form blocked, in the prior slice, BELOW
    // the writer layer by parser-level concrete syntax: an internal-subset DOCTYPE
    // (`<!DOCTYPE rdf:RDF [ <!ENTITY …> ]>`), §4.1 numeric character references
    // (`&#39;`), §4.1 general-entity references (`&rdfs;seeAlso`), and interspersed
    // §2.5 comments. The L3 byte kernel adds each as STRUCTURED concrete-syntax
    // residue (the DOCTYPE verbatim PROLOG residue; the numeric/general
    // `ExtendedRef` form beside the §4.6 predefined `EntityReferenceForm`; the
    // `ChildSlot::InsertComment` residue), so the recursive node-block writer now
    // reconstructs BOTH byte-for-byte — NO stored DOM, NO raw blob. Each gets the
    // same HARD GATE as the flat family: a real-source byte-exact reconstruct
    // against the pinned `[hashes]` content address, and a corruption meta-test.
    // =========================================================================

    /// Shared body for the striped-vocab HARD GATE: capture the real source,
    /// assert the verbatim internal-subset DOCTYPE is genuinely captured (the
    /// headline new residue — a vacuous round-trip would lie), reconstruct, and
    /// hash to the pinned `[hashes]` content address.
    fn assert_striped_vocab_byte_exact(file: &str, expect_hash: &str) {
        let path = format!("{}/data/ontologies/{}", env!("CARGO_MANIFEST_DIR"), file);
        let Ok(source) = std::fs::read(&path) else {
            eprintln!("{file} absent — skipping (large, externally provisioned)");
            return;
        };
        let src = core::str::from_utf8(&source).expect("striped vocab is UTF-8");
        let (_ont, complement) =
            capture_owl_complement(src).unwrap_or_else(|e| panic!("capture {file}: {e}"));

        // Non-vacuity of the NEW L3 residue species: the verbatim internal-subset
        // DOCTYPE with its `<!ENTITY>` declarations is captured.
        assert!(
            complement
                .document_residue
                .doctype
                .as_ref()
                .and_then(|d| d.verbatim.as_deref())
                .is_some_and(|v| v.contains("<!ENTITY")),
            "{file}: the verbatim <!DOCTYPE …[ <!ENTITY …> ]> internal subset is captured"
        );
        assert!(
            !complement.regenerated.content_whitespace.is_empty(),
            "{file}: inter-element/comment residue captured"
        );

        let out = reconstruct_owl_rdfxml_source(&complement)
            .unwrap_or_else(|e| panic!("reconstruct {file}: {e}"));
        if out != source {
            let first = out
                .iter()
                .zip(source.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(source.len().min(out.len()));
            let lo = first.saturating_sub(80);
            panic!(
                "{file}: byte mismatch at offset {first} (out.len()={}, source.len()={})\n  \
                 expected: {:?}\n  got:      {:?}",
                out.len(),
                source.len(),
                String::from_utf8_lossy(&source[lo..(first + 80).min(source.len())]),
                String::from_utf8_lossy(&out[lo..(first + 80).min(out.len())]),
            );
        }
        assert_eq!(
            ContentAddress::of(&out).to_hex(),
            expect_hash,
            "{file}: reconstructed must hash to the pinned praxis.lock [hashes] source pin"
        );
    }

    /// prov_o byte-exact over the real bundled source — the L3 PROOF.
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn prov_o_reconstruct_byte_exact_over_real_source() {
        assert_striped_vocab_byte_exact(
            "prov_o-2013-04-30.owl",
            "93a7265674dc138e9ddb3c0722e333aaeeeec2195aa921b24b6743d0540d9f55",
        );
    }

    /// olia byte-exact over the real bundled source (a 1.2 MB striped vocab with
    /// the same DOCTYPE + 145 interspersed comments + general-entity refs).
    #[pr4xis::praxis_value(Deterministic, Verifiable)]
    #[test]
    fn olia_reconstruct_byte_exact_over_real_source() {
        assert_striped_vocab_byte_exact(
            "olia-2026-04-09.owl",
            "e670db8b3142dcd7cc05d07b890039e8fae708ca4699ddfa7a733e32544dfd43",
        );
    }

    /// CORRUPTION META-TEST for the striped family — the gate has teeth on the NEW
    /// residue too. Corrupting the verbatim DOCTYPE internal subset, OR a triple's
    /// leaf text, MUST make the reconstruction diverge from the source.
    fn assert_striped_vocab_corruption_diverges(file: &str) {
        let path = format!("{}/data/ontologies/{}", env!("CARGO_MANIFEST_DIR"), file);
        let Ok(source) = std::fs::read(&path) else {
            eprintln!("{file} absent — skipping corruption meta-test");
            return;
        };
        let src = core::str::from_utf8(&source).expect("striped vocab is UTF-8");
        let (_ont, base) =
            capture_owl_complement(src).unwrap_or_else(|e| panic!("capture {file}: {e}"));
        assert_eq!(
            reconstruct_owl_rdfxml_source(&base).expect("reconstruct base"),
            source,
            "{file}: baseline must be byte-exact before corruption"
        );

        // (1) Corrupt the verbatim DOCTYPE internal subset.
        {
            let mut c = base.clone();
            let dt = c.document_residue.doctype.as_mut().expect("doctype");
            dt.verbatim = dt
                .verbatim
                .as_ref()
                .map(|v| v.replacen("<!ENTITY", "<!ENTITY CORRUPT", 1));
            let out = reconstruct_owl_rdfxml_source(&c).expect("reconstruct corrupt-doctype");
            assert_ne!(
                out, source,
                "{file}: corrupting the DOCTYPE internal subset must DIVERGE"
            );
        }

        // (2) Corrupt a triple leaf text (recursing into nested members).
        {
            let mut c = base.clone();
            let mut flipped = false;
            'outer: for block in &mut c.structure.node_blocks {
                if corrupt_first_text(&mut block.properties) {
                    flipped = true;
                    break 'outer;
                }
            }
            assert!(flipped, "{file}: found a leaf-text property to corrupt");
            let out = reconstruct_owl_rdfxml_source(&c).expect("reconstruct corrupt-triple");
            assert_ne!(
                out, source,
                "{file}: corrupting a triple's leaf text must DIVERGE"
            );
        }
    }

    /// prov_o corruption meta-test.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn prov_o_corruption_diverges_red() {
        assert_striped_vocab_corruption_diverges("prov_o-2013-04-30.owl");
    }

    /// olia corruption meta-test.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn olia_corruption_diverges_red() {
        assert_striped_vocab_corruption_diverges("olia-2026-04-09.owl");
    }
}
