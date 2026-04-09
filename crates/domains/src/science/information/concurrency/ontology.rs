use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

// Concurrency ontology — the science of simultaneous activity.
//
// Concurrency is not just "threads" — it's the fundamental concept of
// multiple agents acting on shared resources with coordination.
// Chess is concurrent: two players, one board, turn-taking.
// Traffic is concurrent: many cars, shared intersections, signal control.
// Conversation is concurrent: two speakers, shared discourse, turn-taking.
//
// References:
// - C.A.R. Hoare, Communicating Sequential Processes (1978)
// - Robin Milner, A Calculus of Communicating Systems (1980)
// - Carl Hewitt, Actor Model (1973)

/// Core concepts of concurrency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConcurrencyConcept {
    /// An entity that can act — a player, a car, a speaker, a process.
    Agent,

    /// Something agents compete for or share — the board, the road, the channel.
    SharedResource,

    /// An action performed by an agent on a shared resource.
    Action,

    /// A mechanism that controls who can act when.
    /// Turn-taking, locks, semaphores, traffic signals.
    Synchronization,

    /// The current configuration of all agents and the shared resource.
    State,

    /// A rule about what an agent is allowed to do.
    /// In chess: legal moves. In traffic: right of way.
    Protocol,

    /// When agents cannot proceed because they're waiting for each other.
    /// In chess: impossible (turn-taking prevents it).
    /// In traffic: gridlock.
    Deadlock,

    /// When the outcome depends on the order of concurrent actions.
    /// In chess: n/a (strict alternation). In traffic: who enters first.
    RaceCondition,

    /// A value that will exist after an action completes.
    /// The opponent's response, the light change, the server reply.
    Future,

    /// A message passed between agents.
    /// A move announced, a signal displayed, an utterance spoken.
    Message,
}

impl Entity for ConcurrencyConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Agent,
            Self::SharedResource,
            Self::Action,
            Self::Synchronization,
            Self::State,
            Self::Protocol,
            Self::Deadlock,
            Self::RaceCondition,
            Self::Future,
            Self::Message,
        ]
    }
}

/// Relationships between concurrency concepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConcurrencyRelation {
    pub from: ConcurrencyConcept,
    pub to: ConcurrencyConcept,
    pub kind: ConcurrencyRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConcurrencyRelationKind {
    Identity,
    /// Agent acts on SharedResource.
    ActsOn,
    /// Synchronization controls Agent (who can act when).
    Controls,
    /// Protocol governs Action (what's allowed).
    Governs,
    /// Action changes State.
    Changes,
    /// Action produces Message.
    Produces,
    /// Message becomes Future (until received).
    Becomes,
    /// Deadlock arises from mutual waiting.
    ArisesFrom,
    /// RaceCondition arises from unsynchronized access.
    UnsynchronizedAccess,
    /// Composed.
    Composed,
}

impl Relationship for ConcurrencyRelation {
    type Object = ConcurrencyConcept;
    fn source(&self) -> ConcurrencyConcept {
        self.from
    }
    fn target(&self) -> ConcurrencyConcept {
        self.to
    }
}

/// The concurrency category.
pub struct ConcurrencyCategory;

impl Category for ConcurrencyCategory {
    type Object = ConcurrencyConcept;
    type Morphism = ConcurrencyRelation;

    fn identity(obj: &ConcurrencyConcept) -> ConcurrencyRelation {
        ConcurrencyRelation {
            from: *obj,
            to: *obj,
            kind: ConcurrencyRelationKind::Identity,
        }
    }

    fn compose(f: &ConcurrencyRelation, g: &ConcurrencyRelation) -> Option<ConcurrencyRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == ConcurrencyRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == ConcurrencyRelationKind::Identity {
            return Some(f.clone());
        }
        Some(ConcurrencyRelation {
            from: f.from,
            to: g.to,
            kind: ConcurrencyRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<ConcurrencyRelation> {
        use ConcurrencyConcept::*;
        use ConcurrencyRelationKind::*;

        let mut m = Vec::new();

        for c in ConcurrencyConcept::variants() {
            m.push(ConcurrencyRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // Agent acts on SharedResource
        m.push(ConcurrencyRelation {
            from: Agent,
            to: SharedResource,
            kind: ActsOn,
        });
        // Synchronization controls Agent
        m.push(ConcurrencyRelation {
            from: Synchronization,
            to: Agent,
            kind: Controls,
        });
        // Protocol governs Action
        m.push(ConcurrencyRelation {
            from: Protocol,
            to: Action,
            kind: Governs,
        });
        // Action changes State
        m.push(ConcurrencyRelation {
            from: Action,
            to: State,
            kind: Changes,
        });
        // Action produces Message
        m.push(ConcurrencyRelation {
            from: Action,
            to: Message,
            kind: Produces,
        });
        // Message becomes Future (pending receipt)
        m.push(ConcurrencyRelation {
            from: Message,
            to: Future,
            kind: Becomes,
        });
        // Deadlock arises from Synchronization (mutual blocking)
        m.push(ConcurrencyRelation {
            from: Synchronization,
            to: Deadlock,
            kind: ArisesFrom,
        });
        // RaceCondition arises from SharedResource (unsynchronized access)
        m.push(ConcurrencyRelation {
            from: SharedResource,
            to: RaceCondition,
            kind: UnsynchronizedAccess,
        });

        // Transitive closure
        // Synchronization → Agent → SharedResource
        m.push(ConcurrencyRelation {
            from: Synchronization,
            to: SharedResource,
            kind: Composed,
        });
        // Protocol → Action → State
        m.push(ConcurrencyRelation {
            from: Protocol,
            to: State,
            kind: Composed,
        });
        // Protocol → Action → Message
        m.push(ConcurrencyRelation {
            from: Protocol,
            to: Message,
            kind: Composed,
        });
        // Agent → SharedResource → RaceCondition
        m.push(ConcurrencyRelation {
            from: Agent,
            to: RaceCondition,
            kind: Composed,
        });
        // Action → Message → Future
        m.push(ConcurrencyRelation {
            from: Action,
            to: Future,
            kind: Composed,
        });

        m
    }
}
