#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    assert_category_laws::<OmvCategory>();
}

#[test]
fn ontology_validates() {
    OmvOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[test]
fn ten_concepts() {
    assert_eq!(OmvConcept::variants().len(), 10);
}

#[test]
fn artefact_has_formality_level_holds() {
    assert!(ArtefactHasFormalityLevel.verify().is_ok());
}

#[test]
fn artefact_has_analytics_holds() {
    assert!(ArtefactHasAnalytics.verify().is_ok());
}

#[test]
fn semantic_artefact_connects_to_all_metadata() {
    let m = OmvCategory::morphisms();
    for target in [
        OmvConcept::FormalityLevel,
        OmvConcept::RepresentationParadigm,
        OmvConcept::Methodology,
        OmvConcept::DesignedTask,
        OmvConcept::Analytics,
        OmvConcept::Evaluation,
        OmvConcept::NaturalLanguage,
        OmvConcept::CompetencyQuestion,
    ] {
        assert!(
            m.iter()
                .any(|r| r.source() == OmvConcept::SemanticArtefact && r.target() == target)
        );
    }
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_omv() -> impl Strategy<Value = OmvConcept> {
        proptest::sample::select(OmvConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_omv()) {
            let id = OmvCategory::identity(&c);
            prop_assert_eq!(OmvCategory::compose(&id, &id), Some(id));
        }

        #[test]
        fn prop_self_identity(c in arb_omv()) {
            let m = OmvCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == c
                && r.target() == c
                && r.kind() == OmvRelationKind::Identity));
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in OmvCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in OmvOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
}
