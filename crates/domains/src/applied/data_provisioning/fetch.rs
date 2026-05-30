//! Network fetch + identity verification + disk write — the materialization
//! half of the data-provisioning layer.
//!
//! TLS uses `rustls` with the `rustls-rustcrypto` `CryptoProvider`. Under
//! the workspace's default + `fetch` feature set, no `ring` / `cc` build
//! path is enabled — `cargo tree --workspace -e build,normal --invert ring`
//! returns empty. The `Cargo.lock` still names `ring` as a gated potential
//! dep of `rustls-webpki` (it's behind webpki's `ring` feature, which we
//! don't turn on); resolution declares the entry, the resolver doesn't
//! activate it.
//!
//! The module is gated behind the `fetch` feature because `ureq` +
//! `flate2` + std-only I/O don't fit the WASM build path (the WASM crate
//! depends on pr4xis-domains with default features). Keeping these deps
//! optional means the default build stays wasm-compatible.
//!
//! The module exposes a small surface:
//!
//! - `FetchOptions` — the knobs (force re-fetch, check-only, offline)
//! - `FetchOutcome` — the structured result of a single fetch
//! - `fetch_entry(entry, opts, workspace_root)` — the per-entry work
//! - `fetch_all(opts, workspace_root)` — every entry in `DATA_SOURCES`
//!
//! Every call is a clean `HTTP GET → verify → write`. Re-running after a
//! successful fetch short-circuits via a local re-verification unless
//! `force` is set, so invocations are idempotent. The downloader wraps
//! each GET in an idempotent retry harness (RFC 9110 §9.2.2: GET is
//! idempotent and a client SHOULD retry on connection-class failure)
//! with an exponential backoff schedule (Jacobson 1988, "Congestion
//! Avoidance and Control", SIGCOMM 1988 §3) so transient TCP RSTs
//! mid-download recover instead of aborting the whole `pr4xis update`
//! run.
//!
//! **Flag precedence.** `--check` is a read-only mode and always wins:
//! `--check --force` ignores `force` and only verifies the current file.
//! `--offline` blocks network access: if a local file exists it is still
//! verified, and verification failure is reported as `VerificationFailed`
//! (not `MissingAndOffline`, which is reserved for actually-absent files).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::RegistryEntry;
use super::registry::data_sources;
use crate::formal::meta::artifact_identity::ontology::{
    ClaimData, IdentityClaim, IdentityConcept, VerificationResult,
};
use crate::formal::meta::artifact_identity::schemes::{raw_hash, xml_element_attribute};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Options controlling a fetch run.
#[derive(Debug, Clone, Copy, Default)]
pub struct FetchOptions {
    /// Re-fetch even when a valid local copy exists.
    pub force: bool,
    /// Verify the existing local copy without touching the network. If a
    /// file is missing, report it as missing rather than fetching.
    pub check: bool,
    /// Refuse to touch the network. Combined with `check=false`, this errors
    /// out on anything that would otherwise fetch.
    pub offline: bool,
}

/// Outcome of a single entry's fetch attempt.
#[derive(Debug)]
pub enum FetchOutcome {
    /// Local copy exists and every identity claim verifies.
    AlreadyVerified { name: String },
    /// Fetched fresh bytes and wrote them to disk; every claim verified.
    Fetched {
        name: String,
        path: PathBuf,
        bytes: usize,
    },
    /// Local file exists but at least one identity claim failed. The file
    /// is kept on disk; callers decide whether to retry or give up.
    VerificationFailed {
        name: String,
        path: PathBuf,
        reason: String,
    },
    /// File is absent and `check` was set, so nothing was fetched.
    MissingAndCheckOnly { name: String, path: PathBuf },
    /// File is absent and `offline` was set, so we couldn't fetch.
    MissingAndOffline { name: String, path: PathBuf },
    /// Network or disk error during fetch.
    FetchError { name: String, reason: String },
    /// Source has only `ClaimData::Stub` identity claims — registered
    /// in `praxis.toml` but no lock hash pinned yet (the loader for
    /// its content type isn't wired). The fetcher skips it because
    /// there's no way to verify what comes back; `DecoderTotalityPerKind`
    /// and `LockManifestAgreement` already treat the same set as
    /// "registered but not yet loadable" pending the materialization
    /// machinery. Reported as success so CI doesn't block on the
    /// existence of a documented-but-deferred entry.
    Skipped { name: String, reason: String },
}

impl FetchOutcome {
    /// Whether this outcome should be treated as a success by the CLI.
    pub fn is_ok(&self) -> bool {
        matches!(
            self,
            FetchOutcome::AlreadyVerified { .. }
                | FetchOutcome::Fetched { .. }
                | FetchOutcome::Skipped { .. }
        )
    }
}

