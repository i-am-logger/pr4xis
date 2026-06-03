//! Tests for [`super::ontology`].

use super::*;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Axiom, Quality};
use proptest::prelude::*;

#[test]
fn category_laws_hold() {
    assert_category_laws::<FunctorSynthesizerCategory>();
}

#[test]
fn ontology_validates() {
    FunctorSynthesizerOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[test]
fn axiom_functor_law_has_both_axioms_holds() {
    assert!(FunctorLawHasBothAxioms.verify().is_ok());
}

#[test]
fn axiom_pipeline_reaches_convergence_holds() {
    assert!(PipelineReachesConvergence.verify().is_ok());
}

#[test]
fn axiom_identity_and_composition_complementary_holds() {
    assert!(IdentityAndCompositionAreComplementary.verify().is_ok());
}

#[test]
fn both_functor_axioms_subsume_functor_law() {
    let sub: alloc::vec::Vec<_> = FunctorSynthesizerCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == FunctorSynthesizerRelationKind::Subsumption)
        .map(|m| (m.source(), m.target()))
        .collect();
    use FunctorSynthesizerConcept as C;
    assert!(sub.contains(&(C::IdentityPreservation, C::FunctorLaw)));
    assert!(sub.contains(&(C::CompositionPreservation, C::FunctorLaw)));
}

#[test]
fn pipeline_first_and_last_present() {
    let causation: alloc::vec::Vec<_> = FunctorSynthesizerCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == FunctorSynthesizerRelationKind::Causation)
        .map(|m| (m.source(), m.target()))
        .collect();
    use FunctorSynthesizerConcept as C;
    assert!(causation.contains(&(C::IngestDiscovery, C::BuildObjectMapping)));
    assert!(causation.contains(&(C::IterateCycle, C::DetectConvergence)));
}

#[test]
fn lineage_quality_total_on_named_concepts() {
    let lineage = FunctorSynthesizerLineage;
    use FunctorSynthesizerConcept as C;
    for c in [
        C::SynthesizedFunctor,
        C::ObjectMapping,
        C::MorphismMapping,
        C::FunctorLaw,
        C::IdentityPreservation,
        C::CompositionPreservation,
        C::BootstrappingCycle,
        C::ConvergenceWitness,
    ] {
        assert!(lineage.get(&c).is_some(), "no lineage for {c:?}");
    }
}

fn arb_concept() -> impl Strategy<Value = FunctorSynthesizerConcept> {
    proptest::sample::select(FunctorSynthesizerConcept::variants())
}

proptest! {
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in FunctorSynthesizerCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    #[test]
    fn prop_lineage_returns_string(c in arb_concept()) {
        let lineage = FunctorSynthesizerLineage;
        prop_assert!(lineage.get(&c).is_some());
    }

    #[test]
    fn prop_opposition_symmetric(_seed in any::<u32>()) {
        let opp: std::collections::HashSet<_> = FunctorSynthesizerCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == FunctorSynthesizerRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        for (a, b) in &opp {
            prop_assert!(opp.contains(&(*b, *a)));
        }
    }
}
