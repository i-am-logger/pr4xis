use super::speech_act::{Intent, SpeechAct};
use crate::cognitive::linguistics::semantics::meaning::MeaningRep;

/// A discourse turn — one utterance in a conversation with its full analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct Turn {
    pub text: String,
    pub speech_act: SpeechAct,
    pub intent: Intent,
    pub meaning: Option<MeaningRep>,
}

impl Turn {
    pub fn new(text: &str, speech_act: SpeechAct, meaning: Option<MeaningRep>) -> Self {
        Self {
            text: text.to_string(),
            intent: Intent::from_speech_act(speech_act),
            speech_act,
            meaning,
        }
    }
}

/// Discourse context — the state of a conversation.
/// Tracks the history of turns and current topic.
#[derive(Debug, Clone)]
pub struct Discourse {
    pub turns: Vec<Turn>,
    pub topic: Option<String>,
}

impl Discourse {
    pub fn new() -> Self {
        Self {
            turns: Vec::new(),
            topic: None,
        }
    }

    pub fn add_turn(&mut self, turn: Turn) {
        // Update topic from the most recent assertion's predicate
        if turn.speech_act == SpeechAct::Assertion
            && let Some(MeaningRep::Atomic(ref prop)) = turn.meaning
        {
            self.topic = Some(prop.predicate.lemma.clone());
        }
        self.turns.push(turn);
    }

    pub fn last_turn(&self) -> Option<&Turn> {
        self.turns.last()
    }

    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }

    /// Does the conversation expect a response (last turn was a question/request)?
    pub fn expects_response(&self) -> bool {
        self.last_turn()
            .map(|t| t.speech_act.expects_response())
            .unwrap_or(false)
    }
}

impl Default for Discourse {
    fn default() -> Self {
        Self::new()
    }
}
