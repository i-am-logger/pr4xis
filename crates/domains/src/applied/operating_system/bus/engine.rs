//! Bus engine — a broker simulator discharged against one canonical
//! loss/retransmission fixture.
//!
//! The simulator realises the pieces of a topic-based event bus
//! (Eugster, Felber, Guerraoui & Kermarrec 2003, ACM CSUR 35(2) §4.1):
//! a **routing table** mapping named topics to their registered
//! subscribers, a **per-subscriber inbox** of messages handed to the
//! application, and a **per-subscriber dedup set** — the endpoint
//! cooperation without which exactly-once delivery is not achievable.
//!
//! The failure model is Birman & Joseph (1987) ACM TOCS 5(1) §2: the
//! channel may lose a message (omission failure), and it may equally
//! lose an *acknowledgment* — the sender cannot distinguish the two, so
//! a retransmission policy that recovers loss can also duplicate. The
//! one fixture drives the same published message sequence through the
//! three delivery semantics and observes: possible loss (at-most-once),
//! possible duplication (at-least-once), and exactly one delivery
//! (at-least-once transport plus endpoint dedup).
//!
//! Every constant below is a documented structural fixture parameter
//! cited to the axiom's source — no free magic numbers.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::engine::{Action, Situation};

// ---------------------------------------------------------------------------
// Identifiers
// ---------------------------------------------------------------------------

/// A topic identity — Eugster et al. (2003) §4.1: a topic is a *named*
/// channel, so it is identified, never an anonymous index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TopicId(pub usize);

/// A subscriber identity — Eugster et al. (2003): the consumer whose
/// registered interest the broker stores in its routing table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubscriberId(pub usize);

/// A message identity — Hewitt, Bishop & Steiger (1973): the message is
/// the self-contained unit of communication; a *stable identity* per
/// message is exactly what endpoint deduplication requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageId(pub usize);

// ---------------------------------------------------------------------------
// Delivery semantics (Birman & Joseph 1987)
// ---------------------------------------------------------------------------

/// The three delivery guarantees — Birman & Joseph (1987) ACM TOCS
/// 5(1): what a transport promises about how many times a published
/// message reaches an operational destination. Parameterises every
/// engine action via [`apply`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// Fire-and-forget: one transmission attempt, no retransmission —
    /// loss is possible, duplication is not.
    AtMostOnce,
    /// Retransmit until acknowledged: no loss, but a delivered-yet-
    /// unacknowledged message may be re-sent — duplication is possible.
    AtLeastOnce,
    /// At-least-once transport plus endpoint deduplication: each message
    /// is handed to the application exactly once (Birman & Joseph 1987:
    /// delivered exactly once at every operational destination).
    ExactlyOnce,
}

impl Delivery {
    /// The closed three-element set of delivery guarantees — Birman &
    /// Joseph (1987); Eugster et al. (2003).
    pub const ALL: [Delivery; 3] = [
        Delivery::AtMostOnce,
        Delivery::AtLeastOnce,
        Delivery::ExactlyOnce,
    ];
}

// ---------------------------------------------------------------------------
// Situation — the broker's state
// ---------------------------------------------------------------------------

/// One routing-table entry: a named topic and its registered
/// subscribers — Eugster et al. (2003) §4.1 topic-based addressing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicRoute {
    /// The named channel.
    pub topic: TopicId,
    /// The subscribers registered on it.
    pub subscribers: Vec<SubscriberId>,
}

/// A message the broker has accepted from a publisher, tagged with the
/// topic it was published on — Eugster et al. (2003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedMessage {
    /// The message's stable identity.
    pub message: MessageId,
    /// The topic it was published on.
    pub topic: TopicId,
}

/// The transport status of one (message, subscriber) transmission —
/// the unit the retransmission protocol of Birman & Joseph (1987)
/// reasons about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransmissionStatus {
    /// A transmission attempt is owed to the channel.
    Pending,
    /// The channel lost the last attempt (omission failure — Birman &
    /// Joseph 1987 §2).
    Lost,
    /// An attempt reached the subscriber. The *sender* may still not
    /// know this: if the acknowledgment was lost, its timeout fires and
    /// it retransmits — the source of duplication under at-least-once.
    Delivered,
}

/// One outstanding transmission: the broker owes (or owed) `message` to
/// `subscriber` — Birman & Joseph (1987).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transmission {
    /// The message being transmitted.
    pub message: MessageId,
    /// The destination subscriber.
    pub subscriber: SubscriberId,
    /// Where the transmission currently stands.
    pub status: TransmissionStatus,
}

/// One subscriber's inbox: the messages handed to the application, in
/// arrival order — Hewitt et al. (1973): the mailbox is the actor's
/// message queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inbox {
    /// The inbox's owner.
    pub subscriber: SubscriberId,
    /// Messages delivered so far, in order (duplicates appear twice).
    pub delivered: Vec<MessageId>,
}

