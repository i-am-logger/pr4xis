//! Functor: ModelTheory → Derivation.
//!
//! Model-theoretic satisfaction (Tarski's T-schema) IS a kind of Witness
//! in the Derivation framework. A model-check verifies that a structure
//! satisfies a formula — the evidence is semantic rather than syntactic,
//! but it still justifies a derivation step. This functor formalises
//! that: model-theoretic satisfaction is Evidence; logical consequence
//! is composition-of-derivations.

use pr4xis::category::{Category, Functor};

use super::ontology::{
    ModelTheoryCategory, ModelTheoryConcept, ModelTheoryRelation, ModelTheoryRelationKind,
};
use crate::formal::logic::derivation::ontology::{
    DerivationCategory, DerivationConcept, DerivationRelation, DerivationRelationKind,
};

pub struct ModelTheoryToDerivation;

fn map_concept(c: &ModelTheoryConcept) -> DerivationConcept {
    use DerivationConcept as D;
    use ModelTheoryConcept as M;
    match c {
        // Satisfaction, truth, validity — these ARE model-theoretic witnesses.
        M::Satisfaction | M::Truth | M::Validity | M::Satisfiability => D::Evidence,
        M::Falsity => D::Witness, // refutation-witness

        // Logical consequence / entailment = composition-of-derivations
        // (chaining premises to conclusions model-theoretically).
        M::LogicalConsequence | M::Entailment | M::Equivalence | M::ElementaryEquivalence => {
            D::Composition
        }

        // The meta-theoretic bridge concepts (soundness, completeness)
        // are claims ABOUT derivations — project to the genus.
        M::Soundness | M::Completeness | M::Consistency | M::Compactness | M::LowenheimSkolem => {
            D::Derivation
        }

        // Structures and their parts are the Hypothesis of a model-theoretic
        // derivation — what's "given" when you model-check.
        M::Model
        | M::Structure
        | M::Interpretation
        | M::Universe
        | M::Signature
        | M::Assignment
        | M::Theory
        | M::AxiomSet => D::Hypothesis,
    }
}

impl Functor for ModelTheoryToDerivation {
    type Source = ModelTheoryCategory;
    type Target = DerivationCategory;

    fn map_object(obj: &ModelTheoryConcept) -> DerivationConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &ModelTheoryRelation) -> DerivationRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        match m.kind {
            ModelTheoryRelationKind::Identity => DerivationCategory::identity(&from),
            _ => DerivationRelation {
                from,
                to,
                kind: DerivationRelationKind::Composed,
            },
        }
    }
}

pr4xis::register_functor!(
    ModelTheoryToDerivation,
    "ModelTheory (Tarski 1936) → Derivation. Model-check is Evidence; consequence is composition."
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws_pass() {
        check_functor_laws::<ModelTheoryToDerivation>().unwrap();
    }

    #[test]
    fn satisfaction_is_evidence() {
        assert_eq!(
            ModelTheoryToDerivation::map_object(&ModelTheoryConcept::Satisfaction),
            DerivationConcept::Evidence,
        );
    }

    #[test]
    fn entailment_is_composition() {
        assert_eq!(
            ModelTheoryToDerivation::map_object(&ModelTheoryConcept::Entailment),
            DerivationConcept::Composition,
        );
    }
}
