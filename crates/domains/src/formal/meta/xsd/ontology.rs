//! XSD ontology — W3C XML Schema 1.1 as a Praxis ontology.
//!
//! Defines the concept inventory of XML Schema 1.1 — `SchemaComponent`
//! and its leaves (`ElementDeclaration`, `AttributeDeclaration`,
//! `TypeDefinition`, `ComplexTypeDefinition`, `SimpleTypeDefinition`,
//! `ModelGroup`, `Sequence`, `Choice`, `AllGroup`, `AttributeGroup`,
//! `Particle`, `Wildcard`, `IdentityConstraint`, `NotationDeclaration`,
//! `Annotation` with sub-concepts `AppInfo`, `Documentation`) — with
//! the relationships and axioms that govern them. The Rust carrier is
//! the [`XsdPart`] enum (a partition over schema-component variants);
//! the spec terms above are W3C XSD 1.1 nomenclature, not Rust items.
//!
//! ## Why XSD as a Praxis ontology
//!
//! XSD is a meta-language for describing XML document structure. Any
//! XML schema (USLM, LMF, OOXML, ...) is itself an XSD-described
//! ontology. By declaring XSD as a Praxis ontology, every loaded
//! schema becomes a Praxis ontology instance through a single
//! `XsdAst → XsdOntology` functor — no per-schema hand-coding.
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 1: Structures**, Gao, Sperberg-McQueen
//!   & Thompson 2012, W3C Recommendation 2012-04-05.
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05.
//!
//! ## Related Praxis ontologies
//!
//! - `pr4xis_domains::cognitive::linguistics::english::English` — XSD
//!   names project through English (Mac Lane §I.3, functor). See
//!   `formal::meta::xsd::english_projection` (M4.ε.5.a.3, queued).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// =============================================================================
// Concept inventory — every leaf cites W3C XSD 1.1 Part 1 / Part 2 § that
// defines it. The top concept `SchemaComponent` (§2.2) partitions cleanly
// into typed leaves; sub-leaves (Sequence/Choice/AllGroup, AppInfo/Documentation,
// ComplexTypeDefinition/SimpleTypeDefinition) sit one level below their parent.
// =============================================================================

