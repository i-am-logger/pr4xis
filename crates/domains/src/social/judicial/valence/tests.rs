//! Tests for the ValenceOntology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    AdvancesMovingParty, MeritsAxisDuality, PartitionCompleteness, ProceduralIsOrthogonal,
    ValenceCategory, ValenceConcept, ValenceOntology, is_leaf, leaves,
};
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws and validation
// =============================================================================

#[test]
fn category_laws() {
    assert_category_laws::<ValenceCategory>();
}

#[test]
fn ontology_validates() {
    ValenceOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[test]
fn four_concepts() {
    // Root Valence + three leaves.
    assert_eq!(ValenceConcept::variants().len(), 4);
}

#[test]
fn three_leaves() {
    assert_eq!(leaves().len(), 3);
    for leaf in leaves() {
        assert!(is_leaf(leaf));
    }
}

#[test]
fn root_is_not_a_leaf() {
    assert!(!is_leaf(ValenceConcept::Valence));
}

// =============================================================================
// Quality: AdvancesMovingParty
// =============================================================================

#[test]
fn supportive_advances_moving_party() {
    assert_eq!(
        AdvancesMovingParty.get(&ValenceConcept::Supportive),
        Some(true)
    );
}

#[test]
fn defensive_does_not_advance_moving_party() {
    assert_eq!(
        AdvancesMovingParty.get(&ValenceConcept::Defensive),
        Some(false)
    );
}

#[test]
fn procedural_does_not_advance_on_merits() {
    assert_eq!(
        AdvancesMovingParty.get(&ValenceConcept::Procedural),
        Some(false)
    );
}

#[test]
fn root_is_unclassified() {
    assert_eq!(AdvancesMovingParty.get(&ValenceConcept::Valence), None);
}

// =============================================================================
// Axioms
// =============================================================================

#[test]
fn axiom_partition_completeness() {
    assert!(PartitionCompleteness.verify().is_ok());
}

#[test]
fn axiom_merits_axis_duality() {
    assert!(MeritsAxisDuality.verify().is_ok());
}

#[test]
fn axiom_procedural_is_orthogonal() {
    assert!(ProceduralIsOrthogonal.verify().is_ok());
}

#[test]
fn all_axioms_hold() {
    for axiom in ValenceOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Property-based
// =============================================================================

fn arb_concept() -> impl Strategy<Value = ValenceConcept> {
    proptest::sample::select(ValenceConcept::variants())
}

proptest! {
    /// AdvancesMovingParty is total on leaves and None on the abstract root.
    #[test]
    fn prop_quality_total_on_leaves(c in arb_concept()) {
        let v = AdvancesMovingParty.get(&c);
        if is_leaf(c) {
            prop_assert!(v.is_some(), "leaf {:?} should have a quality value", c);
        } else {
            prop_assert_eq!(v, None, "root should be unclassified");
        }
    }

    /// Every leaf is-a Valence (single-level taxonomy under the root).
    #[test]
    fn prop_every_leaf_is_a_valence(_seed in any::<u32>()) {
        use pr4xis::category::Arrow;
        let sub: Vec<_> = ValenceCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == super::ontology::ValenceRelationKind::Subsumption)
            .map(|m| (m.source(), m.target()))
            .collect();
        for leaf in leaves() {
            prop_assert!(
                sub.contains(&(leaf, ValenceConcept::Valence)),
                "leaf {:?} should is-a Valence",
                leaf
            );
        }
    }

    /// Opposition is symmetric and only between Supportive/Defensive.
    #[test]
    fn prop_opposition_only_on_merits_axis(_seed in any::<u32>()) {
        use pr4xis::category::Arrow;
        let opp: std::collections::HashSet<_> = ValenceCategory::morphisms()
            .into_iter()
            .filter(|m| m.kind() == super::ontology::ValenceRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        // Exactly the two merits-axis directions, no procedural pairs.
        prop_assert_eq!(opp.len(), 2);
        prop_assert!(opp.contains(&(ValenceConcept::Supportive, ValenceConcept::Defensive)));
        prop_assert!(opp.contains(&(ValenceConcept::Defensive, ValenceConcept::Supportive)));
    }

    /// Every arrow has a non-empty name (smoke check on Provenance).
    #[test]
    fn prop_every_arrow_named(_seed in any::<u32>()) {
        for m in ValenceCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }
}
