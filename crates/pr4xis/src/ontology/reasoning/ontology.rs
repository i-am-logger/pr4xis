//! Reasoning — the substrate ontology grounding `ontology::reasoning`.
//!
//! # Why this ontology lives in core
//!
//! `ontology::reasoning` provides machinery — OnKind structural axioms,
//! the axiom catalog, and the [`Analogy`](super::analogy::Analogy) functor — that operates
//! UNDER the concept of reasoning. Per the project's substrate-grounding
//! principle, the concept itself must live with the machinery. This
//! ontology names Reasoning as an umbrella concept and the four canonical
//! modes (deduction / induction / abduction / analogy) per Peirce
//! (1903).
//!
//! # Scope
//!
//! This is the **umbrella** ontology. It names Reasoning, Inference,
//! Premise, Conclusion, Argument, and the four modes. Specific inference
//! rules (ModusPonens, ConjunctionIntroduction, etc.) live in
//! `crates/domains/src/formal/logic/inference_rules/` which is a
//! downstream specialisation. Proof-theoretic structure (Theorem, Proof,
//! Counterexample) lives in `crates/pr4xis/src/logic/proof_theory/` —
//! the atemporal / sequent-calculus view.
//!
//! # Literature
//!
//! - **Peirce (1903)** *Collected Papers of Charles Sanders Peirce* —
//!   the deduction / induction / abduction trichotomy.
//! - **Frege (1879)** *Begriffsschrift* — formal inference, premise,
//!   conclusion.
//! - **Polya (1954)** *Patterns of Plausible Inference* — heuristic vs
//!   demonstrative reasoning; analogy as reasoning mode.
//! - **Aristotle** *Prior Analytics* — syllogism, valid vs invalid
//!   deduction.
//! - **Gentzen (1935)** *Untersuchungen über das logische Schließen*
//!   — natural-deduction inference rules.
//! - **Hume (1748)** *An Enquiry Concerning Human Understanding* —
//!   induction and the problem of causal inference.
//! - **Mill (1843)** *A System of Logic* — methods of agreement and
//!   difference (inductive reasoning).
//! - **Gentner (1983)** "Structure-Mapping: A Theoretical Framework
//!   for Analogy" — structure-preserving analogical reasoning (mapped
//!   to pr4xis's `Analogy` = named `Functor`).

