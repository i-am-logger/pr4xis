<p align="center">
  <img src="docs/praxis-logo-light.jpg" alt="pr4xis" width="300"/>
</p>

<p align="center">
  <a href="https://creativecommons.org/licenses/by-nc-sa/4.0/"><img src="https://img.shields.io/badge/License-CC%20BY--NC--SA%204.0-lightgrey.svg" alt="License"/></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white" alt="Rust"/></a>
  <a href="https://pr4xis.dev"><img src="https://img.shields.io/badge/demo-pr4xis.dev-blue" alt="Live Demo"/></a>
</p>

# pr4xis — Axiomatic Intelligence

pr4xis is **axiomatic intelligence** — the inverse of statistical AI. Where LLMs predict the next token from training data, pr4xis derives the next claim from a [category-theoretic](https://en.wikipedia.org/wiki/Category_theory) ontology in which every transformation is a mathematically proven [functor](https://en.wikipedia.org/wiki/Functor). Every answer is traceable back to a published axiom; nothing is guessed; the same input always produces the same proof.

The substrate is the point. Scientific [ontologies](https://en.wikipedia.org/wiki/Ontology_(information_science)) already exist — [WordNet](https://wordnet.princeton.edu/) for language, [BioPortal](https://bioportal.bioontology.org/) for biomedicine, the [Gene Ontology](https://geneontology.org/), [DOLCE](https://www.loa.istc.cnr.it/dolce/overview.html) as an upper layer, hundreds more. They have been accumulating for decades. What does not exist is a way to **compose** them with proof: to take two ontologies built by two different research communities and verify that the structure of one maps faithfully into the other. pr4xis is that missing substrate. Ontologies are categories, the maps between them are functors, the functors are checked at compile and test time, and the verification holds across one or one hundred composed domains.

## Mathematical roots

pr4xis shares structural DNA with the modernized syntrometric logic tradition (Heim 1980, reformulated categorically in 2025): both treat domains as categories with structure-preserving functors between them, use Kripke-style aspect-relative semantics, ground part/whole reasoning in classical extensional mereology, and model self-reference as a natural transformation. To our knowledge, pr4xis is the first executable, test-verified instance of this categorical reading. It does not adopt Heim's physical-metaphysical claims.

The name comes from Aristotle's three kinds of knowledge: **episteme** (knowing how things are), **techne** (knowing how to make things), and **praxis** — the doing itself, done well. pr4xis is the doing: a system that *does* the right thing, and can *prove* it.

## The problem

Two failures in current AI infrastructure point at the same gap.

**LLMs hallucinate by design.** Next-token prediction has no ground truth and no proof object. When an LLM gives you the right answer, it is right by accident of training data; when it gives you the wrong answer, you have no way to know which axiom failed because there are no axioms. The hallucination is not a bug to be patched — it is the inevitable behavior of a system that learns patterns instead of deriving consequences. For domains where "probably correct" is acceptable (creative writing, summarization, brainstorming) this is fine. For domains where it kills people, it is unworkable.

**Scientific ontologies do not compose.** Hundreds of formal ontologies exist across science — WordNet, BioPortal, the Gene Ontology, DOLCE, the dozens of OBO Foundry ontologies, Cyc, SUMO. They are rich, well-curated, and almost entirely siloed. There is no executable substrate that lets you take WordNet's lexical taxonomy, compose it with a biomedical mereology, and verify that the composition preserves both structures. Building such a substrate has been a research goal for sixty years and an engineering reality for none of them.

pr4xis fills the second gap, which lets it solve the first.

## What pr4xis is

A five-layer stack written in Rust:

1. **logic** — propositional logic, the connectives, deduction, induction, abduction
2. **category** — entities, relationships, categories, morphisms, functors, natural transformations, with category laws verified at test time
3. **ontology** — categories of concepts plus reasoning systems for taxonomy, mereology, causation, opposition, and a trait that ties them together
4. **engine** — runtime enforcement: situations, actions, preconditions, traces, undo and redo
5. **codegen** — build-time generation of ontologies from declarative source

Every domain is an ontology; every ontology is a category; every cross-domain claim is a functor whose laws are verified by tests. The contrast with statistical AI is not philosophical — it is a row-by-row substitution:

### pr4xis vs LLMs

|   | LLMs | pr4xis |
|---|---|---|
| **How it knows** | Learned from training data | Derived from axioms |
| **Correctness** | Probable — next-token prediction | Provable — category laws + axioms verified at test time |
| **Hallucination** | Inherent — no ground truth | Impossible at the substrate level; every claim returned by the engine traces to a proof |
| **Determinism** | Stochastic — depends on temperature, sampling, seed | Absolute — same input, same proof, every time |
| **Traceability** | Opaque — billions of weights, no audit trail | Full proof path from conclusion back to its axioms, with citations |
| **Cross-domain reasoning** | Implicit blending; no guarantees | Functors mathematically *prove* that two domains share structure |
| **When wrong** | Confidently wrong, hard to find why | The axiom that failed is named, with the trace that triggered it |
| **Undo / redo / branch** | None — each completion is final and irreversible | Built into the engine: `back()` and `forward()` walk the history; a new action after `back()` branches a new timeline. Verified by `cargo test -p pr4xis test_back_forward_roundtrip` and `test_next_after_back_clears_future` |
| **Missing knowledge** | Doesn't know what it doesn't know | [Adjunctions](https://en.wikipedia.org/wiki/Adjoint_functors) detect ontology gaps automatically |
| **Compute footprint** | GPU clusters; terabytes of weights; minutes to load | Single core; one Rust binary; tens of megabytes for the substrate, more for ontology data such as WordNet |
| **Verifiable proofs in the codebase today** | 0 | 4,855 (see "What's in the box" below for the exact verification command) |

## What's in the box today

Every number in this section is paired with the exact command that re-derives it. There is no marketing math; if a number does not have a verification command next to it, it does not belong in this document.

| Claim | Value | Verify with |
|---|---|---|
| Machine-verified tests in the workspace | **4,855** | `cargo test --workspace` |
| Ontologies | **106** | `find crates/domains/src -name ontology.rs \| wc -l` |
| Functor implementations (cross-domain proofs) | **61** | `grep -rn "impl Functor" crates/domains/src/ crates/pr4xis/src/ \| wc -l` |
| Lines of Rust source | **136,296** | `find crates -name "*.rs" -not -path "*/target/*" \| xargs wc -l \| tail -1` |
| Crates in the workspace | **8** | `ls crates/` → `chat`, `cli`, `domains`, `examples`, `pr4xis` (core), `pr4xis-derive` (proc macros), `wasm`, `web` |
| Architecture layers | **5** | logic → category → ontology → engine → codegen — top-level modules under `crates/pr4xis/src/` |

The pr4xis workspace also runs as a live web demo at [pr4xis.dev](https://pr4xis.dev), entirely in the browser via WebAssembly — no server, no GPU, no API key. The demo loads the English ontology (WordNet) at startup and runs the full linguistics pipeline — tokenization, [Lambek pregroup parsing](https://en.wikipedia.org/wiki/Pregroup_grammar), [Montague semantics](https://en.wikipedia.org/wiki/Montague_grammar), taxonomy traversal, response generation — on a single browser thread.

It is a working surface, not a polished product. Some queries return responses that are technically correct but not yet rendered in human-readable form; some queries hit grammar gaps that the system cannot currently parse. These are tracked as open issues, not papered over.

The codebase uses [property-based testing](https://en.wikipedia.org/wiki/Software_testing#Property_testing) extensively: invariants are expressed as properties that must hold for all generated inputs, not just hand-picked examples.

### A concrete result: gap detection in biomedical ontologies

The substrate's most distinctive capability is **automated detection of missing distinctions** in scientific ontologies via categorical [adjunctions](https://en.wikipedia.org/wiki/Adjoint_functors). We have run this analysis end-to-end on the biomedical stack and the numbers are concrete, current, and re-derivable from a single command.

```
cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture
```

The command produces this output (abridged):

| Adjunction | Round-trip unit loss | What the loss means |
|---|---|---|
| Molecular ⊣ Bioelectric | **85.2%** | 85.2% of molecular entities lose their identity when round-tripped through the bioelectric layer — each collapsed entity is a missing distinction the math detected automatically |
| Biology ⊣ Bioelectric | **82.6%** | 82.6% of biological entities collapse the same way at the bioelectric scale |
| Pharmacology ⊣ Molecular | **68.0%** | 68.0% of pharmacological entities collapse at the molecular scale |
| End-to-end (acoustics → biophysics → molecular → bioelectricity) | **92.3%** | 26 acoustic concepts compress to **2** distinct bioelectric concepts across four domains — each domain is a lossy compression of the one below it, quantified categorically for what we believe is the first time |

The most striking single discovery: voltage-gated potassium channels (Kv) serve two functionally different roles — homeostatic and therapeutic — that the molecular ontology had collapsed into one entity. The adjunction surfaced this gap; a `ContextDef` resolution disambiguated the two roles; the gap closed. This specific case is verified by `cargo test -p pr4xis-domains test_kv_gap_is_resolved_by_context`.

## Quick start

Clone, build, and run the test suite:

```bash
git clone https://github.com/i-am-logger/pr4xis
cd pr4xis
cargo test --workspace
```

Try the chatbot CLI on the local English ontology:

```bash
cargo run -p pr4xis-cli
```

Or open the live web demo (no install required): **[pr4xis.dev](https://pr4xis.dev)**.

## A minimal example

The substrate's central pattern is **runtime axiom enforcement**: an `Engine` carries a situation and a list of preconditions, and any action that violates a precondition is *blocked, named, and recoverable* — never silently approximated.

```rust
use pr4xis_domains::social::games::chess::{new_game, ChessAction, Square};

fn main() {
    // A new game enforces the full rules of chess as preconditions.
    let game = new_game();

    // e2-e4: legal opening move, accepted.
    let game = game
        .next(ChessAction::new(Square::new(4, 1), Square::new(4, 3)))
        .unwrap();

    // An illegal move is BLOCKED — the failing precondition is named,
    // the engine is recoverable, and nothing is approximated away.
    let illegal = ChessAction::new(Square::new(0, 0), Square::new(7, 7));
    let result = game.next(illegal);
    assert!(result.is_err());  // axiom violation, not a wrong answer
}
```

This exact pattern is verified by `cargo test -p pr4xis-domains test_engine_illegal_move_blocked` (and many other engine tests in `crates/domains/src/social/games/chess/tests.rs`).

The same `Engine` pattern works for any domain: traffic signals, elevator dispatch, HTTP state machines, judicial case lifecycles, sensor fusion gates, orbital mechanics. A precondition that holds is a proof; a precondition that fails is a blocked action with a named cause.

## What pr4xis is NOT

Foreclosing the obvious hostile readings.

- **Not an LLM replacement.** pr4xis does not generate creative text, complete code, or carry on small talk. It answers questions whose answers are derivable from its loaded ontologies, and tells you when they are not.
- **Not a knowledge graph database.** Knowledge graphs store facts; pr4xis proves theorems. The reasoning systems verify that the stored structure satisfies category laws and the relevant axioms (no cycles in taxonomies, antisymmetric is-a relations, weak supplementation in mereologies, and so on).
- **Not a theorem prover for pure mathematics.** Coq, Lean, and Agda do this and do it well. pr4xis is a substrate for *applied* domain knowledge — an executable place to put scientific ontologies that already exist, with categorical machinery doing the cross-domain bookkeeping.
- **Not a magic ontology generator.** Humans still author ontologies, with assistive tooling planned. The substrate verifies the structure; it does not invent the content.
- **Not Heim's physics-of-everything.** The lineage section above acknowledges intellectual debt to the modernized syntrometric logic tradition. It does not endorse Heim's twelve-dimensional spacetime, particle mass formulas, or teleological cosmology.

## Where this matters

Three categories of use case map cleanly onto what the substrate can already do.

**Safety-critical engineering.** Aerospace navigation, sensor fusion, biomedical decision support, industrial process control, automotive functional safety. The common feature is that "probably correct" is unacceptable, an audit trail is mandatory, and the failing claim has to be named when something goes wrong. pr4xis already includes ontologies for orbital mechanics, attitude estimation, multi-target tracking, Kalman filtering, occupancy grids, AHRS and SLAM, and more — the foundations are in place.

**LLM verification.** The strongest LLM-era use of pr4xis is as a verification layer behind a generative front end. The LLM produces plausible text; pr4xis projects the claims it can recognize onto its loaded ontologies and reports which ones are provable, which contradict known axioms, and which are merely consistent. The factory floor, the clinical decision support tool, the legal contract reviewer — all of them benefit from a deterministic checker behind a probabilistic talker.

**Long-lived knowledge bases.** Personal research notes, organizational SOPs, and academic literature accumulate faster than they can be put to work. A categorical substrate lets a knowledge base grow without rotting: every addition either extends the existing structure (proven by functor laws) or surfaces a contradiction (proven by axiom violation). The substrate becomes a living, machine-checkable epistemology — for an individual researcher, a company, or a discipline.

## Why now

Three forces are converging.

The LLM era has created enormous demand for verification primitives that the AI category was missing. Every serious deployment of an LLM in a high-stakes setting is currently held together by hand-written guardrails. There is a market opening for a substrate whose guarantees are mathematical instead of statistical.

Applied category theory has finally produced executable libraries. Mac Lane's *Categories for the Working Mathematician* is fifty years old, but the Rust, Lean, and Agda ecosystems for working with categories programmatically are recent enough that pr4xis could not have been built with this clarity even five years ago. The substrate has been theoretically possible for decades and practically writeable for two.

Scientific ontologies are accumulating faster than they can be composed. BioPortal alone hosts more than a thousand biomedical ontologies. Each one represents real research, and almost none of them talk to each other. The substrate gap is the single largest reason that decades of careful ontological work has stayed academic.

## Contributing

Three concrete asks.

- **Try the live demo** at [pr4xis.dev](https://pr4xis.dev) and report what breaks. We treat broken queries as bug reports, not user error — file an issue with the exact input.
- **Contribute an ontology.** If you work in a domain whose structure could be encoded categorically, the codebase is open. The authoring workflow will be documented as the in-flight docs reorganization completes; in the meantime, the existing ontologies under `crates/domains/src/` are the working examples.
- **Partner on a real safety-critical use case.** We are looking for a first deep deployment in aerospace, biomedical, industrial, or legal — anywhere the substrate could carry weight that probabilistic AI cannot.

Open issues, particularly the in-progress docs reorg and per-ontology authoring guides, are tracked in [GitHub Issues](https://github.com/i-am-logger/pr4xis/issues).

## Documentation

| Document | What it covers |
|---|---|
| [Foundations](docs/foundations.md) | Academic lineage — every ontology traced to its source paper |
| [Architecture](docs/architecture.md) | The five-layer stack: logic, category, ontology, engine, codegen |
| [Concepts](docs/concepts.md) | What ontologies are and how they compose via functors |
| [Domains](docs/domain-crates.md) | Catalog of available domains |
| [Investor brief](docs/praxis_investment_brief.md) | The same content as this README, framed for investors |

Note: the docs tree is currently being reorganized. Some docs above contain stale claims that are being audited and updated. Track the reorg in [issue #55](https://github.com/i-am-logger/pr4xis/issues/55).

## License

CC BY-NC-SA 4.0 — see [LICENSE](LICENSE).

---

- **Repository:** [github.com/i-am-logger/pr4xis](https://github.com/i-am-logger/pr4xis)
- **Verification:** every numerical claim in this document is paired with the exact command that re-derives it. If you find a number without a verification path, that is a documentation bug — please file an issue.
- **Document date:** 2026-04-14. Numbers reflect the codebase at that date. They will drift; the verification commands will not.
