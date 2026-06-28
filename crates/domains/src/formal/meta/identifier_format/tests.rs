//! Tests for the IdentifierFormatOntology + Identifier value type.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    EveryLeafHasResolverClassification, HasResolver, IdentifierFormatCategory,
    IdentifierFormatConcept, IdentifierFormatOntology, PartitionCompleteness, is_leaf, leaves,
};
use super::{Identifier, IdentifierParseError};
use pr4xis::category::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws + validation
// =============================================================================

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn category_laws() {
    assert_category_laws::<IdentifierFormatCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ontology_validates() {
    IdentifierFormatOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn six_concepts() {
    // Root + five leaves (CURIE, UUID, URI, OID, USLM URN).
    assert_eq!(IdentifierFormatConcept::variants().len(), 6);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn five_leaves() {
    assert_eq!(leaves().len(), 5);
    let l = leaves();
    assert!(l.contains(&IdentifierFormatConcept::Curie));
    assert!(l.contains(&IdentifierFormatConcept::Uuid));
    assert!(l.contains(&IdentifierFormatConcept::Uri));
    assert!(l.contains(&IdentifierFormatConcept::Oid));
    assert!(l.contains(&IdentifierFormatConcept::UslmUrn));
}

// =============================================================================
// CURIE parser (W3C CURIE 1.0 §2)
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn curie_accepts_sox_term_id() {
    let id = Identifier::curie("sox_1514a:a").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Curie);
    assert_eq!(id.value(), "sox_1514a:a");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn curie_accepts_nested_local() {
    let id = Identifier::curie("sox_1514a:b1a").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Curie);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn curie_rejects_empty() {
    assert_eq!(Identifier::curie(""), Err(IdentifierParseError::Empty));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn curie_rejects_no_colon() {
    assert!(matches!(
        Identifier::curie("sox_1514a_a"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[pr4xis::praxis_value(Honest)]
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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn uuid_accepts_canonical_form() {
    let id = Identifier::uuid("550e8400-e29b-41d4-a716-446655440000").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Uuid);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn uuid_rejects_wrong_length() {
    assert!(matches!(
        Identifier::uuid("550e8400"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn uuid_rejects_misplaced_hyphens() {
    assert!(matches!(
        Identifier::uuid("550e8400e-29b-41d4-a716-446655440000"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[pr4xis::praxis_value(Honest)]
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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn uri_accepts_https() {
    let id = Identifier::uri("https://example.com/").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Uri);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn uri_accepts_urn() {
    let id = Identifier::uri("urn:isbn:0451450523").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Uri);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn uri_rejects_no_scheme() {
    assert!(matches!(
        Identifier::uri("//example.com/"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[pr4xis::praxis_value(Honest)]
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

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn oid_accepts_canonical_form() {
    let id = Identifier::oid("1.3.6.1.4.1").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::Oid);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn oid_rejects_single_arc() {
    assert!(matches!(
        Identifier::oid("1"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn oid_rejects_non_numeric_arc() {
    assert!(matches!(
        Identifier::oid("1.3.6.a"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn oid_rejects_empty_arc() {
    assert!(matches!(
        Identifier::oid("1..3"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

// =============================================================================
// USLM URN parser (LRC USLM User Guide §11.5 Identifiers; 1 U.S.C. § 204)
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn uslm_urn_accepts_title_identifier() {
    let id = Identifier::uslm_urn("/us/usc/t18").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::UslmUrn);
    assert_eq!(id.value(), "/us/usc/t18");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn uslm_urn_accepts_section_identifier() {
    let id = Identifier::uslm_urn("/us/usc/t18/s1514A").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::UslmUrn);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn uslm_urn_accepts_subdivision_identifier() {
    let id = Identifier::uslm_urn("/us/usc/t18/s1514A/a/1/A").unwrap();
    assert_eq!(id.format, IdentifierFormatConcept::UslmUrn);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn uslm_urn_accepts_hyphenated_section_number() {
    // Title 49's pub-law-derived section numbers like § 78j-1.
    assert!(Identifier::uslm_urn("/us/usc/t15/s78j-1").is_ok());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn uslm_urn_rejects_empty() {
    assert_eq!(Identifier::uslm_urn(""), Err(IdentifierParseError::Empty));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn uslm_urn_rejects_missing_us_prefix() {
    assert!(matches!(
        Identifier::uslm_urn("/usc/t18"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn uslm_urn_rejects_non_absolute_path() {
    assert!(matches!(
        Identifier::uslm_urn("us/usc/t18"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn uslm_urn_rejects_double_slash() {
    assert!(matches!(
        Identifier::uslm_urn("/us//t18"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn uslm_urn_rejects_disallowed_characters() {
    // Spaces are not permitted in path segments.
    assert!(matches!(
        Identifier::uslm_urn("/us/usc/t 18"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
    // `?` (query) is not part of the USLM URN grammar.
    assert!(matches!(
        Identifier::uslm_urn("/us/usc/t18?q=1"),
        Err(IdentifierParseError::InvalidGrammar { .. })
    ));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn uslm_urn_has_resolver() {
    use crate::formal::meta::identifier_format::ontology::HasResolver;
    use pr4xis::ontology::Quality;
    let q = HasResolver;
    assert_eq!(q.get(&IdentifierFormatConcept::UslmUrn), Some(true));
}

// =============================================================================
// HasResolver classification
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn curie_no_resolver() {
    assert_eq!(
        HasResolver.get(&IdentifierFormatConcept::Curie),
        Some(false)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn uri_has_resolver() {
    assert_eq!(HasResolver.get(&IdentifierFormatConcept::Uri), Some(true));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn oid_has_resolver() {
    assert_eq!(HasResolver.get(&IdentifierFormatConcept::Oid), Some(true));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn uuid_no_resolver() {
    assert_eq!(HasResolver.get(&IdentifierFormatConcept::Uuid), Some(false));
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
fn axiom_every_leaf_has_resolver_classification() {
    assert!(EveryLeafHasResolverClassification.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
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

pr4xis::register_praxis_value!(prop_has_resolver_total_on_leaves, Verifiable);
pr4xis::register_praxis_value!(prop_curie_parses_valid, Verifiable);
