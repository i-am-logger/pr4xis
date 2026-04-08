# Praxis

[![CI](https://github.com/i-am-logger/praxis/actions/workflows/ci.yml/badge.svg)](https://github.com/i-am-logger/praxis/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/i-am-logger/praxis/branch/master/graph/badge.svg)](https://codecov.io/gh/i-am-logger/praxis)
[![License: CC BY-NC-SA 4.0](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc-sa/4.0/)

[![Nix](https://img.shields.io/badge/Nix-5277C3?logo=nixos&logoColor=white)](https://nixos.org)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Crates](https://img.shields.io/badge/crates-18-blue)](https://github.com/i-am-logger/praxis/tree/master/crates)

**Model the rules of any domain. Enforce them automatically.**

Praxis is a Rust framework for defining and enforcing the rules of any domain — business, scientific, or recreational. You describe *what's allowed* using formal ontologies, and the engine guarantees every state transition is valid before it happens.

## Why Praxis?

Every domain has rules. Financial regulations say when to flag a suspicious transaction. Chess says a bishop moves diagonally. Physics says nothing travels faster than light. These rules share a common structure: **a situation, an action, and conditions that must hold**.

Praxis gives you a single framework to model all of them:

- **Define** your domain's entities, relationships, and constraints
- **Enforce** rules as preconditions on every state transition
- **Prove** your model is correct with property-based testing
- **Trace** every action with full audit trails and undo/redo

Instead of scattering validation logic across your codebase, you declare the rules once and let the engine enforce them everywhere.

## Use Cases

### Business & Compliance

| Domain | What Praxis Models |
|---|---|
| **BSA/AML** | Suspicious activity rules, transaction monitoring thresholds, SAR filing obligations |
| **Fraud Detection** | Transaction velocity patterns, risk scoring ontologies, anomaly classification |
| **Blockchain & Micropayments** | Smart contract preconditions, payment flow state machines, settlement rules |
| **Regulatory Compliance** | KYC workflows, sanctions screening, audit trail enforcement, reporting deadlines |
| **Legal** | Case lifecycles, evidence standards, motion workflows, burden of proof |

### Science & Engineering

| Domain | What Praxis Models |
|---|---|
| **Physics** | Newton's laws, Maxwell's equations, energy conservation, Heisenberg uncertainty |
| **Mathematics** | Number theory proofs, set theory, Fibonacci properties, Pythagorean theorem |
| **Music Theory** | Scales, chords, intervals, MIDI constraints, consonance/dissonance |
| **Color Science** | WCAG contrast ratios, color mixing, blending modes |

### Games & Puzzles

| Domain | What Praxis Models |
|---|---|
| **Chess** | Legal moves, check/checkmate, castling, en passant, PGN game replay |
| **Rubik's Cube** | 18 face moves, group theory, color invariant preservation |
| **Classic Puzzles** | Konigsberg bridges, N-queens, Tower of Hanoi, Monty Hall, and more |

## How It Works

Praxis has four layers:

```
praxis-category     Formal foundations (entities, relationships, composition)
      |
praxis-logic        Propositional logic (AND, OR, NOT, IMPLIES, truth tables)
      |
praxis-ontology     Domain modeling (qualities, axioms, validation)
      |
praxis-engine       Runtime enforcement (preconditions, traces, undo/redo)
      |
  your domain       Chess, finance, compliance, physics, ...
```

You implement a few traits — `Situation` (your state), `Action` (what can happen), and `Precondition` (what must be true) — and the engine handles enforcement, history, and diagnostics.

## Quick Example

```rust
use praxis_engine::{Engine, Situation, Action, Precondition};

// 1. Define your state
struct Account { balance: f64, frozen: bool }

// 2. Define what can happen
enum Transaction { Deposit(f64), Withdraw(f64), Freeze }

// 3. Define what must be true
struct SufficientFunds;
impl Precondition<Account> for SufficientFunds {
    fn check(&self, state: &Account, action: &Transaction) -> Option<String> {
        match action {
            Transaction::Withdraw(amount) if *amount > state.balance =>
                Some(format!("Insufficient funds: {} > {}", amount, state.balance)),
            _ => None, // no violation
        }
    }
}

// 4. The engine enforces the rules
let engine = Engine::new(account, vec![Box::new(SufficientFunds)]);
// Overdraft? Rejected with full diagnostic.
// Valid withdrawal? Applied and traced.
```

## Quick Start

Add any domain crate to your project:

```bash
cargo add praxis-chess    # or any domain crate
```

Or use the core crates to build your own domain:

```bash
cargo add praxis-engine praxis-ontology
```

See the [Getting Started Guide](docs/getting-started.md) for a step-by-step tutorial.

## Crates

### Core

| Crate | Description |
|---|---|
| [`praxis-category`](crates/praxis-category) | Category theory: entities, relationships, functors, natural transformations |
| [`praxis-logic`](crates/praxis-logic) | Propositional logic, generic composition (AllOf, AnyOf, threshold) |
| [`praxis-ontology`](crates/praxis-ontology) | Domain modeling: qualities, axioms, ontology validation |
| [`praxis-engine`](crates/praxis-engine) | Runtime enforcement: preconditions, traces, undo/redo |

### Domain

| Crate | Domain | What It Enforces |
|---|---|---|
| [`praxis-chess`](crates/praxis-chess) | Chess | Legal moves, check, checkmate, castling, en passant, PGN replay |
| [`praxis-calculator`](crates/praxis-calculator) | Scientific calculator | Domain constraints, exact rationals, number hierarchy |
| [`praxis-rubik`](crates/praxis-rubik) | Rubik's cube | Group theory, 18 moves, color invariant |
| [`praxis-elevator`](crates/praxis-elevator) | Elevator dispatch | Capacity, direction commitment, starvation prevention |
| [`praxis-traffic`](crates/praxis-traffic) | Traffic signals | Intersection conflict prevention, timing |
| [`praxis-tetris`](crates/praxis-tetris) | Tetris | Collision, rotation, wall kicks, line clears |
| [`praxis-simon`](crates/praxis-simon) | Simon Says | Sequence memory, state machine |
| [`praxis-http`](crates/praxis-http) | HTTP | Method semantics, connection states, retries |
| [`praxis-music`](crates/praxis-music) | Music theory | Scales, chords, intervals, MIDI range |
| [`praxis-colors`](crates/praxis-colors) | Color science | WCAG contrast, mixing modes, blending |
| [`praxis-legal`](crates/praxis-legal) | Legal cases | Case lifecycle, motions, rulings, evidence |
| [`praxis-puzzles`](crates/praxis-puzzles) | Logic puzzles | 11 classic puzzles with formal proofs |
| [`praxis-math`](crates/praxis-math) | Mathematics | Number theory, set theory, Fibonacci, primes |
| [`praxis-physics`](crates/praxis-physics) | Physics | Mechanics, electromagnetism, relativity, quantum |

## What Praxis Proves

Every domain crate includes proofs — not just tests, but formal verification that the rules hold across all inputs:

| Proof | How | Crate |
|---|---|---|
| Konigsberg bridges are impossible | Exhaustive search of all paths | praxis-puzzles |
| Chess rules are complete | 5 famous games replayed to checkmate | praxis-chess |
| Rubik's cube preserves colors | Random sequences maintain 9 of each color | praxis-rubik |
| Monty Hall: switching wins 2/3 | Property test over all scenarios | praxis-puzzles |
| F = ma | Verified for all random mass/force/time | praxis-physics |
| Energy is conserved | KE + PE = constant after drop/rise | praxis-physics |
| v < c (speed limit) | Engine blocks any velocity >= speed of light | praxis-physics |
| Heisenberg uncertainty | Measuring position increases momentum uncertainty | praxis-physics |
| De Morgan's laws | Proven exhaustively for both logic and sets | praxis-logic, praxis-math |
| NAND is universal | NOT, AND, OR all built from NAND | praxis-logic |
| Category laws hold | Identity, associativity, closure verified | praxis-category |

[Full list of 56+ proofs](docs/proofs.md)

## Documentation

- **[Getting Started](docs/getting-started.md)** — build your first domain in 15 minutes
- **[Architecture](docs/architecture.md)** — three-layer design, dependency flow, engine lifecycle
- **[Concepts](docs/concepts.md)** — ontology, situations, actions, preconditions, composition
- **[Domain Crates](docs/domain-crates.md)** — enforcement details for all 14 domain crates

## Testing

Property-based testing with [proptest](https://github.com/proptest-rs/proptest). Every invariant is verified across thousands of random inputs.

```bash
cargo test --workspace
```

## License

CC BY-NC-SA 4.0 — see [LICENSE](LICENSE).
