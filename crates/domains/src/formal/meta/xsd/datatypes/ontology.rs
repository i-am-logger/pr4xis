//! XSD built-in datatype hierarchy — W3C XML Schema 1.1 Part 2.
//!
//! Declares the closed inventory of built-in datatypes (Peterson et
//! al. 2012 §§3.2–3.4) as a Praxis ontology. The `is_a` edges encode
//! each datatype's {base type definition} — the W3C "built-in
//! datatype hierarchy" diagram — so the substrate's transitive-
//! subsumption reasoning gives derivation reachability for free.
//!
//! ## Why a separate ontology from [`super::super::ontology`]
//!
//! The XSD *meta-model* ontology (`XsdConcept`) describes what a
//! schema component / facet / type-construction construct *is*. The
//! datatype hierarchy is a different axis: it is the fixed lattice of
//! value spaces (`xs:int ⊏ xs:long ⊏ xs:integer ⊏ xs:decimal`) that a
//! `SimpleTypeDefinition` restricts. Modelling it as its own ontology
//! keeps each enum coherent (Spivak 2014 §3: one category per
//! sort of object) and lets the lattice axioms speak only of
//! datatypes.
//!
//! ## Citation
//!
//! - **W3C XML Schema 1.1 Part 2: Datatypes**, Peterson, Gao,
//!   Akhmedov, Malhotra, Biron & Sperberg-McQueen 2012, W3C
//!   Recommendation 2012-04-05. §3.2 special datatypes, §3.3 the 19
//!   primitive datatypes, §3.4 the 28 derived datatypes.
//! - **Spivak, D.** *Category Theory for the Sciences*, MIT Press
//!   2014, §3 (categories).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::FinitelyGenerated;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// =============================================================================
// Concept inventory — every datatype cites the W3C XSD 1.1 Part 2 § that
// defines it. Three groups: §3.2 special, §3.3 the 19 primitives, §3.4 the
// 28 derived datatypes (25 from XSD 1.0 + 3 new in XSD 1.1).
// =============================================================================

