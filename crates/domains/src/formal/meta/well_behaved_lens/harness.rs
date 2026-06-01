//! Round-trip verification harness for every registered
//! [`super::WellBehavedLens`].
//!
//! # What the harness does
//!
//! For every lens registration in the [`LENS_REGISTRATIONS`]
//! distributed slice the harness:
//!
//! 1. Resolves the source's expected on-disk path via the registry.
//! 2. Reads the bytes (if present — sources awaiting `pr4xis update`
//!    are reported as `SourceNotOnDisk`, not as a hard failure).
//! 3. Dispatches on the lens's [`RoundTripFidelity`]:
//!    `RawBytesComplementFloor` runs the canonical-form
//!    [`super::WellBehavedLens::assert_put_get_law`];
//!    `ByteExactGraphFaithful` runs the strict
//!    [`super::WellBehavedLens::assert_byte_exact_law`] (Foster,
//!    Greenwald, Moore, Pierce & Schmitt 2007 *ACM TOPLAS* 29(3) §2.2
//!    well-behaved lens laws).
//! 4. Computes the SHA-256 — canonical-form for the FLOOR path, raw
//!    bytes for the byte-exact path.
//! 5. Compares against the pinned signature from
//!    [`crate::applied::data_provisioning::registry`] —
//!    `[canonical_signatures]` or `[byte_exact_signatures]` respectively.
//!
//! Every outcome is reported via [`HarnessOutcome`]:
//!
//! - `Verified` — lens law holds AND the computed signature matches
//!   the lock.
//! - `LawHoldsSignatureUnpinned` — lens law holds, but the source
//!   isn't yet round-trip-pinned in `praxis.lock`.
//! - `SignatureMismatch` — lens law holds but the computed signature
//!   differs from the pinned one. Catches drift in the lens
//!   implementation or in the underlying canonical form.
//! - `LawViolated` — `put(get(s))` is not canonically equal to `s`.
//!   The ontology does not yet capture the full structure of the
//!   source.
//! - `SourceNotOnDisk` — `pr4xis update` hasn't fetched the source
//!   yet. Reported but does not fail CI; the lock entry is still
//!   present so future verification will catch any drift the moment
//!   the bytes land.
//!
//! # Registration
//!
//! Each lens implementation registers itself with the harness via
//! the [`crate::register_lens`] macro:
//!
//! ```text
//! crate::register_lens!(USC_TITLE_18, "usc_title_18", "pl-119-90", UslmXmlLens);
//! ```
//!
//! The macro emits a `static` decorated with `#[linkme(distributed_slice)]`
//! so the harness picks it up at link time — no central list to
//! edit, no risk of forgetting to wire a new lens in.
//!
//! # CI gate
//!
//! The [`RoundTripHarnessAllVerified`] axiom (verified by the test
//! in this module) fails whenever any entry reports
//! `SignatureMismatch` or `LawViolated`. `SourceNotOnDisk` and
//! `LawHoldsSignatureUnpinned` are non-fatal — they signal pending
//! work without blocking other commits.
//!
//! # Literature
//!
//! - Foster, Greenwald, Moore, Pierce & Schmitt (2007). "Combinators
//!   for Bidirectional Tree Transformations", *ACM TOPLAS* 29(3)
//!   Article 17 §2.2.
//! - Dolstra (2006). "The Purely Functional Software Deployment
//!   Model", PhD thesis Utrecht §3 — content-addressed storage as
//!   drift-detection vehicle.
//! - NIST FIPS 180-4 (2015). *Secure Hash Standard* §6.2 (SHA-256).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::Axiom;
use sha2::{Digest, Sha256};

use super::lens_trait::{LensLawFailure, RoundTripFidelity};
use crate::applied::data_provisioning::registry::{
    by_name_version, lock_byte_exact_signature, lock_canonical_signature,
};

// =============================================================================
// Registration: one entry per (lens type, source key) pair.
// =============================================================================

