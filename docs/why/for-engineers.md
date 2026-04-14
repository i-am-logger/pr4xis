# pr4xis for Engineers

You are an engineer evaluating whether to put pr4xis in your stack. This page is for you.

## What it is

pr4xis is **the domain engine** for systems that need to be correct, not just plausible. It is a different kind of AI — it derives its answers from formally proven axioms instead of statistical patterns — and it runs as a Rust workspace you embed directly in your code. There is no service, no GPU, no API key, no model weights. The engine carries the knowledge, the engine carries the intelligence, and the engine *runs* — it doesn't just check what something else produced.

Concretely:

- **A first-class state engine.** You define situations, actions, and preconditions; the engine validates every transition against its loaded ontologies and either applies it or names the exact rule that blocked it. No silent approximation, no probabilistic answer.
- **An axiomatic reasoner.** When pr4xis tells you that a fact in one domain implies a claim in another, the implication is a chain of categorical morphisms back to a published source. You can ask which axioms participated; you can reproduce the derivation deterministically.
- **A composable knowledge substrate.** pr4xis loads 106 ontologies today (biomedical, sensor fusion, navigation, perception, tracking, space, underwater, industrial, linguistics, formal mathematics, music, colors, judicial workflow), composed through 61 verified cross-domain functors. More ontologies are still to be added — that's the bigger opportunity, and the substrate exists precisely so the additions can be machine-checkable.

The contrast with statistical AI: where an LLM predicts the next token, pr4xis derives the next claim. Where an LLM hallucinates as a structural feature, pr4xis cannot — every claim it makes traces to a proof. Where an LLM is opaque, pr4xis names the failing axiom when something doesn't hold. See the [README](../../README.md) for the side-by-side comparison.

## What you get out of the box today

| Capability | Where it lives |
|---|---|
| 5-layer stack (logic → category → ontology → engine → codegen) | [Architecture](../understand/architecture.md) |
| 106 ontologies covering biomedical, sensor fusion, navigation, perception, tracking, space, underwater, industrial, linguistics, formal math | `crates/domains/src/` |
| 61 cross-domain functor proofs (verified by `check_functor_laws()`) | `grep -rn "impl Functor" crates/domains/src/ crates/pr4xis/src/` |
| Adjunction-based gap detection (the bioelectricity Kv discovery) | [Gap detection](../research/gap-detection.md) |
| Engine with `back()`, `forward()`, branching, and full trace | `crates/pr4xis/src/engine/` |
| WASM browser surface — runs entirely in-browser, no server | [pr4xis.dev](https://pr4xis.dev) |
| Property-based testing throughout | [Glossary: property-based testing](../reference/glossary.md#property-based-testing) |

## How it composes with what you already have

pr4xis is **a Rust library, not a service**. There is no daemon, no API key, no GPU requirement. You add `pr4xis-domains` (or just the parts of it you need) as a Cargo dependency and call the engine directly. The chat surface is a thin wrapper; the WASM demo is one binary that's <1 MB plus the loaded ontology data.

The composition patterns:

1. **As the runtime decision engine inside your service.** Your code calls `engine.next(action)` whenever a state transition happens. The engine validates against preconditions and either returns the new state or returns a named violation. The engine *is* the decision; it doesn't checkpoint someone else's decision.
2. **As a domain rules engine for state machines.** HTTP connection state, judicial case lifecycle, traffic signal control, elevator dispatch — pr4xis already has these as worked examples. The pattern generalizes to anything where "what's allowed next" depends on rich context.
3. **Alongside an LLM, where the LLM produces text and pr4xis produces the answers that have to be right.** This is not "pr4xis checks the LLM" — it's "pr4xis is the part of the stack that knows things, and the LLM is the part that talks". They are different functions, both first-class. Use each for what it's good at.

## What you should do first

1. Run the test suite locally:
   ```bash
   git clone https://github.com/i-am-logger/pr4xis
   cd pr4xis
   cargo test --workspace
   ```
2. Open one of the existing ontologies (`crates/domains/src/social/judicial/ontology.rs` is a good starter — lifecycle of a legal case modeled as a kinded relation graph). Notice that it's a single `define_ontology!` block.
3. Run the WASM demo: [pr4xis.dev](https://pr4xis.dev). Try a few queries. File issues on the ones that break.
4. If your domain isn't covered: [build your own ontology from a paper](../use/build-ontology-from-paper.md).

## What it is NOT

- **Not a knowledge graph database.** Knowledge graphs store facts; pr4xis proves theorems against axioms. The reasoning systems verify category laws, not just SPARQL queries.
- **Not a theorem prover for pure math.** Coq, Lean, and Agda do that, and they do it well. pr4xis is the engine for *applied* domain knowledge.
- **Not a magic ontology generator.** Humans still author ontologies, with assistive tooling planned. pr4xis runs them; it does not invent them.
- **Not a complement to LLMs.** It is the alternative AI for tasks where accuracy and verifiability matter. The composition with an LLM (point 3 above) is one valid pattern, not the project's identity.

## Where to go from here

- [Architecture](../understand/architecture.md) — how the five-layer stack works
- [Concepts](../understand/concepts.md) — categories, functors, adjunctions, in plain English
- [Get started](../learn/get-started.md) — install, run, write your first interaction
- [Build an ontology from a paper](../use/build-ontology-from-paper.md) — the contributor authoring guide
- [Glossary](../reference/glossary.md) — every term defined

---

- **Document date:** 2026-04-14
