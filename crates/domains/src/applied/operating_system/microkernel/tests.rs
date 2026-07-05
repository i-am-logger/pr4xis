//! Property-based tests for the Microkernel ontology, engine, and
//! functors.

#![cfg(test)]

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use proptest::prelude::*;

use super::engine::{
    AddressSpaceId, EndpointId, FIXTURE_ADDRESS_SPACE_COUNT, FIXTURE_ENDPOINT_COUNT,
    FIXTURE_THREAD_COUNT, KernelAction, KernelMessage, ThreadId, apply, enabled_actions,
    fixture_payload, kernel_initial,
};
use super::ontology::{
    IsMechanism, KernelPrivilege, MicrokernelCategory, MicrokernelConcept, MicrokernelOntology,
    Privilege,
};
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Ontology, Quality};

fn arb_concept() -> impl Strategy<Value = MicrokernelConcept> {
    proptest::sample::select(MicrokernelConcept::variants())
}

/// The documented `KernelPrivilege` classification, restated once so
/// the proptest checks the quality's totality against an independent
/// listing (Liedtke 1995 §2–3; Levin et al. 1975; Dijkstra 1968).
fn expected_privilege(c: &MicrokernelConcept) -> Option<Privilege> {
    use MicrokernelConcept as C;
    match c {
        C::AddressSpace
        | C::Thread
        | C::Ipc
        | C::Scheduler
        | C::Kernel
        | C::Microkernel
        | C::MonolithicKernel
        | C::Nucleus
        | C::PrivilegedMode => Some(Privilege::Privileged),
        C::UserServer | C::Pager | C::Policy | C::UserMode => Some(Privilege::UserSpace),
        C::Capability | C::Message | C::Endpoint | C::Mechanism | C::TrustedComputingBase => None,
    }
}

proptest! {
    /// KernelPrivilege is total over the classification the ontology
    /// documents: every concept gets exactly its specified value, and
    /// exactly the five mode-neutral concepts get None.
    #[test]
    fn prop_kernel_privilege_totality(c in arb_concept()) {
        prop_assert_eq!(KernelPrivilege.get(&c), expected_privilege(&c));
    }

    /// IsMechanism is defined exactly on the mechanism/policy
    /// classification (Levin et al. 1975) and nowhere else.
    #[test]
    fn prop_is_mechanism_exactly_on_classified(c in arb_concept()) {
        use MicrokernelConcept as C;
        let classified = matches!(c, C::Mechanism | C::Ipc | C::Scheduler | C::Policy);
        prop_assert_eq!(IsMechanism.get(&c).is_some(), classified);
    }

    /// Every arrow of the category carries a non-empty name.
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in MicrokernelCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    /// Structural + domain axioms all discharge, regardless of the
    /// sampling that drives the test.
    #[test]
    fn prop_all_axioms_hold(_seed in 0..16u32) {
        for axiom in MicrokernelOntology::axioms() {
            if let Err(c) = axiom.verify() {
                prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
            }
        }
    }

    /// Determinism of the kernel transition: applying the same enabled
    /// entry to the same situation always yields the same successor,
    /// along a bounded random walk from the fixture initial state.
    #[test]
    fn prop_step_is_deterministic(picks in proptest::collection::vec(any::<prop::sample::Index>(), 0..16)) {
        let mut situation = kernel_initial();
        for pick in picks {
            let actions = enabled_actions(&situation);
            // Switch is always enabled, so the walk never dead-ends.
            prop_assert!(!actions.is_empty());
            let action = &actions[pick.index(actions.len())];
            let a = apply(&situation, action);
            let b = apply(&situation, action);
            prop_assert_eq!(a.clone(), b);
            situation = a.unwrap_or_else(|e| panic!("enabled action must apply: {e}"));
        }
    }

    /// The kernel-mediation invariant along every bounded random walk:
    /// each delivered message names an existing mediating endpoint —
    /// there is no delivery path around the endpoint queues (Brinch
    /// Hansen 1970).
    #[test]
    fn prop_every_delivery_is_mediated(picks in proptest::collection::vec(any::<prop::sample::Index>(), 0..16)) {
        let mut situation = kernel_initial();
        for pick in picks {
            let actions = enabled_actions(&situation);
            prop_assert!(!actions.is_empty());
            let action = &actions[pick.index(actions.len())];
            situation = apply(&situation, action)
                .unwrap_or_else(|e| panic!("enabled action must apply: {e}"));
            prop_assert!(situation.every_delivery_is_endpoint_mediated());
        }
    }

    /// Address-space isolation: a Send whose buffer lies in any space
    /// other than the sender's own is rejected, for every thread and
    /// every foreign space (Liedtke 1995 §2.1).
    #[test]
    fn prop_foreign_buffer_send_rejected(
        thread in 0..FIXTURE_THREAD_COUNT,
        space_offset in 1..FIXTURE_ADDRESS_SPACE_COUNT,
        endpoint in 0..FIXTURE_ENDPOINT_COUNT,
    ) {
        let mut situation = kernel_initial();
        // Make the probed thread the running one — the isolation guard
        // must reject the send even for the legitimate caller.
        situation = apply(&situation, &KernelAction::Switch { to: ThreadId(thread) })
            .unwrap_or_else(|e| panic!("switch must apply: {e}"));
        let own_space = situation.threads[thread].space;
        let foreign = AddressSpaceId((own_space.0 + space_offset) % FIXTURE_ADDRESS_SPACE_COUNT);
        prop_assert!(foreign != own_space);
        let result = apply(&situation, &KernelAction::Send {
            from: ThreadId(thread),
            endpoint: EndpointId(endpoint),
            msg: KernelMessage { buffer_space: foreign, payload: fixture_payload(ThreadId(thread)) },
        });
        prop_assert!(result.is_err());
    }
}