/// One subscriber's deduplication set: the message identities already
/// seen — the *endpoint cooperation* that turns at-least-once transport
/// into exactly-once delivery (Birman & Joseph 1987).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DedupSet {
    /// The set's owner.
    pub subscriber: SubscriberId,
    /// Message identities already handed to the application.
    pub seen: Vec<MessageId>,
}

/// The broker's whole state — the engine `Situation`: routing table,
/// accepted messages, outstanding transmissions, per-subscriber inboxes
/// and dedup sets (Eugster et al. 2003; Birman & Joseph 1987).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokerSituation {
    /// Topic-based routing table (Eugster et al. 2003 §4.1).
    pub routing_table: Vec<TopicRoute>,
    /// Messages accepted from publishers, with their topics.
    pub accepted: Vec<PublishedMessage>,
    /// Outstanding per-subscriber transmissions.
    pub transmissions: Vec<Transmission>,
    /// Per-subscriber inboxes (what the application saw).
    pub inboxes: Vec<Inbox>,
    /// Per-subscriber dedup sets (exactly-once endpoint cooperation).
    pub dedup: Vec<DedupSet>,
}

impl Situation for BrokerSituation {}

/// The empty broker: no routes, no messages, no subscribers.
pub fn initial_situation() -> BrokerSituation {
    BrokerSituation {
        routing_table: Vec::new(),
        accepted: Vec::new(),
        transmissions: Vec::new(),
        inboxes: Vec::new(),
        dedup: Vec::new(),
    }
}

/// How many times `message` was handed to `subscriber`'s application —
/// the observable the three delivery guarantees constrain (Birman &
/// Joseph 1987).
pub fn delivered_count(
    situation: &BrokerSituation,
    subscriber: SubscriberId,
    message: MessageId,
) -> usize {
    situation
        .inboxes
        .iter()
        .filter(|i| i.subscriber == subscriber)
        .flat_map(|i| i.delivered.iter())
        .filter(|m| **m == message)
        .count()
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/// One step of the bus protocol — the engine `Action`, parameterised by
/// the [`Delivery`] semantics via [`apply`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BusAction {
    /// Register interest: add `subscriber` to `topic`'s route — Eugster
    /// et al. (2003) §4.1.
    Subscribe {
        /// The consumer registering interest.
        subscriber: SubscriberId,
        /// The named channel subscribed to.
        topic: TopicId,
    },
    /// A producer emits `message` on `topic` — Eugster et al. (2003).
    Publish {
        /// The named channel published on.
        topic: TopicId,
        /// The emitted message's fresh identity.
        message: MessageId,
    },
    /// The broker routes an accepted message to every subscriber of its
    /// topic — Carzaniga, Rosenblum & Wolf (2001).
    Route {
        /// The accepted message to route.
        message: MessageId,
    },
    /// A transmission attempt reaches `subscriber` — Birman & Joseph
    /// (1987). Under exactly-once, the endpoint dedup set may suppress
    /// the hand-off to the application.
    Deliver {
        /// The message transmitted.
        message: MessageId,
        /// The destination subscriber.
        subscriber: SubscriberId,
    },
    /// The channel loses a transmission attempt (omission failure —
    /// Birman & Joseph 1987 §2).
    Drop {
        /// The message whose attempt is lost.
        message: MessageId,
        /// The destination whose attempt is lost.
        subscriber: SubscriberId,
    },
    /// The sender retransmits — Birman & Joseph (1987): allowed after a
    /// lost attempt *or* after a delivery whose acknowledgment was lost
    /// (the sender cannot tell the two apart). Refused outright under
    /// at-most-once, which never retransmits.
    Retry {
        /// The message to retransmit.
        message: MessageId,
        /// The destination to retransmit to.
        subscriber: SubscriberId,
    },
}

impl Action for BusAction {
    type Sit = BrokerSituation;
}

// ---------------------------------------------------------------------------
// Transition function
// ---------------------------------------------------------------------------

