//! Shared machinery for the caregiver/HCBS-workforce question corpus —
//! factored out of the harness so BOTH the build-time codegen
//! (`build.rs`, which emits one `#[test]` per question into
//! `tests/caregiver_questions_generated.rs`) and the snapshot-regeneration
//! tool below can reuse the exact same fixture loading, reasoner setup, and
//! gap-classification logic. No question text or expected label is ever
//! hand-typed in Rust source — both come from the committed JSON fixture and
//! its companion snapshot; the generated `.rs` is a mechanical, re-derivable
//! projection of that data.

use std::rc::Rc;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_chat::ChatOutcome;
use pr4xis_domains::applied::data_provisioning::chat_lexicons::definitional_lexicon_runtime_ontology;
use pr4xis_domains::applied::data_provisioning::registry::data_sources;
use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
use pr4xis_domains::cognitive::linguistics::english::{English, LexicalReasoner, english_loaded};
use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::usc_runtime_ontology_from_cached_defines;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::prx::load_usc_defines_overlay_from_disk;
use pr4xis_runtime::ontology::RuntimeOntology;
use serde::Deserialize;

const FIXTURE: &str = include_str!("../tests/fixtures/caregiver_question_corpus.json");
const SNAPSHOT: &str = include_str!("../tests/fixtures/caregiver_question_corpus.snapshot.json");

/// The committed capability ratchet: per-class ceilings that may only FALL,
/// a Green count that may only RISE, and the corpus size both are measured
/// against.
///
/// Lives here, in the library, rather than in the gate that enforces it,
/// because three consumers must read one number: the gate
/// (`tests/caregiver_capability_ratchet.rs`), the snapshot regenerator
/// (`tests/caregiver_questions_generated.rs`, which publishes these into
/// `docs/caregiver-corpus-status.json` for the live demo to draw), and the
/// demo itself. A ceiling the page renders and a ceiling CI enforces that
/// are separately typed can silently disagree; one constant cannot.
///
/// `GREEN` and `TOTAL` are what make the ceilings mean anything. Four
/// downward-only ceilings alone are satisfied by DELETING hard questions —
/// every gap class falls, and capability has not moved. Pinning the corpus
/// size and flooring the Green count closes that: a commit may only shrink
/// the corpus by also declaring it, and may only lower Green by failing.
///
/// Every value is measured, never chosen — regenerate the snapshot (the
/// command is published in `docs/caregiver-corpus-status.json`'s own
/// `regenerated_by` field) and read them off.
pub mod ratchet {
    /// Corpus size, pinned. Changing it is a deliberate, declared act.
    pub const TOTAL: usize = 4219;
    /// Green floor — the count may only rise from here.
    pub const GREEN: usize = 144;
    /// Per-gap-class ceilings — each may only fall.
    /// Provenance for every movement is in the ratchet gate's module docs.
    pub const MISSING_TERM: usize = 2786;
    pub const UNPARSED_KNOWN_TERM: usize = 1223;
    pub const OVER_ANSWERED: usize = 0;
    pub const POSSIBLE_MISROUTE: usize = 66;
}

#[derive(Deserialize)]
pub struct HarvestedQuestion {
    pub question: String,
    #[allow(dead_code)]
    pub source: String,
    #[allow(dead_code)]
    pub track: String,
    #[allow(dead_code)]
    #[serde(rename = "topicCategory")]
    pub topic_category: String,
    #[serde(rename = "praxisCapability")]
    pub praxis_capability: String,
    #[allow(dead_code)]
    #[serde(rename = "expectedBehavior")]
    pub expected_behavior: String,
    #[serde(rename = "keyTerm")]
    pub key_term: String,
}

/// The full harvested-question fixture, in commit order — index `i` here is
/// the SAME index the generated test `q_{i:04}_...` and the snapshot array
/// entry `i` refer to.
pub fn fixture() -> Vec<HarvestedQuestion> {
    serde_json::from_str(FIXTURE).expect("the committed fixture is well-formed JSON")
}

/// The committed classification snapshot — one label per fixture entry, same
/// order. `"Green"` or a [`GapClass`] variant name.
pub fn snapshot() -> Vec<String> {
    serde_json::from_str(SNAPSHOT).expect("the committed snapshot is well-formed JSON")
}

