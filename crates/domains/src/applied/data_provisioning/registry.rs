//! Runtime registry — `RegistryEntry` instances loaded from the
//! `praxis.toml` manifest. Lock hashes loaded from `praxis.lock`.
//!
//! Both texts are recovered from the committed registry MANIFEST `.prx`
//! (`include_bytes!` of `data/registry/praxis-registry.prx`, admitted only
//! through the fail-closed baked-root content gate
//! [`super::registry_prx::load_registry_manifest`]) and
//! parsed lazily on first access via `OnceLock`. The `.prx` is the SOLE carrier
//! of the manifest in the published crate — no raw `praxis.toml` / `praxis.lock`
//! ships. The schema and parsing rules are described inline below; the
//! user-facing documentation lives in the comments at the top of `praxis.toml`
//! and `praxis.lock`.
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
//! `praxis.lock` pins their content digests in the tagged grammar
//! `<algorithm>:<64 lowercase hex>`:
//!
//! ```toml
//! [hashes]
//! "english_wordnet@2025" = "blake3:41ad53811..."
//! ```
//!
//! At load time the parser:
//! 1. Reads each `[sources.<name>]` table.
//! 2. Maps `type = "<concept>"` → typed [`SourceTaxonomyConcept`] via
//!    [`parse_concept`]. Unknown names panic at startup (fail-closed).
//! 3. Looks up `"<name>@<version>"` in `praxis.lock`'s `[hashes]` table
//!    to get the pinned digest, parsed into a typed [`LockDigest`]
//!    (`blake3:` → BLAKE3, `sha256:` / bare hex → SHA-256, any other tag
//!    fails closed).
//! 4. Synthesizes the `CompositeIdentity`:
//!      - `XmlElementAttribute` claim for `<Lexicon version=...>` if the
//!        kind is in the Lexicon family (Global WordNet LMF convention).
//!      - a `RawHash` claim carrying the lock digest's algorithm + hex,
//!        applied to every kind (Dolstra 2006 content-addressing; verified
//!        through the one multi-algorithm leg,
//!        [`pr4xis_runtime::address::hash_hex`]).
//! 5. Returns a `Vec<RegistryEntry>` cached for process lifetime.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use pr4xis_runtime::address::{HashAlgorithm, hash_hex};

use super::ontology::{ContentType, RegistryEntry, canonical_encoding};
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

use super::registry_prx::load_registry_manifest;

/// The registered-source MANIFEST, loaded ONCE from the committed
/// `praxis-registry.prx` through the self-contained baked-root gate
/// ([`load_registry_manifest`]). The `.prx` is the SOLE carrier of the manifest
/// in the published crate — there is no raw `praxis.toml` / `praxis.lock` in the
/// package and no `build.rs`-emitted `&str` const. Both files are recovered as
/// text here and parsed by [`data_sources`] / [`lock_data`] exactly as before; a
/// failed load (tampered / stale / mis-baked-root `.prx`) panics fail-closed.
///
/// This is the BOOTSTRAP ROOT: it loads via the baked
/// [`PRAXIS_REGISTRY_ROOT_HEX`](super::registry_prx::PRAXIS_REGISTRY_ROOT_HEX),
/// NOT via any `lock_*` lookup (which would require the lock to already be
/// loaded) — so the manifest can be "just another `.prx`" without the
/// circularity of loading through the registry it itself populates.
fn registry_manifest() -> &'static (String, String) {
    static MANIFEST: OnceLock<(String, String)> = OnceLock::new();
    MANIFEST.get_or_init(|| {
        load_registry_manifest().unwrap_or_else(|e| {
            panic!("registry .prx (the registered-source manifest) failed to load: {e}")
        })
    })
}

/// The `praxis.toml` text recovered from the committed registry `.prx`.
fn praxis_toml() -> &'static str {
    &registry_manifest().0
}

/// The `praxis.lock` text recovered from the committed registry `.prx`.
fn praxis_lock() -> &'static str {
    &registry_manifest().1
}

static REGISTRY: OnceLock<Vec<RegistryEntry>> = OnceLock::new();
static LOCK: OnceLock<LockData> = OnceLock::new();

/// The loaded registry. First call parses both files and synthesizes the
/// identity claims; subsequent calls return the cached slice.
pub fn data_sources() -> &'static [RegistryEntry] {
    REGISTRY
        .get_or_init(|| {
            let manifest = parse_praxis_toml(praxis_toml())
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

/// The pinned digests from `praxis.lock`. Keys are `"<name>@<version>"`
/// strings; each value is the typed [`LockDigest`] of the raw source bytes.
pub fn lock_hashes() -> &'static HashMap<String, LockDigest> {
    &lock_data().hashes
}

/// The pinned canonical-form signatures from `praxis.lock`. Keys are
/// `"<name>@<version>"`; each value is the [`LockDigest`] of the bytes a
/// registered [`WellBehavedLens`] emits as canonical form (W3C XML
/// Canonicalization 1.1, RFC 8785 JCS, Unicode NFKC, etc.).
///
/// Sources that haven't been signed yet (no entry under
/// `[canonical_signatures]`) return `None` from
/// [`lock_canonical_signature`].
///
/// [`WellBehavedLens`]: crate::formal::meta::well_behaved_lens
pub fn lock_canonical_signatures() -> &'static HashMap<String, LockDigest> {
    &lock_data().canonical_signatures
}