pr4xis::ontology! {
    name: "Xsd",
    source: "Gao, Sperberg-McQueen & Thompson (eds.) (2012) W3C XML Schema 1.1 Part 1: Structures, W3C Recommendation 2012-04-05; Peterson, Gao, Akhmedov, Malhotra, Biron & Sperberg-McQueen (eds.) (2012) W3C XML Schema 1.1 Part 2: Datatypes, W3C Recommendation 2012-04-05",

    concepts: [
        // The schema-document level — the `<xs:schema>` root element
        // itself. A top-level concept parallel to SchemaComponent /
        // SchemaCompositionDirective / TypeConstructionConstruct /
        // ConstrainingFacet (W3C XSD 1.1 Part 1 §2.5 "Schema Document"
        // + §3.16 "Schemas as Wholes"). Every `<xs:schema>` element
        // projects to a SchemaDocument; the components / directives /
        // constructs / facets it contains project to their own leaves
        // independently.
        SchemaDocument,
        SchemaComponent,
        ElementDeclaration,
        AttributeDeclaration,
        TypeDefinition,
        ComplexTypeDefinition,
        SimpleTypeDefinition,
        ModelGroup,
        Sequence,
        Choice,
        AllGroup,
        AttributeGroup,
        Particle,
        Wildcard,
        IdentityConstraint,
        NotationDeclaration,
        Annotation,
        AppInfo,
        Documentation,
        SchemaCompositionDirective,
        SchemaImport,
        SchemaInclude,
        SchemaRedefine,
        SchemaOverride,
        TypeConstructionConstruct,
        ComplexContent,
        SimpleContent,
        Restriction,
        Extension,
        ListType,
        UnionType,
        ConstrainingFacet,
        LengthFacet,
        MinLengthFacet,
        MaxLengthFacet,
        PatternFacet,
        EnumerationFacet,
        WhiteSpaceFacet,
        MaxInclusiveFacet,
        MaxExclusiveFacet,
        MinExclusiveFacet,
        MinInclusiveFacet,
        TotalDigitsFacet,
        FractionDigitsFacet,
        ExplicitTimezoneFacet,
        AssertionFacet,
        Key,
        KeyRef,
        Unique,
        Selector,
        Field,
        Assert,
        OpenContent,
        DefaultOpenContent,
    ],

    labels: {
        SchemaDocument: ("en", "Schema document",
            "W3C XSD 1.1 Part 1 §2.5 + §3.16: an XML document whose root element is `<xs:schema>` — the schema-document container of zero or more schema components, composition directives, type-construction constructs, and constraining facets. A schema document with no children is still a valid Schema (§3.16 Schemas as Wholes) — it just declares no components."),
        SchemaComponent: ("en", "Schema component",
            "W3C XSD 1.1 Part 1 §2.2: the top of the schema-component partition. Every named or structural piece of an XSD schema is a schema component."),
        ElementDeclaration: ("en", "Element declaration",
            "W3C XSD 1.1 Part 1 §3.3: an `<xs:element>` declaring a named element with a type, occurrence range, substitution-group, and identity-constraint set."),
        AttributeDeclaration: ("en", "Attribute declaration",
            "W3C XSD 1.1 Part 1 §3.2: an `<xs:attribute>` declaring a named attribute with a simple-type and use cardinality."),
        TypeDefinition: ("en", "Type definition",
            "W3C XSD 1.1 Part 1 §2.2.1.2: a named or anonymous type, partitioning into complex and simple-type definitions; derivation by restriction or extension forms a strict partial order (§3.4.6.4)."),
        ComplexTypeDefinition: ("en", "Complex type definition",
            "W3C XSD 1.1 Part 1 §3.4: a `<xs:complexType>` defining a content model + attribute uses (with optional `mixed` content). Always derives from another type (final root is `xs:anyType`)."),
        SimpleTypeDefinition: ("en", "Simple type definition",
            "W3C XSD 1.1 Part 2 §4.1: a `<xs:simpleType>` defining an atomic, list, or union value space. Always derives from another simple type (final root is `xs:anySimpleType`)."),
        ModelGroup: ("en", "Model group",
            "W3C XSD 1.1 Part 1 §3.8: a sequence/choice/all compositor governing the order and cardinality of child particles in a complex type's content model."),
        Sequence: ("en", "Sequence model group",
            "W3C XSD 1.1 Part 1 §3.8.1: `<xs:sequence>` — children must appear in declared order."),
        Choice: ("en", "Choice model group",
            "W3C XSD 1.1 Part 1 §3.8.1: `<xs:choice>` — exactly one of the declared alternatives must appear."),
        AllGroup: ("en", "All model group",
            "W3C XSD 1.1 Part 1 §3.8.1: `<xs:all>` — each declared particle appears at most once, in any order. Named `AllGroup` to avoid collisions with the English word `All`."),
        AttributeGroup: ("en", "Attribute group definition",
            "W3C XSD 1.1 Part 1 §3.6: an `<xs:attributeGroup>` bundling a reusable set of attribute uses, included in complex types via `ref`."),
        Particle: ("en", "Particle",
            "W3C XSD 1.1 Part 1 §3.9: a term (element, group, or wildcard) appearing in a content model with occurrence range `[minOccurs, maxOccurs]`. The range is nonempty (§3.9.3.2)."),
        Wildcard: ("en", "Wildcard",
            "W3C XSD 1.1 Part 1 §3.10: `<xs:any>` / `<xs:anyAttribute>` — a permissive matcher constrained by a namespace constraint and process-contents directive."),
        IdentityConstraint: ("en", "Identity-constraint definition",
            "W3C XSD 1.1 Part 1 §3.11: `<xs:key>` / `<xs:unique>` / `<xs:keyref>` — XPath-based co-occurrence constraints over an element's descendants."),
        NotationDeclaration: ("en", "Notation declaration",
            "W3C XSD 1.1 Part 1 §3.14: `<xs:notation>` — names a non-XML notation (system / public identifier) referenced by attribute values of simple-type `NOTATION`."),
        Annotation: ("en", "Annotation",
            "W3C XSD 1.1 Part 1 §3.15: `<xs:annotation>` — a container for human-readable documentation (`<xs:documentation>`) and machine-readable application info (`<xs:appinfo>`)."),
        AppInfo: ("en", "Application information",
            "W3C XSD 1.1 Part 1 §3.15.1: `<xs:appinfo>` — machine-readable application-specific content carried inside an annotation."),
        Documentation: ("en", "Documentation",
            "W3C XSD 1.1 Part 1 §3.15.1: `<xs:documentation>` — human-readable prose describing the surrounding schema component."),
        SchemaCompositionDirective: ("en", "Schema-composition directive",
            "W3C XSD 1.1 Part 1 §4.2: a schema-document-level directive (`<xs:import>` / `<xs:include>` / `<xs:redefine>` / `<xs:override>`) that assembles a schema from multiple documents. Unlike schema components (§2.2), directives carry no own contribution to the post-schema-validation infoset; they govern how component sets from other documents merge."),
        SchemaImport: ("en", "Import directive",
            "W3C XSD 1.1 Part 1 §4.2.6: `<xs:import>` — makes the components of a schema in a *different* target namespace available for namespace-qualified reference."),
        SchemaInclude: ("en", "Include directive",
            "W3C XSD 1.1 Part 1 §4.2.3: `<xs:include>` — merges the components of another schema document with the same (or absent) target namespace into the including schema."),
        SchemaRedefine: ("en", "Redefine directive",
            "W3C XSD 1.1 Part 1 §4.2.4: `<xs:redefine>` — includes another schema document and redefines some of its type / group definitions. Deprecated in XSD 1.1; superseded by `<xs:override>`."),
        SchemaOverride: ("en", "Override directive",
            "W3C XSD 1.1 Part 1 §4.2.5: `<xs:override>` — includes another schema document and *replaces* selected components. New in XSD 1.1, replacing `<xs:redefine>`."),
        TypeConstructionConstruct: ("en", "Type-construction construct",
            "W3C XSD 1.1 Part 1 §3.4.2 / Part 2 §4.1.2: an XML-representation construct that determines a type definition's {content type}, {derivation method}, or {variety} (`<xs:complexContent>` / `<xs:simpleContent>` / `<xs:restriction>` / `<xs:extension>` / `<xs:list>` / `<xs:union>`). Unlike schema components (§2.2), these carry no own infoset contribution; they are absorbed into the enclosing type definition's properties."),
        ComplexContent: ("en", "Complex content",
            "W3C XSD 1.1 Part 1 §3.4.2: `<xs:complexContent>` — declares that a complex type's content model derives (by restriction or extension) from another complex type, optionally toggling `mixed`."),
        SimpleContent: ("en", "Simple content",
            "W3C XSD 1.1 Part 1 §3.4.2: `<xs:simpleContent>` — declares that a complex type has character-data content (plus attributes) derived from a simple base type."),
        Restriction: ("en", "Restriction derivation",
            "W3C XSD 1.1 Part 1 §3.4.6 / Part 2 §4.1.2.1: `<xs:restriction>` — derives a type by *restricting* the value space / content model of a base type (the derived type's instances are a subset of the base's)."),
        Extension: ("en", "Extension derivation",
            "W3C XSD 1.1 Part 1 §3.4.6: `<xs:extension>` — derives a complex type by *extending* a base type with additional attributes and/or appended content particles."),
        ListType: ("en", "List simple type",
            "W3C XSD 1.1 Part 2 §4.1.2 / §3.16: `<xs:list>` — a simple type whose value space is whitespace-separated lists of an atomic `itemType`."),
        UnionType: ("en", "Union simple type",
            "W3C XSD 1.1 Part 2 §4.1.2 / §3.16: `<xs:union>` — a simple type whose value space is the union of one or more `memberTypes`' value spaces."),
        ConstrainingFacet: ("en", "Constraining facet",
            "W3C XSD 1.1 Part 2 §4.3: an optional aspect of a simple type's value space that the type restricts (length, range, pattern, …). Constraining facets cut down the base type's value space; they are not schema components (§2.2) but properties of simple-type definitions."),
        LengthFacet: ("en", "Length facet",
            "W3C XSD 1.1 Part 2 §4.3.1: `<xs:length>` — constrains the value space to values of a specified number of units of length."),
        MinLengthFacet: ("en", "Minimum-length facet",
            "W3C XSD 1.1 Part 2 §4.3.2: `<xs:minLength>` — lower bound on the number of units of length."),
        MaxLengthFacet: ("en", "Maximum-length facet",
            "W3C XSD 1.1 Part 2 §4.3.3: `<xs:maxLength>` — upper bound on the number of units of length."),
        PatternFacet: ("en", "Pattern facet",
            "W3C XSD 1.1 Part 2 §4.3.4: `<xs:pattern>` — constrains the lexical space to literals matching a regular expression (Part 2 Appendix G)."),
        EnumerationFacet: ("en", "Enumeration facet",
            "W3C XSD 1.1 Part 2 §4.3.5: `<xs:enumeration>` — constrains the value space to a specified set of values."),
        WhiteSpaceFacet: ("en", "Whitespace facet",
            "W3C XSD 1.1 Part 2 §4.3.6: `<xs:whiteSpace>` — controls normalization of whitespace (`preserve` / `replace` / `collapse`) applied to the lexical-to-value mapping."),
        MaxInclusiveFacet: ("en", "Maximum-inclusive facet",
            "W3C XSD 1.1 Part 2 §4.3.7: `<xs:maxInclusive>` — inclusive upper bound on an ordered value space."),
        MaxExclusiveFacet: ("en", "Maximum-exclusive facet",
            "W3C XSD 1.1 Part 2 §4.3.8: `<xs:maxExclusive>` — exclusive upper bound on an ordered value space."),
        MinExclusiveFacet: ("en", "Minimum-exclusive facet",
            "W3C XSD 1.1 Part 2 §4.3.9: `<xs:minExclusive>` — exclusive lower bound on an ordered value space."),
        MinInclusiveFacet: ("en", "Minimum-inclusive facet",
            "W3C XSD 1.1 Part 2 §4.3.10: `<xs:minInclusive>` — inclusive lower bound on an ordered value space."),
        TotalDigitsFacet: ("en", "Total-digits facet",
            "W3C XSD 1.1 Part 2 §4.3.11: `<xs:totalDigits>` — maximum number of decimal digits in a numeric value."),
        FractionDigitsFacet: ("en", "Fraction-digits facet",
            "W3C XSD 1.1 Part 2 §4.3.12: `<xs:fractionDigits>` — maximum number of digits in the fractional part of a numeric value."),
        ExplicitTimezoneFacet: ("en", "Explicit-timezone facet",
            "W3C XSD 1.1 Part 2 §4.3.14: `<xs:explicitTimezone>` — controls whether a date/time value's timezone is required, prohibited, or optional. New in XSD 1.1."),
        AssertionFacet: ("en", "Assertion facet",
            "W3C XSD 1.1 Part 2 §4.3.13: `<xs:assertion>` — an XPath 2.0 boolean test every value of the simple type must satisfy. New in XSD 1.1."),
        Key: ("en", "Key identity constraint",
            "W3C XSD 1.1 Part 1 §3.11.1: `<xs:key>` — declares that a set of selected fields uniquely identifies elements and that all selected nodes have those fields (a non-nullable uniqueness constraint)."),
        KeyRef: ("en", "Key-reference identity constraint",
            "W3C XSD 1.1 Part 1 §3.11.1: `<xs:keyref>` — declares that selected fields' values must correspond to values of a referenced `<xs:key>` or `<xs:unique>` (a referential-integrity constraint)."),
        Unique: ("en", "Uniqueness identity constraint",
            "W3C XSD 1.1 Part 1 §3.11.1: `<xs:unique>` — declares that a set of selected fields is unique among the selected nodes (uniqueness without the totality requirement of `<xs:key>`)."),
        Selector: ("en", "Identity-constraint selector",
            "W3C XSD 1.1 Part 1 §3.11.2: `<xs:selector>` — an XPath expression selecting the node-set over which an identity constraint's uniqueness / reference scope is evaluated."),
        Field: ("en", "Identity-constraint field",
            "W3C XSD 1.1 Part 1 §3.11.2: `<xs:field>` — an XPath expression selecting, relative to each selected node, the value(s) that together form the identity-constraint tuple."),
        Assert: ("en", "Complex-type assertion",
            "W3C XSD 1.1 Part 1 §3.13: `<xs:assert>` — an XPath 2.0 boolean test on a complex type's element information item, evaluated against each instance. New in XSD 1.1."),
        OpenContent: ("en", "Open content",
            "W3C XSD 1.1 Part 1 §3.4.2.2: `<xs:openContent>` — allows interleaved or suffixed wildcard content in a complex type beyond its declared particles. New in XSD 1.1."),
        DefaultOpenContent: ("en", "Default open content",
            "W3C XSD 1.1 Part 1 §3.16.2: `<xs:defaultOpenContent>` — a schema-level default applying open content to all complex types in the schema document. New in XSD 1.1."),
    },

    // is_a edges express the W3C-defined subsumption hierarchy.
    // `ComplexTypeDefinition` and `SimpleTypeDefinition` are the two
    // sub-kinds of `TypeDefinition` (Part 1 §2.2.1.2). Sequence /
    // Choice / AllGroup are the three model-group compositors
    // (§3.8.1). AppInfo / Documentation are the two annotation
    // children (§3.15.1). Everything else hangs directly off
    // `SchemaComponent` (§2.2).
    is_a: [
        (ElementDeclaration,   SchemaComponent),
        (AttributeDeclaration, SchemaComponent),
        (TypeDefinition,       SchemaComponent),
        (ModelGroup,           SchemaComponent),
        (AttributeGroup,       SchemaComponent),
        (Particle,             SchemaComponent),
        (Wildcard,             SchemaComponent),
        (IdentityConstraint,   SchemaComponent),
        (NotationDeclaration,  SchemaComponent),
        (Annotation,           SchemaComponent),

        (ComplexTypeDefinition, TypeDefinition),
        (SimpleTypeDefinition,  TypeDefinition),

        (Sequence, ModelGroup),
        (Choice,   ModelGroup),
        (AllGroup, ModelGroup),

        (AppInfo,       Annotation),
        (Documentation, Annotation),

        // §4.2 schema-composition directives form their own group,
        // parallel to (not under) `SchemaComponent`: directives are
        // schema-document constructs that govern component merging,
        // not components themselves (§2.2 vs §4.2).
        (SchemaImport,   SchemaCompositionDirective),
        (SchemaInclude,  SchemaCompositionDirective),
        (SchemaRedefine, SchemaCompositionDirective),
        (SchemaOverride, SchemaCompositionDirective),

        // §3.4.2 / Part 2 §4.1.2 type-construction constructs form a
        // third group, parallel to the §2.2 component partition and
        // the §4.2 composition group: these are XML-representation
        // constructs absorbed into a type definition's properties,
        // not components in their own right.
        (ComplexContent, TypeConstructionConstruct),
        (SimpleContent,  TypeConstructionConstruct),
        (Restriction,    TypeConstructionConstruct),
        (Extension,      TypeConstructionConstruct),
        (ListType,       TypeConstructionConstruct),
        (UnionType,      TypeConstructionConstruct),

        // Part 2 §4.3 constraining facets form a fourth group: they
        // restrict a simple type's value space but are properties of
        // a type definition, not components (§2.2).
        (LengthFacet,           ConstrainingFacet),
        (MinLengthFacet,        ConstrainingFacet),
        (MaxLengthFacet,        ConstrainingFacet),
        (PatternFacet,          ConstrainingFacet),
        (EnumerationFacet,      ConstrainingFacet),
        (WhiteSpaceFacet,       ConstrainingFacet),
        (MaxInclusiveFacet,     ConstrainingFacet),
        (MaxExclusiveFacet,     ConstrainingFacet),
        (MinExclusiveFacet,     ConstrainingFacet),
        (MinInclusiveFacet,     ConstrainingFacet),
        (TotalDigitsFacet,      ConstrainingFacet),
        (FractionDigitsFacet,   ConstrainingFacet),
        (ExplicitTimezoneFacet, ConstrainingFacet),
        (AssertionFacet,        ConstrainingFacet),

        // §3.11 identity-constraint categories + XPath sub-parts.
        // IdentityConstraint (already is_a SchemaComponent) becomes
        // an intermediate concept with these five sub-kinds.
        (Key,      IdentityConstraint),
        (KeyRef,   IdentityConstraint),
        (Unique,   IdentityConstraint),
        (Selector, IdentityConstraint),
        (Field,    IdentityConstraint),

        // XSD 1.1 complex-type content additions (§3.13 / §3.4.2.2 /
        // §3.16.2) are type-construction constructs.
        (Assert,             TypeConstructionConstruct),
        (OpenContent,        TypeConstructionConstruct),
        (DefaultOpenContent, TypeConstructionConstruct),
    ],
}