/// The registry-driven definitional lexicon set — the SAME selection the
/// CLI/wasm hosts compose at startup. ~13ms, two small `.prx` lexicons.
///
/// Uncached: `RuntimeOntology` is `!Sync`, so no `static` can hold one, and a
/// `thread_local!` never hits because libtest runs each `#[test]` on a fresh
/// thread. Callers amortize by building one reasoner per worker and
/// classifying a whole chunk against it — see [`corpus_labels_ordered`].
pub fn registered_lexicons() -> Vec<Rc<RuntimeOntology>> {
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

/// The U.S. Code corpus, WITH its `defines`-grounding overlay merged in —
/// the SAME bespoke (non-registry-kind-dispatched) load `crates/cli/src/
/// main.rs`'s `run_chat` performs at startup, mirrored here so the corpus
/// test harness exercises the identical composed reasoner production chat
/// answers through — not a lookalike missing the USC/defines data
/// entirely. Before this, `setup_reasoner()` composed ONLY
/// `registered_lexicons()` (the registry-kind-dispatched WN-LMF lexicons),
/// so a real statutory definition grounded via
/// `statute_structure::grounding::defines_lens` was unreachable from every
/// corpus-gate test even after `ComposedReasoner`/`define_word` learned to
/// consult it — the SAME orphaned-mechanism failure mode one layer up.
///
/// `None` when the USC corpus isn't loaded on disk at all (a fresh checkout
/// with no `pr4xis update`/`compile` run) — the corpus behaves exactly as
/// it did before this function existed, never a hard failure over an
/// optional, fetch-provisioned corpus.
///
/// 2.0–2.6s per call, and the dominant term in `setup_reasoner()`'s 2.71s:
/// the underlying corpora are parse-once statics (`english_loaded()` is 110ns
/// warm), but projecting them into a `RuntimeOntology` is redone on every
/// call. Build a reasoner once per worker and classify a whole chunk against
/// it ([`corpus_labels_ordered`]); never once per question.
pub fn usc_with_defines_overlay() -> Option<Rc<RuntimeOntology>> {
    let usc = pr4xis_domains::social::software::markup::xml::uslm::corpus::loaded();
    if usc.section_count().value == 0.0 {
        return None;
    }
    let overlay = load_usc_defines_overlay_from_disk(&crate::workspace_root());
    let onto = usc_runtime_ontology_from_cached_defines(usc, OntologyName::new("usc"), &overlay)
        .unwrap_or_else(|e| panic!("USC corpus with defines overlay must materialize: {e}"));
    Some(Rc::new(onto))
}

/// Build the production `English ⊕ registered lexicons ⊕ USC(+defines)`
/// composition — the exact reasoner every question in this corpus is
/// routed through, matching `crates/cli/src/main.rs`'s `run_chat` load
/// order exactly (LegalSources base aside, which this test corpus's
/// questions never need).
pub fn setup_reasoner() -> (ComposedReasoner, &'static English) {
    let english = english_loaded();
    let mut lexicons = registered_lexicons();
    assert!(
        !lexicons.is_empty(),
        "the registry must carry at least one definitional chat lexicon"
    );
    if let Some(usc) = usc_with_defines_overlay() {
        lexicons.push(usc);
    }
    (ComposedReasoner::new(english, lexicons), english)
}

/// Cheap on-disk presence check for the corpus this crate needs (avoids the
/// full 89 MB WN-LMF XML re-parse [`crate::load_wordnet_corpus`] would pay — that
/// function is for tests that need the PARSED source, not merely a presence
/// gate). Panics naming `pr4xis update english_wordnet` when absent — tests
/// do not skip.
pub fn require_english_wordnet_present() {
    let path = crate::domains_data_dir().join("wordnet/english-wordnet-2025.xml");
    assert!(
        path.exists(),
        "corpus `english_wordnet` not on disk — run `pr4xis update english_wordnet` to fetch it; \
         tests do not skip"
    );
}

/// A red case's mechanically-derived gap class — never a per-question special
/// case, always read off the structured (expected, actual, unresolved,
/// response) tuple.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapClass {
    /// Expected an answer (define/is_a/directional); the pipeline abstained;
    /// the question's own `keyTerm` resolves to NO loaded concept anywhere
    /// (embedded English or a registered lexicon) — the term is genuinely
    /// absent, a real coverage gap in the loaded lexicon/taxonomy.
    MissingTerm,
    /// Expected an answer; the pipeline abstained; but `keyTerm` DOES resolve
    /// to a loaded concept — the real-world sentence's surface/grammar
    /// didn't route to it (a tokenizer/chart/collapse gap over authentic
    /// phrasing, distinct from the term itself being unloaded).
    UnparsedKnownTerm,
    /// Expected abstention (procedural/personal/medical/legal-counsel/
    /// emotional questions the loaded ontologies cannot ground); the
    /// pipeline answered anyway — see `project_chat_illocution_routing_gap`
    /// memory for the exact-line root-cause diagnosis (Track 2.3).
    OverAnswered,
    /// Expected an answer and got one, but the response text never mentions
    /// the asked-about `keyTerm` — a weak signal the pipeline answered a
    /// different, unintended concept.
    PossibleMisroute,
}

