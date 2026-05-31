//! XSD ontology instance — data types projected from a loaded XSD
//! schema document.
//!
//! Defines [`XsdOntologyInstance`] and its sub-records
//! ([`ElementDeclarationInfo`], [`SchemaImportInfo`],
//! [`TypeDerivationInfo`], …) — the praxis-native runtime view of
//! every `<xs:schema>` document the lens layer reads. These types are
//! the *output* of two projection paths:
//!
//! - [`super::from_xml::project_from_xml_document`] — the canonical
//!   path: praxis-native XML parser → `XmlDocument` →
//!   [`XsdOntologyInstance`]. W3C XML 1.0 + W3C XSD 1.1 throughout.
//! - [`project_from_xsd_text`] — a deliberately minimal text scan that
//!   recovers element-declaration metadata directly from XSD source.
//!   Used where wiring the full XML parse path is overkill (e.g.
//!   inside lens leaf readers that already hold the XSD bytes).
//!
//! The runtime queries on the instance — [`XsdOntologyInstance::lookup_element`],
//! [`XsdOntologyInstance::is_member_of_substitution_group`], etc. —
//! drive every dispatch decision the USLM lens makes (W3C XSD 1.1
//! Part 1 §3.3 + §3.3.6 + §3.4), per
//! `feedback_bottom_up_loaded_not_encoded` — no hand-coded
//! element-name list lives in any consumer.
//!
//! The legacy file name (`from_xsd_parser`) reflects an earlier
//! design that depended on Sebastian Bergmann's external
//! `xsd-parser` crate. That dependency was removed; the data types
//! and the runtime-dispatch impl block remain. A rename is left as
//! cosmetic follow-up.
//!
//! ## Citations
//!
//! - **Gao, S., Sperberg-McQueen, C. M., & Thompson, H. S. (eds.) (2012)**
//!   *W3C XML Schema Definition Language (XSD) 1.1 Part 1: Structures*,
//!   W3C Recommendation 5 April 2012.
//!   <https://www.w3.org/TR/xmlschema11-1/>.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::XsdConcept;

/// Projected XSD ontology instance — produced from a loaded XSD
/// schema document by either [`super::from_xml::project_from_xml_document`]
/// (canonical, XML-parser-grounded) or [`project_from_xsd_text`]
/// (minimal text scan).
///
/// In addition to the projected `components` (the kind-only view), the
/// instance optionally carries a list of `(XsdConcept, local_name)`
/// pairs — the `named` field — populated when the loader saw a
/// `name="…"` attribute on the originating declaration. This field is
/// consulted by the runtime adjunction `XsdOntology ⊣ English`
/// ([`super::english_adjunction::lift_english_term_to_schema_components`])
/// when lifting English terms back into the schema component set
/// (M4.ε.5.a.4, generalising the M4.ε.5 Layer-3 `resolve_legal_role`
/// pattern from USC sections to any loaded XSD schema).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct XsdOntologyInstance {
    pub components: Vec<XsdConcept>,
    /// Per-component name carrier — `(concept, local_name)` pairs for
    /// every declaration the loader saw with a `name="…"` attribute.
    /// May be empty when the loader didn't track names (e.g. the
    /// kind-only projection). The instance-level adjunction Lift consults this list.
    pub named: Vec<NamedSchemaComponentEntry>,
    /// Per-`<xs:element>` declaration record — carries the local name,
    /// the `type="…"` reference (or `None` / `xs:anyType` default per
    /// W3C XSD 1.1 Part 1 §3.3.2.3), and the `substitutionGroup="…"`
    /// head (W3C XSD 1.1 Part 1 §3.3.6). Populated by
    /// [`project_from_xsd_text`]; empty when the loader only emitted a
    /// kind-only projection. Consulted by runtime walkers (e.g. the USLM
    /// `WellBehavedLens` at
    /// [`crate::social::software::markup::xml::uslm::lens`]) that
    /// dispatch on the loaded schema rather than on hand-coded
    /// element-name lists.
    pub elements: Vec<ElementDeclarationInfo>,
    /// Per-`<xs:import>` directive records — the schema-composition
    /// instances projected for the [`XsdConcept::SchemaImport`] concept
    /// (W3C XSD 1.1 Part 1 §4.2.6). Parallels the `elements` field for the
    /// [`XsdConcept::ElementDeclaration`] concept: the *concept* lives
    /// in the XSD ontology, this carries per-instance data.
    pub imports: Vec<SchemaImportInfo>,
    /// Per-`<xs:include>` directive records for the
    /// [`XsdConcept::SchemaInclude`] concept. W3C XSD 1.1 Part 1 §4.2.3.
    pub includes: Vec<SchemaIncludeInfo>,
    /// Per-`<xs:redefine>` directive records for the
    /// [`XsdConcept::SchemaRedefine`] concept. W3C XSD 1.1 Part 1 §4.2.4.
    pub redefines: Vec<SchemaRedefineInfo>,
    /// Per-`<xs:override>` directive records for the
    /// [`XsdConcept::SchemaOverride`] concept. W3C XSD 1.1 Part 1 §4.2.5.
    pub overrides: Vec<SchemaOverrideInfo>,
    /// Per-`<xs:annotation>` records for the
    /// [`XsdConcept::Annotation`] concept (W3C XSD 1.1 Part 1 §3.15),
    /// collected from anywhere in the schema tree.
    pub annotations: Vec<AnnotationInfo>,
    /// Per-`<xs:restriction>` / `<xs:extension>` records for the
    /// [`XsdConcept::Restriction`] / [`XsdConcept::Extension`]
    /// concepts — the type-derivation edges (method + base type)
    /// from W3C XSD 1.1 Part 1 §3.4.6.
    pub derivations: Vec<TypeDerivationInfo>,
}

