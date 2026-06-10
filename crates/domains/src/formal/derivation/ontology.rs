//! Derivation — types of logical inference, proof components, and the
//! canonical proof-construction pipeline.
//!
//! This is a PURE-SCIENCE ontology of derivation — not an implementation
//! of a theorem prover. It formalises the reasoning that
//! ontology_diagnostics uses when constructing proof chains for axiom
//! verification.
//!
//! # Literature
//!
//! - **Gentzen (1935)** "Untersuchungen über das logische Schließen",
//!   *Mathematische Zeitschrift* 39 — natural deduction and sequent
//!   calculus; introduction/elimination rules per connective.
//! - **Prawitz (1965)** *Natural Deduction: A Proof-Theoretical Study*,
//!   Almqvist & Wiksell — normalisation theorem; canonical proof form.
//! - **Martin-Löf (1984)** *Intuitionistic Type Theory*, Bibliopolis —
//!   propositions-as-types; constructive proofs.
//! - **Peirce (1903)** *Pragmatism as a Principle and Method of Right
//!   Thinking* (Harvard Lectures) — abduction as inference to the best
//!   explanation; the third mode of reasoning beyond deduction and
//!   induction.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Derivation",
    source: "Gentzen (1935) Untersuchungen über das logische Schliessen, Mathematische Zeitschrift 39; Prawitz (1965) Natural Deduction; Martin-Löf (1984) Intuitionistic Type Theory; Peirce (1903) Harvard Lectures on Pragmatism",

    concepts: [
        // === Modes of inference ===
        Deduction,
        Induction,
        Abduction,
        Analogy,
        Composition,

        // === Components of a derivation ===
        Premise,
        Conclusion,
        InferenceRule,
        Evidence,
        Justification,
        ProofStep,

        // === Logical properties of a system ===
        Soundness,
        Completeness,
        Validity,
        Decidability,

        // === Abstract categories ===
        DerivationType,
        DerivationComponent,
        LogicalProperty,

        // === Pipeline stages (Gentzen 1935; Prawitz 1965) ===
        PremiseEstablishment,
        RuleApplication,
        IntermediateConclusion,
        ChainExtension,
        ValidityCheck,
        SoundnessVerification,
        ProofCompletion,
        KnowledgeExtension,
    ],

    labels: {
        Deduction: ("en", "Deduction",
            "Gentzen (1935): truth-preserving inference from premises to conclusion via natural-deduction rules — if premises true, conclusion true."),
        Induction: ("en", "Induction",
            "Generalisation from observed instances to a universal claim — ampliative; conclusion may be false even if all premises are true."),
        Abduction: ("en", "Abduction",
            "Peirce (1903): inference to the best explanation — given a surprising observation, propose the hypothesis that would best account for it."),
        Analogy: ("en", "Analogy",
            "Inference by structural similarity between two domains — suggestive, not guaranteed; breaks with new disanalogies."),
        Composition: ("en", "Composition",
            "Sequential composition of valid inferences — Gentzen (1935): cut and substitution preserve validity."),
        Premise: ("en", "Premise",
            "Gentzen (1935): a hypothesis (axiom, assumption, or given) from which inference begins."),
        Conclusion: ("en", "Conclusion",
            "Gentzen (1935): the proposition the derivation establishes from its premises."),
        InferenceRule: ("en", "Inference rule",
            "Gentzen (1935): a schematic transformation of sequents — e.g., modus ponens, AND-introduction."),
        Evidence: ("en", "Evidence",
            "Empirical or testimonial grounding for a premise — distinct from the inference itself."),
        Justification: ("en", "Justification",
            "Prawitz (1965): the explicit appeal to a rule or premise that warrants a proof step."),
        ProofStep: ("en", "Proof step",
            "Prawitz (1965): one inference within a derivation — a rule applied to premises yielding a conclusion."),
        Soundness: ("en", "Soundness",
            "Gentzen (1935): every theorem of the system is true in every model — provable ⇒ valid."),
        Completeness: ("en", "Completeness",
            "Gentzen (1935): every valid formula of the system is provable in it — valid ⇒ provable. Goedel's completeness theorem (1929) for first-order logic."),
        Validity: ("en", "Validity",
            "Truth in every model — a meta-logical property of formulas, complementary to provability."),
        Decidability: ("en", "Decidability",
            "Existence of a terminating algorithm that decides theoremhood — propositional logic is decidable; first-order logic is not."),
        DerivationType: ("en", "Derivation type",
            "Abstract category for modes of inference (Deduction, Induction, Abduction, Analogy, Composition)."),
        DerivationComponent: ("en", "Derivation component",
            "Abstract category for the structural pieces of a derivation (premises, conclusions, rules, justifications)."),
        LogicalProperty: ("en", "Logical property",
            "Abstract category for meta-logical properties of a derivation system (Soundness, Completeness, Validity, Decidability)."),

        PremiseEstablishment: ("en", "Premise establishment",
            "Pipeline stage 1: state the axioms, assumptions, and given facts. Gentzen (1935) §1."),
        RuleApplication: ("en", "Rule application",
            "Pipeline stage 2: apply an inference rule to premises. Gentzen (1935) §2."),
        IntermediateConclusion: ("en", "Intermediate conclusion",
            "Pipeline stage 3: record the proposition produced by a rule application."),
        ChainExtension: ("en", "Chain extension",
            "Pipeline stage 4: extend the proof chain with further steps. Prawitz (1965) — canonical-form normalisation."),
        ValidityCheck: ("en", "Validity check",
            "Pipeline stage 5: verify that each step matches its rule schema."),
        SoundnessVerification: ("en", "Soundness verification",
            "Pipeline stage 6: confirm the overall argument is sound (premises sound; every rule sound)."),
        ProofCompletion: ("en", "Proof completion",
            "Pipeline stage 7: declare the derivation closed when all open sub-goals are discharged."),
        KnowledgeExtension: ("en", "Knowledge extension",
            "Pipeline stage 8: integrate the proved conclusion into the knowledge base as a new axiom."),
    },

    is_a: [
        // Modes of inference ⊆ DerivationType.
        (Deduction, DerivationType),
        (Induction, DerivationType),
        (Abduction, DerivationType),
        (Analogy, DerivationType),
        (Composition, DerivationType),
        // Components ⊆ DerivationComponent.
        (Premise, DerivationComponent),
        (Conclusion, DerivationComponent),
        (InferenceRule, DerivationComponent),
        (Evidence, DerivationComponent),
        (Justification, DerivationComponent),
        (ProofStep, DerivationComponent),
        // Meta-logical properties ⊆ LogicalProperty.
        (Soundness, LogicalProperty),
        (Completeness, LogicalProperty),
        (Validity, LogicalProperty),
        (Decidability, LogicalProperty),
    ],

    causes: [
        // Canonical proof-construction pipeline. Gentzen (1935) §1–§2.
        (PremiseEstablishment, RuleApplication),
        (RuleApplication, IntermediateConclusion),
        (IntermediateConclusion, ChainExtension),
        (ChainExtension, ValidityCheck),
        (ValidityCheck, SoundnessVerification),
        (SoundnessVerification, ProofCompletion),
        (ProofCompletion, KnowledgeExtension),
    ],

    opposes: [
        // Deduction vs Abduction (Peirce 1903): truth-preserving vs
        // explanatory; certain vs plausible.
        (Deduction, Abduction),
        (Abduction, Deduction),
        // Soundness vs Completeness (Goedel 1929/31): "all proved are true"
        // vs "all true are provable" — dual meta-properties.
        (Soundness, Completeness),
        (Completeness, Soundness),
    ],
}

