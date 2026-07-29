//! Multi-turn conversation test corpus (task #18) — real, hand-authored
//! dialogues that drive [`pr4xis_chat::ChatSession`]'s slot-filling state
//! machine (task #17) through the real production `English`/
//! `ComposedReasoner` pipeline, no stubs, matching the same discipline as
//! task #28's own end-to-end witnesses in `pr4xis_chat::capability`.
//!
//! Unlike the harvested 4617-question single-turn corpus
//! ([`crate::caregiver`]), no real-world source of multi-turn caregiver
//! dialogue transcripts exists to harvest from — so this corpus is authored,
//! not harvested, following the SAME methodology `adversarial_question_
//! corpus.json` (task #15) already established for this codebase: every
//! turn's expected outcome was measured by actually running the real
//! pipeline before being committed here, never guessed (this predicate's
//! grammar routing is sensitive to subject phrasing in ways that are easy to
//! get wrong by assumption — e.g. "is my mother eligible for the assets"
//! does NOT reach the rule, while "is the applicant eligible for the
//! assets" does; both were verified, not assumed). Because the ground truth
//! is authored rather than measured, this corpus is held to a ZERO-DEFECT
//! bar, like the two authored lexicons in `lexicon_chat_capability.rs` — not
//! a ratchet.
//!
//! Honestly bounded scope: the registry (`conditional_rule::registry`)
//! currently holds exactly one real rule with exactly one `Boolean`-typed
//! evidence element, so every dialogue here exercises the Boolean slot-fill
//! path (`ChatSession::resume`'s `EvidenceType::Boolean` branch). The
//! `Concept`-typed path (task #17) has no real-registry rule to drive it
//! yet and remains covered only by the `StubConditionalReasoner` witnesses
//! in `pr4xis_chat::capability` — not a gap in this corpus, a gap in the
//! registry this corpus honestly does not paper over.

use pr4xis_chat::{ChatOutcome, ChatSession};
use serde::Deserialize;

const FIXTURE: &str = include_str!("../tests/fixtures/caregiver_dialogue_corpus.json");

#[derive(Deserialize)]
pub struct DialogueTurn {
    pub utterance: String,
    #[serde(rename = "expectedOutcome")]
    pub expected_outcome: String,
    #[serde(rename = "expectedApplies", default)]
    pub expected_applies: Option<bool>,
}

#[derive(Deserialize)]
pub struct Dialogue {
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
    pub turns: Vec<DialogueTurn>,
}

/// The full authored dialogue fixture, in commit order — index `i` here is
/// the SAME index the generated test `d_{i:02}_{name}` refers to.
pub fn fixture() -> Vec<Dialogue> {
    serde_json::from_str(FIXTURE).expect("the committed dialogue fixture is well-formed JSON")
}

/// A turn's outcome, reduced to the two fields a fixture entry can express:
/// the `ChatOutcome` variant name, and (only for `RuleResolved`) `applies`.
fn describe(outcome: &ChatOutcome) -> (&'static str, Option<bool>) {
    match outcome {
        ChatOutcome::Answered => ("Answered", None),
        ChatOutcome::Abstained { .. } => ("Abstained", None),
        ChatOutcome::Conditional { .. } => ("Conditional", None),
        ChatOutcome::RuleResolved { applies, .. } => ("RuleResolved", Some(*applies)),
    }
}

/// Run dialogue `index` turn-by-turn through ONE [`ChatSession`] over the
/// real production reasoner ([`crate::caregiver::setup_reasoner`] — the
/// exact composition the CLI/WASM hosts use), asserting every turn's
/// outcome against the fixture's authored expectation before advancing to
/// the next turn.
#[track_caller]
pub fn assert_dialogue_matches_fixture(index: usize) {
    let dialogues = fixture();
    let dialogue = &dialogues[index];
    let (reasoner, english) = crate::caregiver::setup_reasoner();
    let mut session = ChatSession::new();
    for (turn_index, turn) in dialogue.turns.iter().enumerate() {
        let result = session.ask(english, &reasoner, &turn.utterance);
        let (actual_outcome, actual_applies) = describe(&result.outcome);
        assert_eq!(
            actual_outcome, turn.expected_outcome,
            "dialogue {:?} turn {turn_index} ({:?}): expected outcome {:?}, got {actual_outcome:?} \
             (response: {:?})",
            dialogue.name, turn.utterance, turn.expected_outcome, result.response
        );
        assert_eq!(
            actual_applies, turn.expected_applies,
            "dialogue {:?} turn {turn_index} ({:?}): expected applies {:?}, got {actual_applies:?}",
            dialogue.name, turn.utterance, turn.expected_applies
        );
    }
}
