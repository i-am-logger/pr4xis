#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

// XSD ontology — the typed values that an XSD 1.1 schema document encodes.
//
// Reference: W3C XML Schema Definition Language (XSD) 1.1
// Part 1: Structures (Gao, Sperberg-McQueen & Thompson 2012)
// Part 2: Datatypes (Peterson et al. 2012)
//
// The scope of types declared here is the XSD subset used by USLM 1.0.x.
// See `mod.rs` for the full inventory.

/// The W3C XSD 1.1 namespace URI.
///
/// W3C XSD 1.1 Part 1 §3.1.1 declares `http://www.w3.org/2001/XMLSchema`
/// as the namespace for all schema-component elements (`xsd:schema`,
/// `xsd:element`, `xsd:complexType`, …). Element membership in XSD is
/// determined by W3C XML Namespaces 1.0 §6 — namespace-URI match, not
/// prefix.
pub const XSD_NAMESPACE_URI: &str = "http://www.w3.org/2001/XMLSchema";

/// A parsed XSD schema document.
///
/// Maps directly to the top-level `<xsd:schema>` element. Per W3C XSD 1.1
/// Part 1 §3.1.2, an XSD document declares a set of top-level schema
/// components — elements, types, groups, attributeGroups — all collected
/// here.
#[derive(Debug, Clone, PartialEq)]
pub struct XsdSchema {
    /// The `targetNamespace` attribute (W3C XSD 1.1 Part 1 §3.1.2.5).
    /// Documents the namespace this schema declares vocabulary for.
    pub target_namespace: Option<String>,
    /// The `version` attribute on the root `<xsd:schema>` (W3C XSD 1.1
    /// Part 1 §3.1.2.5). USLM uses this to mark its publication version
    /// (e.g. `"1.0.18"`).
    pub version: Option<String>,
    /// `elementFormDefault` attribute (W3C XSD 1.1 Part 1 §3.1.2.5).
    /// "qualified" iff local elements inherit the targetNamespace.
    pub element_form_default: Option<String>,
    /// `attributeFormDefault` attribute (W3C XSD 1.1 Part 1 §3.1.2.5).
    pub attribute_form_default: Option<String>,
    /// Top-level element declarations (W3C XSD 1.1 Part 1 §3.3).
    pub elements: Vec<XsdElement>,
    /// Top-level named complexType declarations (W3C XSD 1.1 Part 1 §3.4).
    pub complex_types: Vec<XsdComplexType>,
    /// Top-level named simpleType declarations (W3C XSD 1.1 Part 1 §3.16).
    pub simple_types: Vec<XsdSimpleType>,
    /// Top-level named attributeGroup declarations
    /// (W3C XSD 1.1 Part 1 §3.6).
    pub attribute_groups: Vec<XsdAttributeGroup>,
    /// Top-level named group declarations (W3C XSD 1.1 Part 1 §3.7).
    pub groups: Vec<XsdGroup>,
    /// `xsd:import` declarations (W3C XSD 1.1 Part 1 §4.2.6.2). Each
    /// names an external namespace this schema references.
    pub imports: Vec<XsdImport>,
}

/// `<xsd:element>` — element declaration (W3C XSD 1.1 Part 1 §3.3).
#[derive(Debug, Clone, PartialEq)]
pub struct XsdElement {
    /// The `name` attribute. Required for top-level element decls; for
    /// nested decls inside content models, `name` may be `None` and
    /// `ref_name` holds the referenced element.
    pub name: Option<String>,
    /// The `ref` attribute (W3C XSD 1.1 Part 1 §3.3.2.4). Used inside a
    /// content model to point at a top-level element decl by name.
    pub ref_name: Option<String>,
    /// The `type` attribute — names the complex or simple type that
    /// defines this element's content.
    pub type_name: Option<String>,
    /// The `substitutionGroup` attribute (W3C XSD 1.1 Part 1 §3.3.2.4):
    /// the head element this declaration substitutes for. USLM uses this
    /// to declare e.g. `<ref>` as a substitute for `<property>`.
    pub substitution_group: Option<String>,
    /// The `abstract` attribute (W3C XSD 1.1 Part 1 §3.3.2.4): an
    /// abstract element cannot itself appear in instance documents; only
    /// concrete substitutes can.
    pub is_abstract: bool,
    /// `minOccurs` (W3C XSD 1.1 Part 1 §3.9.2.4). `None` ⇒ default 1.
    pub min_occurs: Option<Occurs>,
    /// `maxOccurs` (W3C XSD 1.1 Part 1 §3.9.2.4). `None` ⇒ default 1.
    pub max_occurs: Option<Occurs>,
    /// Inline anonymous complex type (W3C XSD 1.1 Part 1 §3.4.2): an
    /// `<xsd:complexType>` nested directly inside `<xsd:element>` instead
    /// of a top-level type reference via `type=`.
    pub inline_complex_type: Option<Box<XsdComplexType>>,
    /// Inline anonymous simple type, same shape as inline complex type
    /// but for simple-typed content.
    pub inline_simple_type: Option<Box<XsdSimpleType>>,
    /// Documentation text from `<xsd:annotation><xsd:documentation>`.
    pub documentation: Option<String>,
}

