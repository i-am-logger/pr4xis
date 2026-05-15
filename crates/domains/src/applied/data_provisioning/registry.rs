//! Runtime registry — `RegistryEntry` instances loaded from the
//! workspace-root `praxis.toml` (Cargo-precedent naming: manifest →
//! `praxis.toml`, future lock file → `praxis.lock`).
//!
//! The TOML bytes are embedded at compile time via `include_str!` and
//! parsed lazily on first access via `OnceLock`. Adding a managed dataset
//! = appending a `[[source]]` table to `praxis.toml`; no Rust code
//! changes are needed for new entries that fit the existing decoder
//! dispatch.
//!
//! ### TOML schema
//!
//! ```toml
//! [[source]]
//! name = "wordnet"
//! description = "..."
//! remote_location = "https://..."
//! local_path = "crates/domains/data/wordnet/english-wordnet-2025.xml"
//! content_type = "XmlLmf"   # one of the ContentType variants
//! gzipped = true
//!
//! [[source.identity]]
//! kind = "XmlElementAttribute"
//! element = "Lexicon"
//! attribute = "version"
//! expected = "2025"
//!
//! [[source.identity]]
//! kind = "RawHash"
//! sha256 = "6f49adeec1..."
//! ```
//!
//! The `kind` discriminator on `[[source.identity]]` selects the
//! `ClaimData` variant. Currently understood kinds: `XmlElementAttribute`,
//! `RawHash`, `Sha256` (alias for `RawHash`). Unknown kinds fail the parse
//! at startup — fail-closed.
//!
//! ### Why `include_str!` not `fs::read`
//!
//! The registry is needed in WASM and other no-filesystem contexts.
//! Embedding the TOML at compile time keeps the registry a single
//! self-contained artifact. `pr4xis source add` (M6) will rewrite the
//! checked-in file; the running process re-reads on next start.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use std::sync::OnceLock;

use serde::Deserialize;

use super::ontology::{ContentType, RegistryEntry};
use crate::formal::meta::artifact_identity::ontology::{
    ClaimData, CompositeIdentity, IdentityClaim, IdentityConcept,
};

/// Embedded manifest. Single source of truth for the bundled (committed)
/// entries; M6's `pr4xis source add` mutates the file on disk and the
/// running process picks up changes on next start.
///
/// Path is workspace-root relative: this file is at
/// `crates/domains/src/applied/data_provisioning/registry.rs`, so four
/// `..` segments reach the workspace root where `praxis.toml` lives.
const PRAXIS_TOML: &str = include_str!("../../../../../praxis.toml");

/// Process-wide cache of the parsed registry. Lazily initialized on
/// first call to [`data_sources`]; any parse error panics, because a
/// broken `praxis.toml` would corrupt every axiom that iterates the
/// registry — fail-closed at startup is preferable to fail-mysteriously
/// at axiom-check time.
static REGISTRY: OnceLock<Vec<RegistryEntry>> = OnceLock::new();

/// Return the loaded registry. First call parses `praxis.toml`;
/// subsequent calls return the cached slice.
pub fn data_sources() -> &'static [RegistryEntry] {
    REGISTRY
        .get_or_init(|| {
            parse_sources_toml(PRAXIS_TOML)
                .unwrap_or_else(|e| panic!("invalid workspace-root praxis.toml: {e}"))
        })
        .as_slice()
}

/// Look up a `RegistryEntry` by name. Linear scan because the registry is
/// small; switch to a map if it grows past ~100 entries.
pub fn by_name(name: &str) -> Option<&'static RegistryEntry> {
    data_sources().iter().find(|e| e.name == name)
}

/// Resolve the composite identity for a registered entry. Returns the
/// entry's embedded `CompositeIdentity` (cloned) so callers don't have to
/// deal with the registry's `'static` lifetime. `None` if `name` is not
/// registered.
///
/// Kept as a name-keyed function for source-compatibility with the old
/// hardcoded-match implementation; equivalent to
/// `by_name(name).map(|e| e.identity.clone())`.
pub fn resolve_identity(name: &str) -> Option<CompositeIdentity> {
    by_name(name).map(|e| e.identity.clone())
}

/// Every registry entry's resolved identity. Used by callers that want to
/// iterate the full set without re-doing the name lookup.
pub fn resolved_identities() -> Vec<(&'static str, CompositeIdentity)> {
    data_sources()
        .iter()
        .map(|e| (e.name.as_str(), e.identity.clone()))
        .collect()
}

// =============================================================================
// TOML parsing
// =============================================================================

#[derive(Debug, Deserialize)]
struct RawManifest {
    #[serde(default)]
    source: Vec<RawSource>,
}

#[derive(Debug, Deserialize)]
struct RawSource {
    name: String,
    description: String,
    remote_location: String,
    local_path: String,
    content_type: String,
    #[serde(default)]
    gzipped: bool,
    #[serde(default)]
    identity: Vec<RawIdentity>,
}

#[derive(Debug, Deserialize)]
struct RawIdentity {
    kind: String,
    // XmlElementAttribute fields
    #[serde(default)]
    element: Option<String>,
    #[serde(default)]
    attribute: Option<String>,
    #[serde(default)]
    expected: Option<String>,
    // RawHash / Sha256 field
    #[serde(default)]
    sha256: Option<String>,
}

