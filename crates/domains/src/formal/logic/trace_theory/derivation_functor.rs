//! Functor: TraceTheory → Derivation.
//!
//! Every trace-theoretic object is a specialisation of Derivation's
//! temporal view. Reduction sequences, plays, innocent strategies are
//! all derivations — just presented operationally (Plotkin 1981) or
//! game-semantically (Hyland-Ong 2000). This functor witnesses the
//! inclusion.

use pr4xis::category::{Category, Functor};

use super::ontology::{
    TraceTheoryCategory, TraceTheoryConcept, TraceTheoryRelation, TraceTheoryRelationKind,
};
use crate::formal::logic::derivation::ontology::{
    DerivationCategory, DerivationConcept, DerivationRelation, DerivationRelationKind,
};

pub struct TraceTheoryToDerivation;

fn map_concept(c: &TraceTheoryConcept) -> DerivationConcept {
    use DerivationConcept as D;
    use TraceTheoryConcept as T;
    match c {
        // All trace-flavored objects collapse to the genus Trace.
        T::Trace
        | T::ReductionSequence
        | T::ObservationalTrace
        | T::Play
        | T::Strategy
        | T::InnocentStrategy => D::Trace,

        // Atomic moves within a trace are derivation-steps.
        T::ReductionStep | T::SmallStep | T::BigStep | T::Move | T::Event | T::Observation => {
            D::DerivationStep
        }

        // Reductions map to the genus Reduction.
        T::Interaction => D::Composition,

        // Configurations / positions / contexts are the state of a trace —
        // project to Hypothesis (the starting datum of the next step).
        T::Configuration | T::Position | T::EvaluationContext => D::Hypothesis,

        // Equivalences over traces are derivation-level claims.
        T::TraceEquivalence | T::BisimulationCandidate => D::Witness,
    }
}

impl Functor for TraceTheoryToDerivation {
    type Source = TraceTheoryCategory;
    type Target = DerivationCategory;

    fn map_object(obj: &TraceTheoryConcept) -> DerivationConcept {
        map_concept(obj)
    }

    fn map_morphism(m: &TraceTheoryRelation) -> DerivationRelation {
        let from = map_concept(&m.from);
        let to = map_concept(&m.to);
        match m.kind {
            TraceTheoryRelationKind::Identity => DerivationCategory::identity(&from),
            _ => DerivationRelation {
                from,
                to,
                kind: DerivationRelationKind::Composed,
            },
        }
    }
}

pr4xis::register_functor!(
    TraceTheoryToDerivation,
    "TraceTheory (Plotkin 1981; Hyland-Ong 2000) → Derivation. Traces are operational/temporal derivations."
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;

    #[test]
    fn functor_laws_pass() {
        check_functor_laws::<TraceTheoryToDerivation>().unwrap();
    }

    #[test]
    fn trace_maps_to_derivation_trace() {
        assert_eq!(
            TraceTheoryToDerivation::map_object(&TraceTheoryConcept::Trace),
            DerivationConcept::Trace,
        );
    }

    #[test]
    fn reduction_sequence_maps_to_trace() {
        assert_eq!(
            TraceTheoryToDerivation::map_object(&TraceTheoryConcept::ReductionSequence),
            DerivationConcept::Trace,
        );
    }
}