/// `<xsd:complexType>` — complex type declaration (W3C XSD 1.1 Part 1
/// §3.4).
#[derive(Debug, Clone, PartialEq)]
pub struct XsdComplexType {
    /// The `name` attribute (for top-level types). Inline anonymous types
    /// have `name == None`.
    pub name: Option<String>,
    /// `mixed="true"` allows character data interleaved with child
    /// elements (W3C XSD 1.1 Part 1 §3.4.2.5). USLM uses `mixed="true"`
    /// on inline-markup types (`InlineType`, `NameType`, `DateType`)
    /// so they pass text through.
    pub mixed: bool,
    /// `abstract="true"`: type cannot be instantiated directly.
    pub is_abstract: bool,
    /// The content model — `xsd:sequence`, `xsd:choice`, `xsd:all`, or
    /// a `xsd:complexContent`/`xsd:simpleContent` derivation.
    pub content_model: XsdContentModel,
    /// `xsd:attribute` declarations directly inside this type.
    pub attributes: Vec<XsdAttribute>,
    /// `xsd:attributeGroup ref="..."` declarations.
    pub attribute_group_refs: Vec<String>,
    /// `xsd:anyAttribute` wildcard, if present.
    pub any_attribute: Option<XsdAnyAttribute>,
    /// Documentation text from `<xsd:annotation><xsd:documentation>`.
    pub documentation: Option<String>,
}

/// The content model of a complex type (W3C XSD 1.1 Part 1 §3.4.2.3).
#[derive(Debug, Clone, PartialEq)]
pub enum XsdContentModel {
    /// Empty content model — no element children.
    Empty,
    /// `xsd:complexContent` + `xsd:extension base="…"`: extends another
    /// type by adding particles (W3C XSD 1.1 Part 1 §3.4.2.2).
    ExtensionOf {
        base: String,
        body: Box<XsdContentModel>,
        attributes: Vec<XsdAttribute>,
        attribute_group_refs: Vec<String>,
    },
    /// `xsd:complexContent` + `xsd:restriction base="…"`: restricts
    /// another type (W3C XSD 1.1 Part 1 §3.4.2.2).
    RestrictionOf {
        base: String,
        body: Box<XsdContentModel>,
        attributes: Vec<XsdAttribute>,
        attribute_group_refs: Vec<String>,
    },
    /// `xsd:simpleContent`: a simple-typed body (no element children;
    /// optional attributes). Per W3C XSD 1.1 Part 1 §3.4.2.4.
    SimpleContent {
        base: String,
        attributes: Vec<XsdAttribute>,
        attribute_group_refs: Vec<String>,
    },
    /// A `xsd:sequence` particle (W3C XSD 1.1 Part 1 §3.8).
    Sequence(XsdParticle),
    /// A `xsd:choice` particle.
    Choice(XsdParticle),
    /// A `xsd:all` particle.
    All(XsdParticle),
    /// A `xsd:group ref="…"` reference to a top-level named group.
    GroupRef {
        ref_name: String,
        min_occurs: Option<Occurs>,
        max_occurs: Option<Occurs>,
    },
}

/// A particle in a content model (W3C XSD 1.1 Part 1 §3.9). Encodes the
/// inner content of a `xsd:sequence` / `xsd:choice` / `xsd:all`.
#[derive(Debug, Clone, PartialEq)]
pub struct XsdParticle {
    /// `minOccurs` on the particle itself (default 1).
    pub min_occurs: Option<Occurs>,
    /// `maxOccurs` on the particle itself (default 1).
    pub max_occurs: Option<Occurs>,
    /// The terms inside the particle, in document order.
    pub terms: Vec<XsdTerm>,
}

