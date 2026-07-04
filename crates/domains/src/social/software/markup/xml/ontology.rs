use pr4xis::category::{Arrow, Category, Concept, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use super::super::ontology::{MarkupNode, NodeKind, is_well_formed};

// XML ontology — from W3C XML 1.0 Specification (Fifth Edition)
// https://www.w3.org/TR/xml/
//
// XML (eXtensible Markup Language) is a W3C standard markup language
// for encoding structured documents. This ontology defines what XML IS
// through the symbols it uses and the rules it imposes.

/// XML-specific element types — extensions of the base markup NodeKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum XmlNodeKind {
    /// The XML document (has prolog + root element).
    Document,
    /// An XML element: `<name attr="val">children</name>`.
    Element,
    /// An attribute: `name="value"` on an element.
    Attribute,
    /// Text content within an element (PCDATA).
    Text,
    /// CDATA section: `<![CDATA[...]]>` — literal text, no escaping.
    CData,
    /// Comment: `<!-- ... -->`.
    Comment,
    /// Processing instruction: `<?target data?>`.
    ProcessingInstruction,
    /// XML declaration: `<?xml version="1.0" encoding="UTF-8"?>`.
    XmlDeclaration,
    /// Document Type Declaration (DTD reference).
    DocType,
    /// A namespace declaration: `xmlns:prefix="uri"`.
    Namespace,
}

/// Relation kind for XML containment arrows.
///
/// Per OBO-RO (Smith et al. 2005), every arrow carries a relation-kind
/// tag. XML's structural relation is the W3C-defined parent-child
/// containment between node kinds (W3C XML 1.0 §2.1, §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlRelationKind {
    Containment,
}

/// XML containment relationships.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XmlContains {
    pub parent: XmlNodeKind,
    pub child: XmlNodeKind,
}

