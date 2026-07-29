<p align="center">
  <img src="docs/praxis-logo-light.jpg" alt="pr4xis" width="300"/>
</p>

<p align="center">
  <a href="https://crates.io/crates/pr4xis"><img src="https://img.shields.io/crates/v/pr4xis.svg" alt="crates.io"/></a>
  <a href="https://docs.rs/pr4xis-domains"><img src="https://docs.rs/pr4xis-domains/badge.svg" alt="docs.rs"/></a>
  <a href="https://doi.org/10.5281/zenodo.20755387"><img src="https://zenodo.org/badge/DOI/10.5281/zenodo.20755387.svg" alt="DOI"/></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white" alt="Rust"/></a>
  <a href="https://nixos.org/"><img src="https://img.shields.io/badge/built_with-nix-5277C3?logo=nixos&logoColor=white" alt="Built with Nix"/></a>
  <a href="https://creativecommons.org/licenses/by-nc-sa/4.0/"><img src="https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg" alt="License"/></a>
</p>

<p align="center">
  <a href="https://github.com/i-am-logger/pr4xis/actions/workflows/ci.yml"><img src="https://github.com/i-am-logger/pr4xis/actions/workflows/ci.yml/badge.svg?branch=master" alt="CI"/></a>
  <a href="https://codecov.io/gh/i-am-logger/pr4xis"><img src="https://codecov.io/gh/i-am-logger/pr4xis/branch/master/graph/badge.svg" alt="Coverage"/></a>
  <a href="https://scorecard.dev/viewer/?uri=github.com/i-am-logger/pr4xis"><img src="https://api.securityscorecards.dev/projects/github.com/i-am-logger/pr4xis/badge" alt="OpenSSF Scorecard"/></a>
  <a href="https://pr4xis.dev"><img src="https://img.shields.io/badge/demo-pr4xis.dev-blue" alt="Live Demo"/></a>
</p>

# pr4xis — Axiomatic Intelligence

**pr4xis is a new kind of AI: axiomatic, not statistical.** Where LLMs predict the next token from training data, pr4xis derives the next claim from accepted axioms — the same way mathematicians prove theorems.

Aristotle named three kinds of knowing:

- **episteme** — knowing how things are
- **techne** — knowing how to make things
- **praxis** — *the doing itself, done well*

pr4xis is the doing.

## Demo

