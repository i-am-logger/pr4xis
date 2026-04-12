use pr4xis::category::Entity;
use pr4xis::define_category;

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
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

define_category! {
    /// The concurrency category.
    pub ConcurrencyCategory {
        entity: ConcurrencyConcept,
        relation: ConcurrencyRelation,
        kind: ConcurrencyRelationKind,
        kinds: [
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
        ],
        edges: [
            // Agent acts on SharedResource
            (Agent, SharedResource, ActsOn),
            // Synchronization controls Agent
            (Synchronization, Agent, Controls),
            // Protocol governs Action
            (Protocol, Action, Governs),
            // Action changes State
            (Action, State, Changes),
            // Action produces Message
            (Action, Message, Produces),
            // Message becomes Future (pending receipt)
            (Message, Future, Becomes),
            // Deadlock arises from Synchronization (mutual blocking)
            (Synchronization, Deadlock, ArisesFrom),
            // RaceCondition arises from SharedResource (unsynchronized access)
            (SharedResource, RaceCondition, UnsynchronizedAccess),
        ],
        composed: [
            // Synchronization → Agent → SharedResource
            (Synchronization, SharedResource),
            // Protocol → Action → State
            (Protocol, State),
            // Protocol → Action → Message
            (Protocol, Message),
            // Agent → SharedResource → RaceCondition
            (Agent, RaceCondition),
            // Action → Message → Future
            (Action, Future),
            // Dense connectivity needed for systems functor
            (Agent, State),
            (State, Agent),
            (State, SharedResource),
            (State, Synchronization),
            (State, Protocol),
            (State, Action),
            (State, RaceCondition),
            (State, Deadlock),
            (Synchronization, State),
            (Synchronization, Action),
            (Agent, Action),
            (Agent, Synchronization),
            (Protocol, SharedResource),
            (SharedResource, State),
        ],
    }
}
