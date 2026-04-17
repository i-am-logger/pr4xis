# Dialectics ontology — bibliography

## Primary sources

- **Aristotle (~350 BCE).** *Peri Hermeneias (De Interpretatione)*, chs. 6–7; *Topics*. Source of the Square of Opposition (contrary / contradictory / subaltern / subcontrary) and of dialectical argument as reasoning from `Endoxa`.
- **Apuleius of Madaura (c. 150 CE).** *Peri Hermeneias*. The first extant diagrammatic presentation of Aristotle's Square of Opposition.
- **Hegel, G. W. F. (1807).** *Phänomenologie des Geistes* (Phenomenology of Spirit). Primary source for `Thesis`, `Antithesis`, `Synthesis`, `DialecticalMoment`.
- **Hegel, G. W. F. (1812–16).** *Wissenschaft der Logik* (Science of Logic). Source for `DeterminateNegation` (§§80–82) and `Sublation` (*Aufhebung*) — the triple move of negating-preserving-elevating.
- **Marx, K. (1867).** *Das Kapital*, Vol. I. Source for `InternalContradiction` as the driver of immanent (rather than external) systemic change.
- **Adorno, T. W. (1966).** *Negative Dialektik*. Suhrkamp. Source for `NegativeDialectics` and `NonIdentity` — dialectical reasoning that refuses Hegelian Synthesis.
- **Priest, G. (1987).** *In Contradiction: A Study of the Transconsistent*. Springer. Source for `TrueContradiction` (dialetheism) and the requirement of `Paraconsistent` logic.
- **Blanché, R. (1966).** *Structures intellectuelles: Essai sur l'organisation systématique des concepts*. The hexagonal extension of the Aristotelian Square — not directly encoded here but referenced for future extension.

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../docs/papers/references.md)
- Code-level citations: `grep -n 'Source:\|Aristotle\|Hegel\|Marx\|Adorno\|Priest' *.rs` in this directory
- Related workspace ontologies:
  - `crates/domains/src/cognitive/cognition/distinction.rs` — Spencer-Brown *Laws of Form*. Every dialectical movement begins with a distinction; a future `Distinction → Dialectics` cross-functor would carry `Boundary` ↦ `DeterminateNegation`.
  - `crates/domains/src/formal/meta/syntrometry/` — Heim's syntrometric logic. The `Syntrometry → Dialectics` cross-functor lives at `formal::meta::syntrometry::dialectics_functor` and carries `Dialektik` ↦ `DialecticalMoment`.
  - `crates/pr4xis/src/ontology/reasoning/opposition.rs` — the generic `opposes` reasoning module that Dialectics's own `opposes:` block consumes.

## Pending verification

- [ ] Cross-check every primary source against `docs/papers/references.md` (Aristotle editions, Hegel editions, Marx Vol. I, Adorno English translation, Priest edition).
- [ ] Add code-comment-level citations (`// Source: ...`) to the six domain axioms.
- [ ] Add the `Distinction → Dialectics` cross-functor (Spencer-Brown `Boundary` ↦ `DeterminateNegation`) once design is confirmed.
- [ ] Consider encoding Blanché (1966) hexagonal extension as a sub-ontology or additional concepts if downstream use emerges.

## Project-internal references

- Opposition structure in pr4xis lives in this ontology; the `Syntrometry → Dialectics` cross-functor (at `formal::meta::syntrometry::dialectics_functor`) routes Heim's `Dialektik` here.

---

- **Document date:** 2026-04-17
- **How this file is maintained:** auto-initialized by the per-ontology rollout (issue #57) from `README.md`'s *Key references* section. Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
