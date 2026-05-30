//! `.prx.gz` — the self-describing, load-validated distribution envelope
//! for a loaded OWL vocabulary.
//!
//! This is the OWL leaf of praxis's M4.ι archival path: a registered OWL
//! source is parsed from its authoritative RDF/XML exactly once (the
//! [`read_owl`] reader), projected through the established codegen
//! interchange (`owl_to_builder` → [`CodegenData`]), and frozen into a
//! content-addressed binary blob the runtime materializes back into a
//! [`LoadedOwlVocabulary`] without re-parsing the XML. Same shape the
//! WordNet → English and USLM → UsCode sources use; no new reader, no new
//! IR, no second parser.
//!
//! ## The two layers, bottom-up
//!
//! ```text
//! read_owl(source)                     authoritative RDF/XML  (one reader)
//!   └► owl_to_builder ─► CodegenData    the M4.ε interchange shape
//!        └► OwnedCodegenData            rkyv-serializable owned mirror
//!             └► PrxEnvelope            + OMV/PROV-O metadata block
//!                  └► rkyv bytes        deterministic, bytecheck-validated
//!                       └► gzip         RFC 1952 wrapper ─► `.prx.gz`
//! ```
//!
//! [`OwnedCodegenData`] is the owned, serializable mirror of the build-
//! time→runtime [`CodegenData<P>`] interchange (whose fields are
//! `&'static` slices). It carries the same data with `String`/`Vec`
//! ownership so rkyv can serialize it, and rebuilds a typed
//! `CodegenData<P>` view by leaking its owned columns to `'static`
//! (process-lifetime, identical in effect to a build-emitted `static`,
//! the same trade [`LoadedOwlVocabulary`]'s test helper and the USC
//! corpus loader make).
//!
//! [`PrxEnvelope`] wraps that mirror with an **ontologically grounded**
//! metadata block: every metadata field is typed/documented by the OMV
//! (Ontology Metadata Vocabulary) or PROV-O (PROV Ontology) term it
//! realizes — not ad-hoc strings. See [`PrxMetadata`].
//!
//! ## The load-validation gate
//!
//! [`load_prx_gz`] is fail-closed. It gunzips, rkyv-validates the
//! envelope (bytecheck rejects a corrupted/truncated blob), then asserts
//! the envelope's embedded `source_sha256` equals the pin recorded in
//! `praxis.lock` for `"{name}@{version}"` (read through the registry's
//! [`lock_hashes`] accessor — the same hash space the
//! `LockManifestAgreement` / `RegistryLocalPathsExist` axioms verify). On
//! any mismatch it returns `Err` and installs nothing. Only on a match
//! does it rebuild the `CodegenData` view and hand back a validated
//! [`LoadedOwlVocabulary`].
//!
//! ## Bidirectional-transformation law
//!
//! `emit`/`load` form a well-behaved lens between bytes and the loaded
//! vocabulary (Foster, Greenwald, Moore, Pierce & Schmitt 2007,
//! "Combinators for Bidirectional Tree Transformations", *ACM TOPLAS*
//! 29(3) §2.2). The rkyv put is deterministic — equal envelopes
//! serialize to equal bytes — so the blob's SHA-256 is a stable content
//! address; gzip (RFC 1952) round-trips losslessly, so
//! `gunzip(gzip(x)) == x`; together the GetPut round-trip holds up to the
//! materialized vocabulary value.
//!
//! ## Distribution model (commit vs in-memory)
//!
//! Following the M4.ι decision that dropped committed `.rkyv` blobs in
//! favour of loading authoritative XML directly, **no `.prx.gz` file is
//! committed to the tree**. rkyv's wire layout is determined by the
//! `rkyv` version and target, so a committed blob would be a
//! cross-toolchain liability; the canonical source of truth stays the
//! bundled `.owl` plus its `praxis.lock` pin. The `emit`→`load`
//! round-trip is exercised in-memory by the tests here; producing a
//! published `.prx.gz` artifact (and serving it) is the distribution
//! layer's job (#256), and the wasm dual-load is #257.
//!
//! ## Deferred (flagged, not built here)
//!
//! - A full **queryable runtime OMV/PROV-O instance-graph** — this brick
//!   grounds the metadata *schema* in OMV+PROV-O via typed, cited fields;
//!   materializing those fields into a navigable PROV/OMV `Category` of
//!   instances is follow-on.
//! - `write_owl` + RDF canonicalization to regenerate the *source* bytes
//!   (the full [`WellBehavedLens`] PutGet leg) — #258.
//! - wasm dual-load UI/fetch — #257.
//!
//! ## Citations
//!
//! - **Hartmann, Palma & Sure (2005)** "OMV — Ontology Metadata
//!   Vocabulary", *ISWC 2005 Workshop on Ontology Patterns*. OMV
//!   namespace `http://omv.ontoware.org/2005/05/ontology#`.
//! - **Lebo, Sahoo & McGuinness (eds.) (2013)** *PROV-O: The PROV
//!   Ontology*, W3C Recommendation 2013-04-30. PROV namespace
//!   `http://www.w3.org/ns/prov#`. <https://www.w3.org/TR/prov-o/>.
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators
//!   for Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2.
//! - **Deutsch, P. (1996)** *GZIP file format specification version
//!   4.3*, RFC 1952. <https://www.rfc-editor.org/rfc/rfc1952>.
//! - **NIST (2015)** *Secure Hash Standard (SHS)*, FIPS PUB 180-4 §6.2
//!   (SHA-256). The integrity hash space.
//! - **Dolstra, E. (2006)** *The Purely Functional Software Deployment
//!   Model*, PhD thesis — content-addressing by cryptographic hash.
//! - **Hill, D.** *rkyv: zero-copy deserialization framework for Rust*,
//!   v0.8, <https://github.com/rkyv/rkyv>.
//!
//! [`read_owl`]: super::reader::read_owl
//! [`CodegenData`]: pr4xis::codegen_data::CodegenData
//! [`CodegenData<P>`]: pr4xis::codegen_data::CodegenData
//! [`LoadedOwlVocabulary`]: super::vocabulary::LoadedOwlVocabulary
//! [`WellBehavedLens`]: crate::formal::meta::well_behaved_lens
//! [`lock_hashes`]: crate::applied::data_provisioning::registry::lock_hashes

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use std::io::{Read, Write};

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use sha2::{Digest, Sha256};

use pr4xis::EntityRef;
use pr4xis::codegen_data::CodegenData;

use super::vocabulary::LoadedOwlVocabulary;

// =============================================================================
// OMV / PROV-O term IRIs — the vocabulary the metadata block is typed by.
// =============================================================================

/// The OMV namespace (Hartmann, Palma & Sure 2005). Each OMV term IRI is
/// this base + the term's local name.
pub const OMV_NS: &str = "http://omv.ontoware.org/2005/05/ontology#";
/// The PROV-O namespace (Lebo, Sahoo & McGuinness 2013).
pub const PROV_NS: &str = "http://www.w3.org/ns/prov#";

