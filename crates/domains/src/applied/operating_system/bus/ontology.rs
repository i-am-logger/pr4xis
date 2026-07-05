//! Bus — the event/message bus: a medium decoupling senders from
//! receivers. One ontology covers both faces of the same structure:
//! the kernel IPC bus and the publish/subscribe event bus.
//!
//! Source traditions:
//!
//! - **Eugster, Felber, Guerraoui & Kermarrec (2003)** *The Many Faces
//!   of Publish/Subscribe*, ACM Computing Surveys 35(2) — the
//!   decoupling analysis (§2), topic-based addressing (§4.1), and the
//!   publisher/subscriber/broker vocabulary.
//! - **Carzaniga, Rosenblum & Wolf (2001)** *Design and Evaluation of a
//!   Wide-Area Event Notification Service*, ACM TOCS 19(3) (SIENA) —
//!   subscriptions as predicates and content-based routing.
//! - **Hewitt, Bishop & Steiger (1973)** *A Universal Modular ACTOR
//!   Formalism for Artificial Intelligence*, IJCAI — actors and
//!   messages.
//! - **Agha (1986)** *Actors: A Model of Concurrent Computation in
//!   Distributed Systems*, MIT Press — the actor's mail queue at its
//!   mail address (the mailbox).
//! - **Spector (1982)** *Performing Remote Operations Efficiently on a
//!   Local Computer Network*, CACM 25(4), and **Birrell & Nelson
//!   (1984)** *Implementing Remote Procedure Calls*, ACM TOCS 2(1) — the
//!   remote-operation call semantics (retransmission, duplicate
//!   suppression) and the at-most-once / at-least-once / exactly-once
//!   trichotomy, whose modern pub/sub statement is OASIS MQTT 5.0 §4.3
//!   (QoS 0/1/2).
//! - **Birman & Joseph (1987)** *Reliable Communication in the Presence
//!   of Failures*, ACM TOCS 5(1) (ISIS) — atomic all-or-nothing
//!   delivery (exactly-once at every operational destination) and the
//!   broadcast primitives.
//! - **Birman & Joseph (1987)** *Exploiting Virtual Synchrony in
//!   Distributed Systems*, SOSP — virtual synchrony (ordered, gap-free
//!   multicast within a view).
//!
//! The behavioural axiom is discharged against the broker simulator in
//! [`super::engine`]: the same published message sequence run under the
//! three delivery semantics exhibits loss (at-most-once), duplication
//! (at-least-once), and exactly one hand-off (exactly-once with
//! endpoint dedup).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use super::engine::{
    BusAction, DROPPED_MESSAGE, Delivery, EXACTLY_ONE_DELIVERY, FIXTURE_MESSAGE_COUNT,
    FIXTURE_SUBSCRIBER, UNACKED_MESSAGE, apply, delivered_count, run_fixture, scenario_messages,
};

