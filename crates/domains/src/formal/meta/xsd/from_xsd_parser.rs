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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct XsdOntologyInstance {
    pub components: Vec<XsdConcept>,
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
}

/// Project an xsd-parser AST through the functor into an XSD ontology
/// instance. Mac Lane §I.3: this is `F` applied to a discrete-object
/// set, returning the image set.
pub fn project(ast: &XsdAst) -> XsdOntologyInstance {
    XsdOntologyInstance {
        components: ast.nodes.iter().copied().map(project_node_kind).collect(),
    }
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