impl Arrow for XmlContains {
    type Object = XmlNodeKind;
    type Kind = XmlRelationKind;
    fn source(&self) -> XmlNodeKind {
        self.parent
    }
    fn target(&self) -> XmlNodeKind {
        self.child
    }
    fn kind(&self) -> XmlRelationKind {
        XmlRelationKind::Containment
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("XmlContains"),
            description: Label::new_static(
                "XML containment — parent node kind contains child node kind per W3C XML 1.0 §3",
            ),
            citation: Citation::parse_static("W3C XML 1.0 (2008) Fifth Edition §2.1, §3"),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

/// The XML category — W3C well-formedness rules as category laws.
pub struct XmlCategory;

impl Category for XmlCategory {
    type Object = XmlNodeKind;
    type Morphism = XmlContains;

    fn identity(obj: &XmlNodeKind) -> XmlContains {
        XmlContains {
            parent: *obj,
            child: *obj,
        }
    }

    fn compose(f: &XmlContains, g: &XmlContains) -> Option<XmlContains> {
        if f.child != g.parent {
            return None;
        }
        if f.parent == f.child {
            return Some(g.clone());
        }
        if g.parent == g.child {
            return Some(f.clone());
        }
        Some(XmlContains {
            parent: f.parent,
            child: g.child,
        })
    }

    fn morphisms() -> Vec<XmlContains> {
        use XmlNodeKind::*;
        let mut m = Vec::new();

        // Identity
        for n in XmlNodeKind::variants() {
            m.push(XmlContains {
                parent: n,
                child: n,
            });
        }

        // Document contains (W3C: document = prolog, element, Misc*)
        m.push(XmlContains {
            parent: Document,
            child: XmlDeclaration,
        });
        m.push(XmlContains {
            parent: Document,
            child: DocType,
        });
        m.push(XmlContains {
            parent: Document,
            child: Element,
        });
        m.push(XmlContains {
            parent: Document,
            child: Comment,
        });
        m.push(XmlContains {
            parent: Document,
            child: ProcessingInstruction,
        });

        // Element contains (W3C: element = content | EmptyElemTag)
        m.push(XmlContains {
            parent: Element,
            child: Element,
        });
        m.push(XmlContains {
            parent: Element,
            child: Attribute,
        });
        m.push(XmlContains {
            parent: Element,
            child: Namespace,
        });
        m.push(XmlContains {
            parent: Element,
            child: Text,
        });
        m.push(XmlContains {
            parent: Element,
            child: CData,
        });
        m.push(XmlContains {
            parent: Element,
            child: Comment,
        });
        m.push(XmlContains {
            parent: Element,
            child: ProcessingInstruction,
        });

        // Transitive closure (Document → Element → *)
        for child in [Attribute, Namespace, Text, CData] {
            m.push(XmlContains {
                parent: Document,
                child,
            });
        }

        m
    }
}

/// The symbols XML uses and their meanings.
/// In XML, these characters have special meaning (unlike in English):
/// - `<` = element open tag start
/// - `>` = tag end
/// - `&` = entity reference start
/// - `"` and `'` = attribute value delimiters
/// - `=` = attribute name-value separator
/// - `/` = empty element or closing tag
/// - `?` = processing instruction delimiter
/// - `!` = comment/doctype/CDATA prefix
pub struct XmlSymbols;

impl XmlSymbols {
    /// Characters that have special meaning in XML.
    pub fn special_chars() -> Vec<(char, &'static str)> {
        vec![
            ('<', "element open / tag start"),
            ('>', "tag end"),
            ('&', "entity reference start"),
            ('"', "attribute value delimiter (double)"),
            ('\'', "attribute value delimiter (single)"),
            ('=', "attribute name-value separator"),
            ('/', "closing tag or empty element"),
            ('?', "processing instruction delimiter"),
            ('!', "comment/doctype/CDATA prefix"),
        ]
    }

    /// XML predefined entity references.
    pub fn entities() -> Vec<(&'static str, char)> {
        vec![
            ("&lt;", '<'),
            ("&gt;", '>'),
            ("&amp;", '&'),
            ("&quot;", '"'),
            ("&apos;", '\''),
        ]
    }
}

// # rkyv-serializability under `prx`
//
// The graph-faithful WordNet `.prx` envelope carries the SourceSyntax
// COMPLEMENT ([`WnSyntaxComplement`](super::lmf::writer::WnSyntaxComplement)),
// whose residue types ([`DocumentResidue`](super::parser::source_syntax::DocumentResidue),
// [`ElementAttributes`](super::parser::source_syntax::ElementAttributes),
// `NodeDecisions`, …) reference these concrete-syntax XML types directly —
// [`XmlDoctype`] / [`XmlNamespace`] / [`XmlAttribute`] / [`XmlElement`] /
// [`XmlNode`] / [`XmlName`] / [`XmlExternalId`] / [`XmlGeneralEntity`] /
// [`XmlEntityKind`]. So each must be rkyv-serializable under the `prx` feature
// where the archive consumes it. The derive is CFG-GATED
// (`#[cfg_attr(feature = "prx", derive(rkyv::Archive, …))]`): rkyv is an
// OPTIONAL dependency (`prx = ["dep:rkyv", …]`), so an unconditional derive
// would break the default + wasm32 builds that do not link it. Gated, it is
// present exactly where it is needed and absent everywhere else — the same
// discipline the WN-LMF model types use.

/// An XML element — the rich type (not just a string).
///
/// `namespace` retains the first `xmlns` / `xmlns:prefix` declaration on
/// the element (for backward compatibility); `namespaces` carries *every*
/// such declaration in document order, as required by Namespaces in XML
/// 1.0 (Bray, Hollander, Layman & Tobin 2009 §3 — Declaring Namespaces).
/// Consumers that need the full prefix→URI map (e.g. RDF/XML, which must
/// expand a predicate element's `(prefix, local)` against the in-scope
/// namespaces per RDF 1.1 XML Syntax §2.4) read `namespaces`; consumers
/// that only need a single representative declaration continue to read
/// `namespace`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
// `XmlElement` and `XmlNode` are mutually recursive (`children: Vec<XmlNode>`,
// `XmlNode::Element(XmlElement)`). rkyv's derive would otherwise recurse
// infinitely when computing the `Archive`/`Serialize`/`Deserialize` where-bounds
// (rustc E0275 overflow), so the recursive field carries `#[rkyv(omit_bounds)]`
// and the container supplies the resolved serialize/deserialize/bytecheck bounds
// explicitly — the canonical rkyv 0.8 recursive-type pattern (rkyv book,
// "Derive macro features"). The attributes are CFG-GATED on `prx`: without rkyv
// linked there is no `#[rkyv(…)]` helper to read, so the default + wasm32 builds
// see a plain struct.
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
pub struct XmlElement {
    pub name: XmlName,
    pub namespace: Option<XmlNamespace>,
    /// Every `xmlns` / `xmlns:prefix` declaration on this element, in
    /// document order (Bray, Hollander, Layman & Tobin 2009 §3). An
    /// element with no declarations has an empty `Vec`.
    pub namespaces: Vec<XmlNamespace>,
    pub attributes: Vec<XmlAttribute>,
    #[cfg_attr(feature = "prx", rkyv(omit_bounds))]
    pub children: Vec<XmlNode>,
}

/// An XML qualified name (optional prefix + local name).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct XmlName {
    pub prefix: Option<String>,
    pub local: String,
}

impl XmlName {
    pub fn new(local: &str) -> Self {
        Self {
            prefix: None,
            local: local.into(),
        }
    }