/// `omv:version` — the version string an ontology self-declares
/// (Hartmann 2005 OMV core). Realized by [`PrxMetadata::version`].
pub const OMV_VERSION: &str = "http://omv.ontoware.org/2005/05/ontology#version";
/// `omv:URI` — the canonical IRI under which the ontology is published
/// (Hartmann 2005 OMV core). Realized by [`PrxMetadata::ontology_uri`].
pub const OMV_URI: &str = "http://omv.ontoware.org/2005/05/ontology#URI";
/// `omv:name` — the short symbolic name of the ontology (Hartmann 2005).
/// Realized by [`PrxMetadata::name`].
pub const OMV_NAME: &str = "http://omv.ontoware.org/2005/05/ontology#name";
/// `omv:numberOfClasses` — structural metric (Hartmann 2005 OMV
/// `omv:OntologyMetrics`; MOD 2.0 Analytics). Realized by
/// [`PrxMetadata::number_of_classes`].
pub const OMV_NUMBER_OF_CLASSES: &str = "http://omv.ontoware.org/2005/05/ontology#numberOfClasses";
/// `omv:numberOfProperties` — structural metric (Hartmann 2005). Realized
/// by [`PrxMetadata::number_of_properties`].
pub const OMV_NUMBER_OF_PROPERTIES: &str =
    "http://omv.ontoware.org/2005/05/ontology#numberOfProperties";

/// `prov:wasDerivedFrom` — "a transformation of an entity into another"
/// (Lebo 2013 §3, PROV-O). The `.prx.gz` envelope is
/// `prov:wasDerivedFrom` the source RDF/XML; the value pinned is that
/// source entity's content address. Realized by
/// [`PrxMetadata::source_url`] (the derivation target) together with
/// [`PrxMetadata::source_sha256`] (its content hash).
pub const PROV_WAS_DERIVED_FROM: &str = "http://www.w3.org/ns/prov#wasDerivedFrom";
/// `prov:Entity` — "a physical, digital, conceptual, or other kind of
/// thing" (Lebo 2013 §2). The source RDF/XML bytes are the
/// `prov:Entity` this envelope is derived from; its identity is the
/// SHA-256 content hash. Realized by [`PrxMetadata::source_sha256`].
pub const PROV_ENTITY: &str = "http://www.w3.org/ns/prov#Entity";
/// `prov:atLocation` — "the Location of any resource" (Lebo 2013 §3.2).
/// The URL the source entity is published at. Realized by
/// [`PrxMetadata::source_url`].
pub const PROV_AT_LOCATION: &str = "http://www.w3.org/ns/prov#atLocation";

// =============================================================================
// OwnedCodegenData — the rkyv-serializable owned mirror of CodegenData<P>.
// =============================================================================

/// Owned, serializable mirror of [`CodegenData`].
///
/// Field-for-field identical to `CodegenData<P>` except that every
/// `&'static str` becomes an owned [`String`] and every typed
/// [`pr4xis::EntityRef`] becomes its raw `u64` handle (the phantom marker
/// `P` is reconstructed when a typed [`CodegenData`] view is rebuilt — the
/// integer handle is identical machine data either way). This is the
/// `prov:Entity` the envelope archives.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct OwnedCodegenData {
    pub entity_count: u64,
    pub entity_ids: Vec<String>,
    pub entity_kind: Vec<String>,
    pub entity_labels: Vec<String>,
    pub entity_defs: Vec<String>,
    /// `(word, concept-handles)` — mirrors `CodegenData::word_index`.
    pub word_index: Vec<(String, Vec<u64>)>,
    /// `(child, parent)` subsumption edges.
    pub taxonomy: Vec<(u64, u64)>,
    /// `(whole, part)` mereology edges.
    pub mereology: Vec<(u64, u64)>,
    pub opposition: Vec<(u64, u64)>,
    pub equivalence: Vec<(u64, u64)>,
    pub causation: Vec<(u64, u64)>,
    pub references: Vec<(u64, u64)>,
}

impl OwnedCodegenData {
    /// Project a typed build-time [`CodegenData`] into the owned archival
    /// shape. The forgetful direction of the embed/forget adjunction: the
    /// phantom marker `P` is dropped (handles become raw integers).
    pub fn from_codegen_data<P: 'static>(data: &CodegenData<P>) -> Self {
        let edges = |s: &[(EntityRef<P>, EntityRef<P>)]| -> Vec<(u64, u64)> {
            s.iter().map(|(a, b)| (a.value(), b.value())).collect()
        };
        Self {
            entity_count: data.entity_count as u64,
            entity_ids: data.entity_ids.iter().map(|s| s.to_string()).collect(),
            entity_kind: data.entity_kind.iter().map(|s| s.to_string()).collect(),
            entity_labels: data.entity_labels.iter().map(|s| s.to_string()).collect(),
            entity_defs: data.entity_defs.iter().map(|s| s.to_string()).collect(),
            word_index: data
                .word_index
                .iter()
                .map(|(w, refs)| (w.to_string(), refs.iter().map(|r| r.value()).collect()))
                .collect(),
            taxonomy: edges(data.taxonomy),
            mereology: edges(data.mereology),
            opposition: edges(data.opposition),
            equivalence: edges(data.equivalence),
            causation: edges(data.causation),
            references: edges(data.references),
        }
    }

    /// Rebuild a typed [`CodegenData`] view from this owned data.
    ///
    /// The *re-embed* direction: raw `u64` handles are re-tagged with the
    /// phantom marker `P` chosen by the caller, and the owned strings /
    /// edge tables are promoted to the `&'static` lifetimes that
    /// [`CodegenData`] requires by [`Box::leak`]. The leaks persist for
    /// process lifetime — identical in effect to a build-time-emitted
    /// `static`, the same trade the on-disk USLM loader and the
    /// `OnceLock`-cached corpus singleton make.
    pub fn to_codegen_data_leaked<P: 'static>(&self) -> CodegenData<P> {
        fn leak_str(s: &str) -> &'static str {
            Box::leak(s.to_string().into_boxed_str())
        }
        fn leak_strs(v: &[String]) -> &'static [&'static str] {
            let leaked: Vec<&'static str> = v.iter().map(|s| leak_str(s)).collect();
            Box::leak(leaked.into_boxed_slice())
        }
        fn leak_edges<P: 'static>(v: &[(u64, u64)]) -> &'static [(EntityRef<P>, EntityRef<P>)] {
            let leaked: Vec<(EntityRef<P>, EntityRef<P>)> = v
                .iter()
                .map(|(a, b)| (EntityRef::new(*a), EntityRef::new(*b)))
                .collect();
            Box::leak(leaked.into_boxed_slice())
        }

        let word_index: Vec<(&'static str, &'static [EntityRef<P>])> = self
            .word_index
            .iter()
            .map(|(w, refs)| {
                let r: Vec<EntityRef<P>> = refs.iter().map(|x| EntityRef::new(*x)).collect();
                let r: &'static [EntityRef<P>] = Box::leak(r.into_boxed_slice());
                (leak_str(w), r)
            })
            .collect();

        CodegenData {
            entity_count: self.entity_count as usize,
            entity_ids: leak_strs(&self.entity_ids),
            entity_kind: leak_strs(&self.entity_kind),
            entity_labels: leak_strs(&self.entity_labels),
            entity_defs: leak_strs(&self.entity_defs),
            word_index: Box::leak(word_index.into_boxed_slice()),
            taxonomy: leak_edges(&self.taxonomy),
            mereology: leak_edges(&self.mereology),
            opposition: leak_edges(&self.opposition),
            equivalence: leak_edges(&self.equivalence),
            causation: leak_edges(&self.causation),
            references: leak_edges(&self.references),
        }
    }
}

