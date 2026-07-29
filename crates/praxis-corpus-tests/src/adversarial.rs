//! Shared machinery for the adversarial honesty corpus (task #15/#37) — the
//! ACL Caregiver AI Challenge's own "Safety Exhibit Test" demand that the
//! system, fed a fabricated protocol, *prove* it safely refuses rather than
//! hallucinates. Factored out of the test harness so both the build-time
//! codegen (`build.rs`, which emits one `#[test]` per question into
//! `tests/adversarial_questions_generated.rs`) and the snapshot-regeneration
//! tool can reuse the exact same fixture loading, reasoner setup, and
//! classification logic.
//!
//! `tests/fixtures/adversarial_question_corpus.json` is a TEST-ONLY,
//! AUTHORED (not harvested) fixture: every question is deliberately
//! fabricated, in one of four categories grounded in the QA-honesty
//! literature:
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
//! real-world corpus harness ([`crate::caregiver`]) grades:
//! [`pr4xis_chat::ChatOutcome::Abstained`], never `Answered` or
//! `Conditional`.
//!
//! Per-question SNAPSHOT comparison (task #16/#37's pattern, mirroring
//! [`crate::caregiver`]), not a bare category ceiling: comparing each
//! question's CURRENT classification against a committed snapshot label
//! catches a lateral shift (one question flipping unsafe while another
//! flips safe, netting to the same category count) that an aggregate
//! ceiling cannot see.

use std::rc::Rc;

use pr4xis_chat::ChatOutcome;
use pr4xis_domains::applied::data_provisioning::chat_lexicons::definitional_lexicon_runtime_ontology;
use pr4xis_domains::applied::data_provisioning::registry::data_sources;
use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
use pr4xis_domains::cognitive::linguistics::english::english_loaded;
use pr4xis_runtime::ontology::RuntimeOntology;
use serde::Deserialize;

const FIXTURE: &str = include_str!("../tests/fixtures/adversarial_question_corpus.json");
const SNAPSHOT: &str = include_str!("../tests/fixtures/adversarial_question_corpus.snapshot.json");

#[derive(Deserialize)]
pub struct AdversarialQuestion {
    pub question: String,
    pub category: String,
    #[allow(dead_code)]
    #[serde(rename = "fabricationNote")]
    pub fabrication_note: String,
    #[serde(rename = "keyTerm")]
    pub key_term: String,
}

/// The full authored fixture, in commit order — index `i` here is the SAME
/// index the generated test `a_{i:04}_...` and the snapshot array entry `i`
/// refer to.
pub fn fixture() -> Vec<AdversarialQuestion> {
    serde_json::from_str(FIXTURE).expect("the committed adversarial fixture is well-formed JSON")
}

/// The committed classification snapshot — one label per fixture entry, same
/// order. `"Safe"` (abstained, as every question here should) or the actual
/// [`ChatOutcome`] variant name when it did not.
pub fn snapshot() -> Vec<String> {
    serde_json::from_str(SNAPSHOT).expect("the committed adversarial snapshot is well-formed JSON")
}

/// The registry-driven definitional lexicon set — the SAME selection the
/// CLI/wasm hosts compose at startup.
fn registered_lexicons() -> Vec<Rc<RuntimeOntology>> {
    let mut out = Vec::new();
    for entry in data_sources() {
        let Some(result) = definitional_lexicon_runtime_ontology(entry) else {
            continue;
        };
        let onto = result
            .unwrap_or_else(|e| panic!("registered lexicon {} must materialize: {e}", entry.name));
        out.push(Rc::new(onto));
    }
    out
}

/// Build the production `English ⊕ registered lexicons ⊕ USC(+defines)`
/// composition — the exact reasoner every question in this corpus is routed
/// through, matching `crates/cli/src/main.rs`'s `run_chat` load order. The
/// adversarial "Safety Exhibit Test" corpus must test abstention against the
/// SAME knowledge surface a real user's chat session has — testing against a
/// narrower composition (missing USC/defines, as this reasoner did before)
/// could hide a real safety regression the wider production surface exposes,
/// the same "test harness doesn't match production" gap
/// `crate::caregiver::setup_reasoner`'s own doc now documents.
pub fn setup_reasoner() -> (
    ComposedReasoner,
    &'static pr4xis_domains::cognitive::linguistics::english::English,
) {
    let english = english_loaded();
    let mut lexicons = registered_lexicons();
    assert!(
        !lexicons.is_empty(),
        "the registry must carry at least one definitional chat lexicon"
    );
    if let Some(usc) = crate::caregiver::usc_with_defines_overlay() {
        lexicons.push(usc);
    }
    (ComposedReasoner::new(english, lexicons), english)
}

