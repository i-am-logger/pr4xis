//! Canonical-text audit for AIR21 § 42121 — same pattern as
//! `sox_1514a::canonical_audit`, scoped to 49 U.S.C. § 42121.
//!
//! Pins a hand-transcribed canonical text fixture for the substantive
//! prohibition (subsection (a)) and procedural framework (subsections
//! (b)(1)-(b)(6) including the four-clause burden-shifting structure
//! in (b)(2)(B)(i)-(iv)). Cross-references each lock term's
//! CURIE-derived subsection path against canonical text markers and
//! produces a per-term audit finding.
//!
//! # CURIE convention for AIR21
//!
//! Slightly different from SOX 1514A's convention — AIR21 CURIEs use
//! `_` as the *subdivision separator* (every subdivision below the
//! letter-digit-letter triplet appears after an underscore):
//!
//! - `air21_42121:a` → `(a)`
//! - `air21_42121:a_1` → `(a)(1)` (NOT a praxis variant — actual (a)(1))
//! - `air21_42121:b1` → `(b)(1)`
//! - `air21_42121:b2b` → `(b)(2)(B)`
//! - `air21_42121:b2b_i` → `(b)(2)(B)(i)` (Roman numeral 4th-level)
//!
//! SOX 1514A's convention differs: SOX uses `_vN` as a *praxis
//! variant suffix* (`:a_v3` is variant 3 of (a), still mapping to
//! the same subsection). Future commits should unify the two
//! conventions, but for now each statute's audit module owns its
//! own CURIE parser.

use crate::social::compliance::statutes::air21_42121::statute;

const CANONICAL_TEXT: &str =
    include_str!("../../../../../data/canonical_text/air21_42121_2010.txt");

/// SHA-256 of the canonical text fixture, pinned in praxis.lock's
/// `[canonical_text."air21_42121@2010"]` section.
pub const CANONICAL_SHA256: &str =
    "4fd31ae95d746b142fc72c22b04a1592578418f5780dd2caa99e75a9bc5582c1";

/// A known paraphrase — practitioner shorthand not present verbatim
/// in the statute but covering an enumerated subsection.
#[derive(Debug, Clone)]
pub struct KnownParaphrase {
    pub term_id: &'static str,
    pub canonical_subsection: &'static str,
    pub rationale: &'static str,
}

