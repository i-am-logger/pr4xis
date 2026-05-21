//! Tests for [`super::harness`].

use super::super::lens_trait::{FailureStage, LensLawFailure};
use super::*;
use pr4xis::ontology::Axiom;

#[test]
fn ci_gate_passes() {
    // The CI gate axiom: every entry in LENS_REGISTRATIONS must
    // either Verify, report SourceNotOnDisk (pending `pr4xis update`),
    // or report LawHoldsSignatureUnpinned (pending lock-file entry).
    //
    // Hard failures (SignatureMismatch / LawViolated / LoadError /
    // SourceNotRegistered) trip the gate.
    let verdict = RoundTripHarnessAllVerified.verify();
    if verdict.is_err() {
        // Dump the report for the human running the test.
        for r in run_round_trip_harness() {
            eprintln!("{}: {:?}", r.key, r.outcome);
        }
    }
    assert!(verdict.is_ok(), "round-trip harness reported hard failures");
}

#[test]
fn harness_returns_one_entry_per_registration() {
    let results = run_round_trip_harness();
    assert_eq!(results.len(), LENS_REGISTRATIONS.len());
}

#[test]
fn harness_results_sorted_by_key() {
    let results = run_round_trip_harness();
    for w in results.windows(2) {
        assert!(w[0].key <= w[1].key, "results not sorted by key");
    }
}

#[test]
fn harness_outcome_is_failure_classifies_correctly() {
    // Non-failure outcomes.
    assert!(!HarnessOutcome::Verified.is_failure());
    assert!(!HarnessOutcome::SourceNotOnDisk { path: "p".into() }.is_failure());
    assert!(
        !HarnessOutcome::LawHoldsSignatureUnpinned {
            computed_sha256_hex: "abc".into()
        }
        .is_failure()
    );

    // Failure outcomes.
    assert!(
        HarnessOutcome::SignatureMismatch {
            expected: "a".into(),
            computed_sha256_hex: "b".into()
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
}

#[test]
fn ci_gate_axiom_metadata_cites_literature() {
    let meta = RoundTripHarnessAllVerified.meta();
    let citation = meta.citation.as_str();
    assert!(
        citation.contains("Foster"),
        "axiom citation should reference Foster et al. 2007; got: {citation}"
    );
}

/// Human-driven helper: prints every harness outcome, including the
/// computed canonical-form SHA-256 for any source still in the
/// `LawHoldsSignatureUnpinned` state. Use this to discover the value
/// to pin in `praxis.lock`'s `[canonical_signatures]` section:
///
/// ```sh
/// direnv exec . cargo test -p pr4xis-domains --lib \
///   dump_unpinned_signatures -- --ignored --nocapture
/// ```
///
/// `#[ignore]` so the (potentially slow — large XML canonicalization)
/// run only fires when a human asks for it.
#[test]
#[ignore = "human-driven signature-dump helper; run with --ignored --nocapture"]
fn dump_unpinned_signatures() {
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
