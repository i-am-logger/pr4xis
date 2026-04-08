# Architecture

## Four-Layer Design

Praxis separates concerns into four layers. Each layer depends only on the layers below it.

### Layer 1: praxis::logic (Foundation)

The logical foundation. Depends on nothing.

- `Axiom` — a statement that must hold unconditionally
- `Proposition` — evaluable statement with context
- `AllOf` / `AnyOf` / `Not` / `Implies` — logical composition
- `Measurable` / `Compare` / `Threshold` — comparison propositions
- `Deduction` — if A and A→B, then B (guaranteed truth)
- `Induction` — pattern across instances → general rule (probable truth)
- `Abduction` — observation → best explanation (hypothesis)
- `Connective` — AND, OR, NOT, IMPLIES, IFF, XOR, NAND, NOR
- Truth tables, tautology verification, De Morgan's laws, NAND universality

### Layer 2: praxis::category (Mathematics)

Category theory primitives. Depends on logic.

- `Entity` — finite, enumerable objects
- `Relationship` — directed connections between entities
- `Category` — entities + relationships with composition and identity laws
- `Morphism` — functional wrapper for chainable composition
- `Functor` — structure-preserving map between categories
- `NaturalTransformation` — morphism between functors
- `NoDeadStates` / `FullyConnected` — structural axioms on categories

Validation functions verify category laws (identity, associativity, closure) exhaustively and via property-based testing.

### Layer 3: praxis::ontology (Structural Rules)

Defines what things ARE and how they relate. Depends on category and logic.

- `Ontology` — ties together a category, qualities, and axioms
- `Quality` — properties that inhere in entities (BFO/DOLCE term)
- `reasoning::taxonomy` — is-a hierarchies (subsumption)
- `reasoning::mereology` — part-whole relationships (has-a)
- `reasoning::causation` — cause-effect relationships
- `reasoning::analogy` — structure-preserving maps between domains (functors)
- `reasoning::equivalence` — equivalence relations
- `reasoning::opposition` — opposites and contrasts
- `reasoning::context` — disambiguation by context

### Layer 4: praxis::engine (Runtime Enforcement)

Defines how things CHANGE. Depends on ontology, category, and logic.

- `Situation` — immutable world state snapshot
- `Action` — a proposed state transition
- `Precondition` — rule checked before action application
- `Engine` — validates, applies, traces, supports undo/redo
- `EngineError` — `Violated` (preconditions blocked) or `LogicalError` (ontological contradiction)
- `Trace` — full action history with human-readable dump

## Dependency Flow

```
praxis-domains
    ↓ depends on
praxis::engine
    ↓ depends on
praxis::ontology
    ↓ depends on
praxis::category
    ↓ depends on
praxis::logic
```

Domain modules never depend on each other. Each is a standalone enforcement engine for its domain.

## Domain Organization

```
praxis-domains
├── science/         — math, physics, music, colors, calculator
├── games/           — chess, rubik, tetris, simon
└── systems/
    ├── communication/protocols/ — http
    ├── transportation/          — elevator, traffic
    └── government/judicial/     — cases, motions, rulings
```

## Engine Lifecycle

```
1. Create Engine with initial Situation + Preconditions + apply function
2. Call engine.next(action)
   a. All preconditions checked against current situation + action
   b. If any violated -> Err(EngineError::Violated) with violations + engine for rollback
   c. If all satisfied -> apply function called
   d. If apply succeeds -> new situation, trace records success
   e. If apply fails -> Err(EngineError::LogicalError) with reason + engine for rollback
3. Call engine.back() to undo (moves current to redo stack)
4. Call engine.forward() to redo
5. New next() after back() clears the redo stack (branch point)
```

## Design Decisions

**Situations are immutable.** Every action produces a new situation. The old one is preserved in the history stack. This enables undo/redo without mutation.

**Preconditions are separate from apply.** The precondition layer validates rules. The apply function transforms state. They are checked independently.

**EngineError returns the engine.** Both `Violated` and `LogicalError` return the engine so the caller can rollback. The system never panics — contradictions are data, not crashes.

**Rich enums carry context.** Every enum variant carries the data of HOW it got there. `MotionStatus::Granted { ruling_date, judge, order }` not `MotionStatus::Granted`. No information is lost between state transitions.

**Logical composition uses trait objects.** `AllOf` and `AnyOf` take `Vec<Box<dyn Proposition>>` so you can mix different proposition types in one logical expression.

**Property-based testing is the primary verification.** Domain invariants are expressed as proptest properties that hold for all generated inputs, not just hand-picked examples.
