// Ontolex-Lemon — the ontology-lexicon interface.
//
// Separates ontological concepts from their linguistic realizations.
// A LexicalEntry has Forms (written/phonological) and Senses (connections
// to ontology concepts). A Lexicon collects entries for one language.
//
// The key insight: labels are NOT properties of ontology concepts.
// Instead, a LexicalEntry in a Lexicon points to the concept via a
// LexicalSense. Multiple lexicons (English, Hebrew) point to the same
// concept — multilinguality without touching the ontology.
//
// Source: W3C Lexicon Model for Ontologies (2016);
//         McCrae et al. (2012, 2017)

use pr4xis::category::Category;
use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Concepts in the Ontolex-Lemon ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum LemonConcept {
    /// ontolex:LexicalEntry — unit of analysis: forms + senses.
    LexicalEntry,
    /// ontolex:Form — one grammatical realization of an entry.
    Form,
    /// ontolex:LexicalSense — the bridge between entry and ontology.
    LexicalSense,
    /// ontolex:LexicalConcept — mental abstraction (skos:Concept subclass).
    LexicalConcept,
    /// lime:Lexicon — entries for one language.
    Lexicon,
    /// The ontology entity being described (target of reference).
    OntologyReference,
}

define_ontology! {
    pub LemonOntology for LemonCategory {
        concepts: LemonConcept,
        relation: LemonRelation,
        kind: LemonRelationKind,
        kinds: [
            /// ontolex:canonicalForm — preferred lemma form (functional).
            CanonicalForm,
            /// ontolex:otherForm — non-canonical inflected form.
            OtherForm,
            /// ontolex:sense — entry's meaning w.r.t. an ontology element.
            Sense,
            /// ontolex:reference — sense points to ontology entity (functional).
            Reference,
            /// ontolex:evokes — entry activates a mental concept.
            Evokes,
            /// ontolex:isConceptOf — concept linked to ontology entity.
            IsConceptOf,
            /// lime:entry — lexicon contains entry.
            Entry,
            /// ontolex:denotes — shortcut: sense ∘ reference.
            Denotes,
            /// ontolex:lexicalizedSense — concept lexicalized by this sense.
            LexicalizedSense,
        ],
        edges: [
            (LexicalEntry, Form, CanonicalForm),
            (LexicalEntry, Form, OtherForm),
            (LexicalEntry, LexicalSense, Sense),
            (LexicalSense, OntologyReference, Reference),
            (LexicalEntry, OntologyReference, Denotes),
            (LexicalEntry, LexicalConcept, Evokes),
            (LexicalConcept, OntologyReference, IsConceptOf),
            (LexicalConcept, LexicalSense, LexicalizedSense),
            (Lexicon, LexicalEntry, Entry),
        ],
        composed: [
            (Lexicon, Form),
            (Lexicon, LexicalSense),
            (Lexicon, LexicalConcept),
            (Lexicon, OntologyReference),
        ],

        being: SocialObject,
        source: "W3C Ontolex (2016); McCrae et al. (2012, 2017)",
    }
}

/// Whether a concept is core (ontolex:) vs. metadata (lime:).
#[derive(Debug, Clone)]
pub struct IsCoreConcept;

impl Quality for IsCoreConcept {
    type Individual = LemonConcept;
    type Value = bool;

    fn get(&self, individual: &LemonConcept) -> Option<bool> {
        Some(!matches!(individual, LemonConcept::Lexicon))
    }
}

/// denotes = sense ∘ reference (W3C Ontolex §3.4).
#[derive(Debug)]
pub struct DenotesIsPropertyChain;

impl Axiom for DenotesIsPropertyChain {
    fn description(&self) -> &str {
        "denotes = sense ∘ reference (W3C Ontolex §3.4)"
    }
    fn holds(&self) -> bool {
        let m = LemonCategory::morphisms();
        let has_sense = m.iter().any(|r| {
            r.from == LemonConcept::LexicalEntry
                && r.to == LemonConcept::LexicalSense
                && r.kind == LemonRelationKind::Sense
        });
        let has_ref = m.iter().any(|r| {
            r.from == LemonConcept::LexicalSense
                && r.to == LemonConcept::OntologyReference
                && r.kind == LemonRelationKind::Reference
        });
        let has_denotes = m.iter().any(|r| {
            r.from == LemonConcept::LexicalEntry
                && r.to == LemonConcept::OntologyReference
                && r.kind == LemonRelationKind::Denotes
        });
        has_sense && has_ref && has_denotes
    }
}

/// canonicalForm is functional (W3C Ontolex §3.2).
#[derive(Debug)]
pub struct CanonicalFormIsFunctional;

impl Axiom for CanonicalFormIsFunctional {
    fn description(&self) -> &str {
        "canonicalForm is functional: at most one per entry (W3C Ontolex §3.2)"
    }
    fn holds(&self) -> bool {
        let m = LemonCategory::morphisms();
        m.iter()
            .filter(|r| {
                r.from == LemonConcept::LexicalEntry
                    && r.to == LemonConcept::Form
                    && r.kind == LemonRelationKind::CanonicalForm
            })
            .count()
            <= 1
    }
}

/// reference is functional (W3C Ontolex §3.4).
#[derive(Debug)]
pub struct ReferenceIsFunctional;

impl Axiom for ReferenceIsFunctional {
    fn description(&self) -> &str {
        "reference is functional: sense → exactly one ontology entity (W3C Ontolex §3.4)"
    }
    fn holds(&self) -> bool {
        let m = LemonCategory::morphisms();
        m.iter()
            .filter(|r| {
                r.from == LemonConcept::LexicalSense
                    && r.to == LemonConcept::OntologyReference
                    && r.kind == LemonRelationKind::Reference
            })
            .count()
            <= 1
    }
}

impl Ontology for LemonOntology {
    type Cat = LemonCategory;
    type Qual = IsCoreConcept;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        LemonOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(DenotesIsPropertyChain),
            Box::new(CanonicalFormIsFunctional),
            Box::new(ReferenceIsFunctional),
        ]
    }
}
