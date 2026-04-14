# What's Novel About pr4xis

This document makes the novelty argument carefully: **what pr4xis claims to be the first to do, what is genuinely new versus standing on prior art, and how the project applies its own honesty principle to its lineage claims.**

The short version: pr4xis is the first **executable, test-verified, multi-domain** instance of a categorical substrate for composing formally-grounded ontologies with proof. The substrate is novel as code; many of the structural ideas have a sixty-year intellectual history.

## The novelty claim

> pr4xis is, **to our knowledge**, the first executable, test-verified instance of a categorical substrate for composing scientific ontologies across many domains, with categorical adjunctions used to detect missing distinctions in the source ontologies automatically.

Each clause is calibrated to be defensible:

- **"To our knowledge"** is a hedge. We have not exhaustively surveyed every Coq/Agda/Lean library, every applied-category-theory project, or every formal ontology framework. If a prior project did the same thing, we would like to know. The hedge stays until we have done the literature survey, which is open work (see "Open verifications" below).
- **"Executable"** distinguishes pr4xis from formal frameworks that exist only as papers, theorems, or interactive proof scripts. pr4xis is a Rust workspace that compiles, runs, serves a WASM browser demo, and ships ontologies as executable artifacts.
- **"Test-verified"** means the structural laws (category laws, functor laws, axiom satisfaction) are checked at test time via `cargo test --workspace`. There are 4,855 such tests today.
- **"Multi-domain"** distinguishes pr4xis from frameworks that demonstrate the categorical substrate on a single domain (a database schema, a single biology taxonomy, etc.). pr4xis has 106 domain ontologies covering biomedical, sensor fusion, navigation, perception, tracking, space, underwater, industrial, linguistics, formal mathematics, music, colors, judicial, and more.
- **"Adjunctions used for gap detection"** is the most concretely novel claim. The bioelectricity Kv discovery (see [Gap detection](gap-detection.md)) is the first instance we know of where a categorical adjunction was used to automatically surface a missing distinction in a *scientific* ontology, leading to a real fix. The methodology is general — any pair of ontologies connected by an adjunction can be analyzed this way.

## What is not original

The honest list of things pr4xis stands on:

- **Category theory** — Mac Lane (1971), Riehl (2016), Awodey (2010), and a sixty-year tradition. pr4xis is a consumer of category theory, not a contributor to its foundations.
- **Applied category theory** — Spivak's ologs and functorial data migration (2010, 2012), Fong & Spivak's *Seven Sketches in Compositionality* (2019), Coecke & Kissinger on categorical quantum mechanics (2017), the broader applied-CT community. pr4xis applies these patterns to scientific ontologies; it does not invent the patterns.
- **Formal ontology** — Guarino (1998), Gruber (1993), DOLCE (Masolo et al. 2003), BFO (Smith, ongoing). pr4xis adopts DOLCE as its upper layer and uses the existing formal-ontology vocabulary.
- **Cybernetics and the Good Regulator Theorem** — Conant & Ashby (1970), Ashby (1956), Wiener (1948), Bateson (1972), von Foerster (1981). The architectural justification (the engine must contain a model of the system) is Conant-Ashby; pr4xis is one specific way to satisfy the theorem.
- **The DisCoCat tradition** for compositional language — Coecke, Sadrzadeh & Clark (2010), Lambek pregroup grammar (1958, 1999). pr4xis's linguistics pipeline is built on this.
- **Distinction-calculus** — Spencer-Brown's *Laws of Form* (1969). The act of drawing a distinction is older than category theory and pr4xis is one descendent of that tradition.
- **Syntrometric logic** — Heim (1980, posthumously published), reformulated categorically in 2025. **The lineage claim with respect to Heim is the most-prominent unverified assertion in the project. See "The Heim lineage — pending machine-verification" below for how we plan to convert it from research claim to test-checked theorem.**

## What is genuinely new

After subtracting the prior art, the things we believe pr4xis contributes — pending the literature survey that would let us drop the "to our knowledge" hedge:

1. **Adjunction-based automatic gap detection in scientific ontologies.** Categorical adjunctions are well-known in math and theoretical computer science; using them as a *diagnostic* on ontology pairs to surface missing distinctions is, as far as we know, original. The bioelectricity Kv result is the proof of concept. See [Gap detection](gap-detection.md).

2. **A multi-domain categorical substrate that compiles and runs.** Spivak's ologs are categorical knowledge representations but are presented mathematically rather than as a living codebase. Coq/Agda/Lean libraries for category theory exist but are oriented toward pure-math theorem proving, not applied domain reasoning across many fields. The combination — categorical substrate, multi-domain coverage, executable Rust, runtime engine, WASM browser surface — does not appear to exist in another project, but we have not done the exhaustive survey.

3. **The `define_ontology!` macro pattern.** A declarative way to specify an ontology that emits the category, the reasoning systems (taxonomy, mereology, causation, opposition), the structural axioms, and the metadata in a single block. The pattern is similar to Spivak's olog notation but is concrete Rust code that the type system checks. We are not aware of an equivalent in another applied-CT framework.

