//! The CORPUS-SCALE chat-capability suite — the heavy lane where the generated
//! faithfulness battery gets real teeth.
//!
//! The witness-scale [`ChatFaithfullyRealizesTheOntology`] axiom (in
//! `pr4xis-chat`) pins the CLEAN core (define a nominal / affirm an is-a edge /
//! deny the reverse of a strict edge) over the tiny sample graph as a green,
//! gate-swept sentinel. THIS test runs the SAME generic generator + checker over
//! the REAL corpus — full Open English WordNet (every glossed concept, every
//! hypernym edge) composed with a real USC title — including the honesty class
//! (a non-edge of two KNOWN concepts must never be answered "I do not know the
//! words …"). Over the unfixed pipeline it is RED, and its panic message is the
//! per-class breakdown of the reproduced defects (mass-noun / verb-homonym
//! define misses, the realizer-conflation false-negations, …).
//!
//! `require`-gated on a provisioned USC title, so a bare checkout HARD-FAILS
//! naming `pr4xis update usc` — tests do not skip. The full WordNet corpus
//! (`english_loaded`) is fetched the same way; CI provisions both.

use std::rc::Rc;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_chat::capability::{Frames, SampleBudget, evaluate_parallel};
use pr4xis_domains::applied::data_provisioning::registry::data_sources;
use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
use pr4xis_domains::cognitive::linguistics::english::english_loaded;
use pr4xis_domains::formal::meta::grounding::ground_loaded_set;
use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::usc_runtime_ontology;
use pr4xis_domains::social::software::markup::xml::uslm::{UsCode, read_uslm_title};
use praxis_corpus_tests::{load_wordnet_corpus, require, workspace_root};

/// A registered seed for the deterministic sampler — identical run-to-run.
const CORPUS_SEED: u64 = 0x0C0F_FEE0_1234_5678;

/// Load the first provisioned USC title as a `UsCode`, or `None` on a fresh
/// checkout — routed through [`require`] to HARD-FAIL (tests do not skip).
fn first_provisioned_title() -> Option<UsCode> {
    let root = workspace_root();
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
            continue;
        }
        let Ok(source) = std::fs::read(root.join(entry.local_path())) else {
            continue;
        };
        let text = core::str::from_utf8(&source).expect("USLM source is UTF-8");
        let title = read_uslm_title(text).expect("parse title");
        return Some(UsCode::from_uslm_titles_owned(vec![title]));
    }
    None
}

