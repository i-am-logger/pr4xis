//! Citation audit — integration test that walks the workspace-root
//! `citations.toml` registry and the source tree, asserting that:
//!
//!   1. Every registry entry has a non-empty `verified_by` field.
//!   2. Every code reference to a citation slug resolves to a
//!      registry entry.
//!   3. Free-form citations in code (Phase A) are surfaced as
//!      warnings with file:line so they can be migrated to slug
//!      references in Phase B.
//!
//! Per the user directive: unverified citations FAIL CI. No allow-
//! list, no warn-only mode. The `Spivak (2014) §3.1 — typed-handle
//! patterns and functorial data migration` citation was confirmed
//! wrong on audit; this gate prevents that failure mode from
//! recurring.
//!
//! The full registry schema, slug-naming convention, verification
//! workflow, and downgrade path are documented in the header of
//! `citations.toml` at the workspace root.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use pr4xis_domains::formal::meta::citation_quality::assessment::{
    CitationVerdict, DimensionStatus,
};
use pr4xis_domains::formal::meta::citation_quality::ontology::CitationQualityConcept;
use pr4xis_domains::formal::meta::citation_quality::record_lens::CitationAssessment;
use pr4xis_domains::formal::meta::citation_quality::registry_projection::{
    EntryFields, VersionFiber, project_entry,
};

// ---------------------------------------------------------------------
// Registry schema (deserialised from citations.toml)
// ---------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Registry {
    #[serde(default)]
    citations: BTreeMap<String, Citation>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // every field is part of the schema even if not all are
// load-bearing for the audit test (e.g. doi, isbn used for
// human verification of the entry).
struct Citation {
    #[serde(default)]
    authors: String,
    #[serde(default)]
    year: Option<toml::Value>,
    #[serde(default)]
    title: String,
    #[serde(default)]
    publisher: String,
    #[serde(default)]
    edition: String,
    #[serde(default)]
    section_or_page: String,
    #[serde(default)]
    isbn: String,
    #[serde(default)]
    doi: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    content_summary: String,
    #[serde(default)]
    verified_by: String,
    #[serde(default)]
    verified_on: String,
    #[serde(default)]
    verification_method: String,
    #[serde(default)]
    verification_notes: String,

    // Multi-version support (the version-polymorphic citation
    // adjunction). When a cited work has several published versions
    // whose section numbering differs (e.g. the LRC USLM User Guide),
    // the entry anchors a version-independent `claim` (the constant
    // complement, Bancilhon & Spyratos 1981) and carries one
    // `[[citations.<slug>.versions]]` fiber per version (Grothendieck
    // fibration). The flat `section_or_page` / `verified_*` fields stay
    // empty in that case; verification is per fiber.
    #[serde(default)]
    claim: String,
    #[serde(default)]
    versions: Vec<CitationVersion>,
}

/// One version-fiber of a multi-version citation: where the entry's
/// `claim` is located, and the verification, *in a single published
/// version* of the cited work. The cross-version mapping of these
/// fibers is the citation's version adjunction (forget-version ⊣
/// locate-in-version).
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // mirrors the per-version registry schema
struct CitationVersion {
    #[serde(default)]
    version: String,
    #[serde(default)]
    edition: String,
    #[serde(default)]
    year: Option<toml::Value>,
    #[serde(default)]
    section_or_page: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    content_summary: String,
    #[serde(default)]
    verified_by: String,
    #[serde(default)]
    verified_on: String,
    #[serde(default)]
    verification_method: String,
    #[serde(default)]
    verification_notes: String,
}

// ---------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------

fn workspace_root() -> PathBuf {
    // crates/domains -> ../../
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR has parent")
        .parent()
        .expect("crates/ has parent")
        .to_path_buf()
}

fn citations_toml_path() -> PathBuf {
    workspace_root().join("citations.toml")
}

fn load_registry() -> Registry {
    let text =
        fs::read_to_string(citations_toml_path()).expect("citations.toml exists at workspace root");
    toml::from_str(&text).expect("citations.toml parses as TOML with the registry schema")
}

// ---------------------------------------------------------------------
// Projection: a deserialised registry `Citation` → the citation_quality
// module's typed `EntryFields` view. Computes field-presence here so the
// projection logic in the module stays independent of this test's
// deserialisation schema. Each presence rule mirrors the dimension
// mapping documented in `registry_projection`.
// ---------------------------------------------------------------------

fn nonempty(s: &str) -> bool {
    !s.trim().is_empty()
}

