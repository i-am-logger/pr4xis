<p align="center">
  <img src="docs/praxis-logo-light.jpg" alt="pr4xis" width="300"/>
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white" alt="Rust"/></a>
  <a href="https://nixos.org/"><img src="https://img.shields.io/badge/built_with-nix-5277C3?logo=nixos&logoColor=white" alt="Built with Nix"/></a>
  <a href="https://creativecommons.org/licenses/by-nc-sa/4.0/"><img src="https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg" alt="License"/></a>
</p>

# pr4xis — reasoning you can check

pr4xis reasons by deriving claims from explicit axioms, and every claim it makes carries a proof path back to those axioms. Where a language model predicts the next token, pr4xis derives the next claim — the way a proof follows from its premises.

The name is the third of Aristotle's kinds of knowing: *episteme* (knowing how things are), *techne* (knowing how to make things), and *praxis* — the doing itself, done well.

## What it is

pr4xis represents knowledge as ontologies — typed concepts and the proven relationships between them — and uses category theory as the connective tissue between domains. Because every step is a checked derivation, a conclusion can always be traced back to the axioms it rests on, and when something doesn't hold the engine names the axiom that failed.

The mathematical lineage runs from Spencer-Brown's *Laws of Form* through Heim's syntrometric logic to contemporary applied category theory ([Foundations](docs/understand/foundations.md)). Every connection in that chain is checked at test time rather than asserted.

## Demo

Try it now: **[pr4xis.dev](https://pr4xis.dev)** — runs entirely in the browser. No server, no GPU, no API key. If a query breaks, [file an issue](https://github.com/i-am-logger/pr4xis/issues) — broken queries are bug reports, not user error.

## What's here today

- **More than 160 domain ontologies** in the workspace — from orbital mechanics, attitude estimation, and Kalman filtering through dialectics and possible-worlds semantics.
- **A live demo** at **[pr4xis.dev](https://pr4xis.dev)** — it runs entirely in the browser; no server, GPU, or API key.
- **A concrete, re-runnable result** — a gap-detection finding in bioelectricity you can verify yourself ([gap detection](docs/research/gap-detection.md)).
- **Verifiable archives.** pr4xis packs what it has loaded into a small, self-contained `.prx` file and reads it back in a moment — instead of re-reading the whole original source each time — checking the archive's fingerprint first and refusing anything that's been altered. This fast, checked read-back works today for the English dictionary (WordNet) and for U.S. Code text; any `.prx` can still rebuild its original source byte-for-byte.
- **A full audit trail** — every conclusion carries the proof path back to its axioms.

## pr4xis and language models

They complement each other: a language model is fluent and broad, pr4xis is precise and checkable. A natural pairing is the model out front and pr4xis behind it, checking which claims actually hold.

|   | Language models | pr4xis |
|---|---|---|
| **How it knows** | Learned from training data | Derived from explicit axioms |
| **When wrong** | Hard to localize | Names the axiom that failed |
| **Traceability** | Opaque weights | Full proof path to the axioms |
| **Determinism** | Varies with seed and temperature | Same input, same derivation |
| **Missing knowledge** | Hard to surface | Gaps are detected |

## Where it helps

- **Safety-critical and regulated work**, where a conclusion needs to be checked rather than merely plausible — aerospace, biomedical decision support, industrial control.
- **Verification behind a language model** — the model writes, pr4xis checks which claims hold.
- **Long-lived knowledge** — research notes, SOPs, literature kept checkable as they grow.

## Get started

Install, run the CLI, and write your first interaction with the engine: **[docs/learn/get-started.md](docs/learn/get-started.md)**.

## Contributing

pr4xis is built in the open, and early contributors are welcome.

- **Try the demo** and [file issues](https://github.com/i-am-logger/pr4xis/issues) for anything that breaks — a broken query is a bug report, not user error.
- **Contribute an ontology** for a domain you know; the ontologies under `crates/domains/src/` are working examples.
- **Partner on a safety-critical deployment** in aerospace, biomedical, industrial, or legal.

## Documentation

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
| [Architecture](docs/understand/architecture.md) | The Rust stack, the engine, how everything fits together |
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
| [Domain catalog](docs/reference/domain-catalog.md) | The domain ontologies in the workspace and how they are organized |
| [Gap detection](docs/research/gap-detection.md) | The bioelectricity finding — a concrete result you can verify |
| [Novelty](docs/research/novelty.md) | What is new, what is prior art, what is pending verification |

## License

CC BY-NC-SA 4.0 — see [LICENSE](LICENSE).

---

- **Repo:** [github.com/i-am-logger/pr4xis](https://github.com/i-am-logger/pr4xis)