pr4xis::ontology! {
    name: "XsdDatatype",
    source: "Peterson, Gao, Akhmedov, Malhotra, Biron & Sperberg-McQueen (eds.) (2012) W3C XML Schema 1.1 Part 2: Datatypes, W3C Recommendation 2012-04-05, §§3.2-3.4",

    concepts: [
        // §3.2 special datatypes.
        AnyType,
        AnySimpleType,
        AnyAtomicType,
        // §3.3 the 19 primitive datatypes.
        StringType,
        Boolean,
        Decimal,
        Float,
        Double,
        Duration,
        DateTime,
        Time,
        Date,
        GYearMonth,
        GYear,
        GMonthDay,
        GDay,
        GMonth,
        HexBinary,
        Base64Binary,
        AnyUri,
        QName,
        Notation,
        // §3.4 derived — string family.
        NormalizedString,
        Token,
        Language,
        NmToken,
        NmTokens,
        Name,
        NcName,
        Id,
        IdRef,
        IdRefs,
        Entity,
        Entities,
        // §3.4 derived — integer family (restrictions of decimal).
        Integer,
        NonPositiveInteger,
        NegativeInteger,
        Long,
        Int,
        Short,
        Byte,
        NonNegativeInteger,
        UnsignedLong,
        UnsignedInt,
        UnsignedShort,
        UnsignedByte,
        PositiveInteger,
        // §3.4 derived — duration / dateTime (new in XSD 1.1).
        YearMonthDuration,
        DayTimeDuration,
        DateTimeStamp,
    ],

    labels: {
        AnyType: ("en", "any type",
            "W3C XSD 1.1 Part 1 §3.4.7 / Part 2 §3.2: `xs:anyType` — the ur-type, root of every type definition (complex and simple). Its {base type definition} is itself."),
        AnySimpleType: ("en", "any simple type",
            "W3C XSD 1.1 Part 2 §3.2.1: `xs:anySimpleType` — the root of the simple-type sub-lattice; {base type definition} is `xs:anyType`. Its variety is ·absent·."),
        AnyAtomicType: ("en", "any atomic type",
            "W3C XSD 1.1 Part 2 §3.2.2: `xs:anyAtomicType` — the root of the atomic datatypes; {base type definition} is `xs:anySimpleType`. Every primitive datatype derives from it. New in XSD 1.1 (in 1.0 the primitives derived directly from `xs:anySimpleType`)."),
        StringType: ("en", "string",
            "W3C XSD 1.1 Part 2 §3.3.1: `xs:string` — a primitive datatype whose value space is the set of finite-length sequences of XML characters."),
        Boolean: ("en", "boolean",
            "W3C XSD 1.1 Part 2 §3.3.2: `xs:boolean` — a primitive datatype with the two-valued logic value space {true, false}."),
        Decimal: ("en", "decimal",
            "W3C XSD 1.1 Part 2 §3.3.3: `xs:decimal` — a primitive datatype for arbitrary-precision decimal numbers."),
        Float: ("en", "float",
            "W3C XSD 1.1 Part 2 §3.3.4: `xs:float` — a primitive datatype patterned on IEEE 754 single-precision binary floating point."),
        Double: ("en", "double",
            "W3C XSD 1.1 Part 2 §3.3.5: `xs:double` — a primitive datatype patterned on IEEE 754 double-precision binary floating point."),
        Duration: ("en", "duration",
            "W3C XSD 1.1 Part 2 §3.3.6: `xs:duration` — a primitive datatype for durations of time, expressed as months and seconds."),
        DateTime: ("en", "date time",
            "W3C XSD 1.1 Part 2 §3.3.7: `xs:dateTime` — a primitive datatype for instants of time, using the seven-property date/time model (Part 2 §D)."),
        Time: ("en", "time",
            "W3C XSD 1.1 Part 2 §3.3.8: `xs:time` — a primitive datatype for an instant of time that recurs every day."),
        Date: ("en", "date",
            "W3C XSD 1.1 Part 2 §3.3.9: `xs:date` — a primitive datatype for a Gregorian calendar date."),
        GYearMonth: ("en", "gregorian year month",
            "W3C XSD 1.1 Part 2 §3.3.10: `xs:gYearMonth` — a primitive datatype for a Gregorian month in a specific year."),
        GYear: ("en", "gregorian year",
            "W3C XSD 1.1 Part 2 §3.3.11: `xs:gYear` — a primitive datatype for a Gregorian calendar year."),
        GMonthDay: ("en", "gregorian month day",
            "W3C XSD 1.1 Part 2 §3.3.12: `xs:gMonthDay` — a primitive datatype for a Gregorian date that recurs every year (month + day)."),
        GDay: ("en", "gregorian day",
            "W3C XSD 1.1 Part 2 §3.3.13: `xs:gDay` — a primitive datatype for a Gregorian day that recurs every month."),
        GMonth: ("en", "gregorian month",
            "W3C XSD 1.1 Part 2 §3.3.14: `xs:gMonth` — a primitive datatype for a Gregorian month that recurs every year."),
        HexBinary: ("en", "hex binary",
            "W3C XSD 1.1 Part 2 §3.3.15: `xs:hexBinary` — a primitive datatype for arbitrary hex-encoded binary data."),
        Base64Binary: ("en", "base sixty four binary",
            "W3C XSD 1.1 Part 2 §3.3.16: `xs:base64Binary` — a primitive datatype for arbitrary Base64-encoded binary data (RFC 2045 §6.8)."),
        AnyUri: ("en", "any uri",
            "W3C XSD 1.1 Part 2 §3.3.17: `xs:anyURI` — a primitive datatype for a Uniform Resource Identifier reference (RFC 3986 / RFC 3987)."),
        QName: ("en", "qualified name",
            "W3C XSD 1.1 Part 2 §3.3.18: `xs:QName` — a primitive datatype for an XML qualified name (Namespaces in XML §4): a namespace-name + local part."),
        Notation: ("en", "notation",
            "W3C XSD 1.1 Part 2 §3.3.19: `xs:NOTATION` — a primitive datatype for the set of QNames of notations declared in the schema. Only usable via derivation by enumeration."),
        NormalizedString: ("en", "normalized string",
            "W3C XSD 1.1 Part 2 §3.4.1: `xs:normalizedString` — `xs:string` whitespace-`replace`d (tab/LF/CR → space). {base type definition} `xs:string`."),
        Token: ("en", "token",
            "W3C XSD 1.1 Part 2 §3.4.2: `xs:token` — `xs:normalizedString` whitespace-`collapse`d (no leading/trailing/runs of spaces). {base} `xs:normalizedString`."),
        Language: ("en", "language",
            "W3C XSD 1.1 Part 2 §3.4.3: `xs:language` — `xs:token` restricted to RFC 3066 / BCP 47 language tags. {base} `xs:token`."),
        NmToken: ("en", "name token",
            "W3C XSD 1.1 Part 2 §3.4.4: `xs:NMTOKEN` — `xs:token` restricted to the XML 1.0 Nmtoken production. {base} `xs:token`."),
        NmTokens: ("en", "name tokens",
            "W3C XSD 1.1 Part 2 §3.4.5: `xs:NMTOKENS` — a list datatype whose item type is `xs:NMTOKEN`. {base type definition} is `xs:anySimpleType` (lists base on anySimpleType)."),
        Name: ("en", "name",
            "W3C XSD 1.1 Part 2 §3.4.6: `xs:Name` — `xs:token` restricted to the XML 1.0 Name production. {base} `xs:token`."),
        NcName: ("en", "no colon name",
            "W3C XSD 1.1 Part 2 §3.4.7: `xs:NCName` — `xs:Name` without a colon (Namespaces in XML NCName production). {base} `xs:Name`."),
        Id: ("en", "id",
            "W3C XSD 1.1 Part 2 §3.4.8: `xs:ID` — `xs:NCName` carrying the XML 1.0 ID attribute semantics (document-unique). {base} `xs:NCName`."),
        IdRef: ("en", "id reference",
            "W3C XSD 1.1 Part 2 §3.4.9: `xs:IDREF` — `xs:NCName` referencing an `xs:ID` value elsewhere in the document. {base} `xs:NCName`."),
        IdRefs: ("en", "id references",
            "W3C XSD 1.1 Part 2 §3.4.10: `xs:IDREFS` — a list datatype whose item type is `xs:IDREF`. {base type definition} is `xs:anySimpleType`."),
        Entity: ("en", "entity",
            "W3C XSD 1.1 Part 2 §3.4.11: `xs:ENTITY` — `xs:NCName` matching an unparsed entity declared in the DTD. {base} `xs:NCName`."),
        Entities: ("en", "entities",
            "W3C XSD 1.1 Part 2 §3.4.12: `xs:ENTITIES` — a list datatype whose item type is `xs:ENTITY`. {base type definition} is `xs:anySimpleType`."),
        Integer: ("en", "integer",
            "W3C XSD 1.1 Part 2 §3.4.13: `xs:integer` — `xs:decimal` with `fractionDigits` fixed at 0. {base} `xs:decimal`."),
        NonPositiveInteger: ("en", "non positive integer",
            "W3C XSD 1.1 Part 2 §3.4.14: `xs:nonPositiveInteger` — `xs:integer` with `maxInclusive` 0. {base} `xs:integer`."),
        NegativeInteger: ("en", "negative integer",
            "W3C XSD 1.1 Part 2 §3.4.15: `xs:negativeInteger` — `xs:nonPositiveInteger` with `maxInclusive` -1. {base} `xs:nonPositiveInteger`."),
        Long: ("en", "long",
            "W3C XSD 1.1 Part 2 §3.4.16: `xs:long` — `xs:integer` bounded to [-2^63, 2^63-1]. {base} `xs:integer`."),
        Int: ("en", "int",
            "W3C XSD 1.1 Part 2 §3.4.17: `xs:int` — `xs:long` bounded to [-2^31, 2^31-1]. {base} `xs:long`."),
        Short: ("en", "short",
            "W3C XSD 1.1 Part 2 §3.4.18: `xs:short` — `xs:int` bounded to [-2^15, 2^15-1]. {base} `xs:int`."),
        Byte: ("en", "byte",
            "W3C XSD 1.1 Part 2 §3.4.19: `xs:byte` — `xs:short` bounded to [-128, 127]. {base} `xs:short`."),
        NonNegativeInteger: ("en", "non negative integer",
            "W3C XSD 1.1 Part 2 §3.4.20: `xs:nonNegativeInteger` — `xs:integer` with `minInclusive` 0. {base} `xs:integer`."),
        UnsignedLong: ("en", "unsigned long",
            "W3C XSD 1.1 Part 2 §3.4.21: `xs:unsignedLong` — `xs:nonNegativeInteger` bounded to [0, 2^64-1]. {base} `xs:nonNegativeInteger`."),
        UnsignedInt: ("en", "unsigned int",
            "W3C XSD 1.1 Part 2 §3.4.22: `xs:unsignedInt` — `xs:unsignedLong` bounded to [0, 2^32-1]. {base} `xs:unsignedLong`."),
        UnsignedShort: ("en", "unsigned short",
            "W3C XSD 1.1 Part 2 §3.4.23: `xs:unsignedShort` — `xs:unsignedInt` bounded to [0, 2^16-1]. {base} `xs:unsignedInt`."),
        UnsignedByte: ("en", "unsigned byte",
            "W3C XSD 1.1 Part 2 §3.4.24: `xs:unsignedByte` — `xs:unsignedShort` bounded to [0, 255]. {base} `xs:unsignedShort`."),
        PositiveInteger: ("en", "positive integer",
            "W3C XSD 1.1 Part 2 §3.4.25: `xs:positiveInteger` — `xs:nonNegativeInteger` with `minInclusive` 1. {base} `xs:nonNegativeInteger`."),
        YearMonthDuration: ("en", "year month duration",
            "W3C XSD 1.1 Part 2 §3.4.26: `xs:yearMonthDuration` — `xs:duration` restricted to the year/month components. {base} `xs:duration`. New in XSD 1.1."),
        DayTimeDuration: ("en", "day time duration",
            "W3C XSD 1.1 Part 2 §3.4.27: `xs:dayTimeDuration` — `xs:duration` restricted to the day/time components. {base} `xs:duration`. New in XSD 1.1."),
        DateTimeStamp: ("en", "date time stamp",
            "W3C XSD 1.1 Part 2 §3.4.28: `xs:dateTimeStamp` — `xs:dateTime` with `explicitTimezone` fixed at `required`. {base} `xs:dateTime`. New in XSD 1.1."),
    },

    // is_a edges encode each datatype's {base type definition} — the
    // W3C "built-in datatype hierarchy" diagram (Part 2 §3.4). The
    // single root `AnyType` has no base. List datatypes
    // (NMTOKENS / IDREFS / ENTITIES) base on `AnySimpleType`, not on
    // their item types (§3.4.5 / §3.4.10 / §3.4.12).
    is_a: [
        (AnySimpleType,  AnyType),
        (AnyAtomicType,  AnySimpleType),

        // §3.3 — every primitive derives from anyAtomicType.
        (StringType,   AnyAtomicType),
        (Boolean,      AnyAtomicType),
        (Decimal,      AnyAtomicType),
        (Float,        AnyAtomicType),
        (Double,       AnyAtomicType),
        (Duration,     AnyAtomicType),
        (DateTime,     AnyAtomicType),
        (Time,         AnyAtomicType),
        (Date,         AnyAtomicType),
        (GYearMonth,   AnyAtomicType),
        (GYear,        AnyAtomicType),
        (GMonthDay,    AnyAtomicType),
        (GDay,         AnyAtomicType),
        (GMonth,       AnyAtomicType),
        (HexBinary,    AnyAtomicType),
        (Base64Binary, AnyAtomicType),
        (AnyUri,       AnyAtomicType),
        (QName,        AnyAtomicType),
        (Notation,     AnyAtomicType),

        // §3.4 string family.
        (NormalizedString, StringType),
        (Token,            NormalizedString),
        (Language,         Token),
        (NmToken,          Token),
        (Name,             Token),
        (NcName,           Name),
        (Id,               NcName),
        (IdRef,            NcName),
        (Entity,           NcName),
        // List datatypes base on anySimpleType (their item types are a
        // separate relation — see `item_type`).
        (NmTokens,  AnySimpleType),
        (IdRefs,    AnySimpleType),
        (Entities,  AnySimpleType),

        // §3.4 integer family (restrictions of decimal).
        (Integer,            Decimal),
        (NonPositiveInteger, Integer),
        (NegativeInteger,    NonPositiveInteger),
        (Long,               Integer),
        (Int,                Long),
        (Short,              Int),
        (Byte,               Short),
        (NonNegativeInteger, Integer),
        (UnsignedLong,       NonNegativeInteger),
        (UnsignedInt,        UnsignedLong),
        (UnsignedShort,      UnsignedInt),
        (UnsignedByte,       UnsignedShort),
        (PositiveInteger,    NonNegativeInteger),

        // §3.4 duration / dateTime (new in XSD 1.1).
        (YearMonthDuration, Duration),
        (DayTimeDuration,   Duration),
        (DateTimeStamp,     DateTime),
    ],
}