/// Fetch every registered entry. Runs through every entry regardless of
/// per-entry failures so the caller gets a full report — one outcome per
/// registered dataset, in registry order.
pub fn fetch_all(opts: FetchOptions, workspace_root: &Path) -> Vec<FetchOutcome> {
    data_sources()
        .iter()
        .map(|entry| fetch_entry(entry, opts, workspace_root))
        .collect()
}

/// Fetch a single entry. See module docs for the contract and flag
/// precedence (`check` dominates `force`; `offline` never changes to
/// `MissingAndOffline` when a local file is present).
pub fn fetch_entry(
    entry: &RegistryEntry,
    opts: FetchOptions,
    workspace_root: &Path,
) -> FetchOutcome {
    // Stub-only identity: registered but not yet loadable. The fetcher
    // has nothing to verify and the upstream URL is documentation, not
    // a verified artifact. Skip without touching the network — same
    // treatment the `DecoderTotalityPerKind` and `LockManifestAgreement`
    // axioms apply to these entries.
    if entry.identity.is_stub_only() {
        return FetchOutcome::Skipped {
            name: entry.name.clone(),
            reason: "stub identity — registered in praxis.toml, no lock hash yet".into(),
        };
    }

    let path = workspace_root.join(entry.local_path());

    // `--check` is read-only and always wins over `--force`.
    if opts.check {
        return if path.exists() {
            match verify_local(entry, &path) {
                Ok(()) => FetchOutcome::AlreadyVerified {
                    name: entry.name.clone(),
                },
                Err(reason) => FetchOutcome::VerificationFailed {
                    name: entry.name.clone(),
                    path,
                    reason,
                },
            }
        } else {
            FetchOutcome::MissingAndCheckOnly {
                name: entry.name.clone(),
                path,
            }
        };
    }

    if path.exists() && !opts.force {
        return match verify_local(entry, &path) {
            Ok(()) => FetchOutcome::AlreadyVerified {
                name: entry.name.clone(),
            },
            // Local file exists but verification failed: report the
            // failure reason. `offline` does NOT mask it — the file is
            // not missing, it's unverified.
            Err(reason) if opts.offline => FetchOutcome::VerificationFailed {
                name: entry.name.clone(),
                path,
                reason,
            },
            Err(_) => do_fetch(entry, &path),
        };
    }

    if opts.offline {
        return FetchOutcome::MissingAndOffline {
            name: entry.name.clone(),
            path,
        };
    }

    do_fetch(entry, &path)
}

// --------------------------------------------------------------------------
// Internal: download + verify + write
// --------------------------------------------------------------------------

fn do_fetch(entry: &RegistryEntry, path: &Path) -> FetchOutcome {
    let bytes = match download(&entry.url) {
        Ok(b) => b,
        Err(e) => {
            return FetchOutcome::FetchError {
                name: entry.name.clone(),
                reason: format!("download failed: {e}"),
            };
        }
    };

    let bytes = if entry.transport_gzip() {
        match gunzip(&bytes) {
            Ok(b) => b,
            Err(e) => {
                return FetchOutcome::FetchError {
                    name: entry.name.clone(),
                    reason: format!("gunzip failed: {e}"),
                };
            }
        }
    } else if entry.zipped() {
        match unzip_single_xml(&bytes) {
            Ok(b) => b,
            Err(e) => {
                return FetchOutcome::FetchError {
                    name: entry.name.clone(),
                    reason: format!("unzip failed: {e}"),
                };
            }
        }
    } else {
        bytes
    };

    if let Err(reason) = verify_bytes(entry, &bytes) {
        return FetchOutcome::VerificationFailed {
            name: entry.name.clone(),
            path: path.to_path_buf(),
            reason,
        };
    }

    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        return FetchOutcome::FetchError {
            name: entry.name.clone(),
            reason: format!("mkdir {}: {e}", parent.display()),
        };
    }
    if let Err(e) = fs::write(path, &bytes) {
        return FetchOutcome::FetchError {
            name: entry.name.clone(),
            reason: format!("write {}: {e}", path.display()),
        };
    }

    FetchOutcome::Fetched {
        name: entry.name.clone(),
        path: path.to_path_buf(),
        bytes: bytes.len(),
    }
}

/// Install the pure-Rust `rustls-rustcrypto` provider as the rustls process
/// default. Called once per process from `download()`; subsequent calls are
/// no-ops via `std::sync::Once`. `ureq` is built with the
/// `rustls-no-provider` feature, so a provider must be installed before any
/// TLS handshake — without this, the first HTTPS request panics inside
/// rustls with "no process-level CryptoProvider available".
///
/// `install_default` returns `Err` if some other library in the same
/// process already installed a provider (e.g. the embedding application
/// or another rustls user). We ignore that `Err` — the existing provider
/// will be used. We do NOT panic, because the rustls invariant is "a
/// provider is installed," not "this specific provider is installed."
fn install_crypto_provider() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        // Ignore `Err` from install_default — it returns Err only when a
        // provider is already installed, which satisfies our invariant.
        let _ = rustls::crypto::CryptoProvider::install_default(rustls_rustcrypto::provider());
    });
}

