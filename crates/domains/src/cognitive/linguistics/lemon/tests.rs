use super::ontology::*;
use pr4xis::category::Category;
use pr4xis::category::entity::Concept;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    check_category_laws::<LemonCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    LemonOntology::validate().unwrap();
}

#[test]
fn six_concepts() {
    assert_eq!(LemonConcept::variants().len(), 6);
}

#[test]
fn denotes_property_chain_holds() {
    assert!(DenotesIsPropertyChain.holds());
}

#[test]
fn canonical_form_is_functional() {
    assert!(CanonicalFormIsFunctional.holds());
}

#[test]
fn reference_is_functional() {
    assert!(ReferenceIsFunctional.holds());
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
    let m = LemonCategory::morphisms();
    for concept in LemonConcept::variants() {
        assert!(
            m.iter()
                .any(|r| r.from == LemonConcept::Lexicon && r.to == concept),
            "Lexicon should reach {:?}",
            concept
        );
    }
}

#[test]
fn all_domain_axioms_hold() {
    for axiom in LemonOntology::domain_axioms() {
        assert!(axiom.holds(), "axiom failed: {}", axiom.description());
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

        #[test]
        fn prop_self_morphisms(c in arb_lemon()) {
            let m = LemonCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c
                && r.kind == LemonRelationKind::Identity);
            let has_composed = m.iter().any(|r| r.from == c && r.to == c
                && r.kind == LemonRelationKind::Composed);
            prop_assert!(has_identity);
            prop_assert!(has_composed);
        }
    }
}
