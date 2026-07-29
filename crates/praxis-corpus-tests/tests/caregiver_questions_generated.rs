//! One `#[test]` PER caregiver-corpus question — 4617 independently named,
//! independently reported, independently PARALLEL-executed tests
//! (`cargo test`'s default thread-per-test runner).
//!
//! The generated body of this file lives in
//! `$OUT_DIR/generated_caregiver_questions.rs` — see `build.rs`. Every
//! question's text comes from `tests/fixtures/caregiver_question_corpus.json`;
//! nothing is hand-typed here.
//!
//! Each generated test's assertion is `praxis_corpus_tests::caregiver::
//! assert_is_green` — a bare pass/fail against the honest truth ("this
//! question is answerable"), not a comparison against a frozen snapshot. A
//! real capability gap (`MissingTerm`, `UnparsedKnownTerm`, `OverAnswered`,
//! `PossibleMisroute`) is therefore a FAILING, named test in ordinary
//! `cargo test` output — "3707 passed; 910 failed" is the real scorecard, not
//! an artifact only visible in a separate regeneration run's log line.
//! (Previously each test compared against a committed snapshot label
//! instead — precise about lateral shifts between gap classes, but it made a
//! known gap read as a PASSING test, which is backwards for a research
//! project that wants to demonstrate both "the engine works" and "here is
//! exactly what remains.")
//!
//! This file intentionally does NOT gate CI on its own: with ~900 questions
//! not yet answerable, this suite is expected to have real, honest failures
//! — the same "red by design" precedent already established by
//! `chat_capability.rs`'s `chat_faithfully_realizes_the_loaded_corpus`. The
//! monotonic regression GATE — the check that actually blocks a commit —
//! lives in the separate `caregiver_capability_ratchet.rs`, an aggregate
//! per-class ceiling test mirroring `chat_capability.rs`'s own ratchet.
//!
//! The snapshot file below remains a human-readable PROGRESS artifact (its
//! `git diff` shows exactly which questions moved and how), not a test
//! oracle — regenerate it after any change to see the honest number move,
//! and to re-derive the ratchet's ceiling constants for the next commit.

include!(concat!(
    env!("OUT_DIR"),
    "/generated_caregiver_questions.rs"
));

/// The exact `cargo test` invocation that re-emits every committed corpus
/// artifact below (snapshot, status, slim). Single source of truth so the
/// `regenerated_by` command chips the Caregiver Evaluation Bench renders read
/// verbatim from the artifact that emits them — never hand-typed into page
/// copy — and so status and slim can never disagree about how they were made.
const REGEN_CMD: &str = "cargo test --manifest-path crates/praxis-corpus-tests/Cargo.toml --release --test caregiver_questions_generated -- --ignored regenerate_caregiver_snapshot --nocapture";

/// `docs/caregiver-corpus-slim.json` — one entry per corpus question, the
/// runtime-random sampler/slice source the bench page fetches. A SIBLING of
/// `docs/caregiver-corpus-status.json` (see that file's write in
/// `regenerate_caregiver_snapshot`) for the same deployed-URL-space reason.
const SLIM_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/caregiver-corpus-slim.json"
);

