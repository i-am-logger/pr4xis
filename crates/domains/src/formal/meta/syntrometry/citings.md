# Syntrometry ontology — bibliography

## Primary sources

- **Heim, B. (~1980).** *Syntrometrische Maximentelezentrik*. Unpublished manuscript; partial summaries surfaced posthumously. The philosophical/logical foundation underneath Heim theory proper.
- **"A Modernized Syntrometric Logic: Foundations and Applications" (2025).** heim-theory.com. <https://heim-theory.com/wp-content/uploads/2025/11/A-Modernized-Syntrometric-Logic-Foundations-and-Applications.pdf>. §1–§2 are the source of every syntrometric concept in `ontology.rs`; §2.2 "The Syntrix as the Category of Leveled Structures (C_SL)" is the exact claim `lineage_functor_laws_pass` verifies.
- **Spencer-Brown, G. (1969).** *Laws of Form*. Allen & Unwin. The distinction calculus underlying Heim's `Predicate` primitive; encoded separately at `crates/domains/src/cognitive/cognition/distinction.rs` and aligned by the `Distinction → Syntrometry` functor.
- **Mac Lane, S. (1971).** *Categories for the Working Mathematician*. Springer. Ch. II §1 for functors, Ch. II §2 for opposite categories, Ch. II §3 for product categories, Ch. II §4 for natural transformations, Ch. V §1 for subobjects and quotient objects. Grounds `SubFunctor`, `SubEndofunctor`, `SubProductCategory`, `SubNaturalTransformation`, `SubObject` in the Pr4xisSubstrate mirror; also the `SynkolatorIsKorporator` axiom (Ch. II §1 specialisation).
- **Mac Lane, S. (1963).** *Homology*. Springer Grundlehren. Ch. II §2 on graded modules — the origin of the categorical "graded object" vocabulary. Grounds `SubGradedObject` in the substrate.
- **Stanley, R. P. (1986).** *Enumerative Combinatorics, Volume 1*. Wadsworth & Brooks/Cole. Ch. 3 on graded posets and ranked structures. Secondary source for `SubGradedObject`.
- **Awodey, S. (2010).** *Category Theory* (2nd edition). Oxford University Press. Ch. 5 on subobjects in modern notation. Secondary source for `SubObject`.
- **Conant, R. & Ashby, W. R. (1970).** "Every Good Regulator of a System Must Be a Model of That System." *Int. J. Systems Science* 1(2), 89-97. The cybernetic equivalent of Heim's telecenter/maxime architecture — grounds the `Maxime → SubIntention` mapping ("every regulator *is* a model").
- **Bratman, M. E. (1987).** *Intention, Plans, and Practical Reason*. Harvard University Press. The BDI (Belief-Desire-Intention) architecture — Heim's extremal-selection operator `Maxime` corresponds to Bratman's `Intention` commitment-to-plan.
- **Dehaene, S. (2014).** *Consciousness and the Brain: Deciphering How the Brain Codes Our Thoughts*. Viking. The Global Neuronal Workspace formulation of attention + broadcast — grounds the `Syntrometry → C1` cross-functor, mapping `Maxime` to `Attention` and `Metroplex` to `GlobalWorkspace`.
- **von Foerster, H. (1981).** *Observing Systems*. Intersystems Publications. Second-order cybernetics; introduced *eigenform* X = F(X) — grounds the `Telecenter → SubEigenform` mapping.
- **Futamura, Y. (1971).** "Partial Evaluation of Computation Process — An Approach to a Compiler-Compiler." *Systems, Computers, Controls* 2(5). The three Futamura projections underlie pr4xis's `staging` ontology; grounds the `Transzendenzstufe → SubStagingLevel` mapping and the `Syntrometry → Staging` cross-functor.
- **Goguen, J.** (various papers on institutions and ontology morphisms) and **Zimmermann, A.** (ontology alignment via Kripke semantics). Grounds the `Syntrometry → Algebra` cross-functor — Heim's composition operators align with pr4xis's `compose` API primitives (Coproduct/Product/Pushout/Pullback).

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../docs/papers/references.md)
- Source attributions per axiom: see the `Axioms` table in [`README.md`](README.md)
- Code-level citations: `grep -n 'Source:\|Reference:\|Heim\|Mac Lane' *.rs` in this directory
- Lineage verification methodology: [`docs/research/kinded-functor-failures.md`](../../../../../../docs/research/kinded-functor-failures.md) — informs the choice to use a dense `Pr4xisSubstrate` target for the primary lineage functor
- Related workspace ontologies:
  - `crates/domains/src/cognitive/cognition/distinction.rs` — Spencer-Brown, source of the `Distinction → Syntrometry` functor
  - `crates/domains/src/cognitive/cognition/consciousness/` — C1/C2; target of the `Syntrometry → C1` functor
  - `crates/pr4xis/src/category/` — the categorical substrate the primary lineage functor targets
- Memory: `project_heim_transport.md` — architectural mapping to `compose` API (Korporator), system-of-systems (Metroplex), DDS transport (Syntrokline Brücken).

## Pending verification

Every entry under **Primary sources** is a short pointer. For each one, confirm that a full citation (Author, Year, Title, DOI/URL) exists in `docs/papers/references.md`. Where no entry exists, add it (or a local PDF under a `papers/` subdirectory) before declaring the ontology citation-complete.

Open items for human review:

- [ ] Cross-check every primary source against `docs/papers/references.md` (Heim 1980, modernized 2025 paper, Spencer-Brown 1969, Dehaene 2014, Bratman 1987, von Foerster 1981, Futamura 1971 are all new additions).
- [ ] Add code-comment-level citations (`// Source: ...`) to the six domain axioms in `ontology.rs`.
- [ ] Mirror the 2025 heim-theory.com PDF into a local `papers/` subdirectory so the lineage verification doesn't depend on an external URL.

## Project-internal references

- Issue [#62](https://github.com/i-am-logger/pr4xis/issues/62) — the operationalisation of this lineage claim.
- Issue [#51](https://github.com/i-am-logger/pr4xis/issues/51) (closed) — the first-pass research verdict; superseded by #62.

---

- **Document date:** 2026-04-17
- **How this file is maintained:** auto-initialized by the per-ontology rollout (issue #57) from `README.md`'s *Key references* section. Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
