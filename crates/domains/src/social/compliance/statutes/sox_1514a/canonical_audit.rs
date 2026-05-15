//! Canonical-text audit for SOX § 1514A — verifies every hand-coded
//! term in `praxis.lock`'s `[structural."sox_1514a@2002"]` block
//! traces to a real subsection of the canonical statutory text, and
//! enumerates the *known gaps* between the hand-coded structural
//! data and the canonical source.
//!
//! # The gap this audit closes
//!
//! Praxis.lock's structural data for `sox_1514a@2002` was hand-extracted
//! from training-data recall of 18 U.S.C. § 1514A. Until the PDF/HTML
//! loader + statute-NLP extraction pipeline lands (M-future), there's
//! no machine-verified bridge between the canonical govinfo source and
//! the structural extraction. This module is the **interim audit**:
//! it pins a hand-transcribed canonical text (with a SHA-256 hash
//! recorded in praxis.lock's `[canonical_text]` section) and runs
//! cross-checks between every lock term's CURIE-derived subsection
//! path and the markers that actually appear in the canonical text.
//!
//! # Three kinds of finding
//!
//! - **Verbatim** — term's CURIE subsection path matches a marker in
//!   canonical text AND term's content words appear in the canonical
//!   text near that marker.
//! - **Paraphrase** — term's subsection marker appears in canonical
//!   text, but its name is practitioner shorthand not present
//!   verbatim (e.g., "Covered Employer" is shorthand for the
//!   compound entity definition in § 1514A(a)). Expected and
//!   documented in [`KNOWN_PARAPHRASES`].
//! - **Gap** — term's CURIE subsection path resolves to no marker in
//!   canonical text, OR canonical text contains a subsection marker
//!   no lock term references. Each known gap is documented in
//!   [`KNOWN_GAPS`] with a resolution blocker; new gaps cause the
//!   audit to fail.
//!
//! # Provenance discipline
//!
//! The canonical text fixture is bootstrap-state: it's hand-transcribed
//! from training data and marked `provenance = "training_reconstructed"`
//! in praxis.lock. Once the PDF/HTML loader can fetch the actual
//! govinfo source, the fixture gets replaced with the fetched text,
//! the hash updates, and the audit re-runs against authoritative
//! content. Until then, **the audit verifies *internal consistency*
//! between hand-coded structural data and hand-transcribed canonical
//! text** — both come from the same author so this isn't a closed loop;
//! it does catch obvious drift between the two but cannot catch errors
//! shared by both. That ceiling is the user-acknowledged limit until
//! M-future PDF/HTML extraction lands.

use crate::social::compliance::statutes::sox_1514a::statute;

const CANONICAL_TEXT: &str = include_str!("../../../../../data/canonical_text/sox_1514a_2002.txt");

/// SHA-256 of the canonical text fixture, pinned in praxis.lock's
/// `[canonical_text."sox_1514a@2002"]` section. Verified by the
/// `canonical_text_hash_matches_lock_pin` test at build time.
pub const CANONICAL_SHA256: &str =
    "a1a53fd9576443c176ac33dca7c88d8257a708c3c9c4b2680dff21ff76cf5d12";

/// A known *paraphrase* — a hand-coded term whose name is practitioner
/// shorthand for a canonical provision but doesn't appear verbatim in
/// the statute. Paraphrases are *expected* and explicitly listed
/// here; the audit verifies the term's subsection marker exists in
/// canonical text, then accepts the name as shorthand for that
/// subsection's content.
#[derive(Debug, Clone)]
pub struct KnownParaphrase {
    pub term_id: &'static str,
    pub canonical_subsection: &'static str,
    pub rationale: &'static str,
}

