//! Tests for the EvidenceRequirement / RequirementLevel ontology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    PartitionCompleteness, RequiredAndOptionalAreDuals, RequirementLevelCategory,
    RequirementLevelConcept, RequirementLevelOntology, Strictness, StrictnessIsTotalOrder, is_leaf,
    leaves, parse_rfc2119,
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
    assert_category_laws::<RequirementLevelCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ontology_validates() {
    RequirementLevelOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn four_concepts() {
    assert_eq!(RequirementLevelConcept::variants().len(), 4);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn three_leaves() {
    assert_eq!(leaves().len(), 3);
}

// =============================================================================
// RFC 2119 parser
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_must_as_required() {
    assert_eq!(
        parse_rfc2119("MUST"),
        Some(RequirementLevelConcept::Required)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_shall_as_required() {
    assert_eq!(
        parse_rfc2119("SHALL"),
        Some(RequirementLevelConcept::Required)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_should_as_recommended() {
    assert_eq!(
        parse_rfc2119("SHOULD"),
        Some(RequirementLevelConcept::Recommended)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_may_as_optional() {
    assert_eq!(
        parse_rfc2119("MAY"),
        Some(RequirementLevelConcept::Optional)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_case_insensitive() {
    assert_eq!(
        parse_rfc2119("must"),
        Some(RequirementLevelConcept::Required)
    );
    assert_eq!(
        parse_rfc2119("Should"),
        Some(RequirementLevelConcept::Recommended)
    );
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn parse_returns_none_on_non_keyword() {
    assert_eq!(parse_rfc2119("dog"), None);
    assert_eq!(parse_rfc2119(""), None);
}

// =============================================================================
// Strictness ordering
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn strictness_ordering() {
    let o = Strictness.get(&RequirementLevelConcept::Optional).unwrap();
    let r = Strictness
        .get(&RequirementLevelConcept::Recommended)
        .unwrap();
    let q = Strictness.get(&RequirementLevelConcept::Required).unwrap();
    assert!(o < r && r < q);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn root_has_no_strictness() {
    assert_eq!(
        Strictness.get(&RequirementLevelConcept::RequirementLevel),
        None
    );
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
fn axiom_strictness_is_total_order() {
    assert!(StrictnessIsTotalOrder.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_required_and_optional_are_duals() {
    assert!(RequiredAndOptionalAreDuals.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn all_axioms_hold() {
    for axiom in RequirementLevelOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Property-based
// =============================================================================

fn arb_concept() -> impl Strategy<Value = RequirementLevelConcept> {
    proptest::sample::select(RequirementLevelConcept::variants())
}

proptest! {
    /// Strictness is total on leaves, None on root.
    #[test]
    fn prop_strictness_total_on_leaves(c in arb_concept()) {
        let v = Strictness.get(&c);
        if is_leaf(c) {
            prop_assert!(v.is_some());
        } else {
            prop_assert_eq!(v, None);
        }
    }

    /// Round-trip: parse_rfc2119 of a canonical keyword maps to its concept.
    #[test]
    fn prop_canonical_keywords_round_trip(_seed in any::<u32>()) {
        prop_assert_eq!(parse_rfc2119("MUST"), Some(RequirementLevelConcept::Required));
        prop_assert_eq!(parse_rfc2119("SHALL"), Some(RequirementLevelConcept::Required));
        prop_assert_eq!(parse_rfc2119("REQUIRED"), Some(RequirementLevelConcept::Required));
        prop_assert_eq!(parse_rfc2119("SHOULD"), Some(RequirementLevelConcept::Recommended));
        prop_assert_eq!(parse_rfc2119("RECOMMENDED"), Some(RequirementLevelConcept::Recommended));
        prop_assert_eq!(parse_rfc2119("MAY"), Some(RequirementLevelConcept::Optional));
        prop_assert_eq!(parse_rfc2119("OPTIONAL"), Some(RequirementLevelConcept::Optional));
    }
}

pr4xis::register_praxis_value!(prop_strictness_total_on_leaves, Verifiable);
pr4xis::register_praxis_value!(prop_canonical_keywords_round_trip, Verifiable);