/// A term inside a particle (W3C XSD 1.1 Part 1 §3.9.2).
#[derive(Debug, Clone, PartialEq)]
pub enum XsdTerm {
    /// A nested element decl or `<xsd:element ref="…"/>`.
    Element(XsdElement),
    /// A nested `<xsd:sequence>`.
    Sequence(XsdParticle),
    /// A nested `<xsd:choice>`.
    Choice(XsdParticle),
    /// A nested `<xsd:all>`.
    All(XsdParticle),
    /// `<xsd:group ref="…"/>` particle.
    GroupRef {
        ref_name: String,
        min_occurs: Option<Occurs>,
        max_occurs: Option<Occurs>,
    },
    /// `<xsd:any>` wildcard.
    Any(XsdAny),
}

/// `<xsd:any>` wildcard particle (W3C XSD 1.1 Part 1 §3.10).
#[derive(Debug, Clone, PartialEq)]
pub struct XsdAny {
    /// `namespace` attribute (whitespace-separated list, or special
    /// values `##any`/`##other`/`##targetNamespace`/`##local`).
    pub namespace: Option<String>,
    /// `processContents` ("strict", "lax", "skip"). Default "strict".
    pub process_contents: Option<String>,
    pub min_occurs: Option<Occurs>,
    pub max_occurs: Option<Occurs>,
}

/// `<xsd:anyAttribute>` wildcard (W3C XSD 1.1 Part 1 §3.10).
#[derive(Debug, Clone, PartialEq)]
pub struct XsdAnyAttribute {
    pub namespace: Option<String>,
    pub process_contents: Option<String>,
}

/// `xsd:simpleType` declaration (W3C XSD 1.1 Part 1 §3.16 + Part 2 §3).
#[derive(Debug, Clone, PartialEq)]
pub struct XsdSimpleType {
    pub name: Option<String>,
    /// The derivation body — `xsd:restriction`, `xsd:union`, or
    /// `xsd:list` per W3C XSD 1.1 Part 2 §4.1.
    pub derivation: XsdSimpleDerivation,
    pub documentation: Option<String>,
}

/// How a simple type is derived (W3C XSD 1.1 Part 2 §4.1):
/// `xsd:restriction`, `xsd:union`, or `xsd:list`.
#[derive(Debug, Clone, PartialEq)]
pub enum XsdSimpleDerivation {
    /// `<xsd:restriction base="…"> … </xsd:restriction>`
    Restriction(XsdSimpleRestriction),
    /// `<xsd:union memberTypes="t1 t2 t3"/>` (W3C XSD 1.1 Part 2 §4.1.2.5).
    /// The member types are a whitespace-separated QName list.
    Union { member_types: Vec<String> },
    /// `<xsd:list itemType="t"/>` (W3C XSD 1.1 Part 2 §4.1.2.4).
    List { item_type: String },
    /// No derivation body present — should not occur in well-formed
    /// XSD but kept as a defensive default so the type is constructible.
    None,
}

impl XsdSimpleType {
    /// Convenience accessor: the restriction body, if this simpleType
    /// was declared with `xsd:restriction`. Returns `None` for union /
    /// list / no-derivation cases. Preserves the pre-derivation API
    /// for callers that only care about restrictions.
    pub fn restriction(&self) -> Option<&XsdSimpleRestriction> {
        match &self.derivation {
            XsdSimpleDerivation::Restriction(r) => Some(r),
            _ => None,
        }
    }
}

/// A `xsd:simpleType > xsd:restriction` body.
#[derive(Debug, Clone, PartialEq)]
pub struct XsdSimpleRestriction {
    pub base: String,
    /// `xsd:enumeration value="…"` facets (W3C XSD 1.1 Part 2 §4.3.5).
    pub enumerations: Vec<String>,
    /// `xsd:pattern value="…"` facets (W3C XSD 1.1 Part 2 §4.3.4).
    pub patterns: Vec<String>,
}

/// `<xsd:attribute>` declaration (W3C XSD 1.1 Part 1 §3.2).
#[derive(Debug, Clone, PartialEq)]
pub struct XsdAttribute {
    pub name: Option<String>,
    pub ref_name: Option<String>,
    pub type_name: Option<String>,
    /// `use` attribute — "required", "optional" (default), "prohibited".
    pub usage: Option<String>,
    pub default: Option<String>,
    pub fixed: Option<String>,
    pub documentation: Option<String>,
}

/// `<xsd:attributeGroup>` declaration (W3C XSD 1.1 Part 1 §3.6).
#[derive(Debug, Clone, PartialEq)]
pub struct XsdAttributeGroup {
    pub name: String,
    pub attributes: Vec<XsdAttribute>,
    /// Nested `<xsd:attributeGroup ref="…"/>` references.
    pub attribute_group_refs: Vec<String>,
    pub any_attribute: Option<XsdAnyAttribute>,
    pub documentation: Option<String>,
}