/// Type-erased registration of a [`super::WellBehavedLens`] applied to a
/// specific praxis-toml source. Populated via the
/// [`crate::register_lens`] macro and discovered by the harness at
/// link time through the `linkme` distributed slice.
pub struct LensRegistration {
    /// `"name@version"` matching the lock-file key.
    pub key: &'static str,
    /// The source's `name` field from `praxis.toml`.
    pub source_name: &'static str,
    /// The source's `version` field from `praxis.toml`.
    pub source_version: &'static str,
    /// Run [`super::WellBehavedLens::assert_put_get_law`] on `bytes`
    /// (the canonical-form PutGet law — used for
    /// [`RoundTripFidelity::RawBytesComplementFloor`] sources).
    pub assert_law: fn(&[u8]) -> Result<(), LensLawFailure>,
    /// Run [`super::WellBehavedLens::assert_byte_exact_law`] on `bytes`
    /// (the strict byte-exact PutGet law — used for
    /// [`RoundTripFidelity::ByteExactGraphFaithful`] sources).
    pub assert_byte_exact: fn(&[u8]) -> Result<(), LensLawFailure>,
    /// Compute the canonical-form SHA-256 of `bytes`.
    pub signature: fn(&[u8]) -> Result<[u8; 32], String>,
    /// The round-trip fidelity this lens guarantees — selects which
    /// PutGet law and which `praxis.lock` signature section the
    /// harness checks.
    pub fidelity: RoundTripFidelity,
}

/// Distributed slice of every [`LensRegistration`] in the workspace
/// (native targets). Each [`crate::register_lens`] invocation appends one
/// entry; the harness iterates the slice at runtime.
///
/// On wasm32, linkme is unsupported (same constraint as
/// `pr4xis::ontology::registry`'s `VOCABULARIES` etc.), so the slice is
/// absent and [`lens_registrations`] returns empty. The round-trip
/// harness is a native build-time / CI audit tool; the wasm runtime never
/// runs it.
#[cfg(not(target_arch = "wasm32"))]
#[::linkme::distributed_slice]
pub static LENS_REGISTRATIONS: [LensRegistration] = [..];

/// The registered lenses — auto-collected on native, empty on wasm32.
#[cfg(not(target_arch = "wasm32"))]
pub fn lens_registrations() -> &'static [LensRegistration] {
    &LENS_REGISTRATIONS
}

/// The registered lenses (wasm32 stub) — empty; linkme is unsupported.
#[cfg(target_arch = "wasm32")]
pub fn lens_registrations() -> &'static [LensRegistration] {
    &[]
}

/// Register a [`super::WellBehavedLens`] implementation for a specific
/// praxis-toml source.
///
/// # Example
///
/// ```text
/// // In `crates/domains/src/social/software/markup/xml/uslm/lens/mod.rs`:
/// crate::register_lens!(USC_TITLE_18, "usc_title_18", "pl-119-90", UslmXmlLens);
/// ```
#[macro_export]
macro_rules! register_lens {
    ($static_ident:ident, $name:literal, $version:literal, $lens_ty:ty $(,)?) => {
        // Native only — linkme's distributed_slice is unsupported on
        // wasm32 (mirrors `pr4xis::ontology::registry`). The round-trip
        // harness these feed is a native CI/audit tool.
        #[cfg(not(target_arch = "wasm32"))]
        #[::linkme::distributed_slice(
            $crate::formal::meta::well_behaved_lens::harness::LENS_REGISTRATIONS
        )]
        static $static_ident: $crate::formal::meta::well_behaved_lens::harness::LensRegistration =
            $crate::formal::meta::well_behaved_lens::harness::LensRegistration {
                key: concat!($name, "@", $version),
                source_name: $name,
                source_version: $version,
                assert_law:
                    <$lens_ty as $crate::formal::meta::well_behaved_lens::WellBehavedLens>::assert_put_get_law,
                assert_byte_exact:
                    <$lens_ty as $crate::formal::meta::well_behaved_lens::WellBehavedLens>::assert_byte_exact_law,
                signature: |b| {
                    <$lens_ty as $crate::formal::meta::well_behaved_lens::WellBehavedLens>::signature(b)
                        .map_err(|e| alloc::format!("{}", e))
                },
                fidelity:
                    <$lens_ty as $crate::formal::meta::well_behaved_lens::WellBehavedLens>::FIDELITY,
            };
    };
}

