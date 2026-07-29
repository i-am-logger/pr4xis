//! Plain-language OUTCOME-EXPLANATION realization frames (the "Why?" layer),
//! carried as LOADED data and interpreted — the SECOND realization of a turn: a
//! plain-language gloss of WHY the engine produced its outcome, sibling of the
//! primary response ([`abstain_frames`](super::abstain_frames) /
//! [`conditional_frames`](super::conditional_frames)), done the praxis way
//! (frame-as-data), NOT an inline `format!` literal in the realizer.
//!
//! Mirrors [`abstain_frames`](super::abstain_frames) exactly: the committed
//! content-addressed `.prx` is `include_bytes!`-embedded, decoded through the
//! generalized raw-source gate
//! ([`raw_source_text_embedded`](crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded)),
//! parsed into `(frame_id, template)` rows, and cached in a process `OnceLock`
//! (`std`) or rebuilt by value (`no_std`). The realizer
//! ([`realize::realize_why`](super::realize::realize_why)) reads the template and
//! fills its typed slots ([`ontology_labels`](super::ontology_labels)-projected
//! `{sources}`, the unresolved `{terms}`) — it never authors the discourse frame
//! in code.
//!
//! Only the frames whose PRIMARY response does not already carry its own reason
//! get a row: `AssertKnowledge` (grounded / ungrounded provenance split),
//! `UnknownVocabulary`, `SuggestInterpretation`, `AdmitLimitation`. The
//! self-explaining frames (`UnprovenRelation`, `RuleAwaitingFact`, `RuleApplies`,
//! `RuleDoesNotApply`, `Comparison`, `PhaticReturn`) get NO row by design — their
//! primary surface already states the reason, so `realize_why` returns `None` and
//! the renderer shows no Why? panel.
//!
//! Cite: Reiter & Dale (2000) *Building Natural Language Generation Systems*
//! (CUP), the realization stage; Reiter (1978) closed-world assumption (the
//! honest boundary of the system's own knowledge).

#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// The committed explain-frames projection `.prx` — the content-addressed
/// envelope carrying the `frame_id<TAB>template` table bytes. The raw `.tsv` is
/// authored source-of-truth (git-tracked, EXCLUDED from the published crate);
/// only this `.prx` is committed + embedded and ships. Loaded through the
/// generalized raw-source gate (content-address hash + `raw_hash::verify`),
/// feature-independent so the table builds on default, `no_std` and wasm.
const EXPLAIN_FRAMES_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/grammar/explain-frames.prx"
));

/// The registry `name@version` key the committed `.prx` is pinned under (see
/// `[sources.explain_frames]` in `praxis.toml`).
const NAME: &str = "explain_frames";
const VERSION: &str = "2026";

/// The answered-from-a-loaded-source explanation — the surface for
/// [`ResponseFrame::AssertKnowledge`](super::response::ResponseFrame::AssertKnowledge)
/// when the turn reasoned over one or more loaded ontologies (non-empty
/// `reasoned_over`). Fills the `{sources}` slot.
pub const ASSERT_KNOWLEDGE_GROUNDED: &str = "assert_knowledge_grounded";
/// The answered-from-the-built-in-substrate explanation — the surface for
/// [`ResponseFrame::AssertKnowledge`](super::response::ResponseFrame::AssertKnowledge)
/// when `reasoned_over` is empty (no loaded `.prx` was needed). No slot.
pub const ASSERT_KNOWLEDGE_UNGROUNDED: &str = "assert_knowledge_ungrounded";
/// The vocabulary-gap abstention explanation, naming the unresolved `{terms}` —
/// the surface for
/// [`ResponseFrame::UnknownVocabulary`](super::response::ResponseFrame::UnknownVocabulary).
pub const UNKNOWN_VOCABULARY: &str = "unknown_vocabulary";
/// The vocabulary-gap explanation for the degenerate case where no specific
/// unresolved surface is recoverable — the `UnknownVocabulary` surface with no
/// `{terms}` slot.
pub const UNKNOWN_VOCABULARY_UNTERMED: &str = "unknown_vocabulary_untermed";
/// The recognized-but-unparsed abstention explanation — the surface for
/// [`ResponseFrame::SuggestInterpretation`](super::response::ResponseFrame::SuggestInterpretation).
pub const SUGGEST_INTERPRETATION: &str = "suggest_interpretation";
/// The DEFINITION-PROVENANCE explanation, naming the lexicon that wrote a
/// recited gloss (`{sources}`) and the documents it was written FROM
/// (`{citations}`) — the surface
/// [`realize_definition_provenance`](super::realize::realize_definition_provenance)
/// fills. Not keyed by `ResponseFrame`: this is a SECOND channel about the same
/// turn, orthogonal to the communicative goal, so it has its own row rather than
/// competing for one frame's template.
pub const DEFINITION_AUTHORED_FROM: &str = "definition_authored_from";
/// The LABEL naming the definition-provenance channel in a provenance panel —
/// loaded rather than typed into a renderer, so the surface that distinguishes
/// "authored from" from "reasoned over" is data like every other surface.
pub const DEFINITION_AUTHORED_FROM_LABEL: &str = "definition_authored_from_label";
/// The nothing-resolved abstention explanation — the surface for
/// [`ResponseFrame::AdmitLimitation`](super::response::ResponseFrame::AdmitLimitation).
pub const ADMIT_LIMITATION: &str = "admit_limitation";

