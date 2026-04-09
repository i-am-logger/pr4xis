# Praxis

[![License: CC BY-NC-SA 4.0](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc-sa/4.0/)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-1790-brightgreen)](.)

Axiomatic intelligence. Provably correct reasoning by traversing and composing ontologies. Define rules as category theory, enforce them as axioms, verify with property-based testing. From chess to quantum mechanics to natural language — if the rules can be stated, praxis proves they hold.

> "Every good regulator of a system must be a model of that system."
> -- Conant & Ashby (1970)

## Architecture

Praxis has five layers. Each layer depends only on the layers below it.

```
praxis::logic        — Axioms, propositions, inference (deduction, induction, abduction)
praxis::category     — Entity, Relationship, Category, Functor, Morphism
praxis::ontology     — Ontology, Quality, Reasoning, Upper Ontology (DOLCE)
praxis::engine       — Situation, Action, Precondition, Engine, Trace
praxis::codegen      — Build-time ontology generation from data sources
```

| Layer | Module | Purpose |
|---|---|---|
| **Logic** | `praxis::logic` | Axioms, propositions, truth tables, inference (deduction, induction, abduction) |
| **Category** | `praxis::category` | Entity, Relationship, Category, Functor, NaturalTransformation — the mathematics |
| **Ontology** | `praxis::ontology` | Ontology, Quality, Reasoning (7 association types), Upper Ontology (DOLCE-aligned) |
| **Engine** | `praxis::engine` | Situation, Action, Precondition, Engine, Trace — how things CHANGE |
| **Codegen** | `praxis::codegen` | Build-time ontology generation from external data (feature-gated) |

### Reasoning Ontology

Seven association types formalize how concepts relate. Each is a reusable category pattern with axioms and query functions:

| Association | What it models | Example |
|---|---|---|
| **Taxonomy** (is-a) | Hierarchical subsumption | Dog is-a Mammal is-a Animal |
| **Mereology** (has-a) | Part-whole relationships | Car has-a Engine has-a Piston |
| **Causation** (causes) | Causal chains | Heating causes Boiling causes Steam |
| **Analogy** (is-like) | Structure-preserving maps | Electric field is-like Gravitational field |
| **Equivalence** (synonymy) | Interchangeability | Big = Large = Huge |
| **Opposition** (antonymy) | Semantic negation | Hot opposes Cold |
| **Context** (disambiguation) | Meaning selection by signal | "bank" + money = finance; "bank" + river = riverbank |

### Upper Ontology (DOLCE)

