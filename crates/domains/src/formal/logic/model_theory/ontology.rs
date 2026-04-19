//! Model theory — Tarski / Chang-Keisler / Hodges semantic vocabulary.
//!
//! Proof theory studies syntactic derivations; model theory studies
//! *truth in a structure*. A sentence is a consequence of a theory
//! (Tarski 1936) iff every model of the theory is also a model of the
//! sentence. Model-theoretic validity is what `Axiom::holds() -> bool`
//! currently checks — a ModelCheck witness in the [`Derivation`] framework.
//!
//! # Literature
//!
//! - **Tarski (1936)** "On the Concept of Logical Consequence"
//!   (originally *O pojęciu wynikania logicznego*; translated in Tarski,
//!   *Logic, Semantics, Metamathematics*, 1956). The founding paper of
//!   model-theoretic semantics.
//! - **Tarski (1933)** "The Concept of Truth in Formalized Languages"
//!   (*Pojęcie prawdy w językach nauk dedukcyjnych*). The T-schema.
//! - **Chang & Keisler (1990)** *Model Theory* (3rd ed., North-Holland).
//!   The canonical graduate textbook.
//! - **Hodges (1997)** *A Shorter Model Theory* (Cambridge). Compact
//!   modern treatment.
//! - **Gödel (1930)** completeness theorem: first-order logic's proof
//!   system is complete for its semantics.
//! - **Gödel (1931)** incompleteness theorems: PA is incomplete.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "ModelTheory",
    source: "Tarski (1936) On the Concept of Logical Consequence; Tarski (1933) The Concept of Truth; Chang & Keisler (1990) Model Theory; Hodges (1997) A Shorter Model Theory; Gödel (1930, 1931)",
    being: AbstractObject,

    concepts: [
        // === Core structures ===
        Model,
        Structure,
        Interpretation,
        Universe,
        Signature,
        Assignment,

        // === Relations to sentences ===
        Satisfaction,
        Truth,
        Falsity,
        Validity,
        Satisfiability,

        // === Consequence (Tarski 1936) ===
        LogicalConsequence,
        Entailment,
        Equivalence,

        // === Theories ===
        Theory,
        AxiomSet,
        ElementaryEquivalence,

        // === Meta-theoretic properties ===
        Soundness,
        Completeness,
        Consistency,
        Compactness,
        LowenheimSkolem,
    ],

    labels: {
        Model: ("en", "Model",
            "Tarski (1936): a structure in which all sentences of a given theory are true. A witness that the theory is satisfiable. Chang-Keisler §1.1."),
        Structure: ("en", "Structure",
            "A set (universe) together with interpretations of the symbols of a signature. The domain of model-theoretic evaluation. Hodges (1997) §1.2."),
        Interpretation: ("en", "Interpretation",
            "An assignment of meanings to the non-logical symbols (constants, functions, relations) of a signature in terms of a structure's universe. Tarski 1933."),
        Universe: ("en", "Universe",
            "The carrier set of a structure — the domain over which quantifiers range. Can be finite (small-model semantics) or infinite."),
        Signature: ("en", "Signature",
            "The list of non-logical symbols (with arities) that a theory or structure uses. Pr4xis ontologies have a signature — their concept enum + relation kinds."),
        Assignment: ("en", "Variable assignment",
            "A function from variables to elements of the universe, used to evaluate open formulas. Tarski's recursive truth definition quantifies over assignments."),

        Satisfaction: ("en", "Satisfaction",
            "The relation M ⊨ φ, read 'M satisfies φ' or 'φ is true in M'. The central relation of model theory, defined recursively on the structure of φ (Tarski 1933)."),
        Truth: ("en", "Truth",
            "A sentence φ is true in structure M iff M ⊨ φ. The minimal instance of satisfaction — for closed formulas, independent of assignment."),
        Falsity: ("en", "Falsity",
            "A sentence φ is false in structure M iff M ⊨ ¬φ. Equivalently, not (M ⊨ φ)."),
        Validity: ("en", "Validity",
            "A sentence is valid iff it is true in every structure. Classical logic: the tautologies. First-order logic: the theorems of pure logic."),
        Satisfiability: ("en", "Satisfiability",
            "A sentence is satisfiable iff it is true in some structure. SAT for propositional logic is NP-complete; first-order satisfiability is r.e.-complete."),

        LogicalConsequence: ("en", "Logical consequence",
            "Tarski (1936): Γ ⊨ φ iff every structure satisfying all sentences in Γ also satisfies φ. The semantic entailment relation. Contrasts with syntactic derivability Γ ⊢ φ (which proof theory provides)."),
        Entailment: ("en", "Entailment",
            "Synonym for Logical Consequence: Γ entails φ iff Γ ⊨ φ. Chang-Keisler §2.1."),
        Equivalence: ("en", "Logical equivalence",
            "φ and ψ are logically equivalent iff {φ} ⊨ ψ AND {ψ} ⊨ φ. Equivalently: every model agrees on them."),

        Theory: ("en", "Theory",
            "A set of sentences closed under logical consequence. Alternatively (looser): any set of sentences (its axioms). Chang-Keisler §1.3."),
        AxiomSet: ("en", "Axiom set",
            "A set of sentences not necessarily closed under consequence, presented as the 'starting points' of a theory. Every theory has one; often a finite or r.e. set."),
        ElementaryEquivalence: ("en", "Elementary equivalence",
            "Two structures M, N are elementarily equivalent (M ≡ N) iff they satisfy exactly the same first-order sentences. Distinct from isomorphism — there are non-isomorphic but elementarily-equivalent structures."),

        Soundness: ("en", "Soundness",
            "A proof system is sound for a semantics iff every derivable sentence is semantically valid: Γ ⊢ φ ⇒ Γ ⊨ φ. The proof system doesn't prove falsehoods."),
        Completeness: ("en", "Completeness",
            "A proof system is complete for a semantics iff every semantically-valid sentence is derivable: Γ ⊨ φ ⇒ Γ ⊢ φ. Gödel (1930) for first-order logic."),
        Consistency: ("en", "Consistency",
            "A theory is consistent iff it has a model. Equivalently (by completeness, for first-order): it does not prove a contradiction."),
        Compactness: ("en", "Compactness theorem",
            "First-order logic: a set of sentences has a model iff every finite subset has a model. Chang-Keisler §1.4. A foundational result."),
        LowenheimSkolem: ("en", "Löwenheim-Skolem",
            "If a first-order theory has an infinite model, it has models of every infinite cardinality. Chang-Keisler §2.1. Downward: smaller models exist; upward: larger models exist."),
    },

    is_a: [
        // Truth / falsity are specialisations of satisfaction
        (Truth, Satisfaction),
        (Falsity, Satisfaction),

        // Validity is universal truth
        (Validity, Truth),

        // Entailment is logical consequence
        (Entailment, LogicalConsequence),

        // Axiom set is a (presented) theory
        (AxiomSet, Theory),

        // Structure is (with interpretation) a model-world
        (Model, Structure),

        // Elementary equivalence is a refinement of logical equivalence
        (ElementaryEquivalence, Equivalence),
    ],

    has_a: [
        // A model has a universe and an interpretation
        (Model, Universe),
        (Model, Interpretation),
        (Structure, Universe),
        (Structure, Interpretation),

        // An interpretation maps a signature's symbols
        (Interpretation, Signature),

        // A theory has axioms
        (Theory, AxiomSet),

        // Satisfaction uses a variable assignment
        (Satisfaction, Assignment),
    ],

    opposes: [
        // The proof-theoretic/semantic opposition Gödel bridged (completeness)
        (Validity, Satisfiability),   // dual via negation: valid iff ¬φ unsatisfiable

        // Truth vs falsity — classical bivalence
        (Truth, Falsity),

        // Soundness vs completeness — the two directions of proof/semantic bridge.
        // Not truly opposed (both hold in first-order logic), but the duality is
        // fundamental.
    ],
}