// =============================================================================
// Group inventories — the three closed groups of the Part 2 datatype space.
// =============================================================================

/// The 3 special datatypes (W3C XSD 1.1 Part 2 §3.2): `anyType`,
/// `anySimpleType`, `anyAtomicType`.
/// The special datatypes (§3.2): the lattice roots `anyType` / `anySimpleType`
/// / `anyAtomicType`. DERIVED from the loaded hierarchy (audit 2026-06-12 D-6) —
/// no longer a hand-enumerated array that could drift from the `is_a:` block.
pub fn special_datatypes() -> Vec<XsdDatatypeConcept> {
    XsdDatatypeConcept::variants()
        .into_iter()
        .filter(|d| is_special(*d))
        .collect()
}

/// The primitive datatypes (W3C XSD 1.1 Part 2 §3.3) — the direct Subsumption
/// children of `anyAtomicType`. DERIVED.
pub fn primitive_datatypes() -> Vec<XsdDatatypeConcept> {
    XsdDatatypeConcept::variants()
        .into_iter()
        .filter(|d| is_primitive(*d))
        .collect()
}

/// The derived datatypes (W3C XSD 1.1 Part 2 §3.4) — those that are neither
/// special nor primitive. DERIVED.
pub fn derived_datatypes() -> Vec<XsdDatatypeConcept> {
    XsdDatatypeConcept::variants()
        .into_iter()
        .filter(|d| is_derived(*d))
        .collect()
}