// =============================================================================
// Harness runner.
// =============================================================================

/// One row of the harness's report — the outcome of running the
/// PutGet law + signature check against a single source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessResult {
    /// `"name@version"`.
    pub key: String,
    /// Outcome of the round-trip + signature check.
    pub outcome: HarnessOutcome,
}

/// What the harness found for one source. `Verified` and the two
/// pending states (`SourceNotOnDisk`, `LawHoldsSignatureUnpinned`)
/// are non-fatal; the other variants fail the CI gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessOutcome {
    /// Lens law holds and the computed signature matches
    /// `[canonical_signatures]` in `praxis.lock`.
    Verified,
    /// Source bytes are not on disk yet. `pr4xis update <name>` lands
    /// them; the harness re-runs on the next test invocation. Not a
    /// hard failure.
    SourceNotOnDisk { path: String },
    /// Lens law holds, but no `canonical_signature` is pinned for
    /// this source yet. The computed signature is included so the
    /// human pinning the lock has the value ready.
    LawHoldsSignatureUnpinned { computed_sha256_hex: String },
    /// Lens law holds, but the computed canonical signature differs
    /// from `praxis.lock`'s pinned value. Either the lens output
    /// drifted, the canonical form drifted, or the source bytes are
    /// stale.
    SignatureMismatch {
        expected: String,
        computed_sha256_hex: String,
    },
    /// `put(get(bytes))` does not canonicalize to the same bytes as
    /// `bytes` — the ontology does not yet capture the full
    /// structure of the source (PutGet law violated).
    LawViolated(LensLawFailure),
    /// `put(get(bytes)) != bytes` byte-for-byte — a byte-affecting
    /// concrete-syntax decision is not yet a first-class node in the
    /// ontology (byte-exact PutGet law violated, M4.ι / #186). Only
    /// produced for [`RoundTripFidelity::ByteExactGraphFaithful`]
    /// sources.
    ByteLawViolated(LensLawFailure),
    /// Reading the source file failed for some reason other than
    /// "file not on disk" (permission error, malformed UTF-8 in
    /// path, etc.).
    LoadError { path: String, message: String },
    /// Source not in the praxis registry — the registration key
    /// points at a `(name, version)` praxis.toml does not declare.
    SourceNotRegistered,
}

impl HarnessOutcome {
    /// Whether this outcome fails the CI gate.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            HarnessOutcome::SignatureMismatch { .. }
                | HarnessOutcome::LawViolated(_)
                | HarnessOutcome::ByteLawViolated(_)
                | HarnessOutcome::LoadError { .. }
                | HarnessOutcome::SourceNotRegistered
        )
    }
}

/// Run the harness over every entry in [`LENS_REGISTRATIONS`].
/// Returns one [`HarnessResult`] per registration, ordered by `key`.
#[must_use]
pub fn run_round_trip_harness() -> Vec<HarnessResult> {
    let mut out: Vec<HarnessResult> = lens_registrations().iter().map(check_one).collect();
    out.sort_by(|a, b| a.key.cmp(&b.key));
    out
}

