use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// OWL 2 Web Ontology Language — W3C Recommendation (2012)
// https://www.w3.org/TR/owl2-syntax/
//
// OWL is built on RDF, adding formal logic (Description Logic SROIQ).
// It defines classes, properties, individuals, and restrictions.
//
// The OWL metamodel has two layers:
// 1. OwlConcept — the abstract syntax: what KINDS of things OWL defines
// 2. OwlOntology — a loaded ontology: the actual classes/properties/individuals
//
// References:
// - W3C OWL 2 Structural Specification and Functional-Style Syntax (2012)
// - W3C OWL 2 Direct Semantics (2012) — SROIQ model theory
// - Baader et al., An Introduction to Description Logics (2003)
// - Chiarcos & Sukhareva, OLiA (Semantic Web journal, 2015)

// =============================================================================
// OWL metamodel — the category of OWL constructs
// =============================================================================

/// The kinds of constructs in OWL 2.
/// From W3C OWL 2 Structural Specification §2.
///
/// Each variant maps to a specific IRI in the OWL 2 namespace.
/// This is NOT string matching — each concept has an identity (its IRI)
/// defined by the W3C spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum OwlConcept {
    // === Class expressions (W3C OWL 2 §8) ===
    /// owl:Class — a named class (W3C OWL 2 §8.1)
    Class,
    /// owl:Restriction — anonymous class defined by a constraint (W3C OWL 2 §8.2)
    Restriction,
    /// owl:unionOf — C1 ∪ C2 ∪ ... (W3C OWL 2 §8.1.3)
    UnionOf,
    /// owl:intersectionOf — C1 ∩ C2 ∩ ... (W3C OWL 2 §8.1.2)
    IntersectionOf,
    /// owl:complementOf — ¬C (W3C OWL 2 §8.1.4)
    ComplementOf,
    /// owl:oneOf — {a, b, c} enumeration (W3C OWL 2 §8.1.5)
    OneOf,

    // === Property expressions (W3C OWL 2 §9) ===
    /// owl:ObjectProperty — relates individuals to individuals (W3C OWL 2 §9.1)
    ObjectProperty,
    /// owl:DatatypeProperty — relates individuals to literals (W3C OWL 2 §9.2)
    DatatypeProperty,
    /// owl:AnnotationProperty — non-logical metadata (W3C OWL 2 §10)
    AnnotationProperty,

    // === Property characteristics (W3C OWL 2 §9.2) ===
    /// owl:FunctionalProperty — at most one value per subject
    FunctionalProperty,
    /// owl:InverseFunctionalProperty — at most one subject per value
    InverseFunctionalProperty,
    /// owl:TransitiveProperty — composition closure
    TransitiveProperty,
    /// owl:SymmetricProperty
    SymmetricProperty,
    /// owl:AsymmetricProperty
    AsymmetricProperty,
    /// owl:ReflexiveProperty
    ReflexiveProperty,
    /// owl:IrreflexiveProperty
    IrreflexiveProperty,

    // === Individuals (W3C OWL 2 §5.6) ===
    /// owl:NamedIndividual — an explicitly named instance
    NamedIndividual,

    // === Restriction fillers (W3C OWL 2 §8.2) ===
    /// owl:someValuesFrom — existential restriction ∃p.C
    SomeValuesFrom,
    /// owl:allValuesFrom — universal restriction ∀p.C
    AllValuesFrom,
    /// owl:hasValue — ∃p.{a}
    HasValue,
    /// owl:minCardinality
    MinCardinality,
    /// owl:maxCardinality
    MaxCardinality,
    /// owl:cardinality (exact)
    ExactCardinality,

    // === Ontology header (W3C OWL 2 §3) ===
    /// owl:Ontology — the ontology node itself
    Ontology,
}

impl OwlConcept {
    /// Is this concept a class expression?
    pub fn is_class_expression(&self) -> bool {
        matches!(
            self,
            Self::Class
                | Self::Restriction
                | Self::UnionOf
                | Self::IntersectionOf
                | Self::ComplementOf
                | Self::OneOf
        )
    }

    /// Is this concept a property expression?
    pub fn is_property(&self) -> bool {
        matches!(
            self,
            Self::ObjectProperty | Self::DatatypeProperty | Self::AnnotationProperty
        )
    }

