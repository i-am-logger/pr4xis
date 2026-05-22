//! Praxis-native XSD-document → [`XsdOntologyInstance`] projection
//! that takes its input from praxis-xml's [`XmlDocument`] rather
//! than xsd-parser's AST.
//!
//! This is the second lens in the
//! `file ↔ XML ↔ XSD ↔ statute-ontology` chain (see
//! `feedback_praxis_way_fix_at_root`): given the XmlDocument that
//! [`crate::social::software::markup::xml::parser::XmlLens`]
//! produced from the on-disk XSD bytes, walk the tree and project
//! every `<xsd:element>` / `<xsd:complexType>` / `<xsd:simpleType>` /
//! `<xsd:attributeGroup>` / `<xsd:group>` / `<xsd:attribute>`
//! declaration in the W3C XSD 1.1 namespace
//! (`http://www.w3.org/2001/XMLSchema`) to the corresponding
//! [`XsdConcept`] instance.
//!
//! Per W3C XML Namespaces 1.0 §6 (Bray, Hollander, Layman & Tobin
//! 2009), schema-component membership is determined by namespace
//! URI, not by prefix — we resolve each element's effective
//! namespace URI through the in-scope `xmlns` / `xmlns:prefix`
//! attribute bindings collected during the descent.
//!
//! # Citations
//!
//! - **Gao, Sperberg-McQueen & Thompson (2012)** *W3C XML Schema
//!   Definition Language (XSD) 1.1 Part 1: Structures*, W3C
//!   Recommendation 5 April 2012:
//!     - §3.3 *Element Declarations* — `<xsd:element>` projection
//!       (also §3.3.2.3 `{type definition}` default to `xs:anyType`
//!       and §3.3.6 *Substitution Groups*).
//!     - §3.4 *Complex Type Definitions* — `<xsd:complexType>`.
//!     - §3.16 *Simple Type Definitions* (Part 2) — `<xsd:simpleType>`.
//!     - §3.6 *Attribute Group Definitions* — `<xsd:attributeGroup>`.
//!     - §3.8 *Model Group Definitions* — `<xsd:group>`.
//!     - §3.2 *Attribute Declarations* — `<xsd:attribute>`.
//! - **Bray, Hollander, Layman & Tobin (2009)** *Namespaces in XML 1.0
//!   Third Edition* §6 — namespace-URI-based identity for elements.

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec::Vec};

use super::XSD_NAMESPACE_URI;
use super::from_xsd_parser::{
    ElementDeclarationInfo, NamedSchemaComponentEntry, XsdOntologyInstance,
};
use super::ontology::XsdConcept;
use crate::social::software::markup::xml::ontology::{
    XmlAttribute, XmlDocument, XmlElement, XmlNode,
};

#[cfg(test)]
mod tests;

/// Project a parsed XSD document into an [`XsdOntologyInstance`].
///
/// Walks the [`XmlDocument`] tree and emits one
/// [`XsdConcept`] / [`NamedSchemaComponentEntry`] per recognised
/// schema-component declaration in the XSD 1.1 namespace.
/// `<xsd:element>` declarations additionally yield an
/// [`ElementDeclarationInfo`] carrying `type=` and
/// `substitutionGroup=` references (W3C XSD 1.1 Part 1 §3.3.2.3,
/// §3.3.6).
pub fn project_from_xml_document(doc: &XmlDocument) -> XsdOntologyInstance {
    let mut state = ProjectState::default();
    let root_scope = NamespaceScope::default();
    walk_element(&doc.root, &root_scope, &mut state);
    XsdOntologyInstance {
        components: state.components,
        named: state.named,
        elements: state.elements,
    }
}

#[derive(Default)]
struct ProjectState {
    components: Vec<XsdConcept>,
    named: Vec<NamedSchemaComponentEntry>,
    elements: Vec<ElementDeclarationInfo>,
}

/// Namespace bindings in scope at an element. Built up by ancestor
/// descent per Bray, Hollander, Layman & Tobin (2009) §6.1
/// "Declaring Namespaces" — child scopes inherit ancestor
/// declarations and override on conflict.
#[derive(Clone, Default)]
struct NamespaceScope {
    default_uri: Option<String>,
    prefixed: Vec<(String, String)>,
}