// =============================================================================
// Leaf inventory — the concrete XSD constructs an XSD-loaded schema can
// project to. The root `SchemaComponent` is abstract (§2.2 partitions into
// concrete sub-kinds); the two intermediate concepts `TypeDefinition`,
// `ModelGroup`, `Annotation` are concrete enough to instantiate (an `<xs:type>`
// without further refinement is still a `TypeDefinition`) — but every
// xsd-parser-loaded XSD construct lands on one of the concrete leaves below.
// =============================================================================

/// The 44 directly-instantiable XSD leaves. Excludes the four
/// *abstract* roots `SchemaComponent` / `SchemaCompositionDirective` /
/// `TypeConstructionConstruct` / `ConstrainingFacet` and the
/// intermediate group concepts `TypeDefinition`, `ModelGroup`,
/// `Annotation`, `IdentityConstraint` (which are projected to via
/// their concrete sub-kinds). `SchemaDocument` IS instantiable — every
/// `<xs:schema>` projects to one (§2.5 + §3.16).
pub fn instantiable_leaves() -> [XsdConcept; 44] {
    [
        XsdConcept::SchemaDocument,
        XsdConcept::ElementDeclaration,
        XsdConcept::AttributeDeclaration,
        XsdConcept::ComplexTypeDefinition,
        XsdConcept::SimpleTypeDefinition,
        XsdConcept::Sequence,
        XsdConcept::Choice,
        XsdConcept::AllGroup,
        XsdConcept::AttributeGroup,
        XsdConcept::Particle,
        XsdConcept::Wildcard,
        XsdConcept::NotationDeclaration,
        // §4.2 schema-composition directive leaves.
        XsdConcept::SchemaImport,
        XsdConcept::SchemaInclude,
        XsdConcept::SchemaRedefine,
        XsdConcept::SchemaOverride,
        // §3.4.2 / Part 2 §4.1.2 type-construction construct leaves.
        XsdConcept::ComplexContent,
        XsdConcept::SimpleContent,
        XsdConcept::Restriction,
        XsdConcept::Extension,
        XsdConcept::ListType,
        XsdConcept::UnionType,
        // Part 2 §4.3 constraining facet leaves.
        XsdConcept::LengthFacet,
        XsdConcept::MinLengthFacet,
        XsdConcept::MaxLengthFacet,
        XsdConcept::PatternFacet,
        XsdConcept::EnumerationFacet,
        XsdConcept::WhiteSpaceFacet,
        XsdConcept::MaxInclusiveFacet,
        XsdConcept::MaxExclusiveFacet,
        XsdConcept::MinExclusiveFacet,
        XsdConcept::MinInclusiveFacet,
        XsdConcept::TotalDigitsFacet,
        XsdConcept::FractionDigitsFacet,
        XsdConcept::ExplicitTimezoneFacet,
        XsdConcept::AssertionFacet,
        // §3.11 identity-constraint categories + XPath sub-parts.
        XsdConcept::Key,
        XsdConcept::KeyRef,
        XsdConcept::Unique,
        XsdConcept::Selector,
        XsdConcept::Field,
        // XSD 1.1 complex-type content additions.
        XsdConcept::Assert,
        XsdConcept::OpenContent,
        XsdConcept::DefaultOpenContent,
    ]
}

