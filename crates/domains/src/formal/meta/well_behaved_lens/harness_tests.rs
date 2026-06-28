//! Tests for [`super::harness`].

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec::Vec};

use super::super::lens_trait::{FailureStage, LensLawFailure};
use super::*;
use pr4xis::ontology::Axiom;

#[pr4xis::praxis_value(Deterministic, Verifiable)]
#[test]
fn ci_gate_passes() {
    // The CI gate axiom (the ALWAYS-RUN fast lane): every entry in
    // LENS_REGISTRATIONS must either Verify, report SourceNotOnDisk (pending
    // `pr4xis update`), report LawHoldsSignatureUnpinned (pending lock-file
    // entry), or — for an oversize byte-exact source (WordNet, the giant USC
    // titles) — report OversizeDeferred (its full reconstruction is proven in the
    // slow `ci_gate_passes_giants` lane, keeping this gate under the per-test
    // nextest budget). All four are non-fatal.
    //
    // Hard failures (SignatureMismatch / LawViolated / ByteLawViolated /
    // LoadError / SourceNotRegistered) trip the gate.
    let verdict = RoundTripHarnessAllVerified.verify();
    if verdict.is_err() {
        // Dump the report for the human running the test.
        for r in run_round_trip_harness() {
            eprintln!("{}: {:?}", r.key, r.outcome);
        }
    }
    assert!(verdict.is_ok(), "round-trip harness reported hard failures");
}

