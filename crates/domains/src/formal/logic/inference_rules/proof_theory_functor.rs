//! Functor: InferenceRules → ProofTheory.
//!
//! Inference rules are the building blocks of proofs. Each rule maps to
//! a specific proof-theoretic structural or logical move. This functor
//! formalises that dependency: proof theory IS inference rules composed.

use pr4xis::category::{Category, Functor};

use super::ontology::{
    InferenceRulesCategory, InferenceRulesConcept, InferenceRulesRelation,
    InferenceRulesRelationKind,
};
use crate::formal::logic::proof_theory::ontology::{
    ProofTheoryCategory, ProofTheoryConcept, ProofTheoryRelation, ProofTheoryRelationKind,
};

pub struct InferenceRulesToProofTheory;

fn map_concept(c: &InferenceRulesConcept) -> ProofTheoryConcept {
    use InferenceRulesConcept as R;
    use ProofTheoryConcept as P;
    match c {
        // The genus Rule maps to the atomic proof-theoretic unit — Sequent.
        R::InferenceRule => P::Sequent,

        // Deductive rules ARE proof-theoretic moves; map to Sequent (the
        // canonical sequent-calculus inference line).
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
        | R::ReductioAdAbsurdum
        | R::DoubleNegationElimination
        | R::ExcludedMiddle => P::Sequent,

        // The cut rule maps to proof-theoretic Cut.
        R::CutRule => P::Cut,

        // Inductive / abductive rules — the non-deductive modes — are
        // not strictly part of classical proof theory; the closest
        // proof-theoretic analogue is Conjecture (unproved statement).
        R::Induction | R::Abduction | R::Generalisation | R::BestExplanation => P::Conjecture,
    }
}

impl Functor for InferenceRulesToProofTheory {
    type Source = InferenceRulesCategory;
    type Target = ProofTheoryCategory;

    fn map_object(obj: &InferenceRulesConcept) -> ProofTheoryConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &InferenceRulesRelation) -> ProofTheoryRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        match m.kind {
            InferenceRulesRelationKind::Identity => ProofTheoryCategory::identity(&from),
            _ => ProofTheoryRelation {
                from,
                to,
                kind: ProofTheoryRelationKind::Composed,
            },
        }
    }
}

pr4xis::register_functor!(
    InferenceRulesToProofTheory,
    "InferenceRules → ProofTheory: rules are the building blocks of sequents."
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws_pass() {
        check_functor_laws::<InferenceRulesToProofTheory>().unwrap();
    }

    #[test]
    fn modus_ponens_maps_to_sequent() {
        assert_eq!(
            InferenceRulesToProofTheory::map_object(&InferenceRulesConcept::ModusPonens),
            ProofTheoryConcept::Sequent,
        );
    }

    #[test]
    fn cut_rule_maps_to_cut() {
        assert_eq!(
            InferenceRulesToProofTheory::map_object(&InferenceRulesConcept::CutRule),
            ProofTheoryConcept::Cut,
        );
    }
}
