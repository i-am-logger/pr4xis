//! Property-based tests for the Concurrency ontology, engine, and
//! Concurrency → System functor.

#![cfg(test)]

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use proptest::prelude::*;

use super::engine::{StepProcess, apply_step, enabled_actions, explore, mutex_initial};
use super::ontology::{
    ConcurrencyCategory, ConcurrencyConcept, ConcurrencyOntology, IsBlockingPrimitive,
    PropertyKind, TemporalPropertyKind,
};
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Ontology, Quality};

fn arb_concept() -> impl Strategy<Value = ConcurrencyConcept> {
    proptest::sample::select(ConcurrencyConcept::variants())
}

proptest! {
    /// PropertyKind is defined exactly on the temporal-property concepts
    /// (Alpern & Schneider 1985 dichotomy) and classifies each on the
    /// correct side: Safety for the safety pole, MutualExclusion (is_a
    /// SafetyProperty) and Deadlock (a deadlocked state is a discrete
    /// "bad thing"); Liveness for the liveness pole and Livelock
    /// (Lamport 1977).
    #[test]
    fn prop_property_kind_exactly_on_properties(c in arb_concept()) {
        use ConcurrencyConcept as C;
        use TemporalPropertyKind as K;
        let expected = match c {
            C::SafetyProperty | C::MutualExclusion | C::Deadlock => Some(K::Safety),
            C::LivenessProperty | C::Livelock => Some(K::Liveness),
            _ => None,
        };
        prop_assert_eq!(PropertyKind.get(&c), expected);
    }

    /// IsBlockingPrimitive is defined exactly on the three concrete
    /// mechanisms (Dijkstra 1968; Hoare 1974).
    #[test]
    fn prop_blocking_primitive_exactly_on_mechanisms(c in arb_concept()) {
        use ConcurrencyConcept as C;
        let is_mechanism = matches!(c, C::Semaphore | C::Monitor | C::Lock);
        prop_assert_eq!(IsBlockingPrimitive.get(&c).is_some(), is_mechanism);
    }

    /// Every arrow of the category carries a non-empty name.
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in ConcurrencyCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    /// Structural + domain axioms all discharge, regardless of the
    /// sampling that drives the test.
    #[test]
    fn prop_all_axioms_hold(_seed in 0..16u32) {
        for axiom in ConcurrencyOntology::axioms() {
            if let Err(c) = axiom.verify() {
                prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
            }
        }
    }

    /// Determinism of the interleaving engine: applying the same
    /// enabled action to the same situation always yields the same
    /// successor — Milner (1980): nondeterminism lives in the CHOICE
    /// of action, never inside a single action.
    #[test]
    fn prop_step_is_deterministic(pick in any::<prop::sample::Index>()) {
        let states = explore(&mutex_initial());
        let state = &states[pick.index(states.len())];
        for action in enabled_actions(state) {
            let a = apply_step(state, &action);
            let b = apply_step(state, &action);
            prop_assert_eq!(a, b);
        }
    }

    /// The semaphore invariant over every reachable state: the
    /// semaphore is taken exactly when some process occupies the
    /// protected region (Dijkstra 1968).
    #[test]
    fn prop_semaphore_matches_occupancy(pick in any::<prop::sample::Index>()) {
        use super::engine::BinarySemaphore;
        let states = explore(&mutex_initial());
        let state = &states[pick.index(states.len())];
        let taken = state.semaphore == BinarySemaphore::Taken;
        prop_assert_eq!(taken, state.critical_occupancy().value > 0.0);
    }
}

pr4xis::register_praxis_value!(prop_property_kind_exactly_on_properties, Verifiable);
pr4xis::register_praxis_value!(prop_blocking_primitive_exactly_on_mechanisms, Verifiable);
pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
pr4xis::register_praxis_value!(prop_all_axioms_hold, Verifiable);
pr4xis::register_praxis_value!(prop_step_is_deterministic, Deterministic);
pr4xis::register_praxis_value!(prop_semaphore_matches_occupancy, Verifiable);

/// Out-of-range process ids are rejected, never panicking — the guard
/// path of the engine's `apply_step`.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn out_of_range_process_is_rejected() {
    use super::engine::ProcessId;
    let initial = mutex_initial();
    let action = StepProcess {
        process: ProcessId(initial.processes.len()),
    };
    assert!(apply_step(&initial, &action).is_err());
}
