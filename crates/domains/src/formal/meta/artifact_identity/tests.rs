//! Tests for the artifact_identity ontology — taxonomy structure, family
//! axioms, real extractor implementations, and property-based tests.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    ArtifactIdentityCategory, ArtifactIdentityOntology, ClaimData, ContentHashIsInjective,
    ContentHashIsOffline, EverySchemeHasAnExtractor, IdentityClaim, IdentityConcept,
    PersistentIdentifierRequiresResolver, SelfDescribingIsWeakestTrust, TrustTier, TrustTierOf,
    VerifiabilityOffline, VerificationResult, ancestors_of, is_family, is_leaf,
};
use super::schemes::{raw_hash, xml_element_attribute};
use pr4xis::category::Concept;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws and validation
// =============================================================================

#[test]
fn category_laws() {
    assert_category_laws::<ArtifactIdentityCategory>();
}

#[test]
fn ontology_validates() {
    ArtifactIdentityOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Entity surface — 25 total (1 root + 4 families + 20 leaves)
// =============================================================================

#[test]
fn twenty_five_identity_concepts() {
    assert_eq!(IdentityConcept::variants().len(), 25);
}

#[test]
fn four_families() {
    let families: Vec<_> = IdentityConcept::variants()
        .into_iter()
        .filter(is_family)
        .collect();
    assert_eq!(families.len(), 4);
}

#[test]
fn twenty_leaves() {
    let leaves: Vec<_> = IdentityConcept::variants()
        .into_iter()
        .filter(is_leaf)
        .collect();
    assert_eq!(leaves.len(), 20);
}

#[test]
fn root_is_identity() {
    assert!(!is_family(&IdentityConcept::Identity));
    assert!(!is_leaf(&IdentityConcept::Identity));
}

// =============================================================================
// Taxonomy — every leaf has exactly one family ancestor at level 1
// =============================================================================

#[test]
fn every_leaf_has_a_family_ancestor() {
    for concept in IdentityConcept::variants() {
        if !is_leaf(&concept) {
            continue;
        }
        let ancestors = ancestors_of(&concept);
        let family_ancestors: Vec<_> = ancestors.iter().filter(|a| is_family(a)).collect();
        assert_eq!(
            family_ancestors.len(),
            1,
            "{:?} should have exactly one family ancestor, got {:?}",
            concept,
            family_ancestors
        );
    }
}

#[test]
fn content_hash_family_has_five_leaves() {
    use IdentityConcept as I;
    for leaf in [
        I::RawHash,
        I::GitObjectSha,
        I::IpfsCid,
        I::NixStorePath,
        I::BittorrentInfoHash,
    ] {
        assert!(ancestors_of(&leaf).contains(&I::ContentHash));
    }
}

#[test]
fn persistent_identifier_family_has_four_leaves() {
    use IdentityConcept as I;
    for leaf in [I::Doi, I::Handle, I::Ark, I::Purl] {
        assert!(ancestors_of(&leaf).contains(&I::PersistentIdentifier));
    }
}

// =============================================================================
// Qualities
// =============================================================================

#[test]
fn content_hash_leaves_are_offline() {
    use IdentityConcept as I;
    let q = VerifiabilityOffline;
    for leaf in [
        I::RawHash,
        I::GitObjectSha,
        I::IpfsCid,
        I::NixStorePath,
        I::BittorrentInfoHash,
    ] {
        assert_eq!(q.get(&leaf), Some(true));
    }
}

#[test]
fn persistent_identifier_leaves_are_online() {
    use IdentityConcept as I;
    let q = VerifiabilityOffline;
    for leaf in [I::Doi, I::Handle, I::Ark, I::Purl] {
        assert_eq!(q.get(&leaf), Some(false));
    }
}

#[test]
fn self_describing_leaves_are_declarative() {
    use IdentityConcept as I;
    let q = TrustTierOf;
    for leaf in [
        I::OwlVersionIri,
        I::OwlVersionInfo,
        I::DctIdentifier,
        I::XmlElementAttribute,
        I::XmlSchemaVersion,
    ] {
        assert_eq!(q.get(&leaf), Some(TrustTier::Declarative));
    }
}

// =============================================================================
// Domain axioms
// =============================================================================

#[test]
fn axiom_every_scheme_has_an_extractor() {
    assert!(EverySchemeHasAnExtractor.verify().is_ok());
}

#[test]
fn axiom_content_hash_is_injective() {
    assert!(ContentHashIsInjective.verify().is_ok());
}

#[test]
fn axiom_content_hash_is_offline() {
    assert!(ContentHashIsOffline.verify().is_ok());
}

#[test]
fn axiom_persistent_identifier_requires_resolver() {
    assert!(PersistentIdentifierRequiresResolver.verify().is_ok());
}

#[test]
fn axiom_self_describing_is_weakest_trust() {
    assert!(SelfDescribingIsWeakestTrust.verify().is_ok());
}

#[test]
fn all_axioms_hold() {
    for axiom in ArtifactIdentityOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!(
                "axiom failed: {} - {}",
                c.meta().name.as_str(),
                c.meta().description.as_str()
            );
        }
    }
}

