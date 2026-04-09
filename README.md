# Praxis

[![License: CC BY-NC-SA 4.0](https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg)](https://creativecommons.org/licenses/by-nc-sa/4.0/)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-1790-brightgreen)](.)

Aristotle classified knowledge into three kinds:

- **Episteme** — knowing how things ARE. Science.
- **Techne** — knowing how to MAKE things. Technology.
- **Praxis** — knowing how to DO the right thing. Action guided by understanding.

This is praxis. A system that doesn't just compute — it understands what it's doing and can prove it's correct. Every rule is an axiom. Every transformation preserves structure. Every claim has a proof.

> "Every good regulator of a system must be a model of that system."
> — Conant & Ashby (1970)

The ontology IS the model. If praxis understands chess, it can prove every move is legal. If it understands English, it can prove the grammar is correct. If it understands physics, it can prove energy is conserved. The understanding and the proof are the same thing.

## How it works

Praxis defines the rules of a domain as an **ontology** — a formal description of what exists and how things relate. Then it enforces those rules through an **engine** that checks every action against the rules before allowing it.

```
Define rules (ontology) → Check rules (engine) → Prove rules hold (tests)
```

The math underneath is **category theory** — the science of composition. Things compose (combine) while preserving structure. A dog is a mammal, and a mammal is an animal, therefore a dog is an animal. That composition is guaranteed by the mathematics, not by code.

### What praxis knows

Praxis understands domains by building ontologies from academic research — not by hardcoding rules.

| Domain | What it knows | Source |
|---|---|---|
| **Physics** | F=ma, E=mc², Maxwell's equations, Heisenberg uncertainty | Newton, Einstein, Maxwell |
| **Chess** | Every rule, 5 famous games replayed to checkmate | FIDE Laws of Chess |
| **English** | Grammar, meaning, speech acts, 107,000 concepts | Lambek (1958), Montague (1973), WordNet |
| **Music** | Notes, intervals, scales, chords, consonance | Music theory |
| **Colors** | RGB, contrast ratios, WCAG accessibility | W3C standards |
| **Traffic** | Signal timing, intersection safety | Traffic engineering |
| **Law** | Case lifecycle, motions, rulings | Judicial procedure |
| **Logic** | Deduction, induction, abduction, truth tables | Aristotle, Peirce |

Each domain is connected to others through **functors** — structure-preserving maps. Chess IS a concurrent system (two players). Traffic IS a control system (feedback loop). These aren't metaphors — they're mathematical proofs that the structure is the same.

### Seven ways things relate

Praxis formalizes seven types of relationships between concepts:

| Relationship | Meaning | Example |
|---|---|---|
| **is-a** | Classification | A dog is a mammal |
| **has-a** | Composition | A car has an engine |
| **causes** | Causation | Heating causes boiling |
| **is-like** | Analogy | Electric fields are like gravitational fields |
| **equals** | Synonymy | Big = Large = Huge |
| **opposes** | Antonymy | Hot opposes Cold |
| **in context** | Disambiguation | "Bank" + money = finance; "bank" + river = riverbank |

### The engine

The engine is a control loop: observe the current state, check the rules, apply an action, observe the new state. This is cybernetics — the science of feedback and regulation (Wiener, 1948).

```rust
let game = new_game()
    .next(ChessAction::new(e2, e4))?  // checks all rules, applies move
    .next(ChessAction::new(e7, e5))?; // same — every move is verified

game.back()?          // undo (full history preserved)
game.forward()?       // redo
game.trace().dump()   // every action, every check, every result
```

If a move violates any rule, the engine rejects it and tells you why. Nothing gets through unchecked.

## Architecture

Five layers. Each depends only on the layers below it.

| Layer | What it does |
|---|---|
| **Logic** | Axioms, propositions, inference (deduction, induction, abduction) |
| **Category** | The mathematics — entities, relationships, composition, functors |
| **Ontology** | Domain knowledge — reasoning patterns, classification (DOLCE) |
| **Engine** | How things change — situations, actions, preconditions, history |
| **Codegen** | Build-time ontology generation from external data sources |

### Principles

- **Nothing mechanical.** If praxis interacts with data, it must understand that data through an ontology. No blind parsing.
- **Research first.** Every ontology is grounded in academic papers. Bugs are ontology gaps, fixed by research — not patches.
- **Composition over custom code.** Use existing ontologies and compose them via functors. Extend, don't reinvent.
- **Nothing in the README until there's a proof.** This document describes only what the codebase demonstrates.

## Proofs

A selection of what the test suite proves:

| Claim | Proof |
|---|---|
| A dog is an animal | Taxonomy transitivity: dog → mammal → animal (3-hop BFS) |
| Chess rules are complete | 5 famous games (1851-1858) replayed from PGN to checkmate |
| F = ma | Property test: Dv = (F/m)*Dt for all random inputs |
| Energy is conserved | KE + PE = constant for all inputs |
| Nothing goes faster than light | Engine blocks any velocity >= c |
| Heisenberg uncertainty | Measuring position more precisely increases momentum uncertainty |
| The speed of light is derivable | c = 1/sqrt(u0*e0) from Maxwell's equations |
| Edit distance is a proper metric | Triangle inequality proven by property-based testing |
| NAND is universal | AND, OR, NOT constructed from NAND alone |
| Monty Hall: always switch | Property test: switching wins 2/3 of the time |

## Crates

| Crate | What's in it |
|---|---|
| `praxis` | Core — category theory, ontology, reasoning, logic, engine |
| `praxis-domains` | Domains — physics, chess, music, linguistics, traffic, law, and more |
| `praxis-examples` | 11 classic puzzles (river crossing, Hanoi, Konigsberg, etc.) |

## Foundations

Praxis draws from and synthesizes:

- **Category theory** — Mac Lane, Awodey, Riehl, Spivak
- **Control systems** — Wiener (cybernetics), Ashby (requisite variety), Conant-Ashby (good regulator theorem)
- **Formal ontology** — DOLCE (Masolo et al.), Guarino, Gruber
- **Linguistics** — Lambek (categorial grammar), Montague (compositional semantics), Kamp (discourse representation)
- **Information theory** — Shannon, Damerau, Brill & Moore
- **Metacognition** — von Foerster (second-order cybernetics), Spencer-Brown (Laws of Form)

Full lineage with paper references: [docs/foundations.md](docs/foundations.md)

## Testing

Property-based testing with [proptest](https://github.com/proptest-rs/proptest).

```bash
cargo test --workspace
```

## License

CC BY-NC-SA 4.0 — see [LICENSE](LICENSE).
