# Bus — the event/message bus

A bus is a medium decoupling senders from receivers (Eugster, Felber, Guerraoui & Kermarrec 2003, ACM CSUR 35(2)). One ontology covers both faces of the same structure: the kernel IPC bus and the publish/subscribe event bus — publishers, subscribers, topics, and brokers (Eugster et al. 2003); subscriptions as content predicates (Carzaniga, Rosenblum & Wolf 2001, SIENA); actors, messages, and mailboxes (Hewitt, Bishop & Steiger 1973); delivery guarantees and virtual synchrony (Birman & Joseph 1987, ISIS).

## Verification

```
cargo test -p pr4xis-domains operating_system::bus
```

Category laws, ontology validation, five domain axioms (the behavioural one discharged against the broker simulator), quality classification tests, proptest sweeps over concepts and scenario families, and the `Bus → System` functor law check.

## Concepts (18)

| Family | Concepts |
|---|---|
| Medium + intermediary (Eugster et al. 2003) | `MessageBus`, `Broker`, `Decoupling` |
| What travels | `Event` (Eugster), `Message` (Hewitt et al. 1973) |
| Pub/sub roles + addressing | `Topic`, `Publisher`, `Subscriber`, `Subscription` (Carzaniga et al. 2001), `TopicBasedRouting`, `ContentBasedRouting` |
| Delivery QoS (Birman & Joseph 1987) | `DeliveryGuarantee`, `AtMostOnce`, `AtLeastOnce`, `ExactlyOnce`, `VirtualSynchrony` |
| Actor face (Hewitt et al. 1973) | `Actor`, `Mailbox` |

Taxonomy: `AtMostOnce`/`AtLeastOnce`/`ExactlyOnce` is-a `DeliveryGuarantee`; `TopicBasedRouting`/`ContentBasedRouting`/`Broker` is-a `MessageBus`. `Actor` has-a `Mailbox` (Parthood). `TopicBasedRouting` opposes `ContentBasedRouting`.

Custom edge kinds: `Publishes` (`Publisher`→`Event`, `Publisher`→`Topic`), `Subscribes` (`Subscriber`→`Topic`, `Subscriber`→`Subscription`), `Routes` (`Broker`→`Message`), `Delivers` (`Broker`→`Subscriber`), `Matches` (`Subscription`→`Event`), `Decouples` (`MessageBus`→`Publisher`, `MessageBus`→`Subscriber`). There is deliberately no `Publisher`→`Subscriber` edge — that absence is the `PubSubDecoupling` axiom.

`ExactlyOnce` is glossed honestly: end-to-end exactly-once is contested — over a lossy channel it is achievable only with transactional or deduplicating cooperation of the endpoints (at-least-once transport plus endpoint dedup), grounded in Birman & Joseph's virtual-synchrony delivery.

## Qualities

- `RoutingStrategy` → `Routing { TopicBased, ContentBased }` — `Some` for exactly the two routing disciplines, `None` elsewhere (Eugster §4.1; Carzaniga et al. 2001).
- `DeliverySemantics` → `Delivery { AtMostOnce, AtLeastOnce, ExactlyOnce }` — `Some` for exactly the three guarantees, `None` elsewhere (Birman & Joseph 1987).
- `IsSpaceDecoupled` → `bool` — `Publisher`/`Subscriber`/`MessageBus`/`Broker`/`Topic` `true` (Eugster §2); `Actor`/`Mailbox` `false` (an actor addresses a specific mailbox — Hewitt et al. 1973); `None` elsewhere.

## Domain axioms

| Axiom | Source | Discharged against |
|---|---|---|
| `ThreeDeliveryGuarantees` | Birman & Joseph (1987); Eugster et al. (2003) | Subsumption-children of `DeliveryGuarantee` in bijection with the closed `Delivery` set via `DeliverySemantics` (set equality, not count) |
| `PubSubDecoupling` | Eugster et al. (2003) §2 | no direct edge of any kind between `Publisher` and `Subscriber`; both `Decouples` edges present; both parties space-decoupled |
| `RoutingDichotomy` | Carzaniga et al. (2001) | both routings are Subsumption-children of `MessageBus`, connected by `Opposition`; the `Matches` edge exists |
| `ActorMessagesOnly` | Hewitt et al. (1973) | the only non-Identity edge incident to `Actor` is the `Mailbox` Parthood edge |
| `DeliverySemanticsBehavioral` | Birman & Joseph (1987) | the engine fixture: same message sequence under all three semantics — loss (at-most-once), duplication (at-least-once), exactly one hand-off (exactly-once with endpoint dedup) |

## Engine

[`engine.rs`](engine.rs) — a broker simulator: a topic routing table, per-subscriber inboxes, and per-subscriber dedup sets (`BrokerSituation`), driven by `Publish`/`Subscribe`/`Route`/`Deliver`/`Drop`/`Retry` actions parameterised by the `Delivery` semantics. The failure model is Birman & Joseph (1987) §2: the channel may lose a message *or its acknowledgment*, and the sender cannot tell the two apart — so the retransmission that recovers loss (at-least-once) is exactly what duplicates, and only endpoint dedup restores exactly-once. The canonical fixture publishes two messages: `DROPPED_MESSAGE` (first attempt lost) and `UNACKED_MESSAGE` (delivered, acknowledgment lost); `run_scenario` generalises it to arbitrary message counts for the proptest sweep. At-most-once refuses `Retry` outright — fire-and-forget never retransmits.

## Cross-functors

- [`system_functor.rs`](system_functor.rs) — `Bus → System` (von Bertalanffy 1968): the communicating parties → `Component`, what travels → `Interaction`, the broker and routing disciplines → `Controller`, subscriptions and delivery guarantees → `Constraint`, the bus and decoupling → `Boundary`, the mailbox → `State`, virtual synchrony → `Homeostasis`.

Related functors not in this directory: the microkernel-to-bus functor (unifying the kernel IPC bus with the pub/sub event bus) belongs to the `operating_system` family integration, and a bus-to-speech_act functor (a weak arrow) is deferred to an issue.

## Files

- `ontology.rs` — `BusOntology`, three qualities, five domain axioms
- `engine.rs` — the broker simulator and the loss/retransmission fixture
- `system_functor.rs` — `BusToSystem` + functor laws
- `tests.rs` — proptest sweeps
- `mod.rs`, `README.md`, `citings.md`
