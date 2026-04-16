//! Network fetch + identity verification + disk write — the materialization
//! half of the data-provisioning layer.
//!
//! This module is gated behind the `fetch` feature because `ureq` + `flate2`
//! don't build on wasm32 without additional getrandom configuration, and the
//! WASM crate depends on pr4xis-domains with default features. Keeping these
//! deps optional means the default build stays wasm-compatible.
//!
//! The module exposes a small surface:
//!
//! - `FetchOptions` — the knobs (force re-fetch, check-only, offline)
//! - `FetchOutcome` — the structured result of a single fetch
//! - `fetch_entry(entry, opts, workspace_root)` — the per-entry work
//! - `fetch_all(opts, workspace_root)` — every entry in `DATA_SOURCES`
//!
//! The implementation is intentionally linear and does not retry or cache.
//! Every call is a clean `HTTP GET → verify → write`. Re-running after a
//! successful fetch short-circuits via a local re-verification unless
//! `force` is set, so invocations are idempotent.
//!
//! **Flag precedence.** `--check` is a read-only mode and always wins:
//! `--check --force` ignores `force` and only verifies the current file.
//! `--offline` blocks network access: if a local file exists it is still
//! verified, and verification failure is reported as `VerificationFailed`
//! (not `MissingAndOffline`, which is reserved for actually-absent files).