/// Quality: whether the inference mode is monotonic (adding premises never
/// invalidates conclusions). Gentzen (1935): structural rule of weakening
/// makes classical deduction monotonic. Peirce (1903): abduction is
/// non-monotonic — new evidence can defeat the best-explanation hypothesis.
#[derive(Debug, Clone)]
pub struct IsMonotonic;

impl Quality for IsMonotonic {
    type Individual = DerivationConcept;
    type Value = bool;

    fn get(&self, c: &DerivationConcept) -> Option<bool> {
        use DerivationConcept as D;
        match c {
            D::Deduction | D::Composition => Some(true),
            D::Induction | D::Abduction | D::Analogy => Some(false),
            _ => None,
        }
    }
}

/// Quality: whether the inference mode preserves truth from premises to
/// conclusion. Deduction by definition (Gentzen 1935); induction and
/// abduction are ampliative — conclusion goes beyond the premises.
#[derive(Debug, Clone)]
pub struct PreservesTruth;

impl Quality for PreservesTruth {
    type Individual = DerivationConcept;
    type Value = bool;

    fn get(&self, c: &DerivationConcept) -> Option<bool> {
        use DerivationConcept as D;
        match c {
            D::Deduction | D::Composition => Some(true),
            D::Induction | D::Abduction | D::Analogy => Some(false),
            _ => None,
        }
    }
}