fn parse_sources_toml(text: &str) -> Result<Vec<RegistryEntry>, String> {
    let manifest: RawManifest = toml::from_str(text).map_err(|e| format!("toml parse: {e}"))?;
    manifest
        .source
        .into_iter()
        .map(raw_to_entry)
        .collect::<Result<Vec<_>, _>>()
}

fn raw_to_entry(raw: RawSource) -> Result<RegistryEntry, String> {
    let content_type = parse_content_type(&raw.content_type).ok_or_else(|| {
        format!(
            "source `{}`: unknown content_type `{}`",
            raw.name, raw.content_type
        )
    })?;
    let claims = raw
        .identity
        .into_iter()
        .map(|i| raw_identity_to_claim(&raw.name, i))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RegistryEntry {
        name: raw.name,
        description: raw.description,
        remote_location: raw.remote_location,
        local_path: raw.local_path,
        content_type,
        gzipped: raw.gzipped,
        identity: CompositeIdentity(claims),
    })
}

fn parse_content_type(s: &str) -> Option<ContentType> {
    Some(match s {
        "XmlLmf" => ContentType::XmlLmf,
        "Pdf" => ContentType::Pdf,
        "Plaintext" => ContentType::Plaintext,
        "Json" => ContentType::Json,
        "Video" => ContentType::Video,
        "Audio" => ContentType::Audio,
        "Binary" => ContentType::Binary,
        "Statute" => ContentType::Statute,
        _ => return None,
    })
}

fn raw_identity_to_claim(source_name: &str, raw: RawIdentity) -> Result<IdentityClaim, String> {
    match raw.kind.as_str() {
        "XmlElementAttribute" => {
            let element = raw.element.ok_or_else(|| {
                format!("source `{source_name}`: XmlElementAttribute claim missing `element`")
            })?;
            let attribute = raw.attribute.ok_or_else(|| {
                format!("source `{source_name}`: XmlElementAttribute claim missing `attribute`")
            })?;
            let expected = raw.expected.ok_or_else(|| {
                format!("source `{source_name}`: XmlElementAttribute claim missing `expected`")
            })?;
            Ok(IdentityClaim {
                concept: IdentityConcept::XmlElementAttribute,
                data: ClaimData::XmlAttribute {
                    element,
                    attribute,
                    expected,
                },
            })
        }
        // `RawHash` and `Sha256` are aliases — the leaf concept is
        // `RawHash`; `Sha256` names the algorithm, which is currently the
        // only supported one.
        "RawHash" | "Sha256" => {
            let hex = raw
                .sha256
                .ok_or_else(|| format!("source `{source_name}`: RawHash claim missing `sha256`"))?;
            Ok(IdentityClaim {
                concept: IdentityConcept::RawHash,
                data: ClaimData::Sha256(hex),
            })
        }
        other => Err(format!(
            "source `{source_name}`: unknown identity kind `{other}` — supported: XmlElementAttribute, RawHash, Sha256"
        )),
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn parses_empty_manifest() {
        let entries = parse_sources_toml("").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn parses_wordnet_shape() {
        let text = r#"
[[source]]
name = "wn"
description = "test"
remote_location = "https://example.com"
local_path = "tmp/wn.xml"
content_type = "XmlLmf"
gzipped = true

[[source.identity]]
kind = "XmlElementAttribute"
element = "Lexicon"
attribute = "version"
expected = "2025"

[[source.identity]]
kind = "RawHash"
sha256 = "deadbeef"
"#;
        let entries = parse_sources_toml(text).unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.name, "wn");
        assert!(matches!(e.content_type, ContentType::XmlLmf));
        assert!(e.gzipped);
        assert_eq!(e.identity.0.len(), 2);
    }

    #[test]
    fn rejects_unknown_content_type() {
        let text = r#"
[[source]]
name = "x"
description = ""
remote_location = ""
local_path = ""
content_type = "NotAType"
"#;
        let err = parse_sources_toml(text).unwrap_err();
        assert!(err.contains("unknown content_type"), "got: {err}");
    }

    #[test]
    fn rejects_unknown_identity_kind() {
        let text = r#"
[[source]]
name = "x"
description = ""
remote_location = ""
local_path = ""
content_type = "XmlLmf"

[[source.identity]]
kind = "InventedScheme"
"#;
        let err = parse_sources_toml(text).unwrap_err();
        assert!(err.contains("unknown identity kind"), "got: {err}");
    }

    #[test]
    fn rejects_xml_claim_missing_field() {
        let text = r#"
[[source]]
name = "x"
description = ""
remote_location = ""
local_path = ""
content_type = "XmlLmf"

[[source.identity]]
kind = "XmlElementAttribute"
element = "Lexicon"
# missing attribute and expected
"#;
        let err = parse_sources_toml(text).unwrap_err();
        assert!(err.contains("missing `attribute`"), "got: {err}");
    }

    #[test]
    fn statute_content_type_parses() {
        let text = r#"
[[source]]
name = "sox"
description = ""
remote_location = ""
local_path = ""
content_type = "Statute"
"#;
        let entries = parse_sources_toml(text).unwrap();
        assert!(matches!(entries[0].content_type, ContentType::Statute));
    }
}