/// The list datatypes (W3C XSD 1.1 Part 2 §3.4.5 / §3.4.10 / §3.4.12):
/// `NMTOKENS` / `IDREFS` / `ENTITIES`. DERIVED — the datatypes carrying an
/// [`item_type`] edge.
pub fn list_datatypes() -> Vec<XsdDatatypeConcept> {
    XsdDatatypeConcept::variants()
        .into_iter()
        .filter(|d| item_type(*d).is_some())
        .collect()
}

/// True if `dt` is a special datatype (§3.2) — `anyAtomicType` or one of its
/// ancestors (the lattice roots above the primitives). DERIVED from the loaded
/// hierarchy (audit 2026-06-12 D-6).
pub fn is_special(dt: XsdDatatypeConcept) -> bool {
    dt == XsdDatatypeConcept::AnyAtomicType
        || ancestors_of(XsdDatatypeConcept::AnyAtomicType).contains(&dt)
}

/// True if `dt` is a primitive datatype (§3.3) — a direct Subsumption child of
/// `anyAtomicType` (i.e. its base type IS `anyAtomicType`). DERIVED.
pub fn is_primitive(dt: XsdDatatypeConcept) -> bool {
    base_type(dt) == Some(XsdDatatypeConcept::AnyAtomicType)
}

