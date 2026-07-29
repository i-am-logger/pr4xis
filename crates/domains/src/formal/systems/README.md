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

**Incoming (5):**

| Functor | Source | File |
|---|---|---|
| ConcurrencyToSystems | concurrency | `../information/concurrency/systems_functor.rs` |
| EventsToSystems | events | `../information/events/systems_functor.rs` |
| SchemaToSystems | schema | `../information/schema/systems_functor.rs` |
| ControlImpl (consumers) | control.rs | `control.rs` |
| ControlTheoryToControl | `formal::math::control_theory` (classical/PID, targets `control.rs`'s `ControlCategory`) | `../math/control_theory/systems_functor.rs` |

## Sibling ontology: Viable System Model

`viable_system_model.rs` is a second, standalone `pr4xis::ontology!` in this
directory (its own `ViableSystemModelCategory`, not a functor target of
`SystemConcept`) — Beer's five subsystems every self-regulating organization
needs: S1 Operations, S2 Coordination, S3 Control, S3\* Audit, S4
Intelligence, S5 Policy, plus Environment (what S4 scans). Kept separate from
`control.rs`'s `ControlConcept` rather than forced into a functor: Beer's
S1-S5 decomposition is an organizational-recursion abstraction, not a
structural specialization of Ashby's single-loop Plant/Controller/Sensor
vocabulary — no sound, non-collapsing mapping between the two exists (see the
module doc for what was tried and rejected).

| Entities (7) | S1Operations, S2Coordination, S3Control, S3StarAudit, S4Intelligence, S5Policy, Environment |
|---|---|
| Axioms | `VsmCompleteness` (Beer 1985) — a declared system is viable iff all five required subsystems are present |
| Qualities | `InsideAndNow` (bool) — S1-S3\* are the inside-and-now subsystems; S4-S5 are outside-and-then |

Scope, honestly bounded: Beer's recursion theorem (a viable system contains,
and is contained in, other viable systems) is not modeled — this module
covers one level of the hierarchy.

## Files

- `ontology.rs` -- `SystemConcept`, cybernetic category, IsCyberneticLoop quality, tests
- `control.rs` -- Control-theory layer built on the systems category
- `viable_system_model.rs` -- Beer's VSM (S1-S5) as a sibling ontology, tests
- `traffic_functor.rs` -- Systems → traffic-signal functor (Signal=Component, etc.)
- `engine_functor.rs` -- Systems → dialogue engine functor
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