/// AIR21-specific paraphrases.
pub const KNOWN_PARAPHRASES: &[KnownParaphrase] = &[
    KnownParaphrase {
        term_id: "air21_42121:a",
        canonical_subsection: "(a)",
        rationale: "\"Discrimination Prohibited\" is the canonical subsection heading. Term name closely tracks the statute.",
    },
    KnownParaphrase {
        term_id: "air21_42121:a_1",
        canonical_subsection: "(a)(1)",
        rationale: "\"Protected Activity: Provided Information\" is doctrinal shorthand for the (a)(1) information-provision clause.",
    },
    KnownParaphrase {
        term_id: "air21_42121:a_2",
        canonical_subsection: "(a)(2)",
        rationale: "\"Protected Activity: Filed Proceeding\" is doctrinal shorthand for the (a)(2) filing-of-proceeding clause.",
    },
    KnownParaphrase {
        term_id: "air21_42121:a_3",
        canonical_subsection: "(a)(3)",
        rationale: "\"Protected Activity: Testified or Participated\" labels the (a)(3) testify-or-will-testify clause.",
    },
    KnownParaphrase {
        term_id: "air21_42121:a_4",
        canonical_subsection: "(a)(4)",
        rationale: "\"Protected Activity: Assisted or Participated\" labels the (a)(4) assist-or-participate clause.",
    },
    KnownParaphrase {
        term_id: "air21_42121:b1",
        canonical_subsection: "(b)(1)",
        rationale: "\"Complaint Filing with Secretary of Labor\" is doctrinal shorthand for (b)(1) FILING AND NOTIFICATION.",
    },
    KnownParaphrase {
        term_id: "air21_42121:b2",
        canonical_subsection: "(b)(2)",
        rationale: "\"Investigation\" is doctrinal shorthand for (b)(2) INVESTIGATION; PRELIMINARY ORDER umbrella.",
    },
    KnownParaphrase {
        term_id: "air21_42121:b2a",
        canonical_subsection: "(b)(2)(A)",
        rationale: "\"Investigation Procedures\" labels (b)(2)(A) IN GENERAL clause's 60-day investigation framework.",
    },
    KnownParaphrase {
        term_id: "air21_42121:b2b",
        canonical_subsection: "(b)(2)(B)",
        rationale: "\"Required Showing of Complaint\" is the canonical (b)(2)(B) REQUIREMENTS umbrella term.",
    },
    KnownParaphrase {
        term_id: "air21_42121:b2b_i",
        canonical_subsection: "(b)(2)(B)(i)",
        rationale: "\"Prima Facie Investigation Gate\" labels the (b)(2)(B)(i) Required showing by complainant clause.",
    },
    KnownParaphrase {
        term_id: "air21_42121:b2b_ii",
        canonical_subsection: "(b)(2)(B)(ii)",
        rationale: "\"Investigation-Gate Same-Action Defense\" labels the (b)(2)(B)(ii) Showing by employer clause.",
    },
    KnownParaphrase {
        term_id: "air21_42121:b2b_iii",
        canonical_subsection: "(b)(2)(B)(iii)",
        rationale: "\"Merits Contributing-Factor Demonstration\" labels the (b)(2)(B)(iii) Criteria for determination by Secretary clause.",
    },
    KnownParaphrase {
        term_id: "air21_42121:b2b_iv",
        canonical_subsection: "(b)(2)(B)(iv)",
        rationale: "\"Merits Same-Action Defense\" labels the (b)(2)(B)(iv) Prohibition clause.",
    },
    KnownParaphrase {
        term_id: "air21_42121:b3",
        canonical_subsection: "(b)(3)",
        rationale: "\"Final Order\" matches the (b)(3) FINAL ORDER heading.",
    },
    KnownParaphrase {
        term_id: "air21_42121:b6",
        canonical_subsection: "(b)(6)",
        rationale: "\"Filing Penalties\" labels (b)(6) FRIVOLOUS COMPLAINTS clause.",
    },
];

