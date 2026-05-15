//! Tests for the SoxProofStandardOntology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    ContributingFactorBelowReferencePartition, PartitionCompleteness, ReferenceMinTierCoherence,
    SoxProofStandardCategory, SoxProofStandardConcept, SoxProofStandardOntology, SoxStringencyOf,
    is_leaf, leaves,
};
use crate::social::judicial::proof_standard::ontology::{
    ProofStandardConcept, StringencyOf as ReferenceStringency,
};
use pr4xis::category::Concept;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};

// =============================================================================
// Category laws and validation
// =============================================================================

#[test]
fn category_laws() {
    assert_category_laws::<SoxProofStandardCategory>();
}

#[test]
fn ontology_validates() {
    SoxProofStandardOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[test]
fn two_concepts_one_leaf() {
    assert_eq!(SoxProofStandardConcept::variants().len(), 2);
    assert_eq!(leaves().len(), 1);
}

#[test]
fn contributing_factor_is_only_leaf() {
    assert!(is_leaf(SoxProofStandardConcept::ContributingFactor));
    assert!(!is_leaf(SoxProofStandardConcept::SoxProofStandard));
}

// =============================================================================
// SoxStringencyOf — tier 0 below reference partition
// =============================================================================

#[test]
fn contributing_factor_is_tier_zero() {
    assert_eq!(
        SoxStringencyOf.get(&SoxProofStandardConcept::ContributingFactor),
        Some(0)
    );
}

#[test]
fn root_has_no_tier() {
    assert_eq!(
        SoxStringencyOf.get(&SoxProofStandardConcept::SoxProofStandard),
        None
    );
}

#[test]
fn contributing_factor_strictly_below_reference_preponderance() {
    let sox = SoxStringencyOf
        .get(&SoxProofStandardConcept::ContributingFactor)
        .unwrap();
    let preponderance = ReferenceStringency
        .get(&ProofStandardConcept::Preponderance)
        .unwrap();
    assert!(
        sox < preponderance,
        "SOX contributing-factor tier {sox} must be strictly below reference preponderance tier {preponderance}"
    );
}

#[test]
fn contributing_factor_below_all_reference_tiers() {
    let sox = SoxStringencyOf
        .get(&SoxProofStandardConcept::ContributingFactor)
        .unwrap();
    for c in [
        ProofStandardConcept::Preponderance,
        ProofStandardConcept::ClearAndConvincing,
        ProofStandardConcept::BeyondReasonableDoubt,
    ] {
        let ref_tier = ReferenceStringency.get(&c).unwrap();
        assert!(
            sox < ref_tier,
            "ContributingFactor tier {sox} must be below {c:?} tier {ref_tier}"
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
    for axiom in SoxProofStandardOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}