    /// Is this concept a property characteristic?
    pub fn is_property_characteristic(&self) -> bool {
        matches!(
            self,
            Self::FunctionalProperty
                | Self::InverseFunctionalProperty
                | Self::TransitiveProperty
                | Self::SymmetricProperty
                | Self::AsymmetricProperty
                | Self::ReflexiveProperty
                | Self::IrreflexiveProperty
        )
    }
}

/// Relation kind for OWL concept arrows.
///
/// Per OBO-RO (Smith et al. 2005), every arrow carries a relation-kind
/// tag. The OWL metamodel category has one structural relation: the
/// type-hierarchy/containment edges declared by the W3C OWL 2
/// Structural Specification (Class expressions subsume Class,
/// property characteristics subsume ObjectProperty, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwlRelationKind {
    Structural,
}

/// A morphism between OWL concepts — the structural relationships
/// defined by the OWL 2 spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwlRelation {
    pub source: OwlConcept,
    pub target: OwlConcept,
}

impl Arrow for OwlRelation {
    type Object = OwlConcept;
    type Kind = OwlRelationKind;
    fn source(&self) -> OwlConcept {
        self.source
    }
    fn target(&self) -> OwlConcept {
        self.target
    }
    fn kind(&self) -> OwlRelationKind {
        OwlRelationKind::Structural
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("OwlRelation"),
            description: Label::new_static(
                "structural relation between OWL 2 constructs per W3C OWL 2 Structural Specification",
            ),
            citation: Citation::parse_static("W3C OWL 2 (2012); Baader et al. (2003)"),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

/// The OWL category — the structural relationships between OWL constructs.
pub struct OwlCategory;

impl Category for OwlCategory {
    type Object = OwlConcept;
    type Morphism = OwlRelation;

    fn identity(obj: &OwlConcept) -> OwlRelation {
        OwlRelation {
            source: *obj,
            target: *obj,
        }
    }

    fn compose(f: &OwlRelation, g: &OwlRelation) -> Option<OwlRelation> {
        if f.target != g.source {
            return None;
        }
        if f.source == f.target {
            return Some(g.clone());
        }
        if g.source == g.target {
            return Some(f.clone());
        }
        let candidate = OwlRelation {
            source: f.source,
            target: g.target,
        };
        // Partial category (#166): composition is only defined when the
        // result is itself a declared morphism. `morphisms()` builds the
        // full reachability closure (Warshall 1962), so any composable pair
        // resolves inside it.
        if Self::morphisms().contains(&candidate) {
            Some(candidate)
        } else {
            None
        }
    }

    fn morphisms() -> Vec<OwlRelation> {
        use OwlConcept::*;
        use std::collections::HashSet;

        // Direct OWL 2 structural relations (W3C OWL 2 §§5, 8.2, 9.2).
        let mut direct: HashSet<(OwlConcept, OwlConcept)> = HashSet::new();

        let class_exprs = [Restriction, UnionOf, IntersectionOf, ComplementOf, OneOf];
        for &ce in &class_exprs {
            direct.insert((ce, Class));
        }

        let prop_chars = [
            FunctionalProperty,
            InverseFunctionalProperty,
            TransitiveProperty,
            SymmetricProperty,
            AsymmetricProperty,
            ReflexiveProperty,
            IrreflexiveProperty,
        ];
        for &pc in &prop_chars {
            direct.insert((pc, ObjectProperty));
        }

        let fillers = [
            SomeValuesFrom,
            AllValuesFrom,
            HasValue,
            MinCardinality,
            MaxCardinality,
            ExactCardinality,
        ];
        for &f in &fillers {
            direct.insert((Restriction, f));
        }

        direct.insert((Restriction, ObjectProperty));
        direct.insert((Restriction, DatatypeProperty));
        direct.insert((ObjectProperty, Class));
        direct.insert((DatatypeProperty, Class));
        direct.insert((NamedIndividual, Class));
        direct.insert((Ontology, Class));
        direct.insert((Ontology, ObjectProperty));
        direct.insert((Ontology, DatatypeProperty));
        direct.insert((Ontology, AnnotationProperty));
        direct.insert((Ontology, NamedIndividual));

        // Warshall (1962) transitive closure — required for ClosureLaw and
        // AssociativityLaw (Mac Lane CWM Ch. I §1).
        let mut closure = direct.clone();
        loop {
            let mut added = false;
            let snap: Vec<_> = closure.iter().cloned().collect();
            for &(a, b) in &snap {
                for &(b2, c) in &snap {
                    if b == b2 && !closure.contains(&(a, c)) {
                        closure.insert((a, c));
                        added = true;
                    }
                }
            }
            if !added {
                break;
            }
        }

        let mut m: Vec<OwlRelation> = Vec::new();
        for c in OwlConcept::variants() {
            m.push(OwlRelation {
                source: c,
                target: c,
            });
        }
        for (s, t) in closure {
            if s != t {
                m.push(OwlRelation {
                    source: s,
                    target: t,
                });
            }
        }
        m
    }
}

// =============================================================================
// OWL 2 vocabulary — canonical IRIs from W3C spec
// =============================================================================

/// Well-known OWL 2 IRIs from W3C OWL 2 Structural Specification §2.4.
/// Each IRI is the canonical identity of an OWL concept.
pub struct OwlVocabulary;

impl OwlVocabulary {
    pub const OWL_NS: &str = "http://www.w3.org/2002/07/owl#";

    pub const OWL_CLASS: &str = "http://www.w3.org/2002/07/owl#Class";
    pub const OWL_RESTRICTION: &str = "http://www.w3.org/2002/07/owl#Restriction";
    pub const OWL_OBJECT_PROPERTY: &str = "http://www.w3.org/2002/07/owl#ObjectProperty";
    pub const OWL_DATATYPE_PROPERTY: &str = "http://www.w3.org/2002/07/owl#DatatypeProperty";
    pub const OWL_ANNOTATION_PROPERTY: &str = "http://www.w3.org/2002/07/owl#AnnotationProperty";
    pub const OWL_NAMED_INDIVIDUAL: &str = "http://www.w3.org/2002/07/owl#NamedIndividual";
    pub const OWL_ONTOLOGY: &str = "http://www.w3.org/2002/07/owl#Ontology";
    pub const OWL_THING: &str = "http://www.w3.org/2002/07/owl#Thing";
    pub const OWL_NOTHING: &str = "http://www.w3.org/2002/07/owl#Nothing";

    // Class constructors
    pub const OWL_UNION_OF: &str = "http://www.w3.org/2002/07/owl#unionOf";
    pub const OWL_INTERSECTION_OF: &str = "http://www.w3.org/2002/07/owl#intersectionOf";
    pub const OWL_COMPLEMENT_OF: &str = "http://www.w3.org/2002/07/owl#complementOf";
    pub const OWL_ONE_OF: &str = "http://www.w3.org/2002/07/owl#oneOf";

    // Property characteristics
    pub const OWL_FUNCTIONAL_PROPERTY: &str = "http://www.w3.org/2002/07/owl#FunctionalProperty";
    pub const OWL_INVERSE_FUNCTIONAL: &str =
        "http://www.w3.org/2002/07/owl#InverseFunctionalProperty";
    pub const OWL_TRANSITIVE_PROPERTY: &str = "http://www.w3.org/2002/07/owl#TransitiveProperty";
    pub const OWL_SYMMETRIC_PROPERTY: &str = "http://www.w3.org/2002/07/owl#SymmetricProperty";

    // Restriction fillers
    pub const OWL_SOME_VALUES_FROM: &str = "http://www.w3.org/2002/07/owl#someValuesFrom";
    pub const OWL_ALL_VALUES_FROM: &str = "http://www.w3.org/2002/07/owl#allValuesFrom";
    pub const OWL_HAS_VALUE: &str = "http://www.w3.org/2002/07/owl#hasValue";
    pub const OWL_MIN_CARDINALITY: &str = "http://www.w3.org/2002/07/owl#minCardinality";
    pub const OWL_MAX_CARDINALITY: &str = "http://www.w3.org/2002/07/owl#maxCardinality";
    pub const OWL_CARDINALITY: &str = "http://www.w3.org/2002/07/owl#cardinality";

    // Axiom predicates
    pub const OWL_EQUIVALENT_CLASS: &str = "http://www.w3.org/2002/07/owl#equivalentClass";
    pub const OWL_DISJOINT_WITH: &str = "http://www.w3.org/2002/07/owl#disjointWith";
    pub const OWL_INVERSE_OF: &str = "http://www.w3.org/2002/07/owl#inverseOf";
    pub const OWL_IMPORTS: &str = "http://www.w3.org/2002/07/owl#imports";
    pub const OWL_VERSION_INFO: &str = "http://www.w3.org/2002/07/owl#versionInfo";
    pub const OWL_ON_PROPERTY: &str = "http://www.w3.org/2002/07/owl#onProperty";

    /// Map an OWL IRI to its concept. This is ontological lookup:
    /// the IRI IS the identity of the concept.
    pub fn from_iri(iri: &str) -> Option<OwlConcept> {
        match iri {
            Self::OWL_CLASS | Self::OWL_THING | Self::OWL_NOTHING => Some(OwlConcept::Class),
            Self::OWL_RESTRICTION => Some(OwlConcept::Restriction),
            Self::OWL_OBJECT_PROPERTY => Some(OwlConcept::ObjectProperty),
            Self::OWL_DATATYPE_PROPERTY => Some(OwlConcept::DatatypeProperty),
            Self::OWL_ANNOTATION_PROPERTY => Some(OwlConcept::AnnotationProperty),
            Self::OWL_NAMED_INDIVIDUAL => Some(OwlConcept::NamedIndividual),
            Self::OWL_ONTOLOGY => Some(OwlConcept::Ontology),
            Self::OWL_FUNCTIONAL_PROPERTY => Some(OwlConcept::FunctionalProperty),
            Self::OWL_INVERSE_FUNCTIONAL => Some(OwlConcept::InverseFunctionalProperty),
            Self::OWL_TRANSITIVE_PROPERTY => Some(OwlConcept::TransitiveProperty),
            Self::OWL_SYMMETRIC_PROPERTY => Some(OwlConcept::SymmetricProperty),
            _ => None,
        }
    }

    /// Map an OWL element local name to its concept.
    /// Used during XML reading — the local name within the owl: namespace
    /// identifies the concept.
    pub fn from_local_name(name: &str) -> Option<OwlConcept> {
        match name {
            "Class" => Some(OwlConcept::Class),
            "Restriction" => Some(OwlConcept::Restriction),
            "ObjectProperty" => Some(OwlConcept::ObjectProperty),
            "DatatypeProperty" => Some(OwlConcept::DatatypeProperty),
            "AnnotationProperty" => Some(OwlConcept::AnnotationProperty),
            "NamedIndividual" => Some(OwlConcept::NamedIndividual),
            "Ontology" => Some(OwlConcept::Ontology),
            "FunctionalProperty" => Some(OwlConcept::FunctionalProperty),
            "InverseFunctionalProperty" => Some(OwlConcept::InverseFunctionalProperty),
            "TransitiveProperty" => Some(OwlConcept::TransitiveProperty),
            "SymmetricProperty" => Some(OwlConcept::SymmetricProperty),
            "AsymmetricProperty" => Some(OwlConcept::AsymmetricProperty),
            "ReflexiveProperty" => Some(OwlConcept::ReflexiveProperty),
            "IrreflexiveProperty" => Some(OwlConcept::IrreflexiveProperty),
            "Thing" => Some(OwlConcept::Class),
            "Nothing" => Some(OwlConcept::Class),
            _ => None,
        }
    }

    /// Resolve a namespace prefix + local name to a full OWL IRI.
    pub fn resolve(prefix: &str, local: &str) -> Option<String> {
        if prefix == "owl" {
            Some(format!("{}{}", Self::OWL_NS, local))
        } else {
            None
        }
    }
}

// =============================================================================
// Loaded ontology types — the output of reading an OWL file
// =============================================================================

/// An RDF 1.1 literal (W3C RDF 1.1 Concepts §3.3) carried verbatim
/// from a triple: a lexical form, an optional `xml:lang` BCP-47 tag
/// (RDF 1.1 §3.3 — only present on `rdf:langString` literals), and an
/// optional datatype IRI (typed literals). The two tags are mutually
/// exclusive per RDF 1.1 §3.3; a literal with neither implicitly types
/// as `xsd:string` (RDF 1.1 §3.5 simple-literal sugar).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwlLiteral {
    pub lexical: String,
    pub lang: Option<String>,
    pub datatype: Option<String>,
}

/// One side of an OWL 2 annotation: either an IRI reference (a named
/// entity) or a literal value (W3C OWL 2 §10.1).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OwlAnnotationValue {
    Iri(String),
    Literal(OwlLiteral),
}