/// Apply one bus action under the given delivery semantics. `Err` when
/// the action is not enabled: duplicate registrations and identities
/// are rejected, transmissions must exist in the right status, and
/// at-most-once refuses to retransmit (Birman & Joseph 1987).
pub fn apply(
    situation: &BrokerSituation,
    action: &BusAction,
    semantics: Delivery,
) -> Result<BrokerSituation, String> {
    let mut next = situation.clone();
    match action {
        BusAction::Subscribe { subscriber, topic } => {
            let route = next.routing_table.iter_mut().find(|r| r.topic == *topic);
            match route {
                Some(r) if r.subscribers.contains(subscriber) => {
                    return Err(format!(
                        "subscriber {subscriber:?} already registered on {topic:?}"
                    ));
                }
                Some(r) => r.subscribers.push(*subscriber),
                None => next.routing_table.push(TopicRoute {
                    topic: *topic,
                    subscribers: vec![*subscriber],
                }),
            }
            if !next.inboxes.iter().any(|i| i.subscriber == *subscriber) {
                next.inboxes.push(Inbox {
                    subscriber: *subscriber,
                    delivered: Vec::new(),
                });
            }
            if !next.dedup.iter().any(|d| d.subscriber == *subscriber) {
                next.dedup.push(DedupSet {
                    subscriber: *subscriber,
                    seen: Vec::new(),
                });
            }
        }
        BusAction::Publish { topic, message } => {
            if next.accepted.iter().any(|p| p.message == *message) {
                return Err(format!("message identity {message:?} already published"));
            }
            next.accepted.push(PublishedMessage {
                message: *message,
                topic: *topic,
            });
        }
        BusAction::Route { message } => {
            let Some(published) = next.accepted.iter().find(|p| p.message == *message) else {
                return Err(format!("message {message:?} was never published"));
            };
            if next.transmissions.iter().any(|t| t.message == *message) {
                return Err(format!("message {message:?} already routed"));
            }
            let destinations: Vec<SubscriberId> = next
                .routing_table
                .iter()
                .filter(|r| r.topic == published.topic)
                .flat_map(|r| r.subscribers.iter().copied())
                .collect();
            for subscriber in destinations {
                next.transmissions.push(Transmission {
                    message: *message,
                    subscriber,
                    status: TransmissionStatus::Pending,
                });
            }
        }
        BusAction::Deliver {
            message,
            subscriber,
        } => {
            let Some(t) = next.transmissions.iter_mut().find(|t| {
                t.message == *message
                    && t.subscriber == *subscriber
                    && t.status == TransmissionStatus::Pending
            }) else {
                return Err(format!(
                    "no pending transmission of {message:?} to {subscriber:?}"
                ));
            };
            t.status = TransmissionStatus::Delivered;
            // Exactly-once = at-least-once transport + endpoint dedup:
            // the dedup set suppresses the hand-off of an already-seen
            // identity (Birman & Joseph 1987).
            let suppressed = if semantics == Delivery::ExactlyOnce {
                let Some(d) = next.dedup.iter_mut().find(|d| d.subscriber == *subscriber) else {
                    return Err(format!("no dedup set for {subscriber:?}"));
                };
                if d.seen.contains(message) {
                    true
                } else {
                    d.seen.push(*message);
                    false
                }
            } else {
                false
            };
            if !suppressed {
                let Some(inbox) = next
                    .inboxes
                    .iter_mut()
                    .find(|i| i.subscriber == *subscriber)
                else {
                    return Err(format!("no inbox for {subscriber:?}"));
                };
                inbox.delivered.push(*message);
            }
        }
        BusAction::Drop {
            message,
            subscriber,
        } => {
            let Some(t) = next.transmissions.iter_mut().find(|t| {
                t.message == *message
                    && t.subscriber == *subscriber
                    && t.status == TransmissionStatus::Pending
            }) else {
                return Err(format!(
                    "no pending transmission of {message:?} to {subscriber:?} to lose"
                ));
            };
            t.status = TransmissionStatus::Lost;
        }
        BusAction::Retry {
            message,
            subscriber,
        } => {
            if semantics == Delivery::AtMostOnce {
                return Err(
                    "at-most-once transport is fire-and-forget: it never retransmits \
                     (Birman & Joseph 1987)"
                        .to_string(),
                );
            }
            let Some(t) = next.transmissions.iter_mut().find(|t| {
                t.message == *message
                    && t.subscriber == *subscriber
                    && t.status != TransmissionStatus::Pending
            }) else {
                return Err(format!(
                    "no lost or unacknowledged transmission of {message:?} to {subscriber:?}"
                ));
            };
            // Lost → the retransmission recovers the omission failure;
            // Delivered → the acknowledgment was lost, the sender's
            // timeout fires and it re-sends (Birman & Joseph 1987: the
            // sender cannot distinguish the two cases).
            t.status = TransmissionStatus::Pending;
        }
    }
    Ok(next)
}

// ---------------------------------------------------------------------------
// Fixture parameters (Birman & Joseph 1987; Eugster et al. 2003)
// ---------------------------------------------------------------------------

/// The fixture's single named channel — Eugster et al. (2003) §4.1:
/// topic-based addressing routes by channel name, so the minimal
/// fixture has exactly one named topic.
pub const FIXTURE_TOPIC: TopicId = TopicId(0);

/// The fixture's single registered consumer — one subscriber suffices
/// to observe loss vs. duplication vs. exact delivery on its inbox
/// (Birman & Joseph 1987: the guarantees are per operational
/// destination).
pub const FIXTURE_SUBSCRIBER: SubscriberId = SubscriberId(0);

