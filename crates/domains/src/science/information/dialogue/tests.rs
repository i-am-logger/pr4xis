use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::validate::check_category_laws;

use super::ontology::*;

#[test]
fn dialogue_category_laws() {
    check_category_laws::<DialogueCategory>().unwrap();
}

#[test]
fn dialogue_has_10_concepts() {
    assert_eq!(DialogueConcept::variants().len(), 10);
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
    use praxis::category::Category;
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
    }
}
