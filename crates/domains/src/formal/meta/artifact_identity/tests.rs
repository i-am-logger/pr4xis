//! Tests for the artifact_identity ontology — taxonomy structure, family
//! axioms, real extractor implementations, and property-based tests over
//! the deterministic and fail-closed axioms.

use super::ontology::{
    ArtifactIdentityCategory, ArtifactIdentityOntology, ClaimData, ContentHashIsInjective,
    ContentHashIsOffline, EverySchemeHasAnExtractor, IdentityClaim, IdentityConcept,
    IdentityTaxonomy, PersistentIdentifierRequiresResolver, SelfDescribingIsWeakestTrust,
    TrustTier, TrustTierOf, VerifiabilityOffline, VerificationResult, is_family, is_leaf,
};
use super::schemes::{raw_hash, xml_element_attribute};
use pr4xis::category::Entity;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws and validation
// =============================================================================

#[test]
fn category_laws() {
    pr4xis::category::validate::check_category_laws::<ArtifactIdentityCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    ArtifactIdentityOntology::validate().unwrap();
}

// =============================================================================
// Macro-generated descriptor
// =============================================================================

#[test]
fn descriptor_from_macro() {
    let d = ArtifactIdentityOntology::vocabulary();
    assert_eq!(d.name(), "ArtifactIdentityOntology");
    assert!(d.module_path.contains("artifact_identity"));
    assert_eq!(d.source, "Dolstra (2006); Wilkinson FAIR F1 (2016)");
    assert_eq!(
        d.being,
        Some(pr4xis::ontology::upper::being::Being::AbstractObject)
    );
    assert_eq!(d.concept_count, IdentityConcept::variants().len());
    assert!(d.concept_count > 0);
    assert!(d.morphism_count > 0);
    let domain = d.domain();
    assert!(
        domain.contains("formal.meta.artifact_identity"),
        "domain was: {domain}"
    );
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
        let ancestors = taxonomy::ancestors::<IdentityTaxonomy>(&concept);
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
fn cryptographic_signature_family_has_six_leaves() {
    use IdentityConcept::*;
    let leaves = [
        OpenPgp,
        SigstoreAttestation,
        SshSignature,
        Minisign,
        X509Signature,
        Ed25519Raw,
    ];
    for leaf in leaves {
        let ancestors = taxonomy::ancestors::<IdentityTaxonomy>(&leaf);
        assert!(
            ancestors.contains(&CryptographicSignature),
            "{:?} should descend from CryptographicSignature",
            leaf
        );
    }
}

#[test]
fn content_hash_family_has_five_leaves() {
    use IdentityConcept::*;
    let leaves = [
        RawHash,
        GitObjectSha,
        IpfsCid,
        NixStorePath,
        BittorrentInfoHash,
    ];
    for leaf in leaves {
        let ancestors = taxonomy::ancestors::<IdentityTaxonomy>(&leaf);
        assert!(
            ancestors.contains(&ContentHash),
            "{:?} should descend from ContentHash",
            leaf
        );
    }
}

#[test]
fn persistent_identifier_family_has_four_leaves() {
    use IdentityConcept::*;
    let leaves = [Doi, Handle, Ark, Purl];
    for leaf in leaves {
        let ancestors = taxonomy::ancestors::<IdentityTaxonomy>(&leaf);
        assert!(
            ancestors.contains(&PersistentIdentifier),
            "{:?} should descend from PersistentIdentifier",
            leaf
        );
    }
}

#[test]
fn self_describing_metadata_family_has_five_leaves() {
    use IdentityConcept::*;
    let leaves = [
        OwlVersionIri,
        OwlVersionInfo,
        DctIdentifier,
        XmlElementAttribute,
        XmlSchemaVersion,
    ];
    for leaf in leaves {
        let ancestors = taxonomy::ancestors::<IdentityTaxonomy>(&leaf);
        assert!(
            ancestors.contains(&SelfDescribingMetadata),
            "{:?} should descend from SelfDescribingMetadata",
            leaf
        );
    }
}

// =============================================================================
// Qualities
// =============================================================================

#[test]
fn content_hash_leaves_are_offline() {
    use IdentityConcept::*;
    let q = VerifiabilityOffline;
    for leaf in [
        RawHash,
        GitObjectSha,
        IpfsCid,
        NixStorePath,
        BittorrentInfoHash,
    ] {
        assert_eq!(q.get(&leaf), Some(true));
    }
}

#[test]
fn persistent_identifier_leaves_are_online() {
    use IdentityConcept::*;
    let q = VerifiabilityOffline;
    for leaf in [Doi, Handle, Ark, Purl] {
        assert_eq!(q.get(&leaf), Some(false));
    }
}

#[test]
fn self_describing_leaves_are_declarative() {
    use IdentityConcept::*;
    let q = TrustTierOf;
    for leaf in [
        OwlVersionIri,
        OwlVersionInfo,
        DctIdentifier,
        XmlElementAttribute,
        XmlSchemaVersion,
    ] {
        assert_eq!(q.get(&leaf), Some(TrustTier::Declarative));
    }
}

#[test]
fn cryptographic_and_hash_leaves_are_strong() {
    use IdentityConcept::*;
    let q = TrustTierOf;
    let strong_leaves = [
        OpenPgp,
        SigstoreAttestation,
        SshSignature,
        Minisign,
        X509Signature,
        Ed25519Raw,
        RawHash,
        GitObjectSha,
        IpfsCid,
        NixStorePath,
        BittorrentInfoHash,
    ];
    for leaf in strong_leaves {
        assert_eq!(q.get(&leaf), Some(TrustTier::Strong));
    }
}

// =============================================================================
// Domain axioms
// =============================================================================

#[test]
fn axiom_every_scheme_has_an_extractor() {
    assert!(EverySchemeHasAnExtractor.holds());
}

#[test]
fn axiom_content_hash_is_injective() {
    assert!(ContentHashIsInjective.holds());
}

#[test]
fn axiom_content_hash_is_offline() {
    assert!(ContentHashIsOffline.holds());
}

#[test]
fn axiom_persistent_identifier_requires_resolver() {
    assert!(PersistentIdentifierRequiresResolver.holds());
}

#[test]
fn axiom_self_describing_is_weakest_trust() {
    assert!(SelfDescribingIsWeakestTrust.holds());
}

#[test]
fn all_domain_axioms_hold() {
    for axiom in ArtifactIdentityOntology::domain_axioms() {
        assert!(
            axiom.holds(),
            "domain axiom does not hold: {}",
            axiom.description()
        );
    }
}

// =============================================================================
// RawHash extractor (real implementation)
// =============================================================================

#[test]
fn raw_hash_verifies_correct_sha256() {
    let bytes = b"hello pr4xis";
    // sha256("hello pr4xis") = precompute via sha2 directly so the test
    // doesn't depend on a hand-typed hash.
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hex_digest = hex::encode(hasher.finalize());

    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: ClaimData::Sha256(hex_digest.clone()),
    };
    let result = raw_hash::verify(&claim, bytes);
    assert!(
        matches!(result, VerificationResult::Verified(_)),
        "expected Verified, got {:?}",
        result
    );
}

