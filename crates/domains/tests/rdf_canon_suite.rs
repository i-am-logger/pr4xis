//! W3C RDF Dataset Canonicalization 1.0 (RDFC-1.0) **conformance gate**.
//!
//! Runs the entire vendored official W3C `rdf-canon` test suite
//! (`tests/data/rdf_canon/`, from <https://github.com/w3c/rdf-canon> at the
//! commit recorded in `COMMIT.txt`) against the in-house implementation in
//! `pr4xis_domains::social::software::markup::xml::rdf::canon`.
//!
//! Three test kinds, driven by the suite's `manifest.csv`:
//!
//! - **RDFC10EvalTest** — parse `*-in.nq`, canonicalize, assert the
//!   serialized canonical form equals `*-rdfc10.nq` byte for byte.
//! - **RDFC10MapTest** — additionally assert the issued-identifiers map
//!   equals `*-rdfc10map.json`.
//! - **RDFC10NegativeEvalTest** — a *poison* graph (the `test074` clique);
//!   assert canonicalization **errors** with the complexity cap, rather
//!   than hanging or returning a (wrong) answer.
//!
//! `test075` uses SHA-384 (`hashAlgorithm = SHA384`); every other test
//! uses the SHA-256 default. `test044`/`045`/`046` are deliberately
//! expensive "computable given defined limits" poison graphs that MUST
//! still produce the correct canonical form under the default cap.
//!
//! The gate is exhaustive and hard: any fixture that fails to match (or a
//! negative test that does *not* error) fails the build. There is no
//! allow-list.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use pr4xis_domains::social::software::markup::xml::rdf::canon::{
    self, CanonError, CanonLimits, HashAlgorithm,
};

/// One row of the suite manifest, joining the `*c` (eval/negative) and
/// `*m` (map) entries that share an input.
#[derive(Debug, Clone)]
struct ManifestRow {
    id: String,
    /// `true` when the manifest's `rdfc10` column is the negative-eval
    /// marker `RDFC10NegativeEvalTest` (vs `TRUE` for a normal eval test).
    negative: bool,
    /// SHA-384 selected by the `hashAlgorithm` column (else SHA-256).
    sha384: bool,
    /// `rdfc10` column is truthy → there is a `*-rdfc10.nq` to compare.
    has_eval: bool,
    /// `rdfc10map` column is `TRUE` → there is a `*-rdfc10map.json`.
    has_map: bool,
}

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/rdf_canon")
}

/// Parse the suite `manifest.csv`. Columns:
/// `test,name,comment,complexity,approval,hashAlgorithm,rdfc10,rdfc10map`.
fn load_manifest() -> Vec<ManifestRow> {
    let csv = fs::read_to_string(data_dir().join("manifest.csv")).expect("read manifest.csv");
    let mut rows = Vec::new();
    for (i, line) in csv.lines().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue; // header
        }
        // The `name`/`comment` fields can be quoted and contain commas, but
        // the columns we need (test id, hashAlgorithm, rdfc10, rdfc10map)
        // are the first and last three and never contain commas. Parse from
        // both ends to skip the messy middle.
        let first_comma = line.find(',').expect("csv id column");
        let id = line[..first_comma].to_string();
        // Last three comma-separated fields.
        let tail: Vec<&str> = line.rsplitn(4, ',').collect(); // [rdfc10map, rdfc10, hashAlgorithm, rest]
        let rdfc10map = tail[0].trim();
        let rdfc10 = tail[1].trim();
        let hash_algo = tail[2].trim();
        rows.push(ManifestRow {
            id,
            negative: rdfc10.eq_ignore_ascii_case("RDFC10NegativeEvalTest"),
            sha384: hash_algo.eq_ignore_ascii_case("SHA384"),
            has_eval: rdfc10.eq_ignore_ascii_case("TRUE")
                || rdfc10.eq_ignore_ascii_case("RDFC10NegativeEvalTest"),
            has_map: rdfc10map.eq_ignore_ascii_case("TRUE"),
        });
    }
    rows
}

fn read_fixture(name: &str) -> String {
    fs::read_to_string(data_dir().join("rdfc10").join(name))
        .unwrap_or_else(|e| panic!("read fixture {name}: {e}"))
}

fn algo_for(row: &ManifestRow) -> HashAlgorithm {
    if row.sha384 {
        HashAlgorithm::Sha384
    } else {
        HashAlgorithm::Sha256
    }
}