/// An OWL 2 *Annotation* (W3C OWL 2 §10): a non-logical (predicate,
/// value) pair attached to an entity, the ontology header, or an
/// axiom. `rdfs:label` and `rdfs:comment` are the most common
/// annotation predicates; `dc:creator`, `dc:title`, `owl:versionInfo`,
/// and the vocabulary's own metadata predicates also flow through
/// here. Distinct annotations may share a predicate (multi-language
/// `rdfs:label`, multiple `dc:creator`s, …); the order is
/// document-order preserved by `read_owl` (see [`super::reader`]).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwlAnnotation {
    pub predicate: String,
    pub value: OwlAnnotationValue,
}

/// An OWL 2 class expression (W3C OWL 2 §8). Named classes are leaf
/// nodes; the remaining variants are the constructors `ObjectUnionOf`
/// / `ObjectIntersectionOf` / `ObjectComplementOf` (§8.1) and the
/// property restrictions (§8.2). Anonymous restrictions are
/// canonically represented by their structural content, never by
/// blank-node ID — the lens's content-addressed blank-node labelling
/// is computed from this shape.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OwlClassExpression {
    /// A named class (W3C OWL 2 §8 — a class IRI denotes a class).
    Named(String),
    /// A property restriction (W3C OWL 2 §8.2).
    Restriction(OwlRestriction),
    /// `owl:unionOf` — C₁ ∪ C₂ ∪ … (W3C OWL 2 §8.1.3).
    Union(Vec<OwlClassExpression>),
    /// `owl:intersectionOf` — C₁ ∩ C₂ ∩ … (W3C OWL 2 §8.1.2).
    Intersection(Vec<OwlClassExpression>),
    /// `owl:complementOf` — ¬C (W3C OWL 2 §8.1.4).
    Complement(Box<OwlClassExpression>),
}