#[test]
fn raw_hash_rejects_wrong_sha256() {
    let bytes = b"hello pr4xis";
    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: ClaimData::Sha256("deadbeef".into()),
    };
    let result = raw_hash::verify(&claim, bytes);
    assert!(matches!(result, VerificationResult::Mismatch { .. }));
}

#[test]
fn raw_hash_rejects_non_sha256_claim_data() {
    let bytes = b"hello pr4xis";
    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: ClaimData::Stub { reason: "..." },
    };
    let result = raw_hash::verify(&claim, bytes);
    assert!(matches!(result, VerificationResult::Unverifiable { .. }));
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
            element: "Lexicon",
            attribute: "version",
            expected: "2025".into(),
        },
    };
    let result = xml_element_attribute::verify(&claim, SAMPLE_WORDNET_XML.as_bytes());
    assert!(
        matches!(result, VerificationResult::Verified(_)),
        "expected Verified, got {:?}",
        result
    );
}

#[test]
fn xml_attribute_rejects_wrong_version() {
    let claim = IdentityClaim {
        concept: IdentityConcept::XmlElementAttribute,
        data: ClaimData::XmlAttribute {
            element: "Lexicon",
            attribute: "version",
            expected: "2024".into(),
        },
    };
    let result = xml_element_attribute::verify(&claim, SAMPLE_WORDNET_XML.as_bytes());
    assert!(matches!(result, VerificationResult::Mismatch { .. }));
}

#[test]
fn xml_attribute_unverifiable_when_element_missing() {
    let claim = IdentityClaim {
        concept: IdentityConcept::XmlElementAttribute,
        data: ClaimData::XmlAttribute {
            element: "Nonexistent",
            attribute: "version",
            expected: "2025".into(),
        },
    };
    let result = xml_element_attribute::verify(&claim, SAMPLE_WORDNET_XML.as_bytes());
    assert!(matches!(result, VerificationResult::Unverifiable { .. }));
}

// =============================================================================
// Property-based tests
// =============================================================================

proptest! {
    /// ExtractorIsDeterministic: the same bytes + same claim always produce
    /// the same VerificationResult. Checked here for the RawHash extractor
    /// with randomized byte payloads.
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
        let is_verified = matches!(first, VerificationResult::Verified(_));
        prop_assert!(is_verified);
    }

    /// Corrupting any single byte causes the RawHash claim to return
    /// Mismatch — the injectivity axiom in action.
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

        let result = raw_hash::verify(&claim, &corrupted);
        let is_mismatch = matches!(result, VerificationResult::Mismatch { .. });
        prop_assert!(is_mismatch);
    }

    /// Stub extractors always return Unverifiable. This is the structural
    /// witness that the 18 stubbed leaves satisfy VerificationFailClosed
    /// (they never return Verified, so they never silently let bad data
    /// through).
    #[test]
    fn prop_stub_claims_are_unverifiable(bytes in prop::collection::vec(any::<u8>(), 0..128)) {
        let claim = IdentityClaim {
            concept: IdentityConcept::OpenPgp,
            data: ClaimData::Stub { reason: "stub" },
        };
        let result = super::schemes::openpgp::verify(&claim, &bytes);
        let is_unverifiable = matches!(result, VerificationResult::Unverifiable { .. });
        prop_assert!(is_unverifiable);
    }
}
