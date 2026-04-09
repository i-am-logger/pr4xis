use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

// RDF 1.1 Concepts and Abstract Syntax — W3C Recommendation (2014)
// https://www.w3.org/TR/rdf11-concepts/
//
// RDF Schema 1.1 — W3C Recommendation (2014)
// https://www.w3.org/TR/rdf-schema/
//
// RDF is a graph data model where everything is a triple:
//   (subject, predicate, object)
//
// Subjects are IRIs or blank nodes. Predicates are IRIs.
// Objects are IRIs, blank nodes, or literals.
// An RDF graph is a set of triples — no ordering, no duplicates.
//
// RDFS extends RDF with a vocabulary for describing classes and properties:
//   rdfs:Class, rdfs:subClassOf, rdf:type, rdfs:domain, rdfs:range

/// The kinds of nodes that can appear in an RDF graph.
/// From W3C RDF 1.1 Concepts §1.2: "the abstract syntax of RDF."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RdfNodeKind {
    /// An IRI-identified resource (W3C RDF 1.1 §3.1).
    IriResource,
    /// An anonymous node with local scope (W3C RDF 1.1 §3.4).
    BlankNode,
    /// A string with optional language tag (W3C RDF 1.1 §3.3).
    PlainLiteral,
    /// A string + datatype IRI, e.g. "42"^^xsd:integer (W3C RDF 1.1 §3.3).
    TypedLiteral,
    /// A reified triple: rdf:Statement (W3C RDF 1.1 §4).
    Statement,
    /// rdfs:Class — a set of resources (W3C RDFS §2.1).
    Class,
    /// rdf:Property — a relation between resources (W3C RDFS §2.2).
    Property,
    /// rdfs:Datatype — a class of literal values (W3C RDFS §2.4).
    Datatype,
    /// rdf:nil — the empty list (W3C RDF 1.1 §5.1).
    Nil,
    /// rdf:List — a linked list node (W3C RDF 1.1 §5.1).
    List,
}

impl Entity for RdfNodeKind {
    fn variants() -> Vec<Self> {
        vec![
            Self::IriResource,
            Self::BlankNode,
            Self::PlainLiteral,
            Self::TypedLiteral,
            Self::Statement,
            Self::Class,
            Self::Property,
            Self::Datatype,
            Self::Nil,
            Self::List,
        ]
    }
}

impl RdfNodeKind {
    /// Can this node kind appear as a subject in an RDF triple? (W3C RDF 1.1 §3)
    pub fn can_be_subject(&self) -> bool {
        !matches!(self, Self::PlainLiteral | Self::TypedLiteral | Self::Nil)
    }

    /// Can this node kind appear as an object in an RDF triple?
    pub fn can_be_object(&self) -> bool {
        true // all node kinds can be objects
    }

    /// Is this a literal node kind?
    pub fn is_literal(&self) -> bool {
        matches!(self, Self::PlainLiteral | Self::TypedLiteral)
    }

    /// Is this a resource (non-literal) node kind?
    pub fn is_resource(&self) -> bool {
        !self.is_literal()
    }
}

/// A morphism in the RDF category: one node kind can relate to another.
/// This captures the "abstract syntax" constraints — which combinations
/// of node kinds can appear together in a triple.
///
/// Follows the XmlContains pattern: a directed edge between node kinds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RdfRelation {
    pub source: RdfNodeKind,
    pub target: RdfNodeKind,
}

impl Relationship for RdfRelation {
    type Object = RdfNodeKind;
    fn source(&self) -> RdfNodeKind {
        self.source
    }
    fn target(&self) -> RdfNodeKind {
        self.target
    }
}

/// The RDF category.
/// Objects are node kinds. Morphisms are the legal connections
/// between node kinds, per W3C RDF 1.1 abstract syntax.
pub struct RdfCategory;

impl Category for RdfCategory {
    type Object = RdfNodeKind;
    type Morphism = RdfRelation;

    fn identity(obj: &RdfNodeKind) -> RdfRelation {
        RdfRelation {
            source: *obj,
            target: *obj,
        }
    }

    fn compose(f: &RdfRelation, g: &RdfRelation) -> Option<RdfRelation> {
        if f.target != g.source {
            return None;
        }
        if f.source == f.target {
            return Some(g.clone());
        }
        if g.source == g.target {
            return Some(f.clone());
        }
        Some(RdfRelation {
            source: f.source,
            target: g.target,
        })
    }

    fn morphisms() -> Vec<RdfRelation> {
        use RdfNodeKind::*;
        let mut m = Vec::new();

        // Identities
        for n in RdfNodeKind::variants() {
            m.push(RdfRelation {
                source: n,
                target: n,
            });
        }

        // Subject → predicate: resources can assert properties
        // "subject (resource) --predicate--> object (anything)"
        let subjects = [
            IriResource,
            BlankNode,
            Class,
            Property,
            Datatype,
            Statement,
            List,
        ];
        let objects = [
            IriResource,
            BlankNode,
            PlainLiteral,
            TypedLiteral,
            Class,
            Property,
            Datatype,
            Nil,
            List,
            Statement,
        ];

        for &s in &subjects {
            for &o in &objects {
                if s != o {
                    m.push(RdfRelation {
                        source: s,
                        target: o,
                    });
                }
            }
        }

        m
    }
}

