//! Tests for the TemporalConstraintOntology + Duration value type.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::Duration;
use super::ontology::{
    ApproximateDays, BusinessDayBelowWeek, GranularityOrdering, ImmediateIsZeroDuration,
    TemporalConstraintCategory, TemporalConstraintConcept, TemporalConstraintOntology,
    granularity_leaves, is_leaf,
};
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Concept, FinitelyGenerated};
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws and validation
// =============================================================================

#[test]
fn category_laws() {
    assert_category_laws::<TemporalConstraintCategory>();
}

#[test]
fn ontology_validates() {
    TemporalConstraintOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[test]
fn seven_concepts() {
    // TemporalConstraint root + 6 leaves (Immediate + 5 granularities).
    assert_eq!(TemporalConstraintConcept::variants().len(), 7);
}

#[test]
fn five_granularity_leaves() {
    assert_eq!(granularity_leaves().len(), 5);
}

// =============================================================================
// Duration value type
// =============================================================================

#[test]
fn duration_days() {
    let d = Duration::days(180);
    assert_eq!(d.unit, TemporalConstraintConcept::Day);
    assert_eq!(d.count, 180);
    assert!(!d.is_immediate());
}

#[test]
fn duration_immediate_is_zero_count() {
    let d = Duration::immediate();
    assert_eq!(d.unit, TemporalConstraintConcept::Immediate);
    assert!(d.is_immediate());
}

#[test]
fn duration_business_days() {
    let d = Duration::business_days(5);
    assert_eq!(d.unit, TemporalConstraintConcept::BusinessDay);
    assert_eq!(d.count, 5);
}

#[test]
fn duration_to_timex3_format() {
    assert_eq!(Duration::days(180).to_timex3(), "P180D");
    assert_eq!(Duration::weeks(2).to_timex3(), "P2W");
    assert_eq!(Duration::months(3).to_timex3(), "P3M");
    assert_eq!(Duration::years(1).to_timex3(), "P1Y");
    assert_eq!(Duration::immediate().to_timex3(), "PT0S");
}

#[test]
fn sox_180_day_sol_round_trips() {
    // The SOX 1514A statute of limitations is 180 days from violation
    // or knowledge thereof. Per 18 U.S.C. § 1514A(b)(2)(D).
    let d = Duration::days(180);
    assert_eq!(d.to_timex3(), "P180D");
    assert_eq!(d.count, 180);
}

// =============================================================================
// Axioms
// =============================================================================

#[test]
fn axiom_granularity_ordering() {
    assert!(GranularityOrdering.verify().is_ok());
}

#[test]
fn axiom_immediate_is_zero_duration() {
    assert!(ImmediateIsZeroDuration.verify().is_ok());
}

#[test]
fn axiom_business_day_below_week() {
    assert!(BusinessDayBelowWeek.verify().is_ok());
}

#[test]
fn all_axioms_hold() {
    for axiom in TemporalConstraintOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Property-based
// =============================================================================

fn arb_concept() -> impl Strategy<Value = TemporalConstraintConcept> {
    proptest::sample::select(TemporalConstraintConcept::variants())
}

proptest! {
    /// ApproximateDays is total on granularity leaves and None on root + Immediate.
    #[test]
    fn prop_approx_days_total_on_granularity(c in arb_concept()) {
        let v = ApproximateDays.get(&c);
        let is_granularity_leaf = matches!(
            c,
            TemporalConstraintConcept::Day
                | TemporalConstraintConcept::BusinessDay
                | TemporalConstraintConcept::Week
                | TemporalConstraintConcept::Month
                | TemporalConstraintConcept::Year
        );
        prop_assert_eq!(v.is_some(), is_granularity_leaf);
    }

    /// Duration::days(n).to_timex3() emits exactly "P{n}D" for any non-zero n.
    #[test]
    fn prop_days_timex3_round_trip(n in 1u32..10000) {
        let d = Duration::days(n);
        prop_assert_eq!(d.to_timex3(), format!("P{}D", n));
    }

    /// Duration::immediate() is the unique Duration with zero count and
    /// the Immediate unit.
    #[test]
    fn prop_immediate_is_canonical(_seed in any::<u32>()) {
        let d = Duration::immediate();
        prop_assert!(d.is_immediate());
        prop_assert_eq!(d.count, 0);
    }

    /// is_leaf includes Immediate and all five granularities.
    #[test]
    fn prop_root_not_a_leaf(c in arb_concept()) {
        if c == TemporalConstraintConcept::TemporalConstraint {
            prop_assert!(!is_leaf(c));
        } else {
            prop_assert!(is_leaf(c));
        }
    }
}