/// Recomputes every question's current classification and overwrites the
/// committed snapshot. `#[ignore]`d — matches this codebase's established
/// `regenerate_*` convention (e.g. `regenerate_raw_source_prx` in
/// pr4xis-domains): run explicitly, on purpose, never as part of a normal
/// test pass, since its whole point is to CHANGE the committed expectation
/// file rather than check against it.
///
/// `cargo test --manifest-path crates/praxis-corpus-tests/Cargo.toml --release \
///   --test caregiver_questions_generated -- --ignored regenerate_caregiver_snapshot --nocapture`
#[test]
#[ignore]
fn regenerate_caregiver_snapshot() {
    // Ordered labels ARE the classification pass (one reasoner per worker
    // thread, not per question — see `corpus_labels_ordered`'s own doc); the
    // breakdown tally below is a cheap O(n) re-count over the SAME already-
    // computed labels, not a second full parallel corpus pass.
    let labels = praxis_corpus_tests::caregiver::corpus_labels_ordered();
    let mut breakdown: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for label in &labels {
        *breakdown.entry(label.clone()).or_insert(0) += 1;
    }
    let green = breakdown.get("Green").copied().unwrap_or(0);
    let by_class: std::collections::BTreeMap<&String, usize> = breakdown
        .iter()
        .filter(|(label, _)| label.as_str() != "Green")
        .map(|(label, count)| (label, *count))
        .collect();
    eprintln!(
        "REGENERATED SNAPSHOT: {green}/{} green; red by class: {by_class:?}",
        labels.len()
    );

    let json = serde_json::to_string_pretty(&labels).expect("serialize snapshot");
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/caregiver_question_corpus.snapshot.json"
    );
    std::fs::write(path, json + "\n").expect("write snapshot file");
    eprintln!("wrote {path}");

    // The SAME breakdown, re-derived into the small public-facing summary
    // the Caregiver AI Challenge WASM tab displays (docs/chat/index.html) —
    // written in the SAME regen step as the snapshot so the two can never
    // drift apart. Deliberately just the aggregate counts, not the 4617
    // question texts themselves (a much larger, separately-licensed
    // dataset) — a judge reading the live tab sees a number this exact test
    // re-derives, not a hand-typed claim.
    //
    // Written to docs/ (a SIBLING of docs/chat/index.html, not inside
    // docs/chat/) to match the page's actual deployed URL space: the CI
    // Pages job copies docs/chat/index.html -> pages/index.html and
    // docs/worker.js -> pages/worker.js side by side, so index.html's own
    // relative fetches ("./worker.js", and now "./caregiver-corpus-
    // status.json") resolve against docs/'s root, not docs/chat/'s.
    // The `ceilings` block is what turns the demo's roadmap from a picture of
    // today into a picture of a COMMITMENT: each figure is the CI-enforced
    // bound a class may not cross, read from the same
    // `praxis_corpus_tests::caregiver::ratchet` constants
    // `caregiver_capability_never_regresses_the_committed_ceilings` enforces.
    // Published rather than hand-typed on the page so the tick a reviewer
    // sees and the gate that fails the build are one number.
    #[derive(serde::Serialize)]
    struct Ceilings {
        green_floor: usize,
        missing_term: usize,
        unparsed_known_term: usize,
        over_answered: usize,
        possible_misroute: usize,
    }
    #[derive(serde::Serialize)]
    struct CorpusStatus {
        total: usize,
        green: usize,
        missing_term: usize,
        unparsed_known_term: usize,
        over_answered: usize,
        possible_misroute: usize,
        ceilings: Ceilings,
        regenerated_by: &'static str,
    }
    let status = CorpusStatus {
        total: labels.len(),
        green,
        missing_term: breakdown.get("MissingTerm").copied().unwrap_or(0),
        unparsed_known_term: breakdown.get("UnparsedKnownTerm").copied().unwrap_or(0),
        over_answered: breakdown.get("OverAnswered").copied().unwrap_or(0),
        possible_misroute: breakdown.get("PossibleMisroute").copied().unwrap_or(0),
        ceilings: Ceilings {
            green_floor: praxis_corpus_tests::caregiver::ratchet::GREEN,
            missing_term: praxis_corpus_tests::caregiver::ratchet::MISSING_TERM,
            unparsed_known_term: praxis_corpus_tests::caregiver::ratchet::UNPARSED_KNOWN_TERM,
            over_answered: praxis_corpus_tests::caregiver::ratchet::OVER_ANSWERED,
            possible_misroute: praxis_corpus_tests::caregiver::ratchet::POSSIBLE_MISROUTE,
        },
        regenerated_by: REGEN_CMD,
    };
    let status_json = serde_json::to_string_pretty(&status).expect("serialize status");
    let status_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/caregiver-corpus-status.json"
    );
    std::fs::write(status_path, status_json + "\n").expect("write status file");
    eprintln!("wrote {status_path}");

    // The per-question slim corpus the caregiver tab (docs/chat/index.html#caregiver)
    // samples for its uncertain, uncurated "from the evaluation corpus" and
    // "run a random slice" surfaces, and reads snapshot labels off for "show
    // me a question we currently fail". Emitted in the SAME regen step as the
    // snapshot from the SAME ordered `labels` pass, so the shipped file's
    // labels and the committed snapshot can never drift — the drift GATE
    // `slim_corpus_artifact_tracks_snapshot` below fails CI if a hand-edit or
    // partial commit ever breaks that. Deliberately carries only the small
    // provenance fields the page renders (question, track, capability, topic,
    // label, source), not the whole harvested record.
    #[derive(serde::Serialize)]
    struct SlimEntry {
        q: String,
        track: String,
        capability: String,
        topic: String,
        label: String,
        source: String,
        // The same field `caregiver::classify_case` checks an `Answered`
        // response against (case-insensitive containment) to distinguish a
        // true correct answer from `PossibleMisroute`. Shipped so a live,
        // in-browser re-run can apply the identical correctness check the
        // committed snapshot uses, not a looser approximation.
        key_term: String,
    }
    #[derive(serde::Serialize)]
    struct SlimCorpus {
        questions: Vec<SlimEntry>,
        regenerated_by: &'static str,
    }
    let cases = praxis_corpus_tests::caregiver::fixture();
    assert_eq!(
        cases.len(),
        labels.len(),
        "fixture and classification labels must align 1:1"
    );
    let questions: Vec<SlimEntry> = cases
        .iter()
        .zip(&labels)
        .map(|(c, label)| SlimEntry {
            q: c.question.clone(),
            track: c.track.clone(),
            capability: c.praxis_capability.clone(),
            topic: c.topic_category.clone(),
            label: label.clone(),
            source: c.source.clone(),
            key_term: c.key_term.clone(),
        })
        .collect();
    let slim = SlimCorpus {
        questions,
        regenerated_by: REGEN_CMD,
    };
    let slim_json = serde_json::to_string_pretty(&slim).expect("serialize slim corpus");
    std::fs::write(SLIM_PATH, slim_json + "\n").expect("write slim corpus file");
    eprintln!("wrote {SLIM_PATH}");
}

