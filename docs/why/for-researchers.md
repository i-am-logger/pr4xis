# pr4xis for Researchers

You are a researcher — academic, industrial, or independent — evaluating whether pr4xis is intellectually credible and worth your time. This page is for you.

## The novelty claim, briefly

pr4xis is, **to our knowledge, the first executable, test-verified instance of a categorical substrate for composing scientific ontologies across many domains**.

That is a careful sentence. Each clause matters:

- **Executable.** The substrate runs as a Rust workspace, not a paper or a Coq library. Every concept, every functor, every axiom corresponds to compilable code.
- **Test-verified.** Category laws and functor laws are checked at test time via `cargo test --workspace`. There are 4,855 tests in the workspace today.
- **Categorical substrate.** Domains are categories in the formal mathematical sense. Compositions between domains are functors with verified laws. Adjunctions detect missing distinctions.
- **For composing scientific ontologies.** The intent is not pure-math theorem proving (Coq/Lean/Agda do that). It is putting WordNet, BioPortal, the Gene Ontology, DOLCE, and the rest into a substrate where they can be combined with proof.
- **Across many domains.** Currently 106 ontologies covering biomedical, sensor fusion, navigation, linguistics, formal mathematics, music, colors, judicial workflow, and more.

The full novelty argument with comparisons to prior art lives in [Novelty](../research/novelty.md).

## The intellectual lineage

pr4xis sits in a sixty-year tradition of **distinction-calculus** and **compositional logic**:

- **G. Spencer-Brown, *Laws of Form* (1969)** — the calculus of indication. Spencer-Brown starts with one instruction: "Draw a distinction." From that single act, all of logic, Boolean algebra, and self-reference emerge. Spencer-Brown's mark is the same gesture as a category-theoretic morphism: the act of distinguishing one object from another.

- **Burkhard Heim, *Syntrometrische Maximentelezentrik* (mid-20th century, published posthumously)** — a logical framework built from predicates, permutation operators, mereological composition, and goal-oriented "telecenters". For most of its history, syntrometry was treated as Heim's philosophical scaffolding for his unified field theory and was not engaged with on its own terms.

- **A Modernized Syntrometric Logic (2025)** — heim-theory.com — explicitly reformulates Heim's syntrometric machinery using modern category theory, modal logic with Kripke semantics, classical extensional mereology, and natural transformations. This is the decisive recent work for the lineage claim.

The structural alignment between pr4xis and the modernized syntrometric logic is concrete in **eight of nine** mapping points (verified during the [#51](https://github.com/i-am-logger/pr4xis/issues/51) first-pass research):

1. **Syntrix as category of leveled structures, Synkolator as endofunctor** ↔ pr4xis's ontology-as-category + functors-as-morphisms-of-ontologies
2. **Aspektrelativität via Kripke frames** ↔ pr4xis's multiple ontologies viewing the same domain via functorial alignment
3. **Classical extensional mereology for Part(A,B)** ↔ pr4xis's `MereologyDef` reasoning system with `WeakSupplementation`
4. **Reflexivity as natural transformation** ↔ pr4xis's `Self-Model` ontology describing its own structure
5. **Korporator as structure-mapping functor** ↔ pr4xis's cross-domain functors
6. **Hypersyntrix as category-of-categories** ↔ pr4xis's higher-order composition through repeated functor application
7. **Predicates as primitives** ↔ pr4xis's `Entity` enums
8. **C/c permutation operators (sequence and orientation)** ↔ pr4xis's morphism composition under associativity

The one that does NOT map: **adjunctions**. Heim does not have adjoint functors; the gap-detection mechanism that pr4xis uses — `F` and `G` as paired functors with unit and counit, round-trip collapse as a missing-distinction signal — has no Heim counterpart. This is genuinely pr4xis's contribution.

What pr4xis explicitly does **not** inherit: Heim's twelve-dimensional spacetime, particle mass formulas, "Metronic Gitter", or teleological cosmology. The structural overlap is real; the metaphysical extensions are not.

## What's verifiable today

| Result | Re-derivation |
|---|---|
| 4,855 machine-verified tests | `cargo test --workspace` |
| 106 ontologies | `find crates/domains/src -name ontology.rs \| wc -l` |
| 61 cross-domain functor implementations | `grep -rn "impl Functor" crates/domains/src/ crates/pr4xis/src/` |
| 85.2% molecular-bioelectric round-trip collapse (the Kv discovery) | `cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture` |
| Kv channel split by `ContextDef` resolution | `cargo test -p pr4xis-domains test_kv_gap_is_resolved_by_context` |
| Engine state navigability (back/forward/branch) | `cargo test -p pr4xis test_back_forward_roundtrip test_next_after_back_clears_future` |

## What's intentionally not yet claimed

- **No claim of formal proof in Coq/Lean/Agda.** pr4xis verifies its axioms via property-based testing and exhaustive enumeration of finite categories, not by interactive theorem proving. If you need a Coq mechanization of one of the proofs, that is open work.
- **No claim that BioPortal, the Gene Ontology, etc., are integrated today.** WordNet is integrated via `codegen::wordnet`. The biomedical stack consists of pr4xis-authored ontologies that are *informed by* the literature but not *imported from* BioPortal. Importing existing ontology corpora is the next layer of work.
- **No claim of completeness in any single domain.** The biology ontology has ~30 concepts; real biology has hundreds of thousands. The point is that the categorical substrate is the right *shape* for them, not that every concept has been authored yet.

## Open research directions

The pr4xis project has several open questions where research collaboration would land:

1. **Adjunction discovery.** The biomedical adjunctions were authored by hand. The categorical question: given two categories with shared morphisms, is there an algorithm that finds candidate adjunctions automatically? This is a real open research problem.
2. **Composition with existing OWL/RDF ontology corpora.** Importing BioPortal, the Gene Ontology, OBO Foundry, and DOLCE as composable categories. The conceptual mapping (RDF triples → category morphisms, OWL axioms → category-theoretic axioms) is sketched but not implemented.
3. **The relationship to Spivak's ologs and functorial data migration** — pr4xis's evolution-via-functor model (see [Evolution](../understand/evolution.md)) is a specific instance of Spivak's pattern, but the connection has not been formalized in writing.
4. **Self-reference and reflexivity.** The `Self-Model` ontology describes pr4xis's own type system. The fixed-point semantics (the system observing itself) is a topic where second-order cybernetics, eigenform theory, and category theory all converge. Pr4xis has the executable substrate for this, but the theoretical write-up is not done.
5. **LLM weight projection** ([#50](https://github.com/i-am-logger/pr4xis/issues/50)) — projecting an external ontology onto LLM internals via a functor as a mechanistic interpretability framework. Speculative, but the substrate is the right shape for it.

## Where to go from here

- [Concepts](../understand/concepts.md) — what an ontology is in pr4xis, why categories, how functors and adjunctions work
- [Foundations](../understand/foundations.md) — the full academic lineage with sources for every claim
- [Novelty](../research/novelty.md) — the long-form novelty argument and the comparison to prior art
- [Gap detection](../research/gap-detection.md) — the bioelectricity result in technical detail
- [Paper outline](../research/paper-outline.md) — the draft architecture paper

---

- **Document date:** 2026-04-14
