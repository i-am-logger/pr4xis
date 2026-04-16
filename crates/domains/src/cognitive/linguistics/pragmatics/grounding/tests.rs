use super::ontology::*;
use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

#[test]
fn category_laws() {
    check_category_laws::<GroundingCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    GroundingOntology::validate().unwrap();
}

#[test]
fn nineteen_concepts() {
    assert_eq!(GroundingConcept::variants().len(), 19);
}

#[test]
fn all_acts_classified() {
    assert!(AllActsClassified.holds());
}

#[test]
fn presentation_has_consequence() {
    assert!(PresentationHasConsequence.holds());
}

#[test]
fn common_ground_has_contributions() {
    use pr4xis::ontology::reasoning::mereology::MereologyDef;
    let parts = GroundingMereology::relations();
    assert!(parts.iter().any(|(whole, part)| {
        *whole == GroundingConcept::CommonGround && *part == GroundingConcept::Contribution
    }));
}

#[test]
fn info_state_has_gameboard() {
    use pr4xis::ontology::reasoning::mereology::MereologyDef;
    let parts = GroundingMereology::relations();
    assert!(parts.iter().any(|(whole, part)| {
        *whole == GroundingConcept::InfoState && *part == GroundingConcept::DialogueGameBoard
    }));
}

#[test]
fn gameboard_has_qud() {
    use pr4xis::ontology::reasoning::mereology::MereologyDef;
    let parts = GroundingMereology::relations();
    assert!(parts.iter().any(|(whole, part)| {
        *whole == GroundingConcept::DialogueGameBoard && *part == GroundingConcept::MaxQUD
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
