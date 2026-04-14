# Events -- Event-driven systems, event sourcing, CQRS

Models event-driven architecture as a ten-concept category: Commands trigger Events, Events are appended to an immutable EventLog, Handlers react to Events, an EventBus routes them, Events change State, Projections are derived from the EventLog (CQRS), Subscriptions listen to the bus, and Sagas compose Events into longer-running processes. This is the formal substrate chess, concurrency, and systems-thinking functors land on.

Key references:
- Fowler 2005: *Event Sourcing*
- Young 2010: *CQRS Documents*
- Guizzardi et al. 2013: *UFO-B: Ontology of Events*
- Almeida & Falbo 2019: *Events as Entities in Ontology-Driven Modeling*

## Entities (10)

| Category | Entities |
|---|---|
| Core (3) | Event, Command, State |
| Pipeline (3) | Handler, EventLog, EventBus |
| CQRS / extensions (4) | Projection, Subscription, Saga, EventSchema |

## Category

Morphisms: `Command Triggers Event`, `Event AppendedTo EventLog`, `Handler ReactsTo Event`, `EventBus Routes Handler`, `Event Changes State`, `Projection DerivedFrom EventLog`, `Subscription ListensTo EventBus`, `Saga Composes Event`, `EventSchema Defines Event`. Composition closes Command → State, Command → EventLog, EventBus → Event, Subscription → Handler, Saga → State and Saga → EventLog.

## Qualities

| Quality | Type | Description |
|---|---|---|
| IsImmutable | bool | Event, EventLog, EventSchema = true; State, Projection = false (derived/mutable views) |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the event-driven kinded relation graph | auto-generated |

## Functors

**Outgoing (3):**

| Functor | Target | File |
|---|---|---|
| EventsToSystems | systems thinking | `systems_functor.rs` |
| EventsToConcurrency | concurrency | `concurrent_functor.rs` |
| EventsToChess | chess | `chess_functor.rs` |

**Incoming:** no incoming cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `EventConcept`, event-driven category, IsImmutable quality, tests
- `systems_functor.rs` -- Events → systems-thinking functor
- `concurrent_functor.rs` -- Events → concurrency functor
- `chess_functor.rs` -- Events → chess functor (moves as events)
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