/// Look up the canonical-form signature for a specific source.
/// Returns `None` if no signature is pinned — the source either
/// hasn't been canonicalized yet or has no registered lens.
pub fn lock_canonical_signature(name: &str, version: &str) -> Option<&'static LockDigest> {
    let key = format!("{name}@{version}");
    lock_canonical_signatures().get(&key)
}

/// The pinned byte-exact signatures from `praxis.lock`. Keys are
/// `"<name>@<version>"`; each value is the [`LockDigest`] of the raw source
/// bytes a `RoundTripFidelity::ByteExactGraphFaithful` lens
/// reconstructs exactly (equal to the `[hashes]` pin). Membership
/// marks a source as held to the strict byte-exact PutGet law
/// (M4.ι / #186).
pub fn lock_byte_exact_signatures() -> &'static HashMap<String, LockDigest> {
    &lock_data().byte_exact_signatures
}

/// Look up the byte-exact signature for a specific source. Returns
/// `None` if the source isn't byte-exact-pinned — it is either held
/// only to the canonical-form FLOOR or hasn't been migrated to
/// graph-faithful byte-exactness yet.
pub fn lock_byte_exact_signature(name: &str, version: &str) -> Option<&'static LockDigest> {
    let key = format!("{name}@{version}");
    lock_byte_exact_signatures().get(&key)
}

/// The pinned `.prx` archive content addresses from `praxis.lock`. Keys
/// are `"<name>@<version>"`; each value is the [`LockDigest`] of the rkyv
/// bytes of the compiled `.prx` envelope (its `BinaryEnvelope` content
/// address / Merkle root). This is the integrity claim the `.prx` load gate
/// checks against the bytes it installs — see
/// [`load_prx_gz`](crate::social::software::markup::xml::owl::prx::load_prx_gz).
pub fn lock_archive_signatures() -> &'static HashMap<String, LockDigest> {
    &lock_data().archive_signatures
}

/// Look up the `.prx` archive content address for a specific source.
/// Returns `None` if the source has no published, pinned `.prx` archive —
/// in which case the load gate has nothing to validate the compiled
/// envelope against and fails closed.
pub fn lock_archive_signature(name: &str, version: &str) -> Option<&'static LockDigest> {
    let key = format!("{name}@{version}");
    lock_archive_signatures().get(&key)
}

/// The pinned COMPACT `.prx` archive content addresses from `praxis.lock`.
/// Keys are `"<name>@<version>"`; each value is the [`LockDigest`] of the
/// compact archive's uncompressed succinct bytes (its content address). Unlike
/// `[archive_signatures]` — which pins the rkyv `BinaryEnvelope`, a
/// toolchain-dependent layout — the compact codec is pure dependency-free
/// bit-packing, so this address is **portable across toolchains and targets**.
/// It is the integrity claim the compact runtime load gate verifies against the
/// bytes it installs; a compact `.prx.gz` whose succinct bytes do not hash to
/// this pin is rejected fail-closed before any data is materialized.
pub fn lock_compact_archive_signatures() -> &'static HashMap<String, LockDigest> {
    &lock_data().compact_archive_signatures
}

/// Look up the COMPACT `.prx` archive content address for a specific source.
/// Returns `None` if the source has no pinned compact archive — the compact
/// load gate then has nothing to validate against and falls back (to the
/// envelope archive or the authoritative source).
pub fn lock_compact_archive_signature(name: &str, version: &str) -> Option<&'static LockDigest> {
    let key = format!("{name}@{version}");
    lock_compact_archive_signatures().get(&key)
}

/// The pinned whole-graph `GraphSnapshot` content addresses from
/// `praxis.lock`. Keys are `GraphVersion` labels; each value is the
/// [`LockDigest`] of the snapshot's rkyv blob (its `MerkleRoot`). The
/// integrity claim the snapshot load gate checks
/// (`crate::formal::meta::praxis_knowledge_graph::snapshot`).
/// Empty in-repo (#271 pins none; the operator pins published snapshots).
pub fn lock_snapshot_signatures() -> &'static HashMap<String, LockDigest> {
    &lock_data().snapshot_signatures
}

/// Look up the snapshot `MerkleRoot` for a specific `GraphVersion`. Returns
/// `None` if that version has no pinned snapshot — the load gate then has
/// nothing to validate against and fails closed.
pub fn lock_snapshot_signature(version: &str) -> Option<&'static LockDigest> {
    lock_snapshot_signatures().get(version)
}