// =============================================================================
// RDFS built-in taxonomy — W3C RDFS §2
// =============================================================================

/// The RDFS built-in class hierarchy.
/// From W3C RDFS 1.1 §2: rdfs:Resource is the superclass of everything.
pub fn rdfs_taxonomy() -> Vec<(RdfNodeKind, RdfNodeKind)> {
    use RdfNodeKind::*;
    vec![
        // Everything is a Resource (§2.1)
        (Class, IriResource),
        (Property, IriResource),
        (Datatype, IriResource),
        (Statement, IriResource),
        (List, IriResource),
        (Nil, List),
        // Datatype is a subclass of Class (§2.4)
        (Datatype, Class),
        // Literals are Resources in RDFS 2.1
        (PlainLiteral, IriResource),
        (TypedLiteral, IriResource),
    ]
}

// =============================================================================
// Well-known RDF/RDFS IRIs — W3C RDF 1.1 and RDFS 1.1
// =============================================================================

/// Well-known RDF/RDFS IRIs — the vocabulary that gives meaning to RDF triples.
/// Each IRI is the identity of a concept from the W3C specs.
pub struct RdfVocabulary;

impl RdfVocabulary {
    // RDF namespace: http://www.w3.org/1999/02/22-rdf-syntax-ns#
    pub const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
    pub const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
    pub const RDF_PROPERTY: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#Property";
    pub const RDF_STATEMENT: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#Statement";
    pub const RDF_SUBJECT: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#subject";
    pub const RDF_PREDICATE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate";
    pub const RDF_OBJECT: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#object";
    pub const RDF_LIST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#List";
    pub const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
    pub const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
    pub const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";

    // RDFS namespace: http://www.w3.org/2000/01/rdf-schema#
    pub const RDFS_NS: &str = "http://www.w3.org/2000/01/rdf-schema#";
    pub const RDFS_CLASS: &str = "http://www.w3.org/2000/01/rdf-schema#Class";
    pub const RDFS_RESOURCE: &str = "http://www.w3.org/2000/01/rdf-schema#Resource";
    pub const RDFS_LITERAL: &str = "http://www.w3.org/2000/01/rdf-schema#Literal";
    pub const RDFS_DATATYPE: &str = "http://www.w3.org/2000/01/rdf-schema#Datatype";
    pub const RDFS_SUB_CLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
    pub const RDFS_SUB_PROPERTY_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subPropertyOf";
    pub const RDFS_DOMAIN: &str = "http://www.w3.org/2000/01/rdf-schema#domain";
    pub const RDFS_RANGE: &str = "http://www.w3.org/2000/01/rdf-schema#range";
    pub const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
    pub const RDFS_COMMENT: &str = "http://www.w3.org/2000/01/rdf-schema#comment";
    pub const RDFS_SEE_ALSO: &str = "http://www.w3.org/2000/01/rdf-schema#seeAlso";
    pub const RDFS_IS_DEFINED_BY: &str = "http://www.w3.org/2000/01/rdf-schema#isDefinedBy";

    /// Resolve a namespace prefix + local name to a full IRI.
    pub fn resolve(prefix: &str, local: &str) -> Option<String> {
        let ns = match prefix {
            "rdf" => Some(Self::RDF_NS),
            "rdfs" => Some(Self::RDFS_NS),
            _ => None,
        }?;
        Some(format!("{}{}", ns, local))
    }
}

// =============================================================================
// Axioms — structural invariants from W3C specs
// =============================================================================

/// W3C axiom: Literals cannot be subjects (RDF 1.1 §3).
pub struct LiteralsCannotBeSubjects;

impl praxis::logic::Axiom for LiteralsCannotBeSubjects {
    fn description(&self) -> &str {
        "RDF literals cannot appear in subject position (W3C RDF 1.1 §3)"
    }

    fn holds(&self) -> bool {
        !RdfNodeKind::PlainLiteral.can_be_subject() && !RdfNodeKind::TypedLiteral.can_be_subject()
    }
}

/// W3C axiom: Predicates must be IRIs — they are Properties (RDF 1.1 §3).
pub struct PredicatesMustBeProperties;

impl praxis::logic::Axiom for PredicatesMustBeProperties {
    fn description(&self) -> &str {
        "RDF predicates must be IRI references (rdf:Property), not blank nodes or literals (W3C RDF 1.1 §3)"
    }

    fn holds(&self) -> bool {
        // Properties are IRIs, and only IRI-identified things can be predicates
        RdfNodeKind::Property.is_resource()
    }
}
