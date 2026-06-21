//! The generalized, registry-driven RAW-SOURCE `.prx` load path — the ONE
//! mechanism every non-graph source (XSD, DTD, XHTML, the XML-1.0 spec, the
//! OOXML ZIP, the TSV vocabularies, the Adobe glyph list) materializes its
//! committed bytes through, the byte-stream sibling of the OWL-vocabulary
//! [`load_owl_vocabulary`] path.
//!
//! ## Why a byte-stream codec, and why it is ONE mechanism
//!
//! Phase 1 generalized the OWL vocabularies onto a single
//! [`load_owl_vocabulary`] path: a source's load is
//! `registry.by_name(X) → resolve X.prx → gated load`. That path materializes a
//! *parsed graph* (`OwnedCodegenData` → `LoadedOwlVocabulary`) — it only fits
//! sources whose runtime value is that graph (OWL, WordNet-LMF, USLM).
//!
//! Phase 2's sources are heterogeneous in their PARSED form — an XSD becomes a
//! schema AST, a DTD a [`DtdSchema`], a TSV a `Vec<StateBitDef>`, the glyph list
//! a `HashMap`. They share NO graph shape. What they DO share is the layer one
//! step below: every one of them loads its **raw committed bytes** (today via
//! `include_str!` / `include_bytes!`) and hands those bytes to a source-specific
//! parser. So the ONE commonality to lift is the *byte materialization*, not the
//! parse. This module is exactly that lift: a single content-addressed
//! raw-bytes envelope codec plus a single registry-driven loader
//! ([`load_raw_source`]) that every phase-2 `include_*!` site is repointed to.
//! Each source's existing parser is untouched — it now consumes
//! `raw_source_text_by_name(X)` / `raw_source_bytes_by_name(X)` instead of an
//! embedded `&'static str`/`&'static [u8]`.
//!
//! Dispatch is by the source's [`ContentType`]: [`is_raw_source_content_type`]
//! decides membership (mirroring how the decoders dispatch by `ContentType`), so
//! the loader walks the registry and routes every raw-source entry through the
//! same gate — never N hand-written per-source loaders.
//!
//! ## The load path (one registered source)
//!
//! ```text
//! raw_prx_path(entry)  ─►  std::fs::read   (the COMMITTED .prx)
//!   └► lock_compact_archive_signature(name, version)   (the trusted pin)
//!        └► load_raw_source_prx_gated(bytes, pin, key)
//!             (hash-check the succinct bytes → decode → blob)
//!                  └► Vec<u8>   (the exact raw source bytes the parser reads)
//! ```
//!
//! The envelope is the [`RawBytesComplementFloor`] tier (Bancilhon & Spyratos
//! 1981 constant-complement): the source bytes ARE the payload, carried as a
//! length-prefixed blob alongside the `name`/`version` key, succinct-encoded
//! (dependency-free LEB128 framing — portable across toolchains and targets,
//! wasm32 included). The content address is taken over those succinct bytes, so
//! it pins into the SAME `praxis.lock` `[compact_archive_signatures]` space the
//! OWL / WordNet / USC compact archives use.
//!
//! ## No-gzip, feature-light by design
//!
//! Unlike the OWL/WordNet compact archives, the committed raw-source `.prx` is
//! **NOT gzip-wrapped**: these text/TSV/glyph sources are small, and gzip would
//! pull `flate2` (the `prx` feature) into the gate — but these sources load in
//! the default `std`-only build (and the schema/grammar loaders even on
//! `no_std`). The gate needs only the content-address hash
//! ([`pr4xis_runtime::address`]) and the `raw_hash` verifier, both
//! feature-independent, so the load path is gated on `std` alone (for
//! `std::fs`), never on `prx`. A committed `.prx` whose succinct bytes do not
//! hash to the pin — or that has no pin — is rejected fail-closed before any
//! bytes are returned (Dolstra 2006 content-addressing; W3C SRI 2016). The raw
//! source is fetch-only (`pr4xis update`); it ships in no published crate.
//!
//! ## Citations
//!
//! - **Bancilhon, F. & Spyratos, N. (1981)** "Update Semantics of Relational
//!   Views", *ACM TODS* 6(4) — the constant-complement view-update tier this
//!   envelope realises ([`RawBytesComplementFloor`]).
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators for
//!   Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §2.2 — the
//!   well-behaved-lens GetPut law `emit`/`load` witness.
//! - **Dolstra, E. (2006)** *The Purely Functional Software Deployment Model* —
//!   content-addressing by cryptographic hash.
//!
//! [`load_owl_vocabulary`]: crate::social::software::markup::xml::owl::loaded_vocabularies::load_owl_vocabulary
//! [`DtdSchema`]: crate::formal::meta::dtd::DtdSchema
//! [`RawBytesComplementFloor`]: crate::formal::meta::well_behaved_lens::RoundTripFidelity::RawBytesComplementFloor
//! [`ContentType`]: super::ontology::ContentType

#[allow(unused_imports)]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use pr4xis_runtime::address::ContentAddress;

use super::ontology::{ContentType, RegistryEntry};
use super::registry::{LockDigest, by_name, data_sources, lock_compact_archive_signature};
use crate::formal::meta::artifact_identity::ontology::{
    IdentityClaim, IdentityConcept, VerificationResult,
};
use crate::formal::meta::artifact_identity::schemes::raw_hash;

