#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::*;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Axiom, Ontology};

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn category_laws() {
    assert_category_laws::<OmvCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ontology_validates() {
    OmvOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ten_concepts() {
    assert_eq!(OmvConcept::variants().len(), 10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn artefact_has_formality_level_holds() {
    assert!(ArtefactHasFormalityLevel.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn artefact_has_analytics_holds() {
    assert!(ArtefactHasAnalytics.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
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

    pr4xis::register_praxis_value!(prop_identity_idempotent, Deterministic);
    pr4xis::register_praxis_value!(prop_self_identity, Deterministic);
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
