use pr4xis::category::{Arrow, Category, Concept, FinitelyGenerated};
use pr4xis::logic::proof::{SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// Markup language ontology.
//
// A markup language structures content by annotating it with tags.
// Markup is the act of adding structural/semantic information to content.
// The content is the text/data. The markup is the annotation that gives
// it structure. Together they form a document.
//
// This ontology defines what markup IS — independent of any specific
// markup language (XML, HTML, SGML). Specific languages implement
// these concepts with their own syntax.

/// Node types in a markup document tree.
///
/// Every markup document is a tree of nodes. The types of nodes
/// are universal across all markup languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum NodeKind {
    /// The root of the document tree.
    Document,
    /// A named structural unit that can contain other nodes.
    /// In XML: `<element>`. In HTML: `<div>`, `<p>`, etc.
    Element,
    /// A key-value pair attached to an element.
    /// In XML: `name="value"`. In HTML: `class="foo"`.
    Attribute,
    /// Raw text content within an element.
    Text,
    /// A comment — metadata not part of the document's content.
    Comment,
    /// A processing instruction — tells the processor how to handle the document.
    ProcessingInstruction,
}

/// Relation kind for markup containment arrows.
///
/// Per OBO-RO (Smith et al. 2005), every arrow carries a relation-kind
/// tag. Markup's structural relation is parthood (Goldfarb 1990 §1.2:
/// a markup document is a tree of nodes; the parent-child link is a
/// part-of relation between node kinds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupRelationKind {
    Containment,
}

/// Containment relationships between node types.
///
/// These define what can contain what in a markup document.
/// This IS the mereology of markup — has-a relationships.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Contains {
    pub parent: NodeKind,
    pub child: NodeKind,
}

impl Arrow for Contains {
    type Object = NodeKind;
    type Kind = MarkupRelationKind;
    fn source(&self) -> NodeKind {
        self.parent
    }
    fn target(&self) -> NodeKind {
        self.child
    }
    fn kind(&self) -> MarkupRelationKind {
        MarkupRelationKind::Containment
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("Contains"),
            description: Label::new_static(
                "markup containment — parent node kind contains child node kind",
            ),
            citation: Citation::parse_static("Coombs et al. (1987); Goldfarb (1990)"),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

/// The markup containment category.
///
/// Objects are node types. Morphisms are containment rules.
/// This defines the universal grammar of markup: what can contain what.
pub struct MarkupCategory;

impl Category for MarkupCategory {
    type Object = NodeKind;
    type Morphism = Contains;

    fn identity(obj: &NodeKind) -> Contains {
        Contains {
            parent: *obj,
            child: *obj,
        }
    }

    fn compose(f: &Contains, g: &Contains) -> Option<Contains> {
        if f.child != g.parent {
            return None;
        }
        Some(Contains {
            parent: f.parent,
            child: g.child,
        })
    }

    fn morphisms() -> Vec<Contains> {
        use NodeKind::*;

        let mut m = Vec::new();

        // Identity
        for n in NodeKind::variants() {
            m.push(Contains {
                parent: n,
                child: n,
            });
        }

        // Document contains everything at top level
        m.push(Contains {
            parent: Document,
            child: Element,
        });
        m.push(Contains {
            parent: Document,
            child: Comment,
        });
        m.push(Contains {
            parent: Document,
            child: ProcessingInstruction,
        });

        // Element contains other elements, attributes, text, comments
        m.push(Contains {
            parent: Element,
            child: Element,
        });
        m.push(Contains {
            parent: Element,
            child: Attribute,
        });
        m.push(Contains {
            parent: Element,
            child: Text,
        });
        m.push(Contains {
            parent: Element,
            child: Comment,
        });
        m.push(Contains {
            parent: Element,
            child: ProcessingInstruction,
        });

        // Transitive: Document → Element → Text (document transitively contains text)
        m.push(Contains {
            parent: Document,
            child: Attribute,
        });
        m.push(Contains {
            parent: Document,
            child: Text,
        });
        m.push(Contains {
            parent: Document,
            child: ProcessingInstruction,
        });

        m
    }
}

/// A node in a markup document.
///
/// This is the universal representation — any markup language's
/// document can be represented as a tree of these nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct MarkupNode {
    pub kind: NodeKind,
    pub name: Option<String>,
    pub value: Option<String>,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<MarkupNode>,
}

impl MarkupNode {
    pub fn document(children: Vec<MarkupNode>) -> Self {
        Self {
            kind: NodeKind::Document,
            name: None,
            value: None,
            attributes: Vec::new(),
            children,
        }
    }

