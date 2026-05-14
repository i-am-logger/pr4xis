use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, Concept, Functor};

use super::ontology::*;
use super::systems_functor::*;

// =============================================================================
// Concurrency category tests
// =============================================================================

#[test]
fn concurrency_category_laws() {
    assert_category_laws::<ConcurrencyCategory>();
}

#[test]
fn concurrency_has_10_concepts() {
    assert_eq!(ConcurrencyConcept::variants().len(), 10);
}

#[test]
fn agent_acts_on_shared_resource() {
    let m = ConcurrencyCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == ConcurrencyConcept::Agent
        && r.target() == ConcurrencyConcept::SharedResource
        && r.kind() == ConcurrencyRelationKind::ActsOn));
}

#[test]
fn synchronization_controls_agent() {
    let m = ConcurrencyCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.source() == ConcurrencyConcept::Synchronization
                && r.target() == ConcurrencyConcept::Agent
                && r.kind() == ConcurrencyRelationKind::Controls)
    );
}

#[test]
fn protocol_governs_action() {
    let m = ConcurrencyCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == ConcurrencyConcept::Protocol
        && r.target() == ConcurrencyConcept::Action
        && r.kind() == ConcurrencyRelationKind::Governs));
}

#[test]
fn deadlock_arises_from_synchronization() {
    // Coffman, Elphick & Shoshani (1971) System Deadlocks.
    let m = ConcurrencyCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.source() == ConcurrencyConcept::Synchronization
                && r.target() == ConcurrencyConcept::Deadlock)
    );
}

// =============================================================================
// THE PROOF: Every system IS concurrent
// =============================================================================

#[test]
fn systems_functor_laws_hold() {
    use pr4xis::category::laws::assert_functor_laws;
    assert_functor_laws::<SystemsToConcurrency>();
}

#[test]
fn feedback_is_synchronization() {
    use crate::formal::systems::ontology::SystemConcept;
    assert_eq!(
        SystemsToConcurrency::map_object(&SystemConcept::Feedback),
        ConcurrencyConcept::Synchronization
    );
}

#[test]
fn emergence_is_race_condition() {
    use crate::formal::systems::ontology::SystemConcept;
    // Lamport (1978): emergence depends on interaction order — just like
    // race conditions.
    assert_eq!(
        SystemsToConcurrency::map_object(&SystemConcept::Emergence),
        ConcurrencyConcept::RaceCondition
    );
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_concurrency() -> impl Strategy<Value = ConcurrencyConcept> {
        proptest::sample::select(ConcurrencyConcept::variants())
    }

    use crate::formal::systems::ontology::SystemConcept;

    fn arb_system_concept() -> impl Strategy<Value = SystemConcept> {
        proptest::sample::select(SystemConcept::variants())
    }

    proptest! {
        /// Identity is idempotent for concurrency.
        #[test]
        fn prop_identity_idempotent(c in arb_concurrency()) {
            let id = ConcurrencyCategory::identity(&c);
            let composed = ConcurrencyCategory::compose(&id, &id);
            prop_assert_eq!(composed, Some(id));
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
            use crate::formal::systems::ontology::SystemCategory;
            let sys_id = SystemCategory::identity(&concept);
            let mapped = SystemsToConcurrency::map_morphism(&sys_id);
            let conc_id = ConcurrencyCategory::identity(&SystemsToConcurrency::map_object(&concept));
            prop_assert_eq!(mapped, conc_id);
        }
    }
}
