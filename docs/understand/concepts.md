# Concepts

This document explains the conceptual model behind pr4xis — what an ontology *is* in this system, why category theory is the substrate, and how domains compose. For the layer structure and runtime mechanics, see [Architecture](architecture.md). For the academic lineage and source papers, see [Foundations](foundations.md).

## What is an ontology in pr4xis?

Most ontology systems treat an ontology as a **graph of facts** — a set of triples saying that A is-a B and B has-part C, queryable via SPARQL or a graph database. pr4xis treats an ontology as a **category** — a mathematical structure with objects, morphisms, composition, and identity, plus a set of reasoning systems built on top of that structure.

The difference matters because it determines what you can prove. A graph of facts can be queried; a category can be *composed* with other categories under proof, and the composition can be checked at compile time and test time. If two ontologies share structure, a categorical functor between them is a theorem about that shared structure — not a heuristic, not an alignment score, not a similarity measurement.

Every domain in `crates/domains/src/` is an ontology in this stricter sense: an enum of concepts (the objects), an enum of relations (the morphisms), a set of reasoning systems that interpret those relations (taxonomy, mereology, causation, opposition), a set of axioms that the system must satisfy (no cycles, antisymmetric is-a, weak supplementation, etc.), and metadata used by the engine for trace attribution. The `define_ontology!` macro emits all of this from a single declarative block.

## Categories

A category has:

- **Objects** — the things the category is about. In pr4xis, every object is an `Entity` (a finite, enumerable Rust enum variant).
- **Morphisms** — directed maps between objects. In pr4xis, every morphism is a `Relationship` between two entities.
- **Composition** — if `f: A → B` and `g: B → C`, then `g ∘ f: A → C` exists and is itself a morphism.
- **Identity** — for every object `A`, there is a morphism `id_A: A → A`.

Two laws govern these:

- **Associativity:** `(h ∘ g) ∘ f = h ∘ (g ∘ f)` — the order of grouping does not matter.
- **Identity:** `id_B ∘ f = f = f ∘ id_A` — composing with identity changes nothing.

These laws sound trivial but they have a consequence pr4xis exploits everywhere: **if your domain model satisfies them, it has no dead states, no unreachable objects, and no broken compositions.** The compiler and `cargo test -p pr4xis category::validate::check_category_laws` verify the math for every category in the workspace.

## Reasoning systems

A category is a structure. **Reasoning systems** are interpretations of that structure that answer specific kinds of questions.

- **Taxonomy** — interprets some morphisms as `is-a`. Adds the `NoCycles` axiom (a thing cannot be its own ancestor) and `Antisymmetric` (if A is-a B and B is-a A, then A = B). Answers questions like "is dog a mammal?".
- **Mereology** — interprets some morphisms as `part-of`. Adds the `WeakSupplementation` axiom (if a whole has a proper part, it has another disjoint part). Answers questions like "what are the parts of an esophagus?".
- **Causation** — interprets some morphisms as `causes`. Adds the `NoSelfCausation` axiom (a thing cannot cause itself). Answers questions like "what caused this event?".
- **Opposition** — interprets some morphism pairs as `opposes`. Adds `Symmetric` (if A opposes B, B opposes A) and `Irreflexive` (a thing does not oppose itself). Answers questions like "what is the opposite of cold?".
- **Context** — disambiguates entities by context (`ContextDef::resolve`). A potassium channel in a constitutive context is not the same as a potassium channel in a therapeutic context. Closes gaps that adjunctions surface.

Every reasoning system is its own trait that an ontology can choose to implement. The `define_ontology!` macro provides a declarative shorthand for all of them.

## Functors

A **functor** is a structure-preserving map between two categories. If `F: Source → Target` is a functor, then:

- For every object `A` in the source, there is an object `F(A)` in the target.
- For every morphism `f: A → B` in the source, there is a morphism `F(f): F(A) → F(B)` in the target.
- Identities are preserved: `F(id_A) = id_{F(A)}`.
- Composition is preserved: `F(g ∘ f) = F(g) ∘ F(f)`.