fn lock_data() -> &'static LockData {
    LOCK.get_or_init(|| {
        parse_praxis_lock(praxis_lock()).unwrap_or_else(|e| panic!("invalid praxis.lock: {e}"))
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
    /// Explicit workspace-relative disk path (audit 2026-06-12 D-8) —
    /// registry data replacing the `local_path_override` Rust table for
    /// sources whose on-disk name isn't the `{name}-{version}` formula.
    #[serde(default)]
    local_path: Option<String>,
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
    lock: &HashMap<String, LockDigest>,
) -> Result<RegistryEntry, String> {
    let RawSourceWithName { name, raw } = item;
    let kind = parse_concept(&raw.kind).ok_or_else(|| {
        format!(
            "source `{}`: unknown type `{}` — must be a leaf concept in SourceTaxonomy",
            name, raw.kind
        )
    })?;

    let key = format!("{}@{}", name, raw.version);
    let lock_digest = lock.get(&key);

    let mut claims = Vec::with_capacity(2);
    // Lexicon-family kinds (Language, DomainLexicon, LegalLexicon) ship
    // as XML-LMF and self-declare their version in the `<Lexicon
    // version="...">` attribute (Global WordNet LMF 1.3 convention).
    // We synthesize the corresponding identity claim so the existing
    // XML attribute extractor validates the upstream's self-description.
    // Gated on the WN-LMF *encoding*, not merely the Lexicon family: a
    // plain-text Lexicon (AGID, `InflectionLexicon`) has no `<Lexicon
    // version=…>` element to validate (audit 2026-06-12 MORPH). Both the
    // open-class WordNet (`XmlLmf`) and the closed-class function-word / legal
    // lexica (`XmlLmfLexicon`) carry the `<Lexicon version="...">` element.
    if is_lexicon(kind)
        && matches!(
            canonical_encoding(kind),
            ContentType::XmlLmf | ContentType::XmlLmfLexicon
        )
    {
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
    // claim (Dolstra 2006 content-addressing), typed with the lock pin's
    // algorithm so verification dispatches through the one verify leg
    // (`pr4xis_runtime::address::hash_hex`). Sources registered in
    // praxis.toml without a lock entry carry a Stub identity claim
    // that EveryDataSourceHasIdentity recognizes as a registered-but-
    // not-yet-loadable state. The LockManifestAgreement axiom skips
    // Stub-only entries (no claim to compare against the lock).
    match lock_digest {
        Some(digest) => claims.push(IdentityClaim {
            concept: IdentityConcept::RawHash,
            data: digest.claim_data(),
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
        local_path: raw.local_path,
        identity: CompositeIdentity(claims),
    })
}

// =============================================================================
// praxis.lock digest values — the tagged grammar
// =============================================================================

/// The lockfile tag naming a [`HashAlgorithm`] — the ONE lowering between the
/// typed algorithm and its `praxis.lock` wire form (`<tag>:<hex>`). The parser
/// inverts it via [`algorithm_from_tag`]; the writer
/// ([`super::lockfile`]) prefixes it onto freshly emitted digests.
pub fn algorithm_tag(algorithm: HashAlgorithm) -> &'static str {
    match algorithm {
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Sha512 => "sha512",
        HashAlgorithm::Blake3 => "blake3",
    }
}

/// Inverse of [`algorithm_tag`] over the tags the lock grammar admits.
/// `None` for an unknown tag — the caller fails closed, naming the tag.
pub fn algorithm_from_tag(tag: &str) -> Option<HashAlgorithm> {
    match tag {
        "sha256" => Some(HashAlgorithm::Sha256),
        "blake3" => Some(HashAlgorithm::Blake3),
        _ => None,
    }
}

/// Why a `praxis.lock` digest value failed to parse — fail-closed: an unknown
/// algorithm tag is NAMED, never guessed past or skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockDigestParseError {
    /// The value carries an `<algorithm>:` tag the lock grammar does not
    /// admit (the grammar admits `blake3` and the legacy `sha256`).
    UnknownAlgorithmTag { tag: String },
    /// The digest part is not exactly 64 lowercase hex characters.
    MalformedDigest { value: String },
}

impl core::fmt::Display for LockDigestParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LockDigestParseError::UnknownAlgorithmTag { tag } => {
                write!(
                    f,
                    "unknown digest algorithm tag {tag:?} (expected `blake3` or `sha256`)"
                )
            }
            LockDigestParseError::MalformedDigest { value } => {
                write!(f, "not a 64-char lowercase hex digest: {value:?}")
            }
        }
    }
}

impl std::error::Error for LockDigestParseError {}

/// A typed `praxis.lock` digest value: the parsed form of the tagged grammar
/// `<algorithm>:<64 lowercase hex>`.
///
/// - `"blake3:<hex>"` → [`HashAlgorithm::Blake3`] — the address algorithm
///   praxis emits under ([`pr4xis_runtime::address::ADDRESS_ALGORITHM`]).
/// - `"sha256:<hex>"` → [`HashAlgorithm::Sha256`] — an explicit legacy pin.
/// - bare `"<hex>"` → [`HashAlgorithm::Sha256`] — the historical untagged
///   form, kept loadable.
/// - any other tag → [`LockDigestParseError::UnknownAlgorithmTag`], fail-closed.
///
/// Emit-one / verify-many: praxis WRITES pins under exactly one algorithm per
/// format epoch (the writer tags them `blake3:`), but VERIFIES a parsed pin
/// under whatever algorithm it names — [`Self::verifies`] dispatches through
/// the one verify leg, [`pr4xis_runtime::address::hash_hex`]. A payload never
/// selects its own verifier: the algorithm comes from the trusted lock value,
/// never from the bytes being admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockDigest {
    /// The hash function this pin's digest was taken under.
    pub algorithm: HashAlgorithm,
    /// The 64-char lowercase hex digest.
    pub digest_hex: String,
}

