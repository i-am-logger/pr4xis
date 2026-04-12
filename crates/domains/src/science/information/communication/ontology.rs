use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::relationship::Relationship;

// Communication ontology — the science of information transfer.
//
// Communication is the foundational process that underlies:
//   - Dialogue (conversation between agents)
//   - HTTP (request/response between client/server)
//   - Events (producer/consumer through bus)
//   - Spelling (writer → channel → reader, with noise)
//   - XML/markup (encoding information in a code)
//
// Each of these is an INSTANCE of communication — connected by functors.
//
// Two foundational models:
//
// Shannon (1948), "A Mathematical Theory of Communication":
//   Source → Encoder → Channel → Decoder → Destination
//   + Noise (interference in the channel)
//   Mathematical: capacity, entropy, redundancy
//
// Jakobson (1960), "Linguistics and Poetics":
//   Sender → Message → Receiver
//   + Context (referential function)
//   + Code (metalingual function)
//   + Channel/Contact (phatic function)
//   Each component has a corresponding language function.
//
// References:
// - Shannon, A Mathematical Theory of Communication (1948)
// - Jakobson, Linguistics and Poetics (1960)
// - Lasswell, The Structure and Function of Communication in Society (1948)
// - Schramm, How Communication Works (1954)
// - Wiener, Cybernetics (1948) — feedback in communication

/// Core concepts of communication.
/// Unified from Shannon (1948) and Jakobson (1960).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommunicationConcept {
    /// The agent producing the message (Shannon: source; Jakobson: addresser).
    Sender,
    /// The agent interpreting the message (Shannon: destination; Jakobson: addressee).
    Receiver,
    /// The information being communicated (Shannon: signal; Jakobson: message).
    Message,
    /// The medium through which the message travels (Shannon: channel; Jakobson: contact).
    Channel,
    /// The shared system for encoding/decoding (Shannon: encoder/decoder; Jakobson: code).
    Code,
    /// Interference that corrupts the message (Shannon: noise source).
    Noise,
    /// The receiver's response back to the sender (Wiener: cybernetic feedback).
    Feedback,
    /// The shared referential frame (Jakobson: context).
    Context,
}

impl Entity for CommunicationConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Sender,
            Self::Receiver,
            Self::Message,
            Self::Channel,
            Self::Code,
            Self::Noise,
            Self::Feedback,
            Self::Context,
        ]
    }
}

/// Relationships between communication concepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommunicationRelation {
    pub from: CommunicationConcept,
    pub to: CommunicationConcept,
    pub kind: CommunicationRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommunicationRelationKind {
    Identity,
    /// Sender produces Message.
    Produces,
    /// Message is transmitted through Channel.
    TransmittedThrough,
    /// Receiver interprets Message.
    Interprets,
    /// Code encodes/decodes Message.
    EncodesDecodes,
    /// Noise corrupts Message in Channel.
    Corrupts,
    /// Feedback flows from Receiver to Sender.
    FlowsBack,
    /// Context grounds Message interpretation.
    Grounds,
    /// Sender and Receiver share Code.
    Shares,
    Composed,
}

impl Relationship for CommunicationRelation {
    type Object = CommunicationConcept;
    fn source(&self) -> CommunicationConcept {
        self.from
    }
    fn target(&self) -> CommunicationConcept {
        self.to
    }
}

pub struct CommunicationCategory;

impl Category for CommunicationCategory {
    type Object = CommunicationConcept;
    type Morphism = CommunicationRelation;

    fn identity(obj: &CommunicationConcept) -> CommunicationRelation {
        CommunicationRelation {
            from: *obj,
            to: *obj,
            kind: CommunicationRelationKind::Identity,
        }
    }

    fn compose(
        f: &CommunicationRelation,
        g: &CommunicationRelation,
    ) -> Option<CommunicationRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == CommunicationRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == CommunicationRelationKind::Identity {
            return Some(f.clone());
        }
        Some(CommunicationRelation {
            from: f.from,
            to: g.to,
            kind: CommunicationRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<CommunicationRelation> {
        use CommunicationConcept::*;
        use CommunicationRelationKind::*;

        let mut m = Vec::new();

        for c in CommunicationConcept::variants() {
            m.push(CommunicationRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // Shannon's chain: Sender → Message → Channel → Receiver
        m.push(CommunicationRelation {
            from: Sender,
            to: Message,
            kind: Produces,
        });
        m.push(CommunicationRelation {
            from: Message,
            to: Channel,
            kind: TransmittedThrough,
        });
        m.push(CommunicationRelation {
            from: Receiver,
            to: Message,
            kind: Interprets,
        });

        // Code encodes and decodes the message
        m.push(CommunicationRelation {
            from: Code,
            to: Message,
            kind: EncodesDecodes,
        });

        // Noise corrupts message in channel
        m.push(CommunicationRelation {
            from: Noise,
            to: Channel,
            kind: Corrupts,
        });

        // Feedback: receiver → sender (Wiener's cybernetic loop)
        m.push(CommunicationRelation {
            from: Feedback,
            to: Sender,
            kind: FlowsBack,
        });
        m.push(CommunicationRelation {
            from: Receiver,
            to: Feedback,
            kind: Produces,
        });

        // Context grounds interpretation
        m.push(CommunicationRelation {
            from: Context,
            to: Message,
            kind: Grounds,
        });

        // Sender and Receiver share Code
        m.push(CommunicationRelation {
            from: Sender,
            to: Code,
            kind: Shares,
        });
        m.push(CommunicationRelation {
            from: Receiver,
            to: Code,
            kind: Shares,
        });

        // Transitive compositions
        m.push(CommunicationRelation {
            from: Sender,
            to: Channel,
            kind: Composed,
        });
        m.push(CommunicationRelation {
            from: Sender,
            to: Receiver,
            kind: Composed,
        });
        m.push(CommunicationRelation {
            from: Noise,
            to: Message,
            kind: Composed,
        });
        m.push(CommunicationRelation {
            from: Receiver,
            to: Sender,
            kind: Composed,
        });

        // Self-composed closure
        for c in CommunicationConcept::variants() {
            m.push(CommunicationRelation {
                from: c,
                to: c,
                kind: Composed,
            });
        }

        m
    }
}

/// Jakobson's six language functions (1960).
/// Each communication component has a corresponding function when
/// the communicative act focuses on that component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JakobsonFunction {
    /// Focus on Context → referential (informative).
    Referential,
    /// Focus on Sender → emotive/expressive.
    Emotive,
    /// Focus on Receiver → conative (persuasive, imperative).
    Conative,
    /// Focus on Channel → phatic (maintaining contact: "hello", "how are you?").
    Phatic,
    /// Focus on Code → metalingual (about the code itself: "what does X mean?").
    Metalingual,
    /// Focus on Message → poetic (the form of the message itself).
    Poetic,
}

impl Entity for JakobsonFunction {
    fn variants() -> Vec<Self> {
        vec![
            Self::Referential,
            Self::Emotive,
            Self::Conative,
            Self::Phatic,
            Self::Metalingual,
            Self::Poetic,
        ]
    }
}

impl JakobsonFunction {
    /// Which communication component does this function focus on?
    pub fn focused_component(&self) -> CommunicationConcept {
        match self {
            Self::Referential => CommunicationConcept::Context,
            Self::Emotive => CommunicationConcept::Sender,
            Self::Conative => CommunicationConcept::Receiver,
            Self::Phatic => CommunicationConcept::Channel,
            Self::Metalingual => CommunicationConcept::Code,
            Self::Poetic => CommunicationConcept::Message,
        }
    }
}
