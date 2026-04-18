//! Property-based tests for the MAPE-K ontology + PipelineStep functor.

#![cfg(test)]

use proptest::prelude::*;

use super::ontology::{
    EveryPhaseConsultsKnowledge, FourPhaseCycle, LoopIsClosed, MapeKCategory, MapeKConcept,
    MapeKOntology,
};
use super::pipeline_step_functor::{PipelineStepCategory, PipelineStepToMapeK};
use crate::formal::information::diagnostics::trace_functors::PipelineStep;
use pr4xis::category::{Category, Entity, Functor};
use pr4xis::ontology::{Axiom, Ontology};

fn arb_mape_k_concept() -> impl Strategy<Value = MapeKConcept> {
    proptest::sample::select(MapeKConcept::variants())
}

fn arb_pipeline_step() -> impl Strategy<Value = PipelineStep> {
    proptest::sample::select(PipelineStep::variants())
}

proptest! {
    /// The three MAPE-K domain axioms are structural invariants — they
    /// hold regardless of which random sampling drives the test.
    #[test]
    fn four_phase_cycle_invariant(_ in 0..128u32) {
        prop_assert!(FourPhaseCycle.holds());
    }

    #[test]
    fn loop_is_closed_invariant(_ in 0..128u32) {
        prop_assert!(LoopIsClosed.holds());
    }

    #[test]
    fn every_phase_consults_knowledge_invariant(_ in 0..128u32) {
        prop_assert!(EveryPhaseConsultsKnowledge.holds());
    }

    /// Ontology validation is a pure function of structure.
    #[test]
    fn ontology_validates_consistently(_ in 0..32u32) {
        prop_assert!(MapeKOntology::validate().is_ok());
    }

    /// For any sampled MAPE-K concept, identity preservation holds on
    /// the generated category: `compose(id, id) == id`.
    #[test]
    fn identity_composes_with_identity(c in arb_mape_k_concept()) {
        let id = MapeKCategory::identity(&c);
        let composed = MapeKCategory::compose(&id, &id);
        prop_assert_eq!(composed, Some(id));
    }

    /// For any sampled PipelineStep, the functor map_object is deterministic.
    #[test]
    fn pipeline_step_functor_is_deterministic(s in arb_pipeline_step()) {
        let a = PipelineStepToMapeK::map_object(&s);
        let b = PipelineStepToMapeK::map_object(&s);
        prop_assert_eq!(a, b);
    }

    /// For any sampled PipelineStep, the functor preserves identity:
    /// `F(id_s) == id_{F(s)}`.
    #[test]
    fn pipeline_step_functor_preserves_identity(s in arb_pipeline_step()) {
        let id_s = PipelineStepCategory::identity(&s);
        let f_id = PipelineStepToMapeK::map_morphism(&id_s);
        let id_fs = MapeKCategory::identity(&PipelineStepToMapeK::map_object(&s));
        prop_assert_eq!(f_id, id_fs);
    }

    /// No PipelineStep ever lands on Knowledge — Knowledge is the
    /// consulted substrate, not a phase. Kephart & Chess (2003).
    #[test]
    fn no_step_is_knowledge(s in arb_pipeline_step()) {
        prop_assert_ne!(
            PipelineStepToMapeK::map_object(&s),
            MapeKConcept::Knowledge,
            "no step should be a Knowledge itself — K is shared substrate"
        );
    }

    /// No PipelineStep lands on the abstract parent MapeKPhase — every
    /// step picks one of the four concrete phases.
    #[test]
    fn no_step_is_abstract_phase(s in arb_pipeline_step()) {
        prop_assert_ne!(
            PipelineStepToMapeK::map_object(&s),
            MapeKConcept::MapeKPhase,
            "steps map to concrete phases, not the abstract parent"
        );
    }
}
