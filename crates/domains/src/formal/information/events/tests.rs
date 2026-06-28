use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::laws::assert_functor_laws;
use pr4xis::category::{Arrow, Category, FinitelyGenerated, Functor};

use super::concurrent_functor::*;
use super::ontology::*;
use super::systems_functor::*;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn event_category_laws() {
    assert_category_laws::<EventCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn event_has_10_concepts() {
    assert_eq!(EventConcept::variants().len(), 10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn command_triggers_event() {
    let m = EventCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == EventConcept::Command
        && r.target() == EventConcept::Event
        && r.kind() == EventRelationKind::Triggers));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn event_appended_to_log() {
    let m = EventCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == EventConcept::Event
        && r.target() == EventConcept::EventLog
        && r.kind() == EventRelationKind::AppendedTo));
}

// === Every event-driven system IS concurrent ===

#[pr4xis::praxis_value(Extensible)]
#[test]
fn events_to_concurrency_functor_laws() {
    assert_functor_laws::<EventsToConcurrency>();
}

#[pr4xis::praxis_value(Verifiable, Extensible)]
#[test]
fn handler_is_agent() {
    assert_eq!(
        EventsToConcurrency::map_object(&EventConcept::Handler),
        crate::formal::information::concurrency::ConcurrencyConcept::Agent
    );
}

#[pr4xis::praxis_value(Verifiable, Extensible)]
#[test]
fn event_bus_is_synchronization() {
    assert_eq!(
        EventsToConcurrency::map_object(&EventConcept::EventBus),
        crate::formal::information::concurrency::ConcurrencyConcept::Synchronization
    );
}

// === Every system IS event-driven ===

#[pr4xis::praxis_value(Extensible)]
#[test]
fn systems_to_events_functor_laws() {
    assert_functor_laws::<SystemsToEvents>();
}

#[pr4xis::praxis_value(Verifiable, Extensible)]
#[test]
fn transition_is_event() {
    use crate::formal::systems::ontology::SystemConcept;
    assert_eq!(
        SystemsToEvents::map_object(&SystemConcept::Transition),
        EventConcept::Event
    );
}

#[pr4xis::praxis_value(Verifiable, Extensible)]
#[test]
fn feedback_is_event_bus() {
    use crate::formal::systems::ontology::SystemConcept;
    assert_eq!(
        SystemsToEvents::map_object(&SystemConcept::Feedback),
        EventConcept::EventBus
    );
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_event() -> impl Strategy<Value = EventConcept> {
        proptest::sample::select(EventConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_event()) {
            let id = EventCategory::identity(&c);
            prop_assert_eq!(EventCategory::compose(&id, &id), Some(id));
        }

        #[test]
        fn prop_events_to_concurrent_valid(concept in arb_event()) {
            let mapped = EventsToConcurrency::map_object(&concept);
            let variants = crate::formal::information::concurrency::ConcurrencyConcept::variants();
            prop_assert!(variants.contains(&mapped));
        }

        #[test]
        fn prop_events_to_concurrent_preserves_identity(concept in arb_event()) {
            let event_id = EventCategory::identity(&concept);
            let mapped = EventsToConcurrency::map_morphism(&event_id);
            let conc_id = crate::formal::information::concurrency::ConcurrencyCategory::identity(
                &EventsToConcurrency::map_object(&concept),
            );
            prop_assert_eq!(mapped, conc_id);
        }
    }

    pr4xis::register_praxis_value!(prop_identity_idempotent, Deterministic);
    pr4xis::register_praxis_value!(prop_events_to_concurrent_valid, Extensible);
    pr4xis::register_praxis_value!(prop_events_to_concurrent_preserves_identity, Extensible);
}
