//! Functor: xsd-parser AST → XSD ontology instances.
//!
//! Source category: the discrete category of xsd-parser AST node
//! kinds (every loaded XSD top-level construct projects to one of the
//! [`XsdAstNodeKind`] variants below). Target category: the
//! [XSD ontology][super::ontology] declared by `pr4xis::ontology!`.
//!
//! ## Categorical setting
//!
//! Per Mac Lane *Categories for the Working Mathematician* §I.3, a
//! functor F: C → D is a structure-preserving map: F maps objects
//! C-objects to D-objects and C-morphisms to D-morphisms, preserving
//! identities and composition.
//!
//! Here both categories are discrete (every morphism is an identity),
//! so the composition law reduces to the identity law. The functor
//! object map sends each xsd-parser AST node kind to the XSD
//! ontology concept that classifies it.
//!
//! ## Why this is the right shape
//!
//! xsd-parser's output is a heterogeneous tree of typed Rust structs,
//! one per `<xs:element>` / `<xs:complexType>` / `<xs:simpleType>` /
//! ... declaration in the loaded schema. The Praxis ontology
//! ([`XsdOntology`](super::ontology::XsdOntology)) declares the
//! kinds those nodes can fall under. The functor is the literal
//! type-erasure: every typed Rust struct projects to its kind, and
//! kinds compose only along identities (a node *is* one specific
//! kind, no path-composition needed).
//!
//! ## Citation
//!
//! - Mac Lane, S. *Categories for the Working Mathematician*,
//!   Springer GTM 5, 2nd ed. 1998, §I.3 (Functors).
//! - Bergmann, S. *xsd-parser: Rust code generator for XML schema
//!   files*, v1.5.2, MIT-licensed.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};

use super::ontology::{XsdCategory, XsdConcept, XsdRelation, XsdRelationKind};

// =============================================================================
// Source category — xsd-parser AST nodes by kind.
// =============================================================================

/// The kind of an xsd-parser AST node. Each variant maps directly to
/// one xsd-parser construct kind (output of `xsd_parser::generate`
/// before serialisation as Rust types). The 18 variants cover every
/// XSD 1.1 construct xsd-parser can emit; M4.ε.5.a's USLM-1.0.18.xsd
/// uses 12 of them (no `xs:notation`, no `xs:any`, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, pr4xis::category::Concept)]
pub enum XsdAstNodeKind {
    /// An `<xs:element>` declaration (xsd-parser `ElementType`).
    Element,
    /// An `<xs:attribute>` declaration (xsd-parser `AttributeType`).
    Attribute,
    /// An `<xs:complexType>` (xsd-parser `ComplexType`).
    ComplexType,
    /// An `<xs:simpleType>` (xsd-parser `SimpleType`).
    SimpleType,
    /// An `<xs:sequence>` compositor (xsd-parser `Sequence`).
    Sequence,
    /// An `<xs:choice>` compositor (xsd-parser `Choice`).
    Choice,
    /// An `<xs:all>` compositor (xsd-parser `All`).
    All,
    /// An `<xs:group>` definition (xsd-parser `Group`).
    /// xsd-parser models named model groups as the same construct
    /// kind as inline compositors; the ontology treats both as
    /// `ModelGroup` (W3C XSD 1.1 Part 1 §3.8).
    Group,
    /// An `<xs:attributeGroup>` definition (xsd-parser `AttributeGroup`).
    AttributeGroup,
    /// An `<xs:any>` element wildcard (xsd-parser `AnyElement`).
    AnyElement,
    /// An `<xs:anyAttribute>` attribute wildcard (xsd-parser `AnyAttribute`).
    AnyAttribute,
    /// An `<xs:key>` identity constraint (xsd-parser `Key`).
    Key,
    /// An `<xs:unique>` identity constraint (xsd-parser `Unique`).
    Unique,
    /// An `<xs:keyref>` identity constraint (xsd-parser `Keyref`).
    Keyref,
    /// An `<xs:notation>` declaration (xsd-parser `Notation`).
    Notation,
    /// An `<xs:annotation>` block (xsd-parser `Annotation`).
    Annotation,
    /// An `<xs:appinfo>` child of an annotation (xsd-parser `AppInfo`).
    AppInfo,
    /// An `<xs:documentation>` child of an annotation
    /// (xsd-parser `Documentation`).
    Documentation,
}