/// Max body bytes accepted by `download()` (the compressed wire size).
/// ureq 3 defaults to 10 MiB; the largest registered dataset (WordNet 2025
/// XML, gzipped on the wire) is ~12 MiB compressed, and other planned
/// corpora can exceed that. 1 GiB is high enough to cover the foreseeable
/// registry while still bounding the worst-case allocation if a server
/// returns a runaway response.
const MAX_BODY_BYTES: u64 = 1024 * 1024 * 1024;

/// Max decompressed bytes accepted by `gunzip()`. The wire-side
/// `MAX_BODY_BYTES` only caps the compressed payload; without a separate
/// decompressed limit, a small zip bomb (e.g. 1 KiB gzip expanding to
/// many GiB) could exhaust memory before identity verification runs.
/// WordNet 2025 XML decompresses to ~89 MiB; 2 GiB is high enough to
/// cover the foreseeable registry while still bounding worst-case
/// allocation.
const MAX_DECOMPRESSED_BYTES: u64 = 2 * 1024 * 1024 * 1024;

fn download(url: &str) -> anyhow::Result<Vec<u8>> {
    install_crypto_provider();
    with_retry(|| download_once(url))
}

/// Single attempt at downloading `url`. The retry harness in
/// [`with_retry`] wraps this for transient-error resilience.
fn download_once(url: &str) -> anyhow::Result<Vec<u8>> {
    let buf = ureq::get(url)
        .call()
        .map_err(|e| anyhow::anyhow!("{e}"))?
        .body_mut()
        .with_config()
        .limit(MAX_BODY_BYTES)
        .read_to_vec()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(buf)
}

/// Total number of attempts (one initial + N retries) for an
/// idempotent download. Three is a small constant: caps worst-case
/// wall-clock at the [`backoff_for_attempt`] sum (≈ 3 s of sleep
/// across the three attempts plus three request RTTs), large enough
/// to recover from the transient TCP RST class we observed in CI
/// (e.g. `Peer disconnected` mid-download).
pub(crate) const RETRY_ATTEMPTS: u32 = 3;

/// Backoff before retry `attempt` (1-indexed against the next attempt
/// — `backoff_for_attempt(2)` is the sleep before attempt 2). Doubles
/// per attempt starting at one second, the canonical
/// exponential-backoff schedule. Single client → no thundering-herd
/// concern → no jitter.
pub(crate) fn backoff_for_attempt(attempt: u32) -> core::time::Duration {
    core::time::Duration::from_secs(1u64 << (attempt - 2))
}

/// Run an idempotent fetch closure with [`RETRY_ATTEMPTS`] tries and
/// exponential backoff between failures.
///
/// HTTP GET is idempotent (RFC 9110 §9.2.2: "GET, HEAD, OPTIONS, PUT,
/// DELETE, and TRACE are defined as idempotent... A client SHOULD
/// retry an idempotent request if the request is determined to be
/// unsuccessful due to a connection issue"), so retrying a failed
/// download is safe by HTTP semantics. Exponential backoff between
/// attempts follows the canonical schedule (Jacobson 1988,
/// "Congestion Avoidance and Control", SIGCOMM 1988 §3, repurposed
/// here for application-layer retry as documented in
/// AWS Architecture Blog, Brooker 2015, "Exponential Backoff And
/// Jitter").
///
/// The harness retries on *any* error — discriminating
/// transient-vs-permanent at the `ureq::Error` variant level would
/// be more precise but at the cost of coupling this file to ureq's
/// internal taxonomy. A permanent 4xx still terminates in at most
/// [`RETRY_ATTEMPTS`] iterations; the worst-case extra wall-clock is
/// the sum of [`backoff_for_attempt`] over the retries.
pub(crate) fn with_retry<F, T>(mut f: F) -> anyhow::Result<T>
where
    F: FnMut() -> anyhow::Result<T>,
{
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=RETRY_ATTEMPTS {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                if attempt < RETRY_ATTEMPTS {
                    let delay = backoff_for_attempt(attempt + 1);
                    eprintln!(
                        "  [retry] attempt {attempt}/{RETRY_ATTEMPTS} failed: {e} \
                         (sleeping {delay:?} before retry)"
                    );
                    std::thread::sleep(delay);
                }
                last_err = Some(e);
            }
        }
    }
    Err(last_err.expect("at least one attempt was made"))
}