/// Per-instance data for the [`XsdConcept::Restriction`] /
/// [`XsdConcept::Extension`] concepts. W3C XSD 1.1 Part 1 §3.4.6 —
/// a type derives from a base type by one of two methods. Captures
/// the `{derivation method}` and the `base="…"` reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDerivationInfo {
    /// The derivation method — exactly one of
    /// [`XsdConcept::Restriction`] or [`XsdConcept::Extension`].
    pub method: DerivationMethod,
    /// The `base="…"` reference — the type this construct derives
    /// from. `None` when absent (e.g. a `<xs:restriction>` with an
    /// inline anonymous `<xs:simpleType>` base per §3.16.6).
    pub base: Option<String>,
}

/// W3C XSD 1.1 Part 1 §3.4.6.4 `{derivation method}` — the two ways
/// one type definition derives from another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivationMethod {
    /// `<xs:restriction>` — the derived type's value space / content
    /// model is a subset of the base's.
    Restriction,
    /// `<xs:extension>` — the derived complex type appends content
    /// and/or attributes to the base.
    Extension,
}

/// Per-instance data for the [`XsdConcept::SchemaImport`] concept.
/// W3C XSD 1.1 Part 1 §4.2.6 `<xs:import>` — references a schema in
/// a *different* target namespace whose components this schema may
/// reference through namespace-qualified names. Parallels
/// [`ElementDeclarationInfo`]: the concept lives in the XSD
/// ontology, this struct carries the per-instance attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaImportInfo {
    /// `namespace="…"` — the imported schema's `targetNamespace`,
    /// or `None` when the import references a chameleon (no-
    /// target-namespace) schema.
    pub namespace: Option<String>,
    /// `schemaLocation="…"` — hint at where the imported schema
    /// document lives (not binding per §4.2.6.1).
    pub schema_location: Option<String>,
}

/// Per-instance data for the [`XsdConcept::SchemaInclude`] concept.
/// W3C XSD 1.1 Part 1 §4.2.3 `<xs:include>` — references another
/// schema document with the same `targetNamespace` whose components
/// are merged into the including schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaIncludeInfo {
    /// `schemaLocation="…"` — required attribute.
    pub schema_location: String,
}