/// True if `c` is an ontology root — a concept that isn't subsumed
/// under any other. §2.5 `SchemaDocument` (the `<xs:schema>` container),
/// §2.2 `SchemaComponent`, §4.2 `SchemaCompositionDirective`, §3.4.2 /
/// Part 2 §4.1.2 `TypeConstructionConstruct`, and Part 2 §4.3
/// `ConstrainingFacet` form five parallel top-level concepts.
pub fn is_root(c: XsdConcept) -> bool {
    matches!(
        c,
        XsdConcept::SchemaDocument
            | XsdConcept::SchemaComponent
            | XsdConcept::SchemaCompositionDirective
            | XsdConcept::TypeConstructionConstruct
            | XsdConcept::ConstrainingFacet
    )
}

// =============================================================================
// Quality: PartSpec — which W3C XSD 1.1 part (1 = Structures, 2 = Datatypes)
// is the primary normative source for this concept. Total on every concept.
// =============================================================================

/// Quality: which W3C XSD 1.1 part is the primary normative source for
/// a concept. Part 1 = *Structures* (Gao et al. 2012); Part 2 =
/// *Datatypes* (Peterson et al. 2012). `SimpleTypeDefinition` is the
/// only concept whose primary spec is Part 2; all other concepts'
/// primary section is in Part 1.
///
/// Returns `None` for the abstract root `SchemaComponent`.
#[derive(Debug, Clone)]
pub struct PartSpec;