/// A known gap — explicit discrepancy with resolution blocker.
#[derive(Debug, Clone)]
pub struct KnownGap {
    pub term_id: &'static str,
    pub kind: GapKind,
    pub canonical_subsection: &'static str,
    pub note: &'static str,
    pub resolution_blocker: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GapKind {
    DefinitionDrift,
    PotentialRedundancy,
    UncoveredSubsection,
    Aggregation,
}

pub const KNOWN_GAPS: &[KnownGap] = &[
    KnownGap {
        term_id: "air21_42121:b4",
        kind: GapKind::DefinitionDrift,
        canonical_subsection: "(b)(4)",
        note: "Hand-coded \"Civil Action to Enforce Order\" with the 210/90-day de-novo language. The canonical (b)(4) is REVIEW with subparagraph (b)(4)(A) APPEAL TO COURT OF APPEALS — not enforcement. The de-novo right is actually in canonical (b)(5) DE NOVO REVIEW; (b)(4) covers circuit-court review of administrative orders. The hand-coded definition conflates these.",
        resolution_blocker: "PDF/HTML loader (M-future) — re-extract from canonical govinfo source; likely split into separate (b)(4) review and (b)(5) de novo terms or rename b4 to match canonical REVIEW heading.",
    },
    KnownGap {
        term_id: "air21_42121:b5",
        kind: GapKind::DefinitionDrift,
        canonical_subsection: "(b)(5)",
        note: "Hand-coded \"De Novo Review in District Court\" references b2b for burden-of-proof governance. Canonical (b)(5) reads \"With respect to a complaint under paragraph (1)...\" and triggers on Secretary inaction within 210/90 days. The substance is correct but the hand-coded definition's burden-of-proof cross-reference is implicit in the canonical text via the general framework, not explicit in (b)(5) itself.",
        resolution_blocker: "PDF/HTML loader (M-future) — re-extract verbatim text and clarify which cross-references are explicit vs. doctrinally derived.",
    },
    // Bridge-audit-surfaced orphans (canonical subsections present in
    // the parsed text with no covering lock term anywhere in their
    // subtree). These are real granularity gaps in the hand-coded
    // structural data — present in the statute but not modeled in
    // praxis.lock.
    KnownGap {
        term_id: "<canonical:b3a>",
        kind: GapKind::UncoveredSubsection,
        canonical_subsection: "(b)(3)(A)",
        note: "Canonical (b)(3)(A) DEADLINE FOR ISSUANCE; SETTLEMENT AGREEMENTS is a sub-clause of (b)(3) but the lock data only has air21_42121:b3 umbrella — no b3a child capturing the 120-day-after-hearing deadline and the settlement-at-any-time-before-final-order rule.",
        resolution_blocker: "Structural-data refinement — add air21_42121:b3a to praxis.lock with the canonical (b)(3)(A) content, OR explicitly fold it into b3's definition. Decision pending Praxis-validation review.",
    },
    KnownGap {
        term_id: "<canonical:b4a>",
        kind: GapKind::UncoveredSubsection,
        canonical_subsection: "(b)(4)(A)",
        note: "Canonical (b)(4)(A) APPEAL TO COURT OF APPEALS is the actual content of subsection (b)(4); the lock data has air21_42121:b4 umbrella with conflated language (per the b4 DefinitionDrift gap above) but no b4a child capturing the circuit-court appeal procedure.",
        resolution_blocker: "Structural-data refinement coupled with b4 DefinitionDrift fix — add air21_42121:b4a (or rename b4 to match canonical heading) when PDF/HTML loader lands.",
    },
];

#[derive(Debug, Clone)]
pub struct Finding {
    pub term_id: String,
    pub canonical_subsection_inferred: Option<String>,
    pub canonical_marker_present: bool,
    pub classification: FindingClassification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingClassification {
    UndocumentedDirect,
    DocumentedParaphrase,
    DocumentedGap,
    UndocumentedNoCanonical,
}

/// Parse an AIR21 CURIE local part to its canonical subsection path.
/// Convention: subdivisions below the letter-digit-letter triplet
/// appear after an underscore (e.g., `b2b_i` → `(b)(2)(B)(i)`).
pub fn parse_curie_subsection_path(curie_local: &str) -> Vec<String> {
    use alloc::string::ToString;

    let mut path = Vec::new();
    let parts: Vec<&str> = curie_local.split('_').collect();

    // First part: letter-digit-letter triplet pattern.
    if let Some(first) = parts.first() {
        let chars: Vec<char> = first.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c.is_ascii_alphabetic() {
                // Letter — single character is one subdivision label.
                // Lowercase = top/intermediate level; uppercase = subparagraph.
                path.push(c.to_string());
                i += 1;
            } else if c.is_ascii_digit() {
                // Digit run — collect consecutive digits.
                let mut num = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    num.push(chars[i]);
                    i += 1;
                }
                path.push(num);
            } else {
                i += 1;
            }
        }
    }

    // Remaining parts: each underscore-separated segment is one
    // subdivision label. Roman numerals (i, ii, iii, iv, v, ...) are
    // preserved as-is; digits are pushed as numeric subdivision.
    for part in parts.iter().skip(1) {
        if part.is_empty() {
            continue;
        }
        // Treat the whole segment as a single subdivision label.
        path.push(part.to_string());
    }

    // Convention: in AIR21 CURIEs, the second char of the triplet
    // (after a lowercase letter) is uppercase if it represents a
    // subparagraph (A/B/C/...). Our raw-char split gave us lowercase
    // for "b" but kept "B" as-is from "b2b" — we need to uppercase
    // the third element if it's a single alphabetic char.
    // Actually re-examine: from "b2b" chars are [b, 2, b] → all lowercase.
    // We need to UPPER-case the trailing alphabetic if it appears AFTER
    // a digit, since by convention that's a subparagraph label.
    for i in 0..path.len() {
        let elem = &path[i];
        if elem.len() == 1
            && elem.chars().next().unwrap().is_ascii_alphabetic()
            && elem.chars().next().unwrap().is_lowercase()
            && i > 0
            && path[i - 1].chars().all(|c| c.is_ascii_digit())
        {
            path[i] = elem.to_uppercase();
        }
    }

    path
}

pub fn render_subsection_marker(path: &[String]) -> String {
    use core::fmt::Write;
    let mut out = String::new();
    for component in path {
        write!(&mut out, "({})", component).expect("write to String never fails");
    }
    out
}