/// The full conformance gate.
#[test]
fn w3c_rdfc10_conformance_suite() {
    let rows = load_manifest();
    assert!(
        rows.len() >= 60,
        "manifest looks truncated: {} rows",
        rows.len()
    );

    let mut eval_pass = 0usize;
    let mut eval_total = 0usize;
    let mut map_pass = 0usize;
    let mut map_total = 0usize;
    let mut negative_pass = 0usize;
    let mut negative_total = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for row in &rows {
        let input = read_fixture(&format!("{}-in.nq", row.id));
        let quads = match canon::parse_nquads(&input) {
            Ok(q) => q,
            Err(e) => {
                failures.push(format!("{}: parse failed: {e}", row.id));
                continue;
            }
        };

        if row.negative {
            // RDFC10NegativeEvalTest: MUST error (the DoS cap), not hang,
            // not return a wrong answer.
            negative_total += 1;
            match canon::canonicalize_with(&quads, CanonLimits::default(), algo_for(row)) {
                Err(CanonError::ComplexityCapExceeded { .. }) => negative_pass += 1,
                Err(other) => failures.push(format!(
                    "{}: negative test errored, but not via the complexity cap: {other}",
                    row.id
                )),
                Ok(_) => failures.push(format!(
                    "{}: negative (poison) test produced a result instead of erroring",
                    row.id
                )),
            }
            continue;
        }

        // Positive eval / map test.
        let (output, issued_map) =
            match canon::canonicalize_with(&quads, CanonLimits::default(), algo_for(row)) {
                Ok(r) => r,
                Err(e) => {
                    failures.push(format!("{}: canonicalization failed: {e}", row.id));
                    continue;
                }
            };

        if row.has_eval {
            eval_total += 1;
            let expected = read_fixture(&format!("{}-rdfc10.nq", row.id));
            if output == expected {
                eval_pass += 1;
            } else {
                failures.push(format!(
                    "{}: canonical N-Quads mismatch\n--- expected ---\n{}\n--- got ---\n{}",
                    row.id,
                    show(&expected),
                    show(&output),
                ));
            }
        }

        if row.has_map {
            map_total += 1;
            let expected_json = read_fixture(&format!("{}-rdfc10map.json", row.id));
            let expected_map = parse_map_json(&expected_json);
            if issued_map == expected_map {
                map_pass += 1;
            } else {
                failures.push(format!(
                    "{}: issued-identifiers map mismatch\n  expected {:?}\n  got      {:?}",
                    row.id, expected_map, issued_map
                ));
            }
        }
    }

    eprintln!(
        "RDFC-1.0 W3C suite: eval {eval_pass}/{eval_total}, map {map_pass}/{map_total}, \
         negative {negative_pass}/{negative_total}"
    );

    assert!(
        failures.is_empty(),
        "RDFC-1.0 conformance failures ({}):\n{}",
        failures.len(),
        failures.join("\n\n")
    );
    assert_eq!(eval_pass, eval_total, "not all eval tests passed");
    assert_eq!(map_pass, map_total, "not all map tests passed");
    assert_eq!(
        negative_pass, negative_total,
        "not all negative tests errored"
    );
    // Sanity: the suite must actually have exercised every category.
    assert!(eval_total > 50, "too few eval tests ran: {eval_total}");
    assert!(map_total > 10, "too few map tests ran: {map_total}");
    assert!(
        negative_total >= 1,
        "the poison/negative test did not run: {negative_total}"
    );
}

/// Pin the poison-graph boundary by name so a regression is unambiguous:
/// the three "computable given defined limits" graphs (`test044`/`045`/
/// `046`, manifest complexity 39) MUST canonicalize to their expected
/// output under the default cap, while the `test074` clique (complexity 40,
/// the suite's only `RDFC10NegativeEvalTest`) MUST hit the cap and error.
#[test]
fn poison_boundary_by_name() {
    // Computable poison graphs: correct answer, no error.
    for id in ["test044", "test045", "test046"] {
        let input = read_fixture(&format!("{id}-in.nq"));
        let quads = canon::parse_nquads(&input).unwrap_or_else(|e| panic!("{id} parse: {e}"));
        let (output, _map) =
            canon::canonicalize_with(&quads, CanonLimits::default(), HashAlgorithm::Sha256)
                .unwrap_or_else(|e| panic!("{id} must be computable under the cap, got: {e}"));
        let expected = read_fixture(&format!("{id}-rdfc10.nq"));
        assert_eq!(output, expected, "{id} canonical form mismatch");
    }

    // The clique poison graph: must error via the complexity cap.
    let input = read_fixture("test074-in.nq");
    let quads = canon::parse_nquads(&input).expect("test074 parse");
    match canon::canonicalize_with(&quads, CanonLimits::default(), HashAlgorithm::Sha256) {
        Err(CanonError::ComplexityCapExceeded { .. }) => {}
        other => panic!("test074 clique must hit the DoS cap, got: {other:?}"),
    }
}

/// Minimal `{ "label": "c14nN", ... }` JSON parser for the
/// `*-rdfc10map.json` fixtures (flat string→string object). Avoids adding a
/// serde dependency to this test for a trivially regular shape.
fn parse_map_json(json: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let body = json.trim().trim_start_matches('{').trim_end_matches('}');
    for pair in body.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let colon = pair.find(':').expect("json pair colon");
        let key = unquote(pair[..colon].trim());
        let val = unquote(pair[colon + 1..].trim());
        map.insert(key, val);
    }
    map
}

fn unquote(s: &str) -> String {
    s.trim().trim_matches('"').to_string()
}

/// Bound a fixture body for an error message so a giant mismatch does not
/// flood the test log.
fn show(s: &str) -> String {
    if s.len() > 2000 {
        format!("{}…(truncated, {} bytes)", &s[..2000], s.len())
    } else {
        s.to_string()
    }
}
