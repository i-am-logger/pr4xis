//! The ONE canonical `.prx` envelope codec — the dependency-light, portable
//! LEB128 framing plus the RAW-SOURCE and REGISTRY envelope grammars, shared
//! byte-for-byte between the runtime load path and the build script.
//!
//! ## Why this module exists (one grammar, not a hand-kept mirror)
//!
//! The runtime materializes committed `.prx` bytes through the raw-source gate
//! (`super::raw_source_prx`) and the registry-manifest gate
//! (`super::registry_prx`); the build script
//! (`crates/domains/build.rs`) materializes the SAME committed `.prx` artifacts
//! at COMPILE time (the XML schema / spec sources it feeds to codegen, and the
//! registry manifest itself when the workspace root is absent). Both sides
//! decode the identical envelope grammar. This module is that grammar, written
//! ONCE:
//!
//! ```text
//! RAW-SOURCE envelope (format version 2):
//!   varint(format_version = 2)
//!   blob(name) blob(version)      (the registry key)
//!   varint(encoding)              (a PayloadEncoding wire tag: 0 Identity, 1 Deflate)
//!   varint(decoded_len)           (the SOURCE byte length)
//!   blob(payload)                 (the encoded source bytes)
//!
//! REGISTRY envelope (the manifest self-root):
//!   blob(praxis.toml) blob(praxis.lock)
//! ```
//!
//! The build script `#[path]`-includes this exact file (`#[allow(dead_code)]`,
//! since it uses a subset), so the "MIRROR-MUST-MATCH INVARIANT" the two decoders
//! previously maintained BY HAND is now structural: there is one decoder.
//!
//! ## Portability + fail-closed totality
//!
//! Pure `core` + `alloc` (no `std`), so it compiles in the default `std` build,
//! on `no_std`, and on wasm32 — and, via the `#[path]` include, in the build
//! script (a `std` binary). Inflation uses `miniz_oxide` directly (pure Rust,
//! already in the graph as flate2's backend); content addresses use `blake3`
//! (both are `[dependencies]` and `[build-dependencies]`). Every decode is TOTAL
//! and fail-closed: a truncated / malformed / unknown-version / unknown-encoding
//! / forged-length / garbage-tailed / non-UTF-8 envelope is `Err`, never a panic
//! and never a guess.
//!
//! ## What stays in the domain modules
//!
//! Only the pure codec lives here. The domain-typed gates stay where they
//! belong: the runtime raw-source gate discharges its content-address claim
//! through the multi-algorithm [`raw_hash`](crate::formal::meta::artifact_identity::schemes::raw_hash)
//! path against a `LockDigest` (`super::registry::LockDigest`); the registry gate
//! verifies against the baked root through
//! [`ContentAddress`](pr4xis_runtime::address::ContentAddress). The build
//! script, which carries only the `blake3` hash dep, verifies through the
//! `content_address_hex` / `blake3_gated_raw_source_text` /
//! `blake3_gated_registry` helpers here.
//!
//! ## Citations
//!
//! - **Deutsch, P. (1996)** *RFC 1951: DEFLATE Compressed Data Format
//!   Specification version 1.3* — the payload compression; **RFC 1952** (gzip)
//!   deliberately NOT used (its `MTIME`/`OS` header fields are nondeterministic).
//! - **Gailly, J.-l. & Adler, M.**, *zlib Technical Details*
//!   (<https://zlib.net/zlib_tech.html>) — DEFLATE's ~1032:1 maximum expansion,
//!   the bound behind `DEFLATE_MAX_EXPANSION`'s decompression-bomb guard.
//! - **Dolstra, E. (2006)** *The Purely Functional Software Deployment Model* —
//!   content-addressing by cryptographic hash.
//! - **Aumasson et al. (2020)** *BLAKE3* — the content-address digest.

extern crate alloc;

use alloc::borrow::Cow;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

// =============================================================================
// Format constants
// =============================================================================

/// The raw-source envelope FORMAT VERSION this build reads and writes — the
/// leading varint of every raw-source envelope, making the layout EXPLICIT in
/// the bytes. Any other leading varint is an unknown format: rejected, never
/// guessed at (a v1-style unversioned envelope's leading varint is a blob
/// length, so it fails this check fail-closed).
pub const RAW_SOURCE_ENVELOPE_FORMAT_VERSION: u64 = 2;