/// Per-instance data for the [`XsdConcept::SchemaRedefine`] concept.
/// W3C XSD 1.1 Part 1 §4.2.4 `<xs:redefine>` — same-namespace
/// composition with derivation overrides. Deprecated in XSD 1.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaRedefineInfo {
    /// `schemaLocation="…"` — required attribute.
    pub schema_location: String,
}

/// Per-instance data for the [`XsdConcept::SchemaOverride`] concept.
/// W3C XSD 1.1 Part 1 §4.2.5 `<xs:override>` — XSD 1.1 same-
/// namespace composition with arbitrary overrides. Replaces
/// `<xs:redefine>` per §4.2.5.4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaOverrideInfo {
    /// `schemaLocation="…"` — required attribute.
    pub schema_location: String,
}

/// Per-instance data for the [`XsdConcept::Annotation`] concept
/// (W3C XSD 1.1 Part 1 §3.15). Captures the optional
/// `<xs:appinfo>` and `<xs:documentation>` blocks the annotation
/// carries. Either may be absent; an `<xs:annotation>` with no
/// children at all is still well-formed but informationally empty.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnnotationInfo {
    /// One entry per `<xs:appinfo>` child — the text content of each.
    /// Empty when no appinfo child is present.
    pub appinfo: Vec<String>,
    /// One entry per `<xs:documentation>` child — the text content of
    /// each. Empty when no documentation child is present.
    pub documentation: Vec<String>,
}

/// `(XsdConcept, local_name)` carrier for the instance-level
/// adjunction Lift. Populated when the loader saw a `name="…"`
/// attribute on a declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedSchemaComponentEntry {
    pub concept: XsdConcept,
    pub local_name: String,
}

/// Per-`<xs:element>` declaration record. Carries the local name,
/// the `type="…"` reference (or `None` / `xs:anyType` default per
/// W3C XSD 1.1 Part 1 §3.3.2.3), and the `substitutionGroup="…"`
/// head (W3C XSD 1.1 Part 1 §3.3.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementDeclarationInfo {
    /// The element's local name (`<xs:element name="…">`).
    pub local_name: String,
    /// The element's `type="…"` reference, or `None` if the
    /// declaration carried no `type=` attribute (W3C XSD 1.1 Part 1
    /// §3.3.2.3 default: `xs:anyType`).
    pub type_ref: Option<String>,
    /// The element's `substitutionGroup="…"` head, or `None` if the
    /// declaration is a substitution-group head itself or
    /// substitution-group-unaffiliated (W3C XSD 1.1 Part 1 §3.3.6).
    pub substitution_group_head: Option<String>,
}

impl XsdOntologyInstance {
    /// Iterate the projected schema components.
    pub fn schema_components(&self) -> impl Iterator<Item = &XsdConcept> {
        self.components.iter()
    }

    /// Count of distinct concept kinds present in the projection.
    pub fn distinct_concept_count(&self) -> usize {
        let mut seen = alloc::collections::BTreeSet::new();
        for c in &self.components {
            // Concepts derive PartialEq+Eq+Hash but not Ord; fall back
            // to debug string for stable ordering in a BTreeSet.
            seen.insert(format!("{c:?}"));
        }
        seen.len()
    }

    /// Iterate the `(XsdConcept, local_name)` pairs the loader
    /// carried into the instance. Empty when the projection was
    /// built without per-component names.
    pub fn named_components(&self) -> impl Iterator<Item = &NamedSchemaComponentEntry> {
        self.named.iter()
    }

    /// Look up an element declaration by its local name. Returns the
    /// per-element record (type reference + substitution-group head)
    /// projected from the loaded XSD. `None` when no `<xs:element>`
    /// declaration with that name was loaded.
    ///
    /// W3C XSD 1.1 Part 1 §3.3 (Element Declarations) — the
    /// authoritative source for what counts as an element-name lookup.
    pub fn lookup_element(&self, local_name: &str) -> Option<&ElementDeclarationInfo> {
        self.elements.iter().find(|e| e.local_name == local_name)
    }