impl LockDigest {
    /// Parse the tagged lock grammar (see the type docs for the rules).
    pub fn parse(value: &str) -> Result<Self, LockDigestParseError> {
        let (algorithm, hex) = match value.split_once(':') {
            Some((tag, hex)) => (
                algorithm_from_tag(tag).ok_or_else(|| {
                    LockDigestParseError::UnknownAlgorithmTag {
                        tag: tag.to_string(),
                    }
                })?,
                hex,
            ),
            None => (HashAlgorithm::Sha256, value),
        };
        if !is_lowercase_hex_digest(hex) {
            return Err(LockDigestParseError::MalformedDigest {
                value: value.to_string(),
            });
        }
        Ok(Self {
            algorithm,
            digest_hex: hex.to_string(),
        })
    }

    /// A pin praxis itself emitted: `digest_hex` under the one emit algorithm
    /// ([`pr4xis_runtime::address::ADDRESS_ALGORITHM`]) — the typed form of a
    /// freshly computed `ContentAddress::to_hex()`.
    pub fn address(digest_hex: impl Into<String>) -> Self {
        Self {
            algorithm: pr4xis_runtime::address::ADDRESS_ALGORITHM,
            digest_hex: digest_hex.into(),
        }
    }

    /// The ONE verify leg: re-derive the digest of `bytes` under this pin's
    /// algorithm ([`hash_hex`]) and compare. Never a parallel hash call.
    pub fn verifies(&self, bytes: &[u8]) -> bool {
        hash_hex(self.algorithm, bytes) == self.digest_hex
    }

    /// The typed identity claim this pin discharges through
    /// [`crate::formal::meta::artifact_identity::schemes::raw_hash::verify`].
    pub fn claim_data(&self) -> ClaimData {
        ClaimData::HashAlgorithm {
            algorithm: self.algorithm,
            digest_hex: self.digest_hex.clone(),
        }
    }
}

impl core::fmt::Display for LockDigest {
    /// The tagged wire form, `<tag>:<hex>` — the inverse of [`Self::parse`]
    /// (the bare legacy input form is never re-emitted).
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", algorithm_tag(self.algorithm), self.digest_hex)
    }
}

// =============================================================================
// praxis.lock parsing
// =============================================================================