4. **Functor-based ontology evolution as a first-class concern.** See [Evolution](../understand/evolution.md). The pattern of "transform via functor, never rewrite" is implicit in Spivak's functorial data migration but pr4xis applies it to ontological evolution explicitly and operationally — every evolution is a functor whose laws are checked at test time.

5. **The explicit codegen / async / mmap functor equivalence.** Three different mechanisms for delivering ontology data into the runtime, all proven equivalent as functors from the same source. This is a small-but-load-bearing piece of infrastructure that lets the same ontology run as a static binary, an asynchronously loaded resource, or a memory-mapped file without semantic drift.

## The Heim lineage — pending machine-verification

The most prominent lineage claim in the project is the structural alignment with the **modernized syntrometric logic tradition** (Heim 1980, reformulated categorically in 2025).

The first-pass research in [#51](https://github.com/i-am-logger/pr4xis/issues/51) found eight of nine concept mappings between Heim's syntrometric machinery and pr4xis to be "concrete":

| Heim / syntrometric concept | pr4xis concept |
|---|---|
| Syntrix as category of leveled structures (C_SL) | Ontology as category |
| Synkolator as endofunctor | Cross-domain functor |
| C/c permutation operators | Morphism composition under associativity |
| Mereological Part(A,B) (CEM) | `MereologyDef` with `WeakSupplementation` |
| Aspektrelativität via Kripke frames | Multiple ontologies viewing same domain via functors |
| Reflexivity ρ as natural transformation | `Self-Model` ontology |
| Predicates as primitives | `Entity` enums |
| Korporator as structure-mapping functor | Cross-domain functor between Syntrices |
| Adjoint functors | (no Heim counterpart — pr4xis's gap detection is novel) |

That research is enough to make the claim *plausible*. It is **not** enough to make the claim *machine-verified* — and per the project's core principle, plausible-but-unverified is exactly the kind of claim pr4xis exists to eliminate.

The fix is to **encode Heim's syntrometric primitives as a pr4xis ontology and prove the structural alignment via a verified functor**. That work is tracked in [#62](https://github.com/i-am-logger/pr4xis/issues/62). When it lands:

- A `SyntrometricLogic` ontology will exist as a `define_ontology!` block in the workspace.
- A functor `SyntrometricToCategory: SyntrometricLogic → pr4xis::CoreCategory` will pass `check_functor_laws()`.
- The CEM mereology alignment will be verified by a test showing Heim's `Part(A,B)` reduces to `WeakSupplementation`.
- The lineage claim becomes citable as a passing test, not a research narrative.

Until #62 lands, this document and every other doc that mentions the Heim lineage uses the hedged framing: *"pr4xis sits in a tradition that includes Heim's syntrometric logic; the structural alignment has been argued in research and is in the process of being machine-verified."* When #62 lands, the framing strengthens to: *"pr4xis is structurally aligned with Heim's syntrometric logic; the alignment is verified by `cargo test ... test_syntrometric_to_pr4xis_functor_laws`."*

What pr4xis explicitly does **not** inherit, regardless of #62: Heim's twelve-dimensional spacetime, particle mass formulas, Metronic Gitter, or teleological cosmology. The structural substrate is real; the metaphysical extensions are not adopted.

## Open verifications

For the novelty claim to lose its "to our knowledge" hedge, two more pieces of work are needed:

1. **A literature survey for prior multi-domain categorical-ontology projects.** Specifically: Coq/Agda/Lean libraries that implement category theory applied to multiple scientific domains; applied-CT frameworks that go beyond a single use case; ontology platforms (BioPortal, Cyc, etc.) that have flirted with categorical formalization. If any of them are doing what pr4xis is doing, we want to cite them — and either acknowledge the prior work or sharpen our novelty claim to the specific dimension where pr4xis is still first.

2. **The Heim ontology and functor implementation** ([#62](https://github.com/i-am-logger/pr4xis/issues/62)). Once the lineage is machine-verified, the "to our knowledge first executable instance of this tradition" claim can be defended without hedging.

Both are tracked. Neither is done.

## How this document will evolve

Per the project's own evolution model (see [Evolution](../understand/evolution.md)), this novelty document is itself subject to change — and the changes should be visible. Two specific updates are pending:

- **When the literature survey lands**: every "to our knowledge" hedge is either dropped (if no prior art is found) or replaced with a specific citation and a sharpened claim about what pr4xis still does that the prior project did not.
- **When [#62](https://github.com/i-am-logger/pr4xis/issues/62) lands**: the Heim section moves from "pending machine-verification" to "verified by the following tests", and the README's lineage sentence loses its "to our knowledge" hedge for the Spencer-Brown / Heim half.

## Related

- [Foundations](../understand/foundations.md) — academic lineage with sources for every claim
- [Concepts](../understand/concepts.md) — what an ontology is in pr4xis, why categories
- [Gap detection](gap-detection.md) — the bioelectricity result, the strongest single concrete demonstration of a novel methodology
- [Paper outline](paper-outline.md) — draft architecture paper
- [#51](https://github.com/i-am-logger/pr4xis/issues/51) — the syntrometric logic first-pass research
- [#62](https://github.com/i-am-logger/pr4xis/issues/62) — the Heim ontology implementation that will machine-verify the lineage

---

- **Document date:** 2026-04-14
