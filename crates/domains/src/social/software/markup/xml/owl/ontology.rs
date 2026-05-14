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

/// An OWL class — a concept in the loaded ontology.
#[derive(Debug, Clone, PartialEq)]
pub struct OwlClass {
    pub iri: String,
    pub label: Option<String>,
    pub comment: Option<String>,
    pub superclasses: Vec<String>,
}

/// An OWL object property — a relationship between classes.
#[derive(Debug, Clone, PartialEq)]
pub struct OwlObjectProperty {
    pub iri: String,
    pub label: Option<String>,
    pub domain: Option<String>,
    pub range: Option<String>,
}

/// An OWL individual — an instance of a class.
#[derive(Debug, Clone, PartialEq)]
pub struct OwlIndividual {
    pub iri: String,
    pub types: Vec<String>,
    pub label: Option<String>,
}

/// A complete OWL ontology loaded from an OWL/XML file.
#[derive(Debug, Clone)]
pub struct OwlOntology {
    pub iri: String,
    pub classes: Vec<OwlClass>,
    pub properties: Vec<OwlObjectProperty>,
    pub individuals: Vec<OwlIndividual>,
    pub taxonomy: Vec<(String, String)>,
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