fn gunzip(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    // Cap the reader at MAX_DECOMPRESSED_BYTES + 1 so we can distinguish
    // "exactly at the limit" from "exceeded the limit." If decode produces
    // more than MAX_DECOMPRESSED_BYTES bytes, the truncated tail is still
    // present, so the size check below catches it.
    let limited = std::io::Read::take(GzDecoder::new(bytes), MAX_DECOMPRESSED_BYTES + 1);
    let mut decoder = limited;
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    if out.len() as u64 > MAX_DECOMPRESSED_BYTES {
        anyhow::bail!(
            "decompressed payload exceeds {} bytes (got > {})",
            MAX_DECOMPRESSED_BYTES,
            out.len()
        );
    }
    Ok(out)
}

/// Extract the single `.xml` member from a PKZIP archive.
///
/// USC release-point title archives (`xml_usc<NN>@<rp>.zip`) hold
/// exactly one `usc<NN>.xml`. This reads the **central directory**
/// (PKWARE APPNOTE.TXT 6.3.10 §4.3.12) for authoritative member sizes
/// and offsets — robust against streaming "data descriptors"
/// (§4.3.9), where the *local* header sizes are zeroed. Members that
/// are stored (method 0) or DEFLATE-compressed (method 8, RFC 1951)
/// are supported; any other method, ZIP64, or an archive without
/// exactly one `.xml` member is a hard error. The choice must be
/// deterministic because the extracted bytes feed sha256 verification,
/// so "more than one `.xml`" fails closed rather than guessing.
fn unzip_single_xml(zip: &[u8]) -> anyhow::Result<Vec<u8>> {
    let eocd = find_eocd(zip)?;
    let total_entries = read_u16(zip, eocd + 10)? as usize;
    let cd_offset = read_u32(zip, eocd + 16)? as usize;

    // Walk the central directory; select the sole `.xml` member.
    let mut p = cd_offset;
    let mut chosen: Option<(usize, u16, usize, usize)> = None; // (local_off, method, comp, uncomp)
    for _ in 0..total_entries {
        if read_u32(zip, p)? != 0x0201_4b50 {
            anyhow::bail!("malformed central-directory header at offset {p}");
        }
        let method = read_u16(zip, p + 10)?;
        let comp_size = read_u32(zip, p + 20)? as usize;
        let uncomp_size = read_u32(zip, p + 24)? as usize;
        let name_len = read_u16(zip, p + 28)? as usize;
        let extra_len = read_u16(zip, p + 30)? as usize;
        let comment_len = read_u16(zip, p + 32)? as usize;
        let local_off = read_u32(zip, p + 42)? as usize;
        let name_start = p + 46;
        let name_end = name_start
            .checked_add(name_len)
            .filter(|e| *e <= zip.len())
            .ok_or_else(|| anyhow::anyhow!("central-directory name out of bounds"))?;
        if zip[name_start..name_end].ends_with(b".xml") {
            if chosen.is_some() {
                anyhow::bail!(
                    "archive has more than one .xml member; refusing to choose non-deterministically"
                );
            }
            chosen = Some((local_off, method, comp_size, uncomp_size));
        }
        p = name_end + extra_len + comment_len;
    }

    let (local_off, method, comp_size, uncomp_size) =
        chosen.ok_or_else(|| anyhow::anyhow!("archive contains no .xml member"))?;

    if read_u32(zip, local_off)? != 0x0403_4b50 {
        anyhow::bail!("malformed local-file header at offset {local_off}");
    }
    let l_name_len = read_u16(zip, local_off + 26)? as usize;
    let l_extra_len = read_u16(zip, local_off + 28)? as usize;
    let data_start = local_off + 30 + l_name_len + l_extra_len;
    let data_end = data_start
        .checked_add(comp_size)
        .filter(|e| *e <= zip.len())
        .ok_or_else(|| anyhow::anyhow!("compressed data out of bounds"))?;
    let comp = &zip[data_start..data_end];

    match method {
        0 => Ok(comp.to_vec()),          // stored
        8 => inflate(comp, uncomp_size), // DEFLATE (RFC 1951)
        m => anyhow::bail!("unsupported zip compression method {m} (only stored/deflate)"),
    }
}

/// Locate the End Of Central Directory record (APPNOTE §4.3.16) by
/// scanning backward for its signature `0x06054b50`, allowing for the
/// variable-length trailing comment (max 65535 bytes).
fn find_eocd(zip: &[u8]) -> anyhow::Result<usize> {
    if zip.len() < 22 {
        anyhow::bail!("not a zip archive (shorter than the 22-byte EOCD record)");
    }
    let scan_floor = zip.len().saturating_sub(22 + 0xFFFF);
    let mut i = zip.len() - 22;
    loop {
        if zip[i..].starts_with(&[0x50, 0x4B, 0x05, 0x06]) {
            return Ok(i);
        }
        if i == scan_floor {
            anyhow::bail!("no End Of Central Directory record found (not a zip, or ZIP64)");
        }
        i -= 1;
    }
}