impl GapClass {
    pub fn label(self) -> &'static str {
        match self {
            GapClass::MissingTerm => "MissingTerm",
            GapClass::UnparsedKnownTerm => "UnparsedKnownTerm",
            GapClass::OverAnswered => "OverAnswered",
            GapClass::PossibleMisroute => "PossibleMisroute",
        }
    }
}

/// Classify a case's outcome into the gap taxonomy — mechanically derived
/// from (actual outcome, response text, whether `keyTerm` resolves anywhere
/// in the loaded set), never a per-question special case. `None` means green.
///
/// EVERY question in this corpus expects a correct, grounded answer. There is
/// no "out of scope" population, and this function no longer takes an
/// `expects_answer` flag.
///
/// It used to. The corpus tagged 4,108 of 4,617 rows `out_of_scope_abstain`
/// and this function scored *declining* them as green, which made 4,040 of
/// the 4,177 "passes" abstentions and put the headline pass rate at 90.47%
/// while the engine actually answered 162 questions. That is a metric
/// measuring fit-to-TOOL, not fit-to-NEED: `out_of_scope_abstain` was never
/// an ontology concept — it entered as a bare JSON string in the harness's
/// own introducing commit (`608036d3`) and was consumed by a
/// `matches!(cap, "define" | "is_a" | "directional")` string comparison, i.e.
/// exactly the string-matching-drives-behaviour pattern this repository
/// forbids everywhere else. A caregiver asking "Am I eligible for the VA
/// Program of Comprehensive Assistance for Family Caregivers?" is owed the
/// enumerated 38 CFR 71.20 criteria and a named missing fact, not a refusal
/// scored as success.
///
/// So the taxonomy is now: an answer naming the asked-about term is green;
/// anything else names WHY it is not, and is RED. `OverAnswered` is retained
/// as a variant only so already-committed snapshots keep deserializing — it
/// is unreachable from here now, because "answered when it should have
/// declined" is not a defect any more; answering the WRONG thing is, and that
/// is `PossibleMisroute`.
pub fn classify_case(
    outcome: &ChatOutcome,
    response: &str,
    key_term: &str,
    key_term_known: bool,
) -> Option<GapClass> {
    let key_term_norm = key_term.to_lowercase();
    let response_norm = response.to_lowercase();
    match outcome {
        ChatOutcome::Answered => {
            if key_term_norm.is_empty() || response_norm.contains(&key_term_norm) {
                None
            } else {
                Some(GapClass::PossibleMisroute)
            }
        }
        ChatOutcome::Abstained { .. } => {
            if key_term_known {
                Some(GapClass::UnparsedKnownTerm)
            } else {
                Some(GapClass::MissingTerm)
            }
        }
        // A `Conditional` names the governing cited rule AND the exact fact it
        // is missing. For a rule-governed question ("am I eligible for X") that
        // IS the correct grounded answer shape — the criteria plus what it
        // would need to know about the asker — not a refusal. Green, and the
        // one honest way this corpus's personal-eligibility questions ever go
        // green.
        ChatOutcome::Conditional { .. } => None,
        // Only `session::ChatSession::ask`'s multi-turn resume path produces
        // this; the single-turn harness never drives it. Kept exhaustive
        // rather than a wildcard.
        ChatOutcome::RuleResolved { .. } => None,
    }
}