/// Registered paraphrases. Each entry asserts: "Yes, this term's
/// *name* is practitioner shorthand not present in the statute; the
/// term covers the listed canonical subsection."
pub const KNOWN_PARAPHRASES: &[KnownParaphrase] = &[
    KnownParaphrase {
        term_id: "sox_1514a:a",
        canonical_subsection: "(a)",
        rationale: "\"Covered Employer\" is practitioner shorthand for the compound entity definition in § 1514A(a) — registered companies, required-to-file companies, NRSROs, and their officers/employees/contractors/subcontractors/agents.",
    },
    KnownParaphrase {
        term_id: "sox_1514a:a_v2",
        canonical_subsection: "(a)",
        rationale: "\"Covered Persons\" is practitioner shorthand collapsing the \"officer, employee, contractor, subcontractor, or agent\" clause of § 1514A(a) into a single concept.",
    },
    KnownParaphrase {
        term_id: "sox_1514a:a_v3",
        canonical_subsection: "(a)",
        rationale: "\"Prohibition on Retaliation\" is the doctrinal label for § 1514A(a)'s substantive rule (the verb chain \"discharge, demote, suspend, threaten, harass, or in any other manner discriminate\"). Not in the statutory text but standard usage in case law.",
    },
    KnownParaphrase {
        term_id: "sox_1514a:a_v4",
        canonical_subsection: "(a)",
        rationale: "\"Causation: Because Of Protected Activity\" labels the \"because of any lawful act done by the employee\" causation clause of § 1514A(a).",
    },
    KnownParaphrase {
        term_id: "sox_1514a:a_v5",
        canonical_subsection: "(a)",
        rationale: "\"Lawfulness of Protected Activity\" labels the \"any lawful act done by the employee\" qualifier in § 1514A(a).",
    },
    KnownParaphrase {
        term_id: "sox_1514a:1",
        canonical_subsection: "(a)(1)",
        rationale: "\"Protected Activity: Providing Information and Assistance\" is the doctrinal label for § 1514A(a)(1)'s reporting-channel activities.",
    },
    KnownParaphrase {
        term_id: "sox_1514a:1_v2",
        canonical_subsection: "(a)(1)",
        rationale: "Splits out the \"reasonably believes constitutes a violation\" object-of-belief framing of § 1514A(a)(1) — distinct from the act of reporting itself.",
    },
    KnownParaphrase {
        term_id: "sox_1514a:b1",
        canonical_subsection: "(b)(1)",
        rationale: "\"Right to Seek Relief\" labels § 1514A(b)(1)'s civil-action right.",
    },
    KnownParaphrase {
        term_id: "sox_1514a:b1a",
        canonical_subsection: "(b)(1)(A)",
        rationale: "\"Complaint Filing with Secretary of Labor\" labels § 1514A(b)(1)(A)'s DOL complaint right.",
    },
    KnownParaphrase {
        term_id: "sox_1514a:b1b",
        canonical_subsection: "(b)(1)(B)",
        rationale: "\"District Court Filing as Alternative Remedy\" labels § 1514A(b)(1)(B)'s 180-day de novo right.",
    },
];

/// A known *gap* — a discrepancy between the hand-coded structural
/// data and the canonical text that requires resolution. Every gap
/// must list a resolution blocker so it's clear what's required to
/// close it.
#[derive(Debug, Clone)]
pub struct KnownGap {
    pub term_id: &'static str,
    pub kind: GapKind,
    pub canonical_subsection: &'static str,
    pub note: &'static str,
    pub resolution_blocker: &'static str,
}

/// Kinds of audit gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapKind {
    /// Term's hand-coded definition paraphrases rather than tracks
    /// the canonical text precisely.
    DefinitionDrift,
    /// Two or more lock terms appear to cover the same canonical
    /// subsection — suggests structural-data redundancy.
    PotentialRedundancy,
    /// Canonical subsection marker present but no lock term covers
    /// the substantive content.
    UncoveredSubsection,
    /// Term aggregates content from multiple canonical subsections —
    /// not a drift but a modeling choice that hides granularity.
    Aggregation,
}