/// True if `dt` is a derived datatype (§3.4) — neither special nor primitive.
/// DERIVED (the three groups partition the inventory).
pub fn is_derived(dt: XsdDatatypeConcept) -> bool {
    !is_special(dt) && !is_primitive(dt)
}

/// The {base type definition} of `dt` — the immediate `is_a` parent in
/// the Part 2 §3.4 hierarchy. Returns `None` only for the root
/// `anyType` (whose base is itself per §3.4.7, modelled here as the
/// lattice top with no proper ancestor).
pub fn base_type(dt: XsdDatatypeConcept) -> Option<XsdDatatypeConcept> {
    // DERIVED from the loaded `is_a` closure (audit 2026-06-12 D-16): the {base
    // type definition} is the IMMEDIATE Subsumption parent. The ontology macro
    // emits the transitive closure, so the immediate parent is recovered by
    // transitive reduction — in this single-parent tree it is the DEEPEST
    // ancestor (the one that itself has the most ancestors). This replaces a
    // hand `match` that re-encoded the `is_a:` block — the dual source of truth
    // the now-deleted `BaseTypeAgreesWithCategory` axiom existed only to police.
    ancestors_of(dt)
        .into_iter()
        .max_by_key(|a| ancestors_of(*a).len())
}

/// The proper ancestors of `dt` in the loaded §3.4 Subsumption hierarchy (the
/// macro-emitted transitive closure of the `is_a:` edges).
fn ancestors_of(dt: XsdDatatypeConcept) -> Vec<XsdDatatypeConcept> {
    use pr4xis::category::{Arrow, Category};
    XsdDatatypeCategory::morphisms()
        .into_iter()
        .filter(|m| {
            m.source() == dt
                && m.source() != m.target()
                && matches!(m.kind(), XsdDatatypeRelationKind::Subsumption)
        })
        .map(|m| m.target())
        .collect()
}

