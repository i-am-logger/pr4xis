use pr4xis::category::entity::Entity;
use pr4xis::category::validate::{check_category_laws, check_functor_laws};
use pr4xis::category::{Category, Functor};

use super::chess_functor::*;
use super::concurrent_functor::*;
use super::ontology::*;
use super::systems_functor::*;

#[test]
fn event_category_laws() {
    check_category_laws::<EventCategory>().unwrap();
}

#[test]
fn event_has_10_concepts() {
    assert_eq!(EventConcept::variants().len(), 10);
}

#[test]
fn command_triggers_event() {
    let m = EventCategory::morphisms();
    assert!(m.iter().any(|r| r.from == EventConcept::Command
        && r.to == EventConcept::Event
        && r.kind == EventRelationKind::Triggers));
}

#[test]
fn event_appended_to_log() {
    let m = EventCategory::morphisms();
    assert!(m.iter().any(|r| r.from == EventConcept::Event
        && r.to == EventConcept::EventLog
        && r.kind == EventRelationKind::AppendedTo));
}

// === Chess IS event-driven ===

#[test]
fn chess_event_category_laws() {
    check_category_laws::<ChessEventCategory>().unwrap();
}

#[test]
fn chess_to_events_functor_laws() {
    check_functor_laws::<ChessToEvents>().unwrap();
}

#[test]
fn pgn_is_event_log() {
    assert_eq!(
        ChessToEvents::map_object(&ChessEvent::GameRecord),
        EventConcept::EventLog
    );
}

#[test]
fn move_is_event() {
    assert_eq!(
        ChessToEvents::map_object(&ChessEvent::Move),
        EventConcept::Event
    );
}

// === Every event-driven system IS concurrent ===

#[test]
fn events_to_concurrency_functor_laws() {
    check_functor_laws::<EventsToConcurrency>().unwrap();
}

#[test]
fn handler_is_agent() {
    assert_eq!(
        EventsToConcurrency::map_object(&EventConcept::Handler),
        crate::science::information::concurrency::ConcurrencyConcept::Agent
    );
}

#[test]
fn event_bus_is_synchronization() {
    assert_eq!(
        EventsToConcurrency::map_object(&EventConcept::EventBus),
        crate::science::information::concurrency::ConcurrencyConcept::Synchronization
    );
}

// === Every system IS event-driven ===

#[test]
fn systems_to_events_functor_laws() {
    check_functor_laws::<SystemsToEvents>().unwrap();
}

#[test]
fn transition_is_event() {
    use crate::science::systems::ontology::SystemConcept;
    assert_eq!(
        SystemsToEvents::map_object(&SystemConcept::Transition),
        EventConcept::Event
    );
}

#[test]
fn feedback_is_event_bus() {
    use crate::science::systems::ontology::SystemConcept;
    assert_eq!(
        SystemsToEvents::map_object(&SystemConcept::Feedback),
        EventConcept::EventBus
    );
}

// === The full triangle: System ↔ EventDriven ↔ Concurrent ===

#[test]
fn chess_is_concurrent_via_events() {
    // Chess → EventDriven (proven by ChessToEvents functor)
    // EventDriven → Concurrent (proven by EventsToConcurrency functor)
    // Therefore: Chess → Concurrent (by functor composition)

    // Verify the composed mapping:
    let chess_move = ChessEvent::Move;
    let event = ChessToEvents::map_object(&chess_move);
    let concurrent = EventsToConcurrency::map_object(&event);

    // A chess move IS a message in a concurrent system
    assert_eq!(
        concurrent,
        crate::science::information::concurrency::ConcurrencyConcept::Message
    );
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_event() -> impl Strategy<Value = EventConcept> {
        prop_oneof![
            Just(EventConcept::Event),
            Just(EventConcept::Command),
            Just(EventConcept::State),
            Just(EventConcept::Handler),
            Just(EventConcept::EventLog),
            Just(EventConcept::EventBus),
            Just(EventConcept::Projection),
            Just(EventConcept::Subscription),
            Just(EventConcept::Saga),
            Just(EventConcept::EventSchema),
        ]
    }

    fn arb_chess_event() -> impl Strategy<Value = ChessEvent> {
        prop_oneof![
            Just(ChessEvent::Move),
            Just(ChessEvent::MoveAttempt),
            Just(ChessEvent::Position),
            Just(ChessEvent::RulesEngine),
            Just(ChessEvent::GameRecord),
            Just(ChessEvent::Game),
            Just(ChessEvent::Evaluation),
            Just(ChessEvent::CheckDetection),
            Just(ChessEvent::GamePhase),
            Just(ChessEvent::NotationRules),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_event()) {
            let id = EventCategory::identity(&c);
            prop_assert_eq!(EventCategory::compose(&id, &id), Some(id));
        }

        #[test]
        fn prop_chess_functor_valid(elem in arb_chess_event()) {
            let concept = ChessToEvents::map_object(&elem);
            prop_assert!(EventConcept::variants().contains(&concept));
        }

        #[test]
        fn prop_chess_functor_preserves_identity(elem in arb_chess_event()) {
            let chess_id = ChessEventCategory::identity(&elem);
            let mapped = ChessToEvents::map_morphism(&chess_id);
            let event_id = EventCategory::identity(&ChessToEvents::map_object(&elem));
            prop_assert_eq!(mapped, event_id);
        }

        #[test]
        fn prop_chess_functor_surjective(concept in arb_event()) {
            let has = ChessEvent::variants().iter().any(|e| ChessToEvents::map_object(e) == concept);
            prop_assert!(has, "no chess element maps to {:?}", concept);
        }

        #[test]
        fn prop_events_to_concurrent_valid(concept in arb_event()) {
            let mapped = EventsToConcurrency::map_object(&concept);
            let variants = crate::science::information::concurrency::ConcurrencyConcept::variants();
            prop_assert!(variants.contains(&mapped));
        }

        #[test]
        fn prop_events_to_concurrent_preserves_identity(concept in arb_event()) {
            let event_id = EventCategory::identity(&concept);
            let mapped = EventsToConcurrency::map_morphism(&event_id);
            let conc_id = crate::science::information::concurrency::ConcurrencyCategory::identity(
                &EventsToConcurrency::map_object(&concept),
            );
            prop_assert_eq!(mapped, conc_id);
        }
    }
}