/// DEFLATE's maximum expansion factor — the decompression-bomb guard bound.
///
/// Gailly & Adler, *zlib Technical Details* (<https://zlib.net/zlib_tech.html>):
/// one length/distance pair can represent at most 258 output bytes and costs at
/// least two bits of input, so the maximum decompression expansion is ~1032:1.
/// A declared `decoded_len` exceeding `payload.len() × 1032` is therefore
/// unsatisfiable by ANY valid DEFLATE stream and is rejected before a single
/// byte is inflated — a forged length can never drive the allocation.
pub const DEFLATE_MAX_EXPANSION: u64 = 1032;

/// The trusted content address (blake3 hex) of the committed registry MANIFEST
/// `.prx` — the registry root, the ONE content-address that lives in Rust. The
/// fail-closed registry gate admits the embedded bytes only if they re-derive to
/// this value. Regenerated (with the `.prx` itself) by
/// `cargo test -p pr4xis-domains -- --ignored regenerate_praxis_registry_prx`.
///
/// This is the SINGLE definition; the runtime
/// ([`registry_prx`](super::registry_prx)) re-exports it, and the build script
/// `#[path]`-includes this module, so there is no build/runtime twin to drift.
pub const PRAXIS_REGISTRY_ROOT_HEX: &str =
    "480a8f74081b5ba732b13e0184e846a623f4085b17dea0b86ad814acc2cd5d94";

// =============================================================================
// LEB128 varint + length-prefixed blob framing
// =============================================================================

/// Append a LEB128 varint.
pub fn put_varint(out: &mut Vec<u8>, mut n: u64) {
    loop {
        let b = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(b);
            break;
        }
        out.push(b | 0x80);
    }
}

/// Append `bytes` length-prefixed (LEB128 varint length + raw bytes).
pub fn put_blob(out: &mut Vec<u8>, bytes: &[u8]) {
    put_varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

/// Read one LEB128 varint with full bounds checking — the panic-proof reader the
/// gate relies on (a truncated buffer is `Err`, never a panic).
pub fn get_varint(buf: &[u8], pos: &mut usize) -> Result<u64, String> {
    let mut n: u64 = 0;
    let mut shift = 0u32;
    loop {
        let b = *buf
            .get(*pos)
            .ok_or_else(|| "varint runs past end of buffer".to_string())?;
        *pos += 1;
        n |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            return Ok(n);
        }
        shift += 7;
        if shift >= 64 {
            return Err("varint length overflow".to_string());
        }
    }
}

/// Read one length-prefixed blob with full bounds checking — the panic-proof
/// reader the gate relies on (a truncated archive is `Err`, never a panic).
pub fn get_blob<'a>(buf: &'a [u8], pos: &mut usize) -> Result<&'a [u8], String> {
    let len = get_varint(buf, pos)? as usize;
    let end = pos
        .checked_add(len)
        .filter(|&e| e <= buf.len())
        .ok_or_else(|| "blob runs past end of buffer".to_string())?;
    let b = &buf[*pos..end];
    *pos = end;
    Ok(b)
}

// =============================================================================
// Raw-source envelope: the self-describing, content-addressed blob
// =============================================================================

/// How a raw-source envelope's payload blob encodes the source bytes — the
/// enumerated, self-described transport written into every envelope (a cited
/// codec concept, never a bare magic number in the stream).
///
/// - `Identity`: the payload IS the source bytes, verbatim.
/// - `Deflate`: the payload is a raw RFC 1951 DEFLATE stream (Deutsch 1996) of
///   the source bytes. Raw DEFLATE, NOT the RFC 1952 gzip wrapper, whose
///   `MTIME`/`OS` member-header fields would make the emitted bytes — and so the
///   `[compact_archive_signatures]` content address — nondeterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadEncoding {
    /// Payload = source bytes, verbatim.
    Identity,
    /// Payload = raw RFC 1951 DEFLATE stream of the source bytes.
    Deflate,
}

impl PayloadEncoding {
    /// The varint wire tag written into the envelope for this encoding.
    #[must_use]
    pub const fn wire_tag(self) -> u64 {
        match self {
            PayloadEncoding::Identity => 0,
            PayloadEncoding::Deflate => 1,
        }
    }

    /// The encoding a wire tag names — `None` for an unknown tag (fail-closed:
    /// the decoder refuses an envelope it cannot name, never guesses).
    #[must_use]
    pub const fn from_wire_tag(tag: u64) -> Option<Self> {
        match tag {
            0 => Some(PayloadEncoding::Identity),
            1 => Some(PayloadEncoding::Deflate),
            _ => None,
        }
    }
}

