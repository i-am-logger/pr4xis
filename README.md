# Praxis

[![CI](https://github.com/i-am-logger/praxis/actions/workflows/ci.yml/badge.svg)](https://github.com/i-am-logger/praxis/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/i-am-logger/praxis/branch/master/graph/badge.svg)](https://codecov.io/gh/i-am-logger/praxis)
[![License: CC BY-NC-SA 4.0](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc-sa/4.0/)

[![Nix](https://img.shields.io/badge/Nix-5277C3?logo=nixos&logoColor=white)](https://nixos.org)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Crates](https://img.shields.io/badge/crates-18-blue)](https://github.com/i-am-logger/praxis/tree/master/crates)
[![Tests](https://img.shields.io/badge/tests-787-brightgreen)](https://github.com/i-am-logger/praxis/actions/workflows/ci.yml)

Prove your domain is correct. Define rules as ontology, enforce them as axioms, verify with property-based testing. From chess to quantum mechanics — if the rules can be stated, praxis proves they hold.

## Features

- **Rule enforcement** — preconditions validate every state transition before it happens
- **Rich diagnostics** — violations carry full context (which rule, why, what state, what was attempted)
- **Undo/redo** — back() and forward() through state history
- **Trace recording** — every action logged with human-readable dump
- **Logical composition** — AND, OR, NOT, IMPLIES, thresholds for building complex rules
- **Property-based testing** — invariants verified across thousands of random inputs via proptest
- **Category theory foundation** — identity, associativity, closure laws proven for domain models
- **Ontology-driven physics** — Newton's laws, energy conservation, Heisenberg uncertainty as enforceable axioms
- **Propositional logic** — 8 connectives, truth tables, De Morgan's proofs, NAND universality
- **PGN parser** — replay authentic chess games from standard notation files

## Proofs

| What | How | Crate |
|---|---|---|
| Königsberg bridges are impossible | Exhaustive search of all paths from all starting nodes | praxis-puzzles |
| Chess rules are complete | 5 famous games (1851–1858) replayed from PGN to checkmate | praxis-chess |
| Rubik's cube preserves colors | Every random move sequence maintains 9 of each color | praxis-rubik |
| Monty Hall: switching wins 2/3 | Property test: switching always wins when initial choice is wrong | praxis-puzzles |
| F = ma | Δv = (F/m)⋅Δt verified for all random mass/force/time | praxis-physics |
| Energy conservation | KE + PE = constant after drop/rise, verified for all inputs | praxis-physics |
| V = IR (Ohm's law) | Enforced as precondition on every circuit change | praxis-physics |
| v < c (speed limit) | Engine blocks any velocity ≥ speed of light | praxis-physics |
| c = 1/√(μ₀ε₀) (speed of light) | Derived from Maxwell's equations, not hardcoded | praxis-physics |
| ΔxΔp ≥ ℏ/2 (Heisenberg) | Enforced: measuring position more precisely increases momentum uncertainty | praxis-physics |
| a² + b² = c² | Pythagorean theorem enforced as precondition on every triangle transformation | praxis-math |
| Roots satisfy ax²+bx+c=0 | Verified for all random coefficients via proptest | praxis-math |
| Goldbach conjecture | Every even n > 2 decomposed into two primes (verified to 1000) | praxis-math |
| Cassini's identity | F(n-1)F(n+1) - F(n)² = (-1)^n verified for n=1..25 | praxis-math |
| Binet's formula | φ^n/√5 matches fib(n) for n=0..25 | praxis-math |
| De Morgan's laws (sets) | A∖(B∪C) = (A∖B)∩(A∖C) verified for random sets | praxis-math |
| De Morgan's laws (logic) | !(A && B) == (!A \|\| !B) proven exhaustively | praxis-logic |
| NAND is universal | NOT, AND, OR all expressed via NAND, proven exhaustively | praxis-logic |
| Modus ponens | (A && (A → B)) → B proven by truth table | praxis-logic |
| Category laws hold | Identity, associativity, closure verified exhaustively + proptest | praxis-category |
| NOT NOT x = x | Double negation elimination proven for all integers | praxis-logic |
| A→B ≡ ¬A∨B | Implication equivalence verified by proptest | praxis-logic |
| N ⊂ Z ⊂ Q ⊂ R ⊂ C | Number hierarchy containment chain axiom | praxis-math |
| N-Queens: no attacks | Every placement verified: no shared column or diagonal | praxis-puzzles |
| Sudoku: no duplicates | Row, column, and box uniqueness enforced and tested | praxis-puzzles |
| sin²+cos² = 1 | Pythagorean identity verified for all angles | praxis-calculator |

## Crates

### Core

| Crate | Purpose |
|---|---|
| `praxis-category` | Category theory: Entity, Relationship, Category, Functor, NaturalTransformation |
| `praxis-logic` | Propositional logic + generic composition: Proposition, AllOf, AnyOf, Connective, truth tables |
| `praxis-ontology` | Ontology: Quality, Axiom, Ontology validation (re-exports praxis-logic) |
| `praxis-engine` | Runtime: Situation, Action, Precondition, Trace, back/forward |

### Domain (14 crates)

| Crate | Domain | Key enforcement |
|---|---|---|
| `praxis-chess` | Chess | Check, checkmate, castling, en passant, promotion, pins, PGN replay |
| `praxis-calculator` | Scientific calculator | Domain constraints, exact rationals, number domain ontology |
| `praxis-rubik` | Rubik's cube | Group theory, 18 moves, color invariant |
| `praxis-elevator` | Elevator dispatch | Capacity, direction commitment, no starvation |
| `praxis-traffic` | Traffic signals | Intersection conflict prevention, timing |
| `praxis-tetris` | Tetris | Collision, rotation, wall kicks, line clears |
| `praxis-simon` | Simon Says | Sequence memory, state machine |
| `praxis-http` | HTTP | Method semantics, connection state machine, retries |
| `praxis-music` | Music theory | Scales, chords, intervals, MIDI range |
| `praxis-colors` | Color theory | WCAG contrast, mixing modes, blending |
| `praxis-legal` | Legal cases | Case lifecycle, motions, rulings, rich state enums |
| `praxis-puzzles` | Logic puzzles | 11 puzzles (see below) |
| `praxis-math` | Mathematics | Pythagorean theorem, quadratic formula, Fibonacci, primes, Feynman path integrals |
| `praxis-physics` | Physics | Mechanics (F=ma), Maxwell's equations, relativity (E=mc²), quantum (Heisenberg) |

### Puzzles

River crossing, Tower of Hanoi, water jugs, missionaries & cannibals, Monty Hall, Byzantine generals, prisoner's dilemma, N-queens, knight's tour, Sudoku, bridges of Königsberg.

## Dependency Graph

```
praxis-category          (math: Entity, Category, Functor)
    ↓
praxis-logic             (propositional logic + generic composition)
    ↓
praxis-ontology          (Quality, Axiom, Ontology validation)
    ↓
praxis-engine            (Situation, Action, Precondition, Trace)
    ↓
domain crates            (chess, physics, math, puzzles, ...)
```

## Documentation

- [Architecture](docs/architecture.md) — three-layer design, dependency flow, engine lifecycle, design decisions
- [Concepts](docs/concepts.md) — ontology vs praxis, situations, actions, preconditions, logical composition, rich enums, property-based testing
- [Domain Crates](docs/domain-crates.md) — enforcement details for all domain crates

## Testing

787 tests with property-based testing ([proptest](https://github.com/proptest-rs/proptest)).

```bash
cargo test --workspace
```

## License

CC BY-NC-SA 4.0 — see [LICENSE](LICENSE).
