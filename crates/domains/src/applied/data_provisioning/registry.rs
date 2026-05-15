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

/// Per-source structural extraction blocks from `praxis.lock`. Populated
/// for sources whose `Store` backend bakes their derived ontology at
/// build time (typically Statutes and other LegalCorpus kinds). Keys
/// match the hash map's `"<name>@<version>"` form.
pub fn lock_structural() -> &'static HashMap<String, StructuralData> {
    &lock_data().structural
}

/// Look up the structural extraction for a specific source. Returns
/// `None` if the lock has no structural block for that `(name,
/// version)` — common for `Heap`-store sources like WordNet whose data
/// is realized at runtime, not lock time.
pub fn structural_for(name: &str, version: &str) -> Option<&'static StructuralData> {
    let key = format!("{name}@{version}");
    lock_structural().get(&key)
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
    let lock_sha = lock
        .get(&key)
        .ok_or_else(|| {
            format!("praxis.lock missing hash for `{key}` — run `pr4xis lock` to regenerate")
        })?
        .clone();

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
    // Every entry carries the cryptographic hash claim from praxis.lock
    // (Dolstra 2006 content-addressing).
    claims.push(IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: ClaimData::Sha256(lock_sha),
    });

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
/// drift-detection vehicle; structural data is the lock-baked ontology
/// content for sources whose `Store` backend is `StaticStore` (statutes
/// today; potentially future cached lexicons).
#[derive(Debug, Default)]
pub struct LockData {
    pub hashes: HashMap<String, String>,
    pub structural: HashMap<String, StructuralData>,
}

/// One source's structural extraction. The schema matches the praxis
/// statute codegen input (`pr4xis::codegen::statute::RawStatuteDoc`);
/// conversion is one-to-one via [`Self::to_raw_statute_doc`].
#[derive(Debug, Default, Deserialize)]
pub struct StructuralData {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub terms: Vec<StructuralTerm>,
    #[serde(default)]
    pub relations: Vec<StructuralRelation>,
}

impl StructuralData {
    /// Convert into the praxis statute codegen's input format. Names
    /// the doc with `name` (caller supplies it because the structural
    /// block in the lock is keyed by `name@version`, not embedded in
    /// the block itself). Relations whose `relation` field doesn't
    /// match a known [`RawRel`] variant are skipped — the upstream
    /// extraction tool is expected to emit only PascalCase relation
    /// names matching the 13-kind enum.
    pub fn to_raw_statute_doc(&self, name: &str) -> pr4xis::codegen::statute::RawStatuteDoc {
        use pr4xis::codegen::statute::{RawRelation, RawStatuteDoc, RawTerm};
        let terms = self
            .terms
            .iter()
            .map(|t| RawTerm {
                id: t.id.clone(),
                name: t.name.clone(),
                definition: t.definition.clone(),
                lemmas: t.lemmas.clone(),
            })
            .collect();
        let relations = self
            .relations
            .iter()
            .filter_map(|r| {
                parse_simple_relation(&r.relation).map(|rel| RawRelation {
                    from: r.from.clone(),
                    to: r.to.clone(),
                    relation: rel,
                })
            })
            .collect();
        RawStatuteDoc {
            name: name.to_string(),
            description: self.description.clone(),
            terms,
            relations,
        }
    }
}

/// Map a PascalCase relation string to a [`RawRel`] variant. Only the
/// parameter-free spellings are supported here; relations whose
/// upstream extraction carries parametric qualifiers (Composes-into,
/// Precedes-max_days, etc.) collapse to their parameter-free form,
/// matching the codegen's lossy mapping.
fn parse_simple_relation(s: &str) -> Option<pr4xis::codegen::statute::RawRel> {
    use pr4xis::codegen::statute::RawRel;
    Some(match s {
        "Requires" => RawRel::Requires,
        "SubtypeOf" => RawRel::SubtypeOf,
        "Contradicts" => RawRel::Contradicts,
        "Negates" => RawRel::Negates,
        "AlternativeTo" => RawRel::AlternativeTo,
        "AffirmativeDefenseTo" => RawRel::AffirmativeDefenseTo,
        "SafeHarborFor" => RawRel::SafeHarborFor,
        "ExhaustionRequiredFor" => RawRel::ExhaustionRequiredFor,
        "Precedes" => RawRel::Precedes { max_days: None },
        "Implies" => RawRel::Implies { consequence: None },
        "Composes" => RawRel::Composes { into: None },
        "Triggers" => RawRel::Triggers { obligation: None },
        "Rebuts" => RawRel::Rebuts { burden: None },
        _ => return None,
    })
}

/// One term in a statute's structural extraction.
#[derive(Debug, Deserialize)]
pub struct StructuralTerm {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub definition: String,
    #[serde(default)]
    pub lemmas: Vec<String>,
}

/// One relation between two terms. The `relation` field is the
/// PascalCase relation name (`Requires`, `Composes`, `SubtypeOf`, …);
/// parametric qualifiers from the source extraction are not carried in
/// the lock per the codegen mapping's documented losses.
#[derive(Debug, Deserialize)]
pub struct StructuralRelation {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Deserialize)]
struct RawLockFile {
    #[serde(default)]
    hashes: HashMap<String, String>,
    #[serde(default)]
    structural: HashMap<String, StructuralData>,
}