/// CI drift gate for the shipped `docs/caregiver-corpus-slim.json` — cheap,
/// non-`#[ignore]`d, no reasoner. `regenerate_caregiver_snapshot` emits the
/// slim file and the committed snapshot from ONE classification pass, but a
/// hand-edit, a merge, or a partial commit could still ship a slim file that
/// disagrees with the snapshot the rest of the suite gates against. This test
/// makes that disagreement a red CI build: the shipped slim file must have the
/// SAME length as both the fixture and the committed snapshot, and its
/// per-question `label`/`q` must equal the committed snapshot label / fixture
/// question at every index. Reads the artifact at RUN time (not `include_str!`)
/// so a missing file is a clear test failure, not a compile error that would
/// block regenerating it on a fresh checkout.
#[test]
fn slim_corpus_artifact_tracks_snapshot() {
    #[derive(serde::Deserialize)]
    struct SlimEntry {
        q: String,
        label: String,
    }
    #[derive(serde::Deserialize)]
    struct SlimCorpus {
        questions: Vec<SlimEntry>,
    }
    let raw = std::fs::read_to_string(SLIM_PATH).unwrap_or_else(|e| {
        panic!("shipped slim corpus {SLIM_PATH} must exist (regenerate with `{REGEN_CMD}`): {e}")
    });
    let slim: SlimCorpus =
        serde_json::from_str(&raw).expect("shipped slim corpus is well-formed JSON");
    let snapshot = praxis_corpus_tests::caregiver::snapshot();
    let fixture = praxis_corpus_tests::caregiver::fixture();
    assert_eq!(
        slim.questions.len(),
        fixture.len(),
        "slim corpus length {} != fixture length {}",
        slim.questions.len(),
        fixture.len()
    );
    assert_eq!(
        slim.questions.len(),
        snapshot.len(),
        "slim corpus length {} != committed snapshot length {}",
        slim.questions.len(),
        snapshot.len()
    );
    for (i, entry) in slim.questions.iter().enumerate() {
        assert_eq!(
            entry.label, snapshot[i],
            "slim corpus label drift at index {i}: slim says {:?}, committed snapshot says {:?} \
             — regenerate both together with `{REGEN_CMD}`",
            entry.label, snapshot[i]
        );
        assert_eq!(
            entry.q, fixture[i].question,
            "slim corpus question drift at index {i}: slim ships a question the fixture does not \
             carry at that index — regenerate with `{REGEN_CMD}`"
        );
    }
}
