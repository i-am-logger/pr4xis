use praxis::category::Functor;

use super::ontology::*;
use crate::science::systems::ontology::*;

/// Functor: Systems → EventDriven.
/// Proves every system IS event-driven.
///
/// A transition IS an event (something that happened).
/// State IS event-sourced (derived from transition history).
/// Feedback IS event routing (state changes trigger new transitions).
/// PGN, HTTP logs, court transcripts, sheet music — all event logs.
pub struct SystemsToEvents;

impl Functor for SystemsToEvents {
    type Source = SystemsCategory;
    type Target = EventCategory;

    fn map_object(obj: &SystemConcept) -> EventConcept {
        match obj {
            SystemConcept::Component => EventConcept::Handler,
            SystemConcept::Interaction => EventConcept::Command,
            SystemConcept::State => EventConcept::State,
            SystemConcept::Transition => EventConcept::Event,
            SystemConcept::Constraint => EventConcept::EventSchema,
            SystemConcept::Feedback => EventConcept::EventBus,
            SystemConcept::Homeostasis => EventConcept::Subscription,
            SystemConcept::Emergence => EventConcept::Projection,
            SystemConcept::Boundary => EventConcept::EventLog,
            SystemConcept::Controller => EventConcept::Saga,
        }
    }

    fn map_morphism(m: &SystemRelation) -> EventRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        let kind = if m.kind == SystemRelationKind::Identity {
            EventRelationKind::Identity
        } else if m.kind == SystemRelationKind::Composed || from == to {
            EventRelationKind::Composed
        } else {
            match (from, to) {
                (EventConcept::Command, EventConcept::Event) => EventRelationKind::Triggers,
                (EventConcept::Event, EventConcept::EventLog) => EventRelationKind::AppendedTo,
                (EventConcept::Handler, EventConcept::Event) => EventRelationKind::ReactsTo,
                (EventConcept::EventBus, EventConcept::Handler) => EventRelationKind::Routes,
                (EventConcept::Event, EventConcept::State) => EventRelationKind::Changes,
                (EventConcept::Projection, EventConcept::EventLog) => {
                    EventRelationKind::DerivedFrom
                }
                (EventConcept::Subscription, EventConcept::EventBus) => {
                    EventRelationKind::ListensTo
                }
                (EventConcept::Saga, EventConcept::Event) => EventRelationKind::Composes,
                (EventConcept::EventSchema, EventConcept::Event) => EventRelationKind::Defines,
                _ => EventRelationKind::Composed,
            }
        };
        EventRelation { from, to, kind }
    }
}