/// Two published messages — the smallest sequence exhibiting both
/// channel failure modes of Birman & Joseph (1987) §2 at once: one
/// message lost on its first transmission, one delivered whose
/// acknowledgment is lost.
pub const FIXTURE_MESSAGE_COUNT: usize = 2;

/// The fixture message whose first transmission attempt the channel
/// loses (omission failure — Birman & Joseph 1987 §2). At-most-once
/// never recovers it; the retransmitting semantics do.
pub const DROPPED_MESSAGE: MessageId = MessageId(0);

/// The fixture message delivered on its first attempt whose
/// *acknowledgment* is lost — the sender cannot distinguish a lost
/// message from a lost acknowledgment, so retransmitting semantics
/// re-send it and at-least-once duplicates (Birman & Joseph 1987).
pub const UNACKED_MESSAGE: MessageId = MessageId(1);

/// Exactly one hand-off per destination — Birman & Joseph (1987):
/// atomic broadcast delivers each message exactly once at every
/// operational destination. The exactly-once contract compares against
/// this count.
pub const EXACTLY_ONE_DELIVERY: usize = 1;

/// The message identities of a scenario publishing `count` messages —
/// structurally derived, not hand-listed.
pub fn scenario_messages(count: usize) -> Vec<MessageId> {
    (0..count).map(MessageId).collect()
}

// ---------------------------------------------------------------------------
// Scenario runner
// ---------------------------------------------------------------------------

/// Run the canonical loss/retransmission scenario under the given
/// semantics: one subscriber on one topic; `count` messages published
/// and routed; `dropped`'s first attempt is lost by the channel and
/// `unacked`'s first attempt is delivered but unacknowledged; the
/// retransmitting semantics (at-least-once, exactly-once) then re-send
/// both, at-most-once sends nothing further (Birman & Joseph 1987).
///
/// The *published message sequence* is identical across the three
/// semantics — only the retransmission policy differs, which is exactly
/// the dimension the [`Delivery`] enum parameterises.
pub fn run_scenario(
    semantics: Delivery,
    count: usize,
    dropped: MessageId,
    unacked: MessageId,
) -> Result<BrokerSituation, String> {
    if dropped == unacked {
        return Err("the dropped and unacknowledged messages must be distinct".to_string());
    }
    if dropped.0 >= count || unacked.0 >= count {
        return Err("the failing messages must be within the published sequence".to_string());
    }
    let mut situation = initial_situation();
    let step = |s: &BrokerSituation, a: BusAction| apply(s, &a, semantics);

    // Registration, publication, routing — identical for all semantics.
    situation = step(
        &situation,
        BusAction::Subscribe {
            subscriber: FIXTURE_SUBSCRIBER,
            topic: FIXTURE_TOPIC,
        },
    )?;
    for message in scenario_messages(count) {
        situation = step(
            &situation,
            BusAction::Publish {
                topic: FIXTURE_TOPIC,
                message,
            },
        )?;
    }
    for message in scenario_messages(count) {
        situation = step(&situation, BusAction::Route { message })?;
    }

    // First transmission round — the channel's behaviour, identical for
    // all semantics: `dropped` is lost, everything else arrives (with
    // `unacked`'s acknowledgment lost on the way back).
    for message in scenario_messages(count) {
        let action = if message == dropped {
            BusAction::Drop {
                message,
                subscriber: FIXTURE_SUBSCRIBER,
            }
        } else {
            BusAction::Deliver {
                message,
                subscriber: FIXTURE_SUBSCRIBER,
            }
        };
        situation = step(&situation, action)?;
    }

    // Retransmission round — the policy the semantics parameterises:
    // at-most-once is fire-and-forget (no further action); the
    // retransmitting semantics re-send both the lost message and the
    // delivered-but-unacknowledged one (Birman & Joseph 1987).
    if semantics != Delivery::AtMostOnce {
        for message in [dropped, unacked] {
            situation = step(
                &situation,
                BusAction::Retry {
                    message,
                    subscriber: FIXTURE_SUBSCRIBER,
                },
            )?;
            situation = step(
                &situation,
                BusAction::Deliver {
                    message,
                    subscriber: FIXTURE_SUBSCRIBER,
                },
            )?;
        }
    }
    Ok(situation)
}

/// The canonical fixture: [`FIXTURE_MESSAGE_COUNT`] messages with
/// [`DROPPED_MESSAGE`] lost and [`UNACKED_MESSAGE`] unacknowledged —
/// the same message sequence the `DeliverySemanticsBehavioral` axiom
/// runs under all three semantics.
pub fn run_fixture(semantics: Delivery) -> Result<BrokerSituation, String> {
    run_scenario(
        semantics,
        FIXTURE_MESSAGE_COUNT,
        DROPPED_MESSAGE,
        UNACKED_MESSAGE,
    )
}
