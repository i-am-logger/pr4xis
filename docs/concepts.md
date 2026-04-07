# Concepts

## Ontology vs Praxis

**Ontology** defines what things ARE — static rules that don't change at runtime. A knight always moves in an L shape. Division by zero is always invalid. These are structural truths.

**Praxis** applies those rules to live state. A chess game has a board that changes with each move. The ontology says "a knight moves in an L" — praxis enforces it by checking preconditions before each action and blocking violations.

The split: if the rule is true regardless of state, it's ontology. If it depends on current state, it's praxis.

## Situations and Actions

A **Situation** is an immutable snapshot of the world. A chess board at move 15. An elevator at floor 3 going up. A case in the Discovery phase.

An **Action** is a proposed change. Moving a piece. Pressing a button. Filing a motion.

Situations never mutate. Actions produce NEW situations. The old situation is preserved in the history stack.

## Preconditions

A **Precondition** is a rule that must hold before an action can be applied. It checks the current situation and the proposed action, and returns either:

- `Satisfied { rule, reason }` — the rule passes, here's why
- `Violated { rule, reason, situation, attempted_action }` — blocked, here's the full diagnostic

Both carry context. Success isn't just `true` — it explains which rule passed and why. This makes traces useful for debugging and auditing.

## The Engine

The Engine is the runtime loop:

1. Receive an action
2. Check all preconditions
3. If any fail → return Err with violations, record in trace
4. If all pass → apply the action, push previous state to history, record in trace
5. Return the new engine (with updated state)

The engine is consumed on each `.next()` call (ownership transfer). This prevents using a stale engine after a state change.

## Back / Forward

The engine maintains two stacks:

- **past** — previous situations (for undo)
- **future** — undone situations (for redo)

`back()` pops from past, pushes current to future.
`forward()` pops from future, pushes current to past.
`next()` clears the future stack (new branch point).

## Logical Composition

Rules can be composed using boolean logic:

- `AllOf` — all must be satisfied (AND)
- `AnyOf` — at least one must be satisfied (OR)
- `Not` — must NOT be satisfied
- `Implies` — if A then B (vacuously true if A is false)
- `Threshold` — value must be less/greater/equal to a bound
- `Compare` — compare two measurable values from the same context

These are generic — they work for any domain, not just legal rules.

## Rich Enums

Every enum variant carries the context of how it got there:

```rust
// Not this:
enum Status { Pending, Granted, Denied }

// This:
enum Status {
    Pending { filed: Date, movant: Entity },
    Granted { date: Date, judge: Entity, order: String },
    Denied { date: Date, judge: Entity, reason: String },
}
```

No information is lost between state transitions. When you pattern-match on a `Granted` status, you know who granted it, when, and what the order says.

## Property-Based Testing

Domain invariants are expressed as properties that hold for ALL inputs, not just hand-picked examples:

- "No legal chess move leaves the king in check" — tested with random squares
- "Rubik's cube always has 9 of each color" — tested with random move sequences
- "NOT NOT x = x" — tested with random integers
- "Elevator never exceeds capacity" — tested with random load sequences

proptest generates hundreds of random test cases per property, finding edge cases that unit tests miss.

## Category Theory Foundation

The mathematical layer provides formal guarantees:

- **Identity law** — composing with identity produces the same morphism
- **Associativity** — (f∘g)∘h = f∘(g∘h)
- **Closure** — composable morphisms always produce a result
- **Functor laws** — structure-preserving maps preserve identity and composition

These aren't just academic — they prove that your domain model has no dead states, no unreachable objects, and no broken compositions. The compiler and proptest verify the math.