    pub fn with_prefix(prefix: &str, local: &str) -> Self {
        Self {
            prefix: Some(prefix.into()),
            local: local.into(),
        }
    }

    pub fn qualified(&self) -> String {
        match &self.prefix {
            Some(p) => format!("{}:{}", p, self.local),
            None => self.local.clone(),
        }
    }
}

/// An XML namespace declaration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct XmlNamespace {
    pub prefix: Option<String>,
    pub uri: String,
}

/// An XML attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct XmlAttribute {
    pub name: XmlName,
    pub value: String,
}

/// An XML node — the universal representation of XML content.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
// The other half of the `XmlElement` ↔ `XmlNode` mutual recursion — same rkyv
// 0.8 recursive-type treatment as `XmlElement`: `omit_bounds` on the recursive
// `Element(XmlElement)` variant field, explicit container bounds. CFG-gated on
// `prx`.
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
pub enum XmlNode {
    Element(#[cfg_attr(feature = "prx", rkyv(omit_bounds))] XmlElement),
    Text(String),
    CData(String),
    Comment(String),
    ProcessingInstruction {
        target: String,
        data: Option<String>,
    },
}

impl XmlNode {
    /// Convert to the generic markup representation.
    pub fn to_markup(&self) -> MarkupNode {
        match self {
            Self::Element(elem) => {
                let attrs: Vec<(&str, &str)> = elem
                    .attributes
                    .iter()
                    .map(|a| (a.name.local.as_str(), a.value.as_str()))
                    .collect();
                MarkupNode::element(
                    &elem.name.qualified(),
                    attrs,
                    elem.children.iter().map(|c| c.to_markup()).collect(),
                )
            }
            Self::Text(t) => MarkupNode::text(t),
            Self::CData(t) => MarkupNode::text(t),
            Self::Comment(t) => MarkupNode::comment(t),
            Self::ProcessingInstruction { target, data } => {
                let value = match data {
                    Some(d) => format!("{} {}", target, d),
                    None => target.clone(),
                };
                MarkupNode {
                    kind: NodeKind::ProcessingInstruction,
                    name: Some(target.clone()),
                    value: Some(value),
                    attributes: Vec::new(),
                    children: Vec::new(),
                }
            }
        }
    }

