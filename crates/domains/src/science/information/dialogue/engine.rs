use praxis::engine::{Action, Engine, Precondition, PreconditionResult, Situation};

use crate::science::linguistics::pragmatics::speech_act::{DialogueType, SpeechAct};

// Dialogue engine — a cybernetic loop for conversation.
//
// Situation: the current state of the dialogue (history, topic, expectations).
// Action: something that happens (receive utterance, generate response).
// Preconditions: what must be true for an action to proceed.
//
// The engine enforces the structure of conversation through ontology,
// not through custom parsing in the CLI.

/// The situation in a dialogue — what the conversation looks like right now.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogueState {
    pub turns: Vec<DialogueTurn>,
    pub topic: Option<String>,
    pub dialogue_type: DialogueType,
    pub expecting_response: bool,
    pub terminated: bool,
}

/// A single turn in the dialogue.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogueTurn {
    pub speaker: Speaker,
    pub text: String,
    pub speech_act: SpeechAct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speaker {
    User,
    System,
}

impl DialogueState {
    pub fn new() -> Self {
        Self {
            turns: Vec::new(),
            topic: None,
            dialogue_type: DialogueType::GoalDirected,
            expecting_response: false,
            terminated: false,
        }
    }

    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }

    pub fn last_speaker(&self) -> Option<Speaker> {
        self.turns.last().map(|t| t.speaker)
    }
}

impl Default for DialogueState {
    fn default() -> Self {
        Self::new()
    }
}

impl Situation for DialogueState {
    fn describe(&self) -> String {
        let last = self
            .turns
            .last()
            .map(|t| t.text.as_str())
            .unwrap_or("(start)");
        format!(
            "turn {} | topic: {} | last: {}",
            self.turns.len(),
            self.topic.as_deref().unwrap_or("none"),
            last
        )
    }

    fn is_terminal(&self) -> bool {
        self.terminated
    }
}

/// Actions in the dialogue engine.
#[derive(Debug, Clone)]
pub enum DialogueAction {
    /// User sends an utterance.
    UserUtterance { text: String, speech_act: SpeechAct },
    /// System responds.
    SystemResponse { text: String, speech_act: SpeechAct },
    /// End the dialogue.
    EndDialogue,
}

impl Action for DialogueAction {
    type Sit = DialogueState;

    fn describe(&self) -> String {
        match self {
            Self::UserUtterance { text, .. } => format!("user: {}", text),
            Self::SystemResponse { text, .. } => format!("system: {}", text),
            Self::EndDialogue => "end dialogue".into(),
        }
    }
}

/// Apply a dialogue action to the state.
pub fn apply_dialogue(
    state: &DialogueState,
    action: &DialogueAction,
) -> Result<DialogueState, String> {
    let mut new_state = state.clone();

    match action {
        DialogueAction::UserUtterance { text, speech_act } => {
            new_state.turns.push(DialogueTurn {
                speaker: Speaker::User,
                text: text.clone(),
                speech_act: *speech_act,
            });
            new_state.expecting_response = speech_act.expects_response();
        }
        DialogueAction::SystemResponse { text, speech_act } => {
            new_state.turns.push(DialogueTurn {
                speaker: Speaker::System,
                text: text.clone(),
                speech_act: *speech_act,
            });
            new_state.expecting_response = false;
        }
        DialogueAction::EndDialogue => {
            new_state.terminated = true;
        }
    }

    Ok(new_state)
}

/// Precondition: system can only respond after user speaks.
pub struct TurnTaking;

impl Precondition<DialogueAction> for TurnTaking {
    fn check(&self, state: &DialogueState, action: &DialogueAction) -> PreconditionResult {
        match action {
            DialogueAction::SystemResponse { .. } => {
                if state.last_speaker() == Some(Speaker::System) && !state.turns.is_empty() {
                    PreconditionResult::violated(
                        "turn-taking",
                        "system cannot respond twice in a row without user input",
                        &state.describe(),
                        &action.describe(),
                    )
                } else {
                    PreconditionResult::satisfied("turn-taking", "turn available")
                }
            }
            _ => PreconditionResult::satisfied("turn-taking", "not a system response"),
        }
    }

    fn describe(&self) -> &str {
        "dialogue turn-taking: system responds only after user speaks"
    }
}

/// Build a dialogue engine with standard preconditions.
pub fn dialogue_engine() -> Engine<DialogueAction> {
    Engine::new(
        DialogueState::new(),
        vec![Box::new(TurnTaking)],
        apply_dialogue,
    )
}
