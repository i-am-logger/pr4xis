//! Inference rules — the canonical catalog.
//!
//! Every proof is built from inference rules. This ontology names the
//! rules themselves — the atomic moves that carry premises to conclusions.
//! Hilbert-style axiom systems, Gentzen's natural-deduction rules, and the
//! three modes of inference (deduction / induction / abduction) each
//! contribute a family of canonical rules.
//!
//! # Literature
//!
//! - **Hilbert & Ackermann (1928)** *Grundzüge der theoretischen Logik*.
//!   Hilbert-style axiom systems: modus ponens + a few axiom schemas.
//! - **Gentzen (1935)** *Untersuchungen über das logische Schließen*
//!   (Math. Z. 39: 176–210, 405–431). Natural deduction rules: for each
//!   connective, one introduction rule and one elimination rule.
//! - **Prawitz (1965)** *Natural Deduction* — systematic rule catalog.
//! - **Peirce (1878)** "Deduction, Induction, and Hypothesis"
//!   (Popular Science Monthly). Abduction as a distinct mode alongside
//!   deduction and induction.
//! - **Carnap (1950)** *Logical Foundations of Probability*. Induction as
//!   inference to a generalisation.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "InferenceRules",
    source: "Hilbert & Ackermann (1928) Grundzüge der theoretischen Logik; Gentzen (1935) natural deduction; Prawitz (1965) Natural Deduction; Peirce (1878) Deduction, Induction, and Hypothesis; Carnap (1950) Logical Foundations of Probability",
    being: AbstractObject,

    concepts: [
        // === Genus ===
        InferenceRule,

        // === Mode families (Peirce 1878) ===
        Deduction,
        Induction,
        Abduction,

        // === Classical Hilbert rules ===
        ModusPonens,
        ModusTollens,

        // === Gentzen conjunction rules ===
        ConjunctionIntroduction,
        ConjunctionElimination,

        // === Gentzen disjunction rules ===
        DisjunctionIntroduction,
        DisjunctionElimination,

        // === Gentzen implication rules ===
        ImplicationIntroduction,
        ImplicationElimination,

        // === Gentzen negation rules ===
        NegationIntroduction,
        NegationElimination,

        // === First-order quantifier rules ===
        UniversalGeneralization,
        UniversalInstantiation,
        ExistentialIntroduction,
        ExistentialElimination,

        // === Structural / meta rules ===
        CutRule,
        ReductioAdAbsurdum,
        DoubleNegationElimination,
        ExcludedMiddle,

        // === Inductive / abductive ===
        Generalisation,
        BestExplanation,
    ],

    labels: {
        InferenceRule: ("en", "Inference rule",
            "The genus concept: an atomic proof-building move carrying premises to a conclusion. Each rule has a canonical form and a justification theorem."),

        Deduction: ("en", "Deduction",
            "Peirce (1878): inference from rule + case to result. If all premises and the rule hold, the conclusion is guaranteed. Truth-preserving."),
        Induction: ("en", "Induction",
            "Peirce (1878) / Carnap (1950): inference from case + result to rule. Generalises from observed instances. Probability-preserving, not truth-preserving."),
        Abduction: ("en", "Abduction",
            "Peirce (1878): inference from rule + result to case — inference to the best explanation. Generates hypotheses. Neither truth-preserving nor probability-preserving alone."),

        ModusPonens: ("en", "Modus ponens",
            "Hilbert-Ackermann (1928): from A and A → B, conclude B. The canonical deductive rule; often the ONLY rule in Hilbert axiom systems."),
        ModusTollens: ("en", "Modus tollens",
            "Classical: from A → B and ¬B, conclude ¬A. The contrapositive of modus ponens; derivable in classical logic."),

        ConjunctionIntroduction: ("en", "Conjunction introduction (∧I)",
            "Gentzen (1935): from A and B, conclude A ∧ B. Introduces a conjunction from its two conjuncts."),
        ConjunctionElimination: ("en", "Conjunction elimination (∧E)",
            "Gentzen (1935): from A ∧ B, conclude A (or B). Extracts a conjunct. Two sub-rules (left/right) usually grouped."),

        DisjunctionIntroduction: ("en", "Disjunction introduction (∨I)",
            "Gentzen (1935): from A, conclude A ∨ B. Introduces a disjunction by weakening to either disjunct."),
        DisjunctionElimination: ("en", "Disjunction elimination (∨E)",
            "Gentzen (1935): from A ∨ B, A ⊢ C, and B ⊢ C, conclude C. Case analysis on a disjunction."),

        ImplicationIntroduction: ("en", "Implication introduction (→I)",
            "Gentzen (1935): from the sub-proof A ⊢ B, conclude A → B (discharging A). The proof-theoretic deduction theorem."),
        ImplicationElimination: ("en", "Implication elimination (→E)",
            "Gentzen's natural-deduction form of modus ponens: from A → B and A, conclude B."),

        NegationIntroduction: ("en", "Negation introduction (¬I)",
            "Gentzen (1935): from the sub-proof A ⊢ ⊥, conclude ¬A (discharging A). Proof by contradiction's constructive kernel."),
        NegationElimination: ("en", "Negation elimination (¬E)",
            "Gentzen (1935): from A and ¬A, conclude ⊥. The explosion rule's entry point; equivalent to modus ponens on ¬A = A → ⊥."),

        UniversalGeneralization: ("en", "Universal generalization (∀I)",
            "From a proof of φ(x) where x is fresh (not in hypotheses), conclude ∀x.φ(x). Gentzen 1935 for first-order logic."),
        UniversalInstantiation: ("en", "Universal instantiation (∀E)",
            "From ∀x.φ(x), conclude φ(t) for any term t. Substitution of a specific witness for the universal."),
        ExistentialIntroduction: ("en", "Existential introduction (∃I)",
            "From φ(t) for some term t, conclude ∃x.φ(x). Generalise to existence from a specific instance."),
        ExistentialElimination: ("en", "Existential elimination (∃E)",
            "From ∃x.φ(x) and the sub-proof φ(x) ⊢ ψ (x fresh), conclude ψ. Reasoning by a generic witness; discharges x."),

        CutRule: ("en", "Cut rule",
            "Gentzen (1935): from Γ ⊢ Δ, A and Γ', A ⊢ Δ', conclude Γ, Γ' ⊢ Δ, Δ'. Uses a lemma A without making it explicit. Eliminable via Gentzen's Hauptsatz."),
        ReductioAdAbsurdum: ("en", "Reductio ad absurdum",
            "Classical: from the sub-proof ¬A ⊢ ⊥, conclude A. Stronger than ¬I (intuitionistic): requires classical / double-negation elimination."),
        DoubleNegationElimination: ("en", "Double negation elimination",
            "Classical: from ¬¬A, conclude A. Distinguishes classical from intuitionistic logic."),
        ExcludedMiddle: ("en", "Law of excluded middle (LEM)",
            "Classical axiom: ⊢ A ∨ ¬A. Rejected in intuitionistic logic. Equivalent to ¬¬A → A and to reductio."),

        Generalisation: ("en", "Generalisation (inductive)",
            "Carnap (1950): from observed instances p(a_1), …, p(a_n), conclude ∀x.p(x). Non-monotonic. Probability-preserving."),
        BestExplanation: ("en", "Inference to the best explanation",
            "Peirce / Harman (1965): given observation O and candidate hypotheses {H_1, …}, infer the H_i that best explains O. The most common abductive schema."),
    },

    is_a: [
        // Mode families are inference rules.
        (Deduction, InferenceRule),
        (Induction, InferenceRule),
        (Abduction, InferenceRule),

        // Classical rules are deductive.
        (ModusPonens, Deduction),
        (ModusTollens, Deduction),
        (ImplicationElimination, ModusPonens),   // same rule, natural-deduction presentation
        (ConjunctionIntroduction, Deduction),
        (ConjunctionElimination, Deduction),
        (DisjunctionIntroduction, Deduction),
        (DisjunctionElimination, Deduction),
        (ImplicationIntroduction, Deduction),
        (NegationIntroduction, Deduction),
        (NegationElimination, Deduction),
        (UniversalGeneralization, Deduction),
        (UniversalInstantiation, Deduction),
        (ExistentialIntroduction, Deduction),
        (ExistentialElimination, Deduction),
        (CutRule, Deduction),
        (ReductioAdAbsurdum, Deduction),
        (DoubleNegationElimination, Deduction),
        (ExcludedMiddle, Deduction),

        // Inductive / abductive.
        (Generalisation, Induction),
        (BestExplanation, Abduction),
    ],

    opposes: [
        // The three modes contrast (but aren't mutually exclusive in use).
        (Deduction, Induction),
        (Deduction, Abduction),
        (Induction, Abduction),

        // Classical vs intuitionistic: LEM / double-negation distinguish them.
        (ExcludedMiddle, NegationIntroduction),
        (DoubleNegationElimination, NegationIntroduction),
        (ReductioAdAbsurdum, NegationIntroduction),
    ],
}

