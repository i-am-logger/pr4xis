//! Tests for the Air21ProofStandardOntology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    Air21ProofStandardCategory, Air21ProofStandardConcept, Air21ProofStandardOntology,
    Air21StringencyOf, ContributingFactorBelowReferencePartition, PartitionCompleteness,
    ReferenceMinTierCoherence, is_leaf, leaves,
};
use crate::social::judicial::proof_standard::ontology::{
    ProofStandardConcept, StringencyOf as ReferenceStringency,
};
use pr4xis::category::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// =============================================================================
// Category laws and validation
// =============================================================================

#[test]
fn category_laws() {
    assert_category_laws::<Air21ProofStandardCategory>();
}

#[test]
fn ontology_validates() {
    Air21ProofStandardOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[test]
fn two_concepts_one_leaf() {
    assert_eq!(Air21ProofStandardConcept::variants().len(), 2);
    assert_eq!(leaves().len(), 1);
}

#[test]
fn contributing_factor_is_only_leaf() {
    assert!(is_leaf(Air21ProofStandardConcept::ContributingFactor));
    assert!(!is_leaf(Air21ProofStandardConcept::Air21ProofStandard));
}

// =============================================================================
// Air21StringencyOf — tier 0 below reference partition
// =============================================================================

#[test]
fn contributing_factor_is_tier_zero() {
    assert_eq!(
        Air21StringencyOf.get(&Air21ProofStandardConcept::ContributingFactor),
        Some(0)
    );
}

#[test]
fn root_has_no_tier() {
    assert_eq!(
        Air21StringencyOf.get(&Air21ProofStandardConcept::Air21ProofStandard),
        None
    );
}

#[test]
fn contributing_factor_strictly_below_reference_preponderance() {
    let air21 = Air21StringencyOf
        .get(&Air21ProofStandardConcept::ContributingFactor)
        .unwrap();
    let preponderance = ReferenceStringency
        .get(&ProofStandardConcept::Preponderance)
        .unwrap();
    assert!(
        air21 < preponderance,
        "AIR21 contributing-factor tier {air21} must be strictly below reference preponderance tier {preponderance}"
    );
}

#[test]
fn contributing_factor_below_all_reference_tiers() {
    let air21 = Air21StringencyOf
        .get(&Air21ProofStandardConcept::ContributingFactor)
        .unwrap();
    for c in [
        ProofStandardConcept::Preponderance,
        ProofStandardConcept::ClearAndConvincing,
        ProofStandardConcept::BeyondReasonableDoubt,
    ] {
        let ref_tier = ReferenceStringency.get(&c).unwrap();
        assert!(
            air21 < ref_tier,
            "ContributingFactor tier {air21} must be below {c:?} tier {ref_tier}"
        );
    }
}

// =============================================================================
// Axioms
// =============================================================================

#[test]
fn axiom_partition_completeness() {
    assert!(PartitionCompleteness.verify().is_ok());
}

#[test]
fn axiom_contributing_factor_below_reference_partition() {
    assert!(ContributingFactorBelowReferencePartition.verify().is_ok());
}

#[test]
fn axiom_reference_min_tier_coherence() {
    assert!(ReferenceMinTierCoherence.verify().is_ok());
}

#[test]
fn all_axioms_hold() {
    for axiom in Air21ProofStandardOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}