/// DEFLATE-decompress (RFC 1951) under the same size cap as `gunzip`,
/// asserting the result matches the archive's declared uncompressed
/// size (the central-directory value).
fn inflate(comp: &[u8], expected: usize) -> anyhow::Result<Vec<u8>> {
    use flate2::read::DeflateDecoder;
    let limited = std::io::Read::take(DeflateDecoder::new(comp), MAX_DECOMPRESSED_BYTES + 1);
    let mut out = Vec::new();
    let mut decoder = limited;
    decoder.read_to_end(&mut out)?;
    if out.len() as u64 > MAX_DECOMPRESSED_BYTES {
        anyhow::bail!("decompressed payload exceeds {MAX_DECOMPRESSED_BYTES} bytes");
    }
    if out.len() != expected {
        anyhow::bail!(
            "decompressed size {} != central-directory declared {expected}",
            out.len()
        );
    }
    Ok(out)
}

/// Little-endian u16 read with bounds checking (no panic on malformed input).
fn read_u16(b: &[u8], at: usize) -> anyhow::Result<u16> {
    let end = at
        .checked_add(2)
        .filter(|e| *e <= b.len())
        .ok_or_else(|| anyhow::anyhow!("u16 read out of bounds at {at}"))?;
    Ok(u16::from_le_bytes([b[at], b[end - 1]]))
}

/// Little-endian u32 read with bounds checking (no panic on malformed input).
fn read_u32(b: &[u8], at: usize) -> anyhow::Result<u32> {
    let end = at
        .checked_add(4)
        .filter(|e| *e <= b.len())
        .ok_or_else(|| anyhow::anyhow!("u32 read out of bounds at {at}"))?;
    let s = &b[at..end];
    Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn verify_local(entry: &RegistryEntry, path: &Path) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    verify_bytes(entry, &bytes)
}

/// Run every declared identity claim against the given bytes. All claims
/// must verify (`CompositeRequiresAll`); the first failure wins the
/// rejection. A stub extractor returns `Unverifiable`, which is a
/// rejection here — the pipeline is fail-closed, so a claim we cannot
/// evaluate is treated as a failure, not a skip. This keeps
/// `VerificationFailClosed` honest.
fn verify_bytes(entry: &RegistryEntry, bytes: &[u8]) -> Result<(), String> {
    let mut verified = 0usize;
    for claim in &entry.identity.0 {
        match run_extractor(claim, bytes) {
            VerificationResult::Verified(_) => verified += 1,
            VerificationResult::Mismatch { expected, actual } => {
                return Err(format!(
                    "{:?} claim mismatch: expected {expected}, got {actual}",
                    claim.concept
                ));
            }
            VerificationResult::Unverifiable { reason } => {
                return Err(format!("{:?} claim unverifiable: {reason}", claim.concept));
            }
        }
    }

    if verified == 0 {
        return Err(format!(
            "no claims verified for {} — identity is empty",
            entry.name
        ));
    }
    Ok(())
}