    /// Get text content recursively.
    pub fn text_content(&self) -> String {
        match self {
            Self::Text(t) | Self::CData(t) => t.clone(),
            Self::Element(elem) => elem
                .children
                .iter()
                .map(|c| c.text_content())
                .collect::<Vec<_>>()
                .join(""),
            _ => String::new(),
        }
    }
}

impl XmlElement {
    /// Look up an attribute by local name (ignoring prefix).
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|a| a.name.local == name)
            .map(|a| a.value.as_str())
    }

    /// Collect all text content from child nodes recursively.
    pub fn text_content(&self) -> String {
        self.children
            .iter()
            .map(|c| c.text_content())
            .collect::<Vec<_>>()
            .join("")
    }
}

/// An XML document.
#[derive(Debug, Clone, PartialEq)]
pub struct XmlDocument {
    pub version: String,
    pub encoding: Option<String>,
    /// `<!DOCTYPE …>` document type declaration if present (W3C XML
    /// 1.0 Fifth Edition §2.8 production \[28\] doctypedecl).
    pub doctype: Option<XmlDoctype>,
    pub root: XmlElement,
}

/// A `<!DOCTYPE>` document type declaration. Carries the root
/// element name, an optional `ExternalID` (W3C XML 1.0 §4.2.2
/// production \[75\]), and the inline general entity declarations
/// parsed from the internal subset (§4.2 production \[70\] GEDecl).
///
/// Element-type and attribute-list declarations from the internal
/// subset are accepted by the parser but not yet projected to
/// typed values — they affect validity, not well-formedness, and
/// can be added incrementally without breaking the document model.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct XmlDoctype {
    /// The root element name declared by `<!DOCTYPE name …>`.
    pub root_name: String,
    /// `ExternalID` declaration if present. `SYSTEM` form carries
    /// just the system literal; `PUBLIC` form carries both pub-id
    /// and system literal per §4.2.2 \[75\].
    pub external_id: Option<XmlExternalId>,
    /// General entity declarations from the internal subset (§4.2
    /// \[70\] EntityDecl → \[71\] GEDecl). Preserves declaration order
    /// via `Vec` rather than a `HashMap` because XML 1.0 §4.5 says
    /// the first declaration wins on duplicate names.
    pub general_entities: Vec<XmlGeneralEntity>,
    /// True iff the internal subset (or any included PE within it)
    /// contained at least one §2.8 \[28a\] `PEReference` at
    /// `DeclSep` position. The §4.1 WFC: Entity Declared carve-out
    /// — "internal DTD subset which contains no parameter-entity
    /// references" — keys off this flag together with the external
    /// subset's presence and the `standalone` attribute.
    pub internal_subset_had_pe_references: bool,
    /// The VERBATIM bytes of the WHOLE declaration `<!DOCTYPE … >`
    /// (§2.8 \[28\] `doctypedecl`), after §2.11 end-of-line
    /// normalization — concrete-syntax PROLOG residue captured so the
    /// byte-exact serializer reproduces the declaration exactly (the
    /// inter-token white-space, the `<!ENTITY>` internal-subset
    /// layout, and any internal-subset comments the structured
    /// projection erases). This is the prolog analogue of re-emitting
    /// the `<?xml?>` declaration bytes — NOT a stored element-tree
    /// DOM. `None` when the document has no DOCTYPE OR when a synthetic
    /// doctype was built without capture (the structured
    /// `write_doctype` re-projection path); `Some` only on the
    /// capturing read path, so the field is purely additive — a flat
    /// SPAR vocab / WordNet / USC document is unaffected.
    pub verbatim: Option<String>,
}

/// One general-entity declaration from §4.2 \[71\] GEDecl. The
/// `kind` distinguishes the three §4.2 \[73\] EntityDef shapes —
/// the distinction matters because the W3C XML 1.0 §4.4 entity-
/// reference table gives the same syntactic `&name;` reference
/// different well-formedness constraints depending on which kind
/// of entity it points at (WFC: Parsed Entity, WFC: No External
/// Entity References, …).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct XmlGeneralEntity {
    /// The entity name (§4.2 \[71\] Name).
    pub name: String,
    /// Internal entities (§4.2 \[73\] EntityDef → EntityValue) carry
    /// their replacement text here. External entities carry the
    /// empty string — their replacement text is fetched at parse
    /// time, which praxis defers (the parser is non-validating
    /// and does not load external resources).
    pub value: String,
    /// Which of the three §4.2 \[73\] EntityDef shapes declared this.
    pub kind: XmlEntityKind,
}