/// An error from emitting or loading a raw-source `.prx`. Fail-closed: every
/// variant names the offending source key and is surfaced, never swallowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawSourcePrxError {
    /// The committed `.prx` succinct bytes are truncated / malformed (a varint
    /// or blob span runs past the buffer).
    Malformed(String),
    /// The committed `.prx`'s succinct bytes did not hash to the trusted
    /// `[compact_archive_signatures]` pin — refusing to install.
    HashMismatch {
        key: String,
        expected: String,
        found: String,
    },
    /// The `raw_hash` verifier could not evaluate the claim (a malformed pin) —
    /// fail-closed.
    IntegrityUnverifiable { key: String, reason: String },
    /// The decoded payload was expected to be UTF-8 text but is not.
    NotUtf8(String),
}

impl core::fmt::Display for RawSourcePrxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RawSourcePrxError::Malformed(m) => write!(f, "raw-source .prx malformed: {m}"),
            RawSourcePrxError::HashMismatch {
                key,
                expected,
                found,
            } => write!(
                f,
                "raw-source .prx hash mismatch for `{key}`: praxis.lock pins {expected}, \
                 archive carries {found} — refusing to install"
            ),
            RawSourcePrxError::IntegrityUnverifiable { key, reason } => write!(
                f,
                "raw-source .prx integrity claim for `{key}` is unverifiable: {reason}"
            ),
            RawSourcePrxError::NotUtf8(m) => write!(f, "raw-source .prx payload not UTF-8: {m}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RawSourcePrxError {}

/// Does this [`ContentType`] load through the generalized RAW-SOURCE `.prx`
/// path (this module), rather than through a graph codec (OWL / WordNet-LMF /
/// USLM) or having no `.prx` consumer at all?
///
/// EXHAUSTIVE — no wildcard arm: adding a `ContentType` variant is a COMPILE
/// ERROR here until its `.prx` disposition is decided. The `true` arms are the
/// phase-2 byte-stream sources whose runtime value is a *parse* of their raw
/// bytes (so the graph codecs do not fit them): XSD, DTD, XHTML, the XML-1.0
/// spec text, the OOXML ZIP, plain text (the Adobe glyph list + the TSV
/// vocabularies), and the conformance-suite archives. The graph-codec content
/// types (`Owl` / `XmlLmf` / `UslmXml`) load through their OWN compact `.prx`
/// path and are `false` here. The not-yet-decodable media types are `false`.
#[must_use]
pub fn is_raw_source_content_type(content_type: ContentType) -> bool {
    use ContentType as CT;
    match content_type {
        // Byte-stream sources: parsed downstream from their raw bytes, no
        // shared graph shape — they ride this generalized raw-bytes envelope.
        CT::XmlXsd
        | CT::XmlDtd
        | CT::Xhtml
        | CT::Plaintext
        | CT::AdobeGlyphList
        | CT::ZipArchive
        | CT::TarGzArchive
        | CT::Json
        // The math-operator vocabulary is WN-LMF-shaped XML but its runtime
        // value is an `OperatorVocabulary` (a parse of its raw bytes), NOT a
        // WordNet graph — so it rides this generalized raw-bytes envelope, not
        // the graph-faithful `WordNetPrxEnvelope` the `XmlLmf` sources use.
        | CT::MathOperatorLmf
        // The Base16/Base24 color-scheme COLLECTION is a directory of named-
        // scheme YAML files archived into ONE deterministic blob — its runtime
        // value is the `path → bytes` theme set (a parse of that blob, decoded
        // by `decoders::theme_collection`), with no shared graph shape — so it
        // rides this generalized raw-bytes envelope exactly like the single-file
        // byte-stream sources. (The archive blob is the source's raw bytes; the
        // committed `.prx` wraps it; the validator decodes it back to schemes.)
        | CT::ThemeCollection
        // The closed-class function-word / legal lexica (`XmlLmfLexicon`) are
        // small, bounded WN-LMF lexica whose loaders run in the default
        // `std`-only build (no `prx`/gzip) — so they materialize their source
        // bytes through THIS feature-light raw-bytes envelope, then `read_wordnet`
        // parses them. (The open-class WordNet `XmlLmf` corpus is too large for
        // this and rides the graph `.prx.gz` envelope under `prx` instead.)
        | CT::XmlLmfLexicon => true,
        // Graph-codec content types — load through their own compact `.prx`
        // path (`owl::prx`, `lmf::prx`, `uslm::corpus::prx`), NOT this one.
        CT::Owl | CT::XmlLmf | CT::UslmXml => false,
        // No `.prx` consumer (no decoder / no runtime materialization yet).
        CT::Pdf | CT::Video | CT::Audio | CT::Binary => false,
    }
}

/// The committed raw-source `.prx` path for a registered entry — the
/// registry-driven sibling of [`RegistryEntry::local_path`] that resolves the
/// committed archive instead of the raw source. It is exactly
/// `entry.local_path()` with its published extension swapped for `.prx`, reusing
/// the one path formula in `local_path()` rather than hand-building a second
/// path — the byte-stream analogue of the OWL `prx_path`.
///
/// This is the committed artifact [`load_raw_source`] reads; the raw source it
/// sits beside is fetch-only and ships in no published crate.
#[must_use]
pub fn raw_prx_path(entry: &RegistryEntry) -> String {
    let raw = entry.local_path();
    match raw.rsplit_once('.') {
        // Swap the final extension for `.prx` (`foo/bar-1.0.xsd` →
        // `foo/bar-1.0.prx`). A compound `.tar.gz` keeps its `.tar`
        // (`x.tar.gz` → `x.tar.prx`) — a deterministic, collision-free sibling.
        Some((stem, _ext)) => format!("{stem}.prx"),
        // No extension: append (degenerate, but never a silent mismatch).
        None => format!("{raw}.prx"),
    }
}

// =============================================================================
// The raw-source envelope codec — a self-describing, content-addressed blob.
// =============================================================================

/// Append `bytes` length-prefixed (LEB128 varint length + raw bytes).
fn put_blob(out: &mut Vec<u8>, bytes: &[u8]) {
    let mut n = bytes.len() as u64;
    loop {
        let b = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(b);
            break;
        }
        out.push(b | 0x80);
    }
    out.extend_from_slice(bytes);
}

