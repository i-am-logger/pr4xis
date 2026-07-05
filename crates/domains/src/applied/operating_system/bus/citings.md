# Bus ontology — bibliography

## Primary sources

- **Eugster, P. T., Felber, P. A., Guerraoui, R. & Kermarrec, A.-M. (2003).** *"The Many Faces of Publish/Subscribe"*. ACM Computing Surveys 35(2), 114–131. DOI: [10.1145/857076.857078](https://doi.org/10.1145/857076.857078). Grounds `MessageBus` (§1), `Broker` (§2), `Event`, `Topic` (§4.1), `Publisher`, `Subscriber`, `TopicBasedRouting` (§4.1), `Decoupling` (§2 — space, time, and synchronization decoupling), the `Publishes`/`Subscribes`/`Decouples` edges, the `RoutingStrategy` and `IsSpaceDecoupled` qualities, and the `PubSubDecoupling` axiom (the deliberate absence of any `Publisher`→`Subscriber` edge).
- **Carzaniga, A., Rosenblum, D. S. & Wolf, A. L. (2001).** *"Design and Evaluation of a Wide-Area Event Notification Service"*. ACM Transactions on Computer Systems 19(3), 332–383 (SIENA). DOI: [10.1145/380749.380767](https://doi.org/10.1145/380749.380767). Grounds `Subscription` (the registered interest as a predicate over notifications), `ContentBasedRouting`, the `Routes` and `Matches` edges, and the `RoutingDichotomy` axiom.
- **Hewitt, C., Bishop, P. & Steiger, R. (1973).** *"A Universal Modular ACTOR Formalism for Artificial Intelligence"*. IJCAI '73, 235–245. Grounds `Message` (the self-contained payload as the sole unit of communication), `Actor`, `Mailbox`, the `Actor` has-a `Mailbox` Parthood edge, the point-to-point (`IsSpaceDecoupled = false`) classification of the actor pair, and the `ActorMessagesOnly` axiom.
- **Birman, K. P. & Joseph, T. A. (1987).** *"Reliable Communication in the Presence of Failures"*. ACM Transactions on Computer Systems 5(1), 47–76 (ISIS). DOI: [10.1145/7351.7478](https://doi.org/10.1145/7351.7478). Grounds `DeliveryGuarantee`, `AtMostOnce`, `AtLeastOnce`, `ExactlyOnce`, `VirtualSynchrony`, the `Delivers` edge, the `DeliverySemantics` quality, the `ThreeDeliveryGuarantees` axiom, the `DeliverySemanticsBehavioral` axiom, and the engine's failure model (§2: the channel may lose a message or its acknowledgment; retransmission recovers loss and is the source of duplication; the fixture constants `FIXTURE_MESSAGE_COUNT`, `DROPPED_MESSAGE`, `UNACKED_MESSAGE`, `EXACTLY_ONE_DELIVERY`).

## Functor source

- **von Bertalanffy, L. (1968).** *General System Theory: Foundations, Development, Applications*. George Braziller, New York. Grounds the `Bus → System` functor: components, interactions, boundaries, constraints, homeostasis.

## Honest-tier notes

- **`ExactlyOnce` is glossed as contested.** End-to-end exactly-once delivery over a lossy channel is not achievable by the transport alone: the sender cannot distinguish a lost message from a lost acknowledgment, so any policy that recovers loss can duplicate. What *is* achievable — and what the gloss, the engine, and the `DeliverySemanticsBehavioral` axiom demonstrate — is at-least-once transport plus transactional/deduplicating cooperation of the endpoints. The axiom is grounded in Birman & Joseph's (1987) virtual-synchrony delivery ("delivered exactly once at every operational destination"), which is a *system-level* guarantee built from exactly such cooperation, not a bare-channel property.
- **The engine is a simulator, not a protocol implementation.** The broker fixture demonstrates the three semantics' observable contracts (loss / duplication / exact hand-off) on a canonical failure trace; it does not implement ISIS's ABCAST/CBCAST protocols.

## Related workspace ontologies

- `formal::systems` — the `Bus → System` functor target (`SystemCategory`).
- `formal::systems::concurrency` — processes and channels (Hoare 1978); the bus is the applied medium those processes communicate over.
- `applied::operating_system` — the sibling family members; the microkernel-to-bus functor (kernel IPC as a bus) is the family-level integration, authored with the family, not here.
