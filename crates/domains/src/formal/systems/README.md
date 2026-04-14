# Systems -- Systems thinking and cybernetics

Models the core concepts of systems thinking and cybernetics as a category over ten `SystemConcept` objects: Component, Interaction, State, Transition, Constraint, Feedback, Homeostasis, Emergence, Boundary, Controller. The cybernetic loop `State → Feedback → Controller → Constraint → Transition → Component → State` is closed under composition, making functors from chess, traffic, concurrency, events, and schema all land inside this single category.

Key references:
- von Bertalanffy 1968: *General System Theory*
- Wiener 1948: *Cybernetics*
- Ashby 1956: *An Introduction to Cybernetics*
- Beer 1972: *Brain of the Firm*
- Meadows 2008: *Thinking in Systems*

## Entities (10)

| Category | Entities |
|---|---|
| Structure (3) | Component, Interaction, Boundary |
| Dynamics (3) | State, Transition, Constraint |
| Cybernetic loop (3) | Feedback, Homeostasis, Controller |
| Emergent (1) | Emergence |

## Category

Kinded morphisms: `Component ComposesInto State`, `Interaction ComposesInto State`, `Transition Changes State`, `Constraint Governs Transition`, `State FeedsBack Feedback`, `Feedback FeedsBack Transition`, `Homeostasis Stabilizes State`, `Feedback Stabilizes Homeostasis`, `Interaction ArisesFrom Emergence`, `Controller Regulates Constraint`, `Boundary Separates Component`, `Feedback FeedsBack Controller` (Ashby). Composition closes the full cybernetic round-trip.

## Qualities

| Quality | Type | Description |
|---|---|---|
| IsCyberneticLoop | bool | State, Feedback, Controller, Constraint, Transition, Homeostasis = true; others = false |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the systems kinded relation graph | auto-generated |

## Functors

**Outgoing (2):**

| Functor | Target | File |
|---|---|---|
| SystemsToTraffic | traffic (signalized intersection) | `traffic_functor.rs` |
| SystemsToEngine | dialogue / ontology engine | `engine_functor.rs` |

**Incoming (4):**

| Functor | Source | File |
|---|---|---|
| ConcurrencyToSystems | concurrency | `../information/concurrency/systems_functor.rs` |
| EventsToSystems | events | `../information/events/systems_functor.rs` |
| SchemaToSystems | schema | `../information/schema/systems_functor.rs` |
| ControlImpl (consumers) | control.rs | `control.rs` |

## Files

- `ontology.rs` -- `SystemConcept`, cybernetic category, IsCyberneticLoop quality, tests
- `control.rs` -- Control-theory layer built on the systems category
- `traffic_functor.rs` -- Systems → traffic-signal functor (Signal=Component, etc.)
- `engine_functor.rs` -- Systems → dialogue engine functor
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
