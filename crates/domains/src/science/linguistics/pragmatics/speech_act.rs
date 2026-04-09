use praxis::category::Entity;

/// Speech act type — what the speaker is DOING with their utterance.
/// Austin/Searle classification: utterances perform actions, not just convey information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpeechAct {
    /// Asserting a fact: "The sky is blue."
    Assertion,
    /// Asking a question: "Is the sky blue?"
    Question,
    /// Giving a command: "Close the door."
    Command,
    /// Making a request: "Could you close the door?"
    Request,
    /// Making a promise: "I will close the door."
    Promise,
    /// Expressing emotion: "What a beautiful day!"
    Exclamation,
}

impl Entity for SpeechAct {
    fn variants() -> Vec<Self> {
        vec![
            Self::Assertion,
            Self::Question,
            Self::Command,
            Self::Request,
            Self::Promise,
            Self::Exclamation,
        ]
    }
}

impl SpeechAct {
    /// Does this speech act expect a response?
    pub fn expects_response(&self) -> bool {
        matches!(self, Self::Question | Self::Request)
    }

    /// Does this speech act commit the speaker to truth?
    pub fn commits_to_truth(&self) -> bool {
        matches!(self, Self::Assertion | Self::Promise)
    }
}

/// Intent — the purpose behind an utterance (higher level than speech act).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Intent {
    /// Seeking information.
    Inform,
    /// Requesting action.
    Direct,
    /// Maintaining social connection.
    Social,
    /// Expressing internal state.
    Express,
}

impl Entity for Intent {
    fn variants() -> Vec<Self> {
        vec![Self::Inform, Self::Direct, Self::Social, Self::Express]
    }
}

impl Intent {
    pub fn from_speech_act(act: SpeechAct) -> Self {
        match act {
            SpeechAct::Assertion => Self::Inform,
            SpeechAct::Question => Self::Inform,
            SpeechAct::Command | SpeechAct::Request => Self::Direct,
            SpeechAct::Promise => Self::Direct,
            SpeechAct::Exclamation => Self::Express,
        }
    }
}
