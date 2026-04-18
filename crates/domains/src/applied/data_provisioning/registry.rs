//! The registry — concrete `RegistryEntry` instances, one per managed
//! dataset. This is the ontology's instance layer. Adding a new dataset =
//! adding a new entry here; no other code changes.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{ContentType, RegistryEntry};
use crate::formal::meta::artifact_identity::ontology::{
    ClaimData, CompositeIdentity, IdentityClaim, IdentityConcept,
};

/// SHA-256 of the decompressed English WordNet 2025 XML payload (the bytes
/// after `gunzip`). This hash pins the exact release from the upstream
/// Global WordNet Association / Open English WordNet repository at
/// <https://github.com/globalwordnet/english-wordnet/releases/tag/2025-edition>.
///
/// `fetch.rs` downloads the gzipped asset, decompresses, then verifies
/// against this hash. If the upstream ever re-publishes a different byte
/// sequence under the same tag, verification fails and pr4xis refuses the
/// update — `VerificationFailClosed`.
const WORDNET_2025_SHA256: &str =
    "6f49adeec174ab3092169fb25cf4a925226b63975a5d29a691a5dff88f0673b2";

/// Managed external data sources. Order is stable for reporting.
pub const DATA_SOURCES: &[RegistryEntry] = &[RegistryEntry {
    name: "wordnet",
    description: "Open English WordNet 2025 — Global WordNet Association WN-LMF 1.3",
    remote_location: "https://github.com/globalwordnet/english-wordnet/releases/download/2025-edition/english-wordnet-2025.xml.gz",
    local_path: "crates/domains/data/wordnet/english-wordnet-2025.xml",
    content_type: ContentType::XmlLmf,
    gzipped: true,
    identity: CompositeIdentity(Vec::new()),
}];

/// Look up a `RegistryEntry` by name. Linear scan because the registry is
/// small; switch to a map if it grows past ~100 entries.
pub fn by_name(name: &str) -> Option<&'static RegistryEntry> {
    DATA_SOURCES.iter().find(|e| e.name == name)
}

/// Build the composite identity for a registry entry at runtime. Since
/// `Vec<IdentityClaim>` is not a const type, the `DATA_SOURCES` entries above
/// declare `CompositeIdentity(Vec::new())` as a placeholder and this function
/// constructs the real identity at runtime. Every function `resolve_identity`
/// returns must match what the registry "should" be declaring statically.
///
/// This is the only place identity claims live in Rust source. Each entry's
/// identity is reconstructed here on every call; the registry entries above
/// hold all the other metadata.
pub fn resolve_identity(name: &str) -> Option<CompositeIdentity> {
    match name {
        "wordnet" => Some(CompositeIdentity(vec![
            // SelfDescribingMetadata: the upstream XML declares <Lexicon version="2025">
            IdentityClaim {
                concept: IdentityConcept::XmlElementAttribute,
                data: ClaimData::XmlAttribute {
                    element: "Lexicon",
                    attribute: "version",
                    expected: "2025".into(),
                },
            },
            // ContentHash: belt-and-braces cryptographic integrity
            IdentityClaim {
                concept: IdentityConcept::RawHash,
                data: ClaimData::Sha256(WORDNET_2025_SHA256.into()),
            },
        ])),
        _ => None,
    }
}

/// Every registry entry's resolved identity (not the const placeholder).
/// Used by the ontology axioms to verify identity claims at test time.
pub fn resolved_identities() -> Vec<(&'static str, CompositeIdentity)> {
    DATA_SOURCES
        .iter()
        .filter_map(|e| resolve_identity(e.name).map(|id| (e.name, id)))
        .collect()
}
