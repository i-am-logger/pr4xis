# Reasoning — substrate meta-ontology

Grounds the `ontology::reasoning` module's machinery (structural-axiom catalog, OnKind axioms, analogy functor) in a named concept vocabulary.

## Why this lives in core

The `ontology::reasoning/` module provides substrate machinery (catalog, OnKind axioms, Analogy). Per pr4xis's substrate-grounding principle, the concept these operations realise — Reasoning — must be named in an ontology alongside the machinery. This is exactly the same pattern as `category::category_theory` grounding `category/arrow.rs` et al., or `logic::proof_theory` grounding `logic/axiom.rs`.

## Scope

**Umbrella ontology.** Names Reasoning, Inference, Premise, Conclusion, InferenceRule, Argument, ValidArgument, SoundArgument, plus the four modes (Deduction / Induction / Abduction / Analogy), plus Syllogism, Hypothesis, Evidence.

**Not here:**
- Specific inference rules (ModusPonens, ConjunctionIntroduction, etc.) — `crates/domains/src/formal/logic/inference_rules/`.
- Proof-theoretic structure (Theorem, Cut, Sequent, Normalisation) — `crates/pr4xis/src/logic/proof_theory/`.
- Model-theoretic semantics — `crates/domains/src/formal/logic/model_theory/`.
- Kripke modal logic — `crates/domains/src/formal/logic/kripke/`.

## Literature

| Concept | Source |
|---|---|
| Reasoning | Peirce (1903) Collected Papers |
| Inference / Premise / Conclusion | Frege (1879) Begriffsschrift §6 |
| Deduction / Induction / Abduction | Peirce (1878) "Deduction, Induction, and Hypothesis"; Peirce (1903) |
| Analogy | Polya (1954) Patterns of Plausible Inference; Gentner (1983) Structure-Mapping |
| Argument / ValidArgument / SoundArgument / Syllogism | Aristotle Prior Analytics |
| InferenceRule | Gentzen (1935) Untersuchungen über das logische Schließen |
| Hypothesis | Peirce (1903); Popper (1934) |
| Evidence | Mill (1843) A System of Logic; Carnap (1950) Logical Foundations of Probability |
| Induction problem | Hume (1748) An Enquiry Concerning Human Understanding |

## Tests

- `category_laws` — Mac Lane (1971) Ch. I §1 laws verified on `ReasoningCategory`.
- `ontology_validates` — aggregate validation: category laws + catalog-inherited OnKind axioms.
- `peircean_trichotomy_plus_analogy_are_reasoning_modes` — structural check.
- `sound_implies_valid` — Aristotle's soundness = validity + premise-truth.
- `inference_has_premise_conclusion_rule` — Frege's inference structure.
- `every_concept_has_tradition` — Gruber (1993) KAS 5 naming principle.
- Four proptest cases: tradition totality, arrow naming (Gruber), dangling-target detection, structural-axiom verification.