/// Parse the loaded TSV into `(frame_id, template)` rows — the generic
/// interpreter (never a per-frame arm). Comment (`#`) and blank lines and the
/// header row (`frame_id`) are skipped.
fn build_table() -> Vec<(String, String)> {
    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    let tsv = raw_source_text_embedded(NAME, VERSION, EXPLAIN_FRAMES_PRX);
    let mut rows: Vec<(String, String)> = Vec::new();
    for line in tsv.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.split('\t');
        let Some(frame_id) = cols.next().map(str::trim) else {
            continue;
        };
        let Some(template) = cols.next() else {
            continue; // header row (`frame_id` with no template column)
        };
        if frame_id.is_empty() || frame_id == "frame_id" {
            continue;
        }
        rows.push((frame_id.to_string(), template.to_string()));
    }
    rows
}

/// The loaded explain-frame table, cached for the process (`std`).
#[cfg(feature = "std")]
fn explain_table() -> &'static Vec<(String, String)> {
    use std::sync::OnceLock;
    static TABLE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    TABLE.get_or_init(build_table)
}

/// The raw template string for a frame id from the loaded table, or `None` if the
/// table carries no such row. First-match (the loader keeps insertion order); the
/// no-duplicate-frame_id test below guarantees the match is unique.
pub fn template(frame_id: &str) -> Option<String> {
    #[cfg(feature = "std")]
    let rows = explain_table().clone();
    #[cfg(not(feature = "std"))]
    let rows = build_table();
    rows.into_iter()
        .find(|(id, _)| id == frame_id)
        .map(|(_, t)| t)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn the_table_loads_every_explain_frame() {
        // Each frame id the realizer keys on must carry a row, and each row's
        // slots are the typed ones the realizer fills — the discourse frame is
        // loaded data, not composed in code.
        let grounded = template(ASSERT_KNOWLEDGE_GROUNDED).expect("grounded row");
        assert!(
            grounded.contains("{sources}"),
            "grounded template carries the sources slot"
        );
        let ungrounded = template(ASSERT_KNOWLEDGE_UNGROUNDED).expect("ungrounded row");
        assert!(
            !ungrounded.contains('{'),
            "ungrounded template has no slot to fill"
        );
        let vocab = template(UNKNOWN_VOCABULARY).expect("unknown-vocabulary row");
        assert!(
            vocab.contains("{terms}"),
            "vocabulary-gap template carries the terms slot"
        );
        for id in [
            UNKNOWN_VOCABULARY_UNTERMED,
            SUGGEST_INTERPRETATION,
            ADMIT_LIMITATION,
        ] {
            let t = template(id).unwrap_or_else(|| panic!("{id} row"));
            assert!(!t.contains('{'), "{id} template has no slot to fill");
        }
    }

    /// The loader is first-match ([`template`]); a duplicated frame id would
    /// silently shadow its second row. Assert the committed table's ids are
    /// unique so a lookup is unambiguous — and (critique fix c) distinct from the
    /// sibling abstain / conditional frame ids, so no reader confuses the tables.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn frame_ids_are_unique() {
        let rows = build_table();
        assert!(!rows.is_empty(), "the committed table is non-empty");
        let mut ids: Vec<&str> = rows.iter().map(|(id, _)| id.as_str()).collect();
        ids.sort_unstable();
        let unique = ids.len();
        ids.dedup();
        assert_eq!(unique, ids.len(), "explain-frame ids must be unique");

        // Namespaced apart from the sibling realization tables — no id collides
        // with abstain-frames.tsv or conditional-frames.tsv.
        for foreign in [
            super::super::abstain_frames::UNPROVEN_RELATION,
            super::super::conditional_frames::RULE_AWAITING_FACT,
            super::super::conditional_frames::RULE_APPLIES,
            super::super::conditional_frames::RULE_DOES_NOT_APPLY,
        ] {
            assert!(
                template(foreign).is_none(),
                "explain table must not carry the sibling id {foreign}"
            );
        }
    }
}