/// A property restriction (W3C OWL 2 §8.2) — an anonymous class
/// defined by a constraint on a property's values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwlRestriction {
    /// `owl:onProperty` IRI (W3C OWL 2 §8.2 — every restriction names
    /// exactly one property; [`RestrictionNeedsProperty`] enforces it).
    pub on_property: String,
    /// The restriction kind + its filler.
    pub kind: OwlRestrictionKind,
}

/// W3C OWL 2 §8.2 property restriction variants — the existential /
/// universal / value / cardinality fillers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OwlRestrictionKind {
    /// `owl:someValuesFrom` — ∃p.C (W3C OWL 2 §8.2.1).
    SomeValuesFrom(Box<OwlClassExpression>),
    /// `owl:allValuesFrom` — ∀p.C (W3C OWL 2 §8.2.2).
    AllValuesFrom(Box<OwlClassExpression>),
    /// `owl:hasValue` — ∃p.{a} (W3C OWL 2 §8.2.3); value is an IRI or
    /// literal target.
    HasValue(OwlAnnotationValue),
    /// `owl:minCardinality` n / `owl:minQualifiedCardinality` n on C
    /// (W3C OWL 2 §8.3.1).
    MinCardinality {
        n: u32,
        on_class: Option<Box<OwlClassExpression>>,
    },
    /// `owl:maxCardinality` n / `owl:maxQualifiedCardinality` n on C
    /// (W3C OWL 2 §8.3.2).
    MaxCardinality {
        n: u32,
        on_class: Option<Box<OwlClassExpression>>,
    },
    /// `owl:cardinality` n / `owl:qualifiedCardinality` n on C (W3C
    /// OWL 2 §8.3.3 — exact cardinality).
    ExactCardinality {
        n: u32,
        on_class: Option<Box<OwlClassExpression>>,
    },
}