use super::ontology::RegistryEntry;
use super::registry::{DATA_SOURCES, resolve_identity};
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
    AlreadyVerified { name: &'static str },
    /// Fetched fresh bytes and wrote them to disk; every claim verified.
    Fetched {
        name: &'static str,
        path: PathBuf,
        bytes: usize,
    },
    /// Local file exists but at least one identity claim failed. The file
    /// is kept on disk; callers decide whether to retry or give up.
    VerificationFailed {
        name: &'static str,
        path: PathBuf,
        reason: String,
    },
    /// File is absent and `check` was set, so nothing was fetched.
    MissingAndCheckOnly { name: &'static str, path: PathBuf },
    /// File is absent and `offline` was set, so we couldn't fetch.
    MissingAndOffline { name: &'static str, path: PathBuf },
    /// Network or disk error during fetch.
    FetchError { name: &'static str, reason: String },
}

impl FetchOutcome {
    /// Whether this outcome should be treated as a success by the CLI.
    pub fn is_ok(&self) -> bool {
        matches!(
            self,
            FetchOutcome::AlreadyVerified { .. } | FetchOutcome::Fetched { .. }
        )
    }
}

/// Fetch every registered entry. Runs through every entry regardless of
/// per-entry failures so the caller gets a full report — one outcome per
/// registered dataset, in registry order.
pub fn fetch_all(opts: FetchOptions, workspace_root: &Path) -> Vec<FetchOutcome> {
    DATA_SOURCES
        .iter()
        .map(|entry| fetch_entry(entry, opts, workspace_root))
        .collect()
}

/// Fetch a single entry. See module docs for the contract and flag
/// precedence (`check` dominates `force`; `offline` never changes to
/// `MissingAndOffline` when a local file is present).
pub fn fetch_entry(
    entry: &'static RegistryEntry,
    opts: FetchOptions,
    workspace_root: &Path,
) -> FetchOutcome {
    let path = workspace_root.join(entry.local_path);

    // `--check` is read-only and always wins over `--force`.
    if opts.check {
        return if path.exists() {
            match verify_local(entry, &path) {
                Ok(()) => FetchOutcome::AlreadyVerified { name: entry.name },
                Err(reason) => FetchOutcome::VerificationFailed {
                    name: entry.name,
                    path,
                    reason,
                },
            }
        } else {
            FetchOutcome::MissingAndCheckOnly {
                name: entry.name,
                path,
            }
        };
    }

    if path.exists() && !opts.force {
        return match verify_local(entry, &path) {
            Ok(()) => FetchOutcome::AlreadyVerified { name: entry.name },
            // Local file exists but verification failed: report the
            // failure reason. `offline` does NOT mask it — the file is
            // not missing, it's unverified.
            Err(reason) if opts.offline => FetchOutcome::VerificationFailed {
                name: entry.name,
                path,
                reason,
            },
            Err(_) => do_fetch(entry, &path),
        };
    }

    if opts.offline {
        return FetchOutcome::MissingAndOffline {
            name: entry.name,
            path,
        };
    }

    do_fetch(entry, &path)
}

// --------------------------------------------------------------------------
// Internal: download + verify + write
// --------------------------------------------------------------------------

fn do_fetch(entry: &'static RegistryEntry, path: &Path) -> FetchOutcome {
    let bytes = match download(entry.remote_location) {
        Ok(b) => b,
        Err(e) => {
            return FetchOutcome::FetchError {
                name: entry.name,
                reason: format!("download failed: {e}"),
            };
        }
    };

    let bytes = if entry.gzipped {
        match gunzip(&bytes) {
            Ok(b) => b,
            Err(e) => {
                return FetchOutcome::FetchError {
                    name: entry.name,
                    reason: format!("gunzip failed: {e}"),
                };
            }
        }
    } else {
        bytes
    };

    if let Err(reason) = verify_bytes(entry, &bytes) {
        return FetchOutcome::VerificationFailed {
            name: entry.name,
            path: path.to_path_buf(),
            reason,
        };
    }

    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        return FetchOutcome::FetchError {
            name: entry.name,
            reason: format!("mkdir {}: {e}", parent.display()),
        };
    }
    if let Err(e) = fs::write(path, &bytes) {
        return FetchOutcome::FetchError {
            name: entry.name,
            reason: format!("write {}: {e}", path.display()),
        };
    }

    FetchOutcome::Fetched {
        name: entry.name,
        path: path.to_path_buf(),
        bytes: bytes.len(),
    }
}

fn download(url: &str) -> anyhow::Result<Vec<u8>> {
    let resp = ureq::get(url).call().map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut buf = Vec::new();
    resp.into_reader().read_to_end(&mut buf)?;
    Ok(buf)
}

fn gunzip(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    use flate2::read::GzDecoder;
    let mut decoder = GzDecoder::new(bytes);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}

fn verify_local(entry: &'static RegistryEntry, path: &Path) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    verify_bytes(entry, &bytes)
}

/// Run every declared identity claim against the given bytes. All claims
/// must verify (`CompositeRequiresAll`); the first failure wins the
/// rejection. A stub extractor returns `Unverifiable`, which is a
/// rejection here — the pipeline is fail-closed, so a claim we cannot
/// evaluate is treated as a failure, not a skip. This keeps
/// `VerificationFailClosed` honest.
fn verify_bytes(entry: &'static RegistryEntry, bytes: &[u8]) -> Result<(), String> {
    let identity = resolve_identity(entry.name)
        .ok_or_else(|| format!("no resolved identity for {}", entry.name))?;

    let mut verified = 0usize;
    for claim in &identity.0 {
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
                element: "Lexicon",
                attribute: "version",
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
                element: "Lexicon",
                attribute: "version",
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
    fn verify_bytes_fails_on_unknown_entry() {
        let bogus = RegistryEntry {
            name: "not-in-registry",
            description: "test",
            remote_location: "",
            local_path: "",
            content_type: super::super::ontology::ContentType::Binary,
            identity: crate::formal::meta::artifact_identity::ontology::CompositeIdentity(
                Vec::new(),
            ),
            gzipped: false,
        };
        let bogus_static: &'static RegistryEntry = Box::leak(Box::new(bogus));
        let result = verify_bytes(bogus_static, b"bytes");
        assert!(result.is_err());
    }

    #[test]
    fn verify_bytes_passes_on_real_wordnet_entry() {
        let wordnet = super::super::registry::by_name("wordnet").expect("wordnet registered");
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let path = workspace_root.join(wordnet.local_path);
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
        let wordnet = super::super::registry::by_name("wordnet").unwrap();
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
        let wordnet = super::super::registry::by_name("wordnet").unwrap();
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
        assert!(FetchOutcome::AlreadyVerified { name: "x" }.is_ok());
        assert!(
            FetchOutcome::Fetched {
                name: "x",
                path: PathBuf::new(),
                bytes: 0,
            }
            .is_ok()
        );
        assert!(
            !FetchOutcome::MissingAndCheckOnly {
                name: "x",
                path: PathBuf::new(),
            }
            .is_ok()
        );
        assert!(
            !FetchOutcome::MissingAndOffline {
                name: "x",
                path: PathBuf::new(),
            }
            .is_ok()
        );
        assert!(
            !FetchOutcome::VerificationFailed {
                name: "x",
                path: PathBuf::new(),
                reason: String::new(),
            }
            .is_ok()
        );
        assert!(
            !FetchOutcome::FetchError {
                name: "x",
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
}
