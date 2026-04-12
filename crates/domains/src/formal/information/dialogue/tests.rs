use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;

use super::ontology::*;

#[test]
fn dialogue_category_laws() {
    check_category_laws::<DialogueCategory>().unwrap();
}

#[test]
fn dialogue_has_15_concepts() {
    // 10 original + 5 new: QUD, CommonGround, Intention, GroundingAct, Repair
    assert_eq!(DialogueConcept::variants().len(), 15);
}

#[test]
fn participant_produces_utterance() {
    let m = DialogueCategory::morphisms();
    assert!(m.iter().any(|r| r.from == DialogueConcept::Participant
        && r.to == DialogueConcept::Utterance
        && r.kind == DialogueRelationKind::Produces));
}

#[test]
fn utterance_expresses_act() {
    let m = DialogueCategory::morphisms();
    assert!(m.iter().any(|r| r.from == DialogueConcept::Utterance
        && r.to == DialogueConcept::DialogueAct
        && r.kind == DialogueRelationKind::Expresses));
}

#[test]
fn understanding_leads_to_grounding() {
    let m = DialogueCategory::morphisms();
    assert!(m.iter().any(|r| r.from == DialogueConcept::Understanding
        && r.to == DialogueConcept::Grounding));
}

#[test]
fn turn_management_controls_participant() {
    let m = DialogueCategory::morphisms();
    assert!(m.iter().any(|r| r.from == DialogueConcept::TurnManagement
        && r.to == DialogueConcept::Participant
        && r.kind == DialogueRelationKind::Controls));
}

mod prop {
    use super::*;
    use pr4xis::category::Category;
    use proptest::prelude::*;

    fn arb_dialogue() -> impl Strategy<Value = DialogueConcept> {
        prop_oneof![
            Just(DialogueConcept::Utterance),
            Just(DialogueConcept::Participant),
            Just(DialogueConcept::DialogueAct),
            Just(DialogueConcept::DialogueState),
            Just(DialogueConcept::Topic),
            Just(DialogueConcept::History),
            Just(DialogueConcept::Understanding),
            Just(DialogueConcept::Generation),
            Just(DialogueConcept::TurnManagement),
            Just(DialogueConcept::Grounding),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_dialogue()) {
            let id = DialogueCategory::identity(&c);
            prop_assert_eq!(DialogueCategory::compose(&id, &id), Some(id));
        }

        /// Participant can reach DialogueState (via Utterance).
        #[test]
        fn prop_participant_reaches_state(_dummy in 0..1i32) {
            let m = DialogueCategory::morphisms();
            let reaches = m.iter().any(|r|
                r.from == DialogueConcept::Participant
                && r.to == DialogueConcept::DialogueState);
            prop_assert!(reaches);
        }

        /// Understanding leads to Grounding for all concepts that understand.
        #[test]
        fn prop_understanding_grounds(_dummy in 0..1i32) {
            let m = DialogueCategory::morphisms();
            let grounds = m.iter().any(|r|
                r.from == DialogueConcept::Understanding
                && r.to == DialogueConcept::Grounding);
            prop_assert!(grounds);
        }

        /// Every concept has both Identity and Composed self-morphisms.
        #[test]
        fn prop_self_morphisms(c in arb_dialogue()) {
            let m = DialogueCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c && r.kind == DialogueRelationKind::Identity);
            let has_composed = m.iter().any(|r| r.from == c && r.to == c && r.kind == DialogueRelationKind::Composed);
            prop_assert!(has_identity);
            prop_assert!(has_composed);
        }
    }
}