/// Quality: whether the inference mode requires every premise to be
/// present. Deduction does (modus ponens needs both premises); abduction
/// works with incomplete evidence by design.
#[derive(Debug, Clone)]
pub struct RequiresAllPremises;

impl Quality for RequiresAllPremises {
    type Individual = DerivationConcept;
    type Value = bool;

    fn get(&self, c: &DerivationConcept) -> Option<bool> {
        use DerivationConcept as D;
        match c {
            D::Deduction | D::Composition => Some(true),
            D::Induction | D::Abduction | D::Analogy => Some(false),
            _ => None,
        }
    }
}

impl Ontology for DerivationOntology {
    type Cat = DerivationCategory;
    type Qual = IsMonotonic;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(DeductionMonotonicAbductionNot));
        axioms.push(Box::new(DeductionPreservesTruthInductionNot));
        axioms.push(Box::new(DeductionRequiresAllAbductionNot));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Domain axioms
// ---------------------------------------------------------------------------

/// Deduction is monotonic; abduction is not. Gentzen (1935) admits
/// weakening on the structural side of classical deduction; Peirce (1903)
/// abduction is defeasible.
pub struct DeductionMonotonicAbductionNot;

impl Axiom for DeductionMonotonicAbductionNot {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use DerivationConcept as D;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if IsMonotonic.get(&D::Deduction) == Some(true)
            && IsMonotonic.get(&D::Abduction) == Some(false)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DeductionMonotonicAbductionNot",
        "Deduction is monotonic but Abduction is not",
        "Gentzen (1935) Untersuchungen über das logische Schliessen; Peirce (1903) Harvard Lectures on Pragmatism"
    );
}

pr4xis::register_axiom!(
    DeductionMonotonicAbductionNot,
    "Gentzen (1935) Untersuchungen über das logische Schliessen; Peirce (1903) Harvard Lectures on Pragmatism"
);

/// Deduction preserves truth; induction is ampliative and does not.
pub struct DeductionPreservesTruthInductionNot;

impl Axiom for DeductionPreservesTruthInductionNot {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use DerivationConcept as D;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if PreservesTruth.get(&D::Deduction) == Some(true)
            && PreservesTruth.get(&D::Induction) == Some(false)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DeductionPreservesTruthInductionNot",
        "Deduction preserves truth from premises to conclusion; Induction does not",
        "Gentzen (1935) Untersuchungen über das logische Schliessen, Mathematische Zeitschrift 39"
    );
}

pr4xis::register_axiom!(
    DeductionPreservesTruthInductionNot,
    "Gentzen (1935) Untersuchungen über das logische Schliessen, Mathematische Zeitschrift 39"
);

