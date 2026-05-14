//! Event-driven systems — events as immutable facts; commands; handlers;
//! event logs; projections; sagas.
//!
//! An event-driven system changes state by reacting to events rather than
//! direct command. Events are immutable facts about what happened. The
//! system reacts to events, producing new events or state changes.
//!
//! # Literature
//!
//! - **Fowler (2005)** "Event Sourcing", martinfowler.com — state
//!   reconstructed from an immutable log; `state = fold(events)`.
//! - **Young (2010)** *CQRS Documents*, cqrs.wordpress.com — Command-
//!   Query Responsibility Segregation; projections as read-optimised
//!   derived views.
//! - **Guizzardi et al. (2013)** "Towards Ontological Foundations for
//!   the Conceptual Modeling of Events", *Conceptual Modeling — ER
//!   2013*, LNCS 8217 — UFO-B: events as perdurants with mereology,
//!   causality, and correlation.
//! - **Almeida & Falbo (2019)** "Events as Entities in Ontology-Driven
//!   Conceptual Modeling", *Applied Ontology* 14(3):293-329.
//! - **Hewitt (1973)** "A Universal Modular ACTOR Formalism for
//!   Artificial Intelligence", IJCAI-73 — message-passing handlers.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Event",
    source: "Fowler (2005) Event Sourcing, martinfowler.com; Young (2010) CQRS Documents; Guizzardi et al. (2013) Towards Ontological Foundations for the Conceptual Modeling of Events, ER 2013 LNCS 8217; Almeida & Falbo (2019) Events as Entities in Ontology-Driven Conceptual Modeling, Applied Ontology 14(3):293-329; Hewitt (1973) A Universal Modular ACTOR Formalism for AI, IJCAI-73",

    concepts: [
        Event,
        Command,
        State,
        Handler,
        EventLog,
        EventBus,
        Projection,
        Subscription,
        Saga,
        EventSchema,
    ],

    labels: {
        Event: ("en", "Event",
            "Guizzardi et al. (2013) UFO-B: an immutable fact about what happened - a move made, a signal changed, a message arrived."),
        Command: ("en", "Command",
            "Young (2010) CQRS: a request to do something - may be accepted (producing an Event) or rejected."),
        State: ("en", "State",
            "Fowler (2005) Event Sourcing: current state derived from the history of events - state = fold(events)."),
        Handler: ("en", "Handler",
            "Hewitt (1973) actor model: reacts to events by producing side effects, new events, or state changes."),
        EventLog: ("en", "Event log",
            "Fowler (2005): ordered, immutable log of all events - the single source of truth in event sourcing."),
        EventBus: ("en", "Event bus",
            "Hewitt (1973): routes events to their subscribed handlers."),
        Projection: ("en", "Projection",
            "Young (2010) CQRS: a read-optimised view derived from the EventLog."),
        Subscription: ("en", "Subscription",
            "Hewitt (1973): listens for specific event patterns on the bus and triggers actions."),
        Saga: ("en", "Saga",
            "Garcia-Molina & Salem (1987) Sagas, ACM SIGMOD; long-running process composed of events that form a logical unit."),
        EventSchema: ("en", "Event schema",
            "Almeida & Falbo (2019): the contract that defines what an Event contains."),
    },

    edges: [
        // Young (2010) CQRS: Command triggers Event (if accepted).
        (Command, Event, Triggers),
        // Fowler (2005): Event appended to immutable EventLog.
        (Event, EventLog, AppendedTo),
        // Hewitt (1973): Handler reacts to Event.
        (Handler, Event, ReactsTo),
        // EventBus routes Event to Handler.
        (EventBus, Handler, Routes),
        // Event changes State (via handler reaction; Fowler 2005).
        (Event, State, Changes),
        // Young (2010): Projection derived from EventLog.
        (Projection, EventLog, DerivedFrom),
        // Subscription listens to EventBus.
        (Subscription, EventBus, ListensTo),
        // Garcia-Molina & Salem (1987): Saga composes Events.
        (Saga, Event, Composes),
        // Almeida & Falbo (2019): EventSchema defines Event.
        (EventSchema, Event, Defines),
    ],
}

/// Quality: whether an event-driven concept is immutable. Fowler (2005)
/// Event Sourcing: events, the log, and the schema are immutable;
/// projections and state are mutable views.
#[derive(Debug, Clone)]
pub struct IsImmutable;

impl Quality for IsImmutable {
    type Individual = EventConcept;
    type Value = bool;

    fn get(&self, c: &EventConcept) -> Option<bool> {
        use EventConcept as E;
        match c {
            E::Event | E::EventLog | E::EventSchema => Some(true),
            E::State | E::Projection => Some(false),
            _ => None,
        }
    }
}

impl Ontology for EventOntology {
    type Cat = EventCategory;
    type Qual = IsImmutable;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<EventCategory>();
    }

    #[test]
    fn ontology_validates() {
        EventOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn ten_concepts() {
        assert_eq!(EventConcept::variants().len(), 10);
    }

    #[test]
    fn command_triggers_event() {
        let m = EventCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == EventConcept::Command
            && r.target() == EventConcept::Event
            && r.kind() == EventRelationKind::Triggers));
    }

    #[test]
    fn event_appended_to_log() {
        let m = EventCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == EventConcept::Event
            && r.target() == EventConcept::EventLog
            && r.kind() == EventRelationKind::AppendedTo));
    }

    #[test]
    fn projection_derived_from_log() {
        // Young (2010) CQRS.
        let m = EventCategory::morphisms();
        assert!(m.iter().any(|r| r.source() == EventConcept::Projection
            && r.target() == EventConcept::EventLog
            && r.kind() == EventRelationKind::DerivedFrom));
    }

    fn arb_concept() -> impl Strategy<Value = EventConcept> {
        proptest::sample::select(EventConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in EventCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in EventOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_immutability_total_on_core(c in arb_concept()) {
            use EventConcept as E;
            let v = IsImmutable.get(&c);
            let is_core = matches!(c,
                E::Event | E::EventLog | E::EventSchema | E::State | E::Projection
            );
            prop_assert_eq!(v.is_some(), is_core);
        }
    }
}