#[pr4xis::praxis_value(Deterministic, Verifiable)]
#[test]
fn ci_gate_passes_giants() {
    // The SLOW-lane companion to `ci_gate_passes`: it includes the oversize
    // (> OVERSIZE_BYTE_EXACT_CAP_BYTES) byte-exact sources the fast lane DEFERS —
    // WordNet and the giant USC titles — and reconstructs them in full. Nextest
    // routes it to the relaxed `usc-giants` test-group (slow-timeout override,
    // serialized) via `.config/nextest.toml`, so the always-run fast lane keeps
    // its strict per-test cap while every big source is still proven on every
    // push. Hard failures trip the gate exactly as `ci_gate_passes`.
    let slow = run_round_trip_harness_including_oversize();
    let failures: Vec<_> = slow.iter().filter(|r| r.outcome.is_failure()).collect();
    assert!(
        failures.is_empty(),
        "slow-lane (include-oversize) harness reported hard failures: {failures:?}"
    );

    // NON-VACUITY: whatever the FAST lane deferred as OversizeDeferred (it skips
    // reconstruction for it), this lane must have actually RECONSTRUCTED — i.e.
    // the same key is no longer OversizeDeferred here. Guards against the slow
    // lane silently degrading into a copy of the fast lane (which would leave the
    // giants unproven on every push). When no oversize source is provisioned the
    // loop is empty and the test is a graceful no-op, the same skip discipline
    // the rest of the harness uses.
    let deferred: Vec<String> = run_round_trip_harness()
        .into_iter()
        .filter(|r| matches!(r.outcome, HarnessOutcome::OversizeDeferred { .. }))
        .map(|r| r.key)
        .collect();
    for key in &deferred {
        let row = slow.iter().find(|r| &r.key == key).unwrap_or_else(|| {
            panic!("{key} deferred in the fast lane but absent from the slow lane")
        });
        assert!(
            !matches!(row.outcome, HarnessOutcome::OversizeDeferred { .. }),
            "{key}: still OversizeDeferred in the include-oversize lane — the slow lane did not reconstruct it"
        );
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn oversize_deferred_is_not_a_failure() {
    // The oversize deferral is an explicit CI-budget choice, NOT a failed or
    // skipped proof — it must never trip the CI gate (the slow lane proves it).
    assert!(
        !HarnessOutcome::OversizeDeferred {
            size_bytes: 113_000_000
        }
        .is_failure()
    );
}

/// Three witness registrations whose keys are deliberately OUT of sorted order
/// (`"z…"`, `"a…"`, `"m…"`), so the harness's `sort_by(key)` has genuine work to
/// do — and whose `(source_name, source_version)` pairs are absent from the
/// praxis registry, so each resolves to `SourceNotRegistered` with NO disk read
/// or source parse. This drives the harness's STRUCTURAL post-conditions (one
/// result per registration; results ordered by key) over a controlled,
/// fast-to-resolve slice, instead of paying the real-source parse the global
/// `run_round_trip_harness()` would.
fn structural_witness_registrations() -> [LensRegistration; 3] {
    [
        witness_registration::<ByteExactWitnessLens>("z_witness@1", "z_witness", "1"),
        witness_registration::<ByteExactWitnessLens>("a_witness@1", "a_witness", "1"),
        witness_registration::<ByteExactWitnessLens>("m_witness@1", "m_witness", "1"),
    ]
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn harness_returns_one_entry_per_registration() {
    // STRUCTURAL post-condition: the harness emits exactly one result per
    // registration. Driven over the witness slice so it is fast (no real-source
    // parse); the real registrations are exercised end-to-end by `ci_gate_passes`.
    let regs = structural_witness_registrations();
    let results = run_harness_over_witnesses(&regs);
    assert_eq!(results.len(), regs.len());
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn harness_results_sorted_by_key() {
    // STRUCTURAL post-condition: the harness orders its results by key. The
    // witness keys are registered out of order ("z","a","m"), so a missing or
    // broken `sort_by` would leave them unsorted here.
    let regs = structural_witness_registrations();
    let results = run_harness_over_witnesses(&regs);
    for w in results.windows(2) {
        assert!(w[0].key <= w[1].key, "results not sorted by key");
    }
    // Non-vacuity: the witnesses really were registered out of sorted order, so
    // the assertion above had genuine ordering work to verify.
    assert_eq!(regs[0].key, "z_witness@1");
    assert_eq!(regs[1].key, "a_witness@1");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn harness_outcome_is_failure_classifies_correctly() {
    // Non-failure outcomes.
    assert!(!HarnessOutcome::Verified.is_failure());
    assert!(!HarnessOutcome::SourceNotOnDisk { path: "p".into() }.is_failure());
    assert!(
        !HarnessOutcome::LawHoldsSignatureUnpinned {
            computed_digest_hex: "abc".into()
        }
        .is_failure()
    );

    // Failure outcomes.
    assert!(
        HarnessOutcome::SignatureMismatch {
            expected: "a".into(),
            computed_digest_hex: "b".into()
        }
        .is_failure()
    );
    assert!(
        HarnessOutcome::LoadError {
            path: "p".into(),
            message: "m".into()
        }
        .is_failure()
    );
    assert!(HarnessOutcome::SourceNotRegistered.is_failure());
    // LawViolated has a payload, so just construct one and check.
    let failure = LensLawFailure {
        stage: FailureStage::DigestMismatch,
        message: "test".into(),
        input_digest: None,
        roundtrip_digest: None,
    };
    assert!(HarnessOutcome::LawViolated(failure).is_failure());
    // ByteLawViolated (the byte-exact leg, M4.ι / #186) also carries a
    // payload. Its FailureStage is ByteMismatch, the stage
    // `assert_byte_exact_law` emits on `put(get(b)) != b`. A refactor that
    // forgot to wire it into `is_failure` would turn a byte-law violation
    // into green CI — this is the regression guard.
    let byte_failure = LensLawFailure {
        stage: FailureStage::ByteMismatch,
        message: "test".into(),
        input_digest: None,
        roundtrip_digest: None,
    };
    assert!(HarnessOutcome::ByteLawViolated(byte_failure).is_failure());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ci_gate_axiom_metadata_cites_literature() {
    let meta = RoundTripHarnessAllVerified.meta();
    let citation = meta.citation.as_str();
    assert!(
        citation.contains("Foster"),
        "axiom citation should reference Foster et al. 2007; got: {citation}"
    );
}

// ============================================================================
// Byte-exact leg (M4.ι / #186) — drive `verify_byte_exact` against witness
// lenses declaring `RoundTripFidelity::ByteExactGraphFaithful`.
//
// `run_round_trip_harness` exercises the registered production lenses against
// their on-disk sources — including the byte-exact graph-faithful lenses (OWL,
// USC titles, WN-LMF). These witness tests ADDITIONALLY drive the byte-exact
// dispatch leg (`verify_loaded_bytes` -> `verify_byte_exact`) with controlled
// `LensRegistration`s, so the success/violation branches are covered even when a
// corpus is not provisioned and independently of any one production lens.
// ----------------------------------------------------------------------------

/// An identity byte-exact witness lens. `get`/`put` round-trip valid UTF-8
/// verbatim, so the byte-exact law `put(get(b)) == b` holds with no
/// complement (mirrors `tests::ByteExactStringSource`).
struct ByteExactWitnessLens;

impl super::super::WellBehavedLens for ByteExactWitnessLens {
    type Target = alloc::string::String;
    type Error = super::super::canonical::CanonicalizationError;

    const FIDELITY: RoundTripFidelity = RoundTripFidelity::ByteExactGraphFaithful;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        core::str::from_utf8(bytes)
            .map(ToString::to_string)
            .map_err(|e| {
                super::super::canonical::CanonicalizationError::new(
                    "byte-exact-witness",
                    format!("non-UTF-8: {}", e),
                )
            })
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(target.as_bytes().to_vec())
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        super::super::canonical::plain_text::canonicalize(bytes)
    }
}

/// A deliberately-broken byte-exact witness whose `put` drops the final
/// byte, so `put(get(b)) != b`. Used to confirm `verify_byte_exact`
/// surfaces the violation as `ByteLawViolated` (mirrors
/// `tests::DroppingStringSource`).
struct BrokenByteExactWitnessLens;

impl super::super::WellBehavedLens for BrokenByteExactWitnessLens {
    type Target = alloc::string::String;
    type Error = super::super::canonical::CanonicalizationError;

    const FIDELITY: RoundTripFidelity = RoundTripFidelity::ByteExactGraphFaithful;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        core::str::from_utf8(bytes)
            .map(ToString::to_string)
            .map_err(|e| {
                super::super::canonical::CanonicalizationError::new(
                    "broken-byte-exact-witness",
                    format!("non-UTF-8: {}", e),
                )
            })
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        // Drop the last byte if any — reconstruction differs from the input.
        let bytes = target.as_bytes();
        if bytes.is_empty() {
            Ok(Vec::new())
        } else {
            Ok(bytes[..bytes.len() - 1].to_vec())
        }
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        super::super::canonical::plain_text::canonicalize(bytes)
    }
}

/// Build a [`LensRegistration`] for witness lens `L` against `(name,
/// version)` — the same field wiring the `register_lens!` macro emits,
/// so the harness treats the witness exactly like a registered lens.
fn witness_registration<L: super::super::WellBehavedLens>(
    key: &'static str,
    name: &'static str,
    version: &'static str,
) -> LensRegistration {
    LensRegistration {
        key,
        source_name: name,
        source_version: version,
        assert_law: L::assert_put_get_law,
        assert_byte_exact: L::assert_byte_exact_law,
        signature: |b, algorithm| L::signature_hex(b, algorithm).map_err(|e| format!("{}", e)),
        fidelity: L::FIDELITY,
    }
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn verify_byte_exact_reports_byte_law_violated_for_broken_witness() {
    // A byte-exact witness whose `put(get(b)) != b` must surface as
    // ByteLawViolated, and that outcome must fail the CI gate. This is the
    // byte-law-violation -> red-CI path the byte-exact leg exists to enforce.
    let reg = witness_registration::<BrokenByteExactWitnessLens>(
        "broken_byte_exact_witness@1",
        "broken_byte_exact_witness",
        "1",
    );
    let outcome = verify_byte_exact(&reg, b"hello world");
    assert!(
        matches!(outcome, HarnessOutcome::ByteLawViolated(_)),
        "broken byte-exact witness should report ByteLawViolated, got: {outcome:?}"
    );
    assert!(outcome.is_failure());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn verify_byte_exact_dispatches_through_fidelity() {
    // `verify_loaded_bytes` must route a ByteExactGraphFaithful lens to the
    // byte-exact leg (not the canonical FLOOR). The broken witness gives a
    // distinctive ByteLawViolated only the byte-exact leg can produce —
    // verify_canonical would instead report LawViolated.
    let reg = witness_registration::<BrokenByteExactWitnessLens>(
        "broken_byte_exact_witness@1",
        "broken_byte_exact_witness",
        "1",
    );
    let outcome = verify_loaded_bytes(&reg, b"hello world");
    assert!(
        matches!(outcome, HarnessOutcome::ByteLawViolated(_)),
        "ByteExactGraphFaithful fidelity should dispatch to the byte-exact leg, got: {outcome:?}"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn verify_byte_exact_reports_unpinned_when_law_holds_but_no_pin() {
    // An identity byte-exact witness satisfies `put(get(b)) == b`. With no
    // `[byte_exact_signatures]` pin for its (name, version), the harness
    // reports LawHoldsSignatureUnpinned (non-fatal) and surfaces the
    // raw-bytes content digest the human would pin. The name is test-only and absent from
    // praxis.lock, so the lock lookup returns None.
    let reg = witness_registration::<ByteExactWitnessLens>(
        "unpinned_byte_exact_witness@1",
        "unpinned_byte_exact_witness",
        "1",
    );
    let outcome = verify_byte_exact(&reg, b"hello world");
    // Non-fatal: a law-holding-but-unpinned source does not trip the gate.
    assert!(!outcome.is_failure());
    let HarnessOutcome::LawHoldsSignatureUnpinned {
        computed_digest_hex,
    } = outcome
    else {
        panic!(
            "identity byte-exact witness with no pin should report LawHoldsSignatureUnpinned, got: {outcome:?}"
        );
    };
    // The surfaced value is the raw-bytes content digest (64-char lowercase
    // hex); because `put(get(b)) == b`, it is also the round-tripped output
    // hash.
    assert_eq!(computed_digest_hex.len(), 64);
}

// NOTE ON COVERAGE GAP — `verify_byte_exact`'s `Verified` and
// `SignatureMismatch` branches are NOT exercised here. Both require
// `lock_byte_exact_signature(name, version)` to return `Some`, i.e. a
// `[byte_exact_signatures]` entry in the workspace `praxis.lock`. That file
// is `include_str!`-embedded and `OnceLock`-cached, with no test seam to
// inject a pin; and a free-standing lock entry trips the no-straggler rule
// of `LockManifestAgreement` (every lock key must have a praxis.toml
// source). Reaching those two branches needs a production seam (e.g.
// `verify_byte_exact` taking the pinned signature as a parameter) — see the
// report accompanying this change. The `[byte_exact_signatures]` parse +
// equality rules themselves are covered by `registry::parser_tests`.

// The human-driven signature-dump helper is `harness::dump_unpinned_signatures`
// — a `pub fn`, not a `#[test]`. Pairs with [[feedback-never-use-ignore]].
