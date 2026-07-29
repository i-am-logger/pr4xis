//! One `#[test]` PER adversarial honesty question (task #15/#37) — the ACL
//! Caregiver AI Challenge's own "Safety Exhibit Test" demand that the
//! system, fed a fabricated protocol, *prove* it safely refuses rather than
//! hallucinates. Replaces the earlier single aggregate-ceiling test
//! (`adversarial_corpus_red_green_by_category`, removed) with one
//! independently named, independently reported, independently PARALLEL-
//! executed test per question, mirroring `caregiver_questions_generated.rs`.
//!
//! The generated body of this file lives in
//! `$OUT_DIR/generated_adversarial_questions.rs` — see `build.rs`. Every
//! question's text and expected classification label come from
//! `tests/fixtures/adversarial_question_corpus.json` and its companion
//! `adversarial_question_corpus.snapshot.json`; nothing is hand-typed here.
//!
//! `tests/fixtures/adversarial_question_corpus.json` is a TEST-ONLY,
//! AUTHORED (not harvested) fixture, 40 questions per category (160 total,
//! expanded 2026-07-16 from an initial 40 total per user direction — "9
//! isn't strong enough, these should be heavily tested"), in four categories
//! grounded in the QA-honesty literature:
//!
//! - `fabricated_citation` — a syntactically well-formed statute/CFR/Federal
//!   Register citation under a REAL title that does not itself exist (Lin,
//!   Hilton & Evans 2022, "TruthfulQA: Measuring How Models Mimic Human
//!   Falsehoods", ACL 2022).
//! - `fabricated_term` — a plausible-sounding program/acronym name that is
//!   not defined anywhere in the loaded lexicons (same TruthfulQA grounding).
//! - `false_presupposition` — a real, loaded term asked about through a
//!   question whose own premise misstates a fact about it (Kim, Pavlick, Karagol Ayan &
//!   Ramachandran 2021, "Which Linguist Invented the Lightbulb?
//!   Presupposition Verification for Question-Answering", ACL-IJCNLP 2021,
//!   pp. 3932-3945). Unlike the other three categories, `keyTerm` here IS expected to
//!   resolve — the fabrication is in the question's premise, never in the
//!   entity itself.
//! - `domain_mimicry` — real domain words recombined into a compound that
//!   corresponds to no loaded concept (Rajpurkar, Jia & Liang 2018, "Know
//!   What You Don't Know: Unanswerable Questions for SQuAD", ACL 2018).
//!
//! The safe outcome is the SAME closed-world refusal (Reiter 1978) the
//! real-world corpus harness (`caregiver_questions_generated.rs`) already
//! grades: [`pr4xis_chat::ChatOutcome::Abstained`], never `Answered` or
//! `Conditional`.
//!
//! Per-question SNAPSHOT comparison, not a bare category ceiling — a lateral
//! shift (one question flipping unsafe while another flips safe, netting to
//! the same category count) fails loud here where an aggregate ceiling could
//! not see it.
//!
//! **2026-07-16, Track 2.3 SHIPPED**: three DISTINCT defects in
//! `attempt_partial_understanding` (`crates/chat/src/lib.rs`) fixed — see
//! `crates/chat/src/lib.rs`'s own doc comments and commit `0fb3637d`.
//! Collapsed the real 4617-question corpus's `OverAnswered` bucket from 3711
//! (80%) to 642 (14%); on THIS corpus, red counts fell from
//! fabricated_citation=7/fabricated_term=7/domain_mimicry=5/
//! false_presupposition=9 (out of the original 12/11/8/9) to 0/4/2/1.
//!
//! **`false_presupposition`** unsafe cases (mostly still unsafe) trace to a
//! capability that does not exist anywhere in this codebase yet:
//! presupposition/premise verification (Kim et al. 2021, cited above). This
//! is separate, unscoped future work (not Track 2.3).

include!(concat!(
    env!("OUT_DIR"),
    "/generated_adversarial_questions.rs"
));

/// The exact `cargo test` invocation that re-emits the committed adversarial
/// snapshot and the shipped `docs/adversarial-corpus.json` open-mic artifact.
/// Single source of truth so the bench page's `regenerated_by` command chip
/// reads verbatim from the artifact it regenerates.
const REGEN_CMD: &str = "cargo test --manifest-path crates/praxis-corpus-tests/Cargo.toml --release --test adversarial_questions_generated -- --ignored regenerate_adversarial_snapshot --nocapture";

/// `docs/adversarial-corpus.json` — the authored adversarial questions the
/// bench page's "adversarial open mic" samples. A SIBLING of the other
/// `docs/*.json` bench artifacts, for the same deployed-URL-space reason
/// documented in `caregiver_questions_generated.rs`.
const ADVERSARIAL_CORPUS_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/adversarial-corpus.json"
);

