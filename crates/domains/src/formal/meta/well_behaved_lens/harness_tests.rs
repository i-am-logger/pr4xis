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

// The human-driven signature-dump helper is `harness::dump_unpinned_signatures`
// — a `pub fn`, not a `#[test]`. Pairs with [[feedback-never-use-ignore]].
