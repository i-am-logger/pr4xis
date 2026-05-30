//! Praxis-native XSD-document →
//! [`XsdOntologyInstance`]
//! projection that takes its input from praxis-xml's [`XmlDocument`] rather
//! than xsd-parser's AST.
//!
//! This is the second lens in the
//! `file ↔ XML ↔ XSD ↔ statute-ontology` chain (see
//! `feedback_praxis_way_fix_at_root`): given the XmlDocument that
//! `crate::social::software::markup::xml::parser::XmlLens` produced from
//! the on-disk XSD bytes, walk the tree and project
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
    AnnotationInfo, DerivationMethod, ElementDeclarationInfo, NamedSchemaComponentEntry,
    SchemaImportInfo, SchemaIncludeInfo, SchemaOverrideInfo, SchemaRedefineInfo,
    TypeDerivationInfo, XsdOntologyInstance,
};
use super::ontology::XsdConcept;
use crate::social::software::markup::xml::ontology::{
    XmlAttribute, XmlDocument, XmlElement, XmlNode,
};

pub mod axioms;

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
        imports: state.imports,
        includes: state.includes,
        redefines: state.redefines,
        overrides: state.overrides,
        annotations: state.annotations,
        derivations: state.derivations,
    }
}

#[derive(Default)]
struct ProjectState {
    components: Vec<XsdConcept>,
    named: Vec<NamedSchemaComponentEntry>,
    elements: Vec<ElementDeclarationInfo>,
    imports: Vec<SchemaImportInfo>,
    includes: Vec<SchemaIncludeInfo>,
    redefines: Vec<SchemaRedefineInfo>,
    overrides: Vec<SchemaOverrideInfo>,
    annotations: Vec<AnnotationInfo>,
    derivations: Vec<TypeDerivationInfo>,
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
    ///
    /// Per Namespaces in XML 1.0 (Bray, Hollander, Layman & Tobin
    /// 2009) §3, every `xmlns` / `xmlns:prefix` attribute on an
    /// element is a namespace declaration, not a regular attribute.
    /// The praxis XML reader surfaces them all on
    /// [`XmlElement::namespaces`] in document order; iterate that
    /// collection so a later declaration shadows an earlier one on
    /// the same prefix (§6.1 — innermost wins).
    fn extend(parent: &Self, element: &XmlElement) -> Self {
        let mut scope = parent.clone();
        for ns in &element.namespaces {
            match &ns.prefix {
                None => scope.default_uri = Some(ns.uri.clone()),
                Some(p) => scope.set_prefix(p, &ns.uri),
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

/// Project a single XSD-namespace element. Dispatch covers:
///
/// - The six schema-component declaration kinds (Part 1 §3.2 / §3.3 /
///   §3.4 / §3.6 / §3.8 / §3.16) — named declarations with a
///   `name="…"` attribute.
/// - The four schema-composition directives (Part 1 §4.2.3 /
///   §4.2.4 / §4.2.5 / §4.2.6) — `<xs:include>`, `<xs:redefine>`,
///   `<xs:override>`, `<xs:import>`.
/// - The three annotation kinds (Part 1 §3.15) — `<xs:annotation>`
///   handled at the top of the walker so its body
///   (`<xs:appinfo>` / `<xs:documentation>`) is consumed without
///   spurious recursion.
///
/// Elements outside this set are ignored at the projection level
/// but their descendants are still walked (the caller controls
/// recursion).
fn project_xsd_declaration(element: &XmlElement, state: &mut ProjectState) {
    match element.name.local.as_str() {
        // -------- §2.5 / §3.16 the schema document itself --------
        // The `<xs:schema>` root element. Every well-formed XSD
        // schema document projects to one SchemaDocument; the
        // components / directives / constructs / facets it contains
        // project to their own leaves via the rest of this dispatch
        // when the walker recurses (W3C XSD 1.1 Part 1 §2.5 "Schema
        // Document" + §3.16 "Schemas as Wholes"). An empty
        // `<xs:schema/>` is a valid Schema Document with zero
        // components — it still projects to SchemaDocument.
        "schema" => {
            state.components.push(XsdConcept::SchemaDocument);
            return;
        }
        // -------- §3.15 Annotations --------
        "annotation" => {
            state.components.push(XsdConcept::Annotation);
            state.annotations.push(project_annotation(element));
            return;
        }
        // -------- §4.2 Schema composition directives --------
        "include" => {
            if let Some(loc) = attr_value(element, "schemaLocation") {
                state.components.push(XsdConcept::SchemaInclude);
                state.includes.push(SchemaIncludeInfo {
                    schema_location: loc,
                });
            }
            return;
        }
        "redefine" => {
            if let Some(loc) = attr_value(element, "schemaLocation") {
                state.components.push(XsdConcept::SchemaRedefine);
                state.redefines.push(SchemaRedefineInfo {
                    schema_location: loc,
                });
            }
            return;
        }
        "override" => {
            if let Some(loc) = attr_value(element, "schemaLocation") {
                state.components.push(XsdConcept::SchemaOverride);
                state.overrides.push(SchemaOverrideInfo {
                    schema_location: loc,
                });
            }
            return;
        }
        "import" => {
            state.components.push(XsdConcept::SchemaImport);
            state.imports.push(SchemaImportInfo {
                namespace: attr_value(element, "namespace"),
                schema_location: attr_value(element, "schemaLocation"),
            });
            return;
        }
        // -------- §3.8 Model groups (anonymous; no name=) --------
        "sequence" => {
            state.components.push(XsdConcept::Sequence);
            return;
        }
        "choice" => {
            state.components.push(XsdConcept::Choice);
            return;
        }
        "all" => {
            state.components.push(XsdConcept::AllGroup);
            return;
        }
        // §3.10 Wildcards — `<xs:any>` / `<xs:anyAttribute>`.
        "any" | "anyAttribute" => {
            state.components.push(XsdConcept::Wildcard);
            return;
        }
        // §3.14 Notation Declarations — `<xs:notation>` binds a name to
        // a public / system identifier pair (the XSD analogue of an XML
        // 1.0 NOTATION declaration, Bray et al. 2008 §4.7). Anonymous at
        // the projection level (the name + public/system attributes
        // live downstream in xsd-parser's AST).
        "notation" => {
            state.components.push(XsdConcept::NotationDeclaration);
            return;
        }
        // -------- §3.4.2 Type-construction content wrappers --------
        "complexContent" => {
            state.components.push(XsdConcept::ComplexContent);
            return;
        }
        "simpleContent" => {
            state.components.push(XsdConcept::SimpleContent);
            return;
        }
        // §3.4.6 Type-derivation methods — capture the `base=` edge.
        "restriction" => {
            state.components.push(XsdConcept::Restriction);
            state.derivations.push(TypeDerivationInfo {
                method: DerivationMethod::Restriction,
                base: attr_value(element, "base"),
            });
            return;
        }
        "extension" => {
            state.components.push(XsdConcept::Extension);
            state.derivations.push(TypeDerivationInfo {
                method: DerivationMethod::Extension,
                base: attr_value(element, "base"),
            });
            return;
        }
        // §3.16 / Part 2 §4.1.2 simple-type varieties.
        "list" => {
            state.components.push(XsdConcept::ListType);
            return;
        }
        "union" => {
            state.components.push(XsdConcept::UnionType);
            return;
        }
        // -------- Part 2 §4.3 constraining facets --------
        "length" => {
            state.components.push(XsdConcept::LengthFacet);
            return;
        }
        "minLength" => {
            state.components.push(XsdConcept::MinLengthFacet);
            return;
        }
        "maxLength" => {
            state.components.push(XsdConcept::MaxLengthFacet);
            return;
        }
        "pattern" => {
            state.components.push(XsdConcept::PatternFacet);
            return;
        }
        "enumeration" => {
            state.components.push(XsdConcept::EnumerationFacet);
            return;
        }
        "whiteSpace" => {
            state.components.push(XsdConcept::WhiteSpaceFacet);
            return;
        }
        "maxInclusive" => {
            state.components.push(XsdConcept::MaxInclusiveFacet);
            return;
        }
        "maxExclusive" => {
            state.components.push(XsdConcept::MaxExclusiveFacet);
            return;
        }
        "minExclusive" => {
            state.components.push(XsdConcept::MinExclusiveFacet);
            return;
        }
        "minInclusive" => {
            state.components.push(XsdConcept::MinInclusiveFacet);
            return;
        }
        "totalDigits" => {
            state.components.push(XsdConcept::TotalDigitsFacet);
            return;
        }
        "fractionDigits" => {
            state.components.push(XsdConcept::FractionDigitsFacet);
            return;
        }
        "explicitTimezone" => {
            state.components.push(XsdConcept::ExplicitTimezoneFacet);
            return;
        }
        // §4.3.13 assertion facet (simple types). The complex-type
        // `<xs:assert>` (§3.13) is a distinct construct handled in
        // M4.λ.2.c.3; the facet `<xs:assertion>` is projected here.
        "assertion" => {
            state.components.push(XsdConcept::AssertionFacet);
            return;
        }
        // -------- §3.11 identity constraints (anonymous) --------
        "key" => {
            state.components.push(XsdConcept::Key);
            return;
        }
        "keyref" => {
            state.components.push(XsdConcept::KeyRef);
            return;
        }
        "unique" => {
            state.components.push(XsdConcept::Unique);
            return;
        }
        "selector" => {
            state.components.push(XsdConcept::Selector);
            return;
        }
        "field" => {
            state.components.push(XsdConcept::Field);
            return;
        }
        // -------- XSD 1.1 complex-type content additions --------
        // §3.13 complex-type assertion.
        "assert" => {
            state.components.push(XsdConcept::Assert);
            return;
        }
        // §3.4.2.2 open content on a complex type.
        "openContent" => {
            state.components.push(XsdConcept::OpenContent);
            return;
        }
        // §3.16.2 schema-level default open content.
        "defaultOpenContent" => {
            state.components.push(XsdConcept::DefaultOpenContent);
            return;
        }
        _ => {}
    }

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

/// Project a `<xs:annotation>` element into a typed [`Annotation`].
/// Per W3C XSD 1.1 Part 1 §3.15, an annotation's body is zero or
/// more `<xs:appinfo>` and `<xs:documentation>` elements; we capture
/// the concatenated text content of each.
fn project_annotation(element: &XmlElement) -> AnnotationInfo {
    let mut annotation = AnnotationInfo::default();
    for child in &element.children {
        if let XmlNode::Element(el) = child {
            match el.name.local.as_str() {
                "appinfo" => annotation.appinfo.push(text_content(el)),
                "documentation" => annotation.documentation.push(text_content(el)),
                _ => {}
            }
        }
    }
    annotation
}

/// Concatenate all `XmlNode::Text` / `XmlNode::CData` children's
/// values into a single string. The W3C XSD 1.1 annotation
/// productions (§3.15) say annotation children are "any well-formed
/// XML content"; consumers typically want the text payload, so we
/// surface that and leave nested-element extraction to callers if
/// they need it.
fn text_content(element: &XmlElement) -> String {
    let mut out = String::new();
    for child in &element.children {
        match child {
            XmlNode::Text(t) | XmlNode::CData(t) => out.push_str(t),
            _ => {}
        }
    }
    out
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
