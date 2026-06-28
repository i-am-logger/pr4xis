use super::ontology::*;
use pr4xis::category::entity::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category};
use pr4xis::ontology::{Axiom, Ontology};

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn category_laws() {
    assert_category_laws::<GroundingCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ontology_validates() {
    GroundingOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn nineteen_concepts() {
    assert_eq!(GroundingConcept::variants().len(), 19);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn all_acts_classified() {
    assert!(AllActsClassified.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn presentation_has_consequence() {
    assert!(PresentationHasConsequence.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn common_ground_has_contributions() {
    let parts: Vec<_> = GroundingCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == GroundingRelationKind::Parthood)
        .collect();
    // part→whole (BFO:0000050): a Contribution is PART of the CommonGround.
    assert!(parts.iter().any(|m| {
        m.source() == GroundingConcept::Contribution && m.target() == GroundingConcept::CommonGround
    }));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn info_state_has_gameboard() {
    let parts: Vec<_> = GroundingCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == GroundingRelationKind::Parthood)
        .collect();
    // part→whole: the DialogueGameBoard is PART of the InfoState.
    assert!(parts.iter().any(|m| {
        m.source() == GroundingConcept::DialogueGameBoard
            && m.target() == GroundingConcept::InfoState
    }));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn gameboard_has_qud() {
    let parts: Vec<_> = GroundingCategory::morphisms()
        .into_iter()
        .filter(|m| m.kind() == GroundingRelationKind::Parthood)
        .collect();
    // part→whole: the MaxQUD is PART of the DialogueGameBoard.
    assert!(parts.iter().any(|m| {
        m.source() == GroundingConcept::MaxQUD && m.target() == GroundingConcept::DialogueGameBoard
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

    pr4xis::register_praxis_value!(prop_identity_idempotent, Deterministic);
}
