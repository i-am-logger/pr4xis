use super::ontology::*;
use pr4xis::category::entity::Concept;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category};
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    assert_category_laws::<GroundingCategory>();
}

#[test]
fn ontology_validates() {
    GroundingOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[test]
fn nineteen_concepts() {
    assert_eq!(GroundingConcept::variants().len(), 19);
}

#[test]
fn all_acts_classified() {
    assert!(AllActsClassified.verify().is_ok());
}

#[test]
fn presentation_has_consequence() {
    assert!(PresentationHasConsequence.verify().is_ok());
}

#[test]
fn common_ground_has_contributions() {
    let parts: Vec<_> = GroundingCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == GroundingRelationKind::Parthood)
        .collect();
    assert!(parts.iter().any(|m| {
        m.source() == GroundingConcept::CommonGround && m.target() == GroundingConcept::Contribution
    }));
}

#[test]
fn info_state_has_gameboard() {
    let parts: Vec<_> = GroundingCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == GroundingRelationKind::Parthood)
        .collect();
    assert!(parts.iter().any(|m| {
        m.source() == GroundingConcept::InfoState
            && m.target() == GroundingConcept::DialogueGameBoard
    }));
}

#[test]
fn gameboard_has_qud() {
    let parts: Vec<_> = GroundingCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == GroundingRelationKind::Parthood)
        .collect();
    assert!(parts.iter().any(|m| {
        m.source() == GroundingConcept::DialogueGameBoard && m.target() == GroundingConcept::MaxQUD
    }));
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_grounding() -> impl Strategy<Value = GroundingConcept> {
        (0..19usize)
            .prop_map(|i| GroundingConcept::variants()[i % GroundingConcept::variants().len()])
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_grounding()) {
            let id = GroundingCategory::identity(&c);
            prop_assert_eq!(GroundingCategory::compose(&id, &id), Some(id));
        }
    }
}