/// THE MONOTONIC RATCHET — the machine form of the "monotonic-or-nothing"
/// gate every chat fix has been measured against by hand. Each class's defect
/// count must never EXCEED its committed ceiling; a fix that lowers a class
/// re-derives the ceilings with the command below and ratchets them DOWN in
/// the same commit. (The zero-defect end-state is asserted by the red-by-design
/// [`chat_faithfully_realizes_the_loaded_corpus`] below; THIS test is the one
/// that must stay green on every commit.)
///
/// Re-derive: `cargo test --manifest-path crates/praxis-corpus-tests/Cargo.toml
/// --test chat_capability -- --nocapture` and read the printed breakdown (run
/// twice; the weighted chart is deterministic, so the runs must agree).
///
/// Ceilings as of 2026-07-12 (task 2.5, Slice B: the adverb quoted-mention
/// question frame — history: 2681 baseline → 2387 FIX-C weighted chart →
/// 1403 FIX-D multi-word dual reading → 590 R-1 gerundial verb frames → 466
/// R-2A take(5) deleted → 442 R-2C predicative alternative → 203 task 2.3
/// gate-invisible-hole fix). 203 → 111 (define): an ADVERB concept has no
/// grammatical bare-copula definitional reading ("*what is a quickly",
/// "*what is quickly" are both ungrammatical, unlike a noun/adjective's "what
/// is a dog"/"what is able") — `wh_definition` was generating one anyway.
/// `Frames::wh_mean` asks "what does \u{201C}{label}\u{201D} mean" instead: a
/// new object-question do-support frame (`svo::wh_what_object` +
/// `svo::does_support` + `svo::bare_transitive_verb`) over a quoted-mention
/// NP recognized by a loaded Unicode Pi/Pf glyph vocabulary
/// (`lambek::quote_glyphs`, `tokenize::collapse_quoted_spans`), routed
/// through the existing single-entity Content-illocution define path
/// unchanged. is-a/directional-No/non-edge are untouched — a pure define-class
/// win.
/// 111 → 42 (task 2.6, Slice D: the fold-on-miss case-folding index).
/// 17,272 of 131,798 loaded WordNet surfaces carry an uppercase letter
/// (12,280 of those multi-word) — the tokenizer lowercases every query
/// surface before the exact-case WordIndex lookup ever runs, so a
/// capitalized lemma ("Section Eight", "Turkish bath") was unreachable by
/// its own generated question. `LexicalReasoner::lookup_case_folded`
/// (embedded `English`, delegated by `ComposedReasoner`) recovers it via
/// the loaded Unicode simple case-folding table
/// (`orthography::case_folding`, `CaseFolding.txt` status C+S — never
/// `str::to_lowercase`), tried as a fallback tier in `resolve_surface` and
/// in `collapse_multiword_surfaces`'s classify closure (so a multi-word
/// capitalized lemma collapses into one token at all). Residual 42: distinct
/// gloss-miss classes unrelated to casing (predicate-adjective/verb senses
/// like "deadly", "full", "backed", "forward"; "O.K." specifically has BOTH
/// a capitalized AND a genuinely separate lowercase lemma, so its exact
/// match already succeeds before the fold fallback would even run).
///
/// 42 → 34 (head-lemma writtenRep variants: `ComposedReasoner` now also
/// indexes every multi-word loaded surface under prefix + each dual-route
/// analysis of its FINAL word — English number is a head inflection,
/// Huddleston & Pullum 2002 Ch. 5 — so a loaded collocation's generated
/// question resolves through its own singular/lemma occurrence form; 8
/// define gloss-misses over USC multi-word headings clear).
const DEFINE_CEILING: usize = 34;
/// 42 → 4 (task 2.6, Slice D, the SAME fold-on-miss fix — the affirmative
/// is-a sentence's subject determiner now byte-matches once "Section
/// Eight"/"Turkish bath"/"Russian bank" resolve through the fold index
/// instead of the old case-mismatched miss the 15 → 42 raise below
/// documents).
///
/// 4 → 0 (the same head-lemma writtenRep variants as the 42 → 34 define
/// ratchet above: the residual affirmative is-a misses were multi-word
/// subjects whose inflected occurrence form missed the exact overlay
/// lookup).
const IS_A_CEILING: usize = 0;
/// 216 → 213 (task 2.6, Slice D, the same fold-on-miss fix: 3 directional-No
/// provable-negation cases over the now-resolving capitalized multi-word
/// concepts).
///
/// Raised 15 → 42 in the SAME commit that fixed the checker, not the
/// pipeline: `Expected::Affirms` used to pass on a bare substring match of
/// the target's label anywhere in the response, which could not tell a real
/// affirmation from an accidental mention. It now requires the realizer's
/// own precise affirmative surface (`sentence_relation(subject, target,
/// None)`), the same string production actually emits. That closed the
/// gate-invisible hole this ceiling exists to catch — and immediately
/// surfaced 27 previously-invisible real defects, all one class: a
/// capitalized multi-word subject ("Turkish bath", "Section Eight", "Russian
/// bank") gets a DIFFERENT determiner at generation time (from the WordNet
/// lemma's own capitalization) than at answer time (from whatever case the
/// multi-word tokenizer's collapse step returns), so the affirmation's
/// subject-determiner does not byte-match. This is the SAME defect Slice D
/// (`.notes/chat-fix-c-build-state.md`) already tracks for a loaded
/// case-folding source — the pipeline was already broken here, the old
/// bare-substring check just could not see it. This is a monotonic
/// IMPROVEMENT in what the ratchet detects, not a pipeline regression.
///
/// 213 → 212 (the head-lemma writtenRep variants again: one provable
/// negation over a now-resolving multi-word subject realizes).
const DIRECTIONAL_NO_CEILING: usize = 212;
const NON_EDGE_CEILING: usize = 8;