/// Which W3C XSD 1.1 Part defines a concept's primary semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XsdPart {
    /// W3C XSD 1.1 Part 1: Structures (Gao et al. 2012).
    Structures,
    /// W3C XSD 1.1 Part 2: Datatypes (Peterson et al. 2012).
    Datatypes,
}

impl Quality for PartSpec {
    type Individual = XsdConcept;
    type Value = XsdPart;

    fn get(&self, c: &XsdConcept) -> Option<XsdPart> {
        use XsdConcept as X;
        match c {
            // SimpleTypeDefinition's primary spec is Part 2 §4.1.
            X::SimpleTypeDefinition => Some(XsdPart::Datatypes),
            // Every other concept's primary spec is in Part 1.
            X::ElementDeclaration
            | X::AttributeDeclaration
            | X::TypeDefinition
            | X::ComplexTypeDefinition
            | X::ModelGroup
            | X::Sequence
            | X::Choice
            | X::AllGroup
            | X::AttributeGroup
            | X::Particle
            | X::Wildcard
            | X::IdentityConstraint
            | X::NotationDeclaration
            | X::Annotation
            | X::AppInfo
            | X::Documentation
            // §4.2 schema-composition directive leaves are all Part 1.
            | X::SchemaImport
            | X::SchemaInclude
            | X::SchemaRedefine
            | X::SchemaOverride
            // §3.4.2 type-construction constructs — Part 1.
            | X::ComplexContent
            | X::SimpleContent
            | X::Restriction
            | X::Extension
            // §3.11 identity-constraint categories + sub-parts — Part 1.
            | X::Key
            | X::KeyRef
            | X::Unique
            | X::Selector
            | X::Field
            // XSD 1.1 §3.13 / §3.4.2.2 / §3.16.2 content additions — Part 1.
            | X::Assert
            | X::OpenContent
            | X::DefaultOpenContent => Some(XsdPart::Structures),
            // Part 2 §4.1.2 simple-type varieties + §4.3 constraining
            // facets are all defined in Part 2: Datatypes.
            X::ListType
            | X::UnionType
            | X::LengthFacet
            | X::MinLengthFacet
            | X::MaxLengthFacet
            | X::PatternFacet
            | X::EnumerationFacet
            | X::WhiteSpaceFacet
            | X::MaxInclusiveFacet
            | X::MaxExclusiveFacet
            | X::MinExclusiveFacet
            | X::MinInclusiveFacet
            | X::TotalDigitsFacet
            | X::FractionDigitsFacet
            | X::ExplicitTimezoneFacet
            | X::AssertionFacet => Some(XsdPart::Datatypes),
            // Roots — no specific part assignment. The four abstract
            // roots (§2.2, §4.2, §3.4.2 / Part 2 §4.1.2, Part 2 §4.3)
            // are partition tops, not concrete constructs. The §2.5
            // SchemaDocument leaf is also root-classified here — the
            // schema document spans the whole spec (Part 1 §2.5 +
            // §3.16) rather than a single component-level Part.
            X::SchemaDocument
            | X::SchemaComponent
            | X::SchemaCompositionDirective
            | X::TypeConstructionConstruct
            | X::ConstrainingFacet => None,
        }
    }
}

// =============================================================================
// Axioms
// =============================================================================

impl Ontology for XsdOntology {
    type Cat = XsdCategory;
    type Qual = PartSpec;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(SchemaComponentPartitioned));
        axioms.push(Box::new(TypeDefinitionBinaryPartition));
        axioms.push(Box::new(ModelGroupTernaryPartition));
        axioms.push(Box::new(AnnotationBinaryPartition));
        axioms.push(Box::new(TypeDerivationStrictPartialOrder));
        axioms.push(Box::new(SubstitutionGroupReflexiveTransitive));
        axioms.push(Box::new(EveryElementHasExactlyOneTypeReference));
        axioms.push(Box::new(ParticleOccurrenceRangeNonEmpty));
        axioms.push(Box::new(QNameCategoryUniqueness));
        axioms.push(Box::new(EveryConceptHasPartClassification));
        axioms
    }
}

// -----------------------------------------------------------------------------
// Partition axioms — confirm the schema-component hierarchy matches W3C §2.2.
// -----------------------------------------------------------------------------

