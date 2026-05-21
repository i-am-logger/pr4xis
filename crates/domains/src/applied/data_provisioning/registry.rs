//! Runtime registry — `RegistryEntry` instances loaded from the
//! workspace-root `praxis.toml`. Lock hashes loaded from `praxis.lock`.
//!
//! Both files are embedded at compile time via `include_str!` and parsed
//! lazily on first access via `OnceLock`. The schema and parsing rules
//! are described inline below; the user-facing documentation lives in
//! the comments at the top of `praxis.toml` and `praxis.lock`.
//!
//! # Mapping
//!
//! `praxis.toml` declares manifest entries:
//!
//! ```toml
//! [sources.english_wordnet]
//! version = "2025"
//! type    = "Language"
//! url     = "https://..."
//! ```
//!
//! `praxis.lock` pins their hashes:
//!
//! ```toml
//! [hashes]
//! "english_wordnet@2025" = "6f49adeec..."
//! ```
//!
//! At load time the parser:
//! 1. Reads each `[sources.<name>]` table.
//! 2. Maps `type = "<concept>"` → typed [`SourceTaxonomyConcept`] via
//!    [`parse_concept`]. Unknown names panic at startup (fail-closed).
//! 3. Looks up `"<name>@<version>"` in `praxis.lock`'s `[hashes]` table
//!    to get the pinned sha256.
//! 4. Synthesizes the `CompositeIdentity`:
//!      - `XmlElementAttribute` claim for `<Lexicon version=...>` if the
//!        kind is in the Lexicon family (Global WordNet LMF convention).
//!      - `RawHash::Sha256` claim from the lock hash, applied to every
//!        kind (Dolstra 2006 content-addressing).
//! 5. Returns a `Vec<RegistryEntry>` cached for process lifetime.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::ontology::RegistryEntry;
use crate::formal::meta::artifact_identity::ontology::{
    ClaimData, CompositeIdentity, IdentityClaim, IdentityConcept,
};
use crate::formal::meta::source_taxonomy::ontology::{
    SourceTaxonomyConcept, is_lexicon, parse_concept,
};

// `SourceTaxonomyConcept` is referenced by the `build_entry_succeeds` parser
// test below; the doc-comment near `build_entry` also references the typed
// variant. Keeping the import unconditional reads cleaner than two
// `#[cfg(test)]` aliases.
#[allow(dead_code)]
const _SOURCE_TAXONOMY_CONCEPT_WITNESS: Option<SourceTaxonomyConcept> = None;

const PRAXIS_TOML: &str = include_str!("../../../../../praxis.toml");
const PRAXIS_LOCK: &str = include_str!("../../../../../praxis.lock");

static REGISTRY: OnceLock<Vec<RegistryEntry>> = OnceLock::new();
static LOCK: OnceLock<LockData> = OnceLock::new();

/// The loaded registry. First call parses both files and synthesizes the
/// identity claims; subsequent calls return the cached slice.
pub fn data_sources() -> &'static [RegistryEntry] {
    REGISTRY
        .get_or_init(|| {
            let manifest = parse_praxis_toml(PRAXIS_TOML)
                .unwrap_or_else(|e| panic!("invalid praxis.toml: {e}"));
            let lock = lock_hashes();
            manifest
                .into_iter()
                .map(|raw| build_entry(raw, lock))
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|e| panic!("praxis.toml/praxis.lock integration: {e}"))
        })
        .as_slice()
}

/// The pinned hashes from `praxis.lock`. Keys are `"<name>@<version>"`
/// strings; values are hex sha256.
pub fn lock_hashes() -> &'static HashMap<String, String> {
    &lock_data().hashes
}

fn lock_data() -> &'static LockData {
    LOCK.get_or_init(|| {
        parse_praxis_lock(PRAXIS_LOCK).unwrap_or_else(|e| panic!("invalid praxis.lock: {e}"))
    })
}

/// Look up a `RegistryEntry` by `name`. Returns the first match; if
/// multiple versions of the same name are registered, callers should use
/// [`by_name_version`] instead.
pub fn by_name(name: &str) -> Option<&'static RegistryEntry> {
    data_sources().iter().find(|e| e.name == name)
}

