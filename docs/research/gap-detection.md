# Gap Detection in Scientific Ontologies

> **One-paragraph result.** A pr4xis [adjunction](../reference/glossary.md#adjunction) automatically detected that the molecular biology ontology had collapsed two functionally distinct roles of voltage-gated potassium channels (Kv) into a single entity. Categorical math surfaced the gap; a `ContextDef` resolution disambiguated the two roles; the gap closed. **85.2%** of molecular entities collapse in the Molecular ⊣ Bioelectric round-trip — every collapse is a missing distinction the math detected automatically. Reproduce in 5 seconds:
>
> ```
> cargo test -p pr4xis-domains test_full_chain_collapse_measurement -- --nocapture
> ```

This page is the casual-reader summary of a categorical methodology that is described in full as a draft research paper — see **[Adjunction Information Loss in Scientific Ontologies](papers/02-adjunction-information-loss.md)** for the methods, the per-entity tables, the literature verification, and the testable predictions.

## What gets verified

- **Three biomedical adjunctions** between Molecular ↔ Bioelectric, Pharmacology ↔ Molecular, Biology ↔ Bioelectric. Code at `crates/domains/src/natural/biomedical/adjunctions.rs`.
- **Per-adjunction collapse percentages** computed live by the gap-analysis runner. The output of `test_full_chain_collapse_measurement` is the source of truth.
- **The Kv channel discovery** — voltage-gated potassium channels serve two functionally distinct roles (constitutive homeostasis vs therapeutic target). The molecular ontology had collapsed them; the adjunction surfaced the gap; a `ContextDef::resolve` distinguishes `(Kv, Constitutive)` from `(Kv, Therapeutic)`. Verified by `cargo test -p pr4xis-domains test_kv_gap_is_resolved_by_context`.

## Why this matters in one sentence

The discovery was made by mathematics, not by domain experts — the categorical adjunction surfaced the conflation as a forced consequence of the round-trip, and the methodology generalizes to any pair of ontologies connected by an adjunction.

## Read more

- **[Paper 02 — Adjunction Information Loss](papers/02-adjunction-information-loss.md)** — the academic version: methods, results, discussion, references, with inline footnote verification next to every numerical claim.
- **[Concepts](../understand/concepts.md#adjunctions-and-gap-detection)** — what adjunctions are and why they detect gaps, in plain English.
- **[Foundations](../understand/foundations.md)** — academic lineage including the categorical machinery this result builds on.
- **[#60](https://github.com/i-am-logger/pr4xis/issues/60)** — once the source-of-truth pipeline lands, the percentages on this page will pull from a deployed JSON instead of being hand-typed.

---

- **Document date:** 2026-04-14