/// Read one length-prefixed blob with full bounds checking — the panic-proof
/// reader the gate relies on (a truncated archive is Err, never a panic).
fn get_blob<'a>(buf: &'a [u8], pos: &mut usize) -> Result<&'a [u8], RawSourcePrxError> {
    let mut len: u64 = 0;
    let mut shift = 0u32;
    loop {
        let b = *buf
            .get(*pos)
            .ok_or_else(|| RawSourcePrxError::Malformed("varint runs past end of buffer".into()))?;
        *pos += 1;
        len |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(RawSourcePrxError::Malformed(
                "varint length overflow".into(),
            ));
        }
    }
    let len = len as usize;
    let end = pos
        .checked_add(len)
        .filter(|&e| e <= buf.len())
        .ok_or_else(|| RawSourcePrxError::Malformed("blob runs past end of buffer".into()))?;
    let b = &buf[*pos..end];
    *pos = end;
    Ok(b)
}

/// Encode the raw source `bytes` into the portable succinct envelope:
/// `put_blob(name) put_blob(version) put_blob(bytes)`. Dependency-free LEB128
/// framing — no rkyv, no gzip — so the layout is stable across toolchains and
/// targets and the content address taken over it is portable.
#[must_use]
pub fn encode_raw_source(name: &str, version: &str, bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len() + name.len() + version.len() + 16);
    put_blob(&mut out, name.as_bytes());
    put_blob(&mut out, version.as_bytes());
    put_blob(&mut out, bytes);
    out
}

/// Decode a raw-source succinct envelope back into `(name, version, bytes)` —
/// the exact inverse of [`encode_raw_source`]. Fail-closed on a truncated /
/// malformed blob.
pub fn decode_raw_source(buf: &[u8]) -> Result<(String, String, Vec<u8>), RawSourcePrxError> {
    let mut pos = 0usize;
    let name = get_blob(buf, &mut pos)?;
    let version = get_blob(buf, &mut pos)?;
    let blob = get_blob(buf, &mut pos)?;
    let name = core::str::from_utf8(name)
        .map_err(|e| RawSourcePrxError::NotUtf8(format!("name: {e}")))?
        .to_string();
    let version = core::str::from_utf8(version)
        .map_err(|e| RawSourcePrxError::NotUtf8(format!("version: {e}")))?
        .to_string();
    Ok((name, version, blob.to_vec()))
}

/// Emit the committed raw-source `.prx` succinct bytes — [`encode_raw_source`].
/// The small, portable, content-addressed artifact committed under `data/` and
/// loaded by [`load_raw_source`]. The byte-stream sibling of
/// `emit_compact_english_prx_gz` (minus the gzip wrap, which these small text
/// sources don't need and which would pull `flate2` into the default build).
#[must_use]
pub fn emit_raw_source_prx(name: &str, version: &str, bytes: &[u8]) -> Vec<u8> {
    encode_raw_source(name, version, bytes)
}

/// The content address of a raw-source `.prx` — the digest of its succinct
/// bytes, as 64-char lowercase hex. The value pinned in `praxis.lock`
/// `[compact_archive_signatures]` and the one [`load_raw_source_prx_gated`]
/// re-derives and verifies. Portable: the LEB128 framing is dependency-free,
/// stable across toolchains and targets.
#[must_use]
pub fn raw_source_archive_address(prx: &[u8]) -> String {
    ContentAddress::of(prx).to_hex()
}

/// Discharge a content-hash `IntegrityClaim` over the succinct bytes against the
/// trusted `[compact_archive_signatures]` pin — the SAME `raw_hash::verify` leg
/// the OWL / WordNet / USC compact gates and the fetch path use (one
/// content-hash primitive, the algorithm taken from the trusted lock value,
/// never the payload).
fn verify_content_address(
    bytes: &[u8],
    pin: &LockDigest,
    key: &str,
) -> Result<(), RawSourcePrxError> {
    let claim = IdentityClaim {
        concept: IdentityConcept::RawHash,
        data: pin.claim_data(),
    };
    match raw_hash::verify(&claim, bytes) {
        VerificationResult::Verified(_) => Ok(()),
        VerificationResult::Mismatch { expected, actual } => Err(RawSourcePrxError::HashMismatch {
            key: key.to_string(),
            expected,
            found: actual,
        }),
        VerificationResult::Unverifiable { reason } => {
            Err(RawSourcePrxError::IntegrityUnverifiable {
                key: key.to_string(),
                reason,
            })
        }
    }
}

/// Load a committed raw-source `.prx` into its exact source bytes through the
/// fail-closed content-address gate: verify the succinct bytes hash to
/// `archive_pin` (the `[compact_archive_signatures]` pin) → decode → return the
/// `blob`. A committed archive whose bytes do not match the pin is rejected
/// before any bytes are returned (Dolstra 2006; W3C SRI 2016).
///
/// This is the SINGLE gated entry point the committed-raw-source load path
/// ([`load_raw_source`]) routes through — every phase-2 byte-stream source.
pub fn load_raw_source_prx_gated(
    prx: &[u8],
    archive_pin: &LockDigest,
    key: &str,
) -> Result<Vec<u8>, RawSourcePrxError> {
    verify_content_address(prx, archive_pin, key)?;
    let (_name, _version, blob) = decode_raw_source(prx)?;
    Ok(blob)
}

