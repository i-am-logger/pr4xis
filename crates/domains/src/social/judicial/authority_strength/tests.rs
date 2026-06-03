//! Tests for the AuthorityStrengthOntology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    AuthorityStrengthCategory, AuthorityStrengthConcept, AuthorityStrengthOntology,
    BINDING_TIER_FLOOR, BindingExceedsAllPersuasive, BindingForceOf, ConstitutionalSupremacy,
    ForceTiersAreDistinct, JurisdictionScopeOf, PartitionCompleteness, StatuteExceedsRegulation,
    SupremeCourtAtopPrecedentHierarchy, at_least_as_binding, binding_leaves, is_binding, is_leaf,
    is_persuasive, leaves, persuasive_leaves,
};
use pr4xis::category::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws and validation
// =============================================================================

#[test]
fn category_laws() {
    assert_category_laws::<AuthorityStrengthCategory>();
}

#[test]
fn ontology_validates() {
    AuthorityStrengthOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[test]
fn twelve_concepts() {
    // 1 root + 2 branches + 9 leaves.
    assert_eq!(AuthorityStrengthConcept::variants().len(), 12);
}

#[test]
fn nine_leaves() {
    assert_eq!(leaves().len(), 9);
}

#[test]
fn five_binding_four_persuasive() {
    assert_eq!(binding_leaves().len(), 5);
    assert_eq!(persuasive_leaves().len(), 4);
}

#[test]
fn binding_persuasive_partitions_leaves() {
    for c in leaves() {
        assert!(
            is_binding(c) ^ is_persuasive(c),
            "{c:?} must be exactly one of binding/persuasive"
        );
    }
}

// =============================================================================
// BindingForceOf — tier ordering
// =============================================================================

#[test]
fn constitutional_is_highest_tier() {
    assert_eq!(
        BindingForceOf.get(&AuthorityStrengthConcept::ConstitutionalText),
        Some(9)
    );
}

#[test]
fn secondary_source_is_lowest_tier() {
    assert_eq!(
        BindingForceOf.get(&AuthorityStrengthConcept::SecondarySource),
        Some(1)
    );
}

#[test]
fn root_and_branches_have_no_tier() {
    for c in [
        AuthorityStrengthConcept::AuthorityStrength,
        AuthorityStrengthConcept::BindingAuthority,
        AuthorityStrengthConcept::PersuasiveAuthority,
    ] {
        assert_eq!(BindingForceOf.get(&c), None);
    }
}

#[test]
fn binding_tier_floor_matches_controlling_circuit() {
    // BINDING_TIER_FLOOR should equal the lowest binding leaf's tier.
    let lowest_binding = BindingForceOf
        .get(&AuthorityStrengthConcept::ControllingCircuitPrecedent)
        .unwrap();
    assert_eq!(BINDING_TIER_FLOOR, lowest_binding);
}

#[test]
fn binding_tier_floor_excludes_arb() {
    // ARB is the highest persuasive tier; it sits below the floor.
    let arb = BindingForceOf
        .get(&AuthorityStrengthConcept::AdministrativeReviewBoardDecision)
        .unwrap();
    assert!(arb < BINDING_TIER_FLOOR);
}

#[test]
fn descending_tier_order() {
    // The leaves array is documented in descending order; verify.
    let mut last = 10u8;
    for c in leaves() {
        let t = BindingForceOf.get(&c).unwrap();
        assert!(t < last, "{c:?} (tier {t}) should be below previous {last}");
        last = t;
    }
}

#[test]
fn at_least_as_binding_total_on_leaves() {
    assert_eq!(
        at_least_as_binding(
            AuthorityStrengthConcept::ConstitutionalText,
            AuthorityStrengthConcept::SecondarySource,
        ),
        Some(true)
    );
    assert_eq!(
        at_least_as_binding(
            AuthorityStrengthConcept::SecondarySource,
            AuthorityStrengthConcept::ConstitutionalText,
        ),
        Some(false)
    );
}

#[test]
fn at_least_as_binding_returns_none_on_non_leaf() {
    assert_eq!(
        at_least_as_binding(
            AuthorityStrengthConcept::AuthorityStrength,
            AuthorityStrengthConcept::SecondarySource,
        ),
        None
    );
    assert_eq!(
        at_least_as_binding(
            AuthorityStrengthConcept::BindingAuthority,
            AuthorityStrengthConcept::ConstitutionalText,
        ),
        None
    );
}

// =============================================================================
// JurisdictionScopeOf — horizontal dimension
// =============================================================================

#[test]
fn universal_federal_scope_for_constitution_statute_scotus() {
    let q = JurisdictionScopeOf;
    for c in [
        AuthorityStrengthConcept::ConstitutionalText,
        AuthorityStrengthConcept::FederalStatute,
        AuthorityStrengthConcept::SupremeCourtPrecedent,
        AuthorityStrengthConcept::FederalRegulation,
    ] {
        let scope = q.get(&c).expect("federal authorities carry scope");
        assert_eq!(scope.value(), "jurisdiction:us_federal");
    }
}

#[test]
fn controlling_circuit_has_placeholder_scope() {
    let scope = JurisdictionScopeOf
        .get(&AuthorityStrengthConcept::ControllingCircuitPrecedent)
        .expect("controlling-circuit carries placeholder scope");
    assert_eq!(scope.value(), "jurisdiction:single_circuit");
}

#[test]
fn persuasive_concepts_have_no_binding_scope() {
    let q = JurisdictionScopeOf;
    for c in persuasive_leaves() {
        assert_eq!(
            q.get(&c),
            None,
            "{c:?} should have no binding-scope at type level"
        );
    }
}

#[test]
fn abstract_concepts_have_no_scope() {
    let q = JurisdictionScopeOf;
    for c in [
        AuthorityStrengthConcept::AuthorityStrength,
        AuthorityStrengthConcept::BindingAuthority,
        AuthorityStrengthConcept::PersuasiveAuthority,
    ] {
        assert_eq!(q.get(&c), None);
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
fn axiom_binding_exceeds_all_persuasive() {
    assert!(BindingExceedsAllPersuasive.verify().is_ok());
}

#[test]
fn axiom_constitutional_supremacy() {
    assert!(ConstitutionalSupremacy.verify().is_ok());
}

#[test]
fn axiom_statute_exceeds_regulation() {
    assert!(StatuteExceedsRegulation.verify().is_ok());
}

#[test]
fn axiom_supreme_court_atop_precedent() {
    assert!(SupremeCourtAtopPrecedentHierarchy.verify().is_ok());
}

#[test]
fn axiom_force_tiers_are_distinct() {
    assert!(ForceTiersAreDistinct.verify().is_ok());
}

#[test]
fn all_axioms_hold() {
    for axiom in AuthorityStrengthOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Property-based
// =============================================================================

fn arb_concept() -> impl Strategy<Value = AuthorityStrengthConcept> {
    proptest::sample::select(AuthorityStrengthConcept::variants())
}

proptest! {
    /// Every leaf has Some tier; every non-leaf has None.
    #[test]
    fn prop_tier_total_on_leaves(c in arb_concept()) {
        let v = BindingForceOf.get(&c);
        if is_leaf(c) {
            prop_assert!(v.is_some());
        } else {
            prop_assert_eq!(v, None);
        }
    }

    /// Binding leaves have tier >= BINDING_TIER_FLOOR; persuasive < FLOOR.
    #[test]
    fn prop_binding_floor_separates(c in arb_concept()) {
        if is_binding(c) {
            let t = BindingForceOf.get(&c).unwrap();
            prop_assert!(t >= BINDING_TIER_FLOOR);
        }
        if is_persuasive(c) {
            let t = BindingForceOf.get(&c).unwrap();
            prop_assert!(t < BINDING_TIER_FLOOR);
        }
    }

    /// `at_least_as_binding` is reflexive on leaves.
    #[test]
    fn prop_at_least_reflexive(c in arb_concept()) {
        if is_leaf(c) {
            prop_assert_eq!(at_least_as_binding(c, c), Some(true));
        }
    }

    /// Antisymmetric: a >= b and b >= a => a == b.
    #[test]
    fn prop_at_least_antisymmetric(a in arb_concept(), b in arb_concept()) {
        if is_leaf(a) && is_leaf(b) {
            let ab = at_least_as_binding(a, b);
            let ba = at_least_as_binding(b, a);
            if ab == Some(true) && ba == Some(true) {
                prop_assert_eq!(a, b);
            }
        }
    }

    /// Transitive: a >= b and b >= c => a >= c.
    #[test]
    fn prop_at_least_transitive(
        a in arb_concept(),
        b in arb_concept(),
        c in arb_concept(),
    ) {
        if is_leaf(a) && is_leaf(b) && is_leaf(c) {
            let ab = at_least_as_binding(a, b);
            let bc = at_least_as_binding(b, c);
            let ac = at_least_as_binding(a, c);
            if ab == Some(true) && bc == Some(true) {
                prop_assert_eq!(ac, Some(true));
            }
        }
    }

    /// All nine leaf tiers are distinct.
    #[test]
    fn prop_tiers_distinct(_seed in any::<u32>()) {
        let mut tiers: Vec<u8> = leaves().iter().map(|c| BindingForceOf.get(c).unwrap()).collect();
        let original = tiers.len();
        tiers.sort();
        tiers.dedup();
        prop_assert_eq!(tiers.len(), original);
    }

    /// Constitutional text dominates every other leaf.
    #[test]
    fn prop_constitutional_dominates(c in arb_concept()) {
        if is_leaf(c) && c != AuthorityStrengthConcept::ConstitutionalText {
            prop_assert_eq!(
                at_least_as_binding(AuthorityStrengthConcept::ConstitutionalText, c),
                Some(true)
            );
        }
    }
}