/// An OWL class — a concept in the loaded ontology.
///
/// Carries the W3C OWL 2 RDF Mapping (Patel-Schneider & Motik 2012)
/// shape: named-class IRI, multi-language `rdfs:label`s and
/// `rdfs:comment`s, free-form annotations (§10), the union of named
/// superclasses with the parsed superclass expressions (§9 SubClassOf
/// axiom), equivalent classes (§9 EquivalentClasses), and disjoint
/// classes (§9 DisjointClasses). The simple `label` / `comment`
/// projections (the first entry of `labels` / `comments`) and the
/// `superclasses` IRI list are retained for backward-compatibility
/// with the historical reader API.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OwlClass {
    pub iri: String,
    /// First-label projection (W3C OWL 2 RDF Mapping §3.2 `rdfs:label`)
    /// — convenience accessor for the historical API.
    pub label: Option<String>,
    /// First-comment projection (W3C OWL 2 RDF Mapping §3.2
    /// `rdfs:comment`).
    pub comment: Option<String>,
    /// Named superclass IRIs, in document order — the simple
    /// `rdfs:subClassOf` projection (W3C RDFS §2.1) historical
    /// consumers depend on.
    pub superclasses: Vec<String>,
    /// All `rdfs:label` literals (RDF 1.1 §3.3 multi-language;
    /// preserved in document order).
    pub labels: Vec<OwlLiteral>,
    /// All `rdfs:comment` literals.
    pub comments: Vec<OwlLiteral>,
    /// Free-form OWL 2 §10 annotations attached to the class entity.
    pub annotations: Vec<OwlAnnotation>,
    /// Full superclass expressions (W3C OWL 2 §9 SubClassOf — both
    /// named superclasses and anonymous class expressions).
    pub superclass_expressions: Vec<OwlClassExpression>,
    /// `owl:equivalentClass` targets (W3C OWL 2 §9.1.1).
    pub equivalent_classes: Vec<OwlClassExpression>,
    /// `owl:disjointWith` targets (W3C OWL 2 §9.1.3).
    pub disjoint_classes: Vec<OwlClassExpression>,
}

