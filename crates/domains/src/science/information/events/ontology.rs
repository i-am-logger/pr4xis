use praxis::category::Category;
use praxis::category::entity::Entity;
use praxis::category::relationship::Relationship;

// Event-driven system ontology.
//
// An event-driven system is one where state changes are triggered by events,
// not by direct command. Events are immutable facts about what happened.
// The system reacts to events, producing new events or state changes.
//
// This connects to:
// - DOLCE: events ARE perdurants (things that happen over time)
// - UFO-B: events have mereology (composition), causality, and correlation
// - Concurrency: events are the messages between concurrent agents
// - Systems thinking: events are transitions in the cybernetic loop
//
// References:
// - Martin Fowler, Event Sourcing (2005)
// - Greg Young, CQRS Documents (2010)
// - Guizzardi et al., UFO-B: Ontology of Events (2013)
// - Almeida & Falbo, Events as Entities in Ontology-Driven Modeling (2019)

/// Core concepts of event-driven systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventConcept {
    /// Something that happened — an immutable fact.
    /// A move was made. A signal changed. A message arrived.
    Event,

    /// A request to do something — may be accepted or rejected.
    /// "Move pawn to e4" is a command; the move happening is an event.
    Command,

    /// The current state — derived from the history of events.
    /// Event sourcing: state = fold(events).
    State,

    /// Reacts to events by producing side effects, new events, or state changes.
    Handler,

    /// An ordered, immutable log of all events that have occurred.
    /// The single source of truth in event sourcing.
    EventLog,

    /// Routes events to the correct handlers.
    EventBus,

    /// A read-optimized view derived from events (CQRS pattern).
    Projection,

    /// Listens for specific event patterns and triggers actions.
    Subscription,

    /// A group of events that form a logical unit (saga/process manager).
    Saga,

    /// The schema/contract that defines what an event contains.
    EventSchema,
}

impl Entity for EventConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::Event,
            Self::Command,
            Self::State,
            Self::Handler,
            Self::EventLog,
            Self::EventBus,
            Self::Projection,
            Self::Subscription,
            Self::Saga,
            Self::EventSchema,
        ]
    }
}

/// Relationships between event-driven concepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventRelation {
    pub from: EventConcept,
    pub to: EventConcept,
    pub kind: EventRelationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventRelationKind {
    Identity,
    /// Command triggers Event (if accepted).
    Triggers,
    /// Event is appended to EventLog.
    AppendedTo,
    /// Handler reacts to Event.
    ReactsTo,
    /// EventBus routes Event to Handler.
    Routes,
    /// Event changes State (via handler).
    Changes,
    /// Projection is derived from EventLog.
    DerivedFrom,
    /// Subscription listens to EventBus.
    ListensTo,
    /// Saga composes Events.
    Composes,
    /// EventSchema defines Event structure.
    Defines,
    /// Composed.
    Composed,
}

impl Relationship for EventRelation {
    type Object = EventConcept;
    fn source(&self) -> EventConcept {
        self.from
    }
    fn target(&self) -> EventConcept {
        self.to
    }
}

/// The event-driven category.
pub struct EventCategory;

impl Category for EventCategory {
    type Object = EventConcept;
    type Morphism = EventRelation;

    fn identity(obj: &EventConcept) -> EventRelation {
        EventRelation {
            from: *obj,
            to: *obj,
            kind: EventRelationKind::Identity,
        }
    }

    fn compose(f: &EventRelation, g: &EventRelation) -> Option<EventRelation> {
        if f.to != g.from {
            return None;
        }
        if f.kind == EventRelationKind::Identity {
            return Some(g.clone());
        }
        if g.kind == EventRelationKind::Identity {
            return Some(f.clone());
        }
        Some(EventRelation {
            from: f.from,
            to: g.to,
            kind: EventRelationKind::Composed,
        })
    }

    fn morphisms() -> Vec<EventRelation> {
        use EventConcept::*;
        use EventRelationKind::*;

        let mut m = Vec::new();

        for c in EventConcept::variants() {
            m.push(EventRelation {
                from: c,
                to: c,
                kind: Identity,
            });
        }

        // Command triggers Event
        m.push(EventRelation {
            from: Command,
            to: Event,
            kind: Triggers,
        });
        // Event appended to EventLog
        m.push(EventRelation {
            from: Event,
            to: EventLog,
            kind: AppendedTo,
        });
        // Handler reacts to Event
        m.push(EventRelation {
            from: Handler,
            to: Event,
            kind: ReactsTo,
        });
        // EventBus routes Event to Handler
        m.push(EventRelation {
            from: EventBus,
            to: Handler,
            kind: Routes,
        });
        // Event changes State
        m.push(EventRelation {
            from: Event,
            to: State,
            kind: Changes,
        });
        // Projection derived from EventLog
        m.push(EventRelation {
            from: Projection,
            to: EventLog,
            kind: DerivedFrom,
        });
        // Subscription listens to EventBus
        m.push(EventRelation {
            from: Subscription,
            to: EventBus,
            kind: ListensTo,
        });
        // Saga composes Events
        m.push(EventRelation {
            from: Saga,
            to: Event,
            kind: Composes,
        });
        // EventSchema defines Event
        m.push(EventRelation {
            from: EventSchema,
            to: Event,
            kind: Defines,
        });

        // Transitive
        // Command → Event → State
        m.push(EventRelation {
            from: Command,
            to: State,
            kind: Composed,
        });
        // Command → Event → EventLog
        m.push(EventRelation {
            from: Command,
            to: EventLog,
            kind: Composed,
        });
        // EventBus → Handler → Event
        m.push(EventRelation {
            from: EventBus,
            to: Event,
            kind: Composed,
        });
        // Subscription → EventBus → Handler
        m.push(EventRelation {
            from: Subscription,
            to: Handler,
            kind: Composed,
        });
        // Saga → Event → State
        m.push(EventRelation {
            from: Saga,
            to: State,
            kind: Composed,
        });
        // Saga → Event → EventLog
        m.push(EventRelation {
            from: Saga,
            to: EventLog,
            kind: Composed,
        });

        // Self-composed (for functor closure)
        for c in EventConcept::variants() {
            m.push(EventRelation {
                from: c,
                to: c,
                kind: Composed,
            });
        }

        m
    }
}