/// Print one line per registration to stderr — TOML-formatted for the
/// `[canonical_signatures]` section of `praxis.lock` when the source's
/// PutGet law holds but its signature hasn't been pinned yet, and a
/// `key: Outcome` line otherwise. Surface for the human workflow of
/// pinning new lens signatures: capture stderr, paste matching lines
/// into `praxis.lock`.
///
/// This is procedural tooling, not a `#[test]` — pairs with
/// [[feedback-never-use-ignore]]: helpers belong in callable
/// functions and CLI subcommands, never as `#[ignore]`d tests.
pub fn dump_unpinned_signatures() {
    for r in run_round_trip_harness() {
        match &r.outcome {
            HarnessOutcome::LawHoldsSignatureUnpinned {
                computed_sha256_hex,
            } => {
                eprintln!(
                    "\"{}\" = \"{}\"  # pin in praxis.lock [canonical_signatures]",
                    r.key, computed_sha256_hex
                );
            }
            other => eprintln!("{}: {:?}", r.key, other),
        }
    }
}

fn check_one(reg: &LensRegistration) -> HarnessResult {
    let outcome = match resolve_source_bytes(reg) {
        Ok(SourceBytes::Loaded { bytes, .. }) => verify_loaded_bytes(reg, &bytes),
        Ok(SourceBytes::NotOnDisk { path }) => HarnessOutcome::SourceNotOnDisk { path },
        Ok(SourceBytes::LoadError { path, message }) => HarnessOutcome::LoadError { path, message },
        Err(SourceLookupError::NotRegistered) => HarnessOutcome::SourceNotRegistered,
    };
    HarnessResult {
        key: reg.key.to_string(),
        outcome,
    }
}

fn verify_loaded_bytes(reg: &LensRegistration, bytes: &[u8]) -> HarnessOutcome {
    match reg.fidelity {
        RoundTripFidelity::RawBytesComplementFloor => verify_canonical(reg, bytes),
        RoundTripFidelity::ByteExactGraphFaithful => verify_byte_exact(reg, bytes),
    }
}

/// Canonical-form PutGet law + `[canonical_signatures]` pin — the FLOOR
/// path for sources with no published canonical form (PDF) and every
/// lens not yet migrated to graph-faithful byte-exactness.
fn verify_canonical(reg: &LensRegistration, bytes: &[u8]) -> HarnessOutcome {
    // Step 1: PutGet law (canonical-form equality).
    if let Err(failure) = (reg.assert_law)(bytes) {
        return HarnessOutcome::LawViolated(failure);
    }
    // Step 2: canonical signature.
    let computed = match (reg.signature)(bytes) {
        Ok(sig) => sig,
        Err(message) => {
            return HarnessOutcome::LoadError {
                path: String::new(),
                message: format!("signature computation failed: {message}"),
            };
        }
    };
    let computed_hex = hex_of(&computed);

    // Step 3: compare against praxis.lock [canonical_signatures].
    match lock_canonical_signature(reg.source_name, reg.source_version) {
        None => HarnessOutcome::LawHoldsSignatureUnpinned {
            computed_sha256_hex: computed_hex,
        },
        Some(expected) if expected == computed_hex => HarnessOutcome::Verified,
        Some(expected) => HarnessOutcome::SignatureMismatch {
            expected: expected.to_string(),
            computed_sha256_hex: computed_hex,
        },
    }
}

/// Strict byte-exact PutGet law + `[byte_exact_signatures]` pin — the
/// graph-faithful path (no constant-complement). The pinned value is
/// the SHA-256 of the *raw* source bytes; `put(get(b)) == b` makes the
/// round-tripped output hash equal to it.
fn verify_byte_exact(reg: &LensRegistration, bytes: &[u8]) -> HarnessOutcome {
    // Step 1: byte-exact PutGet law (raw byte equality, no complement).
    if let Err(failure) = (reg.assert_byte_exact)(bytes) {
        return HarnessOutcome::ByteLawViolated(failure);
    }
    // Step 2: raw-bytes signature. The law just proved the round-tripped
    // output equals the input, so the raw-input hash IS the round-trip
    // output hash — the byte-exact witness.
    let computed_hex = raw_signature_hex(bytes);

    // Step 3: compare against praxis.lock [byte_exact_signatures].
    match lock_byte_exact_signature(reg.source_name, reg.source_version) {
        None => HarnessOutcome::LawHoldsSignatureUnpinned {
            computed_sha256_hex: computed_hex,
        },
        Some(expected) if expected == computed_hex => HarnessOutcome::Verified,
        Some(expected) => HarnessOutcome::SignatureMismatch {
            expected: expected.to_string(),
            computed_sha256_hex: computed_hex,
        },
    }
}

