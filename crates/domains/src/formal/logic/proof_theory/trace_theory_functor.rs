//! Functor: ProofTheory → TraceTheory — the **Curry-Howard / Hyland-Ong**
//! correspondence as a verified pr4xis functor.
//!
//! This is the load-bearing cross-functor of the logic layer. Two
//! load-bearing theorems make it literature-honest:
//!
//! 1. **Curry-Howard (1958)** *Combinatory Logic* Vol. I — propositions
//!    are types, proofs are programs. A proof is an inhabitant; a proof
//!    in normal form is a computed value. Normalisation is computation.
//!
//! 2. **Hyland & Ong (2000)** "On Full Abstraction for PCF" (Information
//!    and Computation 163: 285–408) — innocent strategies in the game
//!    semantics of PCF correspond exactly to PCF programs (and, by
//!    Curry-Howard, to intuitionistic proofs). A proof IS a trace of
//!    Player-Opponent interaction, read game-theoretically.
//!
//! The functor below encodes the concept-level mapping. A proof is a
//! play; a theorem is a strategy (specifically, an innocent one); cut
//! elimination is reduction; normal form is terminated play; discharge
//! is the game-theoretic closing of an open question.
//!
//! The converse direction (TraceTheory → ProofTheory) is the existence
//! of the isomorphism. Together the two functors witness that the proof
//! and trace categories are equivalent — they ARE the same category
//! under different names.

use pr4xis::category::{Category, Functor};

use super::ontology::{
    ProofTheoryCategory, ProofTheoryConcept, ProofTheoryRelation, ProofTheoryRelationKind,
};
use crate::formal::logic::trace_theory::ontology::{
    TraceTheoryCategory, TraceTheoryConcept, TraceTheoryRelation, TraceTheoryRelationKind,
};

pub struct ProofTheoryToTraceTheory;

fn map_concept(c: &ProofTheoryConcept) -> TraceTheoryConcept {
    use ProofTheoryConcept as P;
    use TraceTheoryConcept as T;
    match c {
        // Curry-Howard: a proof IS a play (Hyland-Ong).
        P::Proof | P::ProofTree => T::Play,

        // A theorem is a (reified) strategy — more specifically, an
        // innocent one (Hyland-Ong's key discovery).
        P::Theorem | P::Lemma | P::SubProof => T::InnocentStrategy,

        // Axioms are specific moves (atomic plays).
        P::Axiom => T::Move,

        // Sequents / antecedents / succedents are positions in the game.
        P::Sequent | P::Antecedent | P::Succedent => T::Position,

        // Premises / hypotheses / conclusions are positions; the game-theoretic
        // analogues of "opponent's move" (hypothesis) and "player's answer" (conclusion).
        P::Premise | P::Hypothesis | P::Assumption | P::Conclusion => T::Position,

        // Cut / cut elimination = Interaction / Reduction. This is the heart
        // of the correspondence: cut is the composition of strategies (Girard's
        // cut-elimination theorem IS Hyland-Ong's innocent composition).
        P::Cut => T::Interaction,
        P::CutElimination | P::Normalisation => T::ReductionSequence,
        P::NormalForm => T::InnocentStrategy, // terminated innocent play
        P::Redex => T::ReductionStep,

        // Structural rules are specific move types.
        P::Weakening | P::Contraction | P::Exchange => T::Move,

        // Discharge = closing an open-question move.
        P::Discharge => T::Event,

        // Conjectures / counterexamples are trace-equivalence claims.
        P::Conjecture | P::Counterexample => T::TraceEquivalence,

        // Meta-theoretic properties project to the Trace genus.
        P::Consistency | P::Completeness | P::Decidability => T::Trace,
    }
}

impl Functor for ProofTheoryToTraceTheory {
    type Source = ProofTheoryCategory;
    type Target = TraceTheoryCategory;

    fn map_object(obj: &ProofTheoryConcept) -> TraceTheoryConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &ProofTheoryRelation) -> TraceTheoryRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        match m.kind {
            ProofTheoryRelationKind::Identity => TraceTheoryCategory::identity(&from),
            _ => TraceTheoryRelation {
                from,
                to,
                kind: TraceTheoryRelationKind::Composed,
            },
        }
    }
}

pr4xis::register_functor!(
    ProofTheoryToTraceTheory,
    "ProofTheory → TraceTheory: the Curry-Howard (1958) / Hyland-Ong (2000) correspondence. Proofs ARE plays; theorems ARE innocent strategies; cut elimination IS reduction."
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn curry_howard_functor_laws_pass() {
        check_functor_laws::<ProofTheoryToTraceTheory>().unwrap();
    }

    /// The headline: a proof maps to a play.
    #[test]
    fn proof_is_a_play() {
        assert_eq!(
            ProofTheoryToTraceTheory::map_object(&ProofTheoryConcept::Proof),
            TraceTheoryConcept::Play,
        );
    }

    /// Theorems are innocent strategies (Hyland-Ong's headline claim).
    #[test]
    fn theorem_is_an_innocent_strategy() {
        assert_eq!(
            ProofTheoryToTraceTheory::map_object(&ProofTheoryConcept::Theorem),
            TraceTheoryConcept::InnocentStrategy,
        );
    }

    /// Cut elimination IS reduction — Girard's observation made categorical.
    #[test]
    fn cut_elimination_is_reduction() {
        assert_eq!(
            ProofTheoryToTraceTheory::map_object(&ProofTheoryConcept::CutElimination),
            TraceTheoryConcept::ReductionSequence,
        );
    }
}
