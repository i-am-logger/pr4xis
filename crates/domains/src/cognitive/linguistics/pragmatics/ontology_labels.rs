//! Ontology-name → plain-English label projection, carried as LOADED data — the
//! LEXICALIZATION leg of the NLG realization stage (Reiter & Dale 2000,
//! *Building Natural Language Generation Systems*, CUP: lexicalization is
//! choosing the surface words for a domain entity).
//!
//! The "Why?" layer ([`realize::realize_why`](super::realize::realize_why))
//! fills its `{sources}` slot from the loaded ontologies a turn reasoned over,
//! each carried as a raw internal [`OntologyName`]
//! (`caregiving_lexicon`, `usc_title_42`, …). This module projects each such name
//! to the human-facing surface a reader can understand, so a snake_case internal
//! identifier NEVER leaks into a user-facing sentence (the jargon-leak the
//! why-layer critique forbids).
//!
//! A name absent from the loaded table is projected to the generic fallback
//! [`GENERIC_FALLBACK`] — the honest default for any source the engine can load
//! at runtime but the table does not name individually — so [`plain_label`] is
//! TOTAL over every possible [`OntologyName`] and never emits a raw identifier.
//!
//! Mirrors [`abstain_frames`](super::abstain_frames) exactly: committed
//! content-addressed `.prx` → `raw_source_text_embedded` → `(name, label)` rows →
//! `OnceLock` cache (`std`) / rebuild-by-value (`no_std`).
//!
//! Cite: Reiter & Dale (2000), the realization/lexicalization stage.
//!
//! [`OntologyName`]: pr4xis::ontology::meta::OntologyName

#[allow(unused_imports)]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use pr4xis::ontology::meta::OntologyName;

/// The committed ontology-labels projection `.prx` — the content-addressed
/// envelope carrying the `ontology_name<TAB>label` table bytes. The raw `.tsv` is
/// authored source-of-truth (git-tracked, EXCLUDED from the published crate);
/// only this `.prx` is committed + embedded and ships.
const ONTOLOGY_LABELS_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/grammar/ontology-labels.prx"
));

/// The registry `name@version` key the committed `.prx` is pinned under (see
/// `[sources.ontology_labels]` in `praxis.toml`).
const NAME: &str = "ontology_labels";
const VERSION: &str = "2026";

/// The plain-language label for an [`OntologyName`] the projection table does
/// NOT name individually — the honest generic default for any runtime-loadable
/// source, so no raw snake_case identifier is ever surfaced.
pub const GENERIC_FALLBACK: &str = "your loaded documents";

/// Parse the loaded TSV into `(ontology_name, label)` rows — the generic
/// interpreter (never a per-name arm). Comment (`#`) and blank lines and the
/// header row (`ontology_name`) are skipped.
fn build_table() -> Vec<(String, String)> {
    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    let tsv = raw_source_text_embedded(NAME, VERSION, ONTOLOGY_LABELS_PRX);
    let mut rows: Vec<(String, String)> = Vec::new();
    for line in tsv.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.split('\t');
        let Some(name) = cols.next().map(str::trim) else {
            continue;
        };
        let Some(label) = cols.next() else {
            continue; // header row (`ontology_name` with no label column)
        };
        if name.is_empty() || name == "ontology_name" {
            continue;
        }
        rows.push((name.to_string(), label.to_string()));
    }
    rows
}

/// The loaded ontology-labels table, cached for the process (`std`).
#[cfg(feature = "std")]
fn labels_table() -> &'static Vec<(String, String)> {
    use std::sync::OnceLock;
    static TABLE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    TABLE.get_or_init(build_table)
}

/// The plain-language label for a loaded [`OntologyName`] — the row's label, or
/// [`GENERIC_FALLBACK`] when the table does not name this source individually.
/// TOTAL: never `None`, never a raw snake_case identifier.
pub fn plain_label(name: &OntologyName) -> String {
    #[cfg(feature = "std")]
    let rows = labels_table().clone();
    #[cfg(not(feature = "std"))]
    let rows = build_table();
    rows.into_iter()
        .find(|(n, _)| n == name.as_str())
        .map(|(_, label)| label)
        .unwrap_or_else(|| GENERIC_FALLBACK.to_string())
}

/// Project a list of loaded [`OntologyName`]s to a single plain-language noun
/// phrase for the `{sources}` slot — each name projected via [`plain_label`],
/// de-duplicated (the fallback collapses several unnamed sources to ONE "your
/// loaded documents"), and joined with English list grammar ("A", "A and B",
/// "A, B, and C"). Empty input yields the fallback alone — the caller
/// ([`realize::realize_why`](super::realize)) only reaches the grounded frame
/// with a non-empty list, so this is a defensive floor.
pub fn project_join(names: &[OntologyName]) -> String {
    let mut labels: Vec<String> = Vec::new();
    for n in names {
        let label = plain_label(n);
        if !labels.contains(&label) {
            labels.push(label);
        }
    }
    match labels.len() {
        0 => GENERIC_FALLBACK.to_string(),
        1 => labels.remove(0),
        2 => format!("{} and {}", labels[0], labels[1]),
        _ => {
            let last = labels.pop().expect("len >= 3");
            format!("{}, and {}", labels.join(", "), last)
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn known_names_project_to_plain_labels() {
        // A registered built-in source resolves to its human-facing label — never
        // its raw snake_case OntologyName.
        let care = plain_label(&OntologyName::new("caregiving_lexicon"));
        assert_eq!(care, "the Family Caregiving lexicon");
        assert!(!care.contains('_'), "no snake_case leaks: {care}");

        let title42 = plain_label(&OntologyName::new("usc_title_42"));
        assert!(title42.contains("Title 42"), "got {title42}");
        assert!(!title42.contains('_'), "no snake_case leaks: {title42}");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn unknown_name_projects_to_the_generic_fallback() {
        // A source the table does not name individually never leaks its
        // identifier — it projects to the honest generic default.
        let label = plain_label(&OntologyName::new("some_unregistered_corpus_2027"));
        assert_eq!(label, GENERIC_FALLBACK);
        assert!(!label.contains('_'), "no snake_case leaks: {label}");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn project_join_uses_english_list_grammar_and_dedups() {
        let one = project_join(&[OntologyName::new("caregiving_lexicon")]);
        assert_eq!(one, "the Family Caregiving lexicon");

        let two = project_join(&[
            OntologyName::new("caregiving_lexicon"),
            OntologyName::new("usc_title_42"),
        ]);
        assert_eq!(
            two,
            "the Family Caregiving lexicon and U.S. Code Title 42 (The Public Health and Welfare)"
        );

        // Two unnamed sources collapse to ONE "your loaded documents" — never a
        // repeated fallback, never a raw identifier.
        let deduped = project_join(&[
            OntologyName::new("mystery_a_2027"),
            OntologyName::new("mystery_b_2027"),
        ]);
        assert_eq!(deduped, GENERIC_FALLBACK);
    }
}