/// The {item type definition} of a list datatype (W3C XSD 1.1 Part 2
/// §3.4.5 / §3.4.10 / §3.4.12), or `None` for non-list datatypes. The
/// item type is the type each whitespace-separated token of a list
/// value belongs to — a relation distinct from {base type definition}.
pub fn item_type(dt: XsdDatatypeConcept) -> Option<XsdDatatypeConcept> {
    use XsdDatatypeConcept as D;
    match dt {
        D::NmTokens => Some(D::NmToken),
        D::IdRefs => Some(D::IdRef),
        D::Entities => Some(D::Entity),
        _ => None,
    }
}

// =============================================================================
// Quality: Variety — atomic / list / union (W3C XSD 1.1 Part 2 §2.4.1).
// =============================================================================

/// The {variety} of a simple-type datatype (W3C XSD 1.1 Part 2
/// §2.4.1): atomic, list, or union. `anyType` is the complex ur-type
/// and `anySimpleType` has ·absent· variety, so [`VarietyOf`] returns
/// `None` for both; no built-in datatype has union variety.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variety {
    /// §2.4.1.1: value space is a set of atomic (indivisible) values.
    Atomic,
    /// §2.4.1.2: value space is finite-length sequences of atomic
    /// values of the item type.
    List,
    /// §2.4.1.3: value space is the union of member-type value spaces.
    /// No built-in datatype has union variety; present for totality.
    Union,
}

/// Quality assigning each datatype its {variety} (W3C XSD 1.1 Part 2
/// §2.4.1). `None` for `anyType` (complex) and `anySimpleType`
/// (·absent· variety).
#[derive(Debug, Clone)]
pub struct VarietyOf;

impl Quality for VarietyOf {
    type Individual = XsdDatatypeConcept;
    type Value = Variety;

    fn get(&self, dt: &XsdDatatypeConcept) -> Option<Variety> {
        use XsdDatatypeConcept as D;
        match dt {
            // anyType is complex; anySimpleType's variety is ·absent·.
            D::AnyType | D::AnySimpleType => None,
            // The 3 list datatypes (§3.4.5 / §3.4.10 / §3.4.12).
            D::NmTokens | D::IdRefs | D::Entities => Some(Variety::List),
            // Everything else (anyAtomicType + primitives + atomic
            // derived) is atomic.
            _ => Some(Variety::Atomic),
        }
    }
}

// =============================================================================
// Axioms
// =============================================================================

impl Ontology for XsdDatatypeOntology {
    type Cat = XsdDatatypeCategory;
    type Qual = VarietyOf;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(DatatypeLatticeSingleRoot));
        axioms.push(Box::new(PrimitivesDeriveFromAnyAtomicType));
        axioms.push(Box::new(ListDatatypesHaveBuiltInItemType));
        axioms.push(Box::new(Xsd11DatatypeAdditionsPresent));
        axioms
    }
}

/// Axiom: the datatype lattice has exactly one root — `anyType`
/// (W3C XSD 1.1 Part 2 §3.2 / Part 1 §3.4.7) — and every other
/// datatype reaches it through `is_a` (base-type) edges. `anyType` is
/// the unique concept with [`base_type`] `None`.
pub struct DatatypeLatticeSingleRoot;