/// §4.2 \[73\] `EntityDef ::= EntityValue | (ExternalID NDataDecl?)`
/// distinguishes three concrete shapes once `NDataDecl?` is split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum XmlEntityKind {
    /// `<!ENTITY name "value">` — replacement text is the literal
    /// EntityValue. Internal parsed general entity.
    Internal,
    /// `<!ENTITY name SYSTEM "uri">` (or PUBLIC). External parsed
    /// general entity — its replacement text is the entity body,
    /// to be re-parsed in the referring context.
    ExternalParsed,
    /// `<!ENTITY name SYSTEM "uri" NDATA notation>`. External
    /// unparsed general entity per §4.2.2 \[76\] NDataDecl. The
    /// referenced data is not XML; it can only be named (via an
    /// ENTITY-typed attribute value), never expanded into content.
    /// §4.4 row "Reference in Content" + WFC: Parsed Entity
    /// rejects such references in `&name;` positions.
    ExternalUnparsed,
}

/// W3C XML 1.0 §4.2.2 production \[75\] `ExternalID`. Two shapes:
/// `SYSTEM 'uri'` and `PUBLIC 'pubid' 'uri'`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "prx",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum XmlExternalId {
    System {
        system_literal: String,
    },
    Public {
        public_id: String,
        system_literal: String,
    },
}

impl XmlDocument {
    /// Convert to generic markup representation.
    pub fn to_markup(&self) -> MarkupNode {
        MarkupNode::document(vec![XmlNode::Element(self.root.clone()).to_markup()])
    }

    /// Find all elements by name (recursive).
    pub fn find_all(&self, name: &str) -> Vec<&XmlElement> {
        let mut results = Vec::new();
        find_elements_recursive(&self.root, name, &mut results);
        results
    }
}

fn find_elements_recursive<'a>(
    elem: &'a XmlElement,
    name: &str,
    results: &mut Vec<&'a XmlElement>,
) {
    if elem.name.local == name {
        results.push(elem);
    }
    for child in &elem.children {
        if let XmlNode::Element(child_elem) = child {
            find_elements_recursive(child_elem, name, results);
        }
    }
}

/// W3C well-formedness axiom: an XML document must have exactly one root element.
///
/// W3C XML 1.0 (2008) Fifth Edition §2.1: "There is exactly one element,
/// called the root, or document element, no part of which appears in the
/// content of any other element." [`verify`](SingleRootElement::verify)
/// discharges the rule by exercising the real XML reader
/// ([`read_xml`](super::reader::read_xml)) composed with the markup
/// well-formedness check ([`is_well_formed`]): a conforming document is
/// ACCEPTED and projects to a tree with exactly one root element, while a
/// prolog-only document with NO root element is REJECTED. A check that
/// always returned Ok cannot separate the two — this one does, so it is
/// falsifiable rather than a rubber stamp.
pub struct SingleRootElement;

