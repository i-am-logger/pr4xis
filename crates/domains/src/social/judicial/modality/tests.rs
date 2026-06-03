//! Tests for the ObligationModalityOntology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    CompelsAction, DiscretionaryAndProhibitiveAreContradictories,
    MandatoryAndProhibitiveAreContraries, MandatoryImpliesPermitted, ObligationModalityCategory,
    ObligationModalityConcept, ObligationModalityOntology, ObligationModalityRelationKind,
    PartitionCompleteness, PermitsAction, classify_modal, classify_modal_pair, is_leaf, leaves,
};
use pr4xis::category::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category};
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws and validation
// =============================================================================

#[test]
fn category_laws() {
    assert_category_laws::<ObligationModalityCategory>();
}

#[test]
fn ontology_validates() {
    ObligationModalityOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[test]
fn four_concepts() {
    assert_eq!(ObligationModalityConcept::variants().len(), 4);
}

#[test]
fn three_leaves() {
    assert_eq!(leaves().len(), 3);
}

// =============================================================================
// Qualities
// =============================================================================

#[test]
fn mandatory_permits_and_compels() {
    let p = PermitsAction;
    let c = CompelsAction;
    assert_eq!(p.get(&ObligationModalityConcept::Mandatory), Some(true));
    assert_eq!(c.get(&ObligationModalityConcept::Mandatory), Some(true));
}

#[test]
fn prohibitive_neither_permits_nor_compels() {
    let p = PermitsAction;
    let c = CompelsAction;
    assert_eq!(p.get(&ObligationModalityConcept::Prohibitive), Some(false));
    assert_eq!(c.get(&ObligationModalityConcept::Prohibitive), Some(false));
}

#[test]
fn discretionary_permits_but_does_not_compel() {
    let p = PermitsAction;
    let c = CompelsAction;
    assert_eq!(p.get(&ObligationModalityConcept::Discretionary), Some(true));
    assert_eq!(
        c.get(&ObligationModalityConcept::Discretionary),
        Some(false)
    );
}

// =============================================================================
// Axioms
// =============================================================================

#[test]
fn axiom_partition_completeness() {
    assert!(PartitionCompleteness.verify().is_ok());
}

#[test]
fn axiom_mandatory_and_prohibitive_are_contraries() {
    assert!(MandatoryAndProhibitiveAreContraries.verify().is_ok());
}

#[test]
fn axiom_discretionary_and_prohibitive_are_contradictories() {
    assert!(
        DiscretionaryAndProhibitiveAreContradictories
            .verify()
            .is_ok()
    );
}

#[test]
fn axiom_mandatory_implies_permitted() {
    assert!(MandatoryImpliesPermitted.verify().is_ok());
}

#[test]
fn all_axioms_hold() {
    for axiom in ObligationModalityOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Modal-word recognizer (Halliday 1985 closed-marker set)
// =============================================================================

#[test]
fn classify_shall_as_mandatory() {
    assert_eq!(
        classify_modal("shall"),
        Some(ObligationModalityConcept::Mandatory)
    );
}

#[test]
fn classify_must_as_mandatory() {
    assert_eq!(
        classify_modal("must"),
        Some(ObligationModalityConcept::Mandatory)
    );
}

#[test]
fn classify_may_as_discretionary() {
    assert_eq!(
        classify_modal("may"),
        Some(ObligationModalityConcept::Discretionary)
    );
}

#[test]
fn classify_case_insensitive() {
    assert_eq!(
        classify_modal("SHALL"),
        Some(ObligationModalityConcept::Mandatory)
    );
    assert_eq!(
        classify_modal("Must"),
        Some(ObligationModalityConcept::Mandatory)
    );
}

#[test]
fn classify_pair_shall_not_as_prohibitive() {
    assert_eq!(
        classify_modal_pair("shall", "not"),
        Some(ObligationModalityConcept::Prohibitive)
    );
}

#[test]
fn classify_pair_must_not_as_prohibitive() {
    assert_eq!(
        classify_modal_pair("must", "not"),
        Some(ObligationModalityConcept::Prohibitive)
    );
}

#[test]
fn classify_pair_may_not_as_prohibitive() {
    assert_eq!(
        classify_modal_pair("may", "not"),
        Some(ObligationModalityConcept::Prohibitive)
    );
}

#[test]
fn classify_returns_none_on_non_modal() {
    assert_eq!(classify_modal("dog"), None);
    assert_eq!(classify_modal(""), None);
}

// =============================================================================
// Property-based
// =============================================================================

fn arb_concept() -> impl Strategy<Value = ObligationModalityConcept> {
    proptest::sample::select(ObligationModalityConcept::variants())
}

proptest! {
    /// PermitsAction is total on leaves and None on the abstract root.
    #[test]
    fn prop_permits_total_on_leaves(c in arb_concept()) {
        let v = PermitsAction.get(&c);
        if is_leaf(c) {
            prop_assert!(v.is_some());
        } else {
            prop_assert_eq!(v, None);
        }
    }

    /// CompelsAction is total on leaves and None on the abstract root.
    #[test]
    fn prop_compels_total_on_leaves(c in arb_concept()) {
        let v = CompelsAction.get(&c);
        if is_leaf(c) {
            prop_assert!(v.is_some());
        } else {
            prop_assert_eq!(v, None);
        }
    }

    /// von Wright entailment: if Compels(c) is true, Permits(c) must be true.
    #[test]
    fn prop_compels_entails_permits(c in arb_concept()) {
        if CompelsAction.get(&c) == Some(true) {
            prop_assert_eq!(PermitsAction.get(&c), Some(true));
        }
    }

    /// Mandatory is the unique concept that compels.
    #[test]
    fn prop_only_mandatory_compels(c in arb_concept()) {
        if c == ObligationModalityConcept::Mandatory {
            prop_assert_eq!(CompelsAction.get(&c), Some(true));
        } else if is_leaf(c) {
            prop_assert_eq!(CompelsAction.get(&c), Some(false));
        }
    }

    /// Every leaf is-a ObligationModality.
    #[test]
    fn prop_every_leaf_is_a_root(_seed in any::<u32>()) {
        let sub: std::collections::HashSet<_> = ObligationModalityCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == ObligationModalityRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for leaf in leaves() {
            prop_assert!(sub.contains(&(leaf, ObligationModalityConcept::ObligationModality)));
        }
    }
}
