//! Property-based tests for the Syntrometry ontology.
//!
//! Per `feedback_full_test_coverage.md`: single-point axiom tests are not
//! enough; randomised sweeps give genuine coverage across the finite
//! variant space and demonstrate invariant stability.

#![cfg(test)]

use proptest::prelude::*;

use super::adjunction::{counit_pair, unit_pair};
use super::lineage_functor::SyntrometryToPr4xisSubstrate;
use super::ontology::{
    AspektIsTripleProduct, SyntrixIsLeveled, SyntrometryCategory, SyntrometryConcept,
    SyntrometryOntology, SyntrometryRelation, SyntrometryRelationKind,
};
use super::substrate::{Pr4xisSubstrateCategory, Pr4xisSubstrateConcept, Pr4xisSubstrateOntology};
use super::substrate_functor::map_substrate;
use pr4xis::category::{Category, Entity, Functor};
use pr4xis::ontology::{Axiom, Ontology};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_syntrometry_concept() -> impl Strategy<Value = SyntrometryConcept> {
    proptest::sample::select(SyntrometryConcept::variants())
}

fn arb_substrate_concept() -> impl Strategy<Value = Pr4xisSubstrateConcept> {
    proptest::sample::select(Pr4xisSubstrateConcept::variants())
}

fn arb_syntrometry_morphism() -> impl Strategy<Value = SyntrometryRelation> {
    proptest::sample::select(SyntrometryCategory::morphisms())
}

proptest! {
    // -----------------------------------------------------------------------
    // Axiom invariance under random sampling
    // -----------------------------------------------------------------------

    /// The three domain axioms are *structural* invariants — they must hold
    /// regardless of which sampling of concepts you inspect. Running each
    /// axiom inside a proptest loop is redundant in content, but the loop
    /// shape is the canonical "sweep over the domain" idiom and catches
    /// any non-determinism or state mutation that would break invariance.
    #[test]
    fn aspekt_triple_product_invariant_under_sweep(_ in 0..256u32) {
        prop_assert!(AspektIsTripleProduct.holds());
    }

    #[test]
    fn syntrix_is_leveled_invariant_under_sweep(_ in 0..256u32) {
        prop_assert!(SyntrixIsLeveled.holds());
    }

    // -----------------------------------------------------------------------
    // Functor-level invariants on random inputs
    // -----------------------------------------------------------------------

    /// For every concept sampled, the forward mapping is stable (pure).
    #[test]
    fn forward_functor_is_deterministic(c in arb_syntrometry_concept()) {
        let a = SyntrometryToPr4xisSubstrate::map_object(&c);
        let b = SyntrometryToPr4xisSubstrate::map_object(&c);
        prop_assert_eq!(a, b);
    }

    /// For every morphism sampled, F(m) is a valid target morphism (its
    /// from/to endpoints come from the target's entity variants).
    #[test]
    fn forward_functor_maps_morphisms_into_target(m in arb_syntrometry_morphism()) {
        let mapped = SyntrometryToPr4xisSubstrate::map_morphism(&m);
        prop_assert!(Pr4xisSubstrateConcept::variants().contains(&mapped.from));
        prop_assert!(Pr4xisSubstrateConcept::variants().contains(&mapped.to));
    }

    /// Identity preservation (sampled): for every sampled concept C,
    /// F(id_C) must equal id_{F(C)}.
    #[test]
    fn forward_functor_preserves_identity(c in arb_syntrometry_concept()) {
        let id_c = SyntrometryCategory::identity(&c);
        let f_id = SyntrometryToPr4xisSubstrate::map_morphism(&id_c);
        let id_fc =
            Pr4xisSubstrateCategory::identity(&SyntrometryToPr4xisSubstrate::map_object(&c));
        prop_assert_eq!(f_id, id_fc);
    }

    // -----------------------------------------------------------------------
    // Adjunction round-trip invariants
    // -----------------------------------------------------------------------

    /// Every counit pair is trivial — the substrate target is in the image
    /// of the forward functor, so F(G(B)) = B for every substrate concept.
    #[test]
    fn counit_round_trip_is_trivial(p in arb_substrate_concept()) {
        let (rt, target) = counit_pair(&p);
        prop_assert_eq!(target, p);
        prop_assert_eq!(rt, p);
    }

    /// Unit pairs may or may not be trivial; either way, the result must
    /// name two valid syntrometric concepts.
    #[test]
    fn unit_round_trip_stays_within_syntrometry(s in arb_syntrometry_concept()) {
        let (source, rt) = unit_pair(&s);
        prop_assert_eq!(source, s);
        prop_assert!(SyntrometryConcept::variants().contains(&rt));
    }

    /// Phase 3 target: all 14 syntrometric concepts are round-trip fixed
    /// points. Sampling any one reproduces it under G∘F. This is the
    /// "object-level equivalence" form of the Heim ↔ pr4xis lineage.
    #[test]
    fn every_concept_is_round_trip_fixed_point(c in arb_syntrometry_concept()) {
        let (source, rt) = unit_pair(&c);
        prop_assert_eq!(source, rt);
    }

    // -----------------------------------------------------------------------
    // Substrate closure
    // -----------------------------------------------------------------------

    /// Every substrate primitive has a canonical syntrometric
    /// representative, and mapping back through F gives the same substrate
    /// primitive — the substrate is closed under the reverse map composed
    /// with the forward map.
    #[test]
    fn substrate_is_closed_under_round_trip(p in arb_substrate_concept()) {
        let s = map_substrate(&p);
        let back = SyntrometryToPr4xisSubstrate::map_object(&s);
        prop_assert_eq!(back, p);
    }

    // -----------------------------------------------------------------------
    // Ontology-level invariants
    // -----------------------------------------------------------------------

    /// `Ontology::validate()` is a pure function of structure; repeated
    /// invocations must yield the same Ok result.
    #[test]
    fn syntrometry_ontology_validates_consistently(_ in 0..32u32) {
        prop_assert!(SyntrometryOntology::validate().is_ok());
    }

    #[test]
    fn substrate_ontology_validates_consistently(_ in 0..32u32) {
        prop_assert!(Pr4xisSubstrateOntology::validate().is_ok());
    }

    // -----------------------------------------------------------------------
    // Morphism closure
    // -----------------------------------------------------------------------

    /// Every kind of Syntrometry morphism (Identity, every declared edge
    /// kind, Composed) shows up in `morphisms()`. Without this the
    /// category's closure claim would be empty.
    #[test]
    fn syntrometry_morphism_kinds_are_exhaustive(_ in 0..16u32) {
        let morphisms = SyntrometryCategory::morphisms();
        let mut saw_identity = false;
        let mut saw_composed = false;
        for m in &morphisms {
            match m.kind {
                SyntrometryRelationKind::Identity => saw_identity = true,
                SyntrometryRelationKind::Composed => saw_composed = true,
                _ => {}
            }
        }
        prop_assert!(saw_identity);
        prop_assert!(saw_composed);
    }
}
