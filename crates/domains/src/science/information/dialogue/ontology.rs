use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

// Dialogue ontology — the science of conversation.
//
// A dialogue IS:
// - An event-driven system (utterances are events)
// - Concurrent (two+ agents, turn-taking)
// - A system (feedback: listen → understand → respond)
// - Linguistic (uses grammar, semantics, pragmatics)
//
// References:
// - Austin, How to Do Things with Words (1962) — speech acts
// - Searle, Speech Acts (1969) — illocutionary force
// - Jurafsky & Martin, Speech and Language Processing — dialogue acts
// - Traum, A Computational Theory of Grounding (1994)

/// Core concepts of a dialogue system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogueConcept {
    /// A single utterance from a participant.
    Utterance,
    /// The speaker/listener — an agent in the conversation.
    Participant,
    /// What the speaker intends to achieve with an utterance.
    /// Inform, question, request, confirm, deny, etc.
    DialogueAct,
    /// The shared knowledge between participants at a point in time.
    /// What's been said, what's understood, what's expected.
    DialogueState,
    /// The topic currently being discussed.
    Topic,
    /// The conversational context — everything said so far.
    History,
    /// Understanding an utterance — parsing, meaning extraction, intent recognition.
    Understanding,
    /// Generating a response — selecting content, constructing utterance.
    Generation,
    /// The mechanism controlling who speaks when.
    TurnManagement,
    /// A successful exchange — both parties understand each other.
    Grounding,
}

impl Entity for DialogueConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Utterance,
            Self::Participant,
            Self::DialogueAct,
            Self::DialogueState,
            Self::Topic,
            Self::History,
            Self::Understanding,
            Self::Generation,
            Self::TurnManagement,
            Self::Grounding,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogueRelation {
    pub from: DialogueConcept,
    pub to: DialogueConcept,
    pub kind: DialogueRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogueRelationKind {
    Identity,
    /// Participant produces Utterance.
    Produces,
    /// Utterance expresses DialogueAct.
    Expresses,
    /// Utterance updates DialogueState.
    Updates,
    /// Utterance appended to History.
    AppendedTo,
    /// Understanding interprets Utterance.
    Interprets,
    /// Generation creates Utterance.
    Creates,
    /// TurnManagement controls Participant.
    Controls,
    /// Grounding arises from shared Understanding.
    ArisesFrom,
    /// DialogueAct addresses Topic.
    Addresses,
    Composed,
}

impl Relationship for DialogueRelation {
    type Object = DialogueConcept;
    fn source(&self) -> DialogueConcept {
        self.from
    }
    fn target(&self) -> DialogueConcept {
        self.to
    }
}

pub struct DialogueCategory;

impl Category for DialogueCategory {
    type Object = DialogueConcept;
    type Morphism = DialogueRelation;

    fn identity(obj: &DialogueConcept) -> DialogueRelation {
        DialogueRelation {
            from: *obj,
            to: *obj,
            kind: DialogueRelationKind::Identity,
        }
    }

    fn compose(f: &DialogueRelation, g: &DialogueRelation) -> Option<DialogueRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == DialogueRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == DialogueRelationKind::Identity {
            return Some(f.clone());
        }
        Some(DialogueRelation {
            from: f.from,
            to: g.to,
            kind: DialogueRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<DialogueRelation> {
        use DialogueConcept::*;
        use DialogueRelationKind::*;

        let mut m = Vec::new();

        for c in DialogueConcept::variants() {
            m.push(DialogueRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // Participant produces Utterance
        m.push(DialogueRelation {
            from: Participant,
            to: Utterance,
            kind: Produces,
        });
        // Utterance expresses DialogueAct
        m.push(DialogueRelation {
            from: Utterance,
            to: DialogueAct,
            kind: Expresses,
        });
        // Utterance updates DialogueState
        m.push(DialogueRelation {
            from: Utterance,
            to: DialogueState,
            kind: Updates,
        });
        // Utterance appended to History
        m.push(DialogueRelation {
            from: Utterance,
            to: History,
            kind: AppendedTo,
        });
        // Understanding interprets Utterance
        m.push(DialogueRelation {
            from: Understanding,
            to: Utterance,
            kind: Interprets,
        });
        // Generation creates Utterance
        m.push(DialogueRelation {
            from: Generation,
            to: Utterance,
            kind: Creates,
        });
        // TurnManagement controls Participant
        m.push(DialogueRelation {
            from: TurnManagement,
            to: Participant,
            kind: Controls,
        });
        // Grounding arises from Understanding
        m.push(DialogueRelation {
            from: Understanding,
            to: Grounding,
            kind: ArisesFrom,
        });
        // DialogueAct addresses Topic
        m.push(DialogueRelation {
            from: DialogueAct,
            to: Topic,
            kind: Addresses,
        });

        // Transitive
        m.push(DialogueRelation {
            from: Participant,
            to: DialogueAct,
            kind: Composed,
        });
        m.push(DialogueRelation {
            from: Participant,
            to: DialogueState,
            kind: Composed,
        });
        m.push(DialogueRelation {
            from: Participant,
            to: History,
            kind: Composed,
        });
        m.push(DialogueRelation {
            from: TurnManagement,
            to: Utterance,
            kind: Composed,
        });
        m.push(DialogueRelation {
            from: Understanding,
            to: DialogueAct,
            kind: Composed,
        });
        m.push(DialogueRelation {
            from: Generation,
            to: DialogueAct,
            kind: Composed,
        });

        // Self-composed
        for c in DialogueConcept::variants() {
            m.push(DialogueRelation {
                from: c,
                to: c,
                kind: Composed,
            });
        }

        m
    }
}
