use pr4xis::category::Category;
use pr4xis::category::entity::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;

use super::ontology::*;

#[test]
fn dialogue_category_laws() {
    assert_category_laws::<DialogueCategory>();
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

        /// Participant can reach DialogueState (via Utterance). Per #166
        /// the heterogeneous-kind chain (Produces then Updates) isn't a
        /// direct morphism — walk the graph.
        #[test]
        fn prop_participant_reaches_state(_dummy in 0..1i32) {
            use std::collections::{HashSet, VecDeque};
            use pr4xis::category::Arrow;
            let ms = DialogueCategory::morphisms();
            let mut visited: HashSet<DialogueConcept> = HashSet::new();
            let mut queue: VecDeque<DialogueConcept> = VecDeque::new();
            queue.push_back(DialogueConcept::Participant);
            let mut reaches = false;
            while let Some(n) = queue.pop_front() {
                if n == DialogueConcept::DialogueState {
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

        /// Every concept has an Identity self-morphism. Per #166 the
        /// auto-generated kind no longer emits `Composed` self-loops;
        /// transitive composition is partial.
        #[test]
        fn prop_self_morphisms(c in arb_dialogue()) {
            let m = DialogueCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c && r.kind == DialogueRelationKind::Identity);
            prop_assert!(has_identity);
        }
    }
}