use crate as pr4xis;
use crate::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Reasoning",
    source: "Peirce (1903) Collected Papers; Frege (1879) Begriffsschrift; Polya (1954) Patterns of Plausible Inference; Aristotle Prior Analytics; Gentzen (1935); Hume (1748); Mill (1843); Gentner (1983) Structure-Mapping",

    concepts: [
        // === Umbrella ===
        Reasoning,

        // === Single-step structure ===
        Inference,
        Premise,
        Conclusion,
        InferenceRule,

        // === Argument (chain of inferences) ===
        Argument,
        ValidArgument,
        SoundArgument,

        // === Four modes (Peirce 1878/1903; Polya 1954) ===
        Deduction,
        Induction,
        Abduction,
        Analogy,

        // === Deductive structures ===
        Syllogism,

        // === Epistemic status ===
        Hypothesis,
        Evidence,
    ],

    labels: {
        Reasoning: ("en", "Reasoning",
            "The general practice of deriving conclusions from premises. Per Peirce (1903), reasoning comes in three fundamental modes (deduction, induction, abduction); Polya (1954) adds analogy as a fourth plausible-inference mode."),
        Inference: ("en", "Inference",
            "A single step of reasoning — a premise-to-conclusion move licensed by an InferenceRule. Frege (1879) Begriffsschrift §6."),
        Premise: ("en", "Premise",
            "A proposition assumed or asserted as a starting point of an inference. Aristotle Prior Analytics I.1."),
        Conclusion: ("en", "Conclusion",
            "The proposition reached at the end of an inference — what the premises are claimed to support. Aristotle Prior Analytics I.1."),
        InferenceRule: ("en", "Inference rule",
            "A schema that licenses an inference — maps a shape of premises to a conclusion shape. Gentzen (1935); see `formal::logic::inference_rules` for the catalog."),

        Argument: ("en", "Argument",
            "A sequence of inferences composed end-to-end, terminating at a final conclusion. Aristotle; formalised in Hilbert-style proof systems."),
        ValidArgument: ("en", "Valid argument",
            "An argument whose conclusion follows necessarily from its premises under the inference rules — no model satisfies the premises while falsifying the conclusion. Aristotle; Tarski (1936) on logical consequence."),
        SoundArgument: ("en", "Sound argument",
            "A valid argument whose premises are additionally true. Validity + premise-truth = conclusion-truth. Aristotle."),

        Deduction: ("en", "Deduction",
            "Necessary inference from general to specific — given a rule and a case, derive the result. Peirce (1878/1903)."),
        Induction: ("en", "Induction",
            "Inference from specific to general — given a case and a result, generalise to a rule. Peirce (1878/1903); Hume (1748); Mill (1843)."),
        Abduction: ("en", "Abduction",
            "Inference to the best explanation — given a rule and a result, hypothesise the case. Peirce (1878/1903)."),
        Analogy: ("en", "Analogy",
            "Structure-preserving mapping between domains — Gentner (1983) structure-mapping. In pr4xis realised as a named `Functor`."),

        Syllogism: ("en", "Syllogism",
            "Aristotle's classical deductive pattern: two premises (major + minor) yield a conclusion. Aristotle Prior Analytics I.4ff."),

        Hypothesis: ("en", "Hypothesis",
            "A proposed explanation awaiting confirmation or refutation. The output of abduction; the subject of evidential evaluation."),
        Evidence: ("en", "Evidence",
            "Data that bears on the truth of a hypothesis — supporting or refuting. Carnap (1950) on confirmation; Mill (1843) methods of agreement/difference."),
    },

    is_a: [
        // Four modes are kinds of Reasoning
        (Deduction, Reasoning),
        (Induction, Reasoning),
        (Abduction, Reasoning),
        (Analogy, Reasoning),

        // Syllogism is a kind of Deduction
        (Syllogism, Deduction),

        // Sound implies Valid (and both are kinds of Argument)
        (ValidArgument, Argument),
        (SoundArgument, ValidArgument),
    ],

    has_a: [
        // An Inference has a Premise, a Conclusion, and an InferenceRule
        (Inference, Premise),
        (Inference, Conclusion),
        (Inference, InferenceRule),

        // An Argument is a chain of Inferences — it has Premises + Conclusion
        (Argument, Premise),
        (Argument, Conclusion),
        (Argument, Inference),

        // A Syllogism has two Premises (major + minor) and one Conclusion —
        // captured at concept level as has-a Premise and has-a Conclusion.
        (Syllogism, Premise),
        (Syllogism, Conclusion),

        // A Hypothesis has Evidence (for or against it)
        (Hypothesis, Evidence),
    ],
}

/// Which tradition a Reasoning concept primarily belongs to.
#[derive(Debug, Clone)]
pub struct ReasoningTradition;

impl Quality for ReasoningTradition {
    type Individual = ReasoningConcept;
    type Value = &'static str;

    fn get(&self, c: &ReasoningConcept) -> Option<&'static str> {
        use ReasoningConcept as R;
        Some(match c {
            R::Deduction | R::Induction | R::Abduction => "peirce-1903",
            R::Analogy => "gentner-1983",
            R::Reasoning | R::Inference | R::Premise | R::Conclusion => "frege-1879",
            R::Argument | R::ValidArgument | R::SoundArgument | R::Syllogism => "aristotle",
            R::InferenceRule => "gentzen-1935",
            R::Hypothesis | R::Evidence => "peirce-1903",
        })
    }
}

