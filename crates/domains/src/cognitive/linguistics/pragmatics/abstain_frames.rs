//! Honest-abstention realization frames, carried as LOADED data and interpreted
//! — the response surface for the fifth epistemic cell (*vocabulary known,
//! proposition open*) done the praxis way (frame-as-data), NOT an inline
//! `format!` literal in the realizer.
//!
//! This mirrors the OLiA→CCG projection loader
//! ([`category_projection`](crate::cognitive::linguistics::lambek::category_projection)):
//! the committed content-addressed `.prx` is `include_bytes!`-embedded, decoded
//! through the generalized raw-source gate
//! ([`raw_source_text_embedded`](crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded)),
//! parsed into `(frame_id, template)` rows, and cached in a process `OnceLock`
//! (`std`) or rebuilt by value (`no_std`). The realizer
//! ([`realize::realize_abstain`](super::realize)) reads the template and fills its
//! typed slots — it never authors the discourse frame in code.
//!
//! Cite: Reiter & Dale (2000) *Building Natural Language Generation Systems*
//! (CUP), realization stage; Reiter (1978) closed-world assumption (the honest
//! boundary of the system's own knowledge).

#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// The committed abstain-frames projection `.prx` — the content-addressed
/// envelope carrying the `frame_id<TAB>template` table bytes. The raw `.tsv` is
/// authored source-of-truth (git-tracked, EXCLUDED from the published crate);
/// only this `.prx` is committed + embedded and ships. Loaded through the
/// generalized raw-source gate (content-address hash + `raw_hash::verify`),
/// feature-independent so the table builds on default, `no_std` and wasm.
const ABSTAIN_FRAMES_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/grammar/abstain-frames.prx"
));

/// The registry `name@version` key the committed `.prx` is pinned under (see
/// `[sources.abstain_frames]` in `praxis.toml`).
const NAME: &str = "abstain_frames";
const VERSION: &str = "2026";

/// The `unproven_relation` frame id — the abstain surface for
/// [`ResponseFrame::UnprovenRelation`](super::response::ResponseFrame::UnprovenRelation).
pub const UNPROVEN_RELATION: &str = "unproven_relation";

/// Parse the loaded TSV into `(frame_id, template)` rows — the generic
/// interpreter (never a per-frame arm). Comment (`#`) and blank lines and the
/// header row (`frame_id`) are skipped.
fn build_table() -> Vec<(String, String)> {
    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    let tsv = raw_source_text_embedded(NAME, VERSION, ABSTAIN_FRAMES_PRX);
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

/// The loaded abstain-frame table, cached for the process (`std`).
#[cfg(feature = "std")]
fn abstain_table() -> &'static Vec<(String, String)> {
    use std::sync::OnceLock;
    static TABLE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    TABLE.get_or_init(build_table)
}

/// The raw template string for a frame id from the loaded table, or `None` if the
/// table carries no such row.
pub fn template(frame_id: &str) -> Option<String> {
    #[cfg(feature = "std")]
    let rows = abstain_table().clone();
    #[cfg(not(feature = "std"))]
    let rows = build_table();
    rows.into_iter()
        .find(|(id, _)| id == frame_id)
        .map(|(_, t)| t)
}

/// The `unproven_relation` abstain template — the surface for
/// [`ResponseFrame::UnprovenRelation`](super::response::ResponseFrame::UnprovenRelation).
/// Fail-closed: the committed table always carries this row (a build invariant),
/// so `None` is a genuine defect, surfaced by the caller's realization test.
pub fn unproven_relation_template() -> Option<String> {
    template(UNPROVEN_RELATION)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn the_table_loads_the_unproven_relation_frame() {
        let t =
            unproven_relation_template().expect("the committed table carries unproven_relation");
        // The typed slots the realizer fills are present; the discourse frame is
        // loaded data, not composed here.
        assert!(
            t.contains("{known}"),
            "template carries the known-pair slot"
        );
        assert!(
            t.contains("{claim}"),
            "template carries the relation-claim slot"
        );
        assert!(
            !t.contains("I do not know the word"),
            "the abstain frame must NOT reuse the vocabulary-gap surface"
        );
    }
}