/// Classify one already-loaded `case` against an already-built `reasoner` —
/// the shared inner logic behind both [`classify_label`] (which builds a
/// which reads the shared table, for the one-question-per-generated-test
/// lane) and [`corpus_breakdown`]'s parallel workers (which build one
/// reasoner per worker thread and classify their whole index range against
/// it, avoiding the 2.71s `setup_reasoner()` rebuild on every question).
fn classify_with_reasoner(
    reasoner: &ComposedReasoner,
    english: &'static English,
    case: &HarvestedQuestion,
) -> String {
    let result = pr4xis_chat::process_with_reasoner(english, reasoner, &case.question);
    // BOTH `lookup` AND `lookup_case_folded`, not either alone — the SAME
    // OR-both idiom `process_with_reasoner`'s own multi-word-surface
    // classify closure already uses (`chat/src/lib.rs`, `collapse_
    // multiword_surfaces`'s callback: "!reasoner.lookup(s).is_empty() ||
    // !reasoner.lookup_case_folded(s).is_empty()"), for the identical
    // reason: `ComposedReasoner::lookup_case_folded` DELEGATES to the
    // wrapped English substrate ONLY (`composed.rs`'s own doc: "a loaded
    // surface whose OWN casing needs folding is out of scope here") — it
    // never sees the LOADED curated-lexicon overlay `lookup` checks first.
    // Plain `lookup` alone misses a genuinely case-marked WordNet lemma
    // whose own key does not fold to itself ("Down syndrome" — WordNet
    // spells it capitalized); `lookup_case_folded` ALONE misses a
    // caregiver-domain term that lives ONLY in the loaded overlay, never
    // in WordNet at all (e.g. an HCBS-compliance term with no WordNet
    // headword) — a confirmed real regression this OR-both fix corrects
    // (a bare `lookup_case_folded` swap flipped ~108 rows' classification
    // from `UnparsedKnownTerm` to a fabricated `MissingTerm`, caught by
    // the caregiver_capability_ratchet gate). The classifier's job is
    // "does this concept exist ANYWHERE the composed reasoner already
    // resolves it" — the union of both lookup paths, matching every other
    // call site in this codebase that needs that same union.
    let key_term_norm = case.key_term.to_lowercase();
    let key_term_known = !case.key_term.is_empty()
        && (!reasoner.lookup(&key_term_norm).is_empty()
            || !reasoner.lookup_case_folded(&key_term_norm).is_empty());
    let verdict = classify_case(
        &result.outcome,
        &result.response,
        &case.key_term,
        key_term_known,
    );
    match verdict {
        None => "Green".to_string(),
        Some(class) => class.label().to_string(),
    }
}

/// Run question `index` through the production reasoner and return its
/// current classification label (`"Green"` or a [`GapClass`] name).
pub fn classify_label(index: usize) -> String {
    classification_table()[index].clone()
}

/// Every question's label, computed once per process and shared by all 4,219
/// generated tests. The first test to call it runs the whole parallel pass;
/// the rest block on the `OnceLock` and then index into the result.
///
/// The RESULTS are what is shared, because the reasoner cannot be: a 2.71s
/// `ComposedReasoner` can live in neither a `static` (`RuntimeOntology` is
/// `!Sync`) nor a `thread_local!` (libtest gives each `#[test]` a fresh
/// thread), so one-`#[test]`-per-question otherwise forces 4,219 rebuilds.
/// `Vec<String>` is `Sync`, so the labels cross threads while every
/// `RuntimeOntology` stays inside the [`corpus_labels_ordered`] worker that
/// owns it.
fn classification_table() -> &'static [String] {
    static TABLE: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    TABLE.get_or_init(corpus_labels_ordered)
}