/// Look up a `RegistryEntry` by `(name, version)` — the primary key.
pub fn by_name_version(name: &str, version: &str) -> Option<&'static RegistryEntry> {
    data_sources()
        .iter()
        .find(|e| e.name == name && e.version == version)
}

/// Convenience for callers that want the entry's composite identity by name.
pub fn resolve_identity(name: &str) -> Option<CompositeIdentity> {
    by_name(name).map(|e| e.identity.clone())
}

// =============================================================================
// TOML parsing
// =============================================================================

#[derive(Debug, Deserialize)]
struct RawManifest {
    #[serde(default)]
    sources: HashMap<String, RawSource>,
}

#[derive(Debug, Deserialize)]
struct RawSource {
    version: String,
    #[serde(rename = "type")]
    kind: String,
    url: String,
    #[serde(default)]
    description: Option<String>,
}

/// Intermediate form: name + raw source. We keep names sorted at the
/// output boundary so the registry has a stable iteration order
/// (HashMap iteration is non-deterministic).
struct RawSourceWithName {
    name: String,
    raw: RawSource,
}

fn parse_praxis_toml(text: &str) -> Result<Vec<RawSourceWithName>, String> {
    let manifest: RawManifest = toml::from_str(text).map_err(|e| format!("toml parse: {e}"))?;
    let mut items: Vec<_> = manifest
        .sources
        .into_iter()
        .map(|(name, raw)| RawSourceWithName { name, raw })
        .collect();
    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

fn build_entry(
    item: RawSourceWithName,
    lock: &HashMap<String, String>,
) -> Result<RegistryEntry, String> {
    let RawSourceWithName { name, raw } = item;
    let kind = parse_concept(&raw.kind).ok_or_else(|| {
        format!(
            "source `{}`: unknown type `{}` — must be a leaf concept in SourceTaxonomy",
            name, raw.kind
        )
    })?;

    let key = format!("{}@{}", name, raw.version);
    let lock_sha = lock.get(&key).cloned();

    let mut claims = Vec::with_capacity(2);
    // Lexicon-family kinds (Language, DomainLexicon, LegalLexicon) ship
    // as XML-LMF and self-declare their version in the `<Lexicon
    // version="...">` attribute (Global WordNet LMF 1.3 convention).
    // We synthesize the corresponding identity claim so the existing
    // XML attribute extractor validates the upstream's self-description.
    if is_lexicon(kind) {
        claims.push(IdentityClaim {
            concept: IdentityConcept::XmlElementAttribute,
            data: ClaimData::XmlAttribute {
                element: "Lexicon".into(),
                attribute: "version".into(),
                expected: raw.version.clone(),
            },
        });
    }
    // Sources with a praxis.lock entry carry the cryptographic hash
    // claim (Dolstra 2006 content-addressing). Sources registered in
    // praxis.toml without a lock entry carry a Stub identity claim
    // that EveryDataSourceHasIdentity recognizes as a registered-but-
    // not-yet-loadable state. The LockManifestAgreement axiom skips
    // Stub-only entries (no claim to compare against the lock).
    match lock_sha {
        Some(hex) => claims.push(IdentityClaim {
            concept: IdentityConcept::RawHash,
            data: ClaimData::Sha256(hex),
        }),
        None => claims.push(IdentityClaim {
            concept: IdentityConcept::RawHash,
            data: ClaimData::Stub {
                reason: "registered in praxis.toml; awaiting praxis.lock hash".into(),
            },
        }),
    }

    Ok(RegistryEntry {
        name,
        version: raw.version,
        kind,
        url: raw.url,
        description: raw.description,
        identity: CompositeIdentity(claims),
    })
}

// =============================================================================
// praxis.lock parsing
// =============================================================================

/// The parsed `praxis.lock` payload. Hashes are the universal
/// drift-detection vehicle.
#[derive(Debug, Default)]
pub struct LockData {
    pub hashes: HashMap<String, String>,
}

/// In-memory representation of a statute's structural extraction —
/// terms + relations + description. Materialized by the USLM corpus
/// loader (see `social::software::markup::xml::uslm::corpus`) and
/// consumed by [`Statute::from_structural_with_context`][s] to build
/// the typed runtime `Statute` instance.
///
/// [s]: crate::social::compliance::statutes::Statute::from_structural_with_context
#[derive(Debug, Default)]
pub struct StructuralData {
    pub description: String,
    pub terms: Vec<StructuralTerm>,
    pub relations: Vec<StructuralRelation>,
}

/// One term in a statute's structural extraction.
#[derive(Debug)]
pub struct StructuralTerm {
    pub id: String,
    pub name: String,
    pub definition: String,
    pub lemmas: Vec<String>,
}

/// One relation between two terms. The `relation` field is the
/// PascalCase relation name (`Requires`, `Composes`, `SubtypeOf`, …).
#[derive(Debug)]
pub struct StructuralRelation {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Deserialize)]
struct RawLockFile {
    #[serde(default)]
    hashes: HashMap<String, String>,
}

