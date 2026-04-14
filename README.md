<p align="center">
  <img src="docs/praxis-logo-light.jpg" alt="pr4xis" width="300"/>
</p>

<p align="center">
  <a href="https://nixos.org/"><img src="https://img.shields.io/badge/built_with-nix-5277C3?logo=nixos&logoColor=white" alt="Built with Nix"/></a>
  <a href="https://github.com/i-am-logger/pr4xis/actions/workflows/ci.yml"><img src="https://github.com/i-am-logger/pr4xis/actions/workflows/ci.yml/badge.svg?branch=master" alt="CI"/></a>
  <a href="https://codecov.io/gh/i-am-logger/praxis"><img src="https://codecov.io/gh/i-am-logger/praxis/branch/master/graph/badge.svg" alt="Coverage"/></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white" alt="Rust"/></a>
  <a href="https://creativecommons.org/licenses/by-nc-sa/4.0/"><img src="https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg" alt="License"/></a>
  <a href="https://pr4xis.dev"><img src="https://img.shields.io/badge/demo-pr4xis.dev-blue" alt="Live Demo"/></a>
</p>

# pr4xis — Axiomatic Intelligence

**pr4xis is a new kind of AI: axiomatic, not statistical.** Where LLMs predict the next token from training data, pr4xis derives the next claim from accepted axioms — the same way mathematicians prove theorems.

Aristotle's three kinds of knowledge are:

- **episteme** — knowing how things are
- **techne** — knowing how to make things
- **praxis** — *the doing itself, done well*

pr4xis is the doing.

## The problem

- **LLMs hallucinate by design.** Next-token prediction has no ground truth. When wrong, they cannot tell you which axiom failed because there are no axioms. For creative writing, this is fine. For domains where it kills people, it is unworkable.
- **Scientific knowledge is siloed.** WordNet, BioPortal, the Gene Ontology, DOLCE, OBO Foundry — rich, well-curated, almost entirely unable to be combined and trusted. Decades of expert curation, no executable substrate to compose them.

pr4xis solves both. It runs on formal scientific knowledge humans have already accumulated and on the 106 domain ontologies built directly in the workspace, with mathematical proof that every connection is sound. **Many more ontologies are still to be added** — the substrate exists precisely so that integration with BioPortal, the Gene Ontology, OBO Foundry, and the rest can be machine-checkable instead of merely hopeful.

## Where this matters

- **Safety-critical engineering** — aerospace navigation, sensor fusion, biomedical decision support, industrial process control. pr4xis already includes the foundational ontologies for orbital mechanics, attitude estimation, multi-target tracking, Kalman filtering, AHRS, SLAM, and more.
- **LLM verification** — pr4xis as a deterministic checker behind a generative front end. The LLM produces text; pr4xis verifies which claims actually hold.
- **Long-lived knowledge bases** — personal research notes, organizational SOPs, academic literature. The substrate keeps a knowledge base machine-checkable as it grows.

## pr4xis vs LLMs

|   | LLMs | pr4xis |
|---|---|---|
| **How it knows** | Learned from training data | Derived from accepted axioms |
| **Correctness** | Probable | Provable |
| **Hallucination** | Inherent — no ground truth | Impossible — every claim traces to a proof |
| **Determinism** | Stochastic — depends on temperature and seed | Absolute — same input, same proof, every time |
| **Traceability** | Opaque — billions of weights, no audit trail | Full proof path from conclusion back to its axioms |
| **When wrong** | Confidently wrong, hard to find why | The failing axiom is named |
| **Cross-domain reasoning** | Implicit blending, no guarantees | Proven connections between domains |
| **Undo / redo / branch** | None — each completion is final | Built in: undo, redo, branch from any prior state |
| **Missing knowledge** | Doesn't know what it doesn't know | Detects gaps automatically |
| **Compute footprint** | GPU clusters, terabytes of weights | Single core, one Rust binary, megabytes |

## Demo

Try it now: **[pr4xis.dev](https://pr4xis.dev)** — runs entirely in the browser. No server, no GPU, no API key. If a query breaks, [file an issue](https://github.com/i-am-logger/pr4xis/issues) — broken queries are bug reports, not user error.

## Get started

Install, run the CLI, and write your first interaction with the engine: **[docs/learn/get-started.md](docs/learn/get-started.md)**.

## Contributing

- **Try the demo** at [pr4xis.dev](https://pr4xis.dev) and [file issues](https://github.com/i-am-logger/pr4xis/issues) for what breaks.
- **Contribute an ontology** if you work in a domain that could be encoded as one. Existing ontologies under `crates/domains/src/` are the working examples.
- **Partner on a safety-critical deployment** in aerospace, biomedical, industrial, or legal.

## Documentation

| Doc | What it covers |
|---|---|
| [Get started](docs/learn/get-started.md) | Install, run the CLI, write your first engine interaction |
| [Architecture](docs/understand/architecture.md) | How pr4xis works under the hood — the five-layer stack, the engine, the verification commands |
| [Concepts](docs/understand/concepts.md) | Categories, functors, adjunctions, gap detection — explained for engineers |
| [Foundations](docs/understand/foundations.md) | Academic lineage from Spencer-Brown to applied category theory |
| [Gap detection](docs/research/gap-detection.md) | A concrete result — pr4xis automatically detected a missing distinction in molecular biology that experts had collapsed into one entity |
| [Domain catalog](docs/reference/domain-catalog.md) | The 106 ontologies in the workspace and how they are organized |
| [Paper outline](docs/research/paper-outline.md) | Draft architecture paper |

## License

CC BY-NC-SA 4.0 — see [LICENSE](LICENSE).

---

- **Repo:** [github.com/i-am-logger/pr4xis](https://github.com/i-am-logger/pr4xis)
- **Document date:** 2026-04-14