fn entry_fields(entry: &Citation) -> EntryFields {
    let has_year = entry.year.is_some();
    EntryFields {
        has_authors: nonempty(&entry.authors),
        has_title: nonempty(&entry.title),
        has_publisher: nonempty(&entry.publisher),
        has_year,
        // Existence identifier: a year, DOI, URL, or ISBN locates the work.
        has_identifier: has_year
            || nonempty(&entry.doi)
            || nonempty(&entry.url)
            || nonempty(&entry.isbn),
        has_section_or_page: nonempty(&entry.section_or_page),
        has_content_summary: nonempty(&entry.content_summary),
        has_verified_by: nonempty(&entry.verified_by),
        has_claim: nonempty(&entry.claim),
        verification_method: entry.verification_method.clone(),
        versions: entry
            .versions
            .iter()
            .map(|v| VersionFiber {
                has_locator: nonempty(&v.section_or_page),
                is_verified: nonempty(&v.verified_by),
                // A fiber's identifier is its own year or url (the work's
                // version-located identity).
                has_identifier: v.year.is_some() || nonempty(&v.url),
            })
            .collect(),
    }
}

fn project(entry: &Citation) -> CitationAssessment {
    project_entry(&entry_fields(entry))
}

/// The names of the dimensions a projected assessment leaves
/// `Unverified`, paired with whether each is a sound-gate (blocking)
/// dimension — for the report. Ordered by severity (sound-gate first)
/// so the most important gaps print first. The concept column ties each
/// name to its `CitationQualityConcept` grounding.
fn unverified_dimensions(a: &CitationAssessment) -> Vec<(&'static str, bool)> {
    use CitationQualityConcept as C;
    use pr4xis_domains::formal::meta::citation_quality::ontology::is_sound_gate;
    let pairs = [
        (C::Existence, a.existence, "Existence"),
        (C::ClaimSupport, a.claim_support, "ClaimSupport"),
        (C::LocatorAccuracy, a.locator_accuracy, "LocatorAccuracy"),
        (
            C::BibliographicAccuracy,
            a.bibliographic_accuracy,
            "BibliographicAccuracy",
        ),
        (
            C::FormatConformance,
            a.format_conformance,
            "FormatConformance",
        ),
    ];
    pairs
        .iter()
        .filter(|(_, status, _)| *status == DimensionStatus::Unverified)
        .map(|(concept, _, name)| (*name, is_sound_gate(*concept)))
        .collect()
}

// ---------------------------------------------------------------------
// Slug references in code — Phase A discovery heuristic
// ---------------------------------------------------------------------
//
// Phase A: citation references in code stay free-form. The walker
// recognises a slug reference of the form `cite![<slug>]` or
// `<!-- cite:<slug> -->` (the Phase B forms) so that as code gets
// migrated incrementally, the audit can validate the slugs already
// in place. Phase A code-references discovered are reported by
// the test for visibility; missing-slug code references fail CI.

fn extract_slug_references(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    // `cite![<slug>]` — Rust macro-style reference (Phase B form 1).
    let needle_macro = "cite![";
    let mut i = 0;
    while let Some(off) = text[i..].find(needle_macro) {
        let start = i + off + needle_macro.len();
        if let Some(end_off) = text[start..].find(']') {
            let slug = text[start..start + end_off].trim();
            if is_slug(slug) {
                out.insert(slug.to_string());
            }
            i = start + end_off + 1;
        } else {
            break;
        }
    }
    // `<!-- cite:<slug> -->` — markdown / doc-comment reference (Phase B form 2).
    let needle_comment = "<!-- cite:";
    let mut i = 0;
    while let Some(off) = text[i..].find(needle_comment) {
        let start = i + off + needle_comment.len();
        if let Some(end_off) = text[start..].find("-->") {
            let slug = text[start..start + end_off].trim();
            if is_slug(slug) {
                out.insert(slug.to_string());
            }
            i = start + end_off + 3;
        } else {
            break;
        }
    }
    out
}