/// Question `index`'s classification label — `"Safe"` for
/// [`ChatOutcome::Abstained`], or the actual outcome variant name otherwise.
/// Reads [`classification_table`], so the reasoner is not rebuilt per call.
pub fn classify_label(index: usize) -> String {
    classification_table()[index].clone()
}

/// Classify one already-loaded `case` against an already-built `reasoner`.
fn classify_with_reasoner(
    reasoner: &ComposedReasoner,
    english: &'static pr4xis_domains::cognitive::linguistics::english::English,
    case: &AdversarialQuestion,
) -> String {
    let result = pr4xis_chat::process_with_reasoner(english, reasoner, &case.question);
    match result.outcome {
        ChatOutcome::Abstained { .. } => "Safe".to_string(),
        ChatOutcome::Answered => "Answered".to_string(),
        ChatOutcome::Conditional { .. } => "Conditional".to_string(),
        ChatOutcome::RuleResolved { .. } => "RuleResolved".to_string(),
    }
}

/// Every question's label, computed once per process and shared by all the
/// generated tests — the adversarial twin of
/// [`crate::caregiver::classification_table`], for the same reason: a
/// `ComposedReasoner` fits in neither a `static` (`RuntimeOntology` is
/// `!Sync`) nor a `thread_local!` (libtest gives each `#[test]` a fresh
/// thread), so a per-question build costs `setup_reasoner()` once per test.
/// `Vec<String>` is `Sync`, so the labels cross threads while each reasoner
/// stays inside the worker that owns it.
fn classification_table() -> &'static [String] {
    static TABLE: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    TABLE.get_or_init(labels_ordered)
}

/// One label per question, in [`fixture`] order, one reasoner per WORKER.
/// Chunks are yielded, spawned and joined in order, so concatenating each
/// worker's `Vec` in join order reconstructs the original sequence.
fn labels_ordered() -> Vec<String> {
    let cases = fixture();
    let worker_count = std::thread::available_parallelism()
        .map(core::num::NonZeroUsize::get)
        .unwrap_or(1)
        .min(cases.len().max(1));

    let process_chunk = |chunk: &[AdversarialQuestion]| -> Vec<String> {
        let (reasoner, english) = setup_reasoner();
        chunk
            .iter()
            .map(|case| classify_with_reasoner(&reasoner, english, case))
            .collect()
    };

    if worker_count <= 1 {
        return process_chunk(&cases);
    }
    let chunk_size = cases.len().div_ceil(worker_count);
    std::thread::scope(|scope| {
        cases
            .chunks(chunk_size)
            .map(|chunk| scope.spawn(move || process_chunk(chunk)))
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|handle| {
                handle
                    .join()
                    .expect("adversarial classification worker thread panicked")
            })
            .collect()
    })
}

/// THE PER-QUESTION ASSERTION every generated test calls. Compares the
/// question's CURRENT classification against the committed snapshot label —
/// any change in EITHER direction (an improvement to `"Safe"`, a regression
/// away from it, or a lateral shift between unsafe outcome kinds) fails
/// loud, naming exactly which question and what changed. A genuine
/// improvement is fixed by regenerating the snapshot (see
/// `regenerate_adversarial_snapshot` in
/// `tests/adversarial_questions_generated.rs`), never by loosening this
/// assertion.
#[track_caller]
pub fn assert_matches_snapshot(index: usize, expected_label: &str) {
    let actual = classify_label(index);
    let cases = fixture();
    let question = &cases[index].question;
    assert_eq!(
        actual, expected_label,
        "adversarial question {index} ({question:?}) classification changed: snapshot says \
         {expected_label:?}, pipeline now says {actual:?}. If this is a genuine \
         improvement, regenerate the snapshot; if a regression, fix the pipeline."
    );
}
