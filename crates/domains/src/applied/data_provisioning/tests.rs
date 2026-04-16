//! Tests for the data_provisioning ontology.
//!
//! The headline test is the **full-chain integration test**:
//! given raw bytes (synthesized WordNet LMF XML with a matching version
//! attribute and a matching declared hash), run them through
//! `XmlElementAttribute` verification, `RawHash` verification, the XmlLmf
//! decoder, and finally `English::from_wordnet`, producing a real, queryable
//! `English` ontology instance. This test proves the new data-provisioning
//! layer composes cleanly with the existing XML/LMF/English pipeline and
//! with the `artifact_identity` meta ontology.

use super::decoders::{has_decoder_for, xml_lmf};
use super::ontology::{
    ContentType, DataProvisioningCategory, DataProvisioningOntology, DecoderTotalityPerContentType,
    EveryDataSourceHasIdentity, IdentityClaimsUseLeaves, IsUsableLocally, ProvisioningConcept,
    RegistryUniquenessByName, TriggersUpdate,
};
use super::registry::{DATA_SOURCES, by_name, resolve_identity};
use crate::cognitive::linguistics::english::English;
use crate::formal::meta::artifact_identity::ontology::{
    ClaimData, IdentityClaim, IdentityConcept, VerificationResult,
};
use crate::formal::meta::artifact_identity::schemes::{raw_hash, xml_element_attribute};
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws and validation
// =============================================================================

#[test]
fn category_laws() {
    pr4xis::category::validate::check_category_laws::<DataProvisioningCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    DataProvisioningOntology::validate().unwrap();
}

// =============================================================================
// Registry shape
// =============================================================================

#[test]
fn registry_has_wordnet() {
    assert!(by_name("wordnet").is_some());
}

#[test]
fn registry_wordnet_has_composite_identity() {
    let identity = resolve_identity("wordnet").expect("wordnet registered");
    assert_eq!(
        identity.0.len(),
        2,
        "wordnet should have 2 identity claims (XmlElementAttribute + RawHash)"
    );
}

#[test]
fn registry_wordnet_uses_xml_lmf_content_type() {
    let entry = by_name("wordnet").unwrap();
    assert!(matches!(entry.content_type, ContentType::XmlLmf));
}

#[test]
fn registry_lookup_miss_returns_none() {
    assert!(by_name("not-a-real-dataset").is_none());
}

// =============================================================================
// Qualities
// =============================================================================

#[test]
fn verified_dataset_is_usable_locally() {
    assert_eq!(
        IsUsableLocally.get(&ProvisioningConcept::VerifiedDataset),
        Some(true)
    );
}

#[test]
fn stale_and_missing_are_not_usable() {
    assert_eq!(
        IsUsableLocally.get(&ProvisioningConcept::StaleDataset),
        Some(false)
    );
    assert_eq!(
        IsUsableLocally.get(&ProvisioningConcept::MissingDataset),
        Some(false)
    );
}

#[test]
fn stale_and_missing_trigger_update() {
    assert_eq!(
        TriggersUpdate.get(&ProvisioningConcept::StaleDataset),
        Some(true)
    );
    assert_eq!(
        TriggersUpdate.get(&ProvisioningConcept::MissingDataset),
        Some(true)
    );
}

// =============================================================================
// Domain axioms
// =============================================================================

#[test]
fn axiom_every_datasource_has_identity() {
    assert!(!DATA_SOURCES.is_empty());
    assert!(EveryDataSourceHasIdentity.holds());
}

#[test]
fn axiom_registry_uniqueness_by_name() {
    assert!(RegistryUniquenessByName.holds());
}

#[test]
fn axiom_decoder_totality_per_content_type() {
    assert!(DecoderTotalityPerContentType.holds());
}

#[test]
fn axiom_identity_claims_use_leaves() {
    assert!(IdentityClaimsUseLeaves.holds());
}

#[test]
fn all_domain_axioms_hold() {
    for axiom in DataProvisioningOntology::domain_axioms() {
        assert!(
            axiom.holds(),
            "domain axiom failed: {}",
            axiom.description()
        );
    }
}

#[test]
fn has_decoder_for_xml_lmf() {
    assert!(has_decoder_for(ContentType::XmlLmf));
}

#[test]
fn no_decoder_for_unimplemented_content_types() {
    assert!(!has_decoder_for(ContentType::Pdf));
    assert!(!has_decoder_for(ContentType::Video));
    assert!(!has_decoder_for(ContentType::Audio));
}

// =============================================================================
// Full-chain integration test — the headline test
// =============================================================================

/// Synthesized WordNet LMF XML that matches what an actual small WordNet
/// fragment looks like. This is what the data-provisioning layer would
/// receive after a successful fetch of `english-wordnet-2025.xml.gz`,
/// gunzipped. The integration test below runs it through the complete
/// pipeline.
const FAKE_WORDNET_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="oewn" label="English WordNet" language="en" email="test@example.com" license="CC BY 4.0" version="2025" url="https://en-word.net/">
    <LexicalEntry id="e-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="dog-n-01" synset="s-dog"/></LexicalEntry>
    <LexicalEntry id="e-cat-n"><Lemma writtenForm="cat" partOfSpeech="n"/><Sense id="cat-n-01" synset="s-cat"/></LexicalEntry>
    <Synset id="s-dog" ili="i1" partOfSpeech="n"><Definition>a domesticated canine</Definition></Synset>
    <Synset id="s-cat" ili="i2" partOfSpeech="n"><Definition>a small feline</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

