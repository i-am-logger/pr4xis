//! Propositional logic — the substrate ontology grounding
//! `logic/composition.rs`, `logic/propositional.rs`, and
//! `logic/truth_table.rs`.
//!
//! # Why this ontology lives in core
//!
//! Core has a propositional-logic subsystem: `Proposition` trait,
//! `AllOf` / `AnyOf` / `Not` / `Implies` composition structures,
//! `Connective` enum (AND/OR/NOT/IMPL/IFF/XOR/NAND/NOR), truth tables,
//! and classical-theorem checkers (de Morgan, double negation, modus
//! ponens, contrapositive, excluded middle, non-contradiction, Sheffer
//! completeness). Per the substrate-grounding principle, these Rust
//! constructs must be grounded in a named concept vocabulary.
//!
//! # Literature
//!
//! - **Boole (1854)** *An Investigation of the Laws of Thought* —
//!   algebra of logic; foundational propositional calculus.
//! - **Frege (1879)** *Begriffsschrift* — formal connectives and
//!   derivation.
//! - **Russell & Whitehead (1910–13)** *Principia Mathematica* —
//!   comprehensive classical propositional and predicate logic.
//! - **Post (1921)** "Introduction to a General Theory of Elementary
//!   Propositions" — truth tables, completeness for propositional logic.
//! - **Sheffer (1913)** "A Set of Five Independent Postulates..." —
//!   NAND as functionally-complete connective.
//! - **Aristotle** *Metaphysics* Γ — excluded middle, non-contradiction.
//! - **Tarski (1936)** "Der Wahrheitsbegriff in den formalisierten
//!   Sprachen" — semantic truth.
//! - **Kleene (1952)** *Introduction to Metamathematics* — systematic
//!   modern treatment.

