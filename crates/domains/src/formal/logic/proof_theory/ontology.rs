//! Proof theory — Gentzen / Prawitz structural proof vocabulary.
//!
//! Proof theory studies proofs as *mathematical objects in their own right*:
//! syntactic structures with compositional laws, normalisation properties,
//! and sub-proof relations. This ontology names the canonical vocabulary
//! from four overlapping traditions:
//!
//! 1. **Gentzen (1935)** — sequent calculus + natural deduction as two
//!    presentations of the same proof theory. Introduced the cut rule and
//!    the elimination theorem.
//! 2. **Prawitz (1965)** — normalisation of natural-deduction proofs;
//!    cut-elimination's intuitionistic variant.
//! 3. **Troelstra & Schwichtenberg (2000)** *Basic Proof Theory*
//!    (Cambridge 2nd ed.) — the modern textbook vocabulary.
//! 4. **Girard, Lafont & Taylor (1989)** *Proofs and Types* — Curry-Howard
//!    with full proof-theoretic development.
//!
//! The atemporal view of [`formal::logic::derivation::Derivation`].

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "ProofTheory",
    source: "Gentzen (1935) Untersuchungen über das logische Schließen; Prawitz (1965) Natural Deduction; Troelstra & Schwichtenberg (2000) Basic Proof Theory; Girard-Lafont-Taylor (1989) Proofs and Types",
    being: AbstractObject,

    concepts: [
        // === Core sequent calculus (Gentzen 1935) ===
        Sequent,
        Premise,
        Conclusion,
        Antecedent,
        Succedent,

        // === Proof objects ===
        Proof,
        ProofTree,
        SubProof,
        Axiom,
        Theorem,
        Lemma,
        Conjecture,
        Counterexample,

        // === Assumption handling ===
        Hypothesis,
        Assumption,
        Discharge,

        // === Structural rules (Gentzen, §5) ===
        Weakening,
        Contraction,
        Exchange,

        // === Normalisation (Prawitz 1965) ===
        Cut,
        CutElimination,
        Normalisation,
        NormalForm,
        Redex,

        // === Proof-theoretic strength ===
        Consistency,
        Completeness,
        Decidability,
    ],

    labels: {
        Sequent: ("en", "Sequent",
            "Gentzen (1935): a statement Γ ⊢ Δ asserting that the conjunction of formulas in Γ (antecedent) entails the disjunction of formulas in Δ (succedent). The atomic unit of sequent calculus."),
        Premise: ("en", "Premise",
            "A formula above the inference line — a sub-proof whose conclusion is used to derive the current step's conclusion. Prawitz (1965) §I.2."),
        Conclusion: ("en", "Conclusion",
            "The formula below the inference line — what the current step derives. A proof's overall conclusion is its root formula."),
        Antecedent: ("en", "Antecedent",
            "The left-hand side of a sequent — the hypotheses or assumptions being made (Γ in Γ ⊢ Δ)."),
        Succedent: ("en", "Succedent",
            "The right-hand side of a sequent — the disjunction of conclusions being claimed (Δ in Γ ⊢ Δ)."),

        Proof: ("en", "Proof",
            "A derivation whose witnesses establish logical validity. The atemporal view of a derivation. Lambek (1968): a morphism in the deductive category. Maps to formal::logic::derivation::Derivation via the specialisation-functor."),
        ProofTree: ("en", "Proof tree",
            "Gentzen: the tree-shaped presentation of a proof — axioms at the leaves, inference steps at internal nodes, the proved formula at the root."),
        SubProof: ("en", "Sub-proof",
            "A proof occurring as a premise within a larger proof — the immediate derivations of the premises of an inference step."),
        Axiom: ("en", "Axiom",
            "A statement posited without proof — a leaf of every proof tree that uses it. Distinct from a theorem (which IS proved). Aristotle, Posterior Analytics: the ἀξιώματα / indemonstrables."),
        Theorem: ("en", "Theorem",
            "A statement for which a proof has been constructed. Every theorem's proof is a Proof-object; theorems and their proofs compose."),
        Lemma: ("en", "Lemma",
            "An intermediate theorem proved on the way to a larger result — a reusable sub-proof. Methodologically subordinate but proof-theoretically identical to a Theorem."),
        Conjecture: ("en", "Conjecture",
            "A proposition believed but not yet proved nor refuted. Pending status in the proof-theoretic vocabulary."),
        Counterexample: ("en", "Counterexample",
            "Evidence that refutes a conjecture — a specific case where the proposition fails. In classical logic, equivalent to a proof of the proposition's negation."),

        Hypothesis: ("en", "Hypothesis",
            "An assumption temporarily introduced within a sub-proof, to be discharged by an inference rule (e.g. implication-introduction). Prawitz (1965) §I.2."),
        Assumption: ("en", "Assumption",
            "A synonym for Hypothesis in natural-deduction vocabulary; a starting datum of a sub-derivation."),
        Discharge: ("en", "Discharge",
            "The elimination of a hypothesis by an inference rule — the formal move by which a sub-proof's open assumption becomes closed. E.g. implication-introduction discharges the antecedent."),

        Weakening: ("en", "Weakening",
            "Gentzen (1935) §5 structural rule: from Γ ⊢ Δ, derive Γ, A ⊢ Δ — adding an unused hypothesis. Restricted or absent in substructural logics (Girard linear logic)."),
        Contraction: ("en", "Contraction",
            "Gentzen (1935) §5 structural rule: from Γ, A, A ⊢ Δ, derive Γ, A ⊢ Δ — collapsing duplicated hypotheses. Restricted in substructural logics."),
        Exchange: ("en", "Exchange",
            "Gentzen (1935) §5 structural rule: hypotheses can be reordered. Absent in ordered logics."),

        Cut: ("en", "Cut",
            "Gentzen (1935): the inference rule that from Γ ⊢ Δ, A and Γ', A ⊢ Δ', derives Γ, Γ' ⊢ Δ, Δ' — uses a lemma without making it explicit. The rule whose elimination is the centre of proof theory."),
        CutElimination: ("en", "Cut elimination",
            "Gentzen's Hauptsatz (1935): every sequent-calculus proof can be transformed into a cut-free proof. Equivalently for natural deduction: every proof can be normalised (Prawitz 1965)."),
        Normalisation: ("en", "Normalisation",
            "Prawitz (1965): the process of rewriting a proof to remove redexes, ultimately yielding a proof in Normal Form. The proof-theoretic analogue of computation (Curry-Howard)."),
        NormalForm: ("en", "Normal form",
            "A proof with no redexes — all introductions precede all eliminations along every branch. The canonical presentation of a proof's content."),
        Redex: ("en", "Redex (proof-theoretic)",
            "A detour in a proof: an introduction of a connective immediately followed by its elimination. The target of normalisation. Prawitz (1965) §II.3."),

        Consistency: ("en", "Consistency",
            "A theory is consistent iff it does not prove a contradiction (⊥). Equivalently: there is some formula it does not prove. Gentzen proved PA's consistency via cut-elimination + transfinite induction (1936)."),
        Completeness: ("en", "Completeness",
            "A proof system is complete for a semantics iff every semantically-valid formula is provable. Gödel (1930) proved first-order logic complete."),
        Decidability: ("en", "Decidability",
            "A theory is decidable iff there is an algorithm that, for every formula, determines whether it is provable. Propositional logic is decidable; first-order logic is not (Church 1936)."),
    },

    is_a: [
        // Proof-object hierarchy
        (Theorem, Proof),
        (Lemma, Theorem),
        (Axiom, Proof),           // An axiom is its own trivial proof

        // Assumption variants
        (Assumption, Hypothesis),

        // Structural rules are inference-step specialisations
        (Weakening, Sequent),
        (Contraction, Sequent),
        (Exchange, Sequent),

        // Normalisation produces normal forms
        (NormalForm, Proof),
    ],

    has_a: [
        // A sequent decomposes into antecedent and succedent
        (Sequent, Antecedent),
        (Sequent, Succedent),

        // A proof has premises and a conclusion
        (Proof, Premise),
        (Proof, Conclusion),

        // Proof trees are made of sub-proofs
        (ProofTree, SubProof),

        // Hypotheses get discharged during a proof
        (Proof, Hypothesis),
        (Proof, Discharge),

        // Redexes live inside proofs; normalisation eliminates them
        (Proof, Redex),
    ],

    opposes: [
        // Conjecture vs its counterexample — the proof-theoretic refutation duality.
        (Conjecture, Counterexample),

        // Theorem vs Axiom — proved vs posited.
        (Theorem, Axiom),

        // Cut vs CutElimination — the rule vs its removal.
        (Cut, CutElimination),
    ],
}