// =============================================================================
// PrxMetadata — the OMV/PROV-O-grounded metadata block.
// =============================================================================

/// Self-describing metadata carried by a [`PrxEnvelope`].
///
/// Every field is **ontologically grounded**: its doc names the OMV
/// (Hartmann, Palma & Sure 2005) or PROV-O (Lebo, Sahoo & McGuinness
/// 2013) term it realizes, so the metadata schema is a typed projection of
/// those published vocabularies rather than invented fields. The
/// constants [`OMV_VERSION`], [`PROV_WAS_DERIVED_FROM`], … hold the term
/// IRIs.
///
/// This brick grounds the metadata *schema*; a queryable runtime
/// OMV/PROV-O instance-graph over these fields is deferred (see module
/// docs).
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct PrxMetadata {
    /// `omv:name` (Hartmann 2005) — the registry name of the source, e.g.
    /// `"cito"`. Combined with [`Self::version`] this is the
    /// `"{name}@{version}"` key under which the source's content hash is
    /// pinned in `praxis.lock` and looked up at load time.
    pub name: String,
    /// `omv:version` (Hartmann 2005) — the source's self-declared version,
    /// e.g. `"2.8.1"` (the `owl:versionInfo` on the `owl:Ontology` node).
    pub version: String,
    /// `omv:URI` (Hartmann 2005) — the canonical IRI of the loaded
    /// ontology (`OwlOntology::iri`), e.g.
    /// `"http://purl.org/spar/cito/"`.
    pub ontology_uri: String,
    /// `prov:atLocation` (Lebo 2013 §3.2) of the `prov:Entity` this
    /// envelope is `prov:wasDerivedFrom` — the URL the source RDF/XML is
    /// published at (the registry `url`). Pairs with
    /// [`Self::source_sha256`] to identify the derivation source.
    pub source_url: String,
    /// The content address of the source `prov:Entity`
    /// ([`PROV_WAS_DERIVED_FROM`] / [`PROV_ENTITY`], Lebo 2013) — the
    /// SHA-256 (NIST FIPS 180-4 §6.2; content-addressing per Dolstra 2006)
    /// of the exact source bytes [`read_owl`] consumed, as 64-char
    /// lowercase hex. **The load-validation gate compares this against the
    /// `praxis.lock` pin**; a mismatch fails closed.
    ///
    /// [`read_owl`]: super::reader::read_owl
    pub source_sha256: String,
    /// `omv:numberOfClasses` (Hartmann 2005 `omv:OntologyMetrics`; MOD 2.0
    /// Analytics) — count of `owl:Class` entities in the archived corpus.
    /// A structural metric, redundant with the archived data and used to
    /// self-describe the envelope without materializing it.
    pub number_of_classes: u64,
    /// `omv:numberOfProperties` (Hartmann 2005) — count of
    /// `owl:ObjectProperty` entities in the archived corpus.
    pub number_of_properties: u64,
}

/// Compute the SHA-256 (NIST FIPS 180-4 §6.2) of the source bytes, as
/// 64-char lowercase hex — the content address recorded as
/// [`PrxMetadata::source_sha256`] and pinned in `praxis.lock`.
pub fn source_content_hash(source_bytes: &[u8]) -> String {
    let digest = Sha256::digest(source_bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

// =============================================================================
// PrxEnvelope — owned data + metadata, the thing that gets rkyv'd + gzip'd.
// =============================================================================

/// The rkyv-serializable `.prx` envelope: the archived corpus plus its
/// OMV/PROV-O-grounded metadata. Serialized to rkyv bytes and gzip-wrapped
/// to form the `.prx.gz` artifact.
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct PrxEnvelope {
    /// OMV/PROV-O-grounded self-description, incl. the source content
    /// hash the load gate validates.
    pub metadata: PrxMetadata,
    /// The archived corpus — the owned mirror of the
    /// `CodegenData<LoadedOwlVocabulary>` interchange.
    pub data: OwnedCodegenData,
}

/// Error from emitting or loading a `.prx.gz` envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrxError {
    /// The RDF/XML source could not be read by [`read_owl`].
    ///
    /// [`read_owl`]: super::reader::read_owl
    Read(String),
    /// rkyv serialization or (bytecheck-validated) deserialization failed
    /// — e.g. a corrupted or truncated blob.
    Rkyv(String),
    /// gzip (RFC 1952) compression or decompression failed.
    Gzip(String),
    /// The load-validation gate rejected the envelope: its embedded
    /// `source_sha256` did not match the `praxis.lock` pin for
    /// `"{name}@{version}"`. Fail-closed — nothing is installed.
    HashMismatch {
        key: String,
        expected: String,
        found: String,
    },
    /// `praxis.lock` carries no pin for the envelope's `"{name}@{version}"`
    /// — an unregistered or unpinned source cannot be validated, so the
    /// gate rejects it (fail-closed).
    NoLockPin { key: String },
}

impl core::fmt::Display for PrxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PrxError::Read(m) => write!(f, "OWL read error: {m}"),
            PrxError::Rkyv(m) => write!(f, "rkyv archive error: {m}"),
            PrxError::Gzip(m) => write!(f, "gzip error: {m}"),
            PrxError::HashMismatch {
                key,
                expected,
                found,
            } => write!(
                f,
                "source hash mismatch for `{key}`: praxis.lock pins {expected}, \
                 envelope carries {found} — refusing to install"
            ),
            PrxError::NoLockPin { key } => write!(
                f,
                "no praxis.lock pin for `{key}` — cannot validate, refusing to install"
            ),
        }
    }
}

impl std::error::Error for PrxError {}

// =============================================================================
// gzip layer (RFC 1952).
// =============================================================================

/// Compress bytes with gzip (Deutsch 1996, RFC 1952). The `.prx.gz` *put*
/// half of the gzip lens.
pub fn gzip(bytes: &[u8]) -> Result<Vec<u8>, PrxError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(bytes)
        .map_err(|e| PrxError::Gzip(e.to_string()))?;
    encoder.finish().map_err(|e| PrxError::Gzip(e.to_string()))
}

/// Decompress gzip bytes (Deutsch 1996, RFC 1952). The `.prx.gz` *get*
/// half; `gunzip(gzip(x)) == x`.
pub fn gunzip(bytes: &[u8]) -> Result<Vec<u8>, PrxError> {
    let mut decoder = GzDecoder::new(bytes);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| PrxError::Gzip(e.to_string()))?;
    Ok(out)
}

