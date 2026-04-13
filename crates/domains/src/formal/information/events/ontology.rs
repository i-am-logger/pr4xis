use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Ontology, Quality};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
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

define_ontology! {
    /// The event-driven category.
    pub EventOntology for EventCategory {
        concepts: EventConcept,
        relation: EventRelation,
        kind: EventRelationKind,
        kinds: [
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
        ],
        edges: [
            // Command triggers Event
            (Command, Event, Triggers),
            // Event appended to EventLog
            (Event, EventLog, AppendedTo),
            // Handler reacts to Event
            (Handler, Event, ReactsTo),
            // EventBus routes Event to Handler
            (EventBus, Handler, Routes),
            // Event changes State
            (Event, State, Changes),
            // Projection derived from EventLog
            (Projection, EventLog, DerivedFrom),
            // Subscription listens to EventBus
            (Subscription, EventBus, ListensTo),
            // Saga composes Events
            (Saga, Event, Composes),
            // EventSchema defines Event
            (EventSchema, Event, Defines),
        ],
        composed: [
            // Command → Event → State
            (Command, State),
            // Command → Event → EventLog
            (Command, EventLog),
            // EventBus → Handler → Event
            (EventBus, Event),
            // Subscription → EventBus → Handler
            (Subscription, Handler),
            // Saga → Event → State
            (Saga, State),
            // Saga → Event → EventLog
            (Saga, EventLog),
        ],
    }
}

/// Whether an event concept is immutable (core event sourcing property).
#[derive(Debug, Clone)]
pub struct IsImmutable;

impl Quality for IsImmutable {
    type Individual = EventConcept;
    type Value = bool;

    fn get(&self, individual: &EventConcept) -> Option<bool> {
        match individual {
            EventConcept::Event => Some(true),
            EventConcept::EventLog => Some(true),
            EventConcept::EventSchema => Some(true),
            EventConcept::State => Some(false),
            EventConcept::Projection => Some(false),
            _ => None,
        }
    }
}

impl Ontology for EventOntology {
    type Cat = EventCategory;
    type Qual = IsImmutable;

    fn structural_axioms() -> Vec<Box<dyn pr4xis::ontology::Axiom>> {
        Self::generated_structural_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<EventCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        EventOntology::validate().unwrap();
    }
}