The third and fourth conditions are the **functor laws**. If a Rust `impl Functor for X` passes `cargo test -p pr4xis category::validate::check_functor_laws`, the laws hold and the functor is a categorically valid claim that the source domain's structure embeds into the target.

This is what pr4xis means when it says "domains compose with proof". A functor from `Pharmacology → Molecular` is not an analogy or a heuristic mapping — it is a verified theorem that pharmacological structure faithfully embeds in molecular structure. The current workspace has 61 such functor implementations (`grep -rn "impl Functor" crates/domains/src/ crates/pr4xis/src/ | wc -l`).

## Adjunctions and gap detection

When two functors come in opposed pairs, `F: A → B` and `G: B → A`, with `F` going one way and `G` going the other, the pair may form an **adjunction**. The technical definition involves natural transformations called *unit* (`η: Id_A → G ∘ F`) and *counit* (`ε: F ∘ G → Id_B`); the practical consequence is that `F` and `G` are "optimal inverses" of each other, even when neither is a true inverse.

The reason adjunctions matter for pr4xis is **gap detection**. If you take an object `A` in the source category and apply `G(F(A))` — a round-trip through both functors — you get back to the source category. If `G(F(A)) ≠ A`, the source ontology has a missing distinction: the round-trip collapsed `A` into something else because the target ontology cannot represent the difference.

Every collapsed entity is a missing distinction the math detected automatically. This is how the bioelectricity adjunction in `crates/domains/src/natural/biomedical/` discovered that voltage-gated potassium channels (Kv) serve two functionally distinct roles — homeostatic and therapeutic — that the molecular ontology had collapsed into a single entity. The adjunction surfaced the gap; a `ContextDef` resolution then disambiguated the two roles, and the gap closed.

For the live percentages of how much information is lost in each round-trip across the biomedical stack, run `cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture`.

## Composition is the point

Categories are the substrate. Functors are the maps between them. Adjunctions are the paired functors that detect what's missing. Together they answer the question that no other ontology system can: **does this composition preserve structure, with proof?**

A functor between WordNet's lexical taxonomy and BioPortal's biomedical mereology is not just an alignment exercise. If the functor laws hold, the composition is a theorem. If they don't hold, you cannot pretend the two ontologies are saying the same thing — the system tells you exactly which morphism breaks.

This is what pr4xis adds to the existing landscape of formal ontologies. The ontologies have been there for decades; the categorical substrate that makes their composition machine-checkable is the missing piece.

## The Self-Model — categories all the way down

Because pr4xis describes its own structure with the same machinery it uses for any other domain, there is a `Self-Model` ontology in `crates/domains/src/cognitive/cognition/` that describes pr4xis's own architectural concepts as objects in a category. The system can ask itself questions like "which ontologies are loaded?" and "what reasoning systems does each ontology implement?" through exactly the same `Ontology` trait it uses for biology or chess. Self-reference is modeled categorically as a natural transformation, not as a special case in the runtime.

This is a small but load-bearing detail: it is the reason pr4xis can extend itself without bolting on metaprogramming. Every new capability is just another ontology, and every new capability is automatically composable with everything that came before.

## Related

- [Architecture](architecture.md) — the five-layer Rust stack and runtime mechanics
- [Foundations](foundations.md) — academic lineage; every concept above traced to its source paper
- [README](../../README.md) — the project entry point with the LLM contrast table and the bioelectricity gap-detection result
- Per-ontology READMEs (pending [#57](https://github.com/i-am-logger/pr4xis/issues/57)) — for what each individual ontology contains
- Per-ontology diagrams (pending [#59](https://github.com/i-am-logger/pr4xis/issues/59)) — for the visual "neural network of an ontology" view

---

- **Document date:** 2026-04-14
- **Verification:** every claim about the codebase is verifiable by `cargo test -p pr4xis category::validate::check_category_laws`, `cargo test -p pr4xis category::validate::check_functor_laws`, `grep -rn "impl Functor" crates/domains/src/ crates/pr4xis/src/`, or `cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture`.