/// A parsed (but not yet payload-materialized) raw-source envelope — the
/// structural view [`parse_envelope`] produces and [`materialize_payload`]
/// consumes, borrowing every field from the envelope buffer.
pub struct ParsedEnvelope<'a> {
    /// The registry key's source name (`blob(name)`).
    pub name: &'a [u8],
    /// The registry key's source version (`blob(version)`).
    pub version: &'a [u8],
    /// The declared payload transport.
    pub encoding: PayloadEncoding,
    /// The declared SOURCE byte length (what the payload materializes to).
    pub decoded_len: u64,
    /// The encoded source bytes (`blob(payload)`).
    pub payload: &'a [u8],
}

/// Parse a version-2 envelope's frame, fail-closed and TOTAL: an unknown format
/// version, an unknown encoding tag, a truncated/overrunning blob, trailing
/// garbage, or a structurally unsatisfiable declared length is `Err`, never a
/// panic and never a guess.
pub fn parse_envelope(buf: &[u8]) -> Result<ParsedEnvelope<'_>, String> {
    let mut pos = 0usize;
    let format_version = get_varint(buf, &mut pos)?;
    if format_version != RAW_SOURCE_ENVELOPE_FORMAT_VERSION {
        return Err(format!(
            "unsupported raw-source envelope format version {format_version} (this build reads \
             version {RAW_SOURCE_ENVELOPE_FORMAT_VERSION}) — refusing to guess at the layout"
        ));
    }
    let name = get_blob(buf, &mut pos)?;
    let version = get_blob(buf, &mut pos)?;
    let tag = get_varint(buf, &mut pos)?;
    let encoding = PayloadEncoding::from_wire_tag(tag)
        .ok_or_else(|| format!("unknown payload-encoding wire tag {tag}"))?;
    let decoded_len = get_varint(buf, &mut pos)?;
    let payload = get_blob(buf, &mut pos)?;
    if pos != buf.len() {
        return Err(format!(
            "{} trailing byte(s) after the payload blob",
            buf.len() - pos
        ));
    }
    match encoding {
        // Identity: the declared length IS the payload length, by definition.
        PayloadEncoding::Identity => {
            if decoded_len != payload.len() as u64 {
                return Err(format!(
                    "identity payload length {} ≠ declared decoded length {decoded_len}",
                    payload.len()
                ));
            }
        }
        // Deflate: the decompression-bomb guard. No valid RFC 1951 stream
        // expands beyond ~1032:1 (Gailly & Adler, zlib Technical Details), so a
        // declared length above payload × DEFLATE_MAX_EXPANSION is a forgery —
        // rejected before any allocation is sized from it.
        PayloadEncoding::Deflate => {
            if decoded_len > (payload.len() as u64).saturating_mul(DEFLATE_MAX_EXPANSION) {
                return Err(format!(
                    "declared decoded length {decoded_len} exceeds the RFC 1951 maximum \
                     expansion ({DEFLATE_MAX_EXPANSION}:1) of a {}-byte payload",
                    payload.len()
                ));
            }
        }
    }
    Ok(ParsedEnvelope {
        name,
        version,
        encoding,
        decoded_len,
        payload,
    })
}