/// THE PER-QUESTION ASSERTION every generated test calls. Requires the
/// question's CURRENT classification to be `"Green"` — a real capability gap
/// (`MissingTerm`, `UnparsedKnownTerm`, `OverAnswered`, `PossibleMisroute`)
/// is a FAILING test, by name, in ordinary `cargo test` output, not a value
/// silently absorbed into a frozen "current state" snapshot. This is the
/// honest signal a research project's test suite should give: "3707 passed;
/// 910 failed" IS the real, standard-tooling-legible scorecard, not a
/// artifact only visible via a separate `--ignored` regeneration run's log
/// line. (Previously this function compared against a committed snapshot
/// label instead of `"Green"` directly — replaced because a real gap that
/// matches its own frozen "still red" snapshot value read as a PASSING
/// test, which is backwards: if there is a gap, the test must not be
/// green.) The monotonic regression GATE lives separately, in
/// `caregiver_capability_ratchet.rs` — an aggregate per-class ceiling check
/// (mirroring `chat_capability.rs`'s ratchet), so CI blocks only on a true
/// regression, not on the (large, expected, honestly-tracked) backlog of
/// not-yet-built capability this suite exists to make visible.
#[track_caller]
pub fn assert_is_green(index: usize) {
    let actual = classify_label(index);
    let cases = fixture();
    let question = &cases[index].question;
    assert_eq!(
        actual, "Green",
        "question {index} (\"{question}\") is not yet answerable: the pipeline classifies it \
         {actual:?}. This is an honest, tracked capability gap, not a bug in this test — see \
         `praxis_corpus_tests::caregiver::GapClass` for what {actual:?} means, and \
         `caregiver_capability_ratchet.rs` for the aggregate regression gate this individual \
         test does not itself enforce."
    );
}

/// The full corpus's classification breakdown, computed ONCE by iterating
/// every question — shared by `regenerate_caregiver_snapshot` (which prints
/// it as a human-readable progress artifact) and the aggregate ratchet gate
/// in `caregiver_capability_ratchet.rs` (which enforces it never regresses).
/// Keys are `"Green"` or a [`GapClass`] label; values are counts.
///
/// Parallelized across `std::thread::available_parallelism()` worker threads
/// (the same native `std::thread::scope` pattern already proven in
/// `pr4xis_domains`'s `compute_defines_overlay`, no new dependency): each
/// question's classification (tokenize → chart parse → Montague composition
/// → pragmatics → realize, via `classify_with_reasoner`) is independent —
/// no shared mutable state — so this is an embarrassingly parallel per-
/// question sweep. `ComposedReasoner` holds `Vec<Rc<RuntimeOntology>>`,
/// which is `!Sync` (an `Rc`, not the corpus data underneath), so a single
/// reasoner cannot be shared by reference across worker threads; instead
/// each worker calls `setup_reasoner()` ONCE for its whole chunk (already
/// documented as "~10ms, two small `.prx` lexicons: safe to call fresh") —
/// a net efficiency win over the old per-question rebuild this replaces, not
/// just a parallel-safety workaround. The per-worker `BTreeMap<String,
/// usize>` tally is order-independent (pure counts keyed by classification
/// label) — merged by summing matching keys across workers, which is
/// exactly equal to the serial single-map accumulation regardless of chunk
/// boundaries or thread scheduling order.
pub fn corpus_breakdown() -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for label in corpus_labels_ordered() {
        *counts.entry(label).or_insert(0) += 1;
    }
    counts
}

/// The full corpus's classification, ONE label per question, in the SAME
/// order [`fixture`] yields them — the ordered twin of [`corpus_breakdown`]
/// (which this function now backs), for callers that need the per-question
/// sequence itself (`regenerate_caregiver_snapshot`'s committed snapshot
/// file), not just the aggregate tally.
///
/// Same parallel structure [`corpus_breakdown`] originally used directly
/// (`std::thread::scope`, one `setup_reasoner()` per WORKER not per
/// question — a real fix, not a style preference: the naive per-question
/// `classify_label` this replaces re-parses the whole corpus and rebuilds a
/// fresh `ComposedReasoner` on EVERY call, ~4600 times serially, which is
/// why `regenerate_caregiver_snapshot` used to take 20+ minutes wall-clock
/// against this same function's ~3 minutes). `cases.chunks(chunk_size)`
/// yields chunks in original order, workers are spawned and joined in that
/// same order, so `flat_map`-concatenating each worker's per-chunk `Vec` in
/// join order reconstructs the exact original per-question sequence — no
/// index bookkeeping needed.
pub fn corpus_labels_ordered() -> Vec<String> {
    require_english_wordnet_present();
    let cases = fixture();

    let worker_count = std::thread::available_parallelism()
        .map(core::num::NonZeroUsize::get)
        .unwrap_or(1)
        .min(cases.len().max(1));

    let process_chunk = |chunk: &[HarvestedQuestion]| -> Vec<String> {
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
                    .expect("classification worker thread panicked")
            })
            .collect()
    })
}
