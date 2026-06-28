#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::speech_act::*;

// =============================================================================
// Speech act tests
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn question_expects_response() {
    assert!(SpeechAct::Question.expects_response());
    assert!(SpeechAct::Request.expects_response());
    assert!(!SpeechAct::Assertion.expects_response());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn assertion_commits_to_truth() {
    assert!(SpeechAct::Assertion.commits_to_truth());
    assert!(SpeechAct::Promise.commits_to_truth());
    assert!(!SpeechAct::Question.commits_to_truth());
}

#[pr4xis::praxis_value(Verifiable)]
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