fn parse_praxis_lock(text: &str) -> Result<LockData, String> {
    let raw: RawLockFile = toml::from_str(text).map_err(|e| format!("toml parse: {e}"))?;
    Ok(LockData { hashes: raw.hashes })
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn parses_empty_manifest() {
        let items = parse_praxis_toml("").unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn parses_simple_source() {
        let text = r#"
[sources.english_wordnet]
version = "2025"
type    = "Language"
url     = "https://example.com/wordnet.xml.gz"
"#;
        let items = parse_praxis_toml(text).unwrap();
        assert_eq!(items.len(), 1);
        let item = &items[0];
        assert_eq!(item.name, "english_wordnet");
        assert_eq!(item.raw.version, "2025");
        assert_eq!(item.raw.kind, "Language");
        assert_eq!(item.raw.url, "https://example.com/wordnet.xml.gz");
    }

    #[test]
    fn parses_lock_file() {
        let text = r#"
[hashes]
"english_wordnet@2025" = "deadbeef"
"usc_title_18@2024"    = "cafef00d"
"#;
        let lock = parse_praxis_lock(text).unwrap();
        assert_eq!(lock.hashes.len(), 2);
        assert_eq!(lock.hashes.get("english_wordnet@2025").unwrap(), "deadbeef");
        assert_eq!(lock.hashes.get("usc_title_18@2024").unwrap(), "cafef00d");
    }

    #[test]
    fn parses_empty_lock() {
        let lock = parse_praxis_lock("").unwrap();
        assert!(lock.hashes.is_empty());
    }

    #[test]
    fn build_entry_rejects_unknown_type() {
        let item = RawSourceWithName {
            name: "x".into(),
            raw: RawSource {
                version: "1".into(),
                kind: "NotAConcept".into(),
                url: "".into(),
                description: None,
            },
        };
        let lock = HashMap::new();
        let err = build_entry(item, &lock).unwrap_err();
        assert!(err.contains("unknown type"), "got: {err}");
    }

    #[test]
    fn build_entry_without_lock_yields_stub_identity() {
        // Sources registered in praxis.toml without a praxis.lock entry
        // get a Stub identity claim marking them as pending loadable
        // infrastructure. This is the "registered but not yet loadable"
        // state — the LockManifestAgreement axiom skips Stub entries;
        // EveryDataSourceHasIdentity treats Stub as a valid identity
        // claim.
        let item = RawSourceWithName {
            name: "x".into(),
            raw: RawSource {
                version: "1".into(),
                kind: "UsFederalStatute".into(),
                url: "".into(),
                description: None,
            },
        };
        let lock = HashMap::new();
        let entry = build_entry(item, &lock).expect("registration without lock should succeed");
        assert_eq!(entry.identity.0.len(), 1);
        assert!(matches!(entry.identity.0[0].data, ClaimData::Stub { .. }));
    }

    #[test]
    fn build_entry_succeeds_with_lock_hash() {
        let item = RawSourceWithName {
            name: "english_wordnet".into(),
            raw: RawSource {
                version: "2025".into(),
                kind: "Language".into(),
                url: "https://example.com/wn.xml.gz".into(),
                description: Some("test".into()),
            },
        };
        let mut lock = HashMap::new();
        lock.insert("english_wordnet@2025".into(), "deadbeef".into());
        let entry = build_entry(item, &lock).unwrap();
        assert_eq!(entry.name, "english_wordnet");
        assert_eq!(entry.version, "2025");
        assert_eq!(entry.kind, SourceTaxonomyConcept::Language);
        // Lexicon-family: gets BOTH an XML-attribute claim AND a hash claim.
        assert_eq!(entry.identity.0.len(), 2);
    }
}