/// The parsed `praxis.lock` payload. Every value is a typed [`LockDigest`]
/// parsed from the tagged grammar (`blake3:<hex>` for praxis-emitted pins;
/// `sha256:<hex>` / bare hex for legacy pins, kept loadable).
///
/// Two parallel digest spaces — one over the *raw bytes* of each
/// loaded source, one over the *canonical-form bytes* produced by
/// running the source through its registered well-behaved lens:
///
/// - `hashes`: digest of the source bytes as delivered by the
///   fetch pipeline (after transport-layer decompression, before
///   any decoder runs). Verifies "we got the same source we
///   expected." Per Dolstra (2006) — content-addressed storage.
///
/// - `canonical_signatures`: digest of the canonical-form bytes
///   produced by the source's [`WellBehavedLens`] (Foster, Greenwald,
///   Moore, Pierce & Schmitt 2007 *ACM TOPLAS* 29(3) "Combinators
///   for Bidirectional Tree Transformations") applied to the raw
///   source. Verifies "the source, *after* lens normalization, has
///   the same structural shape" — drift in the lens implementation
///   itself, or in the underlying canonical form (W3C XML
///   Canonicalization 1.1 — Boyer & Marcy 2008 W3C Rec, RFC 8785
///   JCS, Unicode NFKC, etc.), surfaces here while the raw-bytes
///   digest remains stable.
///
/// - `byte_exact_signatures`: digest of the bytes a byte-exact
///   graph-faithful lens (`RoundTripFidelity::ByteExactGraphFaithful`)
///   reconstructs via `put(get(b))`. Because that lens proves
///   byte-identity (`put(get(b)) == b`, M4.ι / #186), this value
///   equals the raw `[hashes]` pin — membership in this section is the
///   auditable signal that a source is held to the strict byte-exact
///   PutGet law rather than only canonical-form equality (Foster et
///   al. 2007 §2.2 PutGet at the byte-identity counit, Mac Lane §IV.4).
///
/// - `archive_signatures`: content address of the rkyv bytes of the
///   compiled `.prx` envelope (`BinaryEnvelope`) for the source — the
///   Merkle root of the archived node (Merkle 1987; IPFS/
///   IPLD — Benet 2014; Git's content-addressed DAG; Dolstra 2006).
///   Unlike the three spaces above, this pins the *compiled archive*,
///   not the source bytes or a lens output: it is the integrity claim
///   the `.prx` load gate verifies against the envelope it is about to
///   install (W3C Subresource Integrity 2016 — the fetched resource must
///   match the pinned digest). A `.prx.gz` whose envelope does not hash
///   to this pin is rejected fail-closed *before* any data is installed,
///   so a tampered archive carrying a genuine source-address label
///   cannot poison the runtime. It need NOT equal the `[hashes]` pin —
///   it addresses a different artifact (the compiled rkyv envelope, not
///   the raw source).
///
/// Five of the six digest spaces (`hashes`, `canonical_signatures`,
/// `byte_exact_signatures`, `archive_signatures`, `compact_archive_signatures`)
/// are keyed by `"<name>@<version>"`. The sixth, `snapshot_signatures`, is keyed
/// by `GraphVersion` instead (see its field doc).
///
/// [`WellBehavedLens`]: crate::formal::meta::well_behaved_lens
#[derive(Debug, Default)]
pub struct LockData {
    pub hashes: HashMap<String, LockDigest>,
    pub canonical_signatures: HashMap<String, LockDigest>,
    pub byte_exact_signatures: HashMap<String, LockDigest>,
    pub archive_signatures: HashMap<String, LockDigest>,
    /// Content address of the COMPACT `.prx` archive's uncompressed succinct
    /// bytes, keyed by `"<name>@<version>"`. The portable (toolchain-independent)
    /// sibling of `[archive_signatures]`: the compact codec is dependency-free
    /// bit-packing, so the same source yields the same compact address on every
    /// toolchain and target. The integrity claim the compact runtime load gate
    /// checks; like `[archive_signatures]` it pins a compiled artifact (not the
    /// raw source) and requires a matching `[hashes]` entry.
    pub compact_archive_signatures: HashMap<String, LockDigest>,
    /// `MerkleRoot` of a whole-graph `GraphSnapshot` rkyv blob
    /// (`crate::formal::meta::praxis_knowledge_graph::snapshot`), keyed by
    /// its `GraphVersion`. Unlike `[archive_signatures]`, a snapshot is keyed
    /// by the synthetic GraphVersion (the slice's own content address) and has
    /// NO upstream fetched source, so — by deliberate asymmetry — an entry
    /// here does NOT require a matching `[hashes]` pin. Empty until the
    /// operator pins a published snapshot (rkyv wire layout is
    /// toolchain-dependent, so nothing is committed in-repo; #271 pins none).
    pub snapshot_signatures: HashMap<String, LockDigest>,
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
    #[serde(default)]
    canonical_signatures: HashMap<String, String>,
    #[serde(default)]
    byte_exact_signatures: HashMap<String, String>,
    #[serde(default)]
    archive_signatures: HashMap<String, String>,
    #[serde(default)]
    compact_archive_signatures: HashMap<String, String>,
    #[serde(default)]
    snapshot_signatures: HashMap<String, String>,
}

/// Parse one section's raw string values into typed [`LockDigest`]s,
/// fail-closed: an unknown tag or malformed digest is an error naming the
/// section and key, never skipped.
fn parse_section(
    section: &str,
    raw: HashMap<String, String>,
) -> Result<HashMap<String, LockDigest>, String> {
    raw.into_iter()
        .map(|(key, value)| {
            let digest = LockDigest::parse(&value)
                .map_err(|e| format!("praxis.lock: `[{section}.\"{key}\"]`: {e}"))?;
            Ok((key, digest))
        })
        .collect()
}