    /// The type reference of `<xs:element local_name>`. Wraps
    /// [`lookup_element`](Self::lookup_element)'s `type_ref` field.
    ///
    /// `None` if no element with that local name is loaded; `Some(None)`
    /// is collapsed to the W3C XSD 1.1 Part 1 §3.3.2.3 default
    /// `xs:anyType` only by the caller — this query returns the loaded
    /// attribute verbatim.
    pub fn type_definition_of(&self, local_name: &str) -> Option<&str> {
        self.lookup_element(local_name)
            .and_then(|e| e.type_ref.as_deref())
    }

    /// Head of the substitution group that `<xs:element local_name>`
    /// declares membership in (W3C XSD 1.1 Part 1 §3.3.6). Returns
    /// `None` if `local_name` isn't loaded, or is loaded but carries
    /// no `substitutionGroup="…"` attribute (i.e. it's a substitution-
    /// group head or substitution-group-unaffiliated).
    pub fn substitution_group_of(&self, local_name: &str) -> Option<&str> {
        self.lookup_element(local_name)
            .and_then(|e| e.substitution_group_head.as_deref())
    }

    /// Enumerate every `<xs:element>` local name declared by the
    /// loaded XSD. Order matches the source's declaration order;
    /// duplicates are not de-duplicated (an XSD that re-declares
    /// the same name violates §3.3 *unique particle attribution*
    /// and is itself malformed — the loader surfaces that).
    pub fn declared_element_names(&self) -> Vec<&str> {
        self.elements
            .iter()
            .map(|e| e.local_name.as_str())
            .collect()
    }

    /// Enumerate every `<xs:element>` local name that is a
    /// (reflexive-transitive) member of `head`'s substitution group
    /// per W3C XSD 1.1 Part 1 §3.3.6. Reflexive: `head` is always
    /// in the result. The output is the **full known set** the
    /// loaded XSD declares — callers MUST NOT hand-curate this list
    /// in Rust source (`feedback_bottom_up_loaded_not_encoded`).
    ///
    /// Result order is the declaration order in the loaded XSD,
    /// with `head` placed first by convention. Duplicates are
    /// suppressed.
    ///
    /// Citation: W3C XML Schema Definition Language (XSD) 1.1 Part 1
    /// §3.3.6 *Substitution Group OK (Transitive)*.
    pub fn substitution_group_members<'a>(&'a self, head: &'a str) -> Vec<&'a str> {
        let mut out: Vec<&'a str> = vec![head];
        // BFS over the loaded element declarations, marking every
        // element whose head chain terminates at `head`.
        let mut changed = true;
        while changed {
            changed = false;
            for e in &self.elements {
                if out.contains(&e.local_name.as_str()) {
                    continue;
                }
                let Some(parent) = &e.substitution_group_head else {
                    continue;
                };
                if out.contains(&parent.as_str()) {
                    out.push(e.local_name.as_str());
                    changed = true;
                }
            }
        }
        out
    }

    /// True iff `member` is a (reflexive-transitive) member of `head`'s
    /// substitution group per W3C XSD 1.1 Part 1 §3.3.6. Reflexive:
    /// `is_member_of_substitution_group(h, h)` is always `true`.
    ///
    /// Builds a [`super::ontology::SubstitutionGroupHierarchy`] from
    /// the loaded `<xs:element>` declarations and asks it the
    /// membership question — no hand-coded element-name list.
    pub fn is_member_of_substitution_group(&self, member: &str, head: &str) -> bool {
        if member == head {
            return true;
        }
        // Build the (member → head) pair list as static-str refs into
        // owned `String`s. The hierarchy struct takes `&'static str`s
        // (literature-cited fixed inputs in the axiom test), so for the
        // runtime query we replicate the walk here without going
        // through `SubstitutionGroupHierarchy::member_of`'s static-str
        // signature. The walk is identical: BFS over the loaded
        // (member, head) edges, terminating when `head` is reached.
        let mut frontier = alloc::collections::BTreeSet::new();
        frontier.insert(member.to_string());
        let mut changed = true;
        while changed {
            changed = false;
            let snapshot: Vec<String> = frontier.iter().cloned().collect();
            for m in &snapshot {
                for e in &self.elements {
                    if e.local_name == *m
                        && let Some(h) = &e.substitution_group_head
                    {
                        if h == head {
                            return true;
                        }
                        if frontier.insert(h.clone()) {
                            changed = true;
                        }
                    }
                }
            }
        }
        false
    }
}

