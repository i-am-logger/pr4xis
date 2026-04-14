# Glossary

Terms used across the pr4xis docs, in plain English. For deeper coverage of any of these, see [Concepts](../understand/concepts.md), [Architecture](../understand/architecture.md), or [Foundations](../understand/foundations.md).

## Axiom

A statement that is taken as given, without needing to be proven from anything else. The starting point of a chain of reasoning. In pr4xis, every axiom is either grounded in a published source (a textbook, a paper, a standard) or is a structural rule of the substrate itself (no cycles in taxonomies, etc.). When pr4xis says a claim is provable, it means the claim can be derived from the axioms by logical and categorical operations.

## Ontology

In pr4xis, an ontology is more than a list of facts. It is a **category** (in the formal mathematical sense) of concepts and the relationships between them, plus reasoning systems that interpret those relationships (taxonomy, mereology, causation, opposition), plus the axioms the structure must satisfy. Every domain in pr4xis — biology, chess, sensor fusion, traffic signals, judicial workflow — is an ontology in this stricter sense. Authored declaratively via the `define_ontology!` macro.

## Category

A mathematical structure with **objects**, **morphisms** (directed maps between objects), **composition** (combining two morphisms into a third), and **identity** (a morphism that does nothing). Two laws govern composition: associativity (the order of grouping doesn't matter) and identity (composing with identity changes nothing). pr4xis treats every domain as a category and verifies the laws at test time. See [Concepts](../understand/concepts.md) for the long version.

## Morphism

A directed map from one object to another inside a category. In pr4xis, a morphism is a `Relationship` between two `Entity` values — for example, in a taxonomy, a morphism `Dog → Mammal` represents "dog is a mammal".

## Functor

A **structure-preserving map between two categories**. If `F: A → B` is a functor, then every object `x` in category `A` has a corresponding object `F(x)` in category `B`, and every morphism `f: x → y` in `A` has a corresponding morphism `F(f): F(x) → F(y)` in `B`. Two laws hold: identities are preserved, and composition is preserved. In pr4xis, functors are how two ontologies are *proved* to share structure — when the functor laws hold, the source ontology faithfully embeds in the target. The current workspace has 61 functor implementations.

## Functor laws

The two conditions a functor must satisfy:

1. **Identity preservation**: `F(id_x) = id_{F(x)}` — the identity morphism in the source maps to the identity morphism in the target.
2. **Composition preservation**: `F(g ∘ f) = F(g) ∘ F(f)` — composing two morphisms before mapping gives the same result as mapping each then composing.

Verified by `cargo test -p pr4xis category::validate::check_functor_laws`. Every functor in pr4xis must pass.

## Adjunction

A pair of functors `F: A → B` and `G: B → A` that are "optimal inverses" of each other in a precise categorical sense (the unit `η: Id_A → G ∘ F` and counit `ε: F ∘ G → Id_B` natural transformations satisfy the triangle identities). In pr4xis, adjunctions are the mechanism for **gap detection**: when you take an object in `A`, apply `F` to get its image in `B`, then apply `G` to come back, you should get the original object. If you don't, the original ontology has a missing distinction the math just surfaced. The bioelectricity Kv discovery is the canonical example — see [Gap detection](../research/gap-detection.md).

## Reasoning system

In pr4xis, a category becomes an ontology when one or more reasoning systems are layered on top of it, each interpreting the morphisms in a specific way:

- **Taxonomy** — interprets some morphisms as `is-a`, with axioms `NoCycles` and `Antisymmetric`
- **Mereology** — interprets some morphisms as `part-of`, with axiom `WeakSupplementation`
- **Causation** — interprets some morphisms as `causes`, with axiom `NoSelfCausation`
- **Opposition** — interprets some morphism pairs as `opposes`, with axioms `Symmetric` and `Irreflexive`
- **Context** — disambiguates entities by context (`ContextDef::resolve`)
- **Analogy** — `Analogy<F>` is a wrapper around a functor `F`, treating the functor as a proven analogy between two domains

Each reasoning system is a Rust trait that an ontology can implement.

## Engine

The runtime layer of pr4xis. An `Engine<A>` carries a current `Situation` (immutable world state), a list of `Precondition`s (rules that must hold before any action), and a function that applies an `Action` to produce a new `Situation`. When you call `engine.next(action)`, the engine checks every precondition; if all pass, the action is applied and a trace entry is recorded; if any fail, an `EngineError::Violated` is returned with the failing precondition named, and the engine is recoverable for rollback. Supports `back()`, `forward()`, and branching.

## Situation

An **immutable snapshot of the world** at a single point in time. Every action produces a new situation; the old situation is preserved in the engine's history stack. This is what enables undo, redo, and branching without mutation.

## Action

A proposed change to the current situation. Actions are checked against the engine's preconditions before they are applied. An action that violates a precondition is blocked, named, and recoverable — never silently approximated.

## Precondition

A rule that must hold before an action can be applied. A precondition takes the current situation and the proposed action and returns either `Satisfied` (with a reason) or `Violated` (with the failing rule and a diagnostic). Both carry context, so traces are useful for debugging and auditing.

## Trace

The full structured history of every action the engine has processed. Each trace entry records the precondition results, the resulting situation (or the violation), and the timing. Used for audit, replay, and the `TracedPipeline` writer monad in chat-style applications.

## Quality

A property that inheres in an entity (DOLCE term). For example, in the colors ontology, an `Rgb` entity has a `Luminance` quality. Qualities are how pr4xis attaches measurable or comparable values to objects without making the objects themselves carry the values.

## Substrate

A word the README uses for "the engineering layer that makes ontologies composable with mathematical proof". Loosely: the parts of pr4xis that aren't specific to any one domain — the categorical machinery, the engine, the reasoning systems, the validators. The opposite of "the domains" themselves.

## DOLCE

A foundational ontology from Masolo et al. (2003) that classifies all of being into Endurants (physical objects, social objects, mental objects), Perdurants (events, processes), and Qualities. pr4xis uses DOLCE as the upper layer that every domain ontology classifies its concepts against, and there is a `PraxisToDolce` functor that proves the workspace's own type system maps faithfully into DOLCE.

## WordNet

An open lexical database of English (~107K concepts, decades of curation). pr4xis ingests WordNet via its `codegen::wordnet` build-time generator and exposes it as the English ontology. The WASM browser demo at [pr4xis.dev](https://pr4xis.dev) loads it at startup.

## define_ontology!

A Rust macro at `pr4xis::ontology::macros::define_ontology` that takes a declarative ontology specification and emits the full implementation: the category, every reasoning system the ontology uses, structural axioms, and the `OntologyMeta` used for trace attribution. The macro is the canonical way to author an ontology in pr4xis. Every file at `crates/domains/src/**/ontology.rs` is an instance.

## OntologyMeta

A struct generated by `define_ontology!` for every ontology, containing the ontology's name, version, source citations, and other metadata. Used by the engine to attribute trace entries to the ontologies that produced them.

## Categorical extensional mereology (CEM)

The classical formal theory of parts and wholes — Simons (1987), Stanford Encyclopedia of Philosophy on [Mereology](https://plato.stanford.edu/entries/mereology/). pr4xis's `MereologyDef` reasoning system implements CEM with the `WeakSupplementation` axiom (if a whole has a proper part, it has another disjoint part). Heim's modernized syntrometric logic also grounds part/whole reasoning in CEM, which is one of the structural alignments cited in [Foundations](../understand/foundations.md).

## Kripke semantics

A formal semantics for modal logic in which truth depends on which "possible world" you are evaluating in. pr4xis does not use full Kripke semantics today, but the modernized syntrometric logic tradition (Heim 1980, formalized 2025) does, and pr4xis's pattern of multiple ontologies viewing the same domain through different functors is the computational realization of an aspect-relative Kripke frame. See the foundations doc for the connection.

## Property-based testing

A testing technique in which invariants are expressed as properties that must hold for ALL inputs, and a library generates random inputs to look for counterexamples. pr4xis uses [proptest](https://github.com/proptest-rs/proptest) for property-based testing of category laws, axiom satisfaction, and domain invariants. See the [Wikipedia article on property testing](https://en.wikipedia.org/wiki/Software_testing#Property_testing) for the broader context.

## Codegen / async loading / mmap

Three different mechanisms pr4xis supports for delivering ontology data into the runtime, all proven equivalent as functors from the same `OntologyBuilder` source:

- **Codegen** (build-time): pre-compile declarative source into static Rust. Used by the WordNet ontology in the WASM demo.
- **Async loading** (runtime): load ontology data from a file or stream asynchronously. Used for ontologies that are too large to embed or that need hot reloading.
- **Memory-mapped files** (runtime, zero-copy): mmap a precomputed binary directly into memory.

The choice between them is operational, not semantic. See [Architecture](../understand/architecture.md) for the layer description.

---

- **Document date:** 2026-04-14
- **Verification:** every term that names a code element (`define_ontology!`, `Engine`, `ContextDef`, etc.) corresponds to actual code in `crates/pr4xis/src/` or `crates/domains/src/`. Grep to verify.