impl Axiom for SingleRootElement {
    fn verify(&self) -> Verdict {
        // Discharge §2.1 concretely against real parser output rather than
        // asserting it at the type level. Read a conforming document, then
        // require its markup projection to carry exactly one root element
        // (`is_well_formed` enforces the count == 1).
        let doc = match super::reader::read_xml("<root><child/><child/></root>") {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let single_root = is_well_formed(&doc.to_markup());
        // A document with no root element (prolog only) must be rejected —
        // §2.1 requires the root/document element to be present.
        let rootless_rejected = super::reader::read_xml("<!-- prolog only, no root -->").is_err();

        if single_root && rootless_rejected {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SingleRootElement",
        "an XML document must have exactly one root element",
        "W3C XML 1.0 (2008) Fifth Edition §2.1"
    );
}
pr4xis::register_axiom!(SingleRootElement, "W3C XML 1.0 (2008) Fifth Edition §2.1");

/// W3C well-formedness axiom: element tags must be properly nested.
///
/// W3C XML 1.0 (2008) Fifth Edition §2.4 ("Character Data and Markup")
/// and §3 ("Logical Structures"): for any non-empty element, the start-tag,
/// content, and end-tag form a contiguous, non-overlapping span — a tree
/// cannot encode overlapping spans. [`verify`](ProperNesting::verify)
/// discharges the rule with the real XML reader
/// ([`read_xml`](super::reader::read_xml)): properly-nested markup is
/// ACCEPTED and parsed into the containment tree it encodes, while
/// overlapping tags (`<a><b></a></b>`) are REJECTED with a mismatched
/// end-tag error. A check that always returned Ok cannot separate the
/// two — this one does, so it is falsifiable.
pub struct ProperNesting;

impl Axiom for ProperNesting {
    fn verify(&self) -> Verdict {
        // Read properly-nested markup and confirm the parsed structure
        // preserves the containment a ⊃ b ⊃ c.
        let doc = match super::reader::read_xml("<a><b><c/></b></a>") {
            Ok(d) => d,
            Err(_) => return Err(Box::new(SimpleCounterexample::new(self.meta()))),
        };
        let nested_ok = doc.root.name.local == "a"
            && matches!(
                doc.root.children.as_slice(),
                [XmlNode::Element(b)]
                    if b.name.local == "b"
                        && matches!(
                            b.children.as_slice(),
                            [XmlNode::Element(c)] if c.name.local == "c"
                        )
            );
        // Overlapping tags cannot form a tree, so the reader must reject
        // the stray end-tag (§2.4, §3).
        let overlap_rejected = super::reader::read_xml("<a><b></a></b>").is_err();

        if nested_ok && overlap_rejected {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ProperNesting",
        "XML elements must be properly nested — no overlapping tags",
        "W3C XML 1.0 (2008) Fifth Edition §2.4, §3"
    );
}
pr4xis::register_axiom!(ProperNesting, "W3C XML 1.0 (2008) Fifth Edition §2.4, §3");

/// Quality: is this XML node kind a content node (can appear inside elements)?
#[derive(Debug, Clone)]
pub struct IsContentNode;

impl Quality for IsContentNode {
    type Individual = XmlNodeKind;
    type Value = ();

    fn get(&self, kind: &XmlNodeKind) -> Option<()> {
        match kind {
            XmlNodeKind::Element
            | XmlNodeKind::Text
            | XmlNodeKind::CData
            | XmlNodeKind::Comment
            | XmlNodeKind::ProcessingInstruction => Some(()),
            _ => None,
        }
    }
}

/// The XML ontology.
pub struct XmlOntology;

impl Ontology for XmlOntology {
    type Cat = XmlCategory;
    type Qual = IsContentNode;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(SingleRootElement), Box::new(ProperNesting)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<XmlCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        XmlOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn single_root_axiom_holds() {
        // The §2.1 axiom accepts a conforming single-root document and
        // rejects a rootless one — its verify() must discharge.
        assert!(SingleRootElement.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn proper_nesting_axiom_holds() {
        // The §2.4/§3 axiom accepts properly-nested markup and rejects
        // overlapping tags — its verify() must discharge.
        assert!(ProperNesting.verify().is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn rootless_document_is_rejected() {
        // §2.1: a prolog-only document has no root element and must be
        // rejected — this is the failing case SingleRootElement detects.
        assert!(super::super::reader::read_xml("<!-- prolog only, no root -->").is_err());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn overlapping_tags_are_rejected() {
        // §2.4/§3: overlapping tags cannot form a tree and must be
        // rejected — this is the failing case ProperNesting detects.
        assert!(super::super::reader::read_xml("<a><b></a></b>").is_err());
    }
}