impl Ontology for ReasoningOntology {
    type Cat = ReasoningCategory;
    type Qual = ReasoningTradition;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        crate::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod ontology_tests {
    use super::*;
    use crate::category::laws::assert_category_laws;
    use crate::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    // ---------- Standard suite: laws + validation ----------

    #[test]
    fn category_laws() {
        assert_category_laws::<ReasoningCategory>();
    }

    #[test]
    fn ontology_validates() {
        ReasoningOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    // ---------- Structural claims (hand-checked) ----------

    #[test]
    fn peircean_trichotomy_plus_analogy_are_reasoning_modes() {
        // Peirce (1903) names deduction / induction / abduction as the
        // three modes; Polya (1954) / Gentner (1983) add analogy. Each
        // is-a Reasoning.
        let modes = [
            ReasoningConcept::Deduction,
            ReasoningConcept::Induction,
            ReasoningConcept::Abduction,
            ReasoningConcept::Analogy,
        ];
        let subsumption_edges: Vec<_> = ReasoningCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ReasoningRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for mode in modes {
            assert!(
                subsumption_edges.contains(&(mode, ReasoningConcept::Reasoning)),
                "{:?} should be-a Reasoning",
                mode
            );
        }
    }

    #[test]
    fn sound_implies_valid() {
        // SoundArgument is-a ValidArgument — Aristotle's soundness
        // requires validity plus premise-truth.
        let subsumption_edges: Vec<_> = ReasoningCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ReasoningRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(subsumption_edges.contains(&(
            ReasoningConcept::SoundArgument,
            ReasoningConcept::ValidArgument
        )));
    }

    #[test]
    fn inference_has_premise_conclusion_rule() {
        // Frege (1879): an inference is essentially premise + rule → conclusion.
        let parthood: Vec<_> = ReasoningCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == ReasoningRelationKind::Parthood)
            .map(|m| (m.source(), m.target()))
            .collect();
        for part in [
            ReasoningConcept::Premise,
            ReasoningConcept::Conclusion,
            ReasoningConcept::InferenceRule,
        ] {
            assert!(
                // part→whole: each premise/conclusion/rule is PART of the Inference.
                parthood.contains(&(part, ReasoningConcept::Inference)),
                "Inference should have-a {:?}",
                part
            );
        }
    }

    #[test]
    fn every_concept_has_tradition() {
        // ReasoningTradition quality is total over ReasoningConcept.
        let tradition = ReasoningTradition;
        for concept in ReasoningConcept::variants() {
            assert!(
                tradition.get(&concept).is_some(),
                "{:?} should have a tradition",
                concept
            );
        }
    }

    // ---------- Property-based tests ----------

    fn arb_reasoning_concept() -> impl Strategy<Value = ReasoningConcept> {
        let variants = ReasoningConcept::variants();
        proptest::sample::select(variants)
    }

    proptest! {
        /// Every concept carries a literature tradition — Gruber (1993):
        /// formally-named relations, every concept is grounded in a source.
        #[test]
        fn prop_tradition_is_total(concept in arb_reasoning_concept()) {
            let tradition = ReasoningTradition;
            prop_assert!(tradition.get(&concept).is_some());
        }

        /// Every arrow carries per-instance meta with a non-empty name —
        /// Gruber (1993) KAS 5 "formally-named relations".
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for arrow in ReasoningCategory::morphisms() {
                let meta = arrow.meta();
                prop_assert!(!meta.name.as_str().is_empty(), "arrow meta.name must be non-empty");
            }
        }

        /// Every subsumption morphism hits a concept that's in the
        /// variant set — no dangling references.
        #[test]
        fn prop_subsumption_targets_valid_concepts(_seed in any::<u32>()) {
            let variants: Vec<_> = ReasoningConcept::variants();
            for m in ReasoningCategory::morphisms() {
                if m.kind() == ReasoningRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }

        /// Structural axioms verify successfully — catalog-emitted
        /// NoCyclesOnKind[Subsumption] / AntisymmetricOnKind[Subsumption]
        /// / NoCyclesOnKind[Parthood] must all hold for the Reasoning
        /// ontology's corpus.
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ReasoningOntology::axioms() {
                match axiom.verify() {
                    Ok(_) => {}
                    Err(c) => prop_assert!(
                        false,
                        "structural axiom failed: {}",
                        c.meta().name.as_str()
                    ),
                }
            }
        }
    }
}