/// Zero-copy twin of [`load_raw_source_prx_gated`]: verify the succinct bytes
/// against `archive_pin`, then return the payload `blob` AS A BORROWED SLICE of
/// `prx` rather than an owned `Vec`. The envelope is uncompressed LEB128 framing
/// (no rkyv, no gzip), so the payload is a contiguous sub-slice — which lets the
/// embedded-`.prx` text accessor hand back a `&str` that borrows the `'static`
/// `include_bytes!` array, instead of leaking a fresh allocation on every call
/// (the `no_std`/wasm re-parse path calls the accessor per use). Same fail-closed
/// gate; nothing is borrowed until the content address verifies.
pub fn load_raw_source_prx_gated_borrowed<'a>(
    prx: &'a [u8],
    archive_pin: &LockDigest,
    key: &str,
) -> Result<&'a [u8], RawSourcePrxError> {
    verify_content_address(prx, archive_pin, key)?;
    let mut pos = 0usize;
    let _name = get_blob(prx, &mut pos)?;
    let _version = get_blob(prx, &mut pos)?;
    get_blob(prx, &mut pos)
}

/// Load ONE registered raw-source entry's bytes from its committed `.prx`
/// through the fail-closed `[compact_archive_signatures]` gate — the single
/// generalized raw-source load mechanism every phase-2 `include_*!` site is
/// repointed to.
///
/// Reads `workspace_root.join(`[`raw_prx_path`]`(entry))`, looks up the source's
/// compact pin, and hands both to [`load_raw_source_prx_gated`]. Returns:
///
/// - `Ok(Some(bytes))` — the committed `.prx` is on disk, pinned, and passed the
///   content gate;
/// - `Ok(None)` — the committed `.prx` is **not on disk** OR the source has no
///   compact pin (graceful skip — a fresh checkout that hasn't run
///   `pr4xis compile`, or a not-yet-pinned source), mirroring the OWL
///   `load_owl_vocabulary` graceful skip;
/// - `Err(_)` — the `.prx` is on disk AND pinned but **failed the content gate**
///   (a stale/poisoned artifact): a defect, surfaced fail-closed.
#[cfg(feature = "std")]
pub fn load_raw_source(entry: &RegistryEntry) -> Result<Option<Vec<u8>>, RawSourcePrxError> {
    let path = workspace_root().join(raw_prx_path(entry));
    let Ok(prx) = std::fs::read(&path) else {
        return Ok(None);
    };
    let Some(pin) = lock_compact_archive_signature(&entry.name, &entry.version) else {
        return Ok(None);
    };
    let key = format!("{}@{}", entry.name, entry.version);
    load_raw_source_prx_gated(&prx, pin, &key).map(Some)
}

/// Load a registered raw-source's bytes BY NAME, panicking with an actionable
/// message if the committed `.prx` is absent / unpinned / fails the gate.
///
/// The accessor the repointed `include_*!` sites call: they previously embedded
/// the raw bytes unconditionally (a build invariant), so they keep that
/// hard-fail contract here — a missing committed `.prx` is a "forgot to run
/// `pr4xis compile`" defect, named, never a silent empty load. Returns the exact
/// source bytes (the same bytes the old `include_*!` produced).
#[cfg(feature = "std")]
#[must_use]
pub fn raw_source_bytes_by_name(name: &str) -> Vec<u8> {
    let entry = by_name(name).unwrap_or_else(|| {
        panic!(
            "raw-source `{name}` is not registered in praxis.toml — cannot load its committed .prx"
        )
    });
    match load_raw_source(entry) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => panic!(
            "raw-source `{name}@{}`: committed `{}` is absent or unpinned — run \
             `pr4xis compile --compact --lock` to emit + pin it",
            entry.version,
            raw_prx_path(entry),
        ),
        Err(e) => panic!(
            "raw-source `{name}@{}`: committed `{}` failed the [compact_archive_signatures] \
             content gate: {e}",
            entry.version,
            raw_prx_path(entry),
        ),
    }
}

/// Like [`raw_source_bytes_by_name`] but returns the bytes as a leaked
/// `&'static str` (the text-source accessor — XSD / DTD / XHTML / XML-spec / TSV
/// / glyph list all parse from text and the old `include_str!` sites returned
/// `&'static str`). The leak is process-lifetime, identical in effect to the
/// build-embedded `static` it replaces. Panics if the committed bytes are not
/// UTF-8 (a build-time invariant for these text sources).
#[cfg(feature = "std")]
#[must_use]
pub fn raw_source_text_by_name(name: &str) -> &'static str {
    let bytes = raw_source_bytes_by_name(name);
    let s = String::from_utf8(bytes)
        .unwrap_or_else(|e| panic!("raw-source `{name}` committed bytes are not UTF-8: {e}"));
    alloc::boxed::Box::leak(s.into_boxed_str())
}

/// Materialize the raw source bytes from an **embedded** committed `.prx` blob —
/// the `no_std`/wasm-safe accessor the repointed `include_bytes!` sites call.
///
/// Phase-2 sources are loaded in builds with NO `std::fs` (`no_std`/wasm) and NO
/// `prx` feature (the default build), so — exactly like phase 1's OWL
/// `include_bytes!` of the committed `.prx.gz` — the repointed site embeds the
/// committed `.prx` with `include_bytes!` and hands it here. The gate
/// (content-address hash + `raw_hash::verify`) is feature-independent, so this
/// works on every target. The trusted pin is reached through the registry
/// (`[compact_archive_signatures]`), never read from the embedded blob.
///
/// Panics fail-closed (the build-time invariant the old `include_str!` had): an
/// unpinned source, or an embedded blob that does not hash to the pin, is a
/// defect named here, never a silent empty/garbage load.
#[must_use]
pub fn raw_source_bytes_embedded(name: &str, version: &str, embedded_prx: &[u8]) -> Vec<u8> {
    let key = format!("{name}@{version}");
    let Some(pin) = lock_compact_archive_signature(name, version) else {
        panic!(
            "raw-source `{key}`: no praxis.lock [compact_archive_signatures] pin — \
             run `pr4xis compile --compact --lock` to pin the committed .prx"
        )
    };
    load_raw_source_prx_gated(embedded_prx, pin, &key).unwrap_or_else(|e| {
        panic!("raw-source `{key}`: embedded committed .prx failed the content gate: {e}")
    })
}

