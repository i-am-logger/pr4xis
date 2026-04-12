use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

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
// - Ginzburg, The Interactive Stance (2012) — KoS, QUD
// - Clark, Using Language (1996) — grounding, common ground
// - Stalnaker, Common Ground (2002) — context set, assertion
// - Levelt, Speaking (1989) — speech production model
// - Grice, Logic and Conversation (1975) — cooperative principle

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
    /// Traum (1994): Initiate → Acknowledge → Ground.
    Grounding,

    // === New concepts from literature ===
    /// Questions Under Discussion — the stack that drives interpretation.
    /// Ginzburg (2012): QUD is a priority queue of open questions.
    /// Every utterance either raises a question or resolves one.
    QUD,
    /// Shared beliefs between participants (Stalnaker 2002).
    /// The context set — propositions both participants accept as true.
    /// Assertion = proposal to add to common ground.
    CommonGround,
    /// What the speaker wants to achieve BEFORE formulating words.
    /// Levelt (1989): the preverbal message from the Conceptualizer.
    /// Contains: topic, focus, mood, speech act type, propositional content.
    Intention,
    /// The act of establishing mutual understanding.
    /// Traum (1994): Initiate, Continue, Acknowledge, Repair, ReqRepair.
    /// Clark (1996): presentation + acceptance.
    GroundingAct,
    /// When understanding fails and needs correction.
    /// Schegloff et al. (1977): self-repair, other-repair, other-initiated self-repair.
    Repair,
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
            Self::QUD,
            Self::CommonGround,
            Self::Intention,
            Self::GroundingAct,
            Self::Repair,
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
    /// Utterance raises/resolves QUD (Ginzburg).
    RaisesOrResolves,
    /// Understanding updates CommonGround (Stalnaker).
    EstablishesIn,
    /// Intention drives Generation (Levelt).
    Drives,
    /// GroundingAct achieves Grounding (Traum).
    Achieves,
    /// Repair restores Understanding (Schegloff).
    Restores,
    /// Intention formed from DialogueState + QUD (what to say next).
    FormedFrom,
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

        // QUD: utterances raise or resolve questions (Ginzburg 2012)
        m.push(DialogueRelation {
            from: Utterance,
            to: QUD,
            kind: RaisesOrResolves,
        });
        // Understanding establishes facts in CommonGround (Stalnaker 2002)
        m.push(DialogueRelation {
            from: Understanding,
            to: CommonGround,
            kind: EstablishesIn,
        });
        // Intention drives Generation (Levelt 1989)
        m.push(DialogueRelation {
            from: Intention,
            to: Generation,
            kind: Drives,
        });
        // GroundingAct achieves Grounding (Traum 1994)
        m.push(DialogueRelation {
            from: GroundingAct,
            to: Grounding,
            kind: Achieves,
        });
        // Repair restores Understanding (Schegloff 1977)
        m.push(DialogueRelation {
            from: Repair,
            to: Understanding,
            kind: Restores,
        });
        // Intention formed from DialogueState + QUD
        m.push(DialogueRelation {
            from: DialogueState,
            to: Intention,
            kind: FormedFrom,
        });
        m.push(DialogueRelation {
            from: QUD,
            to: Intention,
            kind: FormedFrom,
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
        // Intention → Utterance (through Generation)
        m.push(DialogueRelation {
            from: Intention,
            to: Utterance,
            kind: Composed,
        });
        // DialogueState → Generation (through Intention)
        m.push(DialogueRelation {
            from: DialogueState,
            to: Generation,
            kind: Composed,
        });
        // Repair → Grounding (through Understanding → GroundingAct)
        m.push(DialogueRelation {
            from: Repair,
            to: Grounding,
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