pub fn canonical_contains_marker(marker: &str) -> bool {
    if CANONICAL_TEXT.contains(marker) {
        return true;
    }

    // Allow whitespace between marker components.
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

/// Parse the canonical text and produce a [`BridgeReport`] comparing
/// every `praxis.lock` term to the parser's view of the canonical
/// text. Same shape as SOX's `bridge_audit()`.
pub fn bridge_audit() -> crate::social::judicial::statute_structure::bridge::BridgeReport {
    use crate::social::judicial::citation::ontology::PinpointCitationConcept;
    use crate::social::judicial::statute_structure::bridge::audit_lock_against_tree;
    use crate::social::judicial::statute_structure::parse_statute_text;

    let root = crate::social::judicial::citation::PinpointCite::new()
        .push(PinpointCitationConcept::Title, "49")
        .push(PinpointCitationConcept::Section, "42121");
    let tree = parse_statute_text(CANONICAL_TEXT, root, "praxis-lock://air21_42121@2010")
        .expect("AIR21 canonical text must parse");

    let registry =
        crate::applied::data_provisioning::registry::structural_for("air21_42121", "2010")
            .expect("praxis.lock has air21_42121@2010 structural block");

    audit_lock_against_tree(registry, &tree, |local| {
        Some(parse_curie_subsection_path(local))
    })
}

pub fn audit() -> Vec<Finding> {
    use alloc::string::ToString;

    let mut findings = Vec::new();
    for term in statute().terms() {
        let curie = term.id.value.as_str();
        let Some(local) = curie.strip_prefix("air21_42121:") else {
            continue;
        };
        let path = parse_curie_subsection_path(local);
        let marker = render_subsection_marker(&path);
        let marker_present = canonical_contains_marker(&marker);

        let classification = if KNOWN_PARAPHRASES.iter().any(|p| p.term_id == curie) {
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
        assert_eq!(hex, CANONICAL_SHA256);
    }

    #[test]
    fn canonical_text_starts_with_49_usc_42121() {
        assert!(CANONICAL_TEXT.starts_with("49 U.S.C. § 42121"));
    }

    #[test]
    fn canonical_text_has_top_level_subsections() {
        for marker in &["(a)", "(b)"] {
            assert!(canonical_contains_marker(marker));
        }
    }

    #[test]
    fn canonical_text_has_four_clause_framework_markers() {
        // (b)(2)(B)(i) through (iv) — the heart of the burden-shifting framework
        for marker in &[
            "(b)(2)(B)(i)",
            "(b)(2)(B)(ii)",
            "(b)(2)(B)(iii)",
            "(b)(2)(B)(iv)",
        ] {
            assert!(
                canonical_contains_marker(marker),
                "burden-framework marker {} missing from canonical text",
                marker
            );
        }
    }

    #[test]
    fn parse_curie_letter_only() {
        assert_eq!(parse_curie_subsection_path("a"), vec!["a"]);
    }

    #[test]
    fn parse_curie_letter_underscore_digit() {
        assert_eq!(parse_curie_subsection_path("a_1"), vec!["a", "1"]);
        assert_eq!(parse_curie_subsection_path("a_4"), vec!["a", "4"]);
    }

    #[test]
    fn parse_curie_letter_digit_compact() {
        assert_eq!(parse_curie_subsection_path("b1"), vec!["b", "1"]);
        assert_eq!(parse_curie_subsection_path("b6"), vec!["b", "6"]);
    }

    #[test]
    fn parse_curie_letter_digit_letter_uppercases_subparagraph() {
        // "b2b" should parse to ["b", "2", "B"] — third char gets
        // uppercased because it follows a digit (subparagraph convention).
        assert_eq!(parse_curie_subsection_path("b2b"), vec!["b", "2", "B"]);
        assert_eq!(parse_curie_subsection_path("b2a"), vec!["b", "2", "A"]);
    }

    #[test]
    fn parse_curie_roman_numeral_suffix() {
        assert_eq!(
            parse_curie_subsection_path("b2b_i"),
            vec!["b", "2", "B", "i"]
        );
        assert_eq!(
            parse_curie_subsection_path("b2b_iv"),
            vec!["b", "2", "B", "iv"]
        );
    }

    #[test]
    fn render_marker_four_levels() {
        assert_eq!(
            render_subsection_marker(&[
                "b".to_string(),
                "2".to_string(),
                "B".to_string(),
                "i".to_string()
            ]),
            "(b)(2)(B)(i)"
        );
    }

    #[test]
    fn audit_produces_one_finding_per_term() {
        let findings = audit();
        assert_eq!(findings.len(), statute().terms().len());
    }

    #[test]
    fn known_paraphrases_classified_as_paraphrase() {
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
    fn known_gaps_classified_as_gap() {
        let findings = audit();
        for gap in KNOWN_GAPS {
            // UncoveredSubsection gaps don't correspond to a lock
            // term — they describe a canonical subsection with no
            // covering lock term. Skip those in this finding-level
            // check; their integrity is covered by
            // bridge_uncovered_clauses_are_acknowledged.
            if gap.kind == GapKind::UncoveredSubsection {
                continue;
            }
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
        let term_curies: alloc::collections::BTreeSet<String> = statute()
            .terms()
            .iter()
            .map(|t| t.id.value.clone())
            .collect();
        for gap in KNOWN_GAPS {
            // UncoveredSubsection gaps use the `<canonical:...>`
            // sentinel for `term_id` because by definition no lock
            // term exists for that subsection. Skip those.
            if gap.kind == GapKind::UncoveredSubsection {
                continue;
            }
            assert!(
                term_curies.contains(gap.term_id),
                "KNOWN_GAPS references {} which doesn't exist",
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
                "KNOWN_PARAPHRASES references {}",
                para.term_id
            );
        }
    }

    #[test]
    fn every_gap_has_resolution_blocker() {
        for gap in KNOWN_GAPS {
            assert!(!gap.resolution_blocker.is_empty(), "{}", gap.term_id);
            assert!(!gap.note.is_empty(), "{}", gap.term_id);
        }
    }

    // ── Bridge audit ────────────────────────────────────────────────

    #[test]
    fn bridge_audit_parses_canonical_text() {
        use crate::social::judicial::statute_structure::bridge::TermMatchResult;
        let report = bridge_audit();
        assert_eq!(report.by_lock_term.len(), 17);
        for r in &report.by_lock_term {
            assert!(
                matches!(r, TermMatchResult::Matched { .. }),
                "lock term unmatched: {r:?}"
            );
        }
    }

    #[test]
    fn bridge_paraphrases_align_with_known_paraphrases() {
        use crate::social::judicial::statute_structure::bridge::{TermMatchResult, TextMatch};
        let report = bridge_audit();
        let known_para_ids: alloc::collections::BTreeSet<&'static str> =
            KNOWN_PARAPHRASES.iter().map(|p| p.term_id).collect();
        let known_gap_ids: alloc::collections::BTreeSet<&'static str> =
            KNOWN_GAPS.iter().map(|g| g.term_id).collect();

        let mut unclassified: alloc::vec::Vec<&str> = alloc::vec::Vec::new();
        for r in &report.by_lock_term {
            if let TermMatchResult::Matched {
                lock_term_id,
                text_match: TextMatch::Paraphrase,
                ..
            } = r
            {
                let is_classified = known_para_ids.contains(lock_term_id.as_str())
                    || known_gap_ids.contains(lock_term_id.as_str());
                if !is_classified {
                    unclassified.push(lock_term_id.as_str());
                }
            }
        }
        assert!(
            unclassified.is_empty(),
            "{} unclassified paraphrase(s) — add to KNOWN_PARAPHRASES or KNOWN_GAPS: {:?}",
            unclassified.len(),
            unclassified
        );
    }

    #[test]
    fn bridge_uncovered_clauses_are_acknowledged() {
        let report = bridge_audit();
        let orphans: alloc::vec::Vec<String> = report
            .uncovered_orphan_clauses()
            .iter()
            .map(|c| c.to_bluebook())
            .collect();
        // Acknowledged orphans live in KNOWN_GAPS with
        // GapKind::UncoveredSubsection. Each orphan's bluebook
        // representation must end with the gap's
        // canonical_subsection string (e.g. "(49)(42121)(b)(3)(A)"
        // ends with "(b)(3)(A)").
        let acknowledged: alloc::collections::BTreeSet<&'static str> = KNOWN_GAPS
            .iter()
            .filter(|g| g.kind == GapKind::UncoveredSubsection)
            .map(|g| g.canonical_subsection)
            .collect();
        let mut unacknowledged: alloc::vec::Vec<&str> = alloc::vec::Vec::new();
        for o in &orphans {
            let is_known = acknowledged.iter().any(|sub| o.ends_with(sub));
            if !is_known {
                unacknowledged.push(o.as_str());
            }
        }
        assert!(
            unacknowledged.is_empty(),
            "found {} unacknowledged orphan canonical subsections: {:?}\nadd entries to KNOWN_GAPS with GapKind::UncoveredSubsection",
            unacknowledged.len(),
            unacknowledged
        );
    }

    #[test]
    fn bridge_heading_relations_classified() {
        use crate::social::judicial::statute_structure::bridge::{
            HeadingRelation, TermMatchResult, classify_heading_vs_name,
        };

        let report = bridge_audit();
        let known_para_ids: alloc::collections::BTreeSet<&'static str> =
            KNOWN_PARAPHRASES.iter().map(|p| p.term_id).collect();
        let known_gap_ids: alloc::collections::BTreeSet<&'static str> =
            KNOWN_GAPS.iter().map(|g| g.term_id).collect();
        let lock_name_by_id: alloc::collections::BTreeMap<String, String> = statute()
            .terms()
            .iter()
            .map(|t| (t.id.value.clone(), t.name.text.clone()))
            .collect();

        let mut undocumented_divergences: alloc::vec::Vec<(String, String, String)> =
            alloc::vec::Vec::new();
        for r in &report.by_lock_term {
            if let TermMatchResult::Matched {
                lock_term_id,
                canonical_heading,
                ..
            } = r
            {
                let lock_name = lock_name_by_id.get(lock_term_id).expect("lock name");
                let relation = classify_heading_vs_name(lock_name, canonical_heading.as_deref());
                if relation == HeadingRelation::HeadingDiverges {
                    let classified = known_para_ids.contains(lock_term_id.as_str())
                        || known_gap_ids.contains(lock_term_id.as_str());
                    if !classified {
                        undocumented_divergences.push((
                            lock_term_id.clone(),
                            lock_name.clone(),
                            canonical_heading.clone().unwrap_or_default(),
                        ));
                    }
                }
            }
        }
        assert!(
            undocumented_divergences.is_empty(),
            "{} undocumented heading-vs-name divergence(s) — add to KNOWN_PARAPHRASES or KNOWN_GAPS:\n{}",
            undocumented_divergences.len(),
            undocumented_divergences
                .iter()
                .map(|(id, name, h)| alloc::format!(
                    "  - {} lock-name=\"{}\" canonical-heading=\"{}\"",
                    id,
                    name,
                    h
                ))
                .collect::<alloc::vec::Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn print_bridge_report() {
        use crate::social::judicial::statute_structure::bridge::{TermMatchResult, TextMatch};
        let report = bridge_audit();
        eprintln!("\n=== AIR21 § 42121 bridge audit report ===");
        eprintln!("Lock terms: {}", report.by_lock_term.len());
        eprintln!("  matched:   {}", report.matched_term_count());
        eprintln!("  unmatched: {}", report.unmatched_term_count());
        eprintln!("Parsed clauses: {}", report.by_clause.len());
        eprintln!("  covered:   {}", report.covered_clause_count());
        eprintln!("  uncovered: {}", report.uncovered_clause_count());

        let mut name_in_body = 0;
        let mut paraphrase = 0;
        for r in &report.by_lock_term {
            if let TermMatchResult::Matched { text_match, .. } = r {
                match text_match {
                    TextMatch::NameInBody => name_in_body += 1,
                    TextMatch::Paraphrase => paraphrase += 1,
                }
            }
        }
        eprintln!(
            "  text-match breakdown: {name_in_body} verbatim-in-body, {paraphrase} paraphrase"
        );
        eprintln!();
    }

    #[test]
    fn print_gap_report() {
        let findings = audit();
        eprintln!("\n=== AIR21 § 42121 canonical-text audit report ===");
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