// =============================================================================
// rkyv layer — envelope ⇄ bytes (the bidirectional lens, bytecheck-validated).
// =============================================================================

/// Serialize an envelope to rkyv bytes (the lens *put*). Deterministic:
/// equal envelopes yield equal bytes, so the blob's hash is a stable
/// content address.
pub fn envelope_to_bytes(envelope: &PrxEnvelope) -> Result<Vec<u8>, PrxError> {
    rkyv::to_bytes::<rkyv::rancor::Error>(envelope)
        .map(|v| v.to_vec())
        .map_err(|e| PrxError::Rkyv(e.to_string()))
}

/// Materialize an envelope from rkyv bytes (the lens *get*). Copies into
/// an aligned buffer first (a fetched/decompressed `Vec<u8>` carries no
/// alignment guarantee), then `bytecheck`-validates before materializing,
/// so a corrupted blob fails closed rather than producing unsound
/// references.
pub fn envelope_from_bytes(bytes: &[u8]) -> Result<PrxEnvelope, PrxError> {
    let mut aligned = rkyv::util::AlignedVec::<16>::new();
    aligned.extend_from_slice(bytes);
    rkyv::from_bytes::<PrxEnvelope, rkyv::rancor::Error>(&aligned)
        .map_err(|e| PrxError::Rkyv(e.to_string()))
}

// =============================================================================
// The load-validation gate.
// =============================================================================

/// Validate an envelope's source content hash against a pin, fail-closed.
///
/// Returns `Ok(())` only when the envelope's
/// [`PrxMetadata::source_sha256`] equals `lock_pin`. This is the
/// load-bearing gate: the caller installs the materialized vocabulary
/// only on `Ok`.
fn validate_envelope_against_pin(envelope: &PrxEnvelope, lock_pin: &str) -> Result<(), PrxError> {
    let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
    if envelope.metadata.source_sha256 == lock_pin {
        Ok(())
    } else {
        Err(PrxError::HashMismatch {
            key,
            expected: lock_pin.to_string(),
            found: envelope.metadata.source_sha256.clone(),
        })
    }
}

/// Load a `.prx.gz` blob into a validated [`LoadedOwlVocabulary`].
///
/// The full load path and validation gate:
/// 1. gunzip (RFC 1952);
/// 2. rkyv-validate the envelope (bytecheck — corrupted/truncated blobs
///    fail closed);
/// 3. assert `envelope.metadata.source_sha256 == lock_pin` (the
///    `praxis.lock` value for `"{name}@{version}"`) — on mismatch return
///    `Err`, install nothing;
/// 4. rebuild the `CodegenData` view and apply
///    [`LoadedOwlVocabulary::from_codegen`].
///
/// `lock_pin` is the pinned hash; callers reach it through the registry's
/// [`lock_hashes`] accessor — see [`load_prx_gz_from_lock`], which does
/// the lookup. Splitting the pin out keeps this function pure and unit-
/// testable without a `praxis.lock` round-trip.
///
/// [`lock_hashes`]: crate::applied::data_provisioning::registry::lock_hashes
pub fn load_prx_gz(prx_gz: &[u8], lock_pin: &str) -> Result<LoadedOwlVocabulary, PrxError> {
    let rkyv_bytes = gunzip(prx_gz)?;
    let envelope = envelope_from_bytes(&rkyv_bytes)?;
    validate_envelope_against_pin(&envelope, lock_pin)?;
    let data: CodegenData<LoadedOwlVocabulary> = envelope.data.to_codegen_data_leaked();
    Ok(LoadedOwlVocabulary::from_codegen(&data))
}

/// Load a `.prx.gz` blob, reaching the pin through the live registry
/// (`praxis.lock`'s `[hashes]` for `"{name}@{version}"`).
///
/// First peeks the envelope to read `name`/`version`, looks up the pin via
/// [`lock_hashes`], then validates and materializes. Fail-closed if no pin
/// is registered for the source.
///
/// [`lock_hashes`]: crate::applied::data_provisioning::registry::lock_hashes
pub fn load_prx_gz_from_lock(prx_gz: &[u8]) -> Result<LoadedOwlVocabulary, PrxError> {
    use crate::applied::data_provisioning::registry::lock_hashes;
    let rkyv_bytes = gunzip(prx_gz)?;
    let envelope = envelope_from_bytes(&rkyv_bytes)?;
    let key = format!("{}@{}", envelope.metadata.name, envelope.metadata.version);
    let pin = lock_hashes()
        .get(&key)
        .ok_or_else(|| PrxError::NoLockPin { key: key.clone() })?;
    validate_envelope_against_pin(&envelope, pin)?;
    let data: CodegenData<LoadedOwlVocabulary> = envelope.data.to_codegen_data_leaked();
    Ok(LoadedOwlVocabulary::from_codegen(&data))
}

// =============================================================================
// EmittedArtifact — one published `.prx.gz` file, round-trip-validated.
// =============================================================================

/// A `.prx.gz` artifact `emit_all_prx_gz` wrote to disk and then
/// round-trip-validated by loading it back through the fail-closed gate.
///
/// The presence of this value is itself a proof obligation discharged: the
/// emitter returns it only after [`load_prx_gz_from_lock`] re-loaded the
/// written file and the embedded `source_sha256` matched the `praxis.lock`
/// pin. A published artifact therefore is guaranteed loadable AND
/// content-anchored (NIST FIPS 180-4 §6.2; Dolstra 2006 content-addressing)
/// before it is handed to the distribution layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedArtifact {
    /// `omv:name` (Hartmann 2005) — the registry source name, e.g. `"cito"`.
    pub name: String,
    /// `omv:version` (Hartmann 2005) — the source version, e.g. `"2.8.1"`.
    pub version: String,
    /// Absolute path of the written `<name>-<version>.prx.gz` file.
    pub path: std::path::PathBuf,
    /// Size of the written `.prx.gz` in bytes.
    pub byte_len: u64,
}

// =============================================================================
// Emit — read_owl → builder → CodegenData → envelope → rkyv → gzip.
// =============================================================================
//
// Gated on `any(test, feature = "codegen")` because building the envelope
// from OWL source needs `owl_to_builder` + `pr4xis::codegen::OntologyBuilder`,
// which are only present under the `codegen` feature (the WASM-facing
// default dep keeps `pr4xis/codegen` — and its `xsd-parser` transitive —
// out). The load path above needs only `fetch` (rkyv + flate2).

#[cfg(any(test, feature = "codegen"))]
mod emit {
    use super::*;
    use crate::social::software::markup::xml::owl::owl_vocabulary::owl_to_builder;
    use crate::social::software::markup::xml::owl::reader::read_owl;
    use pr4xis::codegen::OntologyBuilder;
    use std::collections::HashMap;

