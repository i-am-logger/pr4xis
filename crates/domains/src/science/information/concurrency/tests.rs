use pr4xis::category::entity::Entity;
use pr4xis::category::validate::{check_category_laws, check_functor_laws};
use pr4xis::category::{Category, Functor};

use super::chess_functor::*;
use super::ontology::*;
use super::systems_functor::*;

// =============================================================================
// Concurrency category tests
// =============================================================================

#[test]
fn concurrency_category_laws() {
    check_category_laws::<ConcurrencyCategory>().unwrap();
}

#[test]
fn concurrency_has_10_concepts() {
    assert_eq!(ConcurrencyConcept::variants().len(), 10);
}

#[test]
fn agent_acts_on_shared_resource() {
    let m = ConcurrencyCategory::morphisms();
    assert!(m.iter().any(|r| r.from == ConcurrencyConcept::Agent
        && r.to == ConcurrencyConcept::SharedResource
        && r.kind == ConcurrencyRelationKind::ActsOn));
}

#[test]
fn synchronization_controls_agent() {
    let m = ConcurrencyCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.from == ConcurrencyConcept::Synchronization
                && r.to == ConcurrencyConcept::Agent
                && r.kind == ConcurrencyRelationKind::Controls)
    );
}

#[test]
fn protocol_governs_action() {
    let m = ConcurrencyCategory::morphisms();
    assert!(m.iter().any(|r| r.from == ConcurrencyConcept::Protocol
        && r.to == ConcurrencyConcept::Action
        && r.kind == ConcurrencyRelationKind::Governs));
}

#[test]
fn deadlock_arises_from_synchronization() {
    let m = ConcurrencyCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.from == ConcurrencyConcept::Synchronization
                && r.to == ConcurrencyConcept::Deadlock)
    );
}

// =============================================================================
// Chess concurrent category tests
// =============================================================================

#[test]
fn chess_concurrent_category_laws() {
    check_category_laws::<ChessConcurrentCategory>().unwrap();
}

#[test]
fn chess_has_10_elements() {
    assert_eq!(ChessConcurrent::variants().len(), 10);
}

#[test]
fn turn_taking_controls_player() {
    let m = ChessConcurrentCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.from == ChessConcurrent::TurnTaking && r.to == ChessConcurrent::Player)
    );
}

#[test]
fn stalemate_is_deadlock() {
    // Stalemate in chess maps to Deadlock in concurrency
    assert_eq!(
        ChessToConcurrency::map_object(&ChessConcurrent::Stalemate),
        ConcurrencyConcept::Deadlock
    );
}

#[test]
fn opponent_response_is_future() {
    assert_eq!(
        ChessToConcurrency::map_object(&ChessConcurrent::OpponentResponse),
        ConcurrencyConcept::Future
    );
}

// =============================================================================
// THE PROOF: Chess IS concurrent
// =============================================================================

#[test]
fn chess_functor_laws_hold() {
    check_functor_laws::<ChessToConcurrency>().unwrap();
}

// =============================================================================
// THE PROOF: Every system IS concurrent
// =============================================================================

#[test]
fn systems_functor_laws_hold() {
    check_functor_laws::<SystemsToConcurrency>().unwrap();
}

#[test]
fn feedback_is_synchronization() {
    use crate::science::systems::ontology::SystemConcept;
    assert_eq!(
        SystemsToConcurrency::map_object(&SystemConcept::Feedback),
        ConcurrencyConcept::Synchronization
    );
}

#[test]
fn emergence_is_race_condition() {
    use crate::science::systems::ontology::SystemConcept;
    // Emergence depends on interaction order — just like race conditions
    assert_eq!(
        SystemsToConcurrency::map_object(&SystemConcept::Emergence),
        ConcurrencyConcept::RaceCondition
    );
}

