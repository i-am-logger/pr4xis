//! Ontolex-Lemon — the ontology-lexicon interface.
//!
//! Separates ontological concepts from their linguistic realizations.
//! A LexicalEntry has Forms (written/phonological) and Senses (connections
//! to ontology concepts). A Lexicon collects entries for one language.
//!
//! The key insight: labels are NOT properties of ontology concepts.
//! Instead, a LexicalEntry in a Lexicon points to the concept via a
//! LexicalSense. Multiple lexicons (English, Hebrew) point to the same
//! concept — multilinguality without touching the ontology.
//!
//! Source: W3C Lexicon Model for Ontologies (2016);
//!         McCrae et al. (2012, 2017)

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Lemon",
    source: "W3C Ontolex (2016); McCrae et al. (2012, 2017)",
    being: SocialObject,

    concepts: [LexicalEntry, Form, LexicalSense, LexicalConcept, Lexicon, OntologyReference],

    labels: {
        LexicalEntry: ("en", "Lexical entry", "ontolex:LexicalEntry — unit of analysis: forms + senses."),
        Form: ("en", "Form", "ontolex:Form — one grammatical realization of an entry."),
        LexicalSense: ("en", "Lexical sense", "ontolex:LexicalSense — the bridge between entry and ontology."),
        LexicalConcept: ("en", "Lexical concept", "ontolex:LexicalConcept — mental abstraction (skos:Concept subclass)."),
        Lexicon: ("en", "Lexicon", "lime:Lexicon — entries for one language."),
        OntologyReference: ("en", "Ontology reference", "The ontology entity being described (target of reference)."),
    },

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
pr4xis::register_axiom!(DenotesIsPropertyChain);

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
pr4xis::register_axiom!(CanonicalFormIsFunctional);

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
pr4xis::register_axiom!(ReferenceIsFunctional);

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
