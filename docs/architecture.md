# Architecture

## Three-Layer Design

Praxis separates concerns into three layers. Each layer depends only on the layer below it.

### Layer 1: rust-category (Mathematics)

Category theory primitives. Pure — no IO, no side effects.

- `Entity` — finite, enumerable objects
- `Relationship` — directed connections between entities
- `Category` — entities + relationships with composition and identity laws
- `Morphism` — functional wrapper for chainable composition
- `Functor` — structure-preserving map between categories
- `NaturalTransformation` — morphism between functors

Validation functions verify category laws (identity, associativity, closure) exhaustively and via property-based testing.

### Layer 2: rust-ontology (Structural Rules)

Defines what things ARE — the static rules of a domain.

- `Ontology` — ties together a category, qualities, and axioms
- `Quality` — properties that inhere in entities (BFO/DOLCE term)
- `Axiom` — domain-specific invariants beyond category laws
- `Proposition` — evaluable rule with context-carrying results
- `AllOf` / `AnyOf` / `Not` / `Implies` — logical composition
- `Measurable` / `Compare` / `Threshold` — comparison propositions

### Layer 3: rust-praxis (Runtime Enforcement)

Defines how things CHANGE — the dynamic enforcement engine.

- `Situation` — immutable world state snapshot
- `Action` — a proposed state transition
- `Precondition` — rule checked before action application
- `Engine` — validates, applies, traces, supports undo/redo
- `Trace` — full action history with human-readable dump

## Dependency Flow

```
rust-category
    ↓
rust-ontology (depends on rust-category)
    ↓
rust-praxis (depends on rust-ontology, rust-category)
    ↓
praxis-* domain crates (depend on all three)
```

Domain crates never depend on each other. Each is a standalone enforcement engine for its domain.

## Engine Lifecycle

```
1. Create Engine with initial Situation + Preconditions + apply function
2. Call engine.next(action)
   a. All preconditions checked against current situation + action
   b. If any violated → Err with violation details, trace records failure
   c. If all satisfied → apply function produces new situation
   d. Previous situation pushed to history stack
   e. Trace records success with all precondition results
3. Call engine.back() to undo (moves current to redo stack)
4. Call engine.forward() to redo
5. New next() after back() clears the redo stack (branch point)
```

## Design Decisions

**Situations are immutable.** Every action produces a new situation. The old one is preserved in the history stack. This enables undo/redo without mutation.

**Preconditions are separate from apply.** The precondition layer validates rules. The apply function transforms state. They are checked independently. If a precondition passes but apply would fail, that's a bug — the apply function should `expect()` success.

**Rich enums carry context.** Every enum variant carries the data of HOW it got there. `MotionStatus::Granted { ruling_date, judge, order }` not `MotionStatus::Granted`. No information is lost between state transitions.

**Logical composition uses trait objects.** `AllOf` and `AnyOf` take `Vec<Box<dyn Proposition>>` so you can mix different proposition types in one logical expression.

**Property-based testing is the primary verification.** Domain invariants are expressed as proptest properties that hold for all generated inputs, not just hand-picked examples.