/// Materialize a parsed envelope's payload back into the SOURCE bytes: zero-copy
/// (`Cow::Borrowed`) for `Identity`, inflated (`Cow::Owned`) for `Deflate`.
/// Fail-closed and total: a DEFLATE stream that does not inflate cleanly to
/// EXACTLY the declared length is `Err`, never a panic — the inflater is bounded
/// by the declared length, so it can neither run away nor silently truncate.
///
/// Canonicality guard: the inflater must consume the WHOLE payload blob. A valid
/// RFC 1951 stream self-terminates at its final block (Deutsch 1996 §3.2.3,
/// BFINAL), so a convenience inflater would silently ignore any bytes appended
/// after it INSIDE the blob — admitting infinitely many distinct envelopes that
/// decode to the same source bytes. The stateful core inflate reports consumed
/// input, and a blob with unconsumed tail bytes is `Err`.
pub fn materialize_payload<'a>(env: &ParsedEnvelope<'a>) -> Result<Cow<'a, [u8]>, String> {
    match env.encoding {
        PayloadEncoding::Identity => Ok(Cow::Borrowed(env.payload)),
        PayloadEncoding::Deflate => {
            let mut out = alloc::vec![0u8; env.decoded_len as usize];
            let mut state = miniz_oxide::inflate::core::DecompressorOxide::new();
            // No `TINFL_FLAG_HAS_MORE_INPUT`: the blob is the entire stream.
            // Non-wrapping output: `out` is sized to the full declared length.
            let (status, consumed, produced) = miniz_oxide::inflate::core::decompress(
                &mut state,
                env.payload,
                &mut out,
                0,
                miniz_oxide::inflate::core::inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF,
            );
            if status != miniz_oxide::inflate::TINFLStatus::Done {
                return Err(format!("DEFLATE payload does not inflate: {status:?}"));
            }
            if consumed != env.payload.len() {
                return Err(format!(
                    "DEFLATE stream ended with {} unconsumed byte(s) inside the payload blob \
                     — non-canonical envelope refused",
                    env.payload.len() - consumed
                ));
            }
            if produced as u64 != env.decoded_len {
                return Err(format!(
                    "DEFLATE payload inflated to {produced} byte(s), declared decoded length is {}",
                    env.decoded_len
                ));
            }
            Ok(Cow::Owned(out))
        }
    }
}

/// Encode the raw source `bytes` into the portable succinct envelope
/// (see [`RAW_SOURCE_ENVELOPE_FORMAT_VERSION`] for the exact layout).
/// Dependency-light LEB128 framing + optional raw RFC 1951 payload — so the
/// layout is stable across toolchains and targets and the content address taken
/// over it is portable and DETERMINISTIC.
///
/// `requested` is the transport the caller asks for; the emitter is
/// STORE-IF-SMALLER: a `Deflate` whose stream is not strictly smaller than the
/// source bytes is downgraded to `Identity` (so already-compressed payloads
/// never grow, and `encode` stays a pure deterministic function of its inputs).
#[must_use]
pub fn encode_raw_source(
    name: &str,
    version: &str,
    bytes: &[u8],
    requested: PayloadEncoding,
) -> Vec<u8> {
    let (encoding, payload): (PayloadEncoding, Cow<'_, [u8]>) = match requested {
        PayloadEncoding::Identity => (PayloadEncoding::Identity, Cow::Borrowed(bytes)),
        PayloadEncoding::Deflate => {
            // `BestCompression` is miniz_oxide's named zlib level-9 setting —
            // these envelopes are compressed ONCE at emit and decoded on every
            // load, so the slowest-emit/smallest-artifact point is correct.
            let deflated = miniz_oxide::deflate::compress_to_vec(
                bytes,
                miniz_oxide::deflate::CompressionLevel::BestCompression as u8,
            );
            if deflated.len() < bytes.len() {
                (PayloadEncoding::Deflate, Cow::Owned(deflated))
            } else {
                // STORE-IF-SMALLER: deflate gained nothing — store verbatim.
                (PayloadEncoding::Identity, Cow::Borrowed(bytes))
            }
        }
    };
    let mut out = Vec::with_capacity(payload.len() + name.len() + version.len() + 24);
    put_varint(&mut out, RAW_SOURCE_ENVELOPE_FORMAT_VERSION);
    put_blob(&mut out, name.as_bytes());
    put_blob(&mut out, version.as_bytes());
    put_varint(&mut out, encoding.wire_tag());
    put_varint(&mut out, bytes.len() as u64);
    put_blob(&mut out, &payload);
    out
}

// =============================================================================
// Registry manifest envelope: `blob(praxis.toml) blob(praxis.lock)`
// =============================================================================

/// Encode the registry manifest into the portable succinct envelope:
/// `put_blob(praxis.toml bytes) put_blob(praxis.lock bytes)`. Dependency-free
/// LEB128 framing — the content address taken over these bytes is the registry
/// root [`PRAXIS_REGISTRY_ROOT_HEX`] pins.
#[must_use]
pub fn encode_registry(toml: &[u8], lock: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(toml.len() + lock.len() + 16);
    put_blob(&mut out, toml);
    put_blob(&mut out, lock);
    out
}

