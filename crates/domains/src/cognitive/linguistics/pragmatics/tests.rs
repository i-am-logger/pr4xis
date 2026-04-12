use super::discourse::*;
use super::speech_act::*;
use crate::cognitive::linguistics::semantics::meaning::*;

// =============================================================================
// Speech act tests
// =============================================================================

#[test]
fn question_expects_response() {
    assert!(SpeechAct::Question.expects_response());
    assert!(SpeechAct::Request.expects_response());
    assert!(!SpeechAct::Assertion.expects_response());
}

#[test]
fn assertion_commits_to_truth() {
    assert!(SpeechAct::Assertion.commits_to_truth());
    assert!(SpeechAct::Promise.commits_to_truth());
    assert!(!SpeechAct::Question.commits_to_truth());
}

#[test]
fn intent_from_speech_act() {
    assert_eq!(
        Intent::from_speech_act(SpeechAct::Assertion),
        Intent::Inform
    );
    assert_eq!(Intent::from_speech_act(SpeechAct::Question), Intent::Inform);
    assert_eq!(Intent::from_speech_act(SpeechAct::Command), Intent::Direct);
    assert_eq!(
        Intent::from_speech_act(SpeechAct::Exclamation),
        Intent::Express
    );
}

// =============================================================================
// Discourse tests
// =============================================================================

#[test]
fn discourse_empty() {
    let d = Discourse::new();
    assert_eq!(d.turn_count(), 0);
    assert!(!d.expects_response());
    assert!(d.topic.is_none());
}

#[test]
fn discourse_add_assertion() {
    let mut d = Discourse::new();
    let meaning = MeaningRep::Atomic(SemanticProposition::new(
        Predicate::unary("run"),
        vec![EntityRef {
            name: "dog".into(),
            role: SemanticRole::Agent,
        }],
    ));
    d.add_turn(Turn::new(
        "the dog runs",
        SpeechAct::Assertion,
        Some(meaning),
    ));
    assert_eq!(d.turn_count(), 1);
    assert_eq!(d.topic, Some("run".into()));
    assert!(!d.expects_response());
}

#[test]
fn discourse_question_expects_response() {
    let mut d = Discourse::new();
    d.add_turn(Turn::new("is the dog running?", SpeechAct::Question, None));
    assert!(d.expects_response());
}

#[test]
fn discourse_topic_updates() {
    let mut d = Discourse::new();

    let run_meaning = MeaningRep::Atomic(SemanticProposition::new(
        Predicate::unary("run"),
        vec![EntityRef {
            name: "dog".into(),
            role: SemanticRole::Agent,
        }],
    ));
    d.add_turn(Turn::new(
        "the dog runs",
        SpeechAct::Assertion,
        Some(run_meaning),
    ));
    assert_eq!(d.topic, Some("run".into()));

    let see_meaning = MeaningRep::Atomic(SemanticProposition::new(
        Predicate::binary("see"),
        vec![
            EntityRef {
                name: "cat".into(),
                role: SemanticRole::Agent,
            },
            EntityRef {
                name: "bird".into(),
                role: SemanticRole::Patient,
            },
        ],
    ));
    d.add_turn(Turn::new(
        "the cat sees the bird",
        SpeechAct::Assertion,
        Some(see_meaning),
    ));
    assert_eq!(d.topic, Some("see".into()));
}

#[test]
fn discourse_multi_turn() {
    let mut d = Discourse::new();
    d.add_turn(Turn::new("hello", SpeechAct::Exclamation, None));
    d.add_turn(Turn::new("what is the weather?", SpeechAct::Question, None));
    d.add_turn(Turn::new("it is sunny", SpeechAct::Assertion, None));

    assert_eq!(d.turn_count(), 3);
    assert!(!d.expects_response()); // last turn was an assertion
}
