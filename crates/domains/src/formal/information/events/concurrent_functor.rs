use pr4xis::category::Functor;

use super::ontology::*;
use crate::formal::information::concurrency::ontology::*;

/// Functor: EventDriven → Concurrent.
/// Proves every event-driven system IS concurrent.
///
/// Event-driven systems have multiple handlers reacting to events
/// asynchronously — that IS concurrency. The event bus IS synchronization.
/// The event log IS the shared resource. Handlers ARE agents.
pub struct EventsToConcurrency;

impl Functor for EventsToConcurrency {
    type Source = EventCategory;
    type Target = ConcurrencyCategory;

    fn map_object(obj: &EventConcept) -> ConcurrencyConcept {
        match obj {
            EventConcept::Event => ConcurrencyConcept::Message,
            EventConcept::Command => ConcurrencyConcept::Action,
            EventConcept::State => ConcurrencyConcept::State,
            EventConcept::Handler => ConcurrencyConcept::Agent,
            EventConcept::EventLog => ConcurrencyConcept::SharedResource,
            EventConcept::EventBus => ConcurrencyConcept::Synchronization,
            EventConcept::Projection => ConcurrencyConcept::State,
            EventConcept::Subscription => ConcurrencyConcept::Protocol,
            EventConcept::Saga => ConcurrencyConcept::Agent,
            EventConcept::EventSchema => ConcurrencyConcept::Protocol,
        }
    }

    fn map_morphism(m: &EventRelation) -> ConcurrencyRelation {
        let from = Self::map_object(&m.from);
        let to = Self::map_object(&m.to);
        // #166 removed the dense `Composed` kind. Identity maps to
        // identity; otherwise project the (source, target) pair to the
        // most specific concurrency kind available (heterogeneous
        // compositions are partial — fall back to Identity at the same
        // object).
        let kind = if m.kind == EventRelationKind::Identity {
            ConcurrencyRelationKind::Identity
        } else {
            match (from, to) {
                (ConcurrencyConcept::Action, ConcurrencyConcept::Message) => {
                    ConcurrencyRelationKind::Produces
                }
                (ConcurrencyConcept::Synchronization, ConcurrencyConcept::Agent) => {
                    ConcurrencyRelationKind::Controls
                }
                (ConcurrencyConcept::Message, ConcurrencyConcept::State) => {
                    ConcurrencyRelationKind::Changes
                }
                // Off-pattern pairs use Subsumption (canonical OBO-RO kind,
                // Smith 2005) rather than Identity — otherwise target's
                // identity-aware compose breaks FunctorCompositionLaw
                // (Mac Lane CWM Ch. II §1) when source ≠ target.
                _ => ConcurrencyRelationKind::Subsumption,
            }
        };
        ConcurrencyRelation { from, to, kind }
    }
}
pr4xis::register_functor!(EventsToConcurrency);
