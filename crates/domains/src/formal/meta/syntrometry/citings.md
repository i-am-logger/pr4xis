# Syntrometry ontology — bibliography

## Primary sources

- **Heim, B. (~1980).** *Syntrometrische Maximentelezentrik*. Unpublished manuscript; partial summaries surfaced posthumously. The philosophical/logical foundation underneath Heim theory proper.
- **"A Modernized Syntrometric Logic: Foundations and Applications" (2025).** heim-theory.com. <https://heim-theory.com/wp-content/uploads/2025/11/A-Modernized-Syntrometric-Logic-Foundations-and-Applications.pdf>. §1–§2 are the source of every Phase 1 concept in `ontology.rs`; §2.2 "The Syntrix as the Category of Leveled Structures (C_SL)" is the exact claim `lineage_functor_laws_pass` verifies.
- **Spencer-Brown, G. (1969).** *Laws of Form*. Allen & Unwin. The distinction calculus underlying Heim's `Predicate` primitive; encoded separately at `crates/domains/src/cognitive/cognition/distinction.rs`.
- **Mac Lane, S. (1971).** *Categories for the Working Mathematician*. Springer. Ch. II §1 for functors, Ch. II §2 for opposite categories. The endofunctor-as-specialised-functor axiom `SynkolatorIsKorporator` is from Ch. II §1.
- **Conant, R. & Ashby, W. R. (1970).** "Every Good Regulator of a System Must Be a Model of That System." *Int. J. Systems Science* 1(2), 89-97. The cybernetic equivalent of Heim's telecenter/maxime architecture. Phase 2 binds `Maxime → SubIntention` via this result: every regulator *is* a model.
- **Bratman, M. E. (1987).** *Intention, Plans, and Practical Reason*. Harvard University Press. The BDI (Belief-Desire-Intention) architecture. Source for the Phase 2 `Maxime → SubIntention` mapping — Heim's extremal-selection operator is the `Intention` commitment-to-plan.
- **Dehaene, S. (2014).** *Consciousness and the Brain: Deciphering How the Brain Codes Our Thoughts*. Viking. The Global Neuronal Workspace formulation of attention + broadcast. Phase 2 binds `Maxime → SubIntention → C1 Attention` via Dehaene's spotlight.
- **von Foerster, H. (1981).** *Observing Systems*. Intersystems Publications. Second-order cybernetics; introduced *eigenform* X = F(X) as the categorical / self-referential content of identity. Phase 2 binds `Telecenter → SubEigenform` via this construction.
- **Futamura, Y. (1971).** "Partial Evaluation of Computation Process — An Approach to a Compiler-Compiler." *Systems, Computers, Controls* 2(5). The three Futamura projections — the staging hierarchy that pr4xis uses for its `staging` ontology. Phase 2 binds `Transzendenzstufe → SubStagingLevel` here.

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../docs/papers/references.md)
- Source attributions per axiom: see the `Axioms` table in [`README.md`](README.md)
- Code-level citations: `grep -n 'Source:\|Reference:\|Heim\|Mac Lane' *.rs` in this directory
- Lineage verification methodology: [`docs/research/kinded-functor-failures.md`](../../../../../../docs/research/kinded-functor-failures.md) — informs the Phase 1 decision to use a dense `Pr4xisSubstrate` target
- Related workspace ontologies:
  - `crates/domains/src/cognitive/cognition/distinction.rs` — Spencer-Brown. Phase 2 will add `Distinction → Syntrometry::Predicate` functor.
  - `crates/pr4xis/src/category/` — the functor target's reference definitions.
  - `crates/domains/src/cognitive/cognition/consciousness/` — C1/C2. Phase 2 binds transzendenzstufen.
- Memory: `project_heim_transport.md` — architectural mapping to `compose` API (Korporator), system-of-systems (Metroplex), DDS transport (Syntrokline Brücken). Phase 2 functors.

## Pending verification

Every entry under **Primary sources** is a short pointer. For each one, confirm that a full citation (Author, Year, Title, DOI/URL) exists in `docs/papers/references.md`. Where no entry exists, add it (or a local PDF under a `papers/` subdirectory) before declaring the ontology citation-complete.

Open items for human review:

- [ ] Cross-check every primary source against `docs/papers/references.md` (add Heim 1980, modernized 2025 paper, Spencer-Brown 1969 entries if missing).
- [ ] Add code-comment-level citations (`// Source: ...`) to the three domain axioms in `ontology.rs`.
- [ ] Mirror the 2025 heim-theory.com PDF into a local `papers/` subdirectory so the lineage verification doesn't depend on an external URL.
- [ ] Phase 2 — add Dehaene (GWT, 2014), Tononi (IIT, 2008), Bratman (BDI, 1987) citations when telecenters/maximes/transzendenzstufen land.

## Project-internal references

- Issue [#62](https://github.com/i-am-logger/pr4xis/issues/62) — the operationalisation of this lineage claim.
- Issue [#51](https://github.com/i-am-logger/pr4xis/issues/51) (closed) — the first-pass research verdict; superseded by #62.

---

- **Document date:** 2026-04-17
- **How this file is maintained:** auto-initialized by the per-ontology rollout (issue #57) from `README.md`'s *Key references* section. Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