/// SHA-256 of raw bytes as lowercase hex — the byte-exact signature
/// (NIST FIPS 180-4 §6.2 SHA-256).
fn raw_signature_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex_of(&h.finalize())
}

enum SourceBytes {
    Loaded {
        #[allow(dead_code)]
        path: String,
        bytes: Vec<u8>,
    },
    NotOnDisk {
        path: String,
    },
    LoadError {
        path: String,
        message: String,
    },
}

enum SourceLookupError {
    NotRegistered,
}

fn resolve_source_bytes(reg: &LensRegistration) -> Result<SourceBytes, SourceLookupError> {
    let entry = by_name_version(reg.source_name, reg.source_version)
        .ok_or(SourceLookupError::NotRegistered)?;
    // RegistryEntry::local_path() is workspace-relative — the
    // harness must resolve it against the workspace root. We use
    // `CARGO_MANIFEST_DIR` (path to `crates/domains/`) and step up
    // two levels to reach the workspace root.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path_str = entry.local_path();
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .and_then(std::path::Path::parent);
    let abs_path = workspace_root
        .map(|root| root.join(&path_str))
        .unwrap_or_else(|| std::path::PathBuf::from(&path_str));
    match std::fs::read(&abs_path) {
        Ok(bytes) => Ok(SourceBytes::Loaded {
            path: abs_path.to_string_lossy().into_owned(),
            bytes,
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(SourceBytes::NotOnDisk {
            path: abs_path.to_string_lossy().into_owned(),
        }),
        Err(e) => Ok(SourceBytes::LoadError {
            path: abs_path.to_string_lossy().into_owned(),
            message: format!("{e}"),
        }),
    }
}

fn hex_of(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

// =============================================================================
// CI gate axiom.
// =============================================================================

/// CI gate: the round-trip harness reports zero hard failures.
///
/// Passes when every entry in [`LENS_REGISTRATIONS`] returns either
/// [`HarnessOutcome::Verified`], [`HarnessOutcome::SourceNotOnDisk`],
/// or [`HarnessOutcome::LawHoldsSignatureUnpinned`]. Fails on
/// [`HarnessOutcome::SignatureMismatch`], [`HarnessOutcome::LawViolated`],
/// [`HarnessOutcome::LoadError`], or [`HarnessOutcome::SourceNotRegistered`].
///
/// `SourceNotOnDisk` is non-fatal so that committers without
/// `pr4xis update`-ed corpora don't break the build. The
/// `LawHoldsSignatureUnpinned` allowance lets a lens land before its
/// `canonical_signature` is pinned in `praxis.lock` — the harness
/// surfaces the computed value so a follow-up commit can pin it.
pub struct RoundTripHarnessAllVerified;

impl Axiom for RoundTripHarnessAllVerified {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let results = run_round_trip_harness();
        let failures: Vec<_> = results.iter().filter(|r| r.outcome.is_failure()).collect();
        if failures.is_empty() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "RoundTripHarnessAllVerified",
        "every registered WellBehavedLens passes the PutGet law and (if signed) matches its pinned canonical_signature in praxis.lock",
        "Foster, Greenwald, Moore, Pierce & Schmitt (2007) ACM TOPLAS 29(3) §2.2; Dolstra (2006) Purely Functional Software Deployment §3"
    );
}

pr4xis::register_axiom!(
    RoundTripHarnessAllVerified,
    "Foster et al. (2007) ACM TOPLAS 29(3) §2.2; Dolstra (2006) Purely Functional Software Deployment §3"
);

#[cfg(test)]
#[path = "harness_tests.rs"]
mod tests;
