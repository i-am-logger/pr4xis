use super::ontology::*;
use pr4xis::category::Category;
use pr4xis::category::entity::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    assert_category_laws::<LemonCategory>();
}

#[test]
fn ontology_validates() {
    LemonOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[test]
fn six_concepts() {
    assert_eq!(LemonConcept::variants().len(), 6);
}

#[test]
fn denotes_property_chain_holds() {
    assert!(DenotesIsPropertyChain.verify().is_ok());
}

#[test]
fn canonical_form_is_functional() {
    assert!(CanonicalFormIsFunctional.verify().is_ok());
}

#[test]
fn reference_is_functional() {
    assert!(ReferenceIsFunctional.verify().is_ok());
}

#[test]
fn three_way_bridge_exists() {
    let m = LemonCategory::morphisms();
    assert!(
        m.iter().any(|r| r.from == LemonConcept::LexicalEntry
            && r.to == LemonConcept::LexicalSense
            && r.kind == LemonRelationKind::Sense),
        "missing: LexicalEntry → LexicalSense"
    );
    assert!(
        m.iter().any(|r| r.from == LemonConcept::LexicalSense
            && r.to == LemonConcept::OntologyReference
            && r.kind == LemonRelationKind::Reference),
        "missing: LexicalSense → OntologyReference"
    );
    assert!(
        m.iter().any(|r| r.from == LemonConcept::LexicalEntry
            && r.to == LemonConcept::OntologyReference
            && r.kind == LemonRelationKind::Denotes),
        "missing: LexicalEntry → OntologyReference (denotes)"
    );
}

#[test]
fn lexicon_contains_entries() {
    let m = LemonCategory::morphisms();
    assert!(m.iter().any(|r| r.from == LemonConcept::Lexicon
        && r.to == LemonConcept::LexicalEntry
        && r.kind == LemonRelationKind::Entry));
}

#[test]
fn lexicon_reaches_all_concepts() {
    // The Lemon model (W3C OntoLex 2016) is rooted at Lexicon; every
    // other concept is reachable through the heterogeneous-kind edges
    // (Entry / Form / Sense / Reference / etc.). Per #166 closure across
    // heterogeneous kinds isn't materialized as a single morphism — walk
    // the graph.
    use pr4xis::category::Arrow;
    use std::collections::{HashSet, VecDeque};
    let ms = LemonCategory::morphisms();
    for concept in LemonConcept::variants() {
        let mut visited: HashSet<LemonConcept> = HashSet::new();
        let mut queue: VecDeque<LemonConcept> = VecDeque::new();
        queue.push_back(LemonConcept::Lexicon);
        let mut reaches = LemonConcept::Lexicon == concept;
        while let Some(n) = queue.pop_front() {
            if n == concept {
                reaches = true;
                break;
            }
            if !visited.insert(n) {
                continue;
            }
            for m in ms.iter().filter(|m| m.source() == n) {
                queue.push_back(m.target());
            }
        }
        assert!(reaches, "Lexicon should reach {:?}", concept);
    }
}

#[test]
fn all_domain_axioms_hold() {
    for axiom in LemonOntology::axioms() {
        match axiom.verify() {
            Ok(_) => {}
            Err(c) => panic!("axiom failed: {}", c.meta().description.as_str()),
        }
    }
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_lemon() -> impl Strategy<Value = LemonConcept> {
        prop_oneof![
            Just(LemonConcept::LexicalEntry),
            Just(LemonConcept::Form),
            Just(LemonConcept::LexicalSense),
            Just(LemonConcept::LexicalConcept),
            Just(LemonConcept::Lexicon),
            Just(LemonConcept::OntologyReference),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_lemon()) {
            let id = LemonCategory::identity(&c);
            prop_assert_eq!(LemonCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has an Identity self-morphism. Per #166 the
        /// auto-generated kind no longer emits `Composed` self-loops;
        /// composition is partial.
        #[test]
        fn prop_self_morphisms(c in arb_lemon()) {
            let m = LemonCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c
                && r.kind == LemonRelationKind::Identity);
            prop_assert!(has_identity);
        }
    }
}