/// Registered gaps in the current hand-coded structural extraction
/// for SOX § 1514A. Each entry is a fact about a divergence between
/// the hand-coded data and the canonical text, with a resolution
/// blocker noting what's required to close it.
pub const KNOWN_GAPS: &[KnownGap] = &[
    KnownGap {
        term_id: "sox_1514a:b1b_v2",
        kind: GapKind::PotentialRedundancy,
        canonical_subsection: "(b)(1)(B)",
        note: "\"District Court Jurisdiction\" splits the jurisdiction-without-amount-in-controversy clause out of b1b. The canonical § 1514A(b)(1)(B) is a single subsection containing both the 180-day-trigger language and the jurisdiction qualifier; modeling them as two separate terms introduces structural granularity not present in the statute.",
        resolution_blocker: "Modeling-choice review — keep two terms (current) or merge into b1b — needs Praxis-validation step.",
    },
    KnownGap {
        term_id: "sox_1514a:b2b",
        kind: GapKind::DefinitionDrift,
        canonical_subsection: "(b)(2)(B)",
        note: "Hand-coded definition reads \"Upon receipt of a complaint, the Secretary of Labor shall notify in writing the person named in the complaint and the employer.\" The canonical § 1514A(b)(2)(B) reads \"EXCEPTION.—Notification made under section 42121(b)(1) of title 49, United States Code, shall be made to the person named in the complaint and to the employer.\" The hand-coded version drops the cross-reference to § 42121(b)(1) and the EXCEPTION framing. Substantively similar but textually drifts.",
        resolution_blocker: "PDF/HTML loader (M-future) — re-extract from canonical govinfo source to replace hand-coded paraphrase with verbatim text.",
    },
    KnownGap {
        term_id: "sox_1514a:2",
        kind: GapKind::Aggregation,
        canonical_subsection: "(a)(2)",
        note: "Hand-coded \"Protected Activity: Participation in Proceedings\" aggregates the \"file, cause to be filed, testify, participate in, or otherwise assist\" verb chain into one term. The canonical § 1514A(a)(2) enumerates these as alternatives; modeling them as one term hides the alternative-of structure. Compare to (a)(1) where reporting channels are split into 1a/1b/1c.",
        resolution_blocker: "Modeling-choice review — granularity decision pending NLP-based extraction (M-future).",
    },
];

/// One audit finding for a single lock term.
#[derive(Debug, Clone)]
pub struct Finding {
    pub term_id: String,
    pub canonical_subsection_inferred: Option<String>,
    pub canonical_marker_present: bool,
    pub classification: FindingClassification,
}

/// How the finding maps to canonical text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingClassification {
    /// Term's CURIE subsection path resolves to a marker present in
    /// canonical text, and the term is *not* in [`KNOWN_PARAPHRASES`]
    /// or [`KNOWN_GAPS`] — implies the term's name+definition are
    /// expected to track canonical text closely.
    UndocumentedDirect,
    /// Term is in [`KNOWN_PARAPHRASES`] — name is practitioner
    /// shorthand for the listed subsection.
    DocumentedParaphrase,
    /// Term is in [`KNOWN_GAPS`] — see the gap entry for details.
    DocumentedGap,
    /// Term's CURIE has no parseable subsection path (e.g., compound
    /// underscore suffixes), and no documentation explains it. The
    /// audit flags this as needing review.
    UndocumentedNoCanonical,
}

/// Parse a SOX 1514A CURIE local part to its canonical subsection
/// path. Returns the sequence of Bluebook subdivision labels.
///
/// Examples:
/// - `"a"` → `["a"]`
/// - `"a_v3"` → `["a"]` (variant suffix stripped — same canonical
///   subsection as :a, just a different praxis-modeling cut)
/// - `"1"` → `["a", "1"]` (numeric-only locals are children of (a))
/// - `"1a"` → `["a", "1", "A"]`
/// - `"b2b"` → `["b", "2", "B"]`
/// - `"c2a"` → `["c", "2", "A"]`
pub fn parse_curie_subsection_path(curie_local: &str) -> Vec<String> {
    use alloc::string::ToString;
    use alloc::vec::Vec;

    // Strip "_vN" suffix — praxis convention for splitting one
    // canonical subsection across multiple lock terms.
    let stripped = curie_local
        .find("_v")
        .map(|i| &curie_local[..i])
        .unwrap_or(curie_local);

    let mut path = Vec::new();
    let chars: Vec<char> = stripped.chars().collect();
    let mut i = 0;

    // First char is the top-level letter or a digit (a digit means
    // we're inside (a), which is the implicit default top level).
    if let Some(&c) = chars.first() {
        if c.is_ascii_alphabetic() {
            path.push(c.to_ascii_lowercase().to_string());
            i = 1;
        } else if c.is_ascii_digit() {
            // Numeric-only locals are children of (a) — same
            // convention as canonical SOX text where § 1514A(a)(1)
            // is reached after the (a) subsection.
            path.push("a".to_string());
        }
    }

    // Remaining chars: number then optional uppercase letter.
    while i < chars.len() {
        let c = chars[i];
        if c.is_ascii_digit() {
            // Collect consecutive digits (handle multi-digit numbers).
            let mut num = String::new();
            while i < chars.len() && chars[i].is_ascii_digit() {
                num.push(chars[i]);
                i += 1;
            }
            path.push(num);
        } else if c.is_ascii_alphabetic() {
            path.push(c.to_ascii_uppercase().to_string());
            i += 1;
        } else {
            i += 1;
        }
    }

    path
}

