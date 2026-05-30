//! Tests for the PinpointCitation ontology + parser.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::PinpointCite;
use super::ontology::{
    NestingDepth, NestingDepthIsStrictTotalOrder, PinpointCitationCategory,
    PinpointCitationConcept, PinpointCitationOntology, is_leaf, leaves,
};
use pr4xis::category::Concept;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws + ontology validation
// =============================================================================

#[test]
fn category_laws() {
    assert_category_laws::<PinpointCitationCategory>();
}

#[test]
fn ontology_validates() {
    PinpointCitationOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[test]
fn seven_concepts() {
    // Root + 6 levels.
    assert_eq!(PinpointCitationConcept::variants().len(), 7);
}

#[test]
fn six_leaves() {
    assert_eq!(leaves().len(), 6);
}

// =============================================================================
// PinpointCite parser (Bluebook §3.3)
// =============================================================================

#[test]
fn parse_sox_subsection() {
    // SOX 1514A's prohibition-on-retaliation subsection is "(a)".
    let cite = PinpointCite::parse_subdivisions("(a)").unwrap();
    assert_eq!(cite.segments.len(), 1);
    assert_eq!(cite.segments[0].level, PinpointCitationConcept::Subsection);
    assert_eq!(cite.segments[0].label, "a");
}

#[test]
fn parse_sox_180_day_sol_pinpoint() {
    // The 180-day SOL is at 18 U.S.C. § 1514A(b)(2)(D) — three nested levels.
    let cite = PinpointCite::parse_subdivisions("(b)(2)(D)").unwrap();
    assert_eq!(cite.segments.len(), 3);
    assert_eq!(cite.segments[0].level, PinpointCitationConcept::Subsection);
    assert_eq!(cite.segments[0].label, "b");
    assert_eq!(cite.segments[1].level, PinpointCitationConcept::Paragraph);
    assert_eq!(cite.segments[1].label, "2");
    assert_eq!(
        cite.segments[2].level,
        PinpointCitationConcept::Subparagraph
    );
    assert_eq!(cite.segments[2].label, "D");
}

#[test]
fn parse_four_levels_deep() {
    let cite = PinpointCite::parse_subdivisions("(a)(1)(A)(ii)").unwrap();
    assert_eq!(cite.segments.len(), 4);
    assert_eq!(cite.segments[3].level, PinpointCitationConcept::Clause);
    assert_eq!(cite.segments[3].label, "ii");
}

#[test]
fn parse_rejects_unmatched_paren() {
    assert!(PinpointCite::parse_subdivisions("(a").is_none());
    assert!(PinpointCite::parse_subdivisions("a)").is_none());
}

#[test]
fn parse_rejects_special_chars() {
    assert!(PinpointCite::parse_subdivisions("(a-b)").is_none());
    assert!(PinpointCite::parse_subdivisions("(a/b)").is_none());
}

#[test]
fn parse_empty_string_yields_empty_cite() {
    let cite = PinpointCite::parse_subdivisions("").unwrap();
    assert!(cite.segments.is_empty());
    assert_eq!(cite.to_bluebook(), "");
}

#[test]
fn parse_to_bluebook_round_trips() {
    let cite = PinpointCite::parse_subdivisions("(b)(2)(D)").unwrap();
    assert_eq!(cite.to_bluebook(), "(b)(2)(D)");
}

#[test]
fn builder_pattern_works() {
    let cite = PinpointCite::new()
        .push(PinpointCitationConcept::Subsection, "a")
        .push(PinpointCitationConcept::Paragraph, "1");
    assert_eq!(cite.segments.len(), 2);
    assert_eq!(cite.to_bluebook(), "(a)(1)");
}

// =============================================================================
// Axioms
// =============================================================================

#[test]
fn axiom_nesting_depth_strict_total_order() {
    assert!(NestingDepthIsStrictTotalOrder.verify().is_ok());
}

#[test]
fn all_axioms_hold() {
    for axiom in PinpointCitationOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Property-based
// =============================================================================

fn arb_concept() -> impl Strategy<Value = PinpointCitationConcept> {
    proptest::sample::select(PinpointCitationConcept::variants())
}

proptest! {
    /// NestingDepth is total on leaves, None on root.
    #[test]
    fn prop_nesting_depth_total_on_leaves(c in arb_concept()) {
        let v = NestingDepth.get(&c);
        if is_leaf(c) {
            prop_assert!(v.is_some());
        } else {
            prop_assert_eq!(v, None);
        }
    }

    /// to_bluebook always emits the right number of parens (2 per segment).
    #[test]
    fn prop_bluebook_paren_count(seg_count in 0usize..6) {
        let mut cite = PinpointCite::new();
        for i in 0..seg_count {
            cite = cite.push(PinpointCitationConcept::Subsection, format!("{}", i));
        }
        let s = cite.to_bluebook();
        let open = s.chars().filter(|&c| c == '(').count();
        let close = s.chars().filter(|&c| c == ')').count();
        prop_assert_eq!(open, seg_count);
        prop_assert_eq!(close, seg_count);
    }
}
