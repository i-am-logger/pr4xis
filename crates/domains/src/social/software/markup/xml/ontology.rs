use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::logic::proof::{SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use super::super::ontology::{MarkupNode, NodeKind};

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

/// An XML element — the rich type (not just a string).
#[derive(Debug, Clone, PartialEq)]
pub struct XmlElement {
    pub name: XmlName,
    pub namespace: Option<XmlNamespace>,
    pub attributes: Vec<XmlAttribute>,
    pub children: Vec<XmlNode>,
}

/// An XML qualified name (optional prefix + local name).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
pub struct XmlNamespace {
    pub prefix: Option<String>,
    pub uri: String,
}

/// An XML attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct XmlAttribute {
    pub name: XmlName,
    pub value: String,
}

/// An XML node — the universal representation of XML content.
#[derive(Debug, Clone, PartialEq)]
pub enum XmlNode {
    Element(XmlElement),
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
    pub root: XmlElement,
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
/// content of any other element." This is enforced structurally by
/// [`XmlDocument`] holding exactly one `root: XmlElement` field; the axiom
/// asserts the rule at the ontology level.
pub struct SingleRootElement;

impl Axiom for SingleRootElement {
    fn verify(&self) -> Verdict {
        // Structural: enforced by XmlDocument having a single `root` field.
        // The W3C rule is satisfied at the type level — there is no way to
        // construct a multi-root XmlDocument.
        Ok(Box::new(SimpleProof::new(self.meta())))
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
/// content, and end-tag form a contiguous, non-overlapping span. This
/// constraint is enforced structurally by the [`XmlNode`] tree
/// representation — a tree cannot encode overlapping spans.
pub struct ProperNesting;

impl Axiom for ProperNesting {
    fn verify(&self) -> Verdict {
        // Structural: enforced by the tree representation. A tree of
        // XmlNode values cannot encode overlapping tags by construction.
        Ok(Box::new(SimpleProof::new(self.meta())))
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

    #[test]
    fn category_laws() {
        assert_category_laws::<XmlCategory>();
    }

    #[test]
    fn ontology_validates() {
        XmlOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