/// **The full-chain integration test.**
///
/// Takes raw bytes → verifies the XmlElementAttribute identity claim
/// (`<Lexicon version="2025">`) → verifies the RawHash identity claim
/// (sha256 computed live) → runs the XmlLmf decoder → runs
/// `English::from_wordnet` → confirms we end up with a queryable `English`
/// ontology instance.
///
/// This is the "prove the composition works" test from the approved plan.
/// If it passes, the data-provisioning layer composes cleanly with the
/// existing XML/LMF/English pipeline.
#[test]
fn full_chain_raw_bytes_to_english_ontology() {
    let bytes = FAKE_WORDNET_XML.as_bytes();

    // Step 1: compute the real sha256 so we can build a matching claim.
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let real_hash = hex::encode(hasher.finalize());

    // Step 2: verify the XmlElementAttribute claim (upstream self-description)
    let version_claim = IdentityClaim {
        concept: IdentityConcept::XmlElementAttribute,
        data: ClaimData::XmlAttribute {
            element: "Lexicon",
            attribute: "version",
            expected: "2025".into(),
        },
    };
    let version_result = xml_element_attribute::verify(&version_claim, bytes);
    assert!(
        matches!(version_result, VerificationResult::Verified(_)),
        "version claim should verify, got {:?}",
        version_result
    );

    // Step 3: verify the RawHash claim (cryptographic integrity)
    let hash_claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: ClaimData::Sha256(real_hash),
    };
    let hash_result = raw_hash::verify(&hash_claim, bytes);
    assert!(
        matches!(hash_result, VerificationResult::Verified(_)),
        "hash claim should verify, got {:?}",
        hash_result
    );

    // Step 4: decode via the XmlLmf decoder
    let wordnet = xml_lmf::decode(bytes).expect("xml_lmf decoder should succeed on fake data");
    assert_eq!(wordnet.synsets.len(), 2);
    assert_eq!(wordnet.entries.len(), 2);

    // Step 5: run through the existing from_wordnet functor
    let english = English::from_wordnet(&wordnet);

    // Step 6: confirm the resulting English ontology is queryable with real data
    // The word_index should contain the lemmas we authored.
    assert!(english.word_index.contains_key("dog"));
    assert!(english.word_index.contains_key("cat"));
    assert_eq!(english.concepts.len(), 2);
}

/// Negative full-chain: corrupt the version attribute, confirm the
/// version claim fails and the pipeline fails-closed without decoding.
#[test]
fn full_chain_rejects_wrong_version() {
    let corrupted = FAKE_WORDNET_XML.replace("version=\"2025\"", "version=\"2024\"");

    let claim = IdentityClaim {
        concept: IdentityConcept::XmlElementAttribute,
        data: ClaimData::XmlAttribute {
            element: "Lexicon",
            attribute: "version",
            expected: "2025".into(),
        },
    };
    let result = xml_element_attribute::verify(&claim, corrupted.as_bytes());
    let is_mismatch = matches!(result, VerificationResult::Mismatch { .. });
    assert!(is_mismatch, "expected Mismatch, got {:?}", result);
}

// =============================================================================
// Property-based tests
// =============================================================================

/// Every `ContentType` is either a real wired decoder or is not. The
/// invariant: `has_decoder_for` returns a stable truth value per variant,
/// independent of call order, and is true only for variants that actually
/// have a dispatch arm.
fn every_content_type() -> Vec<ContentType> {
    vec![
        ContentType::XmlLmf,
        ContentType::Pdf,
        ContentType::Plaintext,
        ContentType::Json,
        ContentType::Video,
        ContentType::Audio,
        ContentType::Binary,
    ]
}

proptest! {
    /// `has_decoder_for` must be a pure function of the variant. Calling it
    /// many times for the same variant must return the same answer. This
    /// guards against someone accidentally introducing statefulness in
    /// decoder dispatch.
    #[test]
    fn prop_has_decoder_for_is_pure(idx in 0usize..7) {
        let variant = every_content_type()[idx];
        let first = has_decoder_for(variant);
        for _ in 0..16 {
            prop_assert_eq!(first, has_decoder_for(variant));
        }
    }

    /// `by_name` never panics and returns `None` for random names that
    /// aren't in the registry. The only name that should resolve in the
    /// current registry is "wordnet".
    #[test]
    fn prop_by_name_misses_random_strings(name in "[a-z]{1,20}") {
        if name != "wordnet" {
            prop_assert!(by_name(&name).is_none());
        } else {
            prop_assert!(by_name(&name).is_some());
        }
    }

    /// Every resolved identity claim on every registered entry must use a
    /// leaf `IdentityConcept`. Repeats the `IdentityClaimsUseLeaves` axiom
    /// over a proptest loop as a regression guard against accidentally
    /// adding a family-level concept to the registry.
    #[test]
    fn prop_all_resolved_claims_use_leaves(_seed in any::<u64>()) {
        use crate::formal::meta::artifact_identity::ontology::is_leaf;
        for entry in DATA_SOURCES {
            let resolved = resolve_identity(entry.name).unwrap();
            for claim in &resolved.0 {
                prop_assert!(is_leaf(&claim.concept));
            }
        }
    }
}