/// Deduction requires every premise; abduction works with incomplete
/// evidence (Peirce 1903 — that is its function).
pub struct DeductionRequiresAllAbductionNot;

impl Axiom for DeductionRequiresAllAbductionNot {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use DerivationConcept as D;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if RequiresAllPremises.get(&D::Deduction) == Some(true)
            && RequiresAllPremises.get(&D::Abduction) == Some(false)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "DeductionRequiresAllAbductionNot",
        "Deduction requires all premises; Abduction works with incomplete evidence",
        "Peirce (1903) Harvard Lectures on Pragmatism"
    );
}

pr4xis::register_axiom!(
    DeductionRequiresAllAbductionNot,
    "Peirce (1903) Harvard Lectures on Pragmatism"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category, FinitelyGenerated};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<DerivationCategory>();
    }

    #[test]
    fn ontology_validates() {
        DerivationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn pipeline_stages_form_causal_chain() {
        use DerivationConcept as D;
        let causation: Vec<_> = DerivationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == DerivationRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        for edge in [
            (D::PremiseEstablishment, D::RuleApplication),
            (D::RuleApplication, D::IntermediateConclusion),
            (D::IntermediateConclusion, D::ChainExtension),
            (D::ChainExtension, D::ValidityCheck),
            (D::ValidityCheck, D::SoundnessVerification),
            (D::SoundnessVerification, D::ProofCompletion),
            (D::ProofCompletion, D::KnowledgeExtension),
        ] {
            assert!(causation.contains(&edge));
        }
    }

    #[test]
    fn premise_transitively_reaches_knowledge_extension() {
        let causation: Vec<_> = DerivationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == DerivationRelationKind::Causation)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(causation.contains(&(
            DerivationConcept::PremiseEstablishment,
            DerivationConcept::KnowledgeExtension
        )));
    }

    #[test]
    fn modes_subsume_derivation_type() {
        use DerivationConcept as D;
        let sub: Vec<_> = DerivationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == DerivationRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for mode in [
            D::Deduction,
            D::Induction,
            D::Abduction,
            D::Analogy,
            D::Composition,
        ] {
            assert!(sub.contains(&(mode, D::DerivationType)));
        }
    }

    #[test]
    fn deduction_opposes_abduction() {
        let opp: Vec<_> = DerivationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == DerivationRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(DerivationConcept::Deduction, DerivationConcept::Abduction)));
    }

    #[test]
    fn soundness_opposes_completeness() {
        let opp: Vec<_> = DerivationCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == DerivationRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(
            DerivationConcept::Soundness,
            DerivationConcept::Completeness
        )));
    }

    #[test]
    fn deduction_monotonic_abduction_not_holds() {
        assert!(DeductionMonotonicAbductionNot.verify().is_ok());
    }

    #[test]
    fn deduction_preserves_truth_induction_not_holds() {
        assert!(DeductionPreservesTruthInductionNot.verify().is_ok());
    }

    #[test]
    fn deduction_requires_all_abduction_not_holds() {
        assert!(DeductionRequiresAllAbductionNot.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = DerivationConcept> {
        proptest::sample::select(DerivationConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in DerivationCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in DerivationOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }

        #[test]
        fn prop_monotonicity_total_on_modes(c in arb_concept()) {
            use DerivationConcept as D;
            let v = IsMonotonic.get(&c);
            let is_mode = matches!(c,
                D::Deduction | D::Induction | D::Abduction | D::Analogy | D::Composition
            );
            prop_assert_eq!(v.is_some(), is_mode);
        }

        #[test]
        fn prop_opposition_is_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = DerivationCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == DerivationRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} -> {:?} but not back", a, b);
            }
        }

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = DerivationConcept::variants();
            for m in DerivationCategory::morphisms() {
                if m.kind() == DerivationRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }
    }
}
