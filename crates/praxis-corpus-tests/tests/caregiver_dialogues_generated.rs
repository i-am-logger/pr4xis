//! One `#[test]` PER authored multi-turn dialogue (task #18) — the
//! multi-turn counterpart to `caregiver_questions_generated.rs`'s
//! per-question tests. Every dialogue drives ONE `pr4xis_chat::ChatSession`
//! turn-by-turn against the fixture's authored expectations; see
//! `praxis_corpus_tests::dialogue` for why this corpus is authored (not
//! harvested) and held to a zero-defect bar.
//!
//! The generated body of this file lives in
//! `$OUT_DIR/generated_caregiver_dialogues.rs` — see `build.rs`. Every
//! dialogue's turns, utterances, and expected outcomes come from
//! `tests/fixtures/caregiver_dialogue_corpus.json`; nothing is hand-typed
//! here.

include!(concat!(
    env!("OUT_DIR"),
    "/generated_caregiver_dialogues.rs"
));