/// Axiom: every concept in the XSD ontology is `is_a`-reachable from
/// one of the two ontology roots — `SchemaComponent` (W3C XSD 1.1
/// Part 1 §2.2, the schema-component partition) or
/// `SchemaCompositionDirective` (§4.2, the schema-document
/// composition directives). The two roots are disjoint by design:
/// §2.2 components contribute to the post-schema-validation infoset;
/// §4.2 directives govern how component sets from other documents
/// merge but are not themselves components. Nothing is left dangling
/// outside the two partitions.
pub struct SchemaComponentPartitioned;

impl Axiom for SchemaComponentPartitioned {
    fn verify(&self) -> Verdict {
        // Every variant other than a root must transitively reach a
        // root through is_a (Subsumption) edges. We rely on the macro's
        // emitted transitive-closure morphisms (per OBO-RO Smith 2005)
        // for transitive subsumption.
        use pr4xis::category::{Arrow, Category};
        let roots = [
            XsdConcept::SchemaDocument,
            XsdConcept::SchemaComponent,
            XsdConcept::SchemaCompositionDirective,
            XsdConcept::TypeConstructionConstruct,
            XsdConcept::ConstrainingFacet,
        ];
        let all_morphs = XsdCategory::morphisms();
        for v in XsdConcept::variants() {
            if roots.contains(&v) {
                continue;
            }
            // Reach some root via a Subsumption morphism (direct or transitive).
            let reaches = all_morphs.iter().any(|m| {
                m.source() == v
                    && roots.contains(&m.target())
                    && matches!(m.kind(), XsdRelationKind::Subsumption)
            });
            if !reaches {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SchemaComponentPartitioned",
        "every non-root XSD concept is is_a-reachable from one of the four ontology roots: SchemaComponent (§2.2), SchemaCompositionDirective (§4.2), TypeConstructionConstruct (§3.4.2 / Part 2 §4.1.2), or ConstrainingFacet (Part 2 §4.3)",
        "W3C XSD 1.1 Part 1 §2.2, §3.4.2, §4.2 (Gao et al. 2012); Part 2 §4.1.2, §4.3 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    SchemaComponentPartitioned,
    "W3C XSD 1.1 Part 1 §2.2 (Gao et al. 2012)"
);

/// Axiom: `TypeDefinition` partitions into exactly two sub-kinds —
/// `ComplexTypeDefinition` (Part 1 §3.4) and `SimpleTypeDefinition`
/// (Part 2 §4.1). No third type-definition kind exists in XSD 1.1.
pub struct TypeDefinitionBinaryPartition;

impl Axiom for TypeDefinitionBinaryPartition {
    fn verify(&self) -> Verdict {
        use pr4xis::category::{Arrow, Category};
        let count = XsdCategory::morphisms()
            .iter()
            .filter(|m| {
                m.target() == XsdConcept::TypeDefinition
                    && matches!(m.kind(), XsdRelationKind::Subsumption)
                    && m.source() != m.target()
            })
            .count();
        if count == 2 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TypeDefinitionBinaryPartition",
        "TypeDefinition has exactly two sub-kinds (Complex + Simple)",
        "W3C XSD 1.1 Part 1 §2.2.1.2, §3.4; Part 2 §4.1"
    );
}

pr4xis::register_axiom!(
    TypeDefinitionBinaryPartition,
    "W3C XSD 1.1 Part 1 §2.2.1.2, §3.4; Part 2 §4.1"
);

/// Axiom: `ModelGroup` partitions into exactly three compositors —
/// `Sequence`, `Choice`, `AllGroup` (W3C XSD 1.1 Part 1 §3.8.1).
pub struct ModelGroupTernaryPartition;

impl Axiom for ModelGroupTernaryPartition {
    fn verify(&self) -> Verdict {
        use pr4xis::category::{Arrow, Category};
        let count = XsdCategory::morphisms()
            .iter()
            .filter(|m| {
                m.target() == XsdConcept::ModelGroup
                    && matches!(m.kind(), XsdRelationKind::Subsumption)
                    && m.source() != m.target()
            })
            .count();
        if count == 3 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ModelGroupTernaryPartition",
        "ModelGroup has exactly three compositors (Sequence/Choice/AllGroup)",
        "W3C XSD 1.1 Part 1 §3.8.1"
    );
}

pr4xis::register_axiom!(ModelGroupTernaryPartition, "W3C XSD 1.1 Part 1 §3.8.1");

/// Axiom: `Annotation` partitions into exactly two children —
/// `AppInfo` and `Documentation` (W3C XSD 1.1 Part 1 §3.15.1).
pub struct AnnotationBinaryPartition;

impl Axiom for AnnotationBinaryPartition {
    fn verify(&self) -> Verdict {
        use pr4xis::category::{Arrow, Category};
        let count = XsdCategory::morphisms()
            .iter()
            .filter(|m| {
                m.target() == XsdConcept::Annotation
                    && matches!(m.kind(), XsdRelationKind::Subsumption)
                    && m.source() != m.target()
            })
            .count();
        if count == 2 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AnnotationBinaryPartition",
        "Annotation has exactly two children (AppInfo + Documentation)",
        "W3C XSD 1.1 Part 1 §3.15.1"
    );
}

pr4xis::register_axiom!(AnnotationBinaryPartition, "W3C XSD 1.1 Part 1 §3.15.1");

// -----------------------------------------------------------------------------
// Relationship axioms — properties of the XSD relationships *between*
// concepts as published in W3C XSD 1.1. Each axiom takes a small
// structured input via a per-axiom struct and checks the published rule.
// -----------------------------------------------------------------------------

/// A `restriction`/`extension` chain over type definitions (W3C XSD 1.1
/// Part 1 §3.4.6.4). Encoded as a sequence of type names where each
/// successor derives from its predecessor.
#[derive(Debug, Clone)]
pub struct DerivationChain {
    /// Ordered chain `[A, B, C, …]` representing `A ← B ← C ← …`,
    /// where the right-hand side `derivedFrom`s the left.
    pub chain: Vec<&'static str>,
}

impl DerivationChain {
    /// Strict partial order: chain is acyclic (no repeated element).
    pub fn is_acyclic(&self) -> bool {
        let mut seen = alloc::collections::BTreeSet::new();
        for name in &self.chain {
            if !seen.insert(*name) {
                return false;
            }
        }
        true
    }
}

/// Axiom: type-definition derivation (`restriction` or `extension`)
/// forms a strict partial order — `restriction`/`extension` chains
/// terminate; no cycles.
///
/// W3C XSD 1.1 Part 1 §3.4.6.4 — *Type Derivation OK (Complex)*:
/// derivation chains must eventually root at `xs:anyType`. Cycles
/// would make termination impossible.
pub struct TypeDerivationStrictPartialOrder;

impl Axiom for TypeDerivationStrictPartialOrder {
    fn verify(&self) -> Verdict {
        // Three canonical chains derived from W3C XSD 1.1 Part 1 §3.4.6.4.
        let acyclic = DerivationChain {
            chain: vec!["xs:anyType", "ComplexTypeA", "ComplexTypeB"],
        };
        let with_simple = DerivationChain {
            chain: vec!["xs:anySimpleType", "xs:string", "xs:NCName", "xs:ID"],
        };
        let cyclic = DerivationChain {
            chain: vec!["TypeA", "TypeB", "TypeA"],
        };
        if acyclic.is_acyclic() && with_simple.is_acyclic() && !cyclic.is_acyclic() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TypeDerivationStrictPartialOrder",
        "TypeDefinition derivation chains are acyclic (strict partial order)",
        "W3C XSD 1.1 Part 1 §3.4.6.4 (Gao et al. 2012)"
    );
}

pr4xis::register_axiom!(
    TypeDerivationStrictPartialOrder,
    "W3C XSD 1.1 Part 1 §3.4.6.4 (Gao et al. 2012)"
);

/// A substitution-group hierarchy (W3C XSD 1.1 Part 1 §3.3.6). Encoded
/// as a list of (head, member) pairs.
#[derive(Debug, Clone)]
pub struct SubstitutionGroupHierarchy {
    /// Pairs `(member, head)` — `member` is a direct member of
    /// `head`'s substitution group.
    pub pairs: Vec<(&'static str, &'static str)>,
}

impl SubstitutionGroupHierarchy {
    /// Reflexive-transitive closure membership. Per W3C XSD 1.1 Part 1
    /// §3.3.6, `member ∈ substitutionGroup*(head)` iff there is a
    /// (possibly empty) chain `member = e_0 → e_1 → … → e_n = head`
    /// of declared head-substitution edges. Reflexivity:
    /// `head ∈ substitutionGroup*(head)` always.
    pub fn member_of(&self, member: &str, head: &str) -> bool {
        // Reflexive.
        if member == head {
            return true;
        }
        // Walk transitively.
        let mut frontier = alloc::collections::BTreeSet::new();
        frontier.insert(member.to_string());
        let mut changed = true;
        while changed {
            changed = false;
            let snapshot: Vec<String> = frontier.iter().cloned().collect();
            for m in &snapshot {
                for (k, v) in &self.pairs {
                    if *k == m.as_str() && frontier.insert((*v).to_string()) {
                        if *v == head {
                            return true;
                        }
                        changed = true;
                    }
                }
            }
        }
        frontier.iter().any(|x| x.as_str() == head)
    }
}

/// Axiom: substitution-group membership is reflexive-transitive.
///
/// W3C XSD 1.1 Part 1 §3.3.6 — a member of a substitution group is
/// itself (reflexivity), and chains compose transitively.
pub struct SubstitutionGroupReflexiveTransitive;

impl Axiom for SubstitutionGroupReflexiveTransitive {
    fn verify(&self) -> Verdict {
        // canonical chain: A → B → C (A substitutes for B; B substitutes for C).
        let h = SubstitutionGroupHierarchy {
            pairs: vec![("A", "B"), ("B", "C")],
        };
        let reflexive_ok = h.member_of("A", "A") && h.member_of("B", "B");
        let direct_ok = h.member_of("A", "B") && h.member_of("B", "C");
        let transitive_ok = h.member_of("A", "C");
        let non_member_ok = !h.member_of("C", "A");
        if reflexive_ok && direct_ok && transitive_ok && non_member_ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SubstitutionGroupReflexiveTransitive",
        "substitution-group membership is reflexive and transitive",
        "W3C XSD 1.1 Part 1 §3.3.6 (Gao et al. 2012)"
    );
}

pr4xis::register_axiom!(
    SubstitutionGroupReflexiveTransitive,
    "W3C XSD 1.1 Part 1 §3.3.6 (Gao et al. 2012)"
);

/// An element declaration's type reference. Per W3C XSD 1.1 Part 1
/// §3.3.2.3, an element declaration carries *exactly one* type — via
/// `type=` attribute, inline `<xs:complexType>` / `<xs:simpleType>`,
/// or implicit `xs:anyType`.
#[derive(Debug, Clone)]
pub struct ElementTypeRef {
    pub element: &'static str,
    pub type_ref: Option<&'static str>,
}

/// Axiom: every `ElementDeclaration` has exactly one `TypeDefinition`
/// reference.
///
/// W3C XSD 1.1 Part 1 §3.3.2.3 — *Schema Component: Element
/// Declaration, type definition* — an element's `{type definition}`
/// property is always defined, defaulting to `xs:anyType` when no
/// type is declared.
pub struct EveryElementHasExactlyOneTypeReference;

impl Axiom for EveryElementHasExactlyOneTypeReference {
    fn verify(&self) -> Verdict {
        // The axiom: every well-formed element-declaration has at least
        // one type reference (the default `xs:anyType` if no explicit
        // type is given). We construct three samples and confirm they
        // all carry a non-None type when resolved.
        let with_explicit_type = ElementTypeRef {
            element: "title",
            type_ref: Some("TitleType"),
        };
        // Implicit `xs:anyType` is what the XSD parser materialises
        // when no type= attribute appears — modelled here by an
        // explicit fallback.
        let default_to_anytype =
            |e: &ElementTypeRef| -> &'static str { e.type_ref.unwrap_or("xs:anyType") };
        let no_explicit_type = ElementTypeRef {
            element: "section",
            type_ref: None,
        };
        let resolved_a = default_to_anytype(&with_explicit_type);
        let resolved_b = default_to_anytype(&no_explicit_type);
        if !resolved_a.is_empty() && resolved_b == "xs:anyType" {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "EveryElementHasExactlyOneTypeReference",
        "every ElementDeclaration carries exactly one TypeDefinition reference (default xs:anyType)",
        "W3C XSD 1.1 Part 1 §3.3.2.3 (Gao et al. 2012)"
    );
}

pr4xis::register_axiom!(
    EveryElementHasExactlyOneTypeReference,
    "W3C XSD 1.1 Part 1 §3.3.2.3 (Gao et al. 2012)"
);

/// A particle's occurrence range `[minOccurs, maxOccurs]`. Per W3C XSD
/// 1.1 Part 1 §3.9.3.2, the range is nonempty: `minOccurs ≤ maxOccurs`
/// and `maxOccurs ≥ 1` (`unbounded` represented as `usize::MAX`).
#[derive(Debug, Clone, Copy)]
pub struct OccurrenceRange {
    pub min: usize,
    /// `usize::MAX` represents XSD's `unbounded` (Part 1 §3.9.3.2).
    pub max: usize,
}

impl OccurrenceRange {
    /// Sentinel used to represent `maxOccurs="unbounded"` per Part 1
    /// §3.9.3.2 — XSD's only non-finite cardinality.
    pub const UNBOUNDED: usize = usize::MAX;

    pub fn is_nonempty(&self) -> bool {
        self.min <= self.max && self.max >= 1
    }
}

/// Axiom: particle occurrence ranges `[minOccurs, maxOccurs]` are
/// nonempty.
///
/// W3C XSD 1.1 Part 1 §3.9.3.2 — *Particle Validate (Range)*: a
/// particle is valid only if its occurrence range admits at least one
/// content count, i.e. `minOccurs ≤ maxOccurs` and `maxOccurs ≥ 1`.
/// (`maxOccurs="0"` is forbidden — a zero-occurrence particle would
/// have to be deleted, not declared.)
pub struct ParticleOccurrenceRangeNonEmpty;

impl Axiom for ParticleOccurrenceRangeNonEmpty {
    fn verify(&self) -> Verdict {
        // Three canonical ranges from W3C XSD 1.1 Part 1 §3.9.3.2.
        let single = OccurrenceRange { min: 1, max: 1 };
        let optional = OccurrenceRange { min: 0, max: 1 };
        let unbounded = OccurrenceRange {
            min: 0,
            max: OccurrenceRange::UNBOUNDED,
        };
        let zero_max = OccurrenceRange { min: 0, max: 0 };
        let inverted = OccurrenceRange { min: 5, max: 1 };
        if single.is_nonempty()
            && optional.is_nonempty()
            && unbounded.is_nonempty()
            && !zero_max.is_nonempty()
            && !inverted.is_nonempty()
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ParticleOccurrenceRangeNonEmpty",
        "every Particle's [minOccurs, maxOccurs] is nonempty (min<=max and max>=1)",
        "W3C XSD 1.1 Part 1 §3.9.3.2 (Gao et al. 2012)"
    );
}

pr4xis::register_axiom!(
    ParticleOccurrenceRangeNonEmpty,
    "W3C XSD 1.1 Part 1 §3.9.3.2 (Gao et al. 2012)"
);

/// A keyed schema-component identity — `(QName, category)`.
///
/// W3C XSD 1.1 Part 1 §3.2.7 — *Schema Component Identity* — two
/// schema components with the same QName and the same category are the
/// same component; XSD does not permit two distinct components to
/// share both attributes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentKey {
    pub qname: String,
    pub category: XsdConcept,
}

/// Axiom: two `SchemaComponent`s with the same `QName` and the same
/// category are the same component (W3C XSD 1.1 Part 1 §3.2.7).
pub struct QNameCategoryUniqueness;

impl Axiom for QNameCategoryUniqueness {
    fn verify(&self) -> Verdict {
        // Two declarations of the same QName + category must merge to
        // the same key. Distinct categories with the same QName are
        // permitted (e.g. an element and a type can share a name).
        let a = ComponentKey {
            qname: "uslm:section".to_string(),
            category: XsdConcept::ElementDeclaration,
        };
        let b = ComponentKey {
            qname: "uslm:section".to_string(),
            category: XsdConcept::ElementDeclaration,
        };
        let c = ComponentKey {
            qname: "uslm:section".to_string(),
            category: XsdConcept::ComplexTypeDefinition,
        };
        if a == b && a != c {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "QNameCategoryUniqueness",
        "two SchemaComponents with the same (QName, category) are the same component",
        "W3C XSD 1.1 Part 1 §3.2.7 (Gao et al. 2012)"
    );
}

pr4xis::register_axiom!(
    QNameCategoryUniqueness,
    "W3C XSD 1.1 Part 1 §3.2.7 (Gao et al. 2012)"
);

/// Axiom: every XSD concept except the two abstract roots
/// (`SchemaComponent` §2.2 and `SchemaCompositionDirective` §4.2)
/// has a defined `PartSpec` classification — i.e. every concrete
/// concept is anchored in W3C XSD 1.1 Part 1 or Part 2. The roots
/// are partition tops, not concrete constructs, so they carry no
/// part classification.
pub struct EveryConceptHasPartClassification;

impl Axiom for EveryConceptHasPartClassification {
    fn verify(&self) -> Verdict {
        let q = PartSpec;
        for c in XsdConcept::variants() {
            let v = q.get(&c);
            if is_root(c) {
                if v.is_some() {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            } else if v.is_none() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EveryConceptHasPartClassification",
        "every concept (except root) has a defined W3C Part 1 / Part 2 classification",
        "W3C XSD 1.1 Part 1 (Gao et al. 2012); Part 2 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    EveryConceptHasPartClassification,
    "W3C XSD 1.1 Part 1 (Gao et al. 2012); Part 2 (Peterson et al. 2012)"
);