use crate as pr4xis;
use crate::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "PropositionalLogic",
    source: "Boole (1854); Frege (1879); Russell & Whitehead (1910–13); Post (1921); Sheffer (1913); Aristotle Metaphysics Γ; Tarski (1936); Kleene (1952)",

    concepts: [
        // === Umbrella ===
        Proposition,
        Formula,
        Atom,
        TruthValue,

        // === Connectives (is-a Connective) ===
        Connective,
        Conjunction,
        Disjunction,
        Negation,
        Implication,
        Biconditional,
        ExclusiveOr,
        NAND,
        NOR,

        // === Semantic status (is-a Formula) ===
        Tautology,
        Contradiction,
        Satisfiable,
        Valid,

        // === Truth-table structure ===
        TruthTable,
        Row,
        Assignment,

        // === Measurables (composition.rs) ===
        Measurable,
        Comparison,
        Threshold,

        // === Classical theorems (is-a Tautology) ===
        DeMorgansLaws,
        DoubleNegation,
        ModusPonens,
        Contrapositive,
        ExcludedMiddle,
        NonContradiction,
        ShefferCompleteness,
    ],

    labels: {
        Proposition: ("en", "Proposition",
            "Anything that has a truth value. Boole (1854); Frege (1879) — the atomic unit of logical discourse."),
        Formula: ("en", "Formula",
            "A compound proposition built from atoms via connectives. Post (1921)."),
        Atom: ("en", "Atomic proposition",
            "A proposition with no internal logical structure — a propositional variable. Russell & Whitehead (1910) *PM* *1."),
        TruthValue: ("en", "Truth value",
            "The semantic value of a proposition — True or False. Frege (1892) 'Über Sinn und Bedeutung' — propositions denote truth values."),

        Connective: ("en", "Connective",
            "A function that combines propositions into compound formulae. Boole (1854); Post (1921)."),
        Conjunction: ("en", "Conjunction (∧, AND)",
            "The binary connective true iff both operands are true. Boole's logical product."),
        Disjunction: ("en", "Disjunction (∨, OR)",
            "The binary connective true iff at least one operand is true (inclusive). Boole's logical sum."),
        Negation: ("en", "Negation (¬, NOT)",
            "The unary connective that flips truth value. Aristotle *De Interpretatione*."),
        Implication: ("en", "Implication (→)",
            "The binary connective false iff the antecedent is true and the consequent is false; material conditional. Frege (1879) §5."),
        Biconditional: ("en", "Biconditional (↔, IFF)",
            "The binary connective true iff both operands have the same truth value."),
        ExclusiveOr: ("en", "Exclusive disjunction (⊕, XOR)",
            "The binary connective true iff exactly one operand is true."),
        NAND: ("en", "NAND (Sheffer stroke, ↑)",
            "Negated conjunction — Sheffer (1913) showed it is functionally complete (all other connectives definable from NAND)."),
        NOR: ("en", "NOR (Peirce arrow, ↓)",
            "Negated disjunction — also functionally complete (Peirce 1880). Dual to NAND."),

        Tautology: ("en", "Tautology",
            "A formula true under every truth assignment. Wittgenstein (1921) *Tractatus* 4.46; Post (1921)."),
        Contradiction: ("en", "Contradiction",
            "A formula false under every truth assignment — dual to tautology. Aristotle *Metaphysics* Γ.3."),
        Satisfiable: ("en", "Satisfiable",
            "A formula true under at least one truth assignment."),
        Valid: ("en", "Valid argument form",
            "An argument form whose conclusion is a tautological consequence of its premises. Tarski (1936) semantic consequence."),

        TruthTable: ("en", "Truth table",
            "Post (1921): the finite tabulation of a formula's truth value under every possible assignment of its atoms."),
        Row: ("en", "Truth-table row",
            "A single assignment-to-result entry in a truth table."),
        Assignment: ("en", "Truth assignment",
            "A mapping from atomic propositions to truth values. Tarski (1936) — the ground of semantic evaluation."),

        Measurable: ("en", "Measurable quantity",
            "A context-dependent quantity that can be compared against a threshold (pr4xis-specific composition primitive)."),
        Comparison: ("en", "Comparison",
            "A relation between two measurables and a comparison operator (< ≤ = ≠ ≥ >)."),
        Threshold: ("en", "Threshold",
            "A cut-off value for a measurable, yielding a proposition (above/below)."),

        DeMorgansLaws: ("en", "De Morgan's laws",
            "¬(A ∧ B) ≡ (¬A ∨ ¬B) and ¬(A ∨ B) ≡ (¬A ∧ ¬B). Attributed to De Morgan (1847)."),
        DoubleNegation: ("en", "Double negation",
            "¬¬A ≡ A (classical; constructively, only one direction). Classical since Aristotle."),
        ModusPonens: ("en", "Modus ponens",
            "From A and (A → B), derive B. The canonical inference rule. Stoic logic; Frege (1879). See also `reasoning::InferenceRule` and `formal::logic::inference_rules::ModusPonens`."),
        Contrapositive: ("en", "Contrapositive",
            "(A → B) ≡ (¬B → ¬A). Classical equivalence."),
        ExcludedMiddle: ("en", "Excluded middle (tertium non datur)",
            "A ∨ ¬A — every proposition is either true or false, no third option. Aristotle *Metaphysics* Γ.7."),
        NonContradiction: ("en", "Non-contradiction",
            "¬(A ∧ ¬A) — no proposition is both true and false. Aristotle *Metaphysics* Γ.3 — 'the most certain of all principles'."),
        ShefferCompleteness: ("en", "Sheffer functional completeness",
            "Every propositional connective is definable in terms of NAND alone (equivalently NOR). Sheffer (1913)."),
    },

    is_a: [
        // Specific connectives are Connectives
        (Conjunction, Connective),
        (Disjunction, Connective),
        (Negation, Connective),
        (Implication, Connective),
        (Biconditional, Connective),
        (ExclusiveOr, Connective),
        (NAND, Connective),
        (NOR, Connective),

        // Atoms and Formulas are Propositions
        (Atom, Proposition),
        (Formula, Proposition),

        // Semantic-status specialisations are Formulas
        (Tautology, Formula),
        (Contradiction, Formula),
        (Satisfiable, Formula),
        (Valid, Formula),

        // Classical theorems are Tautologies
        (DeMorgansLaws, Tautology),
        (DoubleNegation, Tautology),
        (ModusPonens, Tautology),
        (Contrapositive, Tautology),
        (ExcludedMiddle, Tautology),
        (NonContradiction, Tautology),
    ],

    has_a: [
        // A Formula is built from Atoms via Connectives
        (Formula, Atom),
        (Formula, Connective),

        // A TruthTable has Rows, each with an Assignment
        (TruthTable, Row),
        (Row, Assignment),
        (Assignment, TruthValue),
        (Assignment, Atom),

        // A Comparison has Measurables and a CompareOp
        (Comparison, Measurable),

        // A Threshold has a Measurable
        (Threshold, Measurable),
    ],

    opposes: [
        // Tautology vs Contradiction — dual semantic statuses.
        (Tautology, Contradiction),
        (Contradiction, Tautology),

        // Conjunction vs Disjunction — De Morgan duals.
        (Conjunction, Disjunction),
        (Disjunction, Conjunction),

        // NAND vs NOR — De Morgan duals.
        (NAND, NOR),
        (NOR, NAND),
    ],
}