/// Build the production `English ⊕ (provisioned USC title, grounded)`
/// composition — the exact reasoner every generated probe case is routed
/// through. Cheap enough (a USLM parse + materialize + one grounding pass,
/// no per-node `defines_lens` pipeline — see `usc_runtime_ontology`'s own
/// doc contrasting it with `usc_runtime_ontology_with_defines`) to call
/// fresh per worker thread in [`corpus_breakdown`]'s parallel evaluation,
/// mirroring `praxis_corpus_tests::caregiver::setup_reasoner`.
fn build_reasoner(
    english: &'static pr4xis_domains::cognitive::linguistics::english::English,
) -> ComposedReasoner {
    let mut set = Vec::new();
    if let Some(usc) = first_provisioned_title()
        && let Ok(usc_onto) = usc_runtime_ontology(&usc, OntologyName::new_static("usc"))
    {
        set.push(Rc::new(usc_onto));
        let _ = ground_loaded_set(&mut set, english);
    }
    ComposedReasoner::new(english, set)
}

/// The corpus breakdown, computed ONCE per test-binary process and shared by
/// the ratchet and the red-by-design gate below (the multi-minute evaluation
/// must not run twice; both tests read the same deterministic result).
///
/// Parallelized via [`evaluate_parallel`] (`std::thread::scope`, the same
/// pattern `pr4xis_domains`'s `compute_defines_overlay` and
/// `praxis_corpus_tests::caregiver::corpus_breakdown` already prove): each
/// worker thread builds its own `ComposedReasoner` via [`build_reasoner`]
/// (a `ComposedReasoner` is `!Sync` — `Vec<Rc<RuntimeOntology>>` — so it
/// cannot be shared by reference across threads) and evaluates its own
/// chunk of generated probe cases, with results joined back in original
/// chunk order for a byte-identical breakdown to the serial `evaluate`.
fn corpus_breakdown() -> &'static pr4xis_chat::capability::FailureBreakdown {
    use std::sync::OnceLock;
    static BREAKDOWN: OnceLock<pr4xis_chat::capability::FailureBreakdown> = OnceLock::new();
    BREAKDOWN.get_or_init(|| {
        let wn = load_wordnet_corpus();
        require(wn.english(), "english_wordnet");
        let english = english_loaded();
        let frames = Frames::from_realization();
        evaluate_parallel(
            english,
            || build_reasoner(english),
            &frames,
            SampleBudget::corpus(CORPUS_SEED),
        )
    })
}

#[test]
fn chat_capability_never_regresses_the_committed_ceilings() {
    let breakdown = corpus_breakdown();
    eprintln!("CHAT CAPABILITY (ratchet): {}", breakdown.summary());
    for (class, count, ceiling) in [
        ("define", breakdown.define, DEFINE_CEILING),
        ("is-a", breakdown.is_a, IS_A_CEILING),
        (
            "directional-No",
            breakdown.directional_no,
            DIRECTIONAL_NO_CEILING,
        ),
        ("non-edge", breakdown.non_edge, NON_EDGE_CEILING),
    ] {
        assert!(
            count <= ceiling,
            "REGRESSION: the {class} class rose to {count} (committed ceiling {ceiling}).\n\
             A change may only move a class DOWN (then ratchet the ceiling down \
             in the same commit) — monotonic-or-nothing.\nFull breakdown: {}",
            breakdown.summary(),
        );
    }
}

/// THE CORPUS GATE: over full WordNet ⊕ a real USC title, EVERY (question,
/// expected) the generic generator derives from the loaded graph must have the
/// chat pipeline realize the expected property — the definition carries the
/// gloss, an is-a edge affirms, the reverse of a strict edge denies, and a
/// non-edge over KNOWN concepts denies-or-abstains WITHOUT ever claiming not to
/// know a known word. This is RED on the unfixed pipeline; the panic message is
/// the per-class defect breakdown.
#[test]
fn chat_faithfully_realizes_the_loaded_corpus() {
    // Full WordNet ⊕ USC via the shared once-per-process evaluation (see
    // `corpus_breakdown`); a bare checkout HARD-FAILS inside it naming
    // `pr4xis update english_wordnet` — tests do not skip.
    let breakdown = corpus_breakdown();

    // The RED breakdown — printed even on the (future) green run so the corpus
    // coverage is visible.
    eprintln!("CHAT CAPABILITY (corpus): {}", breakdown.summary());
    for line in &breakdown.examples {
        eprintln!("  {line}");
    }

    assert!(
        breakdown.is_empty(),
        "the chat pipeline is UNFAITHFUL to the loaded ontology on {} generated cases:\n{}\n{}",
        breakdown.total,
        breakdown.summary(),
        breakdown.examples.join("\n"),
    );
}