// -----------------------------------------------------------------------------
// Qualities
// -----------------------------------------------------------------------------

/// Which logical tradition the rule comes from.
#[derive(Debug, Clone)]
pub struct RuleOrigin;

impl Quality for RuleOrigin {
    type Individual = InferenceRulesConcept;
    type Value = &'static str;

    fn get(&self, c: &InferenceRulesConcept) -> Option<&'static str> {
        use InferenceRulesConcept as R;
        Some(match c {
            R::ModusPonens | R::ModusTollens => "hilbert-ackermann-1928",
            R::ConjunctionIntroduction
            | R::ConjunctionElimination
            | R::DisjunctionIntroduction
            | R::DisjunctionElimination
            | R::ImplicationIntroduction
            | R::ImplicationElimination
            | R::NegationIntroduction
            | R::NegationElimination
            | R::UniversalGeneralization
            | R::UniversalInstantiation
            | R::ExistentialIntroduction
            | R::ExistentialElimination
            | R::CutRule => "gentzen-1935",
            R::ReductioAdAbsurdum | R::DoubleNegationElimination | R::ExcludedMiddle => {
                "classical-logic"
            }
            R::Deduction | R::Induction | R::Abduction | R::InferenceRule => "peirce-1878",
            R::Generalisation => "carnap-1950",
            R::BestExplanation => "peirce-harman",
        })
    }
}