/// Project from a literal XSD source text into an [`XsdOntologyInstance`]
/// populated with per-element declarations.
///
/// Scans the source for every `<xsd:element>` / `<xs:element>`
/// declaration that carries a `name="…"` attribute, capturing the
/// optional `type="…"` reference (W3C XSD 1.1 Part 1 §3.3.2.3) and
/// the optional `substitutionGroup="…"` head (§3.3.6). The same
/// pass also produces the kind-only `components` projection and the
/// `(concept, local_name)` `named` entries that the
/// English-adjunction surface expects.
///
/// This is a deliberately minimal text scan — the goal is the
/// runtime-walker dispatch needed by the USLM lens. Full XSD
/// validity (acyclicity of derivation, every `substitutionGroup`
/// pointing to an existing head) is enforced upstream by the
/// canonical [`super::from_xml::project_from_xml_document`] +
/// `axioms.rs` pipeline that operates on the parsed `XmlDocument`.
pub fn project_from_xsd_text(xsd_src: &str) -> XsdOntologyInstance {
    let mut elements: Vec<ElementDeclarationInfo> = Vec::new();
    let mut named: Vec<NamedSchemaComponentEntry> = Vec::new();
    let mut components: Vec<XsdConcept> = Vec::new();

    // The six XSD declaration kinds we want to project. We accept
    // both `xsd:` and `xs:` namespace prefixes (USLM uses `xsd:`;
    // others use `xs:`).
    for (tag_prefix, concept) in [
        ("element", XsdConcept::ElementDeclaration),
        ("complexType", XsdConcept::ComplexTypeDefinition),
        ("simpleType", XsdConcept::SimpleTypeDefinition),
        ("attributeGroup", XsdConcept::AttributeGroup),
        ("group", XsdConcept::ModelGroup),
        ("attribute", XsdConcept::AttributeDeclaration),
    ] {
        for full_prefix in [format!("<xsd:{tag_prefix} "), format!("<xs:{tag_prefix} ")] {
            let mut cursor = 0;
            while let Some(rel) = xsd_src[cursor..].find(full_prefix.as_str()) {
                let abs = cursor + rel + full_prefix.len();
                let end = xsd_src[abs..]
                    .find('>')
                    .map(|p| abs + p)
                    .unwrap_or(xsd_src.len());
                let attr_slice = &xsd_src[abs..end];
                if let Some(name) = extract_attr(attr_slice, "name") {
                    named.push(NamedSchemaComponentEntry {
                        concept,
                        local_name: name.clone(),
                    });
                    components.push(concept);
                    // Only `<xs:element>` declarations carry the
                    // dispatch-relevant `type` + `substitutionGroup`
                    // attributes. Other kinds (`<xs:complexType>` etc.)
                    // get the kind-only projection.
                    if concept == XsdConcept::ElementDeclaration {
                        let type_ref = extract_attr(attr_slice, "type");
                        let sg = extract_attr(attr_slice, "substitutionGroup");
                        elements.push(ElementDeclarationInfo {
                            local_name: name,
                            type_ref,
                            substitution_group_head: sg,
                        });
                    }
                }
                cursor = end + 1;
            }
        }
    }

    XsdOntologyInstance {
        components,
        named,
        elements,
        imports: Vec::new(),
        includes: Vec::new(),
        redefines: Vec::new(),
        overrides: Vec::new(),
        annotations: Vec::new(),
        derivations: Vec::new(),
    }
}

/// Extract `<key>="value"` from an attribute slice (no full XML
/// parsing; works on well-formed XSD attribute syntax).
fn extract_attr(slice: &str, key: &str) -> Option<String> {
    let pattern = format!("{key}=\"");
    let start = slice.find(&pattern)? + pattern.len();
    let end = slice[start..].find('"')? + start;
    Some(slice[start..end].to_string())
}