impl Axiom for DatatypeLatticeSingleRoot {
    fn verify(&self) -> Verdict {
        let roots: Vec<_> = XsdDatatypeConcept::variants()
            .into_iter()
            .filter(|d| base_type(*d).is_none())
            .collect();
        if roots.as_slice() != [XsdDatatypeConcept::AnyType] {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        // Every non-root reaches AnyType by following base_type.
        for d in XsdDatatypeConcept::variants() {
            let mut cur = d;
            let mut steps = 0usize;
            while let Some(parent) = base_type(cur) {
                cur = parent;
                steps += 1;
                if steps > XsdDatatypeConcept::variants().len() {
                    // A cycle would loop past the variant count.
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
            if cur != XsdDatatypeConcept::AnyType {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "DatatypeLatticeSingleRoot",
        "the built-in datatype hierarchy has exactly one root (anyType) and the base-type relation is acyclic, reaching anyType from every datatype",
        "W3C XSD 1.1 Part 2 §3.2 (Peterson et al. 2012); Part 1 §3.4.7 (Gao et al. 2012)"
    );
}

pr4xis::register_axiom!(
    DatatypeLatticeSingleRoot,
    "W3C XSD 1.1 Part 2 §3.2; Part 1 §3.4.7"
);

// `BaseTypeAgreesWithCategory` deleted (audit 2026-06-12 D-5): it existed only
// to police drift between the hand-coded `base_type` match and the `is_a:`
// edges. `base_type` is now DERIVED from those edges (D-16), so the two
// encodings are one — there is nothing to drift, and nothing to police.

/// Axiom: every one of the 19 primitive datatypes (§3.3) derives
/// directly from `anyAtomicType` (§3.2.2).
pub struct PrimitivesDeriveFromAnyAtomicType;

impl Axiom for PrimitivesDeriveFromAnyAtomicType {
    fn verify(&self) -> Verdict {
        let all = primitive_datatypes()
            .into_iter()
            .all(|p| base_type(p) == Some(XsdDatatypeConcept::AnyAtomicType));
        if all {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PrimitivesDeriveFromAnyAtomicType",
        "each of the 19 primitive datatypes has anyAtomicType as its base type definition",
        "W3C XSD 1.1 Part 2 §3.3, §3.2.2 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    PrimitivesDeriveFromAnyAtomicType,
    "W3C XSD 1.1 Part 2 §3.3, §3.2.2"
);

/// Axiom: each list datatype (§3.4.5 / §3.4.10 / §3.4.12) has a
/// built-in [`item_type`], that item type is atomic (§2.4.1.2 — list
/// item types are atomic or union; the three built-ins are atomic),
/// and no non-list datatype has an item type.
pub struct ListDatatypesHaveBuiltInItemType;

impl Axiom for ListDatatypesHaveBuiltInItemType {
    fn verify(&self) -> Verdict {
        let lists = list_datatypes();
        for d in XsdDatatypeConcept::variants() {
            match item_type(d) {
                Some(item) => {
                    // Only the three built-in lists have item types.
                    if !lists.contains(&d) {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                    // The item type must itself be a built-in datatype
                    // with atomic variety.
                    if VarietyOf.get(&item) != Some(Variety::Atomic) {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
                None => {
                    if lists.contains(&d) {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ListDatatypesHaveBuiltInItemType",
        "exactly the three built-in list datatypes (NMTOKENS/IDREFS/ENTITIES) have an item type, and each item type is an atomic built-in datatype",
        "W3C XSD 1.1 Part 2 §3.4.5, §3.4.10, §3.4.12, §2.4.1.2 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    ListDatatypesHaveBuiltInItemType,
    "W3C XSD 1.1 Part 2 §3.4.5, §3.4.10, §3.4.12"
);

/// Axiom: the three datatypes new in XSD 1.1 — `yearMonthDuration`
/// (§3.4.26), `dayTimeDuration` (§3.4.27), `dateTimeStamp` (§3.4.28) —
/// are present and derive from the expected bases (duration, duration,
/// dateTime respectively).
pub struct Xsd11DatatypeAdditionsPresent;

impl Axiom for Xsd11DatatypeAdditionsPresent {
    fn verify(&self) -> Verdict {
        use XsdDatatypeConcept as D;
        let ok = base_type(D::YearMonthDuration) == Some(D::Duration)
            && base_type(D::DayTimeDuration) == Some(D::Duration)
            && base_type(D::DateTimeStamp) == Some(D::DateTime);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "Xsd11DatatypeAdditionsPresent",
        "the three XSD 1.1 datatype additions (yearMonthDuration, dayTimeDuration, dateTimeStamp) are present and derive from duration/duration/dateTime",
        "W3C XSD 1.1 Part 2 §3.4.26, §3.4.27, §3.4.28 (Peterson et al. 2012)"
    );
}

pr4xis::register_axiom!(
    Xsd11DatatypeAdditionsPresent,
    "W3C XSD 1.1 Part 2 §3.4.26, §3.4.27, §3.4.28"
);