Try it now: **[pr4xis.dev](https://pr4xis.dev)** — runs entirely in the browser. No server, no GPU, no API key. If a query breaks, [file an issue](https://github.com/i-am-logger/pr4xis/issues) — broken queries are bug reports, not user error.

## Foundation

The mathematical foundation runs from G. Spencer-Brown's *Laws of Form* (1969) through Heim's syntrometric logic to contemporary applied category theory — see [Foundations](docs/understand/foundations.md) for the academic lineage. Every step in that chain is **verified at test time**, not asserted:

```
cargo test -p pr4xis-domains -- syntrometry
```

runs the whole suite — the primary `Syntrometry → Pr4xisSubstrate` functor (14 of 18 concepts round-trip as fixed points; four intentional collapses whose richer semantics lives in the dedicated Dialectics and Kripke ontologies), the `Distinction → Syntrometry` embedding (Spencer-Brown → Heim), and cross-functors into `MetaOntology`, `Staging` (Futamura), `Algebra` (Goguen/Zimmermann), `Dialectics` (Hegel/Aristotle/Marx/Adorno/Priest), `Kripke` (possible-worlds semantics), and `C1` (Dehaene GWT).

## The problem

- **LLMs hallucinate by design.** Next-token prediction has no ground truth. When wrong, they cannot tell you which axiom failed because there are no axioms. For creative writing, this is fine. For domains where it kills people, it is unworkable.
- **Scientific knowledge is siloed.** WordNet, BioPortal, the Gene Ontology, DOLCE, OBO Foundry — rich, well-curated, almost entirely unable to be combined and trusted. Decades of expert curation, no executable substrate to compose them.

pr4xis solves both. It runs on formal scientific knowledge humans have already accumulated and on the 182 domain ontologies built directly in the workspace (`find crates/domains/src -name ontology.rs | wc -l`), with mathematical proof that every connection is sound. **Many more ontologies are still to be added** — the substrate exists precisely so that integration with BioPortal, the Gene Ontology, OBO Foundry, and the rest can be machine-checkable instead of merely hopeful.

## Where this matters

- **Safety-critical engineering** — aerospace navigation, sensor fusion, biomedical decision support, industrial process control. pr4xis already includes the foundational ontologies for orbital mechanics, attitude estimation, multi-target tracking, Kalman filtering, AHRS, SLAM, and more.
- **LLM verification** — pr4xis as a deterministic checker behind a generative front end. The LLM produces text; pr4xis verifies which claims actually hold.
- **Long-lived knowledge bases** — personal research notes, organizational SOPs, academic literature. The substrate keeps a knowledge base machine-checkable as it grows.

## pr4xis vs LLMs

|   | LLMs | pr4xis |
|---|---|---|
| **How it knows** | Learned from training data | Derived from accepted axioms |
| **Correctness** | Approximate — best guess from training patterns | Proven — every claim verified by math |
| **Hallucination** | Inherent — no ground truth | Impossible — every claim traces to a proof |
| **Determinism** | Stochastic — depends on temperature and seed | Absolute — same input, same proof, every time |
| **Traceability** | Opaque — billions of weights, no audit trail | Full proof path from conclusion back to its axioms |
| **When wrong** | Confidently wrong, hard to find why | The failing axiom is named |
| **Cross-domain reasoning** | Implicit blending, no guarantees | Proven connections between domains |
| **Undo / redo / branch** | None — each completion is final | Built in: undo, redo, branch from any prior state |
| **Missing knowledge** | Doesn't know what it doesn't know | Detects gaps automatically |

## The guarantees

pr4xis holds properties about its own reasoning. They are not promises to a user — they are invariants it cannot violate and remain itself. Five are **answer-guarantees** (properties of a single answer); **Extensible** is second-order (those five are preserved under composition):

- **Verifiable** — every claim carries its source; nothing is asserted without a citation.
- **Deterministic** — same input, same output, byte for byte, every run.
- **Explainable** — the system describes its own structure; the reasoning path is the answer.
- **Honest** — what it cannot ground, it leaves ungrounded rather than confabulate.
- **Consistent** — the axiom base derives no contradiction; it cannot prove a thing and its negation.
- **Extensible** *(composition)* — new ontologies compose by law-checked functor; the five answer-guarantees still hold.

Honest is the keystone of the answer-guarantees: the others are credible only because the system can stop. And these are not slogans — **every one of the 7,226 tests in `pr4xis-domains` declares, via `#[pr4xis::praxis_value(..)]`, which guarantee it witnesses**, a completeness gate enforces that no test escapes classification, and each guarantee is additionally backed by a machine-checkable axiom or universal property (a consistency check, a totality fuzz, a proof-as-explanation check) — not merely a count of tests. One command re-derives the whole partition:

```
cargo test -p pr4xis-domains --lib -- constitution_coverage --nocapture
```

| Guarantee | Tests | of which ∀-properties |
|---|---:|---:|
| Verifiable | 5,163 (71.5%) | 957 |
| Deterministic | 923 (12.8%) | 277 |
| Honest | 594 (8.2%) | 101 |
| Extensible | 400 (5.5%) | 41 |
| Explainable | 145 (2.0%) | 95 |
| Consistent | 1 (0.0%) | 0 |

The per-test declarations are the hard guarantee — `scripts/constitution-gate.sh` fails if any test is untagged or any tag is a typo. The *percentages* are a directional diagnostic: they show that Honest concentrates in operational/adversarial-input code and is thin in pure-knowledge domains, and that Explainable is rare everywhere — real gaps, surfaced by the suite classifying itself. Full account — what enforces each guarantee, what would violate it, why a statistical model cannot make the same promises, and how the classification works — in [The Constitution](docs/understand/constitution.md).

## Get started

Install, run the CLI, and write your first interaction with the engine: **[docs/learn/get-started.md](docs/learn/get-started.md)**.

## Contributing

- **Try the demo** at [pr4xis.dev](https://pr4xis.dev) and [file issues](https://github.com/i-am-logger/pr4xis/issues) for what breaks.
- **Contribute an ontology** if you work in a domain that could be encoded as one. Existing ontologies under `crates/domains/src/` are the working examples.
- **Partner on a safety-critical deployment** in aerospace, biomedical, industrial, or legal.

## Documentation

Also browsable as a site at **[pr4xis.dev/docs](https://pr4xis.dev/docs/)** — the links below go to the same content on GitHub.

**For a specific audience:**

| Doc | Audience |
|---|---|
| [for engineers](docs/why/for-engineers.md) | What pr4xis does for your stack, how it composes, what to do first |
| [for researchers](docs/why/for-researchers.md) | The novelty claim, the academic lineage, the open research directions |

**To get started:**

| Doc | What it covers |
|---|---|
| [Get started](docs/learn/get-started.md) | Three-step tutorial: install → first query → first ontology |

**To go deeper:**

| Doc | What it covers |
|---|---|
| [The Constitution](docs/understand/constitution.md) | The five guarantees pr4xis holds about its own reasoning, each with the test that enforces it |
| [Architecture](docs/understand/architecture.md) | The five-layer Rust stack, the engine, how everything fits together |
| [Concepts](docs/understand/concepts.md) | Categories, functors, adjunctions, gap detection — explained for engineers |
| [Evolution](docs/understand/evolution.md) | How ontologies grow without breaking — transform via functor, never rewrite |
| [Foundations](docs/understand/foundations.md) | Academic lineage from Spencer-Brown to applied category theory |

**To contribute:**

| Doc | What it covers |
|---|---|
| [Build an ontology from a paper](docs/use/build-ontology-from-paper.md) | The contributor authoring workflow, end to end |
| [Compose via functor](docs/use/compose-via-functor.md) | How to write a verified cross-domain functor |
| [Write axioms](docs/use/write-axioms.md) | How to write a domain axiom the engine enforces |

**Reference and research:**

| Doc | What it covers |
|---|---|
| [Glossary](docs/reference/glossary.md) | Every pr4xis term, in plain English |
| [Domain catalog](docs/reference/domain-catalog.md) | The 182 ontologies in the workspace and how they are organized |
| [Gap detection](docs/research/gap-detection.md) | The bioelectricity Kv discovery — a concrete result you can verify |
| [Novelty](docs/research/novelty.md) | What is new about pr4xis, what is prior art, what is pending verification |
| [Draft papers](docs/research/papers/) | Three drafts: categorical bioelectricity, adjunction-based gap detection, and the ontology-diagnostics meta-ontology |
| [Paper outline](docs/research/paper-outline.md) | Draft architecture paper |

## License

CC BY-NC-SA 4.0 — see [LICENSE](LICENSE).

---

- **Repo:** [github.com/i-am-logger/pr4xis](https://github.com/i-am-logger/pr4xis)
- **Document date:** 2026-07-11