// =============================================================================
// RawHash extractor (real implementation)
// =============================================================================

#[test]
fn raw_hash_verifies_correct_sha256() {
    let bytes = b"hello pr4xis";
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hex_digest = hex::encode(hasher.finalize());

    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: ClaimData::Sha256(hex_digest),
    };
    assert!(matches!(
        raw_hash::verify(&claim, bytes),
        VerificationResult::Verified(_)
    ));
}

#[test]
fn raw_hash_rejects_wrong_sha256() {
    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: ClaimData::Sha256("deadbeef".into()),
    };
    assert!(matches!(
        raw_hash::verify(&claim, b"hello pr4xis"),
        VerificationResult::Mismatch { .. }
    ));
}

#[test]
fn raw_hash_rejects_non_sha256_claim_data() {
    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: ClaimData::Stub {
            reason: "...".into(),
        },
    };
    assert!(matches!(
        raw_hash::verify(&claim, b"hello pr4xis"),
        VerificationResult::Unverifiable { .. }
    ));
}

// =============================================================================
// XmlElementAttribute extractor (real implementation)
// =============================================================================

const SAMPLE_WORDNET_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="oewn" label="English WordNet" language="en" email="test@example.com" license="CC BY 4.0" version="2025" url="https://en-word.net/">
    <LexicalEntry id="e-dog"><Lemma writtenForm="dog" partOfSpeech="n"/></LexicalEntry>
  </Lexicon>
</LexicalResource>"#;

#[test]
fn xml_attribute_verifies_wordnet_version() {
    let claim = IdentityClaim {
        concept: IdentityConcept::XmlElementAttribute,
        data: ClaimData::XmlAttribute {
            element: "Lexicon".into(),
            attribute: "version".into(),
            expected: "2025".into(),
        },
    };
    assert!(matches!(
        xml_element_attribute::verify(&claim, SAMPLE_WORDNET_XML.as_bytes()),
        VerificationResult::Verified(_)
    ));
}

#[test]
fn xml_attribute_rejects_wrong_version() {
    let claim = IdentityClaim {
        concept: IdentityConcept::XmlElementAttribute,
        data: ClaimData::XmlAttribute {
            element: "Lexicon".into(),
            attribute: "version".into(),
            expected: "2024".into(),
        },
    };
    assert!(matches!(
        xml_element_attribute::verify(&claim, SAMPLE_WORDNET_XML.as_bytes()),
        VerificationResult::Mismatch { .. }
    ));
}

#[test]
fn xml_attribute_unverifiable_when_element_missing() {
    let claim = IdentityClaim {
        concept: IdentityConcept::XmlElementAttribute,
        data: ClaimData::XmlAttribute {
            element: "Nonexistent".into(),
            attribute: "version".into(),
            expected: "2025".into(),
        },
    };
    assert!(matches!(
        xml_element_attribute::verify(&claim, SAMPLE_WORDNET_XML.as_bytes()),
        VerificationResult::Unverifiable { .. }
    ));
}

// =============================================================================
// Property-based tests
// =============================================================================

proptest! {
    /// Determinism: same bytes + same claim → same VerificationResult.
    #[test]
    fn prop_raw_hash_is_deterministic(bytes in prop::collection::vec(any::<u8>(), 0..2048)) {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hex_digest = hex::encode(hasher.finalize());
        let claim = IdentityClaim {
            concept: IdentityConcept::RawHash,
            data: ClaimData::Sha256(hex_digest),
        };
        let first = raw_hash::verify(&claim, &bytes);
        let second = raw_hash::verify(&claim, &bytes);
        prop_assert_eq!(&first, &second);
        prop_assert!(matches!(first, VerificationResult::Verified(_)));
    }

    /// Injectivity: any single-byte corruption causes Mismatch.
    #[test]
    fn prop_raw_hash_detects_any_corruption(
        bytes in prop::collection::vec(any::<u8>(), 1..512),
        corrupt_index in any::<usize>(),
    ) {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hex_digest = hex::encode(hasher.finalize());
        let claim = IdentityClaim {
            concept: IdentityConcept::RawHash,
            data: ClaimData::Sha256(hex_digest),
        };
        let mut corrupted = bytes.clone();
        let idx = corrupt_index % corrupted.len();
        corrupted[idx] = corrupted[idx].wrapping_add(1);
        let ok = matches!(
            raw_hash::verify(&claim, &corrupted),
            VerificationResult::Mismatch { .. }
        );
        prop_assert!(ok);
    }

    /// Stub extractors always return Unverifiable (fail-closed witness).
    #[test]
    fn prop_stub_claims_are_unverifiable(bytes in prop::collection::vec(any::<u8>(), 0..128)) {
        let claim = IdentityClaim {
            concept: IdentityConcept::OpenPgp,
            data: ClaimData::Stub { reason: "stub".into() },
        };
        let ok = matches!(
            super::schemes::openpgp::verify(&claim, &bytes),
            VerificationResult::Unverifiable { .. }
        );
        prop_assert!(ok);
    }
}
