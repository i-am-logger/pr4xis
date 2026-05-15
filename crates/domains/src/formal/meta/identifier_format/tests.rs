//! Tests for the IdentifierFormatOntology + Identifier value type.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    EveryLeafHasResolverClassification, HasResolver, IdentifierFormatCategory,
    IdentifierFormatConcept, IdentifierFormatOntology, PartitionCompleteness, is_leaf, leaves,
};
use super::{Identifier, IdentifierParseError};
use pr4xis::category::Concept;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws + validation
// =============================================================================

#[test]
fn category_laws() {
    assert_category_laws::<IdentifierFormatCategory>();
}

#[test]
fn ontology_validates() {
    IdentifierFormatOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[test]
fn five_concepts() {
    assert_eq!(IdentifierFormatConcept::variants().len(), 5);
}

#[test]
fn four_leaves() {
    assert_eq!(leaves().len(), 4);
}

// =============================================================================
// CURIE parser (W3C CURIE 1.0 §2)
// =============================================================================

#[test]
fn curie_accepts_sox_term_id() {
    let id = Identifier::curie("sox_1514a:a").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Curie);
    assert_eq!(id.value, "sox_1514a:a");
}

#[test]
fn curie_accepts_nested_local() {
    let id = Identifier::curie("sox_1514a:b1a").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Curie);
}

#[test]
fn curie_rejects_empty() {
    assert_eq!(Identifier::curie(""), Err(IdentifierParseError::Empty));
}

#[test]
fn curie_rejects_no_colon() {
    assert!(matches!(
        Identifier::curie("sox_1514a_a"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[test]
fn curie_rejects_empty_prefix_or_local() {
    assert!(matches!(
        Identifier::curie(":local"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
    assert!(matches!(
        Identifier::curie("prefix:"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

// =============================================================================
// UUID parser (RFC 4122 §3)
// =============================================================================

#[test]
fn uuid_accepts_canonical_form() {
    let id = Identifier::uuid("550e8400-e29b-41d4-a716-446655440000").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Uuid);
}

#[test]
fn uuid_rejects_wrong_length() {
    assert!(matches!(
        Identifier::uuid("550e8400"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[test]
fn uuid_rejects_misplaced_hyphens() {
    assert!(matches!(
        Identifier::uuid("550e8400e-29b-41d4-a716-446655440000"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[test]
fn uuid_rejects_non_hex() {
    assert!(matches!(
        Identifier::uuid("550e8400-e29b-41d4-a716-44665544000Z"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

// =============================================================================
// URI parser (RFC 3986 §3)
// =============================================================================

#[test]
fn uri_accepts_https() {
    let id = Identifier::uri("https://example.com/").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Uri);
}

#[test]
fn uri_accepts_urn() {
    let id = Identifier::uri("urn:isbn:0451450523").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Uri);
}

#[test]
fn uri_rejects_no_scheme() {
    assert!(matches!(
        Identifier::uri("//example.com/"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[test]
fn uri_rejects_numeric_scheme_start() {
    assert!(matches!(
        Identifier::uri("1http://x"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

// =============================================================================
// OID parser (ISO 8824-1 §32)
// =============================================================================

#[test]
fn oid_accepts_canonical_form() {
    let id = Identifier::oid("1.3.6.1.4.1").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Oid);
}

#[test]
fn oid_rejects_single_arc() {
    assert!(matches!(
        Identifier::oid("1"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[test]
fn oid_rejects_non_numeric_arc() {
    assert!(matches!(
        Identifier::oid("1.3.6.a"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[test]
fn oid_rejects_empty_arc() {
    assert!(matches!(
        Identifier::oid("1..3"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

// =============================================================================
// HasResolver classification
// =============================================================================

#[test]
fn curie_no_resolver() {
    assert_eq!(
        HasResolver.get(&IdentifierFormatConcept::Curie),
        Some(false)
    );
}

#[test]
fn uri_has_resolver() {
    assert_eq!(HasResolver.get(&IdentifierFormatConcept::Uri), Some(true));
}

#[test]
fn oid_has_resolver() {
    assert_eq!(HasResolver.get(&IdentifierFormatConcept::Oid), Some(true));
}

#[test]
fn uuid_no_resolver() {
    assert_eq!(HasResolver.get(&IdentifierFormatConcept::Uuid), Some(false));
}

// =============================================================================
// Axioms
// =============================================================================

#[test]
fn axiom_partition_completeness() {
    assert!(PartitionCompleteness.verify().is_ok());
}

#[test]
fn axiom_every_leaf_has_resolver_classification() {
    assert!(EveryLeafHasResolverClassification.verify().is_ok());
}

#[test]
fn all_axioms_hold() {
    for axiom in IdentifierFormatOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Property-based
// =============================================================================

fn arb_concept() -> impl Strategy<Value = IdentifierFormatConcept> {
    proptest::sample::select(IdentifierFormatConcept::variants())
}

proptest! {
    #[test]
    fn prop_has_resolver_total_on_leaves(c in arb_concept()) {
        let v = HasResolver.get(&c);
        if is_leaf(c) {
            prop_assert!(v.is_some());
        } else {
            prop_assert_eq!(v, None);
        }
    }

    /// Random CURIE-shaped strings always parse.
    #[test]
    fn prop_curie_parses_valid(prefix in "[a-z][a-z0-9_]{0,20}", local in "[a-z0-9_]{1,20}") {
        let s = format!("{prefix}:{local}");
        prop_assert!(Identifier::curie(s).is_ok());
    }
}