/// Whether the rule is truth-preserving (deductive), probability-preserving
/// (inductive), or hypothesis-generating (abductive).
#[derive(Debug, Clone)]
pub struct RuleMode;

impl Quality for RuleMode {
    type Individual = InferenceRulesConcept;
    type Value = &'static str;

    fn get(&self, c: &InferenceRulesConcept) -> Option<&'static str> {
        use InferenceRulesConcept as R;
        Some(match c {
            R::Deduction
            | R::ModusPonens
            | R::ModusTollens
            | R::ConjunctionIntroduction
            | R::ConjunctionElimination
            | R::DisjunctionIntroduction
            | R::DisjunctionElimination
            | R::ImplicationIntroduction
            | R::ImplicationElimination
            | R::NegationIntroduction
            | R::NegationElimination
            | R::UniversalGeneralization
            | R::UniversalInstantiation
            | R::ExistentialIntroduction
            | R::ExistentialElimination
            | R::CutRule
            | R::ReductioAdAbsurdum
            | R::DoubleNegationElimination
            | R::ExcludedMiddle => "truth-preserving",
            R::Induction | R::Generalisation => "probability-preserving",
            R::Abduction | R::BestExplanation => "hypothesis-generating",
            R::InferenceRule => "genus",
        })
    }
}

impl Ontology for InferenceRulesOntology {
    type Cat = InferenceRulesCategory;
    type Qual = RuleOrigin;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        InferenceRulesOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        InferenceRulesOntology::generated_domain_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<InferenceRulesCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        InferenceRulesOntology::validate().unwrap();
    }

    #[test]
    fn modus_ponens_is_deductive() {
        use pr4xis::ontology::reasoning::taxonomy;
        assert!(taxonomy::is_a::<InferenceRulesTaxonomy>(
            &InferenceRulesConcept::ModusPonens,
            &InferenceRulesConcept::Deduction,
        ));
    }

    #[test]
    fn three_modes_all_are_inference_rules() {
        use pr4xis::ontology::reasoning::taxonomy;
        for mode in [
            InferenceRulesConcept::Deduction,
            InferenceRulesConcept::Induction,
            InferenceRulesConcept::Abduction,
        ] {
            assert!(taxonomy::is_a::<InferenceRulesTaxonomy>(
                &mode,
                &InferenceRulesConcept::InferenceRule,
            ));
        }
    }
}