/// An OWL object property — a relationship between classes. Carries its
/// `rdfs:subPropertyOf` superproperties (the property hierarchy, W3C OWL
/// 2 §9.2.1 / RDF Schema §5.1.7), the parallel of a class's superclasses.
///
/// As with [`OwlClass`], the multi-language `labels` / `comments` and
/// the free-form `annotations` give the full W3C OWL 2 §10 shape;
/// the single-value `label` / `comment` / `domain` / `range` fields
/// remain as first-entry projections for backward compatibility.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OwlObjectProperty {
    pub iri: String,
    /// First-label projection.
    pub label: Option<String>,
    /// First-comment projection.
    pub comment: Option<String>,
    /// First `rdfs:domain` IRI (W3C OWL 2 §9.2.5).
    pub domain: Option<String>,
    /// First `rdfs:range` IRI (W3C OWL 2 §9.2.6).
    pub range: Option<String>,
    /// Named superproperty IRIs from `rdfs:subPropertyOf` (RDFS §5.1.7).
    pub superproperties: Vec<String>,
    /// All `rdfs:label` literals.
    pub labels: Vec<OwlLiteral>,
    /// All `rdfs:comment` literals.
    pub comments: Vec<OwlLiteral>,
    /// Free-form OWL 2 §10 annotations.
    pub annotations: Vec<OwlAnnotation>,
    /// `owl:inverseOf` target IRI (W3C OWL 2 §9.2.7 InverseObjectProperties).
    pub inverse_of: Option<String>,
    /// `owl:equivalentProperty` target IRIs (W3C OWL 2 §9.2.1
    /// EquivalentObjectProperties).
    pub equivalent_properties: Vec<String>,
    /// `owl:propertyDisjointWith` target IRIs (W3C OWL 2 §9.2.2
    /// DisjointObjectProperties).
    pub disjoint_properties: Vec<String>,
}

/// An OWL individual — an instance of a class.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OwlIndividual {
    pub iri: String,
    pub types: Vec<String>,
    /// First-label projection.
    pub label: Option<String>,
    /// All `rdfs:label` literals (RDF 1.1 §3.3 multi-language).
    pub labels: Vec<OwlLiteral>,
    /// All `rdfs:comment` literals.
    pub comments: Vec<OwlLiteral>,
    /// Free-form OWL 2 §10 annotations.
    pub annotations: Vec<OwlAnnotation>,
}