/// Decode a registry envelope back into `(praxis.toml text, praxis.lock text)` —
/// the exact inverse of [`encode_registry`]. Fail-closed on a truncated /
/// malformed envelope or non-UTF-8 payload.
pub fn decode_registry(buf: &[u8]) -> Result<(String, String), String> {
    let mut pos = 0usize;
    let toml = get_blob(buf, &mut pos)?;
    let lock = get_blob(buf, &mut pos)?;
    let toml = core::str::from_utf8(toml)
        .map_err(|e| format!("registry .prx praxis.toml payload is not UTF-8: {e}"))?
        .to_string();
    let lock = core::str::from_utf8(lock)
        .map_err(|e| format!("registry .prx praxis.lock payload is not UTF-8: {e}"))?
        .to_string();
    Ok((toml, lock))
}

// =============================================================================
// blake3 content-address verification helpers (the build-script gate)
//
// The runtime verifies through the domain `ContentAddress` / `raw_hash::verify`
// path (multi-algorithm, from a trusted `LockDigest`); the build script carries
// only the `blake3` hash dep and verifies through these. Both compute the SAME
// blake3 content address (`ContentAddress::of(x).to_hex() == blake3(x) hex`), so
// the pin space is shared. Dead in the lib (which uses the domain path), hence
// the per-item `allow(dead_code)`; live in the build script.
// =============================================================================

/// Recompute the blake3 content address of `bytes` as 64-char lowercase hex —
/// the same value `pr4xis_runtime::address::ContentAddress::of(bytes).to_hex()`
/// yields (that type is blake3-backed). The build script's content gate.
#[allow(dead_code)]
#[must_use]
pub(crate) fn content_address_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Gate a committed raw-source `.prx` against a trusted `blake3:`-tagged pin,
/// then decode it to its source TEXT (UTF-8) — the build-script twin of the
/// runtime raw-source gate, fail-closed: a pin that is not `blake3:`-tagged, an
/// address mismatch, a malformed envelope, or a non-UTF-8 payload is `Err` and
/// NO text is returned. The pin algorithm comes from the pin's TAG (never the
/// artifact); the build script carries only the blake3 hash dep, so any other
/// (or absent) tag is refused BY NAME here — never a silent always-mismatch of a
/// non-blake3 pin against a blake3 recomputation.
#[allow(dead_code)]
pub(crate) fn blake3_gated_raw_source_text(
    prx: &[u8],
    pin: &str,
    key: &str,
) -> Result<String, String> {
    let expected_hex = pin.strip_prefix("blake3:").ok_or_else(|| {
        format!(
            "committed .prx pin for `{key}` is not a `blake3:`-tagged digest (`{pin}`): the \
             build-side gate verifies only blake3 pins — extend it (and its hash build-deps) \
             before pinning a raw source with another algorithm"
        )
    })?;
    let found_hex = content_address_hex(prx);
    if found_hex != expected_hex {
        return Err(format!(
            "committed .prx for `{key}` hash mismatch: praxis.lock pins {expected_hex}, archive \
             carries {found_hex} — refusing to feed codegen"
        ));
    }
    let env = parse_envelope(prx).map_err(|e| format!("committed .prx for `{key}`: {e}"))?;
    let bytes =
        materialize_payload(&env).map_err(|e| format!("committed .prx for `{key}`: {e}"))?;
    String::from_utf8(bytes.into_owned())
        .map_err(|e| format!("committed .prx for `{key}` payload is not UTF-8: {e}"))
}

/// Gate the committed registry MANIFEST `.prx` against the BAKED
/// [`PRAXIS_REGISTRY_ROOT_HEX`] root, then decode it into
/// `(praxis.toml text, praxis.lock text)` — the build-script twin of the runtime
/// `load_registry_manifest` baked-root gate. Runs precisely when the workspace
/// `praxis.lock` is unavailable (the published/unpacked crate), so it chains
/// from the baked root, not the lock it would populate. Fail-closed: a tampered
/// or stale registry `.prx` is refused (`Err`), never fed to codegen.
#[allow(dead_code)]
pub(crate) fn blake3_gated_registry(prx: &[u8]) -> Result<(String, String), String> {
    let found_hex = content_address_hex(prx);
    if found_hex != PRAXIS_REGISTRY_ROOT_HEX {
        return Err(format!(
            "registry .prx root mismatch: baked PRAXIS_REGISTRY_ROOT_HEX is \
             {PRAXIS_REGISTRY_ROOT_HEX}, archive carries {found_hex} — refusing to feed codegen"
        ));
    }
    decode_registry(prx)
}
