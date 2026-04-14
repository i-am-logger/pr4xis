# Write a Domain Axiom

This page covers how to write a domain-specific axiom that the engine enforces at runtime. For the structural axioms that come for free with `define_ontology!` (no cycles in taxonomies, antisymmetric is-a, weak supplementation in mereologies, etc.), see [Architecture](../understand/architecture.md). This page is about the axioms *your* domain adds on top.

## When you need a domain axiom

Whenever a published source says "this must be true" about your domain, and the truth isn't already enforced by category laws or the standard reasoning systems. Examples:

- "A chess king moves at most one square per turn" — chess
- "Voltage = current × resistance" — electromagnetism
- "Energy is conserved across drops and rises" — mechanics
- "Heisenberg uncertainty: Δx · Δp ≥ ℏ/2" — quantum
- "Speed of light is the upper bound on velocity" — relativity
- "An enzyme catalyzes exactly one reaction class" — biochemistry

If the source says it, the axiom should encode it. If the engine ever produces a state where the axiom doesn't hold, the engine should refuse the action that produced it.

## What an axiom is

An axiom in pr4xis is a Rust type that implements the `Axiom` trait:

```rust
use pr4xis::logic::Axiom;

pub struct MyAxiom;

impl Axiom for MyAxiom {
    fn name(&self) -> &'static str {
        "MyAxiom"
    }

    fn description(&self) -> &'static str {
        "What the axiom says, in one sentence. Cite the source."
    }

    fn check(&self) -> bool {
        // Return true if the axiom holds, false otherwise.
        // For axioms that depend on a situation, take the situation as a parameter
        // (the actual signature is per-domain — look at existing axioms for the right shape).
        true
    }
}
```

Axioms come in two shapes:

1. **Unconditional axioms.** Statements that must hold regardless of state. "The speed of light is constant in all reference frames." These are checked at compile time or at startup.
2. **State-dependent axioms.** Statements that must hold *given the current situation*. "The total energy of the system equals the total energy at the previous time step." These are checked by the engine before applying any action.

Most domain axioms are state-dependent — they live as `Precondition` implementations on the engine, not as pure `Axiom` impls. The two are related but distinct.

## The Precondition pattern (state-dependent axioms)

If your axiom needs to know the current situation to be checked, implement `Precondition` instead of (or in addition to) `Axiom`:

```rust
use pr4xis::engine::{Precondition, PreconditionResult, Situation, Action};

pub struct EnergyConservation;

impl Precondition<MySituation, MyAction> for EnergyConservation {
    fn check(&self, situation: &MySituation, action: &MyAction) -> PreconditionResult {
        let energy_before = situation.total_energy();
        let energy_after = situation.simulate_action(action).total_energy();

        if (energy_before - energy_after).abs() < EPSILON {
            PreconditionResult::Satisfied {
                rule: "EnergyConservation",
                reason: "Total energy is preserved by the action",
            }
        } else {
            PreconditionResult::Violated {
                rule: "EnergyConservation",
                reason: format!(
                    "Action would change total energy by {}",
                    energy_after - energy_before
                ),
                situation: situation.clone(),
                attempted_action: action.clone(),
            }
        }
    }
}
```

Two things to note:

1. **The result carries context.** `Satisfied` carries the rule name and a reason. `Violated` carries the rule name, a reason, the situation, and the attempted action. Both can be inspected by the caller — the trace shows exactly which rule passed and why.
2. **The check is total.** Every possible (situation, action) pair must produce one result or the other. There is no "I don't know" — if the axiom depends on something the situation doesn't carry, the situation needs to be enriched, not the precondition.

## Wiring the axiom into the engine

When you create an engine, you pass it the list of preconditions:

```rust
use pr4xis::engine::Engine;

let engine = Engine::new(
    initial_situation,
    vec![
        Box::new(EnergyConservation),
        Box::new(SpeedLimit),       // another precondition
        Box::new(PositiveDuration), // another
    ],
    apply_action,
);
```

Every call to `engine.next(action)` runs every precondition in order. If any return `Violated`, the engine returns `EngineError::Violated` with the list of violations and a recoverable engine reference. If all return `Satisfied`, the apply function runs and produces the new situation.

## Property-based testing for axioms

The strongest verification for a domain axiom is property-based: define a property the axiom should ensure, then let `proptest` generate random inputs and look for counterexamples. Pattern:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn energy_is_conserved_for_all_drops(initial_height in 0.0_f64..1000.0) {
        let situation = MySituation::new_at_rest(initial_height);
        let action = MyAction::Drop;
        let result = EnergyConservation.check(&situation, &action);
        prop_assert!(matches!(result, PreconditionResult::Satisfied { .. }));
    }
}
```

The proptest harness will run hundreds of random inputs and shrink any failure to the smallest counterexample, which makes domain bugs much easier to find than hand-picked unit tests.

## Citing the source

Every axiom needs a citation. The recommended pattern is a doc comment on the axiom struct:

```rust
/// Energy conservation under closed-system mechanics.
///
/// **Source:** Newton's *Principia* (1687), Book I, Definitions and Laws of Motion.
/// Modern formulation: any standard mechanics text, e.g., Goldstein,
/// *Classical Mechanics* (3rd ed., 2002), Chapter 1.
///
/// **Statement:** For an isolated system, the total energy E = KE + PE is
/// constant under all permitted actions.
pub struct EnergyConservation;
```

The doc comment becomes part of the axiom's public documentation and is what the trace will show to users when the axiom fires. Make it precise enough that someone reading the failing trace can find the exact paragraph in the source where the axiom comes from.

## What NOT to do

- **Don't write axioms for "obvious" facts that aren't in any source.** Pr4xis is not a common-sense engine; every axiom must be sourceable. If you can't cite it, don't encode it.
- **Don't use floating-point equality in axiom checks.** Use an epsilon comparison or rational arithmetic. Floating-point equality fails for reasons that have nothing to do with the axiom.
- **Don't make axioms catch unrelated bugs.** If your `EnergyConservation` precondition is also catching velocity-out-of-range errors, split them. Each axiom should fail for one reason.
- **Don't make axioms expensive.** Preconditions run on every engine action. If checking your axiom requires solving a hard problem, cache the result or restructure the check. The engine is meant to be fast.

## Where to look in the codebase

- `crates/domains/src/natural/physics/relativity.rs` — the `SpeedLimit` axiom enforcing `v < c`
- `crates/domains/src/natural/physics/energy.rs` — `EnergyConservation` and `PhysicalConstraints` for KE↔PE transformations
- `crates/domains/src/social/games/chess/preconditions.rs` — `GameNotOver`, `PieceExists`, `OwnPiece`, `LegalMove`
- `crates/pr4xis/src/logic/axiom.rs` — the `Axiom` trait
- `crates/pr4xis/src/engine/precondition.rs` — the `Precondition` trait

## Related

- [Build an ontology from a paper](build-ontology-from-paper.md) — the upstream tutorial; if you are writing axioms, you are extending an ontology you already authored
- [Compose via functor](compose-via-functor.md) — when an axiom from one domain implies an axiom in another, the implication is a functor
- [Concepts](../understand/concepts.md) — the categorical machinery axioms plug into
- [Architecture](../understand/architecture.md) — the engine that runs the preconditions
- [Glossary: Axiom](../reference/glossary.md#axiom), [Glossary: Precondition](../reference/glossary.md#precondition)

---

- **Document date:** 2026-04-14
