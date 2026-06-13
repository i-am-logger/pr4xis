//! Tests for the data_provisioning ontology + new SourceTaxonomy-driven
//! registry.
//!
//! The headline test is the **full-chain integration test**:
//! given raw bytes (synthesized WordNet LMF XML with a matching version
//! attribute and a matching declared hash), run them through
//! `XmlElementAttribute` verification, `RawHash` verification, the XmlLmf
//! decoder, and finally `English::from_wordnet`, producing a real,
//! queryable `English` ontology instance. This proves the data-provisioning
//! layer composes cleanly with the existing XML/LMF/English pipeline and
//! with the `artifact_identity` + `source_taxonomy` meta ontologies.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::decoders::{has_decoder_for, xml_lmf};
use super::ontology::{
    ContentType, DataProvisioningCategory, DataProvisioningConcept, DataProvisioningOntology,
    DecoderTotalityPerKind, EveryDataSourceHasIdentity, IdentityClaimsUseLeaves, IsUsableLocally,
    KindIsTaxonomyLeaf, LockManifestAgreement, RegistryUniquenessByNameVersion, TriggersUpdate,
    canonical_encoding,
};
use super::registry::{by_name, by_name_version, data_sources, lock_hashes, resolve_identity};
use crate::cognitive::linguistics::english::English;
use crate::formal::meta::artifact_identity::ontology::{
    ClaimData, IdentityClaim, IdentityConcept, VerificationResult,
};
use crate::formal::meta::artifact_identity::schemes::{raw_hash, xml_element_attribute};
use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category laws and validation
// =============================================================================

#[test]
fn category_laws() {
    pr4xis::category::laws::assert_category_laws::<DataProvisioningCategory>();
}