/// Dispatch a single claim to its concrete extractor. Two real ones
/// (RawHash, XmlElementAttribute); everything else is a stub that returns
/// `Unverifiable`.
fn run_extractor(claim: &IdentityClaim, bytes: &[u8]) -> VerificationResult {
    match claim.concept {
        IdentityConcept::RawHash => match &claim.data {
            ClaimData::Sha256(_) => raw_hash::verify(claim, bytes),
            _ => VerificationResult::Unverifiable {
                reason: "RawHash claim requires ClaimData::Sha256".into(),
            },
        },
        IdentityConcept::XmlElementAttribute => match &claim.data {
            ClaimData::XmlAttribute { .. } => xml_element_attribute::verify(claim, bytes),
            _ => VerificationResult::Unverifiable {
                reason: "XmlElementAttribute claim requires ClaimData::XmlAttribute".into(),
            },
        },
        _ => VerificationResult::Unverifiable {
            reason: format!("{:?} extractor is not yet wired in fetch", claim.concept),
        },
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for fetch dispatch. Network is not exercised — every test
    //! goes through the non-network branches (check / offline / verify).

    use super::*;
    use proptest::prelude::*;
    use sha2::{Digest, Sha256};

    const SAMPLE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="oewn" label="English WordNet" language="en" email="t@e" license="CC" version="2025" url="https://en-word.net/">
    <LexicalEntry id="e-dog-n"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="s1" synset="d1"/></LexicalEntry>
    <Synset id="d1" ili="i1" partOfSpeech="n"><Definition>a dog</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;

    fn sample_sha256() -> String {
        let mut h = Sha256::new();
        h.update(SAMPLE_XML.as_bytes());
        hex::encode(h.finalize())
    }

    #[test]
    fn run_extractor_raw_hash_verifies() {
        let claim = IdentityClaim {
            concept: IdentityConcept::RawHash,
            data: ClaimData::Sha256(sample_sha256()),
        };
        let result = run_extractor(&claim, SAMPLE_XML.as_bytes());
        assert!(matches!(result, VerificationResult::Verified(_)));
    }

    #[test]
    fn run_extractor_raw_hash_mismatch() {
        let claim = IdentityClaim {
            concept: IdentityConcept::RawHash,
            data: ClaimData::Sha256(
                "0000000000000000000000000000000000000000000000000000000000000000".into(),
            ),
        };
        let result = run_extractor(&claim, SAMPLE_XML.as_bytes());
        assert!(matches!(result, VerificationResult::Mismatch { .. }));
    }

    #[test]
    fn run_extractor_xml_attribute_verifies() {
        let claim = IdentityClaim {
            concept: IdentityConcept::XmlElementAttribute,
            data: ClaimData::XmlAttribute {
                element: "Lexicon".into(),
                attribute: "version".into(),
                expected: "2025".into(),
            },
        };
        let result = run_extractor(&claim, SAMPLE_XML.as_bytes());
        assert!(matches!(result, VerificationResult::Verified(_)));
    }

    #[test]
    fn run_extractor_xml_attribute_mismatch() {
        let claim = IdentityClaim {
            concept: IdentityConcept::XmlElementAttribute,
            data: ClaimData::XmlAttribute {
                element: "Lexicon".into(),
                attribute: "version".into(),
                expected: "2099".into(),
            },
        };
        let result = run_extractor(&claim, SAMPLE_XML.as_bytes());
        assert!(matches!(result, VerificationResult::Mismatch { .. }));
    }

    #[test]
    fn run_extractor_stub_concept_is_unverifiable() {
        let claim = IdentityClaim {
            concept: IdentityConcept::Doi,
            data: ClaimData::Stub {
                reason: "test".into(),
            },
        };
        let result = run_extractor(&claim, b"anything");
        assert!(matches!(result, VerificationResult::Unverifiable { .. }));
    }

    #[test]
    fn run_extractor_wrong_data_shape_is_unverifiable() {
        let claim = IdentityClaim {
            concept: IdentityConcept::RawHash,
            data: ClaimData::Stub {
                reason: "wrong shape".into(),
            },
        };
        let result = run_extractor(&claim, b"bytes");
        assert!(matches!(result, VerificationResult::Unverifiable { .. }));
    }

    #[test]
    fn verify_bytes_fails_on_empty_identity() {
        use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
        let bogus = RegistryEntry {
            name: "not-in-registry".into(),
            version: "0".into(),
            kind: SourceTaxonomyConcept::Statute,
            url: String::new(),
            description: None,
            identity: crate::formal::meta::artifact_identity::ontology::CompositeIdentity(
                Vec::new(),
            ),
        };
        let result = verify_bytes(&bogus, b"bytes");
        assert!(result.is_err());
    }

    #[test]
    fn verify_bytes_passes_on_real_wordnet_entry() {
        let wordnet =
            super::super::registry::by_name("english_wordnet").expect("english_wordnet registered");
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let path = workspace_root.join(wordnet.local_path());
        if !path.exists() {
            eprintln!("skipping: wordnet file not on disk at {}", path.display());
            return;
        }
        let bytes = fs::read(&path).expect("read wordnet file");
        let result = verify_bytes(wordnet, &bytes);
        assert!(
            result.is_ok(),
            "real wordnet bytes should verify against pinned identity: {:?}",
            result
        );
    }

    #[test]
    fn fetch_entry_check_only_missing_returns_missing() {
        let tmp = tempdir_path();
        let wordnet = super::super::registry::by_name("english_wordnet").unwrap();
        let opts = FetchOptions {
            check: true,
            force: false,
            offline: false,
        };
        let outcome = fetch_entry(wordnet, opts, &tmp);
        assert!(matches!(outcome, FetchOutcome::MissingAndCheckOnly { .. }));
    }

    #[test]
    fn fetch_entry_offline_missing_returns_offline() {
        let tmp = tempdir_path();
        let wordnet = super::super::registry::by_name("english_wordnet").unwrap();
        let opts = FetchOptions {
            check: false,
            force: false,
            offline: true,
        };
        let outcome = fetch_entry(wordnet, opts, &tmp);
        assert!(matches!(outcome, FetchOutcome::MissingAndOffline { .. }));
    }

    #[test]
    fn fetch_outcome_is_ok_only_for_success_variants() {
        assert!(FetchOutcome::AlreadyVerified { name: "x".into() }.is_ok());
        assert!(
            FetchOutcome::Fetched {
                name: "x".into(),
                path: PathBuf::new(),
                bytes: 0,
            }
            .is_ok()
        );
        assert!(
            !FetchOutcome::MissingAndCheckOnly {
                name: "x".into(),
                path: PathBuf::new(),
            }
            .is_ok()
        );
        assert!(
            !FetchOutcome::MissingAndOffline {
                name: "x".into(),
                path: PathBuf::new(),
            }
            .is_ok()
        );
        assert!(
            !FetchOutcome::VerificationFailed {
                name: "x".into(),
                path: PathBuf::new(),
                reason: String::new(),
            }
            .is_ok()
        );
        assert!(
            !FetchOutcome::FetchError {
                name: "x".into(),
                reason: String::new(),
            }
            .is_ok()
        );
    }

    /// Isolated temp directory per test, under the system tempdir. No
    /// `tempfile` crate dependency — we just use an ad-hoc pid+nanos name
    /// and skip cleanup (tests don't write here anyway).
    fn tempdir_path() -> PathBuf {
        let base = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        base.join(format!(
            "pr4xis-fetch-test-{}-{}",
            std::process::id(),
            nanos
        ))
    }

    proptest! {
        /// Random byte payloads verified against a freshly computed sha256
        /// must always yield `Verified`. Guards `run_extractor`'s RawHash
        /// arm against subtle hashing bugs.
        #[test]
        fn prop_raw_hash_round_trip(bytes in prop::collection::vec(any::<u8>(), 0..1024)) {
            let mut h = Sha256::new();
            h.update(&bytes);
            let hex = hex::encode(h.finalize());
            let claim = IdentityClaim {
                concept: IdentityConcept::RawHash,
                data: ClaimData::Sha256(hex),
            };
            let result = run_extractor(&claim, &bytes);
            let is_verified = matches!(result, VerificationResult::Verified(_));
            prop_assert!(is_verified);
        }

        /// Random byte payloads against a frozen wrong hash must always
        /// yield `Mismatch`. Guards against false positives.
        #[test]
        fn prop_raw_hash_detects_wrong_hash(bytes in prop::collection::vec(any::<u8>(), 1..1024)) {
            let claim = IdentityClaim {
                concept: IdentityConcept::RawHash,
                data: ClaimData::Sha256(
                    "0000000000000000000000000000000000000000000000000000000000000000".into(),
                ),
            };
            let result = run_extractor(&claim, &bytes);
            let is_mismatch = matches!(result, VerificationResult::Mismatch { .. });
            prop_assert!(is_mismatch);
        }
    }

    // --- ZIP extraction (PKWARE APPNOTE.TXT 6.3.10; DEFLATE RFC 1951) ---

    /// Assemble a minimal single-member PKZIP archive (local header +
    /// data + one central-directory record + EOCD). CRC is left zero —
    /// the reader keys off the central directory, not the CRC.
    fn build_single_entry_zip(name: &[u8], method: u16, data: &[u8], uncomp: u32) -> Vec<u8> {
        let comp = data.len() as u32;
        let nlen = name.len() as u16;
        let mut z = Vec::new();
        // Local file header (offset 0).
        z.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
        z.extend_from_slice(&20u16.to_le_bytes()); // version needed
        z.extend_from_slice(&0u16.to_le_bytes()); // flags
        z.extend_from_slice(&method.to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes()); // mod time
        z.extend_from_slice(&0u16.to_le_bytes()); // mod date
        z.extend_from_slice(&0u32.to_le_bytes()); // crc32
        z.extend_from_slice(&comp.to_le_bytes());
        z.extend_from_slice(&uncomp.to_le_bytes());
        z.extend_from_slice(&nlen.to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes()); // extra len
        z.extend_from_slice(name);
        z.extend_from_slice(data);
        let cd_offset = z.len() as u32;
        // Central-directory header.
        z.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
        z.extend_from_slice(&20u16.to_le_bytes()); // version made by
        z.extend_from_slice(&20u16.to_le_bytes()); // version needed
        z.extend_from_slice(&0u16.to_le_bytes()); // flags
        z.extend_from_slice(&method.to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes()); // mod time
        z.extend_from_slice(&0u16.to_le_bytes()); // mod date
        z.extend_from_slice(&0u32.to_le_bytes()); // crc32
        z.extend_from_slice(&comp.to_le_bytes());
        z.extend_from_slice(&uncomp.to_le_bytes());
        z.extend_from_slice(&nlen.to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes()); // extra len
        z.extend_from_slice(&0u16.to_le_bytes()); // comment len
        z.extend_from_slice(&0u16.to_le_bytes()); // disk number
        z.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
        z.extend_from_slice(&0u32.to_le_bytes()); // external attrs
        z.extend_from_slice(&0u32.to_le_bytes()); // local header offset
        z.extend_from_slice(name);
        let cd_size = z.len() as u32 - cd_offset;
        // End Of Central Directory.
        z.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
        z.extend_from_slice(&0u16.to_le_bytes()); // disk number
        z.extend_from_slice(&0u16.to_le_bytes()); // cd start disk
        z.extend_from_slice(&1u16.to_le_bytes()); // entries on this disk
        z.extend_from_slice(&1u16.to_le_bytes()); // total entries
        z.extend_from_slice(&cd_size.to_le_bytes());
        z.extend_from_slice(&cd_offset.to_le_bytes());
        z.extend_from_slice(&0u16.to_le_bytes()); // comment len
        z
    }

    #[test]
    fn unzip_extracts_sole_stored_xml() {
        let payload = b"<uscDoc/>";
        let zip = build_single_entry_zip(b"usc99.xml", 0, payload, payload.len() as u32);
        assert_eq!(unzip_single_xml(&zip).unwrap(), payload);
    }

    #[test]
    fn unzip_extracts_deflated_xml() {
        use flate2::Compression;
        use flate2::write::DeflateEncoder;
        use std::io::Write;
        let payload = b"<uscDoc><section>SOX 1514A</section></uscDoc>";
        let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
        enc.write_all(payload).unwrap();
        let deflated = enc.finish().unwrap();
        let zip = build_single_entry_zip(b"usc18.xml", 8, &deflated, payload.len() as u32);
        assert_eq!(unzip_single_xml(&zip).unwrap(), payload);
    }

    #[test]
    fn unzip_errors_when_no_xml_member() {
        let zip = build_single_entry_zip(b"readme.txt", 0, b"hi", 2);
        assert!(unzip_single_xml(&zip).is_err());
    }

    #[test]
    fn unzip_errors_on_non_zip_bytes() {
        assert!(unzip_single_xml(b"<not a zip>").is_err());
    }

    // ------------------------------------------------------------------
    // `with_retry` — RFC 9110 §9.2.2 idempotent-retry harness for the
    // GET-based downloader. Tests exercise the retry semantics with a
    // counter closure so no network is touched. The retry attempts use
    // a real `std::thread::sleep`, so the tests pay the
    // [`backoff_for_attempt`] backoff (≤ 3 s worst case).
    // ------------------------------------------------------------------

    use std::cell::Cell;

    #[test]
    fn with_retry_returns_on_first_success() {
        let calls = Cell::new(0u32);
        let result: anyhow::Result<u32> = with_retry(|| {
            calls.set(calls.get() + 1);
            Ok(42)
        });
        assert_eq!(result.expect("first attempt succeeds"), 42);
        assert_eq!(calls.get(), 1, "no retries needed on initial success");
    }

    #[test]
    fn with_retry_recovers_from_transient_failure() {
        // Simulate a transient TCP RST: first attempt fails, second
        // attempt succeeds. The harness must propagate the success
        // and report the value from the surviving attempt.
        let calls = Cell::new(0u32);
        let result: anyhow::Result<&'static str> = with_retry(|| {
            calls.set(calls.get() + 1);
            if calls.get() < 2 {
                Err(anyhow::anyhow!("io: Peer disconnected"))
            } else {
                Ok("recovered")
            }
        });
        assert_eq!(result.expect("retry recovers"), "recovered");
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn with_retry_propagates_last_error_after_attempts_exhausted() {
        // A permanent failure (e.g. 404) doesn't recover. The harness
        // must report the last error after [`RETRY_ATTEMPTS`] tries.
        let calls = Cell::new(0u32);
        let result: anyhow::Result<()> = with_retry(|| {
            calls.set(calls.get() + 1);
            Err(anyhow::anyhow!("attempt {} failed", calls.get()))
        });
        let err = result.expect_err("permanent failure must propagate");
        assert_eq!(
            calls.get(),
            RETRY_ATTEMPTS,
            "harness must exhaust exactly RETRY_ATTEMPTS attempts"
        );
        assert!(
            err.to_string()
                .contains(&format!("attempt {} failed", RETRY_ATTEMPTS)),
            "last attempt's error must be the one returned; got: {err}"
        );
    }

    #[test]
    fn backoff_schedule_is_exponential() {
        // Jacobson 1988 exponential schedule — doubles per attempt.
        // The harness sleeps `backoff_for_attempt(attempt+1)` BEFORE
        // attempt (attempt+1); concrete schedule for the three-attempt
        // run: no sleep before attempt 1, 1 s before attempt 2, 2 s
        // before attempt 3. Total ≤ 3 s.
        assert_eq!(backoff_for_attempt(2), core::time::Duration::from_secs(1));
        assert_eq!(backoff_for_attempt(3), core::time::Duration::from_secs(2));
        assert_eq!(backoff_for_attempt(4), core::time::Duration::from_secs(4));
    }
}