fn parse_praxis_lock(text: &str) -> Result<LockData, String> {
    let raw: RawLockFile = toml::from_str(text).map_err(|e| format!("toml parse: {e}"))?;
    Ok(LockData {
        hashes: raw.hashes,
        structural: raw.structural,
    })
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
    fn parses_multiple_sources_sorted() {
        let text = r#"
[sources.zebra_corpus]
version = "1"
type    = "Statute"
url     = "https://example.com/zebra.txt"

[sources.alpha_corpus]
version = "1"
type    = "Statute"
url     = "https://example.com/alpha.txt"
"#;
        let items = parse_praxis_toml(text).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].name, "alpha_corpus",
            "names should sort lexicographically"
        );
        assert_eq!(items[1].name, "zebra_corpus");
    }

    #[test]
    fn parses_lock_file() {
        let text = r#"
[hashes]
"english_wordnet@2025" = "deadbeef"
"sox_1514a@2002"       = "cafef00d"
"#;
        let lock = parse_praxis_lock(text).unwrap();
        assert_eq!(lock.hashes.len(), 2);
        assert_eq!(lock.hashes.get("english_wordnet@2025").unwrap(), "deadbeef");
        assert_eq!(lock.hashes.get("sox_1514a@2002").unwrap(), "cafef00d");
        assert!(lock.structural.is_empty());
    }

    #[test]
    fn parses_empty_lock() {
        let lock = parse_praxis_lock("").unwrap();
        assert!(lock.hashes.is_empty());
        assert!(lock.structural.is_empty());
    }

    #[test]
    fn parses_lock_with_structural_block() {
        let text = r#"
[hashes]
"sox_1514a@2002" = "deadbeef"

[structural."sox_1514a@2002"]
description = "test statute"

[[structural."sox_1514a@2002".terms]]
id = "sox:a"
name = "Term A"
definition = "First."
lemmas = ["report"]

[[structural."sox_1514a@2002".terms]]
id = "sox:b"
name = "Term B"
definition = "Second."

[[structural."sox_1514a@2002".relations]]
from = "sox:a"
to = "sox:b"
relation = "Requires"
"#;
        let lock = parse_praxis_lock(text).unwrap();
        let s = lock
            .structural
            .get("sox_1514a@2002")
            .expect("structural block present");
        assert_eq!(s.description, "test statute");
        assert_eq!(s.terms.len(), 2);
        assert_eq!(s.terms[0].id, "sox:a");
        assert_eq!(s.terms[0].lemmas, vec!["report".to_string()]);
        assert_eq!(s.relations.len(), 1);
        assert_eq!(s.relations[0].relation, "Requires");
    }

    #[test]
    fn structural_data_converts_to_raw_statute_doc() {
        let s = StructuralData {
            description: "demo".into(),
            terms: vec![
                StructuralTerm {
                    id: "t:a".into(),
                    name: "A".into(),
                    definition: "first".into(),
                    lemmas: vec!["report".into()],
                },
                StructuralTerm {
                    id: "t:b".into(),
                    name: "B".into(),
                    definition: "second".into(),
                    lemmas: vec![],
                },
            ],
            relations: vec![
                StructuralRelation {
                    from: "t:a".into(),
                    to: "t:b".into(),
                    relation: "Requires".into(),
                },
                StructuralRelation {
                    from: "t:a".into(),
                    to: "t:b".into(),
                    relation: "UnknownKind".into(), // should be skipped
                },
            ],
        };
        let doc = s.to_raw_statute_doc("test");
        assert_eq!(doc.name, "test");
        assert_eq!(doc.description, "demo");
        assert_eq!(doc.terms.len(), 2);
        assert_eq!(doc.terms[0].id, "t:a");
        assert_eq!(doc.terms[0].lemmas, vec!["report".to_string()]);
        assert_eq!(
            doc.relations.len(),
            1,
            "unknown relation should have been dropped"
        );
    }

    #[test]
    fn parse_simple_relation_covers_thirteen_variants() {
        assert!(parse_simple_relation("Requires").is_some());
        assert!(parse_simple_relation("SubtypeOf").is_some());
        assert!(parse_simple_relation("Contradicts").is_some());
        assert!(parse_simple_relation("Negates").is_some());
        assert!(parse_simple_relation("AlternativeTo").is_some());
        assert!(parse_simple_relation("AffirmativeDefenseTo").is_some());
        assert!(parse_simple_relation("SafeHarborFor").is_some());
        assert!(parse_simple_relation("ExhaustionRequiredFor").is_some());
        assert!(parse_simple_relation("Precedes").is_some());
        assert!(parse_simple_relation("Implies").is_some());
        assert!(parse_simple_relation("Composes").is_some());
        assert!(parse_simple_relation("Triggers").is_some());
        assert!(parse_simple_relation("Rebuts").is_some());
        assert!(parse_simple_relation("InventedKind").is_none());
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
    fn build_entry_rejects_missing_lock() {
        let item = RawSourceWithName {
            name: "x".into(),
            raw: RawSource {
                version: "1".into(),
                kind: "Statute".into(),
                url: "".into(),
                description: None,
            },
        };
        let lock = HashMap::new();
        let err = build_entry(item, &lock).unwrap_err();
        assert!(err.contains("praxis.lock missing hash"), "got: {err}");
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

    #[test]
    fn build_entry_statute_gets_only_hash_claim() {
        let item = RawSourceWithName {
            name: "sox_1514a".into(),
            raw: RawSource {
                version: "2002".into(),
                kind: "Statute".into(),
                url: "https://example.com/sox.txt".into(),
                description: None,
            },
        };
        let mut lock = HashMap::new();
        lock.insert("sox_1514a@2002".into(), "deadbeef".into());
        let entry = build_entry(item, &lock).unwrap();
        // Non-Lexicon: just the hash claim.
        assert_eq!(entry.identity.0.len(), 1);
        assert!(matches!(
            entry.identity.0[0].concept,
            IdentityConcept::RawHash
        ));
    }
}