    /// Turn an [`OntologyBuilder`] into an owned [`CodegenData`] with the
    /// id→index resolution + dangling-edge drop the build-time emitter
    /// performs (mirrors `pr4xis::codegen::generate::write_raw_relations`
    /// and the `LoadedOwlVocabulary` test helper). Returns the owned mirror
    /// directly — no intermediate `&'static` leak, since [`OwnedCodegenData`]
    /// already owns its columns.
    fn builder_to_owned(builder: &OntologyBuilder) -> OwnedCodegenData {
        let id_to_idx: HashMap<&str, u64> = builder
            .entities
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.as_str(), i as u64))
            .collect();

        let edges = |raw: &[(String, String)]| -> Vec<(u64, u64)> {
            raw.iter()
                .filter_map(|(c, p)| {
                    Some((*id_to_idx.get(c.as_str())?, *id_to_idx.get(p.as_str())?))
                })
                .collect()
        };

        OwnedCodegenData {
            entity_count: builder.entities.len() as u64,
            entity_ids: builder.entities.iter().map(|e| e.id.clone()).collect(),
            entity_kind: builder
                .entities
                .iter()
                .map(|e| e.pos.clone().unwrap_or_default())
                .collect(),
            entity_labels: builder.entities.iter().map(|e| e.label.clone()).collect(),
            entity_defs: builder
                .entities
                .iter()
                .map(|e| e.definitions.first().cloned().unwrap_or_default())
                .collect(),
            word_index: Vec::new(),
            taxonomy: edges(&builder.taxonomy),
            mereology: Vec::new(),
            opposition: Vec::new(),
            equivalence: Vec::new(),
            causation: Vec::new(),
            references: Vec::new(),
        }
    }

    /// Build a [`PrxEnvelope`] from OWL source bytes plus its registry
    /// `(name, version, url)`.
    ///
    /// `read_owl(source) → owl_to_builder → OwnedCodegenData`, then attach
    /// the OMV/PROV-O metadata block: `source_sha256` is the SHA-256 of the
    /// exact `source` bytes (the `prov:Entity` content address);
    /// `number_of_classes` / `number_of_properties` are the
    /// `omv:numberOf*` structural metrics counted from the builder's
    /// entity kinds.
    pub fn build_envelope(
        source: &[u8],
        name: &str,
        version: &str,
        url: &str,
    ) -> Result<PrxEnvelope, PrxError> {
        let text = core::str::from_utf8(source)
            .map_err(|e| PrxError::Read(format!("source is not UTF-8: {e}")))?;
        let ont = read_owl(text).map_err(|e| PrxError::Read(format!("{e}")))?;
        let ontology_uri = ont.iri.clone();
        let builder = owl_to_builder(&ont);

        // omv:numberOfClasses / omv:numberOfProperties — count by the
        // entity_kind tag owl_to_builder wrote (W3C OWL 2 §5).
        let number_of_classes = builder
            .entities
            .iter()
            .filter(|e| e.pos.as_deref() == Some("Class"))
            .count() as u64;
        let number_of_properties = builder
            .entities
            .iter()
            .filter(|e| e.pos.as_deref() == Some("ObjectProperty"))
            .count() as u64;

        let data = builder_to_owned(&builder);
        let metadata = PrxMetadata {
            name: name.to_string(),
            version: version.to_string(),
            ontology_uri,
            source_url: url.to_string(),
            source_sha256: source_content_hash(source),
            number_of_classes,
            number_of_properties,
        };
        Ok(PrxEnvelope { metadata, data })
    }

    /// Emit a `.prx.gz` artifact from OWL source bytes:
    /// `build_envelope → envelope_to_bytes (rkyv) → gzip`.
    pub fn emit_prx_gz(
        source: &[u8],
        name: &str,
        version: &str,
        url: &str,
    ) -> Result<Vec<u8>, PrxError> {
        let envelope = build_envelope(source, name, version, url)?;
        let rkyv_bytes = envelope_to_bytes(&envelope)?;
        gzip(&rkyv_bytes)
    }

    /// Workspace root — the grandparent of `CARGO_MANIFEST_DIR`
    /// (`crates/domains/`). `RegistryEntry::local_path()` is
    /// workspace-relative (`crates/domains/data/...`), so the bundled `.owl`
    /// resolves against this root. Mirrors
    /// `owl::loaded_vocabularies::workspace_root` and the USC corpus loader.
    fn workspace_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }

    /// Emit a `.prx.gz` artifact for **every** registered
    /// [`OntologyVocabulary`][ov] source into `out_dir`, round-trip-validating
    /// each before returning it.
    ///
    /// The distribution-layer entry point for #256. It is registry-driven
    /// (never a hardcoded source set): it walks every entry
    /// [`data_sources`][ds] reports whose `kind` is
    /// [`SourceTaxonomyConcept::OntologyVocabulary`][ov], reads the bundled
    /// RDF/XML from `workspace_root.join(entry.local_path())`, and for each:
    ///
    /// 1. `emit_prx_gz(source, name, version, url)` → the gzip-wrapped rkyv
    ///    envelope;
    /// 2. writes it to `<out_dir>/<name>-<version>.prx.gz`;
    /// 3. **round-trip-validates** by reading the file back and feeding it to
    ///    [`load_prx_gz_from_lock`] — which gunzips, bytecheck-validates the
    ///    rkyv layer, and asserts the embedded `source_sha256` equals the
    ///    `praxis.lock` pin (fail-closed). A success proves the published
    ///    artifact is both *loadable* and *content-anchored* to the lock.
    ///
    /// A registered source whose `.owl` is **not on disk** is skipped
    /// gracefully — the same graceful skip `loaded_vocabularies` and the USC
    /// corpus loader make. A source that **emits but fails to round-trip-load**
    /// is a defect, not a skip: it returns `Err` (the published artifact would
    /// be un-loadable or hash-mismatched), so a broken artifact never reaches
    /// the release or Pages.
    ///
    /// `out_dir` is created if absent. Returns one [`EmittedArtifact`] per
    /// written-and-validated file, in `data_sources` order.
    ///
    /// [ov]: crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept::OntologyVocabulary
    /// [ds]: crate::applied::data_provisioning::registry::data_sources
    pub fn emit_all_prx_gz(out_dir: &std::path::Path) -> Result<Vec<EmittedArtifact>, PrxError> {
        use crate::applied::data_provisioning::registry::data_sources;
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

        std::fs::create_dir_all(out_dir)
            .map_err(|e| PrxError::Gzip(format!("create out_dir {}: {e}", out_dir.display())))?;

        let root = workspace_root();
        let mut emitted = Vec::new();
        for entry in data_sources() {
            if entry.kind != SourceTaxonomyConcept::OntologyVocabulary {
                continue;
            }
            let src_path = root.join(entry.local_path());
            let Ok(source) = std::fs::read(&src_path) else {
                // Registered but not on disk — skip gracefully, exactly as
                // `loaded_vocabularies` and the USC corpus loader skip a
                // granule that isn't bundled.
                continue;
            };

            let prx_gz = emit_prx_gz(&source, &entry.name, &entry.version, &entry.url)?;
            let path = out_dir.join(format!("{}-{}.prx.gz", entry.name, entry.version));
            std::fs::write(&path, &prx_gz)
                .map_err(|e| PrxError::Gzip(format!("write {}: {e}", path.display())))?;

            // Round-trip-validate the *written file*: read it back from disk
            // and run it through the fail-closed load gate. Success proves the
            // published artifact is loadable and its embedded source hash
            // matches the praxis.lock pin (the GetPut leg of the bytes ⇄
            // vocabulary lens, Foster et al. 2007 §2.2). An emit-but-fail-to-
            // load source is a defect — propagate the Err.
            let read_back = std::fs::read(&path)
                .map_err(|e| PrxError::Gzip(format!("read-back {}: {e}", path.display())))?;
            load_prx_gz_from_lock(&read_back)?;

            emitted.push(EmittedArtifact {
                name: entry.name.clone(),
                version: entry.version.clone(),
                path,
                byte_len: prx_gz.len() as u64,
            });
        }
        Ok(emitted)
    }
}

