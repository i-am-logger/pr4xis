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
//! - `check_entry(entry, workspace_root)` — identity check without network
//!
//! The implementation is intentionally linear and does not retry or cache.
//! Every call is a clean `HTTP GET → verify → write`. Re-running after a
//! successful fetch short-circuits via `check_entry` unless `force` is set,
//! so invocations are idempotent.

use super::ontology::RegistryEntry;
use super::registry::{DATA_SOURCES, by_name, resolve_identity};
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

/// Fetch every registered entry. Stops on the first error only if every
/// subsequent entry also errors — otherwise runs through all entries and
/// returns a vector of outcomes so the caller can report each one.
pub fn fetch_all(opts: FetchOptions, workspace_root: &Path) -> Vec<FetchOutcome> {
    DATA_SOURCES
        .iter()
        .map(|entry| fetch_entry(entry, opts, workspace_root))
        .collect()
}

/// Fetch a single entry. See module docs for the contract.
pub fn fetch_entry(
    entry: &'static RegistryEntry,
    opts: FetchOptions,
    workspace_root: &Path,
) -> FetchOutcome {
    let path = workspace_root.join(entry.local_path);

    if path.exists() && !opts.force {
        return match verify_local(entry, &path) {
            Ok(()) => FetchOutcome::AlreadyVerified { name: entry.name },
            Err(reason) if opts.check => FetchOutcome::VerificationFailed {
                name: entry.name,
                path,
                reason,
            },
            Err(_) if opts.offline => FetchOutcome::MissingAndOffline {
                name: entry.name,
                path,
            },
            Err(_) => do_fetch(entry, &path),
        };
    }

    if opts.check {
        return FetchOutcome::MissingAndCheckOnly {
            name: entry.name,
            path,
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

/// Identity check only — no network, no disk write. Used by the CLI's
/// `--check` mode when the caller just wants a report.
pub fn check_entry(name: &str, workspace_root: &Path) -> FetchOutcome {
    let Some(entry) = by_name(name) else {
        return FetchOutcome::FetchError {
            name: "",
            reason: format!("unknown dataset: {name}"),
        };
    };
    let path = workspace_root.join(entry.local_path);
    if !path.exists() {
        return FetchOutcome::MissingAndCheckOnly {
            name: entry.name,
            path,
        };
    }
    match verify_local(entry, &path) {
        Ok(()) => FetchOutcome::AlreadyVerified { name: entry.name },
        Err(reason) => FetchOutcome::VerificationFailed {
            name: entry.name,
            path,
            reason,
        },
    }
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
/// must verify (`CompositeRequiresAll`); the first failure wins the rejection.
/// Claims with stub extractors are skipped — they can't pass or fail today,
/// so they don't block fetching, but also don't count as verification.
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