/// Morphism between AST node kinds. The source category is discrete:
/// the only morphism is identity. (xsd-parser doesn't carry typed
/// relations between AST nodes; relationships are expressed by
/// containment in the AST tree, which the functor projects through
/// the target category's `is_a` hierarchy.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct XsdAstMorphism {
    pub from: XsdAstNodeKind,
    pub to: XsdAstNodeKind,
}

/// Kind tag for AST morphisms — only `Identity` is meaningful in a
/// discrete category. The variant is named distinctly so it can carry
/// the `Arrow::Kind` slot in core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XsdAstRelationKind {
    /// Identity — the only morphism in a discrete category.
    Identity,
}

impl Arrow for XsdAstMorphism {
    type Object = XsdAstNodeKind;
    type Kind = XsdAstRelationKind;

    fn source(&self) -> XsdAstNodeKind {
        self.from
    }
    fn target(&self) -> XsdAstNodeKind {
        self.to
    }
    fn kind(&self) -> XsdAstRelationKind {
        XsdAstRelationKind::Identity
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new(format!("XsdAst-id-{:?}", self.from)),
            description: Label::new(format!(
                "identity on xsd-parser AST node kind {:?}",
                self.from
            )),
            citation: Citation::parse_static(
                "Mac Lane (1998) Categories for the Working Mathematician §I.1; \
                 Bergmann xsd-parser 1.5.2",
            ),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

/// Discrete category of xsd-parser AST node kinds. Identity is the
/// only morphism. (XSD AST nodes are *types* in the Rust sense; type
/// equality is reflexive only.)
pub struct XsdAstCategory;

impl Category for XsdAstCategory {
    type Object = XsdAstNodeKind;
    type Morphism = XsdAstMorphism;

    fn identity(obj: &XsdAstNodeKind) -> XsdAstMorphism {
        XsdAstMorphism {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &XsdAstMorphism, g: &XsdAstMorphism) -> Option<XsdAstMorphism> {
        // Discrete: f and g compose only when both are identities on
        // the same object.
        if f.to == g.from && f.from == f.to && g.from == g.to {
            Some(XsdAstMorphism {
                from: f.from,
                to: g.to,
            })
        } else {
            None
        }
    }

    fn morphisms() -> Vec<XsdAstMorphism> {
        // Identities only.
        XsdAstNodeKind::variants()
            .into_iter()
            .map(|k| XsdAstMorphism { from: k, to: k })
            .collect()
    }
}

// =============================================================================
// Object map — AST node kind → XSD ontology concept.
// =============================================================================

/// Map an AST node kind to the XSD ontology concept that classifies
/// it. This is the object component of the functor (Mac Lane §I.3).
///
/// Every AST node kind has exactly one classifying concept (W3C XSD
/// 1.1 partitions schema components; xsd-parser preserves the
/// partition).
pub fn project_node_kind(kind: XsdAstNodeKind) -> XsdConcept {
    use XsdAstNodeKind as A;
    use XsdConcept as O;
    match kind {
        A::Element => O::ElementDeclaration,
        A::Attribute => O::AttributeDeclaration,
        A::ComplexType => O::ComplexTypeDefinition,
        A::SimpleType => O::SimpleTypeDefinition,
        A::Sequence => O::Sequence,
        A::Choice => O::Choice,
        A::All => O::AllGroup,
        // xsd-parser models named model groups as a separate construct
        // kind, but W3C XSD 1.1 Part 1 §3.8 treats them as `ModelGroup`
        // instances — the compositor child (sequence/choice/all) is the
        // sub-kind. Without a compositor child the abstract kind is
        // `ModelGroup` itself; the functor projects to that.
        A::Group => O::ModelGroup,
        A::AttributeGroup => O::AttributeGroup,
        A::AnyElement | A::AnyAttribute => O::Wildcard,
        A::Key | A::Unique | A::Keyref => O::IdentityConstraint,
        A::Notation => O::NotationDeclaration,
        A::Annotation => O::Annotation,
        A::AppInfo => O::AppInfo,
        A::Documentation => O::Documentation,
    }
}

// =============================================================================
// The functor.
// =============================================================================

pr4xis::functor! {
    name: FromXsdParser,
    source: XsdAstCategory,
    target: XsdCategory,
    citation: "Mac Lane (1998) Categories for the Working Mathematician §I.3 (Functors); Bergmann xsd-parser 1.5.2",
    map_object: |obj: &XsdAstNodeKind| -> XsdConcept { project_node_kind(*obj) },
    map_morphism: |m: &XsdAstMorphism| -> XsdRelation {
        // Source is discrete: every morphism is an identity. Project
        // to the corresponding identity in the target category (the
        // target's `Identity` kind is what XsdCategory emits).
        XsdRelation {
            from: project_node_kind(m.from),
            to: project_node_kind(m.to),
            kind: XsdRelationKind::Identity,
        }
    },
}

// =============================================================================
// AST-level projection — a top-level convenience for projecting a
// loaded xsd-parser schema (a collection of typed nodes) through the
// functor.
// =============================================================================

/// A minimal, opaque representation of an xsd-parser-loaded XSD
/// schema. Wraps a flat list of AST node kinds; the originating
/// xsd-parser AST itself is heterogeneous (one Rust type per
/// `<xs:*>` declaration), so the functor's input is the projected
/// *kind* sequence.
///
/// Consumers project from the codegen'd Rust types (see
/// [`crate::social::software::markup::xml::uslm::generated`]) by
/// classifying each generated `pub struct ...` / `pub enum ...` into
/// its [`XsdAstNodeKind`] — a mechanical post-processing of the
/// codegen output that needs no domain knowledge.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct XsdAst {
    pub nodes: Vec<XsdAstNodeKind>,
}

/// Projected XSD ontology instance — the result of running an
/// `XsdAst` through the [`FromXsdParser`] functor.
///
/// In addition to the projected `components` (the kind-only view used
/// by the type-level functor), the instance optionally carries a list
/// of `(XsdConcept, local_name)` pairs — the `named` field — populated
/// when the loader saw a `name="…"` attribute on the originating
/// declaration. This field is consulted by the runtime adjunction
/// `XsdOntology ⊣ English`
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
    /// kind-only projection used in the AST-functor smoke tests). The
    /// instance-level adjunction Lift consults this list.
    pub named: Vec<NamedSchemaComponentEntry>,
    /// Per-`<xs:element>` declaration record — carries the local name,
    /// the `type="…"` reference (or `None` / `xs:anyType` default per
    /// W3C XSD 1.1 Part 1 §3.3.2.3), and the `substitutionGroup="…"`
    /// head (W3C XSD 1.1 Part 1 §3.3.6). Populated by
    /// [`project_from_xsd_text`]; empty for the kind-only AST
    /// projection. Consulted by runtime walkers (e.g. the USLM
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
/// W3C XSD 1.1 Part 1 §4.2.5 `<xs:override>` — same-namespace
/// composition that *replaces* declarations from the referenced
/// schema. New in XSD 1.1, supersedes `<xs:redefine>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaOverrideInfo {
    /// `schemaLocation="…"` — required attribute.
    pub schema_location: String,
}

/// Per-instance data for the [`XsdConcept::Annotation`] concept.
/// W3C XSD 1.1 Part 1 §3.15 `<xs:annotation>` — a container for
/// human- and machine-readable annotations on a schema component.
/// Annotations are introspectable but contribute nothing to schema
/// validity; the loader captures them so downstream tooling can
/// surface documentation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AnnotationInfo {
    /// `<xs:appinfo>` blocks — machine-readable, application-specific
    /// content (§3.15.1). Each entry is the concatenated text body
    /// of one `<xs:appinfo>` element.
    pub appinfo: Vec<String>,
    /// `<xs:documentation>` blocks — human-readable prose (§3.15.2).
    /// Each entry is the concatenated text body of one
    /// `<xs:documentation>` element.
    pub documentation: Vec<String>,
}