#[test]
fn ontology_validates() {
    DataProvisioningOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Registry shape
// =============================================================================

#[test]
fn registry_has_english_wordnet() {
    assert!(by_name("english_wordnet").is_some());
}

#[test]
fn registry_english_wordnet_kind_is_language() {
    let entry = by_name("english_wordnet").unwrap();
    assert_eq!(entry.kind, SourceTaxonomyConcept::Language);
}

#[test]
fn registry_english_wordnet_has_version_2025() {
    let entry = by_name("english_wordnet").unwrap();
    assert_eq!(entry.version, "2025");
}

#[test]
fn registry_english_wordnet_has_composite_identity() {
    let identity = resolve_identity("english_wordnet").expect("english_wordnet registered");
    assert_eq!(
        identity.0.len(),
        2,
        "Lexicon-family source should have 2 identity claims (XmlElementAttribute + RawHash)"
    );
}

#[test]
fn registry_canonical_encoding_for_wordnet_is_xml_lmf() {
    let entry = by_name("english_wordnet").unwrap();
    assert_eq!(canonical_encoding(entry.kind), ContentType::XmlLmf);
}

#[test]
fn registry_lookup_miss_returns_none() {
    assert!(by_name("not-a-real-dataset").is_none());
}

#[test]
fn by_name_version_matches_pair() {
    assert!(by_name_version("english_wordnet", "2025").is_some());
    assert!(by_name_version("english_wordnet", "9999").is_none());
}

#[test]
fn english_wordnet_local_path_matches_disk() {
    // WordNet predates the Lexicon-family taxonomy and lives at the
    // historical `data/wordnet/english-wordnet-2025.xml` path that the
    // LMF reader's `include_str!` site references. `local_path_override`
    // returns this canonical disk location so `pr4xis update --check`
    // and the `RegistryLocalPathsExist` axiom both see the real bytes.
    let entry = by_name("english_wordnet").unwrap();
    let path = entry.local_path();
    assert_eq!(
        path, "crates/domains/data/wordnet/english-wordnet-2025.xml",
        "WordNet local_path must point at the actual on-disk bytes"
    );
}

#[test]
fn english_wordnet_url_is_gzipped() {
    let entry = by_name("english_wordnet").unwrap();
    assert!(entry.gzipped());
}

#[test]
fn english_wordnet_transport_gzip_yes() {
    // URL ends `.xml.gz`, local path ends `.xml` — fetcher must
    // decompress to reach the on-disk canonical form.
    let entry = by_name("english_wordnet").unwrap();
    assert!(entry.gzipped());
    assert!(
        !entry.local_path().ends_with(".gz"),
        "WordNet local path drops .gz extension"
    );
    assert!(
        entry.transport_gzip(),
        "WordNet must be transport_gzip — fetcher decompresses"
    );
}

#[test]
fn xmlconf_transport_gzip_no() {
    // URL ends `.tar.gz`, local path also ends `.tar.gz` — the gzip
    // wrapper is part of the on-disk form, fetcher must NOT decompress
    // (the lock pins the raw response bytes). Regression-pins the
    // bug that produced the CI "RawHash claim mismatch" by
    // gunzipping every `.gz` URL.
    let entry = by_name("xmlconf_xml_test_suite").unwrap();
    assert!(entry.gzipped(), "URL ends with .gz");
    assert!(
        entry.local_path().ends_with(".gz"),
        "xmlconf local path preserves the .tar.gz wrapper"
    );
    assert!(
        !entry.transport_gzip(),
        "xmlconf must NOT be transport_gzip — fetcher writes raw"
    );
}

#[test]
fn xsts_transport_gzip_no() {
    // Same situation as xmlconf — `.tar.gz` URL, `.tar.gz` local path,
    // raw bytes pinned in the lock.
    let entry = by_name("xsts_xml_schema_test_suite").unwrap();
    assert!(entry.gzipped(), "URL ends with .gz");
    assert!(
        entry.local_path().ends_with(".gz"),
        "xsts local path preserves the .tar.gz wrapper"
    );
    assert!(
        !entry.transport_gzip(),
        "xsts must NOT be transport_gzip — fetcher writes raw"
    );
}

#[test]
fn lock_hashes_contains_wordnet() {
    let key = "english_wordnet@2025";
    assert!(lock_hashes().contains_key(key), "missing {key}");
}

// =============================================================================
// Qualities
// =============================================================================

#[test]
fn verified_dataset_is_usable_locally() {
    assert_eq!(
        IsUsableLocally.get(&DataProvisioningConcept::VerifiedDataset),
        Some(true)
    );
}

#[test]
fn stale_and_missing_are_not_usable() {
    assert_eq!(
        IsUsableLocally.get(&DataProvisioningConcept::StaleDataset),
        Some(false)
    );
    assert_eq!(
        IsUsableLocally.get(&DataProvisioningConcept::MissingDataset),
        Some(false)
    );
}

#[test]
fn stale_and_missing_trigger_update() {
    assert_eq!(
        TriggersUpdate.get(&DataProvisioningConcept::StaleDataset),
        Some(true)
    );
    assert_eq!(
        TriggersUpdate.get(&DataProvisioningConcept::MissingDataset),
        Some(true)
    );
}

// =============================================================================
// Domain axioms
// =============================================================================

#[test]
fn axiom_every_datasource_has_identity() {
    assert!(!data_sources().is_empty());
    assert!(EveryDataSourceHasIdentity.verify().is_ok());
}

#[test]
fn axiom_registry_uniqueness_by_name_version() {
    assert!(RegistryUniquenessByNameVersion.verify().is_ok());
}

#[test]
fn axiom_decoder_totality_per_kind() {
    assert!(DecoderTotalityPerKind.verify().is_ok());
}

#[test]
fn axiom_identity_claims_use_leaves() {
    assert!(IdentityClaimsUseLeaves.verify().is_ok());
}

#[test]
fn axiom_kind_is_taxonomy_leaf() {
    assert!(KindIsTaxonomyLeaf.verify().is_ok());
}

#[test]
fn axiom_lock_manifest_agreement() {
    assert!(LockManifestAgreement.verify().is_ok());
}

#[test]
fn all_axioms_hold() {
    for axiom in DataProvisioningOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

#[test]
fn has_decoder_for_xml_lmf() {
    assert!(has_decoder_for(ContentType::XmlLmf));
}

#[test]
fn has_decoder_for_xml_xsd() {
    // The `XmlXsd` decoder satisfies `DecoderTotalityPerKind` for the
    // `uslm_xsd@1.0.18` registry entry — without it, the axiom would
    // fail at startup because the XSD has a lock hash (so it's
    // not Stub-only) and is treated as a runtime-loadable source.
    assert!(has_decoder_for(ContentType::XmlXsd));
}

#[test]
fn no_decoder_for_unimplemented_content_types() {
    assert!(!has_decoder_for(ContentType::Pdf));
    assert!(!has_decoder_for(ContentType::Video));
    assert!(!has_decoder_for(ContentType::Audio));
}

// =============================================================================
// Full-chain integration test
// =============================================================================

/// Synthesized WordNet LMF XML that matches what an actual small WordNet
/// fragment looks like. After a successful fetch of
/// `english-wordnet-2025.xml.gz` + gunzip, the bytes look like this.
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
/// Bytes → XmlElementAttribute → RawHash → XmlLmf decoder →
/// `English::from_wordnet` → queryable `English`. If it passes, the
/// data-provisioning layer composes cleanly with the existing
/// XML/LMF/English pipeline and with the SourceTaxonomy.
#[test]
fn full_chain_raw_bytes_to_english_ontology() {
    let bytes = FAKE_WORDNET_XML.as_bytes();

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let real_hash = hex::encode(hasher.finalize());

    let version_claim = IdentityClaim {
        concept: IdentityConcept::XmlElementAttribute,
        data: ClaimData::XmlAttribute {
            element: "Lexicon".into(),
            attribute: "version".into(),
            expected: "2025".into(),
        },
    };
    let version_result = xml_element_attribute::verify(&version_claim, bytes);
    assert!(
        matches!(version_result, VerificationResult::Verified(_)),
        "version claim should verify, got {:?}",
        version_result
    );

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

    let wordnet = xml_lmf::decode(bytes).expect("xml_lmf decoder should succeed on fake data");
    assert_eq!(wordnet.synsets.len(), 2);
    assert_eq!(wordnet.entries.len(), 2);

    let english = English::from_wordnet(&wordnet);
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
            element: "Lexicon".into(),
            attribute: "version".into(),
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

fn every_content_type() -> Vec<ContentType> {
    vec![
        ContentType::XmlLmf,
        ContentType::Pdf,
        ContentType::UslmXml,
        ContentType::Plaintext,
        ContentType::AdobeGlyphList,
        ContentType::Json,
        ContentType::Video,
        ContentType::Audio,
        ContentType::Binary,
        ContentType::XmlXsd,
        ContentType::Xhtml,
        ContentType::Owl,
    ]
}

proptest! {
    /// `has_decoder_for` must be a pure function of the variant.
    #[test]
    fn prop_has_decoder_for_is_pure(idx in 0usize..12) {
        let variant = every_content_type()[idx];
        let first = has_decoder_for(variant);
        for _ in 0..16 {
            prop_assert_eq!(first, has_decoder_for(variant));
        }
    }

    /// `by_name` returns the registered entry for the known names
    /// and `None` for everything else random.
    #[test]
    fn prop_by_name_misses_random_strings(name in "[a-z_]{1,20}") {
        // Names that are known to be registered in praxis.toml. Keep this
        // set in sync with the [sources.*] entries in praxis.toml.
        let registered = [
            "english_wordnet",
            "us_legal_lexicon",
            "english_function_words",
            "xml_1_0_namespace_xsd",
            "xml_infoset",
        ];
        if !registered.contains(&name.as_str()) {
            prop_assert!(by_name(&name).is_none());
        } else {
            prop_assert!(by_name(&name).is_some());
        }
    }

    /// Every resolved identity claim on every registered entry uses a
    /// leaf IdentityConcept.
    #[test]
    fn prop_all_resolved_claims_use_leaves(_seed in any::<u64>()) {
        use crate::formal::meta::artifact_identity::ontology::is_leaf;
        for entry in data_sources() {
            for claim in &entry.identity.0 {
                prop_assert!(is_leaf(&claim.concept));
            }
        }
    }

    /// Every registered entry's kind is a leaf in the SourceTaxonomy.
    #[test]
    fn prop_every_entry_kind_is_leaf(_seed in any::<u64>()) {
        use crate::formal::meta::source_taxonomy::ontology::is_leaf as is_kind_leaf;
        for entry in data_sources() {
            prop_assert!(is_kind_leaf(entry.kind),
                "entry {} has non-leaf kind {:?}", entry.name, entry.kind);
        }
    }
}
