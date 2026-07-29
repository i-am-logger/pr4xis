//! Rule-awaiting-fact realization frames, carried as LOADED data — the
//! response surface for `ResponseFrame::RuleAwaitingFact`, done the praxis
//! way (frame-as-data), never an inline `format!` literal.
//!
//! Mirrors `abstain_frames.rs` exactly: committed content-addressed `.prx`
//! → `raw_source_text_embedded` → `(frame_id, template)` rows → `OnceLock`
//! cache (`std`) / rebuild-by-value (`no_std`).
//!
//! Cite: Reiter & Dale (2000) *Building Natural Language Generation
//! Systems* (CUP), realization stage; Bobrow, Kaplan, Norman, Thompson &
//! Winograd (1977) "GUS, A Frame-Driven Dialog System", *Artificial
//! Intelligence* 8(2):155-173.

#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// The committed rule-awaiting-fact frames projection `.prx` — the
/// content-addressed envelope carrying the `frame_id<TAB>template` table
/// bytes. The raw `.tsv` is authored source-of-truth (git-tracked, EXCLUDED
/// from the published crate); only this `.prx` is committed + embedded and
/// ships. Loaded through the generalized raw-source gate (content-address
/// hash + `raw_hash::verify`), feature-independent so the table builds on
/// default, `no_std` and wasm.
const CONDITIONAL_FRAMES_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/grammar/conditional-frames.prx"
));

/// The registry `name@version` key the committed `.prx` is pinned under (see
/// `[sources.conditional_frames]` in `praxis.toml`).
const NAME: &str = "conditional_frames";
const VERSION: &str = "2026";

/// The `rule_awaiting_fact` frame id — the realization surface for
/// [`ResponseFrame::RuleAwaitingFact`](super::response::ResponseFrame::RuleAwaitingFact).
pub const RULE_AWAITING_FACT: &str = "rule_awaiting_fact";
/// The `rule_applies` frame id — the realization surface for
/// [`ResponseFrame::RuleApplies`](super::response::ResponseFrame::RuleApplies).
pub const RULE_APPLIES: &str = "rule_applies";
/// The `rule_does_not_apply` frame id — the realization surface for
/// [`ResponseFrame::RuleDoesNotApply`](super::response::ResponseFrame::RuleDoesNotApply).
pub const RULE_DOES_NOT_APPLY: &str = "rule_does_not_apply";

/// Parse the loaded TSV into `(frame_id, template)` rows — the generic
/// interpreter (never a per-frame arm). Comment (`#`) and blank lines and the
/// header row (`frame_id`) are skipped.
fn build_table() -> Vec<(String, String)> {
    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    let tsv = raw_source_text_embedded(NAME, VERSION, CONDITIONAL_FRAMES_PRX);
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

/// The loaded conditional-frame table, cached for the process (`std`).
#[cfg(feature = "std")]
fn conditional_table() -> &'static Vec<(String, String)> {
    use std::sync::OnceLock;
    static TABLE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    TABLE.get_or_init(build_table)
}

/// The raw template string for a frame id from the loaded table, or `None` if the
/// table carries no such row.
pub fn template(frame_id: &str) -> Option<String> {
    #[cfg(feature = "std")]
    let rows = conditional_table().clone();
    #[cfg(not(feature = "std"))]
    let rows = build_table();
    rows.into_iter()
        .find(|(id, _)| id == frame_id)
        .map(|(_, t)| t)
}

/// The `rule_awaiting_fact` realization template — the surface for
/// [`ResponseFrame::RuleAwaitingFact`](super::response::ResponseFrame::RuleAwaitingFact).
/// Fail-closed: the committed table always carries this row (a build invariant),
/// so `None` is a genuine defect, surfaced by the caller's realization fallback.
pub fn rule_awaiting_fact_template() -> Option<String> {
    template(RULE_AWAITING_FACT)
}

/// The `rule_applies` realization template — the surface for
/// [`ResponseFrame::RuleApplies`](super::response::ResponseFrame::RuleApplies).
/// Fail-closed: the committed table always carries this row.
pub fn rule_applies_template() -> Option<String> {
    template(RULE_APPLIES)
}

/// The `rule_does_not_apply` realization template — the surface for
/// [`ResponseFrame::RuleDoesNotApply`](super::response::ResponseFrame::RuleDoesNotApply).
/// Fail-closed: the committed table always carries this row.
pub fn rule_does_not_apply_template() -> Option<String> {
    template(RULE_DOES_NOT_APPLY)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn the_table_loads_the_rule_awaiting_fact_frame() {
        let t =
            rule_awaiting_fact_template().expect("the committed table carries rule_awaiting_fact");
        // The typed slots the realizer fills are present; the discourse frame is
        // loaded data, not composed here.
        assert!(
            t.contains("{rule_name}"),
            "template carries the rule-name slot"
        );
        assert!(
            t.contains("{citation}"),
            "template carries the citation slot"
        );
        assert!(
            t.contains("{rule_text}"),
            "template carries the rule-text slot"
        );
        assert!(
            t.contains("{missing_facts}"),
            "template carries the missing-facts slot"
        );
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn the_table_loads_the_rule_verdict_frames() {
        for (template_fn, name) in [
            (
                rule_applies_template as fn() -> Option<String>,
                "rule_applies",
            ),
            (
                rule_does_not_apply_template as fn() -> Option<String>,
                "rule_does_not_apply",
            ),
        ] {
            let t = template_fn().unwrap_or_else(|| panic!("the committed table carries {name}"));
            assert!(
                t.contains("{rule_name}"),
                "{name} carries the rule-name slot"
            );
            assert!(t.contains("{citation}"), "{name} carries the citation slot");
            assert!(
                t.contains("{rule_text}"),
                "{name} carries the rule-text slot"
            );
        }
    }
}
