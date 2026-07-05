//! Property-based tests for the Bus ontology, the broker-simulator
//! engine, and the `Bus → System` functor.

#![cfg(test)]

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use proptest::prelude::*;

use super::engine::{
    Delivery, EXACTLY_ONE_DELIVERY, FIXTURE_SUBSCRIBER, MessageId, delivered_count, run_scenario,
    scenario_messages,
};
use super::ontology::{
    BusCategory, BusConcept, BusOntology, DeliverySemantics, IsSpaceDecoupled, RoutingStrategy,
};
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Ontology, Quality};

fn arb_concept() -> impl Strategy<Value = BusConcept> {
    proptest::sample::select(BusConcept::variants())
}

/// A scenario shape: `count` published messages with distinct dropped
/// and unacknowledged indices — the generalisation of the canonical
/// fixture the behavioural axiom pins (Birman & Joseph 1987).
fn arb_scenario() -> impl Strategy<Value = (usize, MessageId, MessageId)> {
    (2usize..6)
        .prop_flat_map(|count| (Just(count), 0..count, 0..count))
        .prop_filter(
            "the dropped and unacknowledged messages must be distinct",
            |(_, dropped, unacked)| dropped != unacked,
        )
        .prop_map(|(count, dropped, unacked)| (count, MessageId(dropped), MessageId(unacked)))
}

proptest! {
    /// `RoutingStrategy` is defined exactly on the two routing
    /// disciplines (Eugster et al. 2003 sec 4.1; Carzaniga et al. 2001)
    /// and nowhere else — including not on the abstract `MessageBus`
    /// parent.
    #[test]
    fn prop_routing_strategy_exactly_on_the_two_routings(c in arb_concept()) {
        use BusConcept as C;
        let is_routing = matches!(c, C::TopicBasedRouting | C::ContentBasedRouting);
        prop_assert_eq!(RoutingStrategy.get(&c).is_some(), is_routing);
    }

    /// `DeliverySemantics` is defined exactly on the three delivery
    /// guarantees (Birman & Joseph 1987) and nowhere else — including
    /// not on the abstract `DeliveryGuarantee` parent.
    #[test]
    fn prop_delivery_semantics_exactly_on_the_three_guarantees(c in arb_concept()) {
        use BusConcept as C;
        let is_guarantee = matches!(c, C::AtMostOnce | C::AtLeastOnce | C::ExactlyOnce);
        prop_assert_eq!(DeliverySemantics.get(&c).is_some(), is_guarantee);
    }

    /// `IsSpaceDecoupled` partitions the communication parties: the
    /// pub/sub parties and media are `Some(true)` (Eugster et al. 2003
    /// sec 2), the point-to-point actor pair is `Some(false)` (Hewitt
    /// et al. 1973), and everything else is `None`.
    #[test]
    fn prop_space_decoupling_partition(c in arb_concept()) {
        use BusConcept as C;
        let expected = match c {
            C::Publisher | C::Subscriber | C::MessageBus | C::Broker | C::Topic => Some(true),
            C::Actor | C::Mailbox => Some(false),
            _ => None,
        };
        prop_assert_eq!(IsSpaceDecoupled.get(&c), expected);
    }

    /// Every arrow of the category carries a non-empty name.
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in BusCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    /// Structural + domain axioms all discharge, regardless of the
    /// sampling that drives the test.
    #[test]
    fn prop_all_axioms_hold(_seed in 0..16u32) {
        for axiom in BusOntology::axioms() {
            if let Err(c) = axiom.verify() {
                prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
            }
        }
    }

    /// The three delivery contracts hold over a family of scenarios,
    /// not just the canonical fixture (Birman & Joseph 1987): for any
    /// message count and any distinct (dropped, unacknowledged) pair —
    /// at-most-once loses the dropped message and never duplicates;
    /// at-least-once loses nothing but duplicates the unacknowledged
    /// message; exactly-once (with endpoint dedup) hands every message
    /// over exactly once.
    #[test]
    fn prop_delivery_contracts_over_scenarios((count, dropped, unacked) in arb_scenario()) {
        let messages = scenario_messages(count);

        let amo = run_scenario(Delivery::AtMostOnce, count, dropped, unacked)
            .expect("the at-most-once scenario must run");
        prop_assert_eq!(delivered_count(&amo, FIXTURE_SUBSCRIBER, dropped), 0);
        for m in &messages {
            prop_assert!(
                delivered_count(&amo, FIXTURE_SUBSCRIBER, *m) <= EXACTLY_ONE_DELIVERY,
                "at-most-once must never duplicate"
            );
        }

        let alo = run_scenario(Delivery::AtLeastOnce, count, dropped, unacked)
            .expect("the at-least-once scenario must run");
        for m in &messages {
            prop_assert!(
                delivered_count(&alo, FIXTURE_SUBSCRIBER, *m) >= EXACTLY_ONE_DELIVERY,
                "at-least-once must never lose"
            );
        }
        prop_assert!(
            delivered_count(&alo, FIXTURE_SUBSCRIBER, unacked) > EXACTLY_ONE_DELIVERY,
            "the unacknowledged message must duplicate under at-least-once"
        );

        let eo = run_scenario(Delivery::ExactlyOnce, count, dropped, unacked)
            .expect("the exactly-once scenario must run");
        for m in &messages {
            prop_assert_eq!(
                delivered_count(&eo, FIXTURE_SUBSCRIBER, *m),
                EXACTLY_ONE_DELIVERY,
                "exactly-once must hand each message over exactly once"
            );
        }
    }

    /// The exactly-once inbox is duplicate-free in *order*, not only in
    /// count: the sequence handed to the application never repeats an
    /// identity (Birman & Joseph 1987: endpoint dedup suppresses
    /// re-deliveries).
    #[test]
    fn prop_exactly_once_inbox_is_duplicate_free((count, dropped, unacked) in arb_scenario()) {
        let eo = run_scenario(Delivery::ExactlyOnce, count, dropped, unacked)
            .expect("the exactly-once scenario must run");
        for inbox in &eo.inboxes {
            for (i, m) in inbox.delivered.iter().enumerate() {
                prop_assert!(
                    inbox.delivered.iter().skip(i + 1).all(|n| n != m),
                    "identity {m:?} appears twice in an exactly-once inbox"
                );
            }
        }
    }
}

pr4xis::register_praxis_value!(
    prop_routing_strategy_exactly_on_the_two_routings,
    Verifiable
);
pr4xis::register_praxis_value!(
    prop_delivery_semantics_exactly_on_the_three_guarantees,
    Verifiable
);
pr4xis::register_praxis_value!(prop_space_decoupling_partition, Verifiable);
pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
pr4xis::register_praxis_value!(prop_all_axioms_hold, Verifiable);
pr4xis::register_praxis_value!(prop_delivery_contracts_over_scenarios, Verifiable);
pr4xis::register_praxis_value!(prop_exactly_once_inbox_is_duplicate_free, Verifiable);