impl NamespaceScope {
    /// Build a fresh scope from a parent by merging in declarations
    /// found on the current element. The parent stays immutable;
    /// the returned scope is the parent's bindings overridden by
    /// this element's `xmlns` / `xmlns:prefix` declarations.
    fn extend(parent: &Self, element: &XmlElement) -> Self {
        let mut scope = parent.clone();
        // The first xmlns/xmlns:prefix declaration on each element
        // is captured by praxis-xml in `element.namespace`; the rest
        // land in `element.attributes` with the `xmlns` or
        // `xmlns:prefix` naming form.
        if let Some(ns) = &element.namespace {
            match &ns.prefix {
                None => scope.default_uri = Some(ns.uri.clone()),
                Some(p) => scope.set_prefix(p, &ns.uri),
            }
        }
        for attr in &element.attributes {
            if let Some(p) = attr.name.prefix.as_deref() {
                if p == "xmlns" {
                    scope.set_prefix(&attr.name.local, &attr.value);
                }
            } else if attr.name.local == "xmlns" {
                scope.default_uri = Some(attr.value.clone());
            }
        }
        scope
    }

    fn set_prefix(&mut self, prefix: &str, uri: &str) {
        if let Some(existing) = self.prefixed.iter_mut().find(|(p, _)| p == prefix) {
            existing.1 = uri.to_string();
        } else {
            self.prefixed.push((prefix.to_string(), uri.to_string()));
        }
    }

    /// Resolve an element's prefix (or absent prefix for the default
    /// namespace) to its effective URI. `None` means "no namespace"
    /// per W3C XML Namespaces 1.0 §5 "Using Qualified Names".
    fn resolve(&self, prefix: Option<&str>) -> Option<&str> {
        match prefix {
            None => self.default_uri.as_deref(),
            Some(p) => self
                .prefixed
                .iter()
                .rev()
                .find(|(bp, _)| bp == p)
                .map(|(_, uri)| uri.as_str()),
        }
    }
}

fn walk_element(element: &XmlElement, parent_scope: &NamespaceScope, state: &mut ProjectState) {
    let scope = NamespaceScope::extend(parent_scope, element);

    let element_uri = scope.resolve(element.name.prefix.as_deref());
    if element_uri == Some(XSD_NAMESPACE_URI) {
        project_xsd_declaration(element, state);
    }

    for child in &element.children {
        if let XmlNode::Element(el) = child {
            walk_element(el, &scope, state);
        }
    }
}

/// Project a single XSD-namespace element whose local name matches
/// one of the six schema-declaration kinds we recognise.
fn project_xsd_declaration(element: &XmlElement, state: &mut ProjectState) {
    let concept = match element.name.local.as_str() {
        // W3C XSD 1.1 Part 1 §3.3 Element Declarations.
        "element" => XsdConcept::ElementDeclaration,
        // §3.4 Complex Type Definitions.
        "complexType" => XsdConcept::ComplexTypeDefinition,
        // §3.16 Simple Type Definitions (refines Part 2 datatypes).
        "simpleType" => XsdConcept::SimpleTypeDefinition,
        // §3.6 Attribute Group Definitions.
        "attributeGroup" => XsdConcept::AttributeGroup,
        // §3.8 Model Group Definitions.
        "group" => XsdConcept::ModelGroup,
        // §3.2 Attribute Declarations.
        "attribute" => XsdConcept::AttributeDeclaration,
        _ => return,
    };

    // The decision to emit a `NamedSchemaComponentEntry` requires a
    // `name="…"` attribute. XSD permits `ref="…"` references that
    // re-use existing named declarations (Part 1 §3.3.3); those are
    // not declarations and we don't project them here.
    let Some(name) = attr_value(element, "name") else {
        return;
    };

    state.components.push(concept);
    state.named.push(NamedSchemaComponentEntry {
        concept,
        local_name: name.clone(),
    });

    if concept == XsdConcept::ElementDeclaration {
        state.elements.push(ElementDeclarationInfo {
            local_name: name,
            type_ref: attr_value(element, "type"),
            substitution_group_head: attr_value(element, "substitutionGroup"),
        });
    }
}

/// Look up an unqualified attribute by local name. XSD declarations
/// universally use unqualified attribute names (Part 1 §3.1
/// "Schemas and Schema Components") so a prefix-blind match suffices.
fn attr_value(element: &XmlElement, local: &str) -> Option<String> {
    element
        .attributes
        .iter()
        .find(|attr: &&XmlAttribute| attr.name.local == local && attr.name.prefix.is_none())
        .map(|attr| attr.value.clone())
}
