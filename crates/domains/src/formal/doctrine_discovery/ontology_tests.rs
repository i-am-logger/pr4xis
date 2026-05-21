//! Tests for [`super::ontology`].

use super::*;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::ontology::{Axiom, Quality};
use proptest::prelude::*;

#[test]
fn category_laws_hold() {
    assert_category_laws::<DoctrineDiscoveryCategory>();
}

#[test]
fn ontology_validates() {
    DoctrineDiscoveryOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Domain axioms.
// =============================================================================

#[test]
fn axiom_outputs_classify_as_discovery_output_holds() {
    assert!(OutputsClassifyAsDiscoveryOutput.verify().is_ok());
}

#[test]
fn axiom_pipeline_is_linear_chain_holds() {
    assert!(PipelineIsLinearChain.verify().is_ok());
}

// =============================================================================
// Structural spot checks.
// =============================================================================

#[test]
fn inputs_classify_under_discovery_input() {
    let sub: alloc::vec::Vec<_> = DoctrineDiscoveryCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == DoctrineDiscoveryRelationKind::Subsumption)
        .map(|m| (m.source(), m.target()))
        .collect();
    use DoctrineDiscoveryConcept as C;
    for input in [
        C::ObjectCorpus,
        C::AttributeExtractor,
        C::FormalContextInput,
    ] {
        assert!(sub.contains(&(input, C::DiscoveryInput)));
    }
}

#[test]
fn pipeline_first_and_last_present() {
    let causation: alloc::vec::Vec<_> = DoctrineDiscoveryCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == DoctrineDiscoveryRelationKind::Causation)
        .map(|m| (m.source(), m.target()))
        .collect();
    use DoctrineDiscoveryConcept as C;
    // CorpusLoad → AttributeExtraction (head).
    assert!(causation.contains(&(C::CorpusLoad, C::AttributeExtraction)));
    // AbductiveLift → OutputAssembly (tail).
    assert!(causation.contains(&(C::AbductiveLift, C::OutputAssembly)));
}

#[test]
fn lineage_quality_total_on_named_concepts() {
    let lineage = DoctrineDiscoveryLineage;
    use DoctrineDiscoveryConcept as C;
    for c in [
        C::ObjectCorpus,
        C::DoctrineCluster,
        C::AttributeClosureImplication,
        C::CanonicalDoctrineBasis,
        C::DoctrineDiscovery,
        C::AbductiveLift,
        C::FibrationLift,
    ] {
        assert!(lineage.get(&c).is_some(), "no lineage for {c:?}");
    }
}

// =============================================================================
// Property tests.
// =============================================================================

fn arb_concept() -> impl Strategy<Value = DoctrineDiscoveryConcept> {
    proptest::sample::select(DoctrineDiscoveryConcept::variants())
}

proptest! {
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in DoctrineDiscoveryCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    #[test]
    fn prop_lineage_returns_string(c in arb_concept()) {
        let lineage = DoctrineDiscoveryLineage;
        prop_assert!(lineage.get(&c).is_some());
    }

    #[test]
    fn prop_subsumption_targets_valid(_seed in any::<u32>()) {
        let variants: alloc::vec::Vec<_> = DoctrineDiscoveryConcept::variants();
        for m in DoctrineDiscoveryCategory::morphisms() {
            if m.kind() == DoctrineDiscoveryRelationKind::Subsumption {
                prop_assert!(variants.contains(&m.source()));
                prop_assert!(variants.contains(&m.target()));
            }
        }
    }
}