pr4xis::register_praxis_value!(prop_kernel_privilege_totality, Verifiable);
pr4xis::register_praxis_value!(prop_is_mechanism_exactly_on_classified, Verifiable);
pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
pr4xis::register_praxis_value!(prop_all_axioms_hold, Verifiable);
pr4xis::register_praxis_value!(prop_step_is_deterministic, Deterministic);
pr4xis::register_praxis_value!(prop_every_delivery_is_mediated, Verifiable);
pr4xis::register_praxis_value!(prop_foreign_buffer_send_rejected, Verifiable);

/// The canonical client–server round trip (Liedtke 1995 §3; Haertig et
/// al. 1997): the client sends via the endpoint, the kernel switches to
/// the server, the server receives — the payload arrives with full
/// mediation provenance and the delivery crosses an address-space
/// boundary.
#[pr4xis::praxis_value(Verifiable, Deterministic)]
#[test]
fn client_server_round_trip_is_kernel_mediated() {
    let client = ThreadId(0);
    let server = ThreadId(1);
    let endpoint = EndpointId(0);

    let s0 = kernel_initial();
    let client_space = s0.threads[client.0].space;
    let s1 = apply(
        &s0,
        &KernelAction::Send {
            from: client,
            endpoint,
            msg: KernelMessage {
                buffer_space: client_space,
                payload: fixture_payload(client),
            },
        },
    )
    .unwrap_or_else(|e| panic!("send must apply: {e}"));
    let s2 = apply(&s1, &KernelAction::Switch { to: server })
        .unwrap_or_else(|e| panic!("switch must apply: {e}"));
    let s3 = apply(
        &s2,
        &KernelAction::Receive {
            thread: server,
            endpoint,
        },
    )
    .unwrap_or_else(|e| panic!("receive must apply: {e}"));

    let delivered = &s3.threads[server.0].delivered;
    assert_eq!(delivered.len(), 1);
    assert_eq!(delivered[0].sender, client);
    assert_eq!(delivered[0].via, endpoint);
    assert_eq!(delivered[0].payload, fixture_payload(client));
    assert!(s3.every_delivery_is_endpoint_mediated());
    // Non-vacuity: the round trip actually crossed an isolation
    // boundary (client and server live in different address spaces).
    assert_eq!(s3.cross_space_delivery_count(), 1);
}

/// A receive with nothing queued is rejected — nothing can be delivered
/// that did not first pass through the endpoint (Brinch Hansen 1970:
/// `wait message` delays the receiver).
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn receive_on_empty_endpoint_is_rejected() {
    let situation = kernel_initial();
    let result = apply(
        &situation,
        &KernelAction::Receive {
            thread: situation.current,
            endpoint: EndpointId(0),
        },
    );
    assert!(result.is_err());
}

/// Out-of-range object ids are rejected, never panicking — the guard
/// paths of the engine's [`apply`].
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn out_of_range_ids_are_rejected() {
    let situation = kernel_initial();
    let bad_thread = ThreadId(situation.threads.len());
    let bad_endpoint = EndpointId(situation.endpoints.len());
    let own_space = situation.threads[situation.current.0].space;
    assert!(apply(&situation, &KernelAction::Switch { to: bad_thread },).is_err());
    assert!(
        apply(
            &situation,
            &KernelAction::Receive {
                thread: situation.current,
                endpoint: bad_endpoint,
            },
        )
        .is_err()
    );
    assert!(
        apply(
            &situation,
            &KernelAction::Send {
                from: situation.current,
                endpoint: bad_endpoint,
                msg: KernelMessage {
                    buffer_space: own_space,
                    payload: fixture_payload(situation.current),
                },
            },
        )
        .is_err()
    );
}

/// Only the running thread can trap into the kernel: a send or receive
/// on behalf of a thread that is not current is rejected.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn non_running_thread_cannot_enter_kernel() {
    let situation = kernel_initial();
    let other = ThreadId(1);
    assert_ne!(situation.current, other);
    let other_space = situation.threads[other.0].space;
    assert!(
        apply(
            &situation,
            &KernelAction::Send {
                from: other,
                endpoint: EndpointId(0),
                msg: KernelMessage {
                    buffer_space: other_space,
                    payload: fixture_payload(other),
                },
            },
        )
        .is_err()
    );
    assert!(
        apply(
            &situation,
            &KernelAction::Receive {
                thread: other,
                endpoint: EndpointId(0),
            },
        )
        .is_err()
    );
}