/// Text form of [`raw_source_bytes_embedded`] — returns the decoded UTF-8 as a
/// `&str` that BORROWS `embedded_prx` (zero-copy), so a per-call `no_std`/wasm
/// accessor (`wm_state_vocabulary` / `english_irregulars` re-parse on every call)
/// allocates and leaks NOTHING. Because `embedded_prx` is the `'static`
/// `include_bytes!` array at every call site, the borrow is `'static` in practice
/// — the same `include_str!` semantics it replaces, now without the per-call
/// `Box::leak`. The accessor every text raw-source (`XSD`/`DTD`/`XHTML`/spec/
/// `TSV`/glyph list) repoints to. Fail-closed: panics on an unpinned source, a
/// gate mismatch, or non-UTF-8 committed bytes.
#[must_use]
pub fn raw_source_text_embedded<'a>(name: &str, version: &str, embedded_prx: &'a [u8]) -> &'a str {
    let key = format!("{name}@{version}");
    let Some(pin) = lock_compact_archive_signature(name, version) else {
        panic!(
            "raw-source `{key}`: no praxis.lock [compact_archive_signatures] pin — \
             run `pr4xis compile --compact --lock` to pin the committed .prx"
        )
    };
    let blob = load_raw_source_prx_gated_borrowed(embedded_prx, pin, &key).unwrap_or_else(|e| {
        panic!("raw-source `{key}`: embedded committed .prx failed the content gate: {e}")
    });
    core::str::from_utf8(blob)
        .unwrap_or_else(|e| panic!("raw-source `{name}` committed bytes are not UTF-8: {e}"))
}

/// Every registered raw-source entry — the set [`is_raw_source_content_type`]
/// admits, in registry (sorted-name) order. The iteration surface the compile
/// emitter and the source-wide property tests walk.
#[must_use]
pub fn raw_source_entries() -> Vec<&'static RegistryEntry> {
    data_sources()
        .iter()
        .filter(|e| is_raw_source_content_type(e.content_type()))
        .collect()
}

/// One emitted committed raw-source `.prx`: where it was written and the content
/// address `pr4xis compile --lock` pins into `[compact_archive_signatures]`. The
/// raw-source analogue of `EmittedArtifact` (kept local so the codec needn't
/// depend on the OWL/`prx` emit types).
#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedRawSource {
    pub name: String,
    pub version: String,
    pub path: std::path::PathBuf,
    pub byte_len: u64,
    pub archive_address: String,
}

/// Emit the committed raw-source `.prx` for EVERY registered raw-source entry
/// whose raw source is on disk, writing each beside its raw source
/// ([`raw_prx_path`]) and round-trip-validating it (the emitted bytes load back
/// through the content gate against the address this emit just produced) before
/// returning it.
///
/// Registry-driven (never a hardcoded set): walks [`raw_source_entries`], reads
/// `workspace_root.join(entry.local_path())` (the FETCHED raw), and for each emits
/// → writes → reads back → gate-loads. A raw source not on disk is skipped
/// gracefully (the same discipline `emit_all_compact_english_prx_gz` follows). The
/// caller (`pr4xis compile --lock`) pins each returned `archive_address` into
/// `praxis.lock` `[compact_archive_signatures]`.
#[cfg(feature = "std")]
pub fn emit_all_compact_raw_source_prx() -> Result<Vec<EmittedRawSource>, RawSourcePrxError> {
    let root = workspace_root();
    let mut emitted = Vec::new();
    for entry in raw_source_entries() {
        let src_path = root.join(entry.local_path());
        let Ok(source) = std::fs::read(&src_path) else {
            continue; // FETCHED raw not on disk — skip gracefully.
        };
        let prx = emit_raw_source_prx(&entry.name, &entry.version, &source);
        let archive_address = raw_source_archive_address(&prx);
        let path = root.join(raw_prx_path(entry));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                RawSourcePrxError::Malformed(format!("create {}: {e}", parent.display()))
            })?;
        }
        std::fs::write(&path, &prx)
            .map_err(|e| RawSourcePrxError::Malformed(format!("write {}: {e}", path.display())))?;

        // Round-trip-validate against the address this emit just produced AND
        // confirm the decoded blob equals the source bytes byte-for-byte.
        let pin = LockDigest::address(archive_address.clone());
        let key = format!("{}@{}", entry.name, entry.version);
        let back = load_raw_source_prx_gated(&prx, &pin, &key)?;
        if back != source {
            return Err(RawSourcePrxError::Malformed(format!(
                "{key}: emitted .prx does not round-trip to the source bytes"
            )));
        }

        emitted.push(EmittedRawSource {
            name: entry.name.clone(),
            version: entry.version.clone(),
            path,
            byte_len: prx.len() as u64,
            archive_address,
        });
    }
    Ok(emitted)
}

