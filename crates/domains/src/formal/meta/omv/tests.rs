#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use pr4xis::category::Category;
use pr4xis::category::entity::Concept;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    check_category_laws::<OmvCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    OmvOntology::validate().unwrap();
}

#[test]
fn ten_concepts() {
    assert_eq!(OmvConcept::variants().len(), 10);
}

#[test]
fn artefact_has_formality_level() {
    assert!(ArtefactHasFormalityLevel.holds());
}

#[test]
fn artefact_has_analytics() {
    assert!(ArtefactHasAnalytics.holds());
}

#[test]
fn catalog_reaches_all() {
    assert!(CatalogReachesAll.holds());
}

#[test]
fn all_domain_axioms_hold() {
    for axiom in OmvOntology::domain_axioms() {
        assert!(axiom.holds(), "axiom failed: {}", axiom.description());
    }
}

#[test]
fn semantic_artefact_connects_to_all_metadata() {
    let m = OmvCategory::morphisms();
    let targets: Vec<OmvConcept> = vec![
        OmvConcept::FormalityLevel,
        OmvConcept::RepresentationParadigm,
        OmvConcept::Methodology,
        OmvConcept::DesignedTask,
        OmvConcept::Analytics,
        OmvConcept::Evaluation,
        OmvConcept::NaturalLanguage,
        OmvConcept::CompetencyQuestion,
    ];
    for target in &targets {
        assert!(
            m.iter()
                .any(|r| r.from == OmvConcept::SemanticArtefact && r.to == *target),
            "SemanticArtefact should connect to {:?}",
            target
        );
    }
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_omv() -> impl Strategy<Value = OmvConcept> {
        prop_oneof![
            Just(OmvConcept::SemanticArtefact),
            Just(OmvConcept::FormalityLevel),
            Just(OmvConcept::RepresentationParadigm),
            Just(OmvConcept::Methodology),
            Just(OmvConcept::DesignedTask),
            Just(OmvConcept::Analytics),
            Just(OmvConcept::Evaluation),
            Just(OmvConcept::Catalog),
            Just(OmvConcept::NaturalLanguage),
            Just(OmvConcept::CompetencyQuestion),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_omv()) {
            let id = OmvCategory::identity(&c);
            prop_assert_eq!(OmvCategory::compose(&id, &id), Some(id));
        }

        #[test]
        fn prop_self_morphisms(c in arb_omv()) {
            let m = OmvCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c
                && r.kind == OmvRelationKind::Identity);
            let has_composed = m.iter().any(|r| r.from == c && r.to == c
                && r.kind == OmvRelationKind::Composed);
            prop_assert!(has_identity);
            prop_assert!(has_composed);
        }
    }
}