#[test]
fn functor_preserves_identity() {
    for elem in ChessConcurrent::variants() {
        let chess_id = ChessConcurrentCategory::identity(&elem);
        let mapped = ChessToConcurrency::map_morphism(&chess_id);
        let conc_id = ConcurrencyCategory::identity(&ChessToConcurrency::map_object(&elem));
        assert_eq!(mapped, conc_id, "identity not preserved for {:?}", elem);
    }
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_concurrency() -> impl Strategy<Value = ConcurrencyConcept> {
        prop_oneof![
            Just(ConcurrencyConcept::Agent),
            Just(ConcurrencyConcept::SharedResource),
            Just(ConcurrencyConcept::Action),
            Just(ConcurrencyConcept::Synchronization),
            Just(ConcurrencyConcept::State),
            Just(ConcurrencyConcept::Protocol),
            Just(ConcurrencyConcept::Deadlock),
            Just(ConcurrencyConcept::RaceCondition),
            Just(ConcurrencyConcept::Future),
            Just(ConcurrencyConcept::Message),
        ]
    }

    fn arb_chess() -> impl Strategy<Value = ChessConcurrent> {
        prop_oneof![
            Just(ChessConcurrent::Player),
            Just(ChessConcurrent::Board),
            Just(ChessConcurrent::Move),
            Just(ChessConcurrent::TurnTaking),
            Just(ChessConcurrent::Position),
            Just(ChessConcurrent::Rules),
            Just(ChessConcurrent::Stalemate),
            Just(ChessConcurrent::TimePressure),
            Just(ChessConcurrent::OpponentResponse),
            Just(ChessConcurrent::MoveNotation),
        ]
    }

    proptest! {
        /// Identity is idempotent for concurrency.
        #[test]
        fn prop_identity_idempotent(c in arb_concurrency()) {
            let id = ConcurrencyCategory::identity(&c);
            let composed = ConcurrencyCategory::compose(&id, &id);
            prop_assert_eq!(composed, Some(id));
        }

        /// Functor maps every chess element to a valid concurrency concept.
        #[test]
        fn prop_functor_maps_valid(elem in arb_chess()) {
            let concept = ChessToConcurrency::map_object(&elem);
            prop_assert!(ConcurrencyConcept::variants().contains(&concept));
        }

        /// Functor preserves identity for all chess elements.
        #[test]
        fn prop_functor_preserves_identity(elem in arb_chess()) {
            let chess_id = ChessConcurrentCategory::identity(&elem);
            let mapped = ChessToConcurrency::map_morphism(&chess_id);
            let conc_id = ConcurrencyCategory::identity(&ChessToConcurrency::map_object(&elem));
            prop_assert_eq!(mapped, conc_id);
        }

        /// Functor is surjective: every concurrency concept has a chess element mapping to it.
        #[test]
        fn prop_functor_surjective(concept in arb_concurrency()) {
            let has_preimage = ChessConcurrent::variants()
                .iter()
                .any(|e| ChessToConcurrency::map_object(e) == concept);
            prop_assert!(has_preimage, "no chess element maps to {:?}", concept);
        }

        /// Functor preserves composition for any composable pair.
        #[test]
        fn prop_functor_preserves_composition(
            a in arb_chess(),
            b in arb_chess(),
            c in arb_chess()
        ) {
            let morphisms = ChessConcurrentCategory::morphisms();
            if let Some(f) = morphisms.iter().find(|m| m.from == a && m.to == b) {
                if let Some(g) = morphisms.iter().find(|m| m.from == b && m.to == c) {
                    if let Some(gf) = ChessConcurrentCategory::compose(f, g) {
                        let mapped_gf = ChessToConcurrency::map_morphism(&gf);
                        let composed_mapped = ConcurrencyCategory::compose(
                            &ChessToConcurrency::map_morphism(f),
                            &ChessToConcurrency::map_morphism(g),
                        );
                        prop_assert_eq!(Some(mapped_gf), composed_mapped);
                    }
                }
            }
        }

        /// Turn-taking (synchronization) prevents deadlock in chess.
        #[test]
        fn prop_turn_taking_is_synchronization(_dummy in 0..1i32) {
            prop_assert_eq!(
                ChessToConcurrency::map_object(&ChessConcurrent::TurnTaking),
                ConcurrencyConcept::Synchronization
            );
        }

        /// SystemsToConcurrency maps every system concept to a valid concurrency concept.
        #[test]
        fn prop_systems_functor_valid(concept in arb_system_concept()) {
            let mapped = SystemsToConcurrency::map_object(&concept);
            prop_assert!(ConcurrencyConcept::variants().contains(&mapped));
        }

        /// SystemsToConcurrency preserves identity.
        #[test]
        fn prop_systems_functor_preserves_identity(concept in arb_system_concept()) {
            use crate::science::systems::ontology::SystemsCategory;
            let sys_id = SystemsCategory::identity(&concept);
            let mapped = SystemsToConcurrency::map_morphism(&sys_id);
            let conc_id = ConcurrencyCategory::identity(&SystemsToConcurrency::map_object(&concept));
            prop_assert_eq!(mapped, conc_id);
        }
    }

    use crate::science::systems::ontology::SystemConcept;

    fn arb_system_concept() -> impl Strategy<Value = SystemConcept> {
        prop_oneof![
            Just(SystemConcept::Component),
            Just(SystemConcept::Interaction),
            Just(SystemConcept::State),
            Just(SystemConcept::Transition),
            Just(SystemConcept::Constraint),
            Just(SystemConcept::Feedback),
            Just(SystemConcept::Homeostasis),
            Just(SystemConcept::Emergence),
            Just(SystemConcept::Boundary),
            Just(SystemConcept::Controller),
        ]
    }
}