// -----------------------------------------------------------------------------
// Qualities
// -----------------------------------------------------------------------------

/// Literature source for each concept.
#[derive(Debug, Clone)]
pub struct ModelTheoryTradition;

impl Quality for ModelTheoryTradition {
    type Individual = ModelTheoryConcept;
    type Value = &'static str;

    fn get(&self, c: &ModelTheoryConcept) -> Option<&'static str> {
        use ModelTheoryConcept as M;
        Some(match c {
            M::Satisfaction | M::Truth | M::Falsity | M::Assignment | M::Interpretation => {
                "tarski-1933"
            }
            M::LogicalConsequence | M::Entailment => "tarski-1936",
            M::Validity
            | M::Satisfiability
            | M::Equivalence
            | M::Model
            | M::Structure
            | M::Universe
            | M::Signature
            | M::Theory
            | M::AxiomSet
            | M::ElementaryEquivalence
            | M::Compactness
            | M::LowenheimSkolem => "chang-keisler-1990",
            M::Soundness | M::Completeness | M::Consistency => "godel-1930-1931",
        })
    }
}

impl Ontology for ModelTheoryOntology {
    type Cat = ModelTheoryCategory;
    type Qual = ModelTheoryTradition;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        ModelTheoryOntology::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        ModelTheoryOntology::generated_domain_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;

    #[test]
    fn category_laws() {
        check_category_laws::<ModelTheoryCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        ModelTheoryOntology::validate().unwrap();
    }

    #[test]
    fn validity_is_a_truth() {
        use pr4xis::ontology::reasoning::taxonomy;
        assert!(taxonomy::is_a::<ModelTheoryTaxonomy>(
            &ModelTheoryConcept::Validity,
            &ModelTheoryConcept::Truth,
        ));
    }

    #[test]
    fn truth_and_falsity_oppose() {
        use pr4xis::ontology::reasoning::opposition;
        let opposed = opposition::opposites::<ModelTheoryOpposition>(&ModelTheoryConcept::Truth);
        assert!(opposed.contains(&ModelTheoryConcept::Falsity));
    }
}