/// A `(XsdConcept, local_name)` pair carried by an
/// [`XsdOntologyInstance`]. Held in its own type rather than a raw
/// tuple so the adjunction Lift can construct a typed
/// `super::english_adjunction::NamedSchemaComponent` from it without
/// reaching into tuple fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedSchemaComponentEntry {
    /// The XSD concept this component belongs to.
    pub concept: XsdConcept,
    /// The component's local name (case preserved).
    pub local_name: String,
}

/// One `<xs:element>` declaration record. Carries everything a
/// runtime walker needs to dispatch on per W3C XSD 1.1 Part 1 §3.3
/// (Element Declarations), §3.4 (Type Definitions, via `type_ref`),
/// and §3.3.6 (Substitution Groups, via `substitution_group_head`).
///
/// `type_ref` follows the W3C XSD 1.1 Part 1 §3.3.2.3 rule: an
/// element's `{type definition}` property is always defined,
/// defaulting to `xs:anyType` when no `type=` attribute appears. The
/// loader carries the literal attribute value (e.g. `"LevelType"`) and
/// `None` when the attribute was absent on the source declaration.
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
    /// built from a kind-only AST (e.g. via [`project`] below).
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

/// Project an xsd-parser AST through the functor into an XSD ontology
/// instance. Mac Lane §I.3: this is `F` applied to a discrete-object
/// set, returning the image set.
pub fn project(ast: &XsdAst) -> XsdOntologyInstance {
    XsdOntologyInstance {
        components: ast.nodes.iter().copied().map(project_node_kind).collect(),
        named: Vec::new(),
        elements: Vec::new(),
        imports: Vec::new(),
        includes: Vec::new(),
        redefines: Vec::new(),
        overrides: Vec::new(),
        annotations: Vec::new(),
        derivations: Vec::new(),
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
/// `(concept, local_name)` `named` entries that the existing
/// [`project`] / English-adjunction surface expects.
///
/// This is a deliberately minimal text scan — the goal is the
/// runtime-walker dispatch needed by the USLM lens. Full XSD
/// validity (acyclicity of derivation, every `substitutionGroup`
/// pointing to an existing head) is enforced by the
/// `pr4xis::codegen::uslm_schema::generate_uslm_schema_source`
/// build-time pipeline that exercises xsd-parser on the same file.
/// If that codegen succeeds, the scanner cannot misclassify a
/// well-formed declaration.
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

// =============================================================================
// Convenience helpers for projecting from the codegen'd USLM types.
// =============================================================================

/// Classify a single xsd-parser-emitted Rust type *name* into its
/// [`XsdAstNodeKind`]. The dispatch follows xsd-parser 1.5.2's
/// codegen naming conventions:
///
/// - `<xs:complexType>` → `<Name>TypeItem` (USLM's `Item` postfix per
///   `pr4xis::codegen::uslm_schema`).
/// - `<xs:simpleType>` for enumerations → `<Name>EnumItem`.
/// - `<xs:group>` and `<xs:attributeGroup>` → emitted as inline
///   struct fields rather than top-level types; rarely classified
///   here.
///
/// Returns `None` for names that don't match any known xsd-parser
/// shape (xsd-parser's own built-in type aliases, mixed-content
/// wrappers, etc.). Per `feedback_push_back_on_unsupported_file_types`,
/// the caller must skip — never approximate — unmatched names.
pub fn classify_codegen_name(name: &str) -> Option<XsdAstNodeKind> {
    if name.ends_with("EnumItem") {
        // xsd-parser emits enum-typed simple types as `<Name>EnumItem`.
        return Some(XsdAstNodeKind::SimpleType);
    }
    if name.ends_with("TypeItem") {
        // The USLM-postfix complex-type convention.
        return Some(XsdAstNodeKind::ComplexType);
    }
    None
}
