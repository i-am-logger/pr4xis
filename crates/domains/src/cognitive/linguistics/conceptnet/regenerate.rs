//! Offline regeneration of the committed, WordNet-lemma-crosswalk-filtered
//! ConceptNet assertion table.
//!
//! ## Why a filter step exists at all
//!
//! No official English-only or WordNet-linked subset of ConceptNet is
//! distributed (verified 2026-07-13 against
//! <https://github.com/commonsense/conceptnet5/wiki/Downloads> — only
//! Numberbatch, a *different* word-vector artifact, ships a smaller `mini`
//! release). The full assertions CSV (conceptnet-assertions-5.7.0.csv.gz,
//! <https://s3.amazonaws.com/conceptnet/downloads/2019/edges/conceptnet-assertions-5.7.0.csv.gz>,
//! confirmed via HTTP HEAD: 497,963,447 bytes) decompresses to 34,074,917
//! assertion rows (confirmed by full download + `zcat | wc -l`) — far too
//! large to commit or to load whole into a runtime store the way VerbNet's
//! 332 small class files are (unlike VerbNet, whose complete corpus IS small
//! enough to bundle in full).
//!
//! The filter this module implements is MECHANICAL, not a quality/relevance
//! judgment: keep an assertion iff both its start and end concepts are
//! `/c/en/…` (English) AND the concept token (the URI segment immediately
//! after `/c/en/`) matches, after normalization, a lemma the loaded WordNet
//! actually carries. This is a pure format-conversion + membership filter —
//! no reweighting, no relation-type curation, no dropping of low-weight rows
//! — consistent with the `AssociativeConceptTable` taxonomy leaf's license
//! note (ConceptNet data is CC BY-SA 4.0; a mechanical subset stays within
//! "the Licensed Material", not an "Adapted Material").
//!
//! Verified against a real download (2026-07-13): the filter keeps 932,948 of
//! 34,074,917 rows (~2.8%) — small enough to commit (~29 MB uncompressed
//! TSV, well under 10 MB once DEFLATE-compressed into the `.prx`, matching
//! `PayloadEncoding::Deflate` already selected for `ContentType::Plaintext`).
//!
//! ## Prerequisite (external, not run by this module)
//!
//! Unlike VerbNet's `git archive` prep step, fetching + decompressing
//! ConceptNet's raw data is a plain HTTP GET (no git involved), so it is run
//! directly, not via `pr4xis update` (which has no generic
//! "download-and-decompress-then-filter" primitive):
//!
//! ```text
//! mkdir -p crates/domains/data/conceptnet-download
//! curl -sS -o crates/domains/data/conceptnet-download/conceptnet-assertions-5.7.0.csv.gz \
//!   https://s3.amazonaws.com/conceptnet/downloads/2019/edges/conceptnet-assertions-5.7.0.csv.gz
//! gunzip -k crates/domains/data/conceptnet-download/conceptnet-assertions-5.7.0.csv.gz
//! ```
//!
//! `data/conceptnet-download/` is gitignored (transient staging, mirroring
//! `data/verbnet-checkout/`) — only this module's OUTPUT, the filtered
//! `.assoc` TSV, is committed.

use crate::cognitive::linguistics::conceptnet::store::normalize_lemma;

/// The set of every normalized WordNet lemma, loaded directly from the
/// registered raw XML (no need for a full `English::from_wordnet` build —
/// unlike VerbNet's sense-key crosswalk, this filter only needs lemma
/// STRINGS, not `ConceptId` resolution; see `store.rs`'s module doc for
/// why ConceptNet's own node granularity makes a `ConceptId`-precise
/// crosswalk meaningless here). Selected by kind, mirroring
/// `verbnet_class_collection::regenerate::build_wordnet_crosswalk_tsv`'s
/// own lookup — not a name literal, so a registry rename can't silently
/// desync this regen step.
fn wordnet_lemma_set() -> alloc::collections::BTreeSet<String> {
    use crate::applied::data_provisioning::registry::data_sources;
    use crate::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;

    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let entry = data_sources()
        .iter()
        .find(|e| e.kind == SourceTaxonomyConcept::Language)
        .expect("no Language-kind source registered");
    let xml_path = workspace_root.join(entry.local_path());
    let xml = std::fs::read_to_string(&xml_path).expect("read WordNet XML");
    let wn = read_wordnet(&xml).expect("parse WordNet XML");

    wn.entries
        .iter()
        .map(|e| normalize_lemma(&e.lemma.written_form))
        .collect()
}

/// Extract the concept token from a ConceptNet node URI (e.g.
/// `/c/en/well_being/n/wn/cognition` -> `Some("well_being")`,
/// `/c/de/hund` -> `None`). The token is everything between the `/c/en/`
/// prefix and the next `/` (or the end of the string) — already in
/// ConceptNet's own underscore-joined, lowercase convention, so no
/// further normalization beyond a defensive [`normalize_lemma`] pass is
/// needed (verified byte-exact against the real corpus: `well-being`
/// surfaces as `well_being`, never `well-being`).
fn en_concept_token(uri: &str) -> Option<&str> {
    let rest = uri.strip_prefix("/c/en/")?;
    Some(rest.split('/').next().unwrap_or(rest))
}

/// Extract the `"weight": <number>` field from a ConceptNet assertion's
/// JSON metadata tail. A minimal scan for this one known key, not a
/// general JSON parse (the metadata blob is 5+ keys of provenance data
/// this filter has no use for beyond weight) — mirrors the rest of this
/// codebase's hand-rolled, single-purpose readers over third-party data
/// formats (the XML tree walkers), never a general-purpose parser this
/// project doesn't otherwise need.
fn extract_weight(json_tail: &str) -> Option<f32> {
    let idx = json_tail.find("\"weight\"")?;
    let after_key = &json_tail[idx + "\"weight\"".len()..];
    let after_colon = after_key.trim_start().strip_prefix(':')?;
    let value_str: String = after_colon
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    value_str.parse::<f32>().ok()
}