fn parse_praxis_lock(text: &str) -> Result<LockData, String> {
    let raw: RawLockFile = toml::from_str(text).map_err(|e| format!("toml parse: {e}"))?;
    // Every value in every section parses through the tagged grammar —
    // `blake3:<hex>` / `sha256:<hex>` / bare legacy hex; an unknown tag or
    // malformed digest fails closed naming the offending entry.
    let hashes = parse_section("hashes", raw.hashes)?;
    let canonical_signatures = parse_section("canonical_signatures", raw.canonical_signatures)?;
    let byte_exact_signatures = parse_section("byte_exact_signatures", raw.byte_exact_signatures)?;
    let archive_signatures = parse_section("archive_signatures", raw.archive_signatures)?;
    let compact_archive_signatures =
        parse_section("compact_archive_signatures", raw.compact_archive_signatures)?;
    let snapshot_signatures = parse_section("snapshot_signatures", raw.snapshot_signatures)?;

    // Every canonical_signatures key must also exist in [hashes] —
    // the signature is the lens-output of an already-pinned source.
    // A free-standing signature without a pinned raw-bytes digest is
    // a configuration error.
    for key in canonical_signatures.keys() {
        if !hashes.contains_key(key) {
            return Err(format!(
                "praxis.lock: `[canonical_signatures.\"{key}\"]` has no matching \
                 entry in `[hashes]` — the signature pins the lens output of an \
                 already-hashed source"
            ));
        }
    }
    // A byte_exact_signature pins a source held to the strict byte-exact
    // PutGet law (M4.ι / #186): `put(get(b)) == b`, so the lens output
    // digest equals the raw-bytes digest. Each entry must therefore (a) name
    // an already-hashed source and (b) EQUAL that source's `[hashes]` pin
    // (same algorithm, same digest) — a byte-exact signature that differs
    // from the raw pin is a contradiction in terms.
    for (key, sig) in &byte_exact_signatures {
        let Some(raw_hash) = hashes.get(key) else {
            return Err(format!(
                "praxis.lock: `[byte_exact_signatures.\"{key}\"]` has no matching \
                 entry in `[hashes]` — the signature pins the byte-exact round-trip \
                 of an already-hashed source"
            ));
        };
        if sig != raw_hash {
            return Err(format!(
                "praxis.lock: `[byte_exact_signatures.\"{key}\"]` = \"{sig}\" must equal \
                 the raw `[hashes]` pin \"{raw_hash}\" — byte-exactness means \
                 put(get(b)) == b, so the round-trip digest is the raw-bytes digest"
            ));
        }
    }
    // An archive_signature pins the content address of the compiled `.prx`
    // envelope for an already-hashed source (the `BinaryEnvelope` Merkle
    // root). Each entry must name a source that also has a raw `[hashes]`
    // pin (the archive is the compiled form of that source). Unlike a
    // byte_exact_signature it will generally NOT equal the `[hashes]` pin
    // (not enforced) — it addresses the compiled rkyv envelope, a different
    // artifact than the raw source bytes.
    for key in archive_signatures.keys() {
        if !hashes.contains_key(key) {
            return Err(format!(
                "praxis.lock: `[archive_signatures.\"{key}\"]` has no matching \
                 entry in `[hashes]` — the signature pins the compiled `.prx` \
                 archive of an already-hashed source"
            ));
        }
    }
    // A compact_archive_signature pins the content address of the COMPACT `.prx`
    // archive for an already-hashed source — same contract as [archive_signatures]
    // (must name a [hashes]-pinned source), but addresses the portable
    // dependency-free succinct bytes, not the rkyv envelope.
    for key in compact_archive_signatures.keys() {
        if !hashes.contains_key(key) {
            return Err(format!(
                "praxis.lock: `[compact_archive_signatures.\"{key}\"]` has no matching \
                 entry in `[hashes]` — the signature pins the compact `.prx` archive \
                 of an already-hashed source"
            ));
        }
    }
    // A snapshot_signature pins the MerkleRoot of a whole-graph GraphSnapshot
    // rkyv blob, keyed by its GraphVersion. DELIBERATE ASYMMETRY vs
    // [archive_signatures]: a snapshot is keyed by the synthetic GraphVersion
    // (the slice's own content address) and has NO upstream fetched source, so
    // it must NOT require a matching [hashes] entry. (Its values still parse
    // through the same tagged grammar above.)
    Ok(LockData {
        hashes,
        canonical_signatures,
        byte_exact_signatures,
        archive_signatures,
        compact_archive_signatures,
        snapshot_signatures,
    })
}

/// Predicate: the input is exactly 64 ASCII characters, all of them
/// `0-9` or `a-f` — the hex form of one 256-bit digest (BLAKE3 and
/// SHA-256 both emit 256 bits).
fn is_lowercase_hex_digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    /// A legacy (bare-hex → SHA-256) digest, as the parser types it.
    fn sha(hex: &str) -> LockDigest {
        LockDigest {
            algorithm: HashAlgorithm::Sha256,
            digest_hex: hex.into(),
        }
    }

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
        // One blake3-tagged pin (the emitted form), one bare legacy pin
        // (parsed as SHA-256, kept loadable).
        let text = r#"