fn is_slug(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

// ---------------------------------------------------------------------
// Free-form citation discovery — Phase A warnings
// ---------------------------------------------------------------------
//
// Heuristic patterns. Listed for documentation; counted at audit
// time but currently NOT a failure mode (Phase A). The migration to
// slug references is queued as a separate dispatch.

const FREEFORM_PATTERNS: &[&str] = &[
    "W3C XSD 1.1",
    "W3C XML 1.0",
    "W3C XML Schema",
    "W3C XHTML 1.0",
    "WHATWG HTML LS",
    "WHATWG HTML Living Standard",
    "Mac Lane",
    "Spivak (",
    "Spivak 2014",
    "Awodey",
    "Fellbaum",
    "Bauer 1983",
    "Pemberton",
    "Raggett",
    "Gao",
    "Sperberg-McQueen",
    "Peterson",
    "Bray",
    "Cowan",
    "RFC 5322",
    "RFC 5646",
    "RFC 8259",
    "ISO 15836",
    "ISO 32000",
    "ISO 8601",
    "Bluebook",
    "Kephart",
    "Smith et al. (2005)",
    "Smith 2005",
    "Dublin Core",
    "DCMI",
    "Adobe Tech Note",
    "GPO Style Manual",
];

// Directories scoped to M4.ε / M4.η work (per the dispatch). These
// are the trees where the audit's bulk-extracted citations come
// from; expanding the walker's scope is a follow-up commit.
const SCOPED_DIRS: &[&str] = &[
    "crates/domains/src/formal/meta/xsd",
    "crates/domains/src/social/software/markup/xml",
    "crates/domains/src/social/software/markup/html",
    "crates/domains/src/social/judicial/statute_structure",
];

// ---------------------------------------------------------------------
// Walking the source tree
// ---------------------------------------------------------------------

fn walk_rust_files(root: &Path, out: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    for entry in fs::read_dir(root).expect("read scoped dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            walk_rust_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

// ---------------------------------------------------------------------
// Reports
// ---------------------------------------------------------------------

#[derive(Default, Debug)]
struct AuditReport {
    // Typed dimensional verdicts. `invalid` is a hard failure (a
    // blocking dimension — Existence or ClaimSupport — is Unverified);
    // `with_issues` is informational (a non-blocking dimension is
    // Unverified). Each carries the offending dimension names.
    invalid: Vec<(String, Vec<&'static str>)>, // slug, blocking dimensions
    with_issues: Vec<(String, Vec<&'static str>)>, // slug, non-blocking dimensions
    valid_count: usize,
    missing_slugs: Vec<(String, PathBuf)>, // slug, file
    freeform_count: usize,
    freeform_examples: Vec<(PathBuf, usize, String)>, // first ~5
}

impl AuditReport {
    fn has_failures(&self) -> bool {
        !self.invalid.is_empty() || !self.missing_slugs.is_empty()
    }
}

fn render_report(report: &AuditReport) -> String {
    let mut s = String::new();
    s.push_str("\n=== Praxis citation audit ===\n\n");

    // Per-verdict distribution over the registry (the typed gate).
    s.push_str(&format!(
        "Verdict distribution over {} registry entries:\n\
         \x20 Valid           : {}\n\
         \x20 ValidWithIssues : {}\n\
         \x20 Invalid         : {}\n\n",
        report.valid_count + report.with_issues.len() + report.invalid.len(),
        report.valid_count,
        report.with_issues.len(),
        report.invalid.len(),
    ));

    if !report.invalid.is_empty() {
        s.push_str(&format!(
            "FAIL: {} citation entr(y/ies) are Invalid — a blocking\n\
             dimension (Existence or ClaimSupport) is Unverified:\n",
            report.invalid.len()
        ));
        for (slug, dims) in &report.invalid {
            s.push_str(&format!("  - {slug}  [blocking: {}]\n", dims.join(", ")));
        }
        s.push_str(
            "\nAction: open the cited work at the section recorded in\n\
             citations.toml, confirm content_summary matches, and fill in\n\
             verified_by + verified_on + verification_method (ClaimSupport),\n\
             or supply authors/title/an identifier (Existence). If the\n\
             section can't be confirmed, use the downgrade path documented\n\
             in the citations.toml header.\n\n",
        );
    }

    if !report.with_issues.is_empty() {
        s.push_str(&format!(
            "INFO: {} citation entr(y/ies) are ValidWithIssues — a\n\
             non-blocking dimension is Unverified (no CI failure):\n",
            report.with_issues.len()
        ));
        for (slug, dims) in &report.with_issues {
            s.push_str(&format!("  - {slug}  [issues: {}]\n", dims.join(", ")));
        }
        s.push('\n');
    }

    if !report.missing_slugs.is_empty() {
        s.push_str(&format!(
            "FAIL: {} code reference(s) point to slugs not in citations.toml:\n",
            report.missing_slugs.len()
        ));
        for (slug, file) in &report.missing_slugs {
            s.push_str(&format!("  - {slug}  ({})\n", file.display()));
        }
        s.push_str(
            "\nAction: either add the missing entry to citations.toml or\n\
             fix the slug spelling in the code reference.\n\n",
        );
    }

    s.push_str(&format!(
        "INFO: {} free-form citation occurrence(s) detected in scoped\n\
         directories. These are Phase A (no failure); Phase B will migrate\n\
         them to slug references and start failing CI on free-form cites.\n",
        report.freeform_count
    ));
    if !report.freeform_examples.is_empty() {
        s.push_str("First few:\n");
        for (file, lineno, line) in &report.freeform_examples {
            s.push_str(&format!(
                "  - {}:{}  {}\n",
                file.display(),
                lineno,
                line.trim().chars().take(120).collect::<String>()
            ));
        }
    }

    if report.has_failures() {
        s.push_str(
            "\n=== CI: RED ===\n\
             Per the no-allow-list policy: unverified citations and missing\n\
             slugs MUST be addressed before CI can pass. See the\n\
             citations.toml header for the verification workflow.\n",
        );
    } else {
        s.push_str("\n=== CI: green ===\nAll registered citations are verified and every slug reference resolves.\n");
    }

    s
}

// ---------------------------------------------------------------------
// The test
// ---------------------------------------------------------------------

#[test]
fn citation_audit() {
    let registry = load_registry();
    let mut report = AuditReport::default();

    // (1) Typed dimensional verdict per entry. Each registry entry is
    //     projected into a `CitationAssessment` (the citation_quality
    //     module's grounded field→dimension mapping) and folded to a
    //     `CitationVerdict`. This replaces the old flat `verified_by`
    //     check: the blocking ClaimSupport dimension *is* the old gate
    //     (content_summary + verified_by, or claim + all fibers for a
    //     multi-version entry), and Existence is an additional blocking
    //     dimension. An `Invalid` verdict (a blocking dimension is
    //     Unverified) is a hard CI failure; `ValidWithIssues` (only a
    //     non-blocking dimension is Unverified) is informational.
    for (slug, entry) in &registry.citations {
        let assessment = project(entry);
        match assessment.verdict() {
            CitationVerdict::Valid => report.valid_count += 1,
            CitationVerdict::ValidWithIssues => {
                let dims: Vec<&'static str> = unverified_dimensions(&assessment)
                    .into_iter()
                    .map(|(name, _)| name)
                    .collect();
                report.with_issues.push((slug.clone(), dims));
            }
            CitationVerdict::Invalid => {
                // Report only the blocking dimensions that drove Invalid.
                let blocking: Vec<&'static str> = unverified_dimensions(&assessment)
                    .into_iter()
                    .filter(|(_, is_blocking)| *is_blocking)
                    .map(|(name, _)| name)
                    .collect();
                report.invalid.push((slug.clone(), blocking));
            }
        }
    }

    // (2) Slug references in code that don't resolve, plus free-form
    //     citation counting.
    let root = workspace_root();
    let mut files = Vec::new();
    for d in SCOPED_DIRS {
        walk_rust_files(&root.join(d), &mut files);
    }

    let registered: BTreeSet<&String> = registry.citations.keys().collect();

    for file in &files {
        let text = match fs::read_to_string(file) {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Slug references.
        for slug in extract_slug_references(&text) {
            if !registered.contains(&slug) {
                report.missing_slugs.push((slug, file.clone()));
            }
        }

        // Free-form citation counting.
        for (lineno, line) in text.lines().enumerate() {
            for pat in FREEFORM_PATTERNS {
                if line.contains(pat) {
                    report.freeform_count += 1;
                    if report.freeform_examples.len() < 5 {
                        report
                            .freeform_examples
                            .push((file.clone(), lineno + 1, line.to_string()));
                    }
                    break;
                }
            }
        }
    }

    // Sort for stable output (registry iteration is already sorted by
    // slug, but the slug-reference scan is file-order).
    report.invalid.sort();
    report.with_issues.sort();
    report.missing_slugs.sort();

    let rendered = render_report(&report);
    eprintln!("{rendered}");

    assert!(
        !report.has_failures(),
        "citation audit failed — see report above"
    );
}

// ---------------------------------------------------------------------
// Self-tests for the audit's own helpers
// ---------------------------------------------------------------------

#[test]
fn slug_predicate_accepts_kebab_underscore() {
    assert!(is_slug("mac_lane_1971_categories_working_mathematician_i3"));
    assert!(is_slug("gao_2012_xsd_1_1_part_1_2_2"));
    assert!(!is_slug(""));
    assert!(!is_slug("MacLane1971"));
    assert!(!is_slug("mac-lane"));
    assert!(!is_slug("mac lane"));
}

#[test]
fn slug_extractor_finds_macro_form() {
    let text = r"some code cite![mac_lane_1971_categories_working_mathematician_i3] more code";
    let slugs = extract_slug_references(text);
    assert_eq!(slugs.len(), 1);
    assert!(slugs.contains("mac_lane_1971_categories_working_mathematician_i3"));
}

#[test]
fn slug_extractor_finds_html_comment_form() {
    let text = "<!-- cite:gao_2012_xsd_1_1_part_1_3_8_1 --> trailing";
    let slugs = extract_slug_references(text);
    assert_eq!(slugs.len(), 1);
    assert!(slugs.contains("gao_2012_xsd_1_1_part_1_3_8_1"));
}

#[test]
fn registry_loads() {
    let registry = load_registry();
    // The registry should have at least the bulk-extracted M4.ε / M4.η
    // citations. If this assertion fires, citations.toml has been
    // truncated or restructured.
    assert!(
        registry.citations.len() >= 30,
        "citations.toml lost entries — only {} present",
        registry.citations.len()
    );
}
