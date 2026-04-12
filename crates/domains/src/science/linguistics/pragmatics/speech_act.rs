use pr4xis::category::Entity;

// Speech act taxonomy — Searle's 5 illocutionary types.
//
// References:
// - Austin, How to Do Things with Words (1962) — speech act theory
// - Searle, Speech Acts (1969) — illocutionary force classification
// - Searle, A Taxonomy of Illocutionary Acts (1976) — the 5 types
// - Stanford Encyclopedia, Speech Acts (2023) — comprehensive survey

/// Speech act type — what the speaker is DOING with their utterance.
///
/// Searle's 5 illocutionary types, extended with Question (a sub-type
/// of Directive that specifically seeks information rather than action).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpeechAct {
    /// Assertive: committing to the truth of a proposition.
    /// "The sky is blue." "Dogs are mammals."
    /// Direction of fit: words match world.
    Assertion,
    /// Directive (question): seeking information from the hearer.
    /// "Is the sky blue?" "What is a dog?"
    /// Direction of fit: world matches words (hearer must supply info).
    Question,
    /// Directive (command): getting the hearer to do something.
    /// "Close the door." "Run!"
    /// Direction of fit: world matches words (hearer must act).
    Command,
    /// Directive (request): politely getting the hearer to do something.
    /// "Could you close the door?"
    Request,
    /// Commissive: committing the speaker to a future action.
    /// "I will close the door." "I promise to help."
    /// Direction of fit: world matches words (speaker must act).
    Promise,
    /// Expressive: expressing the speaker's psychological state.
    /// "What a beautiful day!" "I'm sorry." "Congratulations!"
    /// No direction of fit — presupposes truth.
    Exclamation,
    /// Declaration: changing reality by the utterance itself.
    /// "I pronounce you married." "You're fired." "War is declared."
    /// Both directions of fit — words change the world.
    Declaration,
    /// Social/phatic: maintaining social connection.
    /// "Hello." "How are you?" "Goodbye."
    /// No propositional content — purely relational.
    Greeting,
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
            Self::Declaration,
            Self::Greeting,
        ]
    }
}

impl SpeechAct {
    /// Does this speech act expect a response?
    pub fn expects_response(&self) -> bool {
        matches!(self, Self::Question | Self::Request | Self::Greeting)
    }

    /// Does this speech act commit the speaker to truth?
    pub fn commits_to_truth(&self) -> bool {
        matches!(self, Self::Assertion | Self::Promise | Self::Declaration)
    }

    /// Searle's illocutionary category.
    pub fn searle_category(&self) -> SearleCategory {
        match self {
            Self::Assertion => SearleCategory::Assertive,
            Self::Question | Self::Command | Self::Request => SearleCategory::Directive,
            Self::Promise => SearleCategory::Commissive,
            Self::Exclamation | Self::Greeting => SearleCategory::Expressive,
            Self::Declaration => SearleCategory::Declaration,
        }
    }
}

/// Searle's 5 illocutionary categories (1976).
/// These are the fundamental types of things you can DO with language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearleCategory {
    /// Words match world — asserting truth.
    Assertive,
    /// World matches words — getting hearer to act.
    Directive,
    /// World matches words — committing speaker to act.
    Commissive,
    /// No direction of fit — expressing psychological state.
    Expressive,
    /// Both directions of fit — changing reality by utterance.
    Declaration,
}

impl Entity for SearleCategory {
    fn variants() -> Vec<Self> {
        vec![
            Self::Assertive,
            Self::Directive,
            Self::Commissive,
            Self::Expressive,
            Self::Declaration,
        ]
    }
}

/// Intent — the purpose behind an utterance (higher level than speech act).
/// Not all dialogue is goal-directed (cybernetic). Some is social, some is exploratory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Intent {
    /// Seeking or providing information.
    Inform,
    /// Requesting action from the hearer.
    Direct,
    /// Maintaining social connection (phatic communion).
    Social,
    /// Expressing internal state.
    Express,
    /// Changing institutional reality.
    Declare,
}

impl Entity for Intent {
    fn variants() -> Vec<Self> {
        vec![
            Self::Inform,
            Self::Direct,
            Self::Social,
            Self::Express,
            Self::Declare,
        ]
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
            SpeechAct::Declaration => Self::Declare,
            SpeechAct::Greeting => Self::Social,
        }
    }
}

/// Dialogue type — not all dialogue is cybernetic (goal-directed).
/// Some dialogue has no goal at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogueType {
    /// Goal-directed: question→answer, request→fulfill. Cybernetic feedback loop.
    GoalDirected,
    /// Social/phatic: "how are you?", small talk. Maintains relationship.
    Social,
    /// Exploratory: brainstorming, philosophy. No fixed goal, discovering together.
    Exploratory,
    /// Narrative: storytelling, recounting events. Speaker leads, listener follows.
    Narrative,
    /// Instructional: teaching, explaining. Asymmetric knowledge transfer.
    Instructional,
}

impl Entity for DialogueType {
    fn variants() -> Vec<Self> {
        vec![
            Self::GoalDirected,
            Self::Social,
            Self::Exploratory,
            Self::Narrative,
            Self::Instructional,
        ]
    }
}