[hashes]
"english_wordnet@2025" = "blake3:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
"usc_title_18@2024"    = "cafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00d"
"#;
        let lock = parse_praxis_lock(text).unwrap();
        assert_eq!(lock.hashes.len(), 2);
        assert_eq!(
            lock.hashes.get("english_wordnet@2025").unwrap(),
            &LockDigest {
                algorithm: HashAlgorithm::Blake3,
                digest_hex: "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
                    .into(),
            }
        );
        assert_eq!(
            lock.hashes.get("usc_title_18@2024").unwrap(),
            &LockDigest {
                algorithm: HashAlgorithm::Sha256,
                digest_hex: "cafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00d"
                    .into(),
            }
        );
    }

    #[test]
    fn lock_digest_parses_explicit_sha256_tag() {
        let d = LockDigest::parse(
            "sha256:cafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00d",
        )
        .unwrap();
        assert_eq!(d.algorithm, HashAlgorithm::Sha256);
    }

    #[test]
    fn lock_digest_rejects_unknown_tag_fail_closed() {
        // An unrecognized algorithm tag must fail closed NAMING the tag —
        // never guessed past, never skipped.
        let err = LockDigest::parse(
            "md5:cafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00d",
        )
        .unwrap_err();
        assert_eq!(
            err,
            LockDigestParseError::UnknownAlgorithmTag { tag: "md5".into() }
        );
        // And a lock file carrying one fails to parse, naming section + key.
        let text = r#"
[hashes]
"x@1" = "md5:cafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00d"
"#;
        let err = parse_praxis_lock(text).unwrap_err();
        assert!(
            err.contains("md5") && err.contains("hashes") && err.contains("x@1"),
            "got: {err}"
        );
    }

    #[test]
    fn lock_digest_round_trips_through_display() {
        // Display writes the tagged wire form; parse inverts it.
        let hex = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
        let blake = LockDigest::address(hex);
        assert_eq!(blake.to_string(), format!("blake3:{hex}"));
        assert_eq!(LockDigest::parse(&blake.to_string()).unwrap(), blake);
        let legacy = LockDigest {
            algorithm: HashAlgorithm::Sha256,
            digest_hex: hex.into(),
        };
        assert_eq!(legacy.to_string(), format!("sha256:{hex}"));
        assert_eq!(LockDigest::parse(&legacy.to_string()).unwrap(), legacy);
    }

    #[test]
    fn lock_digest_verifies_dispatches_by_algorithm() {
        // The one verify leg: the SAME bytes verify under a blake3 pin and
        // under a legacy sha256 pin, each dispatched through hash_hex.
        let bytes = b"the ground";
        let blake = LockDigest::address(hash_hex(HashAlgorithm::Blake3, bytes));
        assert!(blake.verifies(bytes));
        let legacy = LockDigest {
            algorithm: HashAlgorithm::Sha256,
            digest_hex: hash_hex(HashAlgorithm::Sha256, bytes),
        };
        assert!(legacy.verifies(bytes));
        assert!(!blake.verifies(b"other bytes"));
        assert!(!legacy.verifies(b"other bytes"));
    }

    #[test]
    fn parses_empty_lock() {
        let lock = parse_praxis_lock("").unwrap();
        assert!(lock.hashes.is_empty());
        assert!(lock.canonical_signatures.is_empty());
    }

    #[test]
    fn parses_canonical_signatures_section() {
        let text = r#"
[hashes]
"english_wordnet@2025" = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
"uslm_xsd@1.0.18"      = "cafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00dcafef00d"

[canonical_signatures]
"uslm_xsd@1.0.18" = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
"#;
        let lock = parse_praxis_lock(text).unwrap();
        assert_eq!(lock.hashes.len(), 2);
        assert_eq!(lock.canonical_signatures.len(), 1);
        assert_eq!(
            lock.canonical_signatures.get("uslm_xsd@1.0.18").unwrap(),
            &sha("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
        );
    }

    #[test]
    fn parses_byte_exact_signatures_section() {
        let text = r#"
[hashes]
"license_notice@1" = "1111111111111111111111111111111111111111111111111111111111111111"

[byte_exact_signatures]
"license_notice@1" = "1111111111111111111111111111111111111111111111111111111111111111"
"#;
        let lock = parse_praxis_lock(text).unwrap();
        assert_eq!(lock.byte_exact_signatures.len(), 1);
        assert_eq!(
            lock.byte_exact_signatures.get("license_notice@1").unwrap(),
            &sha("1111111111111111111111111111111111111111111111111111111111111111")
        );
    }

    #[test]
    fn rejects_byte_exact_signature_without_matching_hash() {
        // A byte_exact_signature pins the byte-exact round-trip of an
        // already-hashed raw source; free-standing entries are an error.
        let text = r#"
[hashes]

[byte_exact_signatures]
"license_notice@1" = "1111111111111111111111111111111111111111111111111111111111111111"
"#;
        let err = parse_praxis_lock(text).unwrap_err();
        assert!(
            err.contains("has no matching entry in `[hashes]`"),
            "got: {err}"
        );
    }

    #[test]
    fn rejects_byte_exact_signature_not_equal_to_raw_hash() {
        // Byte-exactness means put(get(b)) == b, so the round-trip hash
        // IS the raw-bytes hash. A byte_exact_signature that differs from
        // the [hashes] pin is a contradiction in terms.
        let text = r#"
[hashes]
"license_notice@1" = "1111111111111111111111111111111111111111111111111111111111111111"

[byte_exact_signatures]
"license_notice@1" = "2222222222222222222222222222222222222222222222222222222222222222"
"#;
        let err = parse_praxis_lock(text).unwrap_err();
        assert!(
            err.contains("must equal the raw `[hashes]` pin"),
            "got: {err}"
        );
    }

    #[test]
    fn parses_archive_signatures_section() {
        // The archive signature pins the compiled `.prx` envelope's content
        // address — a DIFFERENT artifact than the raw source, so its value
        // legitimately differs from the `[hashes]` pin (here `1111…` source,
        // `2222…` archive). This is the case `[byte_exact_signatures]`
        // forbids and `[archive_signatures]` must allow.
        let text = r#"
[hashes]
"cito@2.8.1" = "1111111111111111111111111111111111111111111111111111111111111111"

[archive_signatures]
"cito@2.8.1" = "2222222222222222222222222222222222222222222222222222222222222222"
"#;
        let lock = parse_praxis_lock(text).unwrap();
        assert_eq!(lock.archive_signatures.len(), 1);
        assert_eq!(
            lock.archive_signatures.get("cito@2.8.1").unwrap(),
            &sha("2222222222222222222222222222222222222222222222222222222222222222")
        );
    }

    #[test]
    fn rejects_archive_signature_without_matching_hash() {
        // An archive signature pins the compiled `.prx` of an already-hashed
        // source; a free-standing entry has no source to be the archive of.
        let text = r#"
[hashes]

[archive_signatures]
"cito@2.8.1" = "2222222222222222222222222222222222222222222222222222222222222222"
"#;
        let err = parse_praxis_lock(text).unwrap_err();
        assert!(
            err.contains("has no matching entry in `[hashes]`"),
            "got: {err}"
        );
    }

    #[test]
    fn rejects_malformed_archive_signature() {
        let text = r#"
[hashes]
"x@1" = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"

[archive_signatures]
"x@1" = "abc123"
"#;
        let err = parse_praxis_lock(text).unwrap_err();
        assert!(
            err.contains("not a 64-char lowercase hex digest"),
            "got: {err}"
        );
    }

    #[test]
    fn accepts_snapshot_signature_without_hashes_entry() {
        // DELIBERATE ASYMMETRY vs [archive_signatures]: a snapshot is keyed by
        // its synthetic GraphVersion (its own content address) and has NO
        // upstream fetched source, so it parses with an EMPTY [hashes] section.
        let text = r#"
[hashes]

[snapshot_signatures]
"graph-v1" = "3333333333333333333333333333333333333333333333333333333333333333"
"#;
        let lock = parse_praxis_lock(text).unwrap();
        assert_eq!(lock.snapshot_signatures.len(), 1);
        assert_eq!(
            lock.snapshot_signatures.get("graph-v1").unwrap(),
            &sha("3333333333333333333333333333333333333333333333333333333333333333")
        );
    }

    #[test]
    fn rejects_snapshot_signature_bad_hex() {
        // "not-a-hash" carries no `:`, so it parses as a bare legacy digest
        // and fails the 64-hex shape check.
        let text = r#"
[snapshot_signatures]
"graph-v1" = "not-a-hash"
"#;
        let err = parse_praxis_lock(text).unwrap_err();
        assert!(
            err.contains("not a 64-char lowercase hex digest"),
            "got: {err}"
        );
    }

    #[test]
    fn rejects_canonical_signature_without_matching_hash() {
        // A canonical_signature pins the lens output of an
        // already-hashed raw source. Free-standing signatures are
        // a configuration error.
        let text = r#"
[hashes]

[canonical_signatures]
"uslm_xsd@1.0.18" = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
"#;
        let err = parse_praxis_lock(text).unwrap_err();
        assert!(
            err.contains("has no matching entry in `[hashes]`"),
            "got: {err}"
        );
    }

    #[test]
    fn rejects_malformed_canonical_signature() {
        // Not 64 chars.
        let text = r#"
[hashes]
"x@1" = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"

[canonical_signatures]
"x@1" = "abc123"
"#;
        let err = parse_praxis_lock(text).unwrap_err();
        assert!(
            err.contains("not a 64-char lowercase hex digest"),
            "got: {err}"
        );

        // Uppercase hex.
        let text = r#"
[hashes]
"x@1" = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"

[canonical_signatures]
"x@1" = "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF"
"#;
        let err = parse_praxis_lock(text).unwrap_err();
        assert!(
            err.contains("not a 64-char lowercase hex digest"),
            "got: {err}"
        );
    }

    #[test]
    fn is_lowercase_hex_digest_works() {
        // 64-char lowercase hex.
        assert!(is_lowercase_hex_digest(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        ));
        // Wrong length.
        assert!(!is_lowercase_hex_digest("abc"));
        assert!(!is_lowercase_hex_digest(&"a".repeat(63)));
        assert!(!is_lowercase_hex_digest(&"a".repeat(65)));
        // Uppercase.
        assert!(!is_lowercase_hex_digest(&"A".repeat(64)));
        // Non-hex character.
        let mut s = "a".repeat(63);
        s.push('g');
        assert!(!is_lowercase_hex_digest(&s));
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
                local_path: None,
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
                local_path: None,
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
                local_path: None,
            },
        };
        let mut lock = HashMap::new();
        lock.insert(
            "english_wordnet@2025".into(),
            LockDigest::address("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"),
        );
        let entry = build_entry(item, &lock).unwrap();
        assert_eq!(entry.name, "english_wordnet");
        assert_eq!(entry.version, "2025");
        assert_eq!(entry.kind, SourceTaxonomyConcept::Language);
        // Lexicon-family: gets BOTH an XML-attribute claim AND a hash claim.
        assert_eq!(entry.identity.0.len(), 2);
    }
}