/// A complete OWL ontology loaded from an OWL/XML file.
#[derive(Debug, Clone, Default)]
pub struct OwlOntology {
    pub iri: String,
    pub classes: Vec<OwlClass>,
    pub properties: Vec<OwlObjectProperty>,
    pub individuals: Vec<OwlIndividual>,
    /// `(child, parent)` class `rdfs:subClassOf` edges.
    pub taxonomy: Vec<(String, String)>,
    /// `(child, parent)` property `rdfs:subPropertyOf` edges — the
    /// property hierarchy, parallel to `taxonomy` for classes.
    pub property_taxonomy: Vec<(String, String)>,
    /// Free-form annotations on the ontology header itself (W3C OWL 2
    /// §3.5 — `owl:versionInfo`, `dc:title`, `dc:creator`, …).
    pub ontology_annotations: Vec<OwlAnnotation>,
}

impl OwlOntology {
    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    pub fn find_class(&self, iri: &str) -> Option<&OwlClass> {
        self.classes.iter().find(|c| c.iri == iri)
    }

    pub fn find_class_by_label(&self, label: &str) -> Option<&OwlClass> {
        self.classes
            .iter()
            .find(|c| c.label.as_deref() == Some(label))
    }

    pub fn subclasses_of(&self, parent_iri: &str) -> Vec<&OwlClass> {
        self.classes
            .iter()
            .filter(|c| c.superclasses.iter().any(|s| s == parent_iri))
            .collect()
    }

    pub fn superclasses_of(&self, child_iri: &str) -> Vec<&str> {
        self.classes
            .iter()
            .find(|c| c.iri == child_iri)
            .map(|c| c.superclasses.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    pub fn property_count(&self) -> usize {
        self.properties.len()
    }

    pub fn find_property(&self, iri: &str) -> Option<&OwlObjectProperty> {
        self.properties.iter().find(|p| p.iri == iri)
    }

    pub fn find_property_by_label(&self, label: &str) -> Option<&OwlObjectProperty> {
        self.properties
            .iter()
            .find(|p| p.label.as_deref() == Some(label))
    }

    pub fn subproperties_of(&self, parent_iri: &str) -> Vec<&OwlObjectProperty> {
        self.properties
            .iter()
            .filter(|p| p.superproperties.iter().any(|s| s == parent_iri))
            .collect()
    }

    pub fn superproperties_of(&self, child_iri: &str) -> Vec<&str> {
        self.properties
            .iter()
            .find(|p| p.iri == child_iri)
            .map(|p| p.superproperties.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
}

// =============================================================================
// Axioms — OWL structural invariants from W3C spec
// =============================================================================

/// W3C OWL 2 §8.2: every `owl:Restriction` must relate to a property.
///
/// W3C OWL 2 Structural Specification (2012) §8.2 ("Property Restrictions"):
/// every restriction is defined by exactly one `owl:onProperty` value
/// (object or datatype). Baader et al. (2003) *Description Logic
/// Handbook* ch. 2 frames the same constraint in DL syntax: restrictions
/// are ∃R.C / ∀R.C / ≥nR.C / ≤nR.C, all parameterised by a property R.
pub struct RestrictionNeedsProperty;

impl Axiom for RestrictionNeedsProperty {
    fn verify(&self) -> Verdict {
        // Structural: Restriction has morphisms to ObjectProperty and DatatypeProperty
        let morphisms = OwlCategory::morphisms();
        let ok = morphisms
            .iter()
            .any(|m| m.source == OwlConcept::Restriction && m.target == OwlConcept::ObjectProperty);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RestrictionNeedsProperty",
        "every owl:Restriction must have exactly one owl:onProperty",
        "W3C OWL 2 (2012) Structural Specification §8.2; Baader et al. (2003) ch. 2"
    );
}
pr4xis::register_axiom!(
    RestrictionNeedsProperty,
    "W3C OWL 2 (2012) Structural Specification §8.2; Baader et al. (2003) ch. 2"
);

/// Quality: is this OWL concept a class expression?
#[derive(Debug, Clone)]
pub struct IsClassExpression;

impl Quality for IsClassExpression {
    type Individual = OwlConcept;
    type Value = ();

    fn get(&self, concept: &OwlConcept) -> Option<()> {
        if concept.is_class_expression() {
            Some(())
        } else {
            None
        }
    }
}

/// The OWL metamodel ontology.
pub struct OwlMetaOntology;

impl Ontology for OwlMetaOntology {
    type Cat = OwlCategory;
    type Qual = IsClassExpression;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(RestrictionNeedsProperty)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<OwlCategory>();
    }

    #[test]
    fn ontology_validates() {
        OwlMetaOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