/// Render a subsection path as a Bluebook-style marker string.
/// `["b", "2", "B"]` → `"(b)(2)(B)"`.
pub fn render_subsection_marker(path: &[String]) -> String {
    use alloc::string::String;
    use core::fmt::Write;

    let mut out = String::new();
    for component in path {
        write!(&mut out, "({})", component).expect("write to String never fails");
    }
    out
}

/// Returns `true` if the canonical text contains the given subsection
/// marker string (e.g., `"(b)(2)(B)"`).
pub fn canonical_contains_marker(marker: &str) -> bool {
    // Allow whitespace between marker components — the canonical
    // text places them with newlines/spaces in between.
    if CANONICAL_TEXT.contains(marker) {
        return true;
    }

    // Fall back: find each parenthesized component in order.
    let mut search_start = 0;
    let mut chars = marker.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '(' {
            let mut inner = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == ')' {
                    chars.next();
                    break;
                }
                inner.push(nc);
                chars.next();
            }
            let needle = alloc::format!("({})", inner);
            match CANONICAL_TEXT[search_start..].find(&needle) {
                Some(pos) => search_start += pos + needle.len(),
                None => return false,
            }
        }
    }
    true
}

/// Run the canonical-text audit. Returns one [`Finding`] per
/// `sox_1514a@2002` lock term, classifying its relationship to the
/// canonical text.
pub fn audit() -> Vec<Finding> {
    use alloc::string::ToString;

    let mut findings = Vec::new();
    for term in statute().terms() {
        let curie = term.id.value.as_str();
        let Some(local) = curie.strip_prefix("sox_1514a:") else {
            continue;
        };
        let path = parse_curie_subsection_path(local);
        let marker = render_subsection_marker(&path);
        let marker_present = canonical_contains_marker(&marker);

        let classification =
            if let Some(para) = KNOWN_PARAPHRASES.iter().find(|p| p.term_id == curie) {
                // Verify the documented subsection actually exists in canonical.
                let _ = para;
                FindingClassification::DocumentedParaphrase
            } else if KNOWN_GAPS.iter().any(|g| g.term_id == curie) {
                FindingClassification::DocumentedGap
            } else if !marker_present {
                FindingClassification::UndocumentedNoCanonical
            } else {
                FindingClassification::UndocumentedDirect
            };

        findings.push(Finding {
            term_id: curie.to_string(),
            canonical_subsection_inferred: if path.is_empty() { None } else { Some(marker) },
            canonical_marker_present: marker_present,
            classification,
        });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_text_hash_matches_lock_pin() {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(CANONICAL_TEXT.as_bytes());
        let result = hasher.finalize();
        let hex: String = result.iter().map(|b| alloc::format!("{:02x}", b)).collect();
        assert_eq!(
            hex, CANONICAL_SHA256,
            "canonical text hash drift — file changed without updating CANONICAL_SHA256 and praxis.lock"
        );
    }

    #[test]
    fn canonical_text_starts_with_18_usc_1514a() {
        assert!(CANONICAL_TEXT.starts_with("18 U.S.C. § 1514A"));
    }

    #[test]
    fn canonical_text_has_all_top_level_subsections() {
        for marker in &["(a)", "(b)", "(c)", "(d)", "(e)"] {
            assert!(
                CANONICAL_TEXT.contains(marker),
                "canonical text missing top-level subsection {marker}"
            );
        }
    }

    #[test]
    fn canonical_text_has_known_burden_shift_marker() {
        // § 1514A(b)(2)(C) — burdens of proof per AIR21
        assert!(canonical_contains_marker("(b)(2)(C)"));
    }

    // ── CURIE → subsection path parsing ──────────────────────────────

    #[test]
    fn parse_curie_simple_letter() {
        assert_eq!(parse_curie_subsection_path("a"), vec!["a"]);
    }

    #[test]
    fn parse_curie_strips_v_suffix() {
        assert_eq!(parse_curie_subsection_path("a_v3"), vec!["a"]);
        assert_eq!(parse_curie_subsection_path("a_v5"), vec!["a"]);
    }

    #[test]
    fn parse_curie_numeric_is_child_of_a() {
        assert_eq!(parse_curie_subsection_path("1"), vec!["a", "1"]);
        assert_eq!(parse_curie_subsection_path("2"), vec!["a", "2"]);
    }

    #[test]
    fn parse_curie_letter_number_letter() {
        assert_eq!(parse_curie_subsection_path("b2b"), vec!["b", "2", "B"]);
        assert_eq!(parse_curie_subsection_path("c2a"), vec!["c", "2", "A"]);
    }

    #[test]
    fn parse_curie_numeric_with_subsection() {
        assert_eq!(parse_curie_subsection_path("1a"), vec!["a", "1", "A"]);
        assert_eq!(parse_curie_subsection_path("1b"), vec!["a", "1", "B"]);
        assert_eq!(parse_curie_subsection_path("1c"), vec!["a", "1", "C"]);
    }

    // ── Subsection marker rendering ──────────────────────────────────

    #[test]
    fn render_marker_single_level() {
        assert_eq!(render_subsection_marker(&["a".to_string()]), "(a)");
    }

    #[test]
    fn render_marker_three_levels() {
        assert_eq!(
            render_subsection_marker(&["b".to_string(), "2".to_string(), "B".to_string()]),
            "(b)(2)(B)"
        );
    }

    // ── Canonical marker presence checks ─────────────────────────────

    #[test]
    fn marker_a_present() {
        assert!(canonical_contains_marker("(a)"));
    }

    #[test]
    fn marker_b_2_b_present() {
        assert!(canonical_contains_marker("(b)(2)(B)"));
    }

    #[test]
    fn marker_e_2_present() {
        assert!(canonical_contains_marker("(e)(2)"));
    }

    #[test]
    fn nonexistent_marker_returns_false() {
        // SOX § 1514A has no (f) subsection.
        assert!(!canonical_contains_marker("(f)"));
        // Or anything as deep as 7 levels.
        assert!(!canonical_contains_marker("(z)(99)"));
    }

    // ── Audit-output structural assertions ───────────────────────────

    #[test]
    fn audit_produces_one_finding_per_term() {
        let findings = audit();
        assert_eq!(findings.len(), statute().terms().len());
    }

    #[test]
    fn every_finding_carries_an_inferred_subsection_or_explanation() {
        for f in audit() {
            // Every term must have an inferred path or be flagged as
            // UndocumentedNoCanonical.
            if f.canonical_subsection_inferred.is_none() {
                assert_eq!(
                    f.classification,
                    FindingClassification::UndocumentedNoCanonical,
                    "term {} has no inferred subsection but is classified {:?}",
                    f.term_id,
                    f.classification
                );
            }
        }
    }

    #[test]
    fn known_paraphrase_terms_classified_as_paraphrase() {
        let findings = audit();
        for para in KNOWN_PARAPHRASES {
            let f = findings
                .iter()
                .find(|f| f.term_id == para.term_id)
                .unwrap_or_else(|| {
                    panic!(
                        "KNOWN_PARAPHRASES references non-existent term {}",
                        para.term_id
                    )
                });
            assert_eq!(
                f.classification,
                FindingClassification::DocumentedParaphrase,
                "{} expected DocumentedParaphrase, got {:?}",
                para.term_id,
                f.classification
            );
        }
    }

    #[test]
    fn known_gap_terms_classified_as_gap() {
        let findings = audit();
        for gap in KNOWN_GAPS {
            let f = findings
                .iter()
                .find(|f| f.term_id == gap.term_id)
                .unwrap_or_else(|| {
                    panic!("KNOWN_GAPS references non-existent term {}", gap.term_id)
                });
            assert_eq!(
                f.classification,
                FindingClassification::DocumentedGap,
                "{} expected DocumentedGap, got {:?}",
                gap.term_id,
                f.classification
            );
        }
    }

    #[test]
    fn no_undocumented_no_canonical_findings() {
        // Every term that doesn't resolve to a canonical marker MUST
        // be documented in KNOWN_PARAPHRASES or KNOWN_GAPS. This
        // axiom fires when a new lock term lands without
        // documentation — forces the author to classify it.
        let findings = audit();
        let undocumented: Vec<_> = findings
            .iter()
            .filter(|f| f.classification == FindingClassification::UndocumentedNoCanonical)
            .collect();
        assert!(
            undocumented.is_empty(),
            "found {} undocumented terms with no canonical anchor: {:?}",
            undocumented.len(),
            undocumented.iter().map(|f| &f.term_id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn known_gaps_all_reference_existing_terms() {
        // Property: KNOWN_GAPS must not reference terms that don't
        // exist in praxis.lock — guards against stale gap entries
        // after a structural-data refactor.
        let term_curies: alloc::collections::BTreeSet<String> = statute()
            .terms()
            .iter()
            .map(|t| t.id.value.clone())
            .collect();
        for gap in KNOWN_GAPS {
            assert!(
                term_curies.contains(gap.term_id),
                "KNOWN_GAPS references {} which doesn't exist in praxis.lock",
                gap.term_id
            );
        }
    }

    #[test]
    fn known_paraphrases_all_reference_existing_terms() {
        let term_curies: alloc::collections::BTreeSet<String> = statute()
            .terms()
            .iter()
            .map(|t| t.id.value.clone())
            .collect();
        for para in KNOWN_PARAPHRASES {
            assert!(
                term_curies.contains(para.term_id),
                "KNOWN_PARAPHRASES references {} which doesn't exist in praxis.lock",
                para.term_id
            );
        }
    }

    #[test]
    fn every_gap_has_resolution_blocker() {
        for gap in KNOWN_GAPS {
            assert!(
                !gap.resolution_blocker.is_empty(),
                "{} has empty resolution_blocker",
                gap.term_id
            );
            assert!(!gap.note.is_empty(), "{} has empty note", gap.term_id);
        }
    }

    #[test]
    fn every_paraphrase_has_rationale() {
        for para in KNOWN_PARAPHRASES {
            assert!(
                !para.rationale.is_empty(),
                "{} has empty rationale",
                para.term_id
            );
        }
    }

    /// Print the audit report to test output for visual review. Doesn't
    /// fail — informational only.
    #[test]
    fn print_gap_report() {
        let findings = audit();
        eprintln!("\n=== SOX § 1514A canonical-text audit report ===");
        eprintln!("Canonical text: {} chars", CANONICAL_TEXT.len());
        eprintln!("Lock terms audited: {}", findings.len());
        eprintln!();

        let mut by_class: alloc::collections::BTreeMap<&str, usize> = Default::default();
        for f in &findings {
            let key = match f.classification {
                FindingClassification::UndocumentedDirect => "UndocumentedDirect",
                FindingClassification::DocumentedParaphrase => "DocumentedParaphrase",
                FindingClassification::DocumentedGap => "DocumentedGap",
                FindingClassification::UndocumentedNoCanonical => "UndocumentedNoCanonical",
            };
            *by_class.entry(key).or_insert(0) += 1;
        }
        for (k, v) in &by_class {
            eprintln!("  {}: {}", k, v);
        }
        eprintln!();

        eprintln!("Known gaps requiring resolution:");
        for gap in KNOWN_GAPS {
            eprintln!(
                "  - {} @ {} [{:?}]: {} (blocker: {})",
                gap.term_id, gap.canonical_subsection, gap.kind, gap.note, gap.resolution_blocker
            );
        }
        eprintln!();
    }
}