/// Filter one already-decompressed ConceptNet assertions CSV (5
/// tab-separated columns: assertion URI, relation URI, start URI, end
/// URI, JSON metadata), given as a LINE ITERATOR rather than one big
/// string — the real file is ~10 GB decompressed, so the regen test
/// streams it line-by-line (raw `\n`-byte splitting + per-line UTF-8
/// decode, not `BufRead::lines()` — see the caller's own comment for why)
/// rather than reading it whole into memory; the unit tests below just
/// iterate a small literal's `.lines()`, the same interface. Keeps the
/// rows whose start AND end
/// both resolve to a loaded WordNet lemma, producing sorted
/// `relation\tstart_lemma\tend_lemma\tweight` output rows. Malformed rows
/// (wrong column count, unparseable weight) are skipped, fail-closed —
/// the same discipline every TSV reader in this codebase applies, never
/// a panic on real-world data.
fn filter_conceptnet_csv(
    lines: impl Iterator<Item = String>,
    wordnet_lemmas: &alloc::collections::BTreeSet<String>,
) -> Vec<String> {
    let mut rows = Vec::new();
    for line in lines {
        let mut fields = line.splitn(5, '\t');
        let (Some(_uri), Some(relation), Some(start), Some(end), Some(meta)) = (
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
        ) else {
            continue;
        };
        let Some(start_token) = en_concept_token(start) else {
            continue;
        };
        let Some(end_token) = en_concept_token(end) else {
            continue;
        };
        let start_lemma = normalize_lemma(start_token);
        let end_lemma = normalize_lemma(end_token);
        if !wordnet_lemmas.contains(&start_lemma) || !wordnet_lemmas.contains(&end_lemma) {
            continue;
        }
        let Some(weight) = extract_weight(meta) else {
            continue;
        };
        let Some(relation) = relation.strip_prefix("/r/") else {
            continue;
        };
        rows.push(alloc::format!(
            "{relation}\t{start_lemma}\t{end_lemma}\t{weight}"
        ));
    }
    rows.sort();
    rows
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
#[ignore]
fn regenerate_conceptnet_archive() {
    use std::io::BufRead;

    let csv_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/conceptnet-download/conceptnet-assertions-5.7.0.csv");
    let file = std::fs::File::open(&csv_path)
        .unwrap_or_else(|e| panic!("open {}: {e}", csv_path.display()));
    // Streamed, not `read_to_string`'d whole — the real file is ~10 GB
    // decompressed. Split on raw `\n` bytes rather than `BufRead::lines()`
    // (clippy::lines_filter_map_ok: `Lines` can loop forever re-reading a
    // PERSISTENT underlying I/O error) and decode each line's bytes as
    // UTF-8 ourselves, skipping only that one line on failure — a single
    // line with invalid UTF-8 must be skipped, not treated as
    // end-of-stream; this codebase never silently truncates a source over
    // one bad row.
    let lines = std::io::BufReader::new(file)
        .split(b'\n')
        .filter_map(Result::ok)
        .filter_map(|bytes| String::from_utf8(bytes).ok());

    let wordnet_lemmas = wordnet_lemma_set();
    eprintln!("loaded {} WordNet lemmas", wordnet_lemmas.len());

    let rows = filter_conceptnet_csv(lines, &wordnet_lemmas);
    eprintln!("filtered {} rows", rows.len());

    let out_text = rows.join("\n");
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/conceptnet/conceptnet-5.7.0.assoc");
    std::fs::create_dir_all(out.parent().expect("has parent")).expect("mkdir data/conceptnet");
    std::fs::write(&out, out_text.as_bytes())
        .unwrap_or_else(|e| panic!("write {}: {e}", out.display()));
    eprintln!(
        "wrote {} ({} bytes) address {}",
        out.display(),
        out_text.len(),
        pr4xis_runtime::address::ContentAddress::of(out_text.as_bytes()).to_hex()
    );
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn en_concept_token_extracts_the_lemma_segment() {
        assert_eq!(en_concept_token("/c/en/well_being"), Some("well_being"));
        assert_eq!(
            en_concept_token("/c/en/well_being/n/wn/cognition"),
            Some("well_being")
        );
        assert_eq!(en_concept_token("/c/de/hund"), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extract_weight_reads_the_json_tail() {
        let meta = r#"{"dataset": "/d/conceptnet/4/en", "weight": 2.0, "sources": []}"#;
        assert_eq!(extract_weight(meta), Some(2.0));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn extract_weight_returns_none_without_panicking_on_missing_field() {
        assert_eq!(extract_weight(r#"{"dataset": "/d/x"}"#), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn filter_keeps_only_rows_with_both_endpoints_in_wordnet() {
        let lemmas: alloc::collections::BTreeSet<String> =
            ["cut", "sever"].iter().map(|s| s.to_string()).collect();
        let csv = "\
/a/1\t/r/RelatedTo\t/c/en/cut\t/c/en/sever\t{\"weight\": 1.0}\n\
/a/2\t/r/IsA\t/c/en/cut\t/c/en/unknown_word\t{\"weight\": 1.0}\n\
/a/3\t/r/RelatedTo\t/c/de/schneiden\t/c/en/sever\t{\"weight\": 1.0}\n";
        let rows = filter_conceptnet_csv(csv.lines().map(str::to_string), &lemmas);
        assert_eq!(rows, alloc::vec!["RelatedTo\tcut\tsever\t1"]);
    }
}
