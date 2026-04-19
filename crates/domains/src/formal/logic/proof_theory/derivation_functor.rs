//! Functor: ProofTheory → Derivation.
//!
//! Every proof-theoretic object is a specialisation of the umbrella
//! `Derivation` concept. This functor witnesses that: a Proof is a
//! Derivation (atemporal view), its premises are hypotheses, its
//! conclusion is a consequence, its sub-proofs are derivation-steps,
//! axioms and theorems collapse to the Derivation's witness / genus
//! position. Cut elimination and normalisation map to the reduction /
//! normalisation vocabulary in Derivation (the operational inheritance
//! that makes proof-theory and trace-theory unify via the cross-functor).
//!
//! Reference: Lambek (1968) — proofs as morphisms in the deductive
//! category, which is precisely how Derivation structures proof objects.

use pr4xis::category::{Category, Functor};

use super::ontology::{
    ProofTheoryCategory, ProofTheoryConcept, ProofTheoryRelation, ProofTheoryRelationKind,
};
use crate::formal::logic::derivation::ontology::{
    DerivationCategory, DerivationConcept, DerivationRelation, DerivationRelationKind,
};

/// The projection from proof-theoretic vocabulary to the genus Derivation.
pub struct ProofTheoryToDerivation;

fn map_concept(c: &ProofTheoryConcept) -> DerivationConcept {
    use DerivationConcept as D;
    use ProofTheoryConcept as P;
    match c {
        // Proofs in the rich sense collapse to the genus Proof.
        P::Proof | P::ProofTree | P::SubProof | P::Theorem | P::Lemma | P::Axiom => D::Proof,

        // Hypothesis / premise / assumption all land on the genus Hypothesis.
        P::Premise | P::Antecedent | P::Hypothesis | P::Assumption => D::Hypothesis,

        // Conclusion / succedent land on the genus Consequence.
        P::Conclusion | P::Succedent => D::Consequence,

        // A sequent is a derivation-step (the atomic sequent-calculus move).
        P::Sequent | P::Weakening | P::Contraction | P::Exchange => D::DerivationStep,

        // Cut and its elimination map to composition (cut chains two derivations);
        // CutElimination + Normalisation map to Normalisation in the genus.
        P::Cut => D::Composition,
        P::CutElimination | P::Normalisation | P::NormalForm => D::Normalisation,
        P::Redex => D::Reduction,

        // Discharge is the closing-off of a hypothesis — a witness-producing move.
        P::Discharge => D::Justification,

        // Conjecture / counterexample are the "unproven" and "refuted" specialisations —
        // map to Witness (positive) and Evidence (negative) respectively.
        P::Conjecture => D::Witness,
        P::Counterexample => D::Evidence,

        // Meta-theoretic properties are about the derivation-genus itself;
        // project to the genus concept.
        P::Consistency | P::Completeness | P::Decidability => D::Derivation,
    }
}

impl Functor for ProofTheoryToDerivation {
    type Source = ProofTheoryCategory;
    type Target = DerivationCategory;

    fn map_object(obj: &ProofTheoryConcept) -> DerivationConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &ProofTheoryRelation) -> DerivationRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        match m.kind {
            ProofTheoryRelationKind::Identity => DerivationCategory::identity(&from),
            _ => DerivationRelation {
                from,
                to,
                kind: DerivationRelationKind::Composed,
            },
        }
    }
}

pr4xis::register_functor!(
    ProofTheoryToDerivation,
    "ProofTheory (Gentzen 1935; Prawitz 1965) → Derivation (Lambek 1968; JSV 1996). Proofs are atemporal derivations."
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws_pass() {
        check_functor_laws::<ProofTheoryToDerivation>().unwrap();
    }

    #[test]
    fn proof_maps_to_derivation_proof() {
        assert_eq!(
            ProofTheoryToDerivation::map_object(&ProofTheoryConcept::Proof),
            DerivationConcept::Proof,
        );
    }

    #[test]
    fn hypothesis_maps_to_hypothesis() {
        assert_eq!(
            ProofTheoryToDerivation::map_object(&ProofTheoryConcept::Hypothesis),
            DerivationConcept::Hypothesis,
        );
    }
}
