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

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Lemon",
    source: "W3C Ontolex (2016); McCrae et al. (2012, 2017)",

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
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
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
        if has_sense && has_ref && has_denotes {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DenotesIsPropertyChain",
        "denotes = sense ∘ reference (W3C Ontolex §3.4)",
        "W3C Lexicon Model for Ontologies (Ontolex-Lemon) (2016) §3.4"
    );
}
pr4xis::register_axiom!(
    DenotesIsPropertyChain,
    "W3C Lexicon Model for Ontologies (Ontolex-Lemon) (2016) §3.4"
);

/// canonicalForm is functional (W3C Ontolex §3.2).
#[derive(Debug)]
pub struct CanonicalFormIsFunctional;

impl Axiom for CanonicalFormIsFunctional {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let m = LemonCategory::morphisms();
        let count = m
            .iter()
            .filter(|r| {
                r.from == LemonConcept::LexicalEntry
                    && r.to == LemonConcept::Form
                    && r.kind == LemonRelationKind::CanonicalForm
            })
            .count();
        if count <= 1 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CanonicalFormIsFunctional",
        "canonicalForm is functional: at most one per entry (W3C Ontolex §3.2)",
        "W3C Lexicon Model for Ontologies (Ontolex-Lemon) (2016) §3.2"
    );
}
pr4xis::register_axiom!(
    CanonicalFormIsFunctional,
    "W3C Lexicon Model for Ontologies (Ontolex-Lemon) (2016) §3.2"
);

/// reference is functional (W3C Ontolex §3.4).
#[derive(Debug)]
pub struct ReferenceIsFunctional;

impl Axiom for ReferenceIsFunctional {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let m = LemonCategory::morphisms();
        let count = m
            .iter()
            .filter(|r| {
                r.from == LemonConcept::LexicalSense
                    && r.to == LemonConcept::OntologyReference
                    && r.kind == LemonRelationKind::Reference
            })
            .count();
        if count <= 1 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ReferenceIsFunctional",
        "reference is functional: sense → exactly one ontology entity (W3C Ontolex §3.4)",
        "W3C Lexicon Model for Ontologies (Ontolex-Lemon) (2016) §3.4"
    );
}
pr4xis::register_axiom!(
    ReferenceIsFunctional,
    "W3C Lexicon Model for Ontologies (Ontolex-Lemon) (2016) §3.4"
);

/// The domain-conditioned predominant-sense order is a STRICT PARTIAL ORDER.
///
/// "Elevation" — Koeling, McCarthy & Carroll (2005): the predominant sense of a
/// polysemous word is domain-dependent — is realised in praxis as a per-query-
/// domain ordering over an entry's senses
/// ([`Sense::salience_in`](super::lexicon::Sense::salience_in)). For the lexicon
/// to have a well-defined predominant sense, that order
/// `a ≺_d b ⟺ salience_d(a) > salience_d(b)` must be a strict partial order —
/// irreflexive, asymmetric, transitive — under every query domain. Since it is
/// induced by a total grading into a totally-ordered codomain it is always such
/// an order; this axiom discharges the three laws over a fixture entry carrying
/// a general sense plus two domain-specific senses (the "person" case: general,
/// legal, economics), spanning every distinct salience value.
#[derive(Debug)]
pub struct SenseOrderIsStrictPartialOrder;

impl Axiom for SenseOrderIsStrictPartialOrder {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use super::lexicon::{ConceptRef, Sense};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};

        // One entry's senses across domains — the elevation fixture.
        let senses = [
            Sense {
                reference: ConceptRef {
                    ontology: "english_wordnet".to_string(),
                    concept: "person.n.01".to_string(),
                },
                domain: None,
            },
            Sense {
                reference: ConceptRef {
                    ontology: "us_legal_lexicon".to_string(),
                    concept: "person".to_string(),
                },
                domain: Some("legal".to_string()),
            },
            Sense {
                reference: ConceptRef {
                    ontology: "economics".to_string(),
                    concept: "person".to_string(),
                },
                domain: Some("economics".to_string()),
            },
        ];
        let queries: [Option<&str>; 3] = [None, Some("legal"), Some("economics")];
        let precedes = |a: &Sense, b: &Sense, d: Option<&str>| a.salience_in(d) > b.salience_in(d);

        for d in queries {
            for a in &senses {
                // Irreflexive: ¬(a ≺ a).
                if precedes(a, a, d) {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
                for b in &senses {
                    // Asymmetric: a ≺ b ⟹ ¬(b ≺ a).
                    if precedes(a, b, d) && precedes(b, a, d) {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                    // Transitive: a ≺ b ∧ b ≺ c ⟹ a ≺ c.
                    for c in &senses {
                        if precedes(a, b, d) && precedes(b, c, d) && !precedes(a, c, d) {
                            return Err(Box::new(SimpleCounterexample::new(self.meta())));
                        }
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SenseOrderIsStrictPartialOrder",
        "the domain-conditioned predominant-sense order over an entry's senses is a strict partial order (irreflexive, asymmetric, transitive)",
        "Koeling, McCarthy & Carroll (2005) Domain-Specific Sense Distributions and Predominant Sense Acquisition, HLT/EMNLP; W3C Ontolex-Lemon (2016) one-entry-many-senses"
    );
}
pr4xis::register_axiom!(
    SenseOrderIsStrictPartialOrder,
    "Koeling, McCarthy & Carroll (2005) HLT/EMNLP; W3C Ontolex-Lemon (2016)"
);

impl Ontology for LemonOntology {
    type Cat = LemonCategory;
    type Qual = IsCoreConcept;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(DenotesIsPropertyChain));
        axioms.push(Box::new(CanonicalFormIsFunctional));
        axioms.push(Box::new(ReferenceIsFunctional));
        axioms.push(Box::new(SenseOrderIsStrictPartialOrder));
        axioms
    }
}