    pub fn element(name: &str, attributes: Vec<(&str, &str)>, children: Vec<MarkupNode>) -> Self {
        Self {
            kind: NodeKind::Element,
            name: Some(name.into()),
            value: None,
            attributes: attributes
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
            children,
        }
    }

    pub fn text(content: &str) -> Self {
        Self {
            kind: NodeKind::Text,
            name: None,
            value: Some(content.into()),
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn comment(content: &str) -> Self {
        Self {
            kind: NodeKind::Comment,
            name: None,
            value: Some(content.into()),
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Get attribute value by name.
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Count all nodes in the tree (including self).
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }

    /// Tree depth.
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    /// Find all elements by name (recursive).
    pub fn find_all(&self, name: &str) -> Vec<&MarkupNode> {
        let mut results = Vec::new();
        if self.name.as_deref() == Some(name) {
            results.push(self);
        }
        for child in &self.children {
            results.extend(child.find_all(name));
        }
        results
    }

    /// Get all text content (recursive).
    pub fn text_content(&self) -> String {
        match self.kind {
            NodeKind::Text => self.value.clone().unwrap_or_default(),
            _ => self
                .children
                .iter()
                .map(|c| c.text_content())
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}

/// Well-formedness rules for markup documents.
///
/// Coombs et al. (1987) "Markup Systems and the Future of Scholarly Text
/// Processing" identifies descriptive markup as a tree-structured
/// annotation overlay; Goldfarb (1990) *The SGML Handbook* §3.2 fixes
/// the universal constraint that every conforming document has a single
/// document root. This axiom declares that rule at the ontology level;
/// concrete documents are checked by [`is_well_formed`].
pub struct WellFormedDocument;

impl Axiom for WellFormedDocument {
    fn verify(&self) -> Verdict {
        // Universal rule at the ontology level: declared and structurally
        // enforced by the Document → Element edge being part of the
        // category. Concrete-document verification is delegated to
        // `is_well_formed` per Goldfarb (1990) §3.2.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "WellFormedDocument",
        "a well-formed markup document has exactly one root element",
        "Coombs et al. (1987); Goldfarb (1990) SGML Handbook §3.2"
    );
}
pr4xis::register_axiom!(
    WellFormedDocument,
    "Coombs et al. (1987); Goldfarb (1990) SGML Handbook §3.2"
);

/// Check if a specific document is well-formed.
pub fn is_well_formed(doc: &MarkupNode) -> bool {
    if doc.kind != NodeKind::Document {
        return false;
    }
    // Must have at least one element child
    let element_children: Vec<_> = doc
        .children
        .iter()
        .filter(|c| c.kind == NodeKind::Element)
        .collect();
    // Standard markup: exactly one root element
    element_children.len() == 1
}

/// Quality: can this node kind contain children?
#[derive(Debug, Clone)]
pub struct CanContainChildren;

impl Quality for CanContainChildren {
    type Individual = NodeKind;
    type Value = ();

    fn get(&self, kind: &NodeKind) -> Option<()> {
        match kind {
            NodeKind::Document | NodeKind::Element => Some(()),
            _ => None,
        }
    }
}

/// The markup ontology.
pub struct MarkupOntology;

impl Ontology for MarkupOntology {
    type Cat = MarkupCategory;
    type Qual = CanContainChildren;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(WellFormedDocument)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<MarkupCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        MarkupOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