#[cfg(any(test, feature = "codegen"))]
pub use emit::{build_envelope, emit_all_prx_gz, emit_prx_gz};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applied::data_provisioning::registry::lock_hashes;
    use proptest::prelude::*;

    /// The bundled CiTO 2.8.1 OWL vocabulary (SPAR), embedded at build
    /// time — the same source the codegen-side tests use.
    const CITO_2_8_1_OWL: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ontologies/cito-2.8.1.owl"
    ));
    const CITO_NAME: &str = "cito";
    const CITO_VERSION: &str = "2.8.1";
    const CITO_URL: &str = "https://sparontologies.github.io/cito/current/cito.xml";
    const CITES_AS_EVIDENCE_IRI: &str = "http://purl.org/spar/cito/citesAsEvidence";
    const CITES_IRI: &str = "http://purl.org/spar/cito/cites";

    /// Materialize an [`OwnedCodegenData`] directly into a
    /// [`LoadedOwlVocabulary`] without any rkyv / gzip round-trip — the
    /// fidelity reference. `load_prx_gz` must reproduce *exactly* this
    /// from the gzip-wrapped rkyv bytes of the *same* owned data (the
    /// GetPut lens law); building the reference from the same owned value
    /// the envelope carries isolates the round-trip from `read_owl`
    /// parse-to-parse variance.
    fn materialize_direct(owned: &OwnedCodegenData) -> LoadedOwlVocabulary {
        let data: CodegenData<LoadedOwlVocabulary> = owned.to_codegen_data_leaked();
        LoadedOwlVocabulary::from_codegen(&data)
    }

    // ── source anchor ────────────────────────────────────────────────

    /// The SHA-256 of the bundled CiTO source equals the `praxis.lock`
    /// pin for `cito@2.8.1`. This is the invariant the load gate enforces;
    /// if it ever breaks, every `.prx.gz` for CiTO must be rejected.
    #[test]
    fn source_anchor_cito_hash_matches_lock() {
        let computed = source_content_hash(CITO_2_8_1_OWL.as_bytes());
        let pinned = lock_hashes()
            .get("cito@2.8.1")
            .expect("praxis.lock must pin cito@2.8.1");
        assert_eq!(
            &computed, pinned,
            "bundled CiTO source hash must equal the praxis.lock pin"
        );
    }

    // ── gzip round-trip ──────────────────────────────────────────────

    #[test]
    fn gzip_round_trips() {
        let payload = b"praxis .prx.gz round-trip \x00\x01\xff payload";
        let compressed = gzip(payload).expect("gzip");
        let restored = gunzip(&compressed).expect("gunzip");
        assert_eq!(&restored, payload, "gunzip(gzip(x)) == x");
    }

    // ── rkyv envelope round-trip + determinism ───────────────────────

    #[test]
    fn envelope_bytes_round_trip_and_deterministic() {
        let envelope = build_envelope(CITO_2_8_1_OWL.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        let a = envelope_to_bytes(&envelope).expect("serialize a");
        let b = envelope_to_bytes(&envelope).expect("serialize b");
        assert_eq!(a, b, "rkyv serialization must be deterministic");
        let back = envelope_from_bytes(&a).expect("deserialize");
        assert_eq!(envelope, back, "rkyv round-trip must be lossless");
    }

    // ── byte-reproducibility: emit twice → identical bytes (#264) ────

    /// Two independent `emit_prx_gz` runs over the *same* source MUST
    /// produce byte-identical `.prx.gz` output.
    ///
    /// This is the guarantee #264 delivers: the published artifact is
    /// reproducible. Each `emit_prx_gz` re-parses the source through
    /// `read_owl` from scratch, so the two runs share no intermediate
    /// state. `read_owl`'s `deduplicate_classes` / `deduplicate_properties`
    /// preserve first-occurrence document order (not hash-map iteration
    /// order, which is ahash-seeded per process), so the entity Vecs — and
    /// therefore the rkyv layout (deterministic put) and the gzip wrapper
    /// (RFC 1952) — are bit-for-bit stable across processes.
    #[test]
    fn emit_prx_gz_is_byte_reproducible() {
        let first = emit_prx_gz(CITO_2_8_1_OWL.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("emit first");
        let second = emit_prx_gz(CITO_2_8_1_OWL.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("emit second");
        assert_eq!(
            first, second,
            "two independent emit_prx_gz runs must yield byte-identical .prx.gz"
        );
    }

    // ── round-trip fidelity: emit(.prx.gz) → load == direct ──────────

    #[test]
    fn emit_then_load_equals_direct_corpus() {
        // The envelope built from real CiTO (read_owl → owl_to_builder →
        // owned). Its owned data is the fidelity reference.
        let envelope = build_envelope(CITO_2_8_1_OWL.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        let direct = materialize_direct(&envelope.data);

        // Now the full gzip ∘ rkyv ∘ leak ∘ from_codegen round-trip of the
        // *same* envelope. GetPut law: it must reproduce `direct` exactly.
        let prx_gz = gzip(&envelope_to_bytes(&envelope).expect("serialize")).expect("gzip");
        let pin = lock_hashes().get("cito@2.8.1").expect("pin").clone();
        let loaded = load_prx_gz(&prx_gz, &pin).expect("load + validate");

        // It came from a real CiTO parse, so it is rich and carries the
        // citesAsEvidence is_a cites edge.
        assert!(direct.entity_count() > 30, "real CiTO is rich");
        assert!(loaded.find(CITES_AS_EVIDENCE_IRI).is_some());
        assert!(
            loaded.is_a(CITES_AS_EVIDENCE_IRI, CITES_IRI),
            "citesAsEvidence is_a cites must survive the round-trip"
        );
        // The full corpus value is equal (entities + index + edges) — the
        // round-trip is lossless.
        assert_eq!(loaded, direct, "loaded corpus must equal the direct corpus");
    }

    // ── metadata grounding: OMV/PROV-O fields are populated correctly ─

    #[test]
    fn metadata_is_omv_prov_grounded() {
        let envelope = build_envelope(CITO_2_8_1_OWL.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("build envelope");
        let m = &envelope.metadata;
        // omv:name / omv:version
        assert_eq!(m.name, CITO_NAME);
        assert_eq!(m.version, CITO_VERSION);
        // prov:atLocation
        assert_eq!(m.source_url, CITO_URL);
        // prov:wasDerivedFrom content address == praxis.lock pin.
        assert_eq!(
            &m.source_sha256,
            lock_hashes().get("cito@2.8.1").expect("pin"),
        );
        // omv:URI — CiTO's ontology IRI is non-empty.
        assert!(!m.ontology_uri.is_empty(), "omv:URI must be populated");
        // omv:numberOfClasses + omv:numberOfProperties partition the
        // corpus and CiTO is property-heavy (~40 citation typing props).
        assert_eq!(
            m.number_of_classes + m.number_of_properties,
            envelope.data.entity_count,
            "class + property counts must partition the entities"
        );
        assert!(m.number_of_properties > 30, "CiTO is property-heavy");
        // The IRI constants are the published OMV/PROV-O terms.
        assert!(OMV_VERSION.starts_with(OMV_NS));
        assert!(PROV_WAS_DERIVED_FROM.starts_with(PROV_NS));
    }

    // ── load validation positive ─────────────────────────────────────

    #[test]
    fn load_validation_accepts_correct_prx_gz() {
        let prx_gz = emit_prx_gz(CITO_2_8_1_OWL.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("emit");
        // Through the live registry lock lookup (no pin passed in).
        let loaded = load_prx_gz_from_lock(&prx_gz).expect("must load + validate via lock");
        assert!(loaded.entity_count() > 30);
        assert!(loaded.find(CITES_AS_EVIDENCE_IRI).is_some());
    }

    // ── load validation negative: corrupted hash is rejected ─────────

    #[test]
    fn load_validation_rejects_tampered_source_hash() {
        // Build an envelope, then corrupt the embedded source hash so it
        // no longer matches the source bytes (and the lock pin).
        let mut envelope =
            build_envelope(CITO_2_8_1_OWL.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
                .expect("build envelope");
        envelope.metadata.source_sha256 =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let rkyv_bytes = envelope_to_bytes(&envelope).expect("serialize");
        let prx_gz = gzip(&rkyv_bytes).expect("gzip");

        // Both the explicit-pin and lock-driven loaders must fail closed.
        let pin = lock_hashes().get("cito@2.8.1").expect("pin").clone();
        let err = load_prx_gz(&prx_gz, &pin).expect_err("must reject tampered hash");
        assert!(
            matches!(err, PrxError::HashMismatch { .. }),
            "expected HashMismatch, got {err:?}"
        );
        let err2 = load_prx_gz_from_lock(&prx_gz).expect_err("lock loader must reject too");
        assert!(matches!(err2, PrxError::HashMismatch { .. }));
    }

    /// A correct envelope under an unregistered name has no lock pin →
    /// the lock-driven loader fails closed.
    #[test]
    fn load_validation_rejects_unpinned_source() {
        let prx_gz = emit_prx_gz(
            CITO_2_8_1_OWL.as_bytes(),
            "not_a_registered_source",
            "9.9.9",
            CITO_URL,
        )
        .expect("emit");
        let err = load_prx_gz_from_lock(&prx_gz).expect_err("unpinned source must be rejected");
        assert!(matches!(err, PrxError::NoLockPin { .. }), "got {err:?}");
    }

    /// A truncated/corrupted gzip or rkyv blob fails closed (bytecheck),
    /// never materializing unsound references.
    #[test]
    fn load_rejects_corrupted_blob() {
        let prx_gz = emit_prx_gz(CITO_2_8_1_OWL.as_bytes(), CITO_NAME, CITO_VERSION, CITO_URL)
            .expect("emit");
        let pin = lock_hashes().get("cito@2.8.1").expect("pin").clone();
        // Truncate the gzip stream.
        let truncated = &prx_gz[..prx_gz.len() / 2];
        assert!(load_prx_gz(truncated, &pin).is_err(), "truncated must fail");
        // Corrupt the rkyv layer: valid gzip wrapping garbage rkyv bytes.
        let garbage = gzip(b"not a valid rkyv envelope at all").expect("gzip");
        let err = load_prx_gz(&garbage, &pin).expect_err("garbage rkyv must fail");
        assert!(matches!(err, PrxError::Rkyv(_)), "got {err:?}");
    }

    // ── distribution emitter: emit_all_prx_gz over the live registry ─

    /// `emit_all_prx_gz` walks the live registry, emits a `.prx.gz` for
    /// every on-disk `OntologyVocabulary`, and round-trip-validates each.
    /// At least one artifact is emitted (the bundled SPAR/OLiA vocabularies
    /// are on disk), each file exists and is non-empty, names follow the
    /// `<name>-<version>.prx.gz` convention, and — the load-bearing claim —
    /// each re-loads through the fail-closed lock gate. The emitter already
    /// validates internally; re-loading here makes the published-artifact
    /// guarantee explicit in the test.
    #[test]
    fn emit_all_prx_gz_writes_and_round_trips_every_vocabulary() {
        use crate::applied::data_provisioning::registry::data_sources;
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;

        let out = std::env::temp_dir().join(format!(
            "prx-emit-test-{}-{}",
            std::process::id(),
            // A per-invocation suffix so parallel test processes don't collide.
            CITO_2_8_1_OWL.len()
        ));
        // Start from a clean dir so the count assertions are exact.
        let _ = std::fs::remove_dir_all(&out);

        let emitted = emit_all_prx_gz(&out).expect("emit_all_prx_gz must succeed");

        // Every registered, on-disk OntologyVocabulary is emitted — the same
        // discovery set `loaded_vocabularies` materializes.
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(std::path::PathBuf::from)
            .expect("workspace root");
        let expected: usize = data_sources()
            .iter()
            .filter(|e| e.kind == SourceTaxonomyConcept::OntologyVocabulary)
            .filter(|e| root.join(e.local_path()).exists())
            .count();
        assert!(expected >= 1, "at least one OntologyVocabulary on disk");
        assert_eq!(
            emitted.len(),
            expected,
            "emit_all_prx_gz emits one artifact per on-disk OntologyVocabulary"
        );

        for art in &emitted {
            // The file exists on disk with the recorded byte length, named by
            // the `<name>-<version>.prx.gz` convention.
            let meta = std::fs::metadata(&art.path).expect("artifact must be on disk");
            assert_eq!(meta.len(), art.byte_len, "recorded byte_len matches disk");
            assert!(art.byte_len > 0, "artifact is non-empty");
            assert_eq!(
                art.path.file_name().and_then(|f| f.to_str()),
                Some(format!("{}-{}.prx.gz", art.name, art.version).as_str()),
                "filename follows <name>-<version>.prx.gz"
            );

            // Explicit round-trip: read the published file and load it through
            // the fail-closed lock gate.
            let bytes = std::fs::read(&art.path).expect("read artifact");
            let loaded = load_prx_gz_from_lock(&bytes)
                .unwrap_or_else(|e| panic!("artifact {} must re-load: {e}", art.path.display()));
            assert!(
                loaded.entity_count() > 0,
                "loaded vocabulary {} is non-empty",
                art.name
            );
        }

        // The bundled CiTO source is on disk, so it appears among the emitted
        // artifacts with its registry version.
        assert!(
            emitted
                .iter()
                .any(|a| a.name == CITO_NAME && a.version == CITO_VERSION),
            "cito-{CITO_VERSION}.prx.gz must be emitted"
        );

        let _ = std::fs::remove_dir_all(&out);
    }

    // ── proptest over synthesised vocabularies ───────────────────────

    /// A synthesised vocabulary: class IRIs, property IRIs, and acyclic
    /// within-kind child→parent edges (parent index strictly greater than
    /// child, matching real `rdfs:subClassOf` DAGs).
    #[derive(Debug, Clone)]
    struct SynthVocab {
        classes: Vec<String>,
        properties: Vec<String>,
        class_edges: Vec<(usize, usize)>,
        prop_edges: Vec<(usize, usize)>,
    }

    fn arb_edges(n: usize) -> BoxedStrategy<Vec<(usize, usize)>> {
        if n < 2 {
            return Just(Vec::new()).boxed();
        }
        proptest::collection::vec((0..n, 1..n), 0..6)
            .prop_map(move |raw| {
                let mut edges: Vec<(usize, usize)> = raw
                    .into_iter()
                    .filter_map(|(child, raw_parent)| {
                        let parent = child + 1 + (raw_parent % (n - 1));
                        (parent < n).then_some((child, parent))
                    })
                    .collect();
                edges.sort_unstable();
                edges.dedup();
                edges
            })
            .boxed()
    }

    fn arb_synth() -> impl Strategy<Value = SynthVocab> {
        (1usize..=5, 1usize..=5).prop_flat_map(|(n_cls, n_prop)| {
            let classes: Vec<String> = (0..n_cls)
                .map(|i| format!("http://ex.org/v#C{i}"))
                .collect();
            let properties: Vec<String> = (0..n_prop)
                .map(|i| format!("http://ex.org/v#p{i}"))
                .collect();
            (
                Just(classes),
                Just(properties),
                arb_edges(n_cls),
                arb_edges(n_prop),
            )
                .prop_map(|(classes, properties, class_edges, prop_edges)| {
                    SynthVocab {
                        classes,
                        properties,
                        class_edges,
                        prop_edges,
                    }
                })
        })
    }

    /// Build an [`OwnedCodegenData`] directly from a synthesised
    /// vocabulary (one entity per class/property, within-kind taxonomy
    /// edges) — no XML round-trip needed; the entity/edge shape is exactly
    /// what `owl_to_builder` produces.
    fn synth_owned(s: &SynthVocab) -> OwnedCodegenData {
        let n_cls = s.classes.len();
        let mut entity_ids = Vec::new();
        let mut entity_kind = Vec::new();
        let mut entity_labels = Vec::new();
        for iri in &s.classes {
            entity_ids.push(iri.clone());
            entity_kind.push("Class".to_string());
            entity_labels.push(iri.clone());
        }
        for iri in &s.properties {
            entity_ids.push(iri.clone());
            entity_kind.push("ObjectProperty".to_string());
            entity_labels.push(iri.clone());
        }
        let mut taxonomy: Vec<(u64, u64)> = Vec::new();
        for (c, p) in &s.class_edges {
            taxonomy.push((*c as u64, *p as u64));
        }
        for (c, p) in &s.prop_edges {
            taxonomy.push(((n_cls + *c) as u64, (n_cls + *p) as u64));
        }
        OwnedCodegenData {
            entity_count: entity_ids.len() as u64,
            entity_ids,
            entity_kind,
            entity_labels,
            entity_defs: s
                .classes
                .iter()
                .chain(s.properties.iter())
                .map(|_| String::new())
                .collect(),
            word_index: Vec::new(),
            taxonomy,
            mereology: Vec::new(),
            opposition: Vec::new(),
            equivalence: Vec::new(),
            causation: Vec::new(),
            references: Vec::new(),
        }
    }

    fn synth_envelope(s: &SynthVocab, name: &str, version: &str) -> PrxEnvelope {
        let data = synth_owned(s);
        let n_cls = s.classes.len() as u64;
        let n_prop = s.properties.len() as u64;
        // A deterministic synthetic "source" whose hash we control.
        let source = format!("{name}@{version}::{}", data.entity_count);
        PrxEnvelope {
            metadata: PrxMetadata {
                name: name.to_string(),
                version: version.to_string(),
                ontology_uri: "http://ex.org/v#".to_string(),
                source_url: "http://ex.org/v".to_string(),
                source_sha256: source_content_hash(source.as_bytes()),
                number_of_classes: n_cls,
                number_of_properties: n_prop,
            },
            data,
        }
    }

    proptest! {
        /// emit→load preserves entities + edges, and validation accepts
        /// the matching pin. Drives the full gzip ∘ rkyv ∘ leak ∘
        /// from_codegen path.
        #[test]
        fn prop_emit_load_preserves_entities_and_edges(s in arb_synth()) {
            let envelope = synth_envelope(&s, "synth", "1.0");
            let pin = envelope.metadata.source_sha256.clone();
            let prx_gz = gzip(&envelope_to_bytes(&envelope).expect("serialize")).expect("gzip");

            let loaded = load_prx_gz(&prx_gz, &pin).expect("matching pin must load");
            prop_assert_eq!(loaded.entity_count(), s.classes.len() + s.properties.len());
            prop_assert_eq!(
                loaded.subsumption_edge_count(),
                s.class_edges.len() + s.prop_edges.len()
            );
            // Every synthesised class edge survives as an is_a.
            for (c, p) in &s.class_edges {
                prop_assert!(loaded.is_a(&s.classes[*c], &s.classes[*p]));
            }
            for (c, p) in &s.prop_edges {
                prop_assert!(loaded.is_a(&s.properties[*c], &s.properties[*p]));
            }
        }

        /// Tampering the embedded hash always makes the load gate reject
        /// (fail-closed) — for every synthesised vocabulary and every
        /// distinct wrong pin.
        #[test]
        fn prop_tampered_hash_always_rejected(s in arb_synth(), flip in 0u8..32) {
            let envelope = synth_envelope(&s, "synth", "1.0");
            let real_pin = envelope.metadata.source_sha256.clone();
            // A wrong pin: flip one hex nibble of the real pin.
            let mut wrong: Vec<char> = real_pin.chars().collect();
            let idx = (flip as usize) % wrong.len();
            wrong[idx] = if wrong[idx] == '0' { '1' } else { '0' };
            let wrong_pin: String = wrong.into_iter().collect();
            prop_assume!(wrong_pin != real_pin);

            let prx_gz = gzip(&envelope_to_bytes(&envelope).expect("serialize")).expect("gzip");
            let err = load_prx_gz(&prx_gz, &wrong_pin).expect_err("wrong pin must reject");
            let is_mismatch = matches!(err, PrxError::HashMismatch { .. });
            prop_assert!(is_mismatch, "expected HashMismatch, got {:?}", err);
        }
    }
}