/// The workspace root — the parent of this crate's parent.
/// `RegistryEntry::local_path()` returns a workspace-relative path
/// (`crates/domains/data/...`), resolved against this root. Mirrors the OWL
/// loader's `workspace_root`.
#[cfg(feature = "std")]
fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn encode_decode_round_trips_exact_bytes() {
        for blob in [
            b"".as_slice(),
            b"hello",
            b"<?xml version=\"1.0\"?><root/>",
            &(0u8..=255).collect::<Vec<u8>>(),
        ] {
            let enc = encode_raw_source("src", "1.0", blob);
            let (n, v, out) = decode_raw_source(&enc).expect("decode");
            assert_eq!(n, "src");
            assert_eq!(v, "1.0");
            assert_eq!(out, blob, "raw-source codec must round-trip bytes exactly");
        }
    }

    #[test]
    fn gated_load_round_trips_through_gate() {
        let blob = b"a real XSD or TSV would go here\n";
        let prx = emit_raw_source_prx("widget", "2", blob);
        let pin = LockDigest::address(raw_source_archive_address(&prx));
        let out = load_raw_source_prx_gated(&prx, &pin, "widget@2").expect("gated load");
        assert_eq!(out, blob);
    }

    #[test]
    fn gated_load_rejects_wrong_pin_fail_closed() {
        let blob = b"payload";
        let prx = emit_raw_source_prx("widget", "2", blob);
        let wrong = LockDigest::address("0".repeat(64));
        let err =
            load_raw_source_prx_gated(&prx, &wrong, "widget@2").expect_err("wrong pin must reject");
        assert!(
            matches!(err, RawSourcePrxError::HashMismatch { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn decode_rejects_truncated_blob_without_panic() {
        let mut prx = emit_raw_source_prx("widget", "2", b"some bytes");
        // Lop off the tail: the final blob span now runs past the end. The
        // bounds-checked decoder must return Err, never panic through.
        prx.truncate(prx.len() - 3);
        let err = decode_raw_source(&prx).expect_err("truncated must be Err");
        assert!(
            matches!(err, RawSourcePrxError::Malformed(_)),
            "got {err:?}"
        );
    }

    #[test]
    fn raw_prx_path_swaps_the_extension_of_real_entries() {
        // For every registered raw-source entry, the committed-`.prx` path is the
        // source path with its final extension swapped for `.prx` — never the raw
        // extension, so the raw never resolves to itself.
        for entry in raw_source_entries() {
            let prx = raw_prx_path(entry);
            assert!(
                prx.ends_with(".prx"),
                "{}: raw_prx_path must end .prx, got {prx}",
                entry.name
            );
            assert_ne!(
                prx,
                entry.local_path(),
                "{}: committed .prx path must differ from the raw source path",
                entry.name
            );
        }
    }

    /// The set of registered raw-source entries that are CONVERTED in phase 2 —
    /// those whose committed `.prx` is on disk AND whose source is
    /// `[compact_archive_signatures]`-pinned. The property tests below walk this
    /// set (derived from the registry + lock + disk, never a hardcoded list).
    fn converted_raw_sources() -> Vec<&'static RegistryEntry> {
        raw_source_entries()
            .into_iter()
            .filter(|e| {
                workspace_root()
                    .join(raw_prx_path(e))
                    .try_exists()
                    .unwrap_or(false)
                    && lock_compact_archive_signature(&e.name, &e.version).is_some()
            })
            .collect()
    }

    /// SOURCE-WIDE ROUND-TRIP PROPERTY (`RawBytesComplementFloor` fidelity): for
    /// EVERY converted raw source, the committed `.prx` loads through the gate to
    /// bytes BYTE-EXACTLY equal to its FETCHED raw on disk, and those bytes hash
    /// to the source's `[hashes]` pin. This is the generalized-loader staleness
    /// guard: it cross-checks the committed `.prx` against the registered raw.
    ///
    /// It HARD-FAILS (never silently skips) if a converted source's FETCHED raw
    /// is absent — the raw is fetch-only (`pr4xis update`), and a committed `.prx`
    /// with no raw to cross-check against is the staleness blind-spot this guard
    /// exists to forbid.
    #[test]
    fn committed_prx_round_trips_to_fetched_raw_byte_exact() {
        use crate::applied::data_provisioning::registry::lock_hashes;
        let converted = converted_raw_sources();
        assert!(
            !converted.is_empty(),
            "no converted raw source on disk + pinned — phase-2 .prx must be committed + pinned"
        );
        let root = workspace_root();
        for entry in converted {
            // Load the committed `.prx` through the generalized gate.
            let loaded = load_raw_source(entry)
                .unwrap_or_else(|e| panic!("{}: committed .prx failed the gate: {e}", entry.name))
                .unwrap_or_else(|| panic!("{}: committed .prx absent/unpinned", entry.name));

            // The STALENESS GUARD: the FETCHED raw must be present and EQUAL.
            let raw_path = root.join(entry.local_path());
            let raw = std::fs::read(&raw_path).unwrap_or_else(|e| {
                panic!(
                    "{}: FETCHED raw `{}` is absent ({e}) — it is fetch-only; run \
                     `pr4xis update` to regenerate it (the committed .prx cannot be \
                     staleness-checked without it)",
                    entry.name,
                    raw_path.display()
                )
            });
            assert_eq!(
                loaded, raw,
                "{}: committed .prx bytes drifted from the FETCHED raw — re-run \
                 `pr4xis compile --compact --lock`",
                entry.name
            );

            // And the loaded bytes hash to the source's durable `[hashes]` pin.
            let key = format!("{}@{}", entry.name, entry.version);
            let raw_pin = lock_hashes()
                .get(&key)
                .unwrap_or_else(|| panic!("{}: no [hashes] pin", entry.name));
            assert!(
                raw_pin.verifies(&loaded),
                "{}: loaded committed-.prx bytes do not match the [hashes] pin",
                entry.name
            );
        }
    }

    /// REGENERATE PATH (`--ignored`, WRITES): rebuild every registered
    /// raw-source's committed `.prx` (including the 3 TSVs) from its on-disk
    /// source bytes via [`emit_all_compact_raw_source_prx`], the SAME emit the
    /// `pr4xis compile --compact` CLI runs. Each emitted `.prx` is round-trip
    /// validated against its own freshly computed address inside the emit. Run by
    /// hand after editing a source-of-truth TSV (then `pr4xis compile --lock` to
    /// re-pin), mirroring `regenerate_english_irregulars_tsv`:
    /// `cargo test -p pr4xis-domains -- --ignored regenerate_raw_source_prx`.
    /// NOTE: this only re-emits the `.prx`; the `[compact_archive_signatures]`
    /// pins are rewritten by `pr4xis compile --compact --lock`, not here.
    #[test]
    #[ignore]
    fn regenerate_raw_source_prx() {
        let emitted =
            emit_all_compact_raw_source_prx().expect("emit all committed raw-source .prx");
        for e in &emitted {
            eprintln!(
                "regenerated {}@{} → {} ({} bytes) address {}",
                e.name,
                e.version,
                e.path.display(),
                e.byte_len,
                e.archive_address
            );
        }
        assert!(
            !emitted.is_empty(),
            "no raw source on disk — nothing regenerated"
        );
    }

    /// TSV-SOURCE STALENESS + DECODE GUARD (the `Plaintext` ContentType leg of
    /// the source-wide round-trip): for EVERY registered `Plaintext` (TSV)
    /// source that is converted (committed `.prx` on disk + pinned), the
    /// committed `.prx` loads through the generalized gate to bytes that (a)
    /// equal the committed `.tsv` BYTE-EXACTLY — the staleness guard — and (b)
    /// DECODE cleanly through the `plaintext_tsv` codec to a non-empty record
    /// stream whose every row recovers from re-rendering. HARD-FAILS (never
    /// skips) if a registered TSV source's committed `.tsv` is absent, and
    /// asserts at least one TSV source is registered + converted (so the guard
    /// can never pass vacuously once a TSV is registered).
    #[test]
    fn committed_tsv_prx_round_trips_and_decodes() {
        use crate::applied::data_provisioning::decoders::plaintext_tsv;
        let root = workspace_root();
        let tsv_sources: Vec<&RegistryEntry> = converted_raw_sources()
            .into_iter()
            .filter(|e| e.content_type() == ContentType::Plaintext)
            .collect();
        assert!(
            !tsv_sources.is_empty(),
            "no converted Plaintext/TSV source — the 3 registered TSVs must be \
             committed as `.prx` + pinned (run `pr4xis compile --compact --lock`)"
        );
        for entry in tsv_sources {
            // Load the committed `.prx` through the generalized gate.
            let loaded = load_raw_source(entry)
                .unwrap_or_else(|e| panic!("{}: committed .prx failed the gate: {e}", entry.name))
                .unwrap_or_else(|| panic!("{}: committed .prx absent/unpinned", entry.name));

            // STALENESS GUARD: the committed `.tsv` source-of-truth must be
            // present (git-tracked, NOT fetch-only) and EQUAL byte-for-byte.
            let tsv_path = root.join(entry.local_path());
            let tsv = std::fs::read(&tsv_path).unwrap_or_else(|e| {
                panic!(
                    "{}: committed `.tsv` source-of-truth `{}` is absent ({e}) — it is \
                     git-tracked (regenerate the `.prx` from it via `pr4xis compile \
                     --compact --lock`); the committed `.prx` cannot be staleness-checked \
                     without it",
                    entry.name,
                    tsv_path.display()
                )
            });
            assert_eq!(
                loaded, tsv,
                "{}: committed .prx bytes drifted from the committed .tsv — re-run \
                 `pr4xis compile --compact --lock`",
                entry.name
            );

            // DECODE leg: the loaded bytes parse through the generic TSV codec to
            // a non-empty record stream, and every decoded row re-renders to its
            // own `\t`-joined wire line (the records ⇄ TSV GetPut law on real data).
            let records = plaintext_tsv::decode(&loaded).unwrap_or_else(|e| {
                panic!("{}: committed .prx does not decode as TSV: {e}", entry.name)
            });
            assert!(
                !records.is_empty(),
                "{}: TSV decoded to zero records — a real vocabulary has rows",
                entry.name
            );
            for row in &records {
                assert!(
                    !row.is_empty(),
                    "{}: a decoded TSV row has no fields",
                    entry.name
                );
                let rendered = row.join("\t");
                assert_eq!(
                    plaintext_tsv::parse(&rendered),
                    vec![row.clone()],
                    "{}: a decoded TSV row does not recover from re-rendering",
                    entry.name
                );
            }
        }
    }

    /// WN-LMF-SOURCE STALENESS + DECODE GUARD (the `XmlLmfLexicon` +
    /// `MathOperatorLmf` content-type legs of the source-wide round-trip — the
    /// phase-2d closed-class lexica + the math-operator vocabulary that ride this
    /// generalized raw-bytes path instead of the graph `.prx.gz` envelope): for
    /// EVERY converted source of these content types, the committed `.prx` loads
    /// through the generalized gate to bytes that (a) equal the git-tracked
    /// source-of-truth `.xml` BYTE-EXACTLY — the staleness guard — and (b) DECODE
    /// cleanly through the `read_wordnet` LMF reader to a non-empty
    /// `<LexicalEntry>` set. HARD-FAILS (never skips) if a converted source's
    /// `.xml` is absent, and asserts at least one such source is registered +
    /// converted (so the guard can never pass vacuously).
    #[test]
    fn committed_xml_lmf_prx_round_trips_and_decodes() {
        use crate::social::software::markup::xml::lmf::reader::read_wordnet;
        let root = workspace_root();
        let lmf_sources: Vec<&RegistryEntry> = converted_raw_sources()
            .into_iter()
            .filter(|e| {
                matches!(
                    e.content_type(),
                    ContentType::XmlLmfLexicon | ContentType::MathOperatorLmf
                )
            })
            .collect();
        assert!(
            !lmf_sources.is_empty(),
            "no converted XmlLmfLexicon/MathOperatorLmf source — english_function_words, \
             us_legal_lexicon, math_operators must be committed as `.prx` + pinned \
             (run `pr4xis compile --compact --lock`)"
        );
        for entry in lmf_sources {
            // Load the committed `.prx` through the generalized gate.
            let loaded = load_raw_source(entry)
                .unwrap_or_else(|e| panic!("{}: committed .prx failed the gate: {e}", entry.name))
                .unwrap_or_else(|| panic!("{}: committed .prx absent/unpinned", entry.name));

            // STALENESS GUARD: the git-tracked source-of-truth `.xml` must be
            // present and EQUAL byte-for-byte. (These are DERIVED/authored
            // sources excluded from the crate, NOT gitignored fetch-only ones —
            // so the source-of-truth is always on disk to cross-check against.)
            let xml_path = root.join(entry.local_path());
            let xml = std::fs::read(&xml_path).unwrap_or_else(|e| {
                panic!(
                    "{}: source-of-truth `{}` is absent ({e}) — it is git-tracked \
                     (regenerate the `.prx` from it via `pr4xis compile --compact --lock`); \
                     the committed `.prx` cannot be staleness-checked without it",
                    entry.name,
                    xml_path.display()
                )
            });
            assert_eq!(
                loaded, xml,
                "{}: committed .prx bytes drifted from the source-of-truth .xml — re-run \
                 `pr4xis compile --compact --lock`",
                entry.name
            );

            // DECODE leg: the loaded bytes parse through the WN-LMF reader to a
            // non-empty `<LexicalEntry>` set (the lexicon ⇄ XML GetPut on real
            // data — the same reader every consumer of these sources runs).
            let text = core::str::from_utf8(&loaded)
                .unwrap_or_else(|e| panic!("{}: committed .prx is not UTF-8: {e}", entry.name));
            let wn = read_wordnet(text).unwrap_or_else(|e| {
                panic!(
                    "{}: committed .prx does not decode as WN-LMF: {e}",
                    entry.name
                )
            });
            assert!(
                !wn.entries.is_empty(),
                "{}: WN-LMF decoded to zero lexical entries — a real lexicon has entries",
                entry.name
            );
        }
    }

    proptest::proptest! {
        /// GATE FAIL-CLOSED PROPERTY: forall single-byte mutation of a real
        /// committed `.prx`'s succinct bytes, the gated load returns `Err`
        /// (never `Ok`, never a panic-through). A mutation changes the content
        /// address, so the hash gate rejects it; a mutation that corrupts the
        /// framing is caught by the bounds-checked decoder — either way Err.
        #[test]
        fn prop_mutated_prx_always_rejected(
            byte_idx in any::<prop::sample::Index>(),
            xor in 1u8..=255,
        ) {
            // A real raw-source envelope (deterministic content, exercised across
            // payload sizes/bytes by the synthetic blob).
            let blob: Vec<u8> = (0u8..=200).cycle().take(777).collect();
            let prx = emit_raw_source_prx("widget", "1", &blob);
            let pin = LockDigest::address(raw_source_archive_address(&prx));

            let i = byte_idx.index(prx.len());
            let mut bad = prx.clone();
            bad[i] ^= xor; // a guaranteed change at a real index

            // The gate must reject the mutated bytes fail-closed. catch_unwind
            // proves it NEVER panics through (returns Err, never unwinds).
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                load_raw_source_prx_gated(&bad, &pin, "widget@1")
            }));
            match res {
                Ok(Ok(_)) => prop_assert!(
                    false,
                    "mutated .prx (byte {i} ^= {xor}) loaded OK — gate is not fail-closed"
                ),
                Ok(Err(_)) => {} // correct: fail-closed Err
                Err(_) => prop_assert!(false, "gate PANICKED on mutated bytes (byte {i})"),
            }
        }

        /// ENCODE/DECODE ROUND-TRIP PROPERTY: forall name/version/blob, the
        /// succinct envelope round-trips byte-exact (`decode(encode(x)) == x`),
        /// the GetPut leg of the bytes ⇄ `.prx` lens. The full gated load also
        /// recovers the blob under the matching pin, and the content address is
        /// a deterministic pure function of the inputs.
        #[test]
        fn prop_encode_decode_round_trips(
            name in "[a-z_]{1,16}",
            version in "[0-9.]{1,8}",
            blob in proptest::collection::vec(any::<u8>(), 0..1024),
        ) {
            let enc = encode_raw_source(&name, &version, &blob);
            let (n, v, out) = decode_raw_source(&enc)
                .map_err(|e| TestCaseError::fail(format!("decode: {e}")))?;
            prop_assert_eq!(&n, &name);
            prop_assert_eq!(&v, &version);
            prop_assert_eq!(&out, &blob);
            // Determinism: re-encoding the same inputs yields the same bytes and
            // the same content address (a pure function of the inputs).
            let enc2 = encode_raw_source(&name, &version, &blob);
            prop_assert_eq!(&enc, &enc2);
            prop_assert_eq!(
                raw_source_archive_address(&enc),
                raw_source_archive_address(&enc2)
            );
            // The full gated load recovers the blob under the matching pin.
            let key = format!("{name}@{version}");
            let pin = LockDigest::address(raw_source_archive_address(&enc));
            let loaded = load_raw_source_prx_gated(&enc, &pin, &key)
                .map_err(|e| TestCaseError::fail(format!("gated load: {e}")))?;
            prop_assert_eq!(loaded, blob);
        }
    }
}
