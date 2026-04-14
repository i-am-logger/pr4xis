# Gap Detection in Scientific Ontologies

> A pr4xis [adjunction](https://en.wikipedia.org/wiki/Adjoint_functors) automatically detected that the molecular biology ontology had collapsed two functionally distinct roles of voltage-gated potassium channels (Kv) into a single entity. The categorical math surfaced the gap, a `ContextDef` resolution disambiguated the two roles, and the gap closed. This document explains how, with re-derivable numbers.

## What this is

The substrate's most distinctive capability is **automated detection of missing distinctions** in scientific ontologies. When two ontologies are connected by a categorical adjunction `F: A → B` and `G: B → A`, the round-trip `G(F(x))` should ideally return `x`. When it does not — when the round-trip *collapses* the original entity into something else — the source ontology has a missing distinction: it represented two things as one, and the target ontology has surfaced the conflation.

Every collapsed entity is a missing distinction the math detected automatically. This is not heuristic and not statistical — it is a categorical theorem about the adjunction. If the adjunction laws hold and the round-trip diverges, the source ontology is provably under-specified at that point.

## How to verify

Every number on this page is re-derivable from a single command:

```
cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture
```

The command runs the live computation against the current codebase and prints the per-adjunction loss percentages plus the per-hop and end-to-end collapse measurements.

## The biomedical stack today

The pr4xis biomedical ontologies (Molecular, Bioelectric, Biology, Pharmacology, Biochemistry, Biophysics, Mechanobiology, Acoustics, plus several more) are connected by three adjunctions and a chain of functors. Running the gap analysis against the current code produces:

| Adjunction | Round-trip unit loss | What the loss means |
|---|---|---|
| Molecular ⊣ Bioelectric | **85.2%** | 85.2% of molecular entities lose their identity when round-tripped through the bioelectric layer. The bioelectric ontology cannot represent the distinctions the molecular ontology makes, so multiple molecular entities collapse onto the same bioelectric concept. Each collapse is a missing distinction. |
| Biology ⊣ Bioelectric | **82.6%** | 82.6% of biological entities collapse the same way at the bioelectric scale. Cell types that are anatomically distinct become indistinguishable when viewed as bioelectric circuits. |
| Pharmacology ⊣ Molecular | **68.0%** | 68.0% of pharmacological entities collapse at the molecular scale. Drugs with distinct clinical profiles share the same molecular target enumeration. |

And the end-to-end measurement across four chained domains:

| Chain | Source entities | Unique targets after composition | Collapse |
|---|---|---|---|
| acoustics → biophysics → molecular → bioelectricity | 26 | **2** | **92.3%** |

26 acoustic concepts compress to 2 distinct bioelectric concepts across four domains. Each domain is a lossy compression of the one below it — a relationship known qualitatively in biology, **quantified categorically here for what we believe is the first time**.

## The Kv channel discovery

The most striking single discovery in the gap analysis is voltage-gated potassium channels (Kv). The molecular ontology had a single `Kv` entity. The Bioelectric ⊣ Molecular adjunction detected that this single entity round-tripped to a different molecular entity, meaning the molecular ontology had collapsed two distinct concepts into one.

Cross-referencing biophysics literature confirmed: Kv channels serve **two functionally different roles**:

1. **Constitutive (homeostatic)** — Kv channels in the resting state, maintaining baseline membrane potential via leak currents
2. **Therapeutic** — Kv channels as pharmacological targets for arrhythmia, epilepsy, and pain disorders

These are the same protein but functionally distinct entities. The molecular ontology had the protein; the contextual disambiguation was missing. A `ContextDef::resolve` was added that distinguishes Kv-in-Constitutive-context from Kv-in-Therapeutic-context, and the gap closed.

This specific case is verified by:

```
cargo test -p pr4xis-domains test_kv_gap_is_resolved_by_context
```

## Why this matters

Three things are unusual about this result.

1. **The discovery was made by mathematics, not by domain experts.** No biologist looked at the molecular ontology and noticed the conflation. The categorical adjunction surfaced it as a forced consequence of the round-trip. The fix required domain knowledge to *interpret* the gap (what does it mean that Kv collapses?), but the gap itself was found mechanically.

2. **The methodology is general.** It works for any pair of ontologies connected by an adjunction. Pharmacology ⊣ Molecular surfaces gaps in pharmacology; Biology ⊣ Bioelectric surfaces gaps in biology. The adjunction is the diagnostic; the loss percentage is the symptom; each collapsed entity is a specific missing distinction.

3. **The gap closure is also categorical.** `ContextDef` is itself a categorical construction (a fibration of contexts over the base category). The fix that closes the gap satisfies the same kind of laws that the adjunction satisfies. The whole loop — gap detection, gap interpretation, gap closure — stays inside the substrate.

## Where to find the code

- `crates/domains/src/formal/meta/gap_analysis.rs` — the gap analysis runner, including `analyze_molecular_bioelectric()`, `analyze_biology_bioelectric()`, `analyze_pharmacology_molecular()`, and `test_full_chain_collapse_measurement`
- `crates/domains/src/natural/biomedical/adjunctions.rs` — the three biomedical adjunctions
- `crates/domains/src/natural/biomedical/molecular/ontology.rs` — the `MolecularEntity` enum, including `Kv`
- `crates/domains/src/natural/biomedical/molecular/` — the `MolecularFunctionalContext` and the `ContextDef` resolution that closed the Kv gap

## Where this fits in the bigger picture

Gap detection is the most distinctive single capability of the pr4xis substrate, but it is not the only one. The same categorical machinery — categories, functors, adjunctions — also enables:

- **Cross-domain reasoning** — proven structural correspondences between domains via 61 functor implementations
- **Composable verification** — domains that pass the functor laws can be composed without breaking other proofs
- **Trace-as-proof** — every chain of reasoning the engine produces is a categorical morphism, traceable from conclusion back to source axioms

For the broader story, see the [README](../../README.md). For the abstract concepts, see [Concepts](../understand/concepts.md). For the academic lineage, see [Foundations](../understand/foundations.md).

## Related issues

- [#60](https://github.com/i-am-logger/pr4xis/issues/60) — once the source-of-truth report pipeline is live, the percentages on this page will pull from the deployed JSON instead of being hand-typed
- [#57](https://github.com/i-am-logger/pr4xis/issues/57) — once each ontology has a per-domain README, the Molecular and Bioelectric READMEs will link directly to this page
- [#59](https://github.com/i-am-logger/pr4xis/issues/59) — the per-ontology mermaid diagrams will visualize the adjunction connections that produced this result

---

- **Document date:** 2026-04-14
- **Verification:** every number on this page is re-derivable by the cited `cargo test` commands. The collapse percentages are computed live from the actual functor implementations every test run; they will update automatically as the biomedical ontologies evolve.
