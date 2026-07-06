//! Tests for the LegalSourcesOntology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    IsEnactedOf, LegalSourcesCategory, LegalSourcesConcept, LegalSourcesOntology,
    PrecedentIsNotADocument, SourceTypesUnderGenus, StatuteIsALaw, concrete_source_types,
    subsumes_transitively,
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
    assert_category_laws::<LegalSourcesCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ontology_validates() {
    LegalSourcesOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept surface
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn nine_concepts() {
    // LegalSource + LegalDocument + 5 enacted species + Precedent + CustomaryLaw.
    assert_eq!(LegalSourcesConcept::variants().len(), 9);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn seven_concrete_source_types() {
    assert_eq!(concrete_source_types().len(), 7);
}

// =============================================================================
// The crux — transitive subsumption closure
// =============================================================================

/// Statute ⊑ LegalDocument ⊑ LegalSource — the closure the ontology is
/// built to guarantee. A statute reaches the genus by subsumption.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn statute_reaches_legalsource_by_subsumption() {
    assert!(subsumes_transitively(
        LegalSourcesConcept::Statute,
        LegalSourcesConcept::LegalSource
    ));
    // …and it passes through LegalDocument (the intermediate species).
    assert!(subsumes_transitively(
        LegalSourcesConcept::Statute,
        LegalSourcesConcept::LegalDocument
    ));
    assert!(subsumes_transitively(
        LegalSourcesConcept::LegalDocument,
        LegalSourcesConcept::LegalSource
    ));
}

/// Precedent reaches the genus directly, NOT through LegalDocument —
/// faithful to LKIF-Core norm.owl.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn precedent_reaches_source_but_not_document() {
    assert!(subsumes_transitively(
        LegalSourcesConcept::Precedent,
        LegalSourcesConcept::LegalSource
    ));
    assert!(!subsumes_transitively(
        LegalSourcesConcept::Precedent,
        LegalSourcesConcept::LegalDocument
    ));
}

// =============================================================================
// IsEnactedOf — Salmond enacted vs. unenacted
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn enacted_species_are_enacted() {
    let q = IsEnactedOf;
    for c in [
        LegalSourcesConcept::Statute,
        LegalSourcesConcept::Regulation,
        LegalSourcesConcept::Constitution,
        LegalSourcesConcept::Treaty,
        LegalSourcesConcept::Code,
    ] {
        assert_eq!(q.get(&c), Some(true), "{c:?} should be enacted");
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn unenacted_sources_are_unenacted() {
    let q = IsEnactedOf;
    assert_eq!(q.get(&LegalSourcesConcept::Precedent), Some(false));
    assert_eq!(q.get(&LegalSourcesConcept::CustomaryLaw), Some(false));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn abstract_genera_have_no_enactment_status() {
    let q = IsEnactedOf;
    assert_eq!(q.get(&LegalSourcesConcept::LegalSource), None);
    assert_eq!(q.get(&LegalSourcesConcept::LegalDocument), None);
}

// =============================================================================
// Axioms
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_statute_is_a_law() {
    assert!(StatuteIsALaw.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_source_types_under_genus() {
    assert!(SourceTypesUnderGenus.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_precedent_is_not_a_document() {
    assert!(PrecedentIsNotADocument.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn all_axioms_hold() {
    for axiom in LegalSourcesOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Property-based
// =============================================================================

fn arb_concept() -> impl Strategy<Value = LegalSourcesConcept> {
    proptest::sample::select(LegalSourcesConcept::variants())
}

proptest! {
    /// Every concrete source type reaches the genus LegalSource.
    #[test]
    fn prop_concrete_sources_reach_genus(c in arb_concept()) {
        if concrete_source_types().contains(&c) {
            prop_assert!(subsumes_transitively(c, LegalSourcesConcept::LegalSource));
        }
    }

    /// Enactment status is total on concrete sources and absent on the
    /// two abstract genera.
    #[test]
    fn prop_enactment_total_on_concrete(c in arb_concept()) {
        let v = IsEnactedOf.get(&c);
        let abstract_genus = c == LegalSourcesConcept::LegalSource
            || c == LegalSourcesConcept::LegalDocument;
        if abstract_genus {
            prop_assert_eq!(v, None);
        } else {
            prop_assert!(v.is_some());
        }
    }

    /// Strict subsumption is IRREFLEXIVE: no concept strictly subsumes
    /// itself. The `Subsumption` closure carries only the strict is_a edges
    /// (an identity is the `Identity` kind, not `Subsumption`), so the
    /// taxonomy is a DAG with no self-loops. (Reflexive "x is-a x" is a
    /// separate reflexive-kind query, not this one.)
    #[test]
    fn prop_no_strict_self_subsumption(c in arb_concept()) {
        prop_assert!(!subsumes_transitively(c, c));
    }
}

pr4xis::register_praxis_value!(prop_concrete_sources_reach_genus, Verifiable);
pr4xis::register_praxis_value!(prop_enactment_total_on_concrete, Verifiable);
pr4xis::register_praxis_value!(prop_no_strict_self_subsumption, Verifiable);