Praxis classifies everything that exists using categories from [DOLCE](https://arxiv.org/abs/2308.01597) (Descriptive Ontology for Linguistic and Cognitive Engineering):

| Being | What it is | Examples |
|---|---|---|
| **Physical Endurant** | Tangible, persists | Traffic light, elevator |
| **Social Object** | Exists by agreement | Chess rules, XML spec, English language, legal system |
| **Mental Object** | Exists in cognition | Concept, belief, intention |
| **Abstract Object** | Timeless | Number, mathematical proof, category |
| **Event** | Instantaneous | A chess move, a key press |
| **Process** | Extended over time | A chess game, a trial, a conversation |
| **Quality** | Measurable property | Color, weight, pitch |

A functor from praxis's type system to DOLCE proves the alignment preserves structure (identity and composition laws verified).

### Ontology Evolution Pattern

When transforming ontologies, praxis doesn't rewrite — it creates the new ontology alongside the old and proves the mapping via functor. The functor guarantees structure preservation. This pattern applies to every ontology migration: domain restructuring, upper ontology alignment, language translation, format conversion.

## Quick Start

```rust
use praxis::engine::{Engine, Situation, Action, Precondition};
use praxis_domains::technology::games::chess::{new_game, ChessAction, Square};

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

| Crate | Description |
|---|---|
| `praxis` | Core framework — category theory, ontology, reasoning, logic, engine, codegen |
| `praxis-domains` | Applied domains — science, games, systems, linguistics |
| `praxis-examples` | Logic puzzles (tested in CI) |

### What's Inside

**praxis** (core):
- `praxis::category` — Entity, Relationship, Category, Functor, Morphism, NaturalTransformation
- `praxis::ontology` — Ontology, Quality, Axiom
- `praxis::ontology::reasoning` — Taxonomy, Mereology, Causation, Analogy, Equivalence, Opposition, Context
- `praxis::ontology::upper` — DOLCE-aligned Being classification, Classified trait, PraxisToDolce functor
- `praxis::engine` — Situation, Action, Precondition, Engine, Trace, back/forward
- `praxis::logic` — Proposition, AllOf/AnyOf/Not/Implies, truth tables, Deduction, Induction, Abduction
- `praxis::codegen` — OntologyBuilder, WordNet XML-LMF parser, Rust code generator (feature-gated)

**praxis-domains**:
- `science::math` — Pythagorean theorem, quadratic formula, Fibonacci, primes, sets, Feynman path integrals
- `science::physics` — Mechanics (F=ma), energy conservation, Ohm's law, Maxwell's equations, relativity (E=mc²), quantum (Heisenberg)
- `science::music` — Notes, intervals, scales, chords, consonance
- `science::colors` — RGB, WCAG contrast, mixing modes, blending
- `science::calculator` — Scientific calculator with exact rationals, complex numbers, unit conversion
- `science::linguistics::lexicon` — Parts of speech as rich types (Noun, Verb, Determiner, etc.), English vocabulary, lexical categories
- `science::linguistics::grammar` — Syntax trees, phrase structure, subject-verb agreement, parse engine
- `science::linguistics::semantics` — Predicate logic, semantic roles (Agent, Patient), meaning representation, interpretation
- `science::linguistics::pragmatics` — Speech acts, intent, discourse context, topic tracking
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
| Taxonomy transitivity | Dog is-a Animal (3-hop path verified via BFS) |
| Quality inheritance | Dogs inherit "is_alive" from LivingThing ancestor |
| Causal asymmetry | Boiling does NOT cause Heating (direction enforced) |
| Mereological supplementation | Every whole has at least two parts |
| Equivalence transitivity | Big = Large = Huge (transitive closure) |
| Opposition exclusivity | Antonyms cannot be synonyms |
| DOLCE functor laws | Praxis→DOLCE mapping preserves identity and composition |
| Subject-verb agreement | "the dog runs" valid; "the dog run" rejected |
| Semantic role extraction | "the dog sees the cat" → see(dog:Agent, cat:Patient) |
| Königsberg bridges impossible | Exhaustive search of all paths from all starting nodes |
| Chess rules complete | 5 famous games (1851–1858) replayed from PGN to checkmate |
| F = ma | Δv = (F/m)·Δt verified for all random mass/force/time |
| Energy conservation | KE + PE = constant verified for all inputs |
| v < c (speed limit) | Engine blocks any velocity ≥ speed of light |
| ΔxΔp ≥ ℏ/2 (Heisenberg) | Measuring position more precisely increases momentum uncertainty |
| Speed of light derived | c = 1/√(μ₀ε₀) from Maxwell's equations |
| a² + b² = c² | Pythagorean theorem enforced on every triangle transformation |
| Goldbach conjecture | Every even n > 2 decomposed into two primes (verified to 1000) |
| NAND is universal | AND, OR, NOT constructed from NAND gates alone |
| Monty Hall: switching wins 2/3 | Property test: switching always wins when initial choice is wrong |
| Modus ponens soundness | Deduction with explicit implication rule; coincidence detection prevented |

## Philosophical Foundation

Praxis draws from three intellectual traditions:

- **Category theory** — the mathematics of composition. Objects, morphisms, functors, natural transformations provide the formal structure.
- **DOLCE** (Descriptive Ontology for Linguistic and Cognitive Engineering) — the philosophical classification of being. Everything is Endurant (persists), Perdurant (happens), or Quality (measurable).
- **Systems thinking and cybernetics** — the science of wholes, feedback, and self-regulation. The Engine is a cybernetic control loop. Metacognition is second-order cybernetics.

### Principles

- **Nothing mechanical.** If praxis interacts with data, it must understand that data through an ontology. No blind parsing.
- **Ontology evolution via functor.** When transforming ontologies, create the new one alongside, prove the mapping via functor, save the transformation. Never rewrite.
- **Rich types, not enums with options.** Domain entities carry their structure in the type system. A Noun is structurally different from a Verb.
- **Nothing in the README until there's a proof.** This document describes only what the codebase demonstrates.

## Documentation

- [Architecture](docs/architecture.md) — layer design, dependency flow, engine lifecycle
- [Concepts](docs/concepts.md) — ontology vs praxis, situations, actions, preconditions
- [Domain Crates](docs/domain-crates.md) — enforcement details for each domain
- [Foundations](docs/foundations.md) — intellectual lineage: category theory, DOLCE, cybernetics, systems thinking

## Testing

Property-based testing with [proptest](https://github.com/proptest-rs/proptest).

```bash
cargo test --workspace
```

## License

CC BY-NC-SA 4.0 — see [LICENSE](LICENSE).
