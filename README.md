<p align="center">
  <img src="docs/praxis-logo-light.jpg" alt="pr4xis" width="300"/>
</p>

<p align="center">
  <a href="https://creativecommons.org/licenses/by-nc-sa/4.0/"><img src="https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg" alt="License"/></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white" alt="Rust"/></a>
  <a href="https://pr4xis.dev"><img src="https://img.shields.io/badge/demo-pr4xis.dev-blue" alt="Live Demo"/></a>
</p>

# pr4xis — Axiomatic Intelligence

**pr4xis is a new kind of AI: axiomatic, not statistical.** LLMs predict the next token from training data. pr4xis derives the next claim from accepted axioms — the same way mathematicians prove theorems. Same input, same proof, every time. When it doesn't know, it says so. When it's wrong, the failed axiom is named.

Its axioms are not invented. They come from the formal scientific knowledge humans have already accumulated — [WordNet](https://wordnet.princeton.edu/) for language, [BioPortal](https://bioportal.bioontology.org/) for biomedicine, the [Gene Ontology](https://geneontology.org/), [DOLCE](https://www.loa.istc.cnr.it/dolce/overview.html) as a foundational layer, plus the 106 domain ontologies pr4xis has built on top. Each is years of careful work by domain experts. pr4xis is the engineering substrate that makes them work together with mathematical proof that every connection is sound — so the AI inherits, on day one, the rigor of every published source it stands on.

## A concrete result you can verify

A pr4xis [adjunction](https://en.wikipedia.org/wiki/Adjoint_functors) detected that the molecular biology ontology had collapsed two functionally distinct roles of voltage-gated potassium channels (Kv) into a single entity. Math surfaced the gap. A `ContextDef` resolution closed it. **85.2%** of molecular entities collapse in the Molecular ⊣ Bioelectric round-trip — every collapse is a missing distinction the math detected automatically.

Re-derive the numbers in 5 seconds:

```
cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture
```

Full story: [Gap Detection in Scientific Ontologies](docs/research/gap-detection.md).

## Demo

Live web demo at **[pr4xis.dev](https://pr4xis.dev)** — entirely in the browser via WebAssembly. No server, no GPU, no API key. Loads ~107K WordNet concepts at startup; runs the full linguistics pipeline (tokenization → [Lambek pregroup parsing](https://en.wikipedia.org/wiki/Pregroup_grammar) → [Montague semantics](https://en.wikipedia.org/wiki/Montague_grammar) → taxonomy traversal → response generation) on a single browser thread.

> Working surface, not a polished product. If a query breaks, [file an issue](https://github.com/i-am-logger/pr4xis/issues) — broken queries are bug reports, not user error.

## The name

Aristotle classified knowledge into three kinds: **episteme** (knowing how things are), **techne** (knowing how to make things), and **praxis** — *the doing itself, done well*. pr4xis is the doing: a system that *does* the right thing, and can *prove* it.

The architecture is justified by the **Good Regulator Theorem** (Conant & Ashby, 1970): every effective controller must contain a model of its system. In pr4xis, the ontology *is* the model. The deeper mathematical lineage — from Spencer-Brown's *Laws of Form* through Heim's syntrometric logic to applied category theory — lives in [Foundations](docs/understand/foundations.md).

## The problem

- **LLMs hallucinate by design.** Next-token prediction has no ground truth. When wrong, they cannot tell you which axiom failed because there are no axioms. For creative writing, this is fine. For domains where it kills people, it is unworkable.
- **Scientific ontologies do not compose.** WordNet, BioPortal, the Gene Ontology, DOLCE, OBO Foundry, Cyc, SUMO — rich, well-curated, almost entirely siloed. There is no executable substrate that lets you take one and combine it with another while verifying the result.

## pr4xis vs LLMs

|   | LLMs | pr4xis |
|---|---|---|
| **How it knows** | Learned from training data | Derived from axioms |
| **Correctness** | Probable — next-token prediction | Provable — laws + axioms verified at test time |
| **Hallucination** | Inherent — no ground truth | Impossible at the substrate level — every claim traces to a proof |
| **Determinism** | Stochastic (temperature, seed) | Absolute — same input, same proof, every time |
| **Traceability** | Opaque — billions of weights | Full proof path back to its axioms |
| **Cross-domain reasoning** | Implicit blending | Functors mathematically *prove* shared structure |
| **When wrong** | Confidently wrong, hard to find why | The failing axiom is named, with the trace that triggered it |
| **Undo / redo / branch** | None — each completion is final | Built into the engine: `back()`, `forward()`, branching from any prior state |
| **Missing knowledge** | Doesn't know what it doesn't know | [Adjunctions](https://en.wikipedia.org/wiki/Adjoint_functors) detect ontology gaps automatically |
| **Compute footprint** | GPU clusters, terabytes | Single core, one Rust binary |
| **Verifiable proofs in the codebase today** | 0 | **4,855** |

## What's in the box

Every number paired with the command that re-derives it. No marketing math.

| Claim | Value | Verify with |
|---|---|---|
| Tests in workspace | **4,855** | `cargo test --workspace` |
| Ontologies | **106** | `find crates/domains/src -name ontology.rs \| wc -l` |
| Cross-domain functor proofs | **61** | `grep -rn "impl Functor" crates/domains/src/ crates/pr4xis/src/ \| wc -l` |
| Lines of Rust | **136,296** | `find crates -name "*.rs" -not -path "*/target/*" \| xargs wc -l \| tail -1` |
| Crates | **8** | `ls crates/` → `chat`, `cli`, `domains`, `examples`, `pr4xis`, `pr4xis-derive`, `wasm`, `web` |
| Architecture layers | **5** | `logic` → `category` → `ontology` → `engine` → `codegen` (top-level under `crates/pr4xis/src/`) |

The 106 ontologies cover biomedical (biology, molecular, bioelectricity, biochemistry, immunology, pharmacology, pathology, mechanobiology, ...), sensor fusion, navigation, linguistics, formal mathematics, music, colors, and more. WordNet is integrated via build-time codegen. **Many more ontologies are still to be added** — importing BioPortal, the Gene Ontology, OBO Foundry, and DOLCE is the bigger opportunity that the substrate exists to enable.

## Quick start

```bash
git clone https://github.com/i-am-logger/pr4xis
cd pr4xis
cargo test --workspace
cargo run -p pr4xis-cli         # local CLI chatbot
```

Or just open **[pr4xis.dev](https://pr4xis.dev)** — no install required.

## Minimal code example

```rust
use pr4xis_domains::social::games::chess::{new_game, ChessAction, Square};

fn main() {
    let game = new_game();
    let game = game
        .next(ChessAction::new(Square::new(4, 1), Square::new(4, 3)))  // e2-e4
        .unwrap();

    // An illegal move is BLOCKED — failing precondition is named, engine recoverable.
    let illegal = ChessAction::new(Square::new(0, 0), Square::new(7, 7));
    assert!(game.next(illegal).is_err());  // axiom violation, not a wrong answer
}
```

Verified by `cargo test -p pr4xis-domains test_engine_illegal_move_blocked`. The same `Engine` pattern works for traffic, elevators, HTTP state machines, judicial workflows, sensor fusion, orbital mechanics, and every other domain in the workspace.

## What pr4xis is NOT

- **Not a knowledge graph database.** Knowledge graphs store facts; pr4xis proves theorems.
- **Not a theorem prover for pure math.** Coq, Lean, Agda do that. pr4xis is a substrate for *applied* domain knowledge.
- **Not a magic ontology generator.** Humans still author ontologies; the substrate verifies them.
- **Not Heim's physics-of-everything.** The mathematical lineage is acknowledged; the metaphysics is not.

(pr4xis *is* a new way to do AI. It is the alternative for tasks where accuracy and verifiability matter — not a complement.)

## Where this matters

- **Safety-critical engineering** — aerospace navigation, sensor fusion, biomedical decision support, industrial process control. pr4xis already includes ontologies for orbital mechanics, attitude estimation, multi-target tracking, Kalman filtering, AHRS, SLAM, and more.
- **LLM verification** — pr4xis as a deterministic checker behind a generative front end. The LLM produces text; pr4xis verifies which claims hold against the loaded ontologies.
- **Long-lived knowledge bases** — personal research notes, organizational SOPs, academic literature. The substrate makes a knowledge base machine-checkable as it grows.

## Contributing

- **Try the demo** at [pr4xis.dev](https://pr4xis.dev) and [file issues](https://github.com/i-am-logger/pr4xis/issues) for what breaks.
- **Contribute an ontology** if you work in a domain that could be encoded categorically. Existing ontologies under `crates/domains/src/` are the working examples.
- **Partner on a safety-critical deployment** in aerospace, biomedical, industrial, or legal.

## Documentation

The docs tree follows a [Diátaxis](https://diataxis.fr/)-inspired layout.

| Doc | What it covers |
|---|---|
| [Architecture](docs/understand/architecture.md) | The five-layer Rust stack |
| [Concepts](docs/understand/concepts.md) | Categories, functors, adjunctions, gap detection |
| [Foundations](docs/understand/foundations.md) | Academic lineage from Spencer-Brown to applied CT |
| [Gap detection](docs/research/gap-detection.md) | The bioelectricity Kv discovery, full numbers |
| [Domain catalog](docs/reference/domain-catalog.md) | Current organization of the 106 ontologies |
| [Paper outline](docs/research/paper-outline.md) | Draft architecture paper |

`docs/why/`, `docs/learn/`, `docs/use/` are scaffolded but unpopulated — track progress in [issue #55](https://github.com/i-am-logger/pr4xis/issues/55).

## License

CC BY-NC-SA 4.0 — see [LICENSE](LICENSE).

---

- **Repo:** [github.com/i-am-logger/pr4xis](https://github.com/i-am-logger/pr4xis)
- **Verification:** every numerical claim is paired with its `cargo test` / `find` / `grep` command. Find one without a verification path? That's a doc bug — file an issue.
- **Document date:** 2026-04-14