/// Which tradition primarily introduces each propositional-logic concept.
#[derive(Debug, Clone)]
pub struct PropositionalTradition;

impl Quality for PropositionalTradition {
    type Individual = PropositionalLogicConcept;
    type Value = &'static str;

    fn get(&self, c: &PropositionalLogicConcept) -> Option<&'static str> {
        use PropositionalLogicConcept as P;
        Some(match c {
            P::Proposition | P::Formula | P::Atom | P::TruthValue => "frege-1879",
            P::Connective
            | P::Conjunction
            | P::Disjunction
            | P::Negation
            | P::Implication
            | P::Biconditional
            | P::ExclusiveOr => "boole-1854",
            P::NAND | P::NOR | P::ShefferCompleteness => "sheffer-1913",
            P::Tautology | P::Satisfiable | P::Valid => "post-1921",
            P::Contradiction | P::ExcludedMiddle | P::NonContradiction => "aristotle",
            P::TruthTable | P::Row | P::Assignment => "post-1921",
            P::Measurable | P::Comparison | P::Threshold => "pr4xis-specific",
            P::DeMorgansLaws => "de-morgan-1847",
            P::DoubleNegation | P::Contrapositive => "classical",
            P::ModusPonens => "stoic-frege",
        })
    }
}

impl Ontology for PropositionalLogicOntology {
    type Cat = PropositionalLogicCategory;
    type Qual = PropositionalTradition;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        crate::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::laws::assert_category_laws;
    use crate::category::{Arrow, Category, Concept};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<PropositionalLogicCategory>();
    }

    #[test]
    fn ontology_validates() {
        PropositionalLogicOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[test]
    fn eight_connectives_are_connectives() {
        let connectives = [
            PropositionalLogicConcept::Conjunction,
            PropositionalLogicConcept::Disjunction,
            PropositionalLogicConcept::Negation,
            PropositionalLogicConcept::Implication,
            PropositionalLogicConcept::Biconditional,
            PropositionalLogicConcept::ExclusiveOr,
            PropositionalLogicConcept::NAND,
            PropositionalLogicConcept::NOR,
        ];
        let sub: Vec<_> = PropositionalLogicCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PropositionalLogicRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for c in connectives {
            assert!(
                sub.contains(&(c, PropositionalLogicConcept::Connective)),
                "{:?} should be-a Connective",
                c
            );
        }
    }

    #[test]
    fn classical_theorems_are_tautologies() {
        let theorems = [
            PropositionalLogicConcept::DeMorgansLaws,
            PropositionalLogicConcept::DoubleNegation,
            PropositionalLogicConcept::ModusPonens,
            PropositionalLogicConcept::Contrapositive,
            PropositionalLogicConcept::ExcludedMiddle,
            PropositionalLogicConcept::NonContradiction,
        ];
        let sub: Vec<_> = PropositionalLogicCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PropositionalLogicRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for t in theorems {
            assert!(
                sub.contains(&(t, PropositionalLogicConcept::Tautology)),
                "{:?} should be-a Tautology",
                t
            );
        }
    }

    #[test]
    fn tautology_opposes_contradiction() {
        let opp: Vec<_> = PropositionalLogicCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == PropositionalLogicRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opp.contains(&(
            PropositionalLogicConcept::Tautology,
            PropositionalLogicConcept::Contradiction
        )));
        assert!(opp.contains(&(
            PropositionalLogicConcept::Contradiction,
            PropositionalLogicConcept::Tautology
        )));
    }

    #[test]
    fn every_concept_has_tradition() {
        let q = PropositionalTradition;
        for c in PropositionalLogicConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing tradition", c);
        }
    }

    fn arb_concept() -> impl Strategy<Value = PropositionalLogicConcept> {
        proptest::sample::select(PropositionalLogicConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_tradition_total(c in arb_concept()) {
            prop_assert!(PropositionalTradition.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in PropositionalLogicCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in PropositionalLogicOntology::axioms() {
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

        #[test]
        fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
            let variants: Vec<_> = PropositionalLogicConcept::variants();
            for m in PropositionalLogicCategory::morphisms() {
                if m.kind() == PropositionalLogicRelationKind::Subsumption {
                    prop_assert!(variants.contains(&m.source()));
                    prop_assert!(variants.contains(&m.target()));
                }
            }
        }
    }
}