/// A companion self-consistency check on the fixture, not the pipeline: for
/// the three genuinely-fabricated categories, `keyTerm` must resolve to
/// nothing loaded — otherwise the item isn't testing fabrication-honesty at
/// all, it's silently testing a real term the grammar merely failed to
/// route (that belongs in the real-world corpus harness, not here).
/// `false_presupposition` is asserted the other way: its `keyTerm` must
/// resolve, since the fabrication there is in the premise, not the entity.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn adversarial_corpus_key_terms_match_their_category_contract() {
    use pr4xis_domains::cognitive::linguistics::english::LexicalReasoner;
    let (reasoner, _english) = praxis_corpus_tests::adversarial::setup_reasoner();

    let cases = praxis_corpus_tests::adversarial::fixture();
    let mut violations: Vec<String> = Vec::new();
    for case in &cases {
        let known = !reasoner.lookup(&case.key_term.to_lowercase()).is_empty();
        let should_be_known = case.category == "false_presupposition";
        if known != should_be_known {
            violations.push(format!(
                "[{}] keyTerm={:?} known={known} (expected known={should_be_known}): {}",
                case.category, case.key_term, case.fabrication_note
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "fixture items whose keyTerm-known state contradicts their category's contract \
         (fix the fixture, not the pipeline):\n{}",
        violations.join("\n")
    );
}

/// Recomputes every question's current classification and overwrites the
/// committed snapshot. `#[ignore]`d — matches this codebase's established
/// `regenerate_*` convention: run explicitly, on purpose, never as part of a
/// normal test pass.
///
/// `cargo test --manifest-path crates/praxis-corpus-tests/Cargo.toml --release \
///   --test adversarial_questions_generated -- --ignored regenerate_adversarial_snapshot --nocapture`
#[test]
#[ignore]
fn regenerate_adversarial_snapshot() {
    let cases = praxis_corpus_tests::adversarial::fixture();
    let labels: Vec<String> = (0..cases.len())
        .map(praxis_corpus_tests::adversarial::classify_label)
        .collect();
    let mut safe = 0usize;
    let mut by_class: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for label in &labels {
        if label == "Safe" {
            safe += 1;
        } else {
            *by_class.entry(label.as_str()).or_default() += 1;
        }
    }
    eprintln!(
        "REGENERATED ADVERSARIAL SNAPSHOT: {safe}/{} safe; unsafe by kind: {by_class:?}",
        labels.len()
    );

    let json = serde_json::to_string_pretty(&labels).expect("serialize snapshot");
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/adversarial_question_corpus.snapshot.json"
    );
    std::fs::write(path, json + "\n").expect("write snapshot file");
    eprintln!("wrote {path}");

    // The bench page's "adversarial open mic" chips are sampled, at runtime,
    // from this shipped artifact — never a curated example set baked into page
    // copy. Emitted in the SAME regen step, from the SAME authored `cases`, so
    // the shipped file cannot drift from the fixture; the drift GATE
    // `adversarial_corpus_artifact_tracks_fixture` below fails CI otherwise.
    // Carries only the two fields the page renders (`q`, `category`) — the
    // fabrication note and key term are test-harness internals, not open-mic
    // copy.
    #[derive(serde::Serialize)]
    struct AdvEntry {
        q: String,
        category: String,
    }
    #[derive(serde::Serialize)]
    struct AdvCorpus {
        questions: Vec<AdvEntry>,
        regenerated_by: &'static str,
    }
    let questions: Vec<AdvEntry> = cases
        .iter()
        .map(|c| AdvEntry {
            q: c.question.clone(),
            category: c.category.clone(),
        })
        .collect();
    let corpus = AdvCorpus {
        questions,
        regenerated_by: REGEN_CMD,
    };
    let corpus_json = serde_json::to_string_pretty(&corpus).expect("serialize adversarial corpus");
    std::fs::write(ADVERSARIAL_CORPUS_PATH, corpus_json + "\n")
        .expect("write adversarial corpus file");
    eprintln!("wrote {ADVERSARIAL_CORPUS_PATH}");
}

/// CI drift gate for the shipped `docs/adversarial-corpus.json` — cheap,
/// non-`#[ignore]`d, no reasoner. `regenerate_adversarial_snapshot` emits the
/// open-mic artifact from the SAME authored fixture the whole suite runs, but
/// a hand-edit or partial commit could still ship a file that disagrees with
/// it. This test makes that a red CI build: the shipped file must have the
/// SAME length as the fixture, and its per-question `q`/`category` must equal
/// the fixture's at every index. Reads the artifact at RUN time so a missing
/// file is a clear failure, not a compile error.
#[test]
fn adversarial_corpus_artifact_tracks_fixture() {
    #[derive(serde::Deserialize)]
    struct AdvEntry {
        q: String,
        category: String,
    }
    #[derive(serde::Deserialize)]
    struct AdvCorpus {
        questions: Vec<AdvEntry>,
    }
    let raw = std::fs::read_to_string(ADVERSARIAL_CORPUS_PATH).unwrap_or_else(|e| {
        panic!(
            "shipped adversarial corpus {ADVERSARIAL_CORPUS_PATH} must exist \
             (regenerate with `{REGEN_CMD}`): {e}"
        )
    });
    let corpus: AdvCorpus =
        serde_json::from_str(&raw).expect("shipped adversarial corpus is well-formed JSON");
    let fixture = praxis_corpus_tests::adversarial::fixture();
    assert_eq!(
        corpus.questions.len(),
        fixture.len(),
        "adversarial corpus length {} != fixture length {}",
        corpus.questions.len(),
        fixture.len()
    );
    for (i, entry) in corpus.questions.iter().enumerate() {
        assert_eq!(
            entry.q, fixture[i].question,
            "adversarial corpus question drift at index {i} — regenerate with `{REGEN_CMD}`"
        );
        assert_eq!(
            entry.category, fixture[i].category,
            "adversarial corpus category drift at index {i} — regenerate with `{REGEN_CMD}`"
        );
    }
}
