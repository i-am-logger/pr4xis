# Praxis

[![License: CC BY-NC-SA 4.0](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc-sa/4.0/)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)

Axiomatic intelligence. Provably correct reasoning by traversing and composing ontologies. Define rules as category theory, enforce them as axioms, verify with property-based testing. From chess to quantum mechanics — if the rules can be stated, praxis proves they hold.

1198 proofs across physics, chess, music theory, and more execute in under a second on a single core.

## Architecture

Praxis has four layers. Each layer depends only on the layers below it.

```mermaid
graph BT
    E[praxis-domains] --> D[praxis::engine]
    D --> C[praxis::ontology]
    C --> B[praxis::category]
    B --> A[praxis::logic]

    A:::logic
    B:::category
    C:::ontology
    D:::engine
    E:::domains

    classDef logic fill:#4a90d9,color:#fff
    classDef category fill:#7b68ee,color:#fff
    classDef ontology fill:#e67e22,color:#fff
    classDef engine fill:#27ae60,color:#fff
    classDef domains fill:#95a5a6,color:#fff
```

| Layer | Module | Purpose |
|---|---|---|
| **Logic** | `praxis::logic` | Axioms, propositions, truth tables — the logical foundation |
| **Category** | `praxis::category` | Entity, Relationship, Category, Functor, Morphism — the mathematics |
| **Ontology** | `praxis::ontology` | Ontology, Quality, Reasoning (taxonomy, causation, mereology, analogy) — what things ARE and how they relate |
| **Engine** | `praxis::engine` | Situation, Action, Precondition, Engine, Trace — how things CHANGE |

## Quick Start

```rust
use praxis::engine::{Engine, Situation, Action, Precondition};
use praxis_domains::games::chess::{new_game, ChessAction, Square};

// Play chess with full rule enforcement
let game = new_game()
    .next(ChessAction::new(Square::new(4, 1), Square::new(4, 3)))? // e4
    .next(ChessAction::new(Square::new(4, 6), Square::new(4, 4)))?; // e5

game.situation()       // current board
game.back()?           // undo
game.forward()?        // redo
game.trace().dump()    // full history
```

## Crates

| Crate | Published | Description |
|---|---|---|
| `praxis` | Yes | Core framework — category theory, ontology, engine, logic |
| `praxis-domains` | Yes | Science, games, systems, government — all applied domains |
| `praxis-examples` | No | Logic puzzles (tested in CI) |

### What's Inside

**praxis** (core):
- `praxis::category` — Entity, Relationship, Category, Functor, Morphism
- `praxis::ontology` — Ontology, Quality, Axiom
- `praxis::engine` — Situation, Action, Precondition, Engine, Trace, back/forward
- `praxis::logic` — Proposition, truth tables, propositional logic

**praxis-domains**:
- `science::math` — Pythagorean theorem, quadratic formula, Fibonacci, primes, sets, Feynman path integrals
- `science::physics` — Mechanics (F=ma), energy conservation, Ohm's law, Maxwell's equations, relativity (E=mc²), quantum (Heisenberg)
- `science::music` — Notes, intervals, scales, chords, consonance
- `science::colors` — RGB, WCAG contrast, mixing modes, blending
- `science::calculator` — Scientific calculator with exact rationals, complex numbers, unit conversion
- `games::chess` — Full rules + PGN parser + famous games (Opera Game, Immortal Game, Evergreen Game)
- `games::rubik` — Group theory, 18 moves, color invariant
- `games::tetris` — Spatial constraints, rotation, wall kicks
- `games::simon` — Sequence memory enforcement
- `systems::communication::protocols::http` — Connection state machine, method semantics, retries
- `systems::transportation::elevator` — Multi-car dispatch with scheduling
- `systems::transportation::traffic` — Signal timing, intersection conflict prevention
- `systems::government::judicial` — Case lifecycle, motions, rulings, rich state enums

**praxis-examples**:
- 11 classic puzzles: river crossing, Tower of Hanoi, water jugs, missionaries & cannibals, Monty Hall, Byzantine generals, prisoner's dilemma, N-queens, knight's tour, Sudoku, bridges of Königsberg

## Proofs

| What | How |
|---|---|
| Königsberg bridges impossible | Exhaustive search of all paths from all starting nodes |
| Chess rules complete | 5 famous games (1851–1858) replayed from PGN to checkmate |
| F = ma | Δv = (F/m)⋅Δt verified for all random mass/force/time |
| Energy conservation | KE + PE = constant verified for all inputs |
| v < c (speed limit) | Engine blocks any velocity ≥ speed of light |
| ΔxΔp ≥ ℏ/2 (Heisenberg) | Measuring position more precisely increases momentum uncertainty |
| Speed of light derived | c = 1/√(μ₀ε₀) from Maxwell's equations |
| a² + b² = c² | Pythagorean theorem enforced on every triangle transformation |
| Goldbach conjecture | Every even n > 2 decomposed into two primes (verified to 1000) |
| NAND is universal | AND, OR, NOT constructed from NAND gates alone |
| Monty Hall: switching wins 2/3 | Property test: switching always wins when initial choice is wrong |

## Documentation

- [Architecture](docs/architecture.md) — layer design, dependency flow, engine lifecycle
- [Concepts](docs/concepts.md) — ontology vs praxis, situations, actions, preconditions
- [Domain Crates](docs/domain-crates.md) — enforcement details for each domain

## Testing

1198 tests with property-based testing ([proptest](https://github.com/proptest-rs/proptest)).

```bash
cargo test --workspace
```

## License

CC BY-NC-SA 4.0 — see [LICENSE](LICENSE).