/// `<xsd:group>` named model-group declaration (W3C XSD 1.1 Part 1 §3.7).
#[derive(Debug, Clone, PartialEq)]
pub struct XsdGroup {
    pub name: String,
    pub content_model: XsdContentModel,
    pub documentation: Option<String>,
}

/// `<xsd:import>` declaration (W3C XSD 1.1 Part 1 §4.2.6.2).
#[derive(Debug, Clone, PartialEq)]
pub struct XsdImport {
    pub namespace: Option<String>,
    pub schema_location: Option<String>,
}

/// An occurrence count (W3C XSD 1.1 Part 1 §3.9.2.4).
///
/// `minOccurs` is a non-negative integer; `maxOccurs` is either a
/// non-negative integer or the literal string `"unbounded"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Occurs {
    /// A specific bound (parsed integer).
    Count(u32),
    /// The `"unbounded"` literal.
    Unbounded,
}

impl Occurs {
    /// Returns the minimum-occurs value for a particle that omits the
    /// attribute (W3C XSD 1.1 Part 1 §3.9.2.4 default).
    pub const fn default_min() -> Self {
        Self::Count(1)
    }

    /// Returns the maximum-occurs value for a particle that omits the
    /// attribute (W3C XSD 1.1 Part 1 §3.9.2.4 default).
    pub const fn default_max() -> Self {
        Self::Count(1)
    }
}

// ---------------------------------------------------------------------------
// Helpers — schema component lookup
// ---------------------------------------------------------------------------

impl XsdSchema {
    /// Find a top-level element declaration by name.
    pub fn element(&self, name: &str) -> Option<&XsdElement> {
        self.elements
            .iter()
            .find(|e| e.name.as_deref() == Some(name))
    }

    /// Find a top-level named complexType by name.
    pub fn complex_type(&self, name: &str) -> Option<&XsdComplexType> {
        self.complex_types
            .iter()
            .find(|t| t.name.as_deref() == Some(name))
    }

    /// Find a top-level named simpleType by name.
    pub fn simple_type(&self, name: &str) -> Option<&XsdSimpleType> {
        self.simple_types
            .iter()
            .find(|t| t.name.as_deref() == Some(name))
    }

    /// Find a top-level named attributeGroup by name.
    pub fn attribute_group(&self, name: &str) -> Option<&XsdAttributeGroup> {
        self.attribute_groups.iter().find(|g| g.name == name)
    }

    /// Find a top-level named group by name.
    pub fn group(&self, name: &str) -> Option<&XsdGroup> {
        self.groups.iter().find(|g| g.name == name)
    }

    /// Every element declared as `substitutionGroup="<head>"` (i.e. every
    /// element that can appear where `<head>` would). Per W3C XSD 1.1
    /// Part 1 §3.3.2.4 — substitution-group membership is the
    /// substitutability relation between element decls.
    pub fn substitution_members_of(&self, head: &str) -> Vec<&XsdElement> {
        self.elements
            .iter()
            .filter(|e| e.substitution_group.as_deref() == Some(head))
            .collect()
    }

    /// Every element whose complex type has `mixed="true"` — those are
    /// the elements that can carry character data interleaved with child
    /// elements (the "transparent inline" elements in USLM terminology).
    /// Per W3C XSD 1.1 Part 1 §3.4.2.5.
    ///
    /// Resolves the type via the element's `type=` attribute pointing at
    /// a top-level complex type. Inline anonymous complex types are also
    /// considered.
    pub fn mixed_content_elements(&self) -> Vec<&XsdElement> {
        self.elements
            .iter()
            .filter(|e| {
                if let Some(inline) = &e.inline_complex_type {
                    return inline.mixed;
                }
                if let Some(type_name) = e.type_name.as_deref() {
                    let local = local_name(type_name);
                    if let Some(ct) = self.complex_type(local) {
                        return ct.mixed;
                    }
                }
                false
            })
            .collect()
    }
}

/// Strip a `prefix:` if present, returning the local part of a QName.
///
/// W3C Namespaces in XML 1.0 §3 — a QName has the form `prefix:local`;
/// when looking up types declared in the local schema, we only care
/// about the local part because the prefix resolves to the same
/// targetNamespace.
pub fn local_name(qname: &str) -> &str {
    match qname.find(':') {
        Some(i) => &qname[i + 1..],
        None => qname,
    }
}