pr4xis::ontology! {
    name: "Bus",
    source: "Eugster, Felber, Guerraoui & Kermarrec (2003) ACM Computing Surveys 35(2); Carzaniga, Rosenblum & Wolf (2001) ACM TOCS 19(3); Hewitt, Bishop & Steiger (1973) IJCAI; Agha (1986) Actors (MIT Press); Spector (1982) CACM 25(4); Birrell & Nelson (1984) ACM TOCS 2(1); OASIS MQTT 5.0 sec 4.3; Birman & Joseph (1987) ACM TOCS 5(1); Birman & Joseph (1987) SOSP",

    concepts: [
        // === The medium and its intermediary (Eugster et al. 2003) ===
        MessageBus,
        Broker,
        Decoupling,

        // === What travels (Eugster 2003; Hewitt et al. 1973) ===
        Event,
        Message,

        // === Pub/sub roles and addressing (Eugster 2003; Carzaniga 2001) ===
        Topic,
        Publisher,
        Subscriber,
        Subscription,
        TopicBasedRouting,
        ContentBasedRouting,

        // === Delivery quality of service (Spector 1982; Birrell &
        //     Nelson 1984; OASIS MQTT 5.0 sec 4.3; Birman & Joseph 1987
        //     for atomic exactly-once) ===
        DeliveryGuarantee,
        AtMostOnce,
        AtLeastOnce,
        ExactlyOnce,
        VirtualSynchrony,

        // === The actor face (Hewitt et al. 1973) ===
        Actor,
        Mailbox,
    ],

    labels: {
        MessageBus: ("en", "Message bus", "Eugster, Felber, Guerraoui & Kermarrec (2003) 'The Many Faces of Publish/Subscribe', ACM Computing Surveys 35(2) sec 1: a medium decoupling senders and receivers - producers hand messages to the bus, never to each other."),
        Broker: ("en", "Broker", "Eugster et al. (2003) ACM CSUR 35(2) sec 2: the routing intermediary - it stores registered interests and forwards events between producers and consumers."),
        Event: ("en", "Event", "Eugster et al. (2003) ACM CSUR 35(2): an asynchronous notification of a state change, propagated from publishers to the subscribers whose interest it matches."),
        Message: ("en", "Message", "Hewitt, Bishop & Steiger (1973) IJCAI 'A Universal Modular ACTOR Formalism for Artificial Intelligence': a self-contained payload - the sole unit of communication between actors."),
        Topic: ("en", "Topic", "Eugster et al. (2003) ACM CSUR 35(2) sec 4.1: a named channel - topic-based addressing groups events by channel name."),
        Publisher: ("en", "Publisher", "Eugster et al. (2003) ACM CSUR 35(2): the producer emitting events into the bus, without knowledge of who consumes them."),
        Subscriber: ("en", "Subscriber", "Eugster et al. (2003) ACM CSUR 35(2): the consumer registering interest in events, without knowledge of who produces them."),
        Subscription: ("en", "Subscription", "Carzaniga, Rosenblum & Wolf (2001) 'Design and Evaluation of a Wide-Area Event Notification Service', ACM TOCS 19(3) (SIENA): the registered interest - a predicate over notifications the service evaluates."),
        TopicBasedRouting: ("en", "Topic-based routing", "Eugster et al. (2003) ACM CSUR 35(2) sec 4.1: routing by channel name - subscribers receive every event published on the named topic."),
        ContentBasedRouting: ("en", "Content-based routing", "Carzaniga, Rosenblum & Wolf (2001) ACM TOCS 19(3): routing by predicates over message content rather than by channel name."),
        DeliveryGuarantee: ("en", "Delivery guarantee", "Spector (1982) 'Performing Remote Operations Efficiently on a Local Computer Network', CACM 25(4); Birrell & Nelson (1984) ACM TOCS 2(1); OASIS MQTT 5.0 sec 4.3 (QoS 0/1/2): the abstract delivery quality of service a transport promises - the parent of the three delivery semantics."),
        AtMostOnce: ("en", "At-most-once", "Spector (1982) CACM 25(4); OASIS MQTT 5.0 sec 4.3 (QoS 0): fire-and-forget - one transmission attempt, no retransmission; loss is possible, duplication is not."),
        AtLeastOnce: ("en", "At-least-once", "Birrell & Nelson (1984) ACM TOCS 2(1); OASIS MQTT 5.0 sec 4.3 (QoS 1): retransmit until acknowledged - no loss, but a delivered-yet-unacknowledged message may be re-sent, so duplication is possible."),
        ExactlyOnce: ("en", "Exactly-once", "OASIS MQTT 5.0 sec 4.3 (QoS 2); Birman & Joseph (1987) ACM TOCS 5(1): each message delivered exactly once at every operational destination (atomic all-or-nothing delivery; the virtual-synchrony reading is Birman & Joseph (1987) SOSP). End-to-end exactly-once is contested: over a lossy channel it is achievable only with transactional or deduplicating cooperation of the endpoints - at-least-once transport plus endpoint dedup."),
        Actor: ("en", "Actor", "Hewitt, Bishop & Steiger (1973) IJCAI: an autonomous entity communicating solely by messages - no shared state, no other interface."),
        Mailbox: ("en", "Mailbox", "Agha (1986) 'Actors: A Model of Concurrent Computation in Distributed Systems', MIT Press: an actor's mail queue at its mail address - arriving messages wait here until the actor processes them."),
        VirtualSynchrony: ("en", "Virtual synchrony", "Birman & Joseph (1987) SOSP, 'Exploiting Virtual Synchrony in Distributed Systems' (ISIS): ordered, gap-free multicast within a view - all operational members observe the same events in the same order; the underlying broadcast primitives are Birman & Joseph (1987) ACM TOCS 5(1)."),
        Decoupling: ("en", "Decoupling", "Eugster et al. (2003) ACM CSUR 35(2) sec 2: the space, time, and synchronization decoupling of publisher and subscriber that defines publish/subscribe."),
    },

    is_a: [
        // Spector (1982) / Birrell & Nelson (1984) / OASIS MQTT 5.0
        // sec 4.3: the three delivery semantics specialise the abstract
        // guarantee.
        (AtMostOnce, DeliveryGuarantee),
        (AtLeastOnce, DeliveryGuarantee),
        (ExactlyOnce, DeliveryGuarantee),
        // Eugster et al. (2003) / Carzaniga et al. (2001): the two
        // routing disciplines, and the broker, are kinds of message bus.
        (TopicBasedRouting, MessageBus),
        (ContentBasedRouting, MessageBus),
        (Broker, MessageBus),
    ],

    has_a: [
        // Agha (1986): the mailbox (mail queue) is part of its actor.
        (Actor, Mailbox),
    ],

    opposes: [
        // Eugster sec 4.1 vs Carzaniga: addressing by channel name and
        // addressing by content predicate are the opposed disciplines.
        (TopicBasedRouting, ContentBasedRouting),
    ],

    edges: [
        // Eugster et al. (2003): producers emit events on named topics.
        (Publisher, Event, Publishes),
        (Publisher, Topic, Publishes),
        // Eugster et al. (2003): consumers register interest.
        (Subscriber, Topic, Subscribes),
        (Subscriber, Subscription, Subscribes),
        // Carzaniga et al. (2001): the broker routes messages.
        (Broker, Message, Routes),
        // Birman & Joseph (1987): the broker delivers to subscribers.
        (Broker, Subscriber, Delivers),
        // Carzaniga et al. (2001): a subscription is a predicate that
        // matches events.
        (Subscription, Event, Matches),
        // Eugster et al. (2003) sec 2: the bus decouples both ends —
        // there is deliberately NO Publisher -> Subscriber edge.
        (MessageBus, Publisher, Decouples),
        (MessageBus, Subscriber, Decouples),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// The two routing disciplines — Eugster et al. (2003) §4.1 (by channel
/// name) vs. Carzaniga et al. (2001) (by content predicate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Routing {
    /// Route by the named channel the event was published on.
    TopicBased,
    /// Route by predicates evaluated over the message content.
    ContentBased,
}

impl Routing {
    /// The closed two-element set of routing disciplines.
    pub const ALL: [Routing; 2] = [Routing::TopicBased, Routing::ContentBased];
}

/// Which routing discipline a concept realises — `Some` for exactly the
/// two routing concepts (Eugster et al. 2003 §4.1; Carzaniga et al.
/// 2001), `None` for every other concept, including the abstract
/// `MessageBus` parent.
#[derive(Debug, Clone)]
pub struct RoutingStrategy;

impl Quality for RoutingStrategy {
    type Individual = BusConcept;
    type Value = Routing;

    fn get(&self, c: &BusConcept) -> Option<Routing> {
        match c {
            BusConcept::TopicBasedRouting => Some(Routing::TopicBased),
            BusConcept::ContentBasedRouting => Some(Routing::ContentBased),
            _ => None,
        }
    }
}

/// Which delivery semantics a concept denotes — `Some` for exactly the
/// three guarantees (Spector 1982; Birrell & Nelson 1984; OASIS MQTT
/// 5.0 §4.3), `None` for every other concept, including the abstract
/// `DeliveryGuarantee` parent.
#[derive(Debug, Clone)]
pub struct DeliverySemantics;

impl Quality for DeliverySemantics {
    type Individual = BusConcept;
    type Value = Delivery;

    fn get(&self, c: &BusConcept) -> Option<Delivery> {
        match c {
            BusConcept::AtMostOnce => Some(Delivery::AtMostOnce),
            BusConcept::AtLeastOnce => Some(Delivery::AtLeastOnce),
            BusConcept::ExactlyOnce => Some(Delivery::ExactlyOnce),
            _ => None,
        }
    }
}

/// Whether a concept's communication is space-decoupled — Eugster et
/// al. (2003) §2: pub/sub parties do not know each other's identity
/// (`true` for the bus, broker, topic, publisher, and subscriber). The
/// actor model is point-to-point: a sender addresses a *specific* mail
/// address / mailbox (Hewitt et al. 1973; Agha 1986), so `Actor` and
/// `Mailbox` are `false`. `None` for concepts that are not
/// communication parties or media.
#[derive(Debug, Clone)]
pub struct IsSpaceDecoupled;

impl Quality for IsSpaceDecoupled {
    type Individual = BusConcept;
    type Value = bool;

    fn get(&self, c: &BusConcept) -> Option<bool> {
        use BusConcept as C;
        match c {
            C::Publisher | C::Subscriber | C::MessageBus | C::Broker | C::Topic => Some(true),
            C::Actor | C::Mailbox => Some(false),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn direct_children_of(parent: BusConcept) -> Vec<BusConcept> {
    use pr4xis::category::{Arrow, Category};
    BusCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == BusRelationKind::Subsumption && m.target() == parent)
        .map(|m| m.source())
        .collect()
}

fn kinded_edge_exists(from: BusConcept, to: BusConcept, kind: BusRelationKind) -> bool {
    use pr4xis::category::{Arrow, Category};
    BusCategory::morphisms()
        .iter()
        .any(|m| m.source() == from && m.target() == to && m.kind() == kind)
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Spector (1982) / Birrell & Nelson (1984) / OASIS MQTT 5.0 §4.3: the
/// Subsumption-children of `DeliveryGuarantee` are exactly the set
/// {`AtMostOnce`, `AtLeastOnce`, `ExactlyOnce`} — verified as a
/// *bijection* with the closed [`Delivery`] value set via the
/// [`DeliverySemantics`] quality (set equality, not a count).
pub struct ThreeDeliveryGuarantees;

impl Axiom for ThreeDeliveryGuarantees {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let children = direct_children_of(BusConcept::DeliveryGuarantee);
        let q = DeliverySemantics;
        // Every child carries a delivery semantics.
        let semantics: Option<Vec<Delivery>> = children.iter().map(|c| q.get(c)).collect();
        let Some(semantics) = semantics else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        // Injective: no two children denote the same semantics.
        let injective = semantics
            .iter()
            .enumerate()
            .all(|(i, a)| semantics.iter().skip(i + 1).all(|b| a != b));
        // Surjective onto the closed set: every semantics is denoted.
        let surjective = Delivery::ALL.iter().all(|d| semantics.contains(d));
        // Set equality: the sizes match the closed set.
        let sized = children.len() == Delivery::ALL.len() && semantics.len() == Delivery::ALL.len();
        if injective && surjective && sized {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ThreeDeliveryGuarantees",
        "the Subsumption-children of DeliveryGuarantee are exactly the set {AtMostOnce, AtLeastOnce, ExactlyOnce} - a bijection with the closed Delivery value set via DeliverySemantics",
        "Spector (1982) CACM 25(4); Birrell & Nelson (1984) ACM TOCS 2(1); OASIS MQTT 5.0 sec 4.3; Eugster et al. (2003) ACM CSUR 35(2)"
    );
}
pr4xis::register_axiom!(
    ThreeDeliveryGuarantees,
    "Spector (1982) CACM 25(4); Birrell & Nelson (1984) ACM TOCS 2(1); OASIS MQTT 5.0 sec 4.3; Eugster et al. (2003) ACM CSUR 35(2)"
);

/// Eugster et al. (2003) §2: the bus decouples publisher from
/// subscriber. Structurally: there is *no* direct edge of any kind
/// between `Publisher` and `Subscriber` in either direction; both
/// `Decouples` edges (`MessageBus → Publisher`, `MessageBus →
/// Subscriber`) exist; and both parties are space-decoupled under
/// [`IsSpaceDecoupled`].
pub struct PubSubDecoupling;

impl Axiom for PubSubDecoupling {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let no_direct_edge = !BusCategory::morphisms().iter().any(|m| {
            m.kind() != BusRelationKind::Identity
                && ((m.source() == BusConcept::Publisher && m.target() == BusConcept::Subscriber)
                    || (m.source() == BusConcept::Subscriber
                        && m.target() == BusConcept::Publisher))
        });
        let decouples_publisher = kinded_edge_exists(
            BusConcept::MessageBus,
            BusConcept::Publisher,
            BusRelationKind::Decouples,
        );
        let decouples_subscriber = kinded_edge_exists(
            BusConcept::MessageBus,
            BusConcept::Subscriber,
            BusRelationKind::Decouples,
        );
        let q = IsSpaceDecoupled;
        let both_space_decoupled = q.get(&BusConcept::Publisher) == Some(true)
            && q.get(&BusConcept::Subscriber) == Some(true);
        if no_direct_edge && decouples_publisher && decouples_subscriber && both_space_decoupled {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PubSubDecoupling",
        "no direct edge of any kind connects Publisher and Subscriber; both Decouples edges exist; both parties are space-decoupled",
        "Eugster, Felber, Guerraoui & Kermarrec (2003) ACM CSUR 35(2) sec 2"
    );
}
pr4xis::register_axiom!(
    PubSubDecoupling,
    "Eugster, Felber, Guerraoui & Kermarrec (2003) ACM CSUR 35(2) sec 2"
);

/// Carzaniga, Rosenblum & Wolf (2001): topic-based and content-based
/// routing are the two opposed specialisations of the message bus —
/// both are Subsumption-children of `MessageBus`, they are connected by
/// `Opposition` (in both directions, opposition being symmetric), and
/// the `Matches` edge carrying the content predicate
/// (`Subscription → Event`) exists.
pub struct RoutingDichotomy;

impl Axiom for RoutingDichotomy {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let children = direct_children_of(BusConcept::MessageBus);
        let both_children = children.contains(&BusConcept::TopicBasedRouting)
            && children.contains(&BusConcept::ContentBasedRouting);
        let opposed = kinded_edge_exists(
            BusConcept::TopicBasedRouting,
            BusConcept::ContentBasedRouting,
            BusRelationKind::Opposition,
        ) && kinded_edge_exists(
            BusConcept::ContentBasedRouting,
            BusConcept::TopicBasedRouting,
            BusRelationKind::Opposition,
        );
        let matches_edge = kinded_edge_exists(
            BusConcept::Subscription,
            BusConcept::Event,
            BusRelationKind::Matches,
        );
        if both_children && opposed && matches_edge {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RoutingDichotomy",
        "TopicBasedRouting and ContentBasedRouting are both Subsumption-children of MessageBus, connected by Opposition, and the Matches edge (the content predicate) exists",
        "Carzaniga, Rosenblum & Wolf (2001) ACM TOCS 19(3)"
    );
}
pr4xis::register_axiom!(
    RoutingDichotomy,
    "Carzaniga, Rosenblum & Wolf (2001) ACM TOCS 19(3)"
);

/// Hewitt, Bishop & Steiger (1973): an actor communicates *solely* by
/// messages — structurally, the only non-Identity edge incident to
/// `Actor` is the Parthood edge with its `Mailbox` (`has_a` sugar emits
/// part → whole, so the edge runs `Mailbox → Actor`). No other kinded
/// edge touches `Actor`.
pub struct ActorMessagesOnly;

impl Axiom for ActorMessagesOnly {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::category::{Arrow, Category};
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let incident: Vec<BusRelation> = BusCategory::morphisms()
            .into_iter()
            .filter(|m| {
                m.kind() != BusRelationKind::Identity
                    && (m.source() == BusConcept::Actor || m.target() == BusConcept::Actor)
            })
            .collect();
        // Non-vacuous: the mailbox Parthood edge is present …
        let non_empty = !incident.is_empty();
        // … and it is the ONLY kinded edge touching Actor.
        let only_mailbox_parthood = incident.iter().all(|m| {
            m.kind() == BusRelationKind::Parthood
                && m.source() == BusConcept::Mailbox
                && m.target() == BusConcept::Actor
        });
        if non_empty && only_mailbox_parthood {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ActorMessagesOnly",
        "the only non-Identity edge incident to Actor is the Parthood edge with Mailbox - no other kinded edge touches Actor",
        "Hewitt, Bishop & Steiger (1973) IJCAI"
    );
}
pr4xis::register_axiom!(ActorMessagesOnly, "Hewitt, Bishop & Steiger (1973) IJCAI");

/// Spector (1982) / Birrell & Nelson (1984) / OASIS MQTT 5.0 §4.3: the
/// behavioural separation of the three delivery semantics, on the
/// *same* fixture message sequence via the engine's broker simulator:
///
/// - **at-most-once** — the dropped message is never redelivered
///   (possible loss, no duplicates), and the transport refuses to
///   retransmit at all;
/// - **at-least-once** — retransmission recovers the loss, but the
///   delivered-yet-unacknowledged message is re-sent and duplicates
///   (no loss, possible duplicates);
/// - **exactly-once** — with endpoint dedup, every published message is
///   handed to the application exactly once.
pub struct DeliverySemanticsBehavioral;

impl Axiom for DeliverySemanticsBehavioral {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let messages = scenario_messages(FIXTURE_MESSAGE_COUNT);

        // At-most-once: possible loss, no duplicates, no retransmission.
        let Ok(amo) = run_fixture(Delivery::AtMostOnce) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let amo_lost = delivered_count(&amo, FIXTURE_SUBSCRIBER, DROPPED_MESSAGE) == 0;
        let amo_no_duplicates = messages
            .iter()
            .all(|m| delivered_count(&amo, FIXTURE_SUBSCRIBER, *m) <= EXACTLY_ONE_DELIVERY);
        let amo_refuses_retry = apply(
            &amo,
            &BusAction::Retry {
                message: DROPPED_MESSAGE,
                subscriber: FIXTURE_SUBSCRIBER,
            },
            Delivery::AtMostOnce,
        )
        .is_err();

        // At-least-once: no loss, but the unacknowledged message duplicates.
        let Ok(alo) = run_fixture(Delivery::AtLeastOnce) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let alo_no_loss = messages
            .iter()
            .all(|m| delivered_count(&alo, FIXTURE_SUBSCRIBER, *m) >= EXACTLY_ONE_DELIVERY);
        let alo_duplicates =
            delivered_count(&alo, FIXTURE_SUBSCRIBER, UNACKED_MESSAGE) > EXACTLY_ONE_DELIVERY;

        // Exactly-once: endpoint dedup yields exactly one hand-off each.
        let Ok(eo) = run_fixture(Delivery::ExactlyOnce) else {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        };
        let eo_exact = messages
            .iter()
            .all(|m| delivered_count(&eo, FIXTURE_SUBSCRIBER, *m) == EXACTLY_ONE_DELIVERY);

        if amo_lost
            && amo_no_duplicates
            && amo_refuses_retry
            && alo_no_loss
            && alo_duplicates
            && eo_exact
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DeliverySemanticsBehavioral",
        "on the same fixture message sequence: at-most-once never redelivers the dropped message (loss, no duplicates), at-least-once recovers the loss but duplicates the unacknowledged message, exactly-once with endpoint dedup hands each message over exactly once",
        "Spector (1982) CACM 25(4); Birrell & Nelson (1984) ACM TOCS 2(1); OASIS MQTT 5.0 sec 4.3; Birman & Joseph (1987) ACM TOCS 5(1) for atomic exactly-once"
    );
}
pr4xis::register_axiom!(
    DeliverySemanticsBehavioral,
    "Spector (1982) CACM 25(4); Birrell & Nelson (1984) ACM TOCS 2(1); OASIS MQTT 5.0 sec 4.3; Birman & Joseph (1987) ACM TOCS 5(1) for atomic exactly-once"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for BusOntology {
    type Cat = BusCategory;
    type Qual = DeliverySemantics;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(ThreeDeliveryGuarantees));
        axioms.push(Box::new(PubSubDecoupling));
        axioms.push(Box::new(RoutingDichotomy));
        axioms.push(Box::new(ActorMessagesOnly));
        axioms.push(Box::new(DeliverySemanticsBehavioral));
        axioms
    }
}

/// The three delivery guarantees — direct Subsumption-children of
/// `DeliveryGuarantee` (Spector 1982; Birrell & Nelson 1984; OASIS MQTT
/// 5.0 §4.3). Grounded in the category's edges, used by tests.
pub fn delivery_guarantees() -> Vec<BusConcept> {
    direct_children_of(BusConcept::DeliveryGuarantee)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<BusCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        BusOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn three_delivery_guarantees_holds() {
        assert!(ThreeDeliveryGuarantees.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn pub_sub_decoupling_holds() {
        assert!(PubSubDecoupling.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn routing_dichotomy_holds() {
        assert!(RoutingDichotomy.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn actor_messages_only_holds() {
        assert!(ActorMessagesOnly.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn delivery_semantics_behavioral_holds() {
        assert!(DeliverySemanticsBehavioral.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn delivery_guarantee_taxonomy() {
        let guarantees = delivery_guarantees();
        let expected = [
            BusConcept::AtMostOnce,
            BusConcept::AtLeastOnce,
            BusConcept::ExactlyOnce,
        ];
        assert_eq!(guarantees.len(), expected.len());
        for c in expected {
            assert!(guarantees.contains(&c), "{c:?} should be a guarantee");
            assert!(
                DeliverySemantics.get(&c).is_some(),
                "{c:?} must denote a delivery semantics"
            );
        }
        assert_eq!(
            DeliverySemantics.get(&BusConcept::DeliveryGuarantee),
            None,
            "the abstract parent denotes no semantics of its own"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn routing_strategy_classification() {
        assert_eq!(
            RoutingStrategy.get(&BusConcept::TopicBasedRouting),
            Some(Routing::TopicBased),
            "topic-based routing routes by channel name (Eugster et al. 2003 sec 4.1)"
        );
        assert_eq!(
            RoutingStrategy.get(&BusConcept::ContentBasedRouting),
            Some(Routing::ContentBased),
            "content-based routing routes by predicate (Carzaniga et al. 2001)"
        );
        assert_eq!(
            RoutingStrategy.get(&BusConcept::MessageBus),
            None,
            "the abstract parent carries no routing discipline of its own"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn space_decoupling_classification() {
        for c in [
            BusConcept::Publisher,
            BusConcept::Subscriber,
            BusConcept::MessageBus,
            BusConcept::Broker,
            BusConcept::Topic,
        ] {
            assert_eq!(
                IsSpaceDecoupled.get(&c),
                Some(true),
                "{c:?} is space-decoupled (Eugster et al. 2003 sec 2)"
            );
        }
        for c in [BusConcept::Actor, BusConcept::Mailbox] {
            assert_eq!(
                IsSpaceDecoupled.get(&c),
                Some(false),
                "{c:?} addresses a specific mailbox (Hewitt et al. 1973)"
            );
        }
        assert_eq!(IsSpaceDecoupled.get(&BusConcept::Event), None);
    }
}