// -----------------------------------------------------------------------------
// Qualities
// -----------------------------------------------------------------------------

/// Which proof-theoretic tradition introduces each concept.
#[derive(Debug, Clone)]
pub struct ProofTheoryTradition;

impl Quality for ProofTheoryTradition {
    type Individual = ProofTheoryConcept;
    type Value = &'static str;

    fn get(&self, c: &ProofTheoryConcept) -> Option<&'static str> {
        use ProofTheoryConcept as P;
        Some(match c {
            P::Sequent
            | P::Antecedent
            | P::Succedent
            | P::Weakening
            | P::Contraction
            | P::Exchange
            | P::Cut
            | P::CutElimination => "gentzen-1935",
            P::Normalisation | P::NormalForm | P::Redex | P::Hypothesis | P::Discharge => {
                "prawitz-1965"
            }
            P::Proof
            | P::ProofTree
            | P::SubProof
            | P::Theorem
            | P::Lemma
            | P::Conjecture
            | P::Counterexample
            | P::Premise
            | P::Conclusion
            | P::Assumption => "troelstra-schwichtenberg-2000",
            P::Axiom | P::Consistency | P::Completeness | P::Decidability => "hilbert-godel",
        })
    }
}

impl Ontology for ProofTheoryOntology {
    type Cat = ProofTheoryCategory;
    type Qual = ProofTheoryTradition;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        ProofTheoryOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        ProofTheoryOntology::generated_domain_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<ProofTheoryCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        ProofTheoryOntology::validate().unwrap();
    }

    #[test]
    fn theorems_are_proofs() {
        use pr4xis::ontology::reasoning::taxonomy;
        assert!(taxonomy::is_a::<ProofTheoryTaxonomy>(
            &ProofTheoryConcept::Theorem,
            &ProofTheoryConcept::Proof,
        ));
    }

    #[test]
    fn lemma_is_a_theorem() {
        use pr4xis::ontology::reasoning::taxonomy;
        assert!(taxonomy::is_a::<ProofTheoryTaxonomy>(
            &ProofTheoryConcept::Lemma,
            &ProofTheoryConcept::Theorem,
        ));
    }
}
