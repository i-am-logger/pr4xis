//! Tests for the ProofStandardOntology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    BeyondReasonableDoubtIsMostStringent, PartitionCompleteness, ProofStandardCategory,
    ProofStandardConcept, ProofStandardOntology, StringencyIsTotalOnLeaves, StringencyOf,
    at_least_as_stringent, is_leaf, leaves,
};
use pr4xis::category::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws and validation
// =============================================================================

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn category_laws() {
    assert_category_laws::<ProofStandardCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ontology_validates() {
    ProofStandardOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn four_concepts() {
    // Root + 3 leaves.
    assert_eq!(ProofStandardConcept::variants().len(), 4);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn three_leaves() {
    assert_eq!(leaves().len(), 3);
}

// =============================================================================
// Stringency ordering
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn preponderance_lowest() {
    assert_eq!(
        StringencyOf.get(&ProofStandardConcept::Preponderance),
        Some(1)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn beyond_reasonable_doubt_highest() {
    assert_eq!(
        StringencyOf.get(&ProofStandardConcept::BeyondReasonableDoubt),
        Some(3)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn preponderance_below_clear_and_convincing() {
    let p = StringencyOf
        .get(&ProofStandardConcept::Preponderance)
        .unwrap();
    let cc = StringencyOf
        .get(&ProofStandardConcept::ClearAndConvincing)
        .unwrap();
    assert!(
        p < cc,
        "preponderance ({}) < clear-and-convincing ({})",
        p,
        cc
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn at_least_as_stringent_is_total() {
    assert_eq!(
        at_least_as_stringent(
            ProofStandardConcept::BeyondReasonableDoubt,
            ProofStandardConcept::Preponderance,
        ),
        Some(true)
    );
    assert_eq!(
        at_least_as_stringent(
            ProofStandardConcept::Preponderance,
            ProofStandardConcept::BeyondReasonableDoubt,
        ),
        Some(false)
    );
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn root_has_no_stringency() {
    assert_eq!(StringencyOf.get(&ProofStandardConcept::ProofStandard), None);
}

// =============================================================================
// Axioms
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_partition_completeness() {
    assert!(PartitionCompleteness.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_stringency_total_on_leaves() {
    assert!(StringencyIsTotalOnLeaves.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_beyond_reasonable_doubt_most_stringent() {
    assert!(BeyondReasonableDoubtIsMostStringent.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn all_axioms_hold() {
    for axiom in ProofStandardOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Property-based
// =============================================================================

fn arb_concept() -> impl Strategy<Value = ProofStandardConcept> {
    proptest::sample::select(ProofStandardConcept::variants())
}

proptest! {
    /// Stringency is total on leaves, None on root.
    #[test]
    fn prop_stringency_total_on_leaves(c in arb_concept()) {
        let v = StringencyOf.get(&c);
        if is_leaf(c) {
            prop_assert!(v.is_some());
        } else {
            prop_assert_eq!(v, None);
        }
    }

    /// `at_least_as_stringent` is reflexive on leaves.
    #[test]
    fn prop_at_least_reflexive(c in arb_concept()) {
        if is_leaf(c) {
            prop_assert_eq!(at_least_as_stringent(c, c), Some(true));
        }
    }

    /// Antisymmetric: a >= b and b >= a implies a == b.
    #[test]
    fn prop_at_least_antisymmetric(a in arb_concept(), b in arb_concept()) {
        if is_leaf(a) && is_leaf(b) {
            let ab = at_least_as_stringent(a, b);
            let ba = at_least_as_stringent(b, a);
            if ab == Some(true) && ba == Some(true) {
                prop_assert_eq!(a, b);
            }
        }
    }

    /// Transitive: a >= b and b >= c implies a >= c.
    #[test]
    fn prop_at_least_transitive(a in arb_concept(), b in arb_concept(), c in arb_concept()) {
        if is_leaf(a) && is_leaf(b) && is_leaf(c) {
            let ab = at_least_as_stringent(a, b);
            let bc = at_least_as_stringent(b, c);
            let ac = at_least_as_stringent(a, c);
            if ab == Some(true) && bc == Some(true) {
                prop_assert_eq!(ac, Some(true));
            }
        }
    }

    /// Every leaf has a unique stringency tier.
    #[test]
    fn prop_tiers_unique(_seed in any::<u32>()) {
        let mut tiers: Vec<u8> = leaves().iter().map(|c| StringencyOf.get(c).unwrap()).collect();
        tiers.sort();
        let original_len = tiers.len();
        tiers.dedup();
        prop_assert_eq!(tiers.len(), original_len);
    }
}

pr4xis::register_praxis_value!(prop_stringency_total_on_leaves, Verifiable);
pr4xis::register_praxis_value!(prop_at_least_reflexive, Verifiable);
pr4xis::register_praxis_value!(prop_at_least_antisymmetric, Verifiable);
pr4xis::register_praxis_value!(prop_at_least_transitive, Verifiable);
pr4xis::register_praxis_value!(prop_tiers_unique, Verifiable);
