//! Relation extractor — detects typed relation phrases in clause
//! body text and produces `RelationCandidate`s suitable for
//! comparison against hand-coded `praxis.lock` relations.
//!
//! Complements [`term_extractor`] which surfaces *what* each clause
//! is. This module surfaces *how clauses relate*: cross-references,
//! affirmative defenses, alternatives, exclusions.
//!
//! [`term_extractor`]: super::term_extractor
//!
//! # Patterns recognized
//!
//! Each pattern is a phrase in the clause body whose match indicates
//! a specific [`RelationKind`]:
//!
//! | Phrase | Relation kind | Source |
//! |---|---|---|
//! | "shall be governed by" / "shall be governed under" | `Requires` | Dickerson (1975) §6.4 (incorporation by reference) |
//! | "Notwithstanding" | `AffirmativeDefenseTo` | Garner (2016) §11 (qualifying clauses) |
//! | "Except as" / "except that" | `Excludes` | Garner (2016) §11 |
//! | "subject to" | `Requires` | Sartor (2005) §21 (conditional norms) |
//!
//! The detector is intentionally conservative: it matches whole-word
//! phrases case-insensitively at word boundaries and emits one
//! `RelationCandidate` per match, capturing the *trailing* text as
//! `target_text` for downstream resolution of which subsection /
//! statute / paragraph is being referenced.
//!
//! # Praxis-way scope
//!
//! Pattern detection is NLP-light: substring + word-boundary check.
//! Real legal-NLP literature (Wyner 2008, Sartor 2005 Ch. 22) uses
//! richer parsing; this module deliberately stays at the regex-class
//! pattern level so it composes with the existing `parse_statute_text`
//! → `extract_terms` pipeline without external dependencies. Future
//! refinement targets are documented in module-level TODOs only if
//! they correspond to specific Praxis gaps.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use crate::social::judicial::citation::PinpointCite;
use crate::social::judicial::statute_structure::parser::{ClauseNode, ClauseTree};

// ─────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────

/// The kind of relation surfaced by a phrase match. Mirrors a subset
/// of `RelationType` in `social::judicial::ontology` — the variants
/// that statutory phrase patterns can confidently identify.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationKind {
    /// Cross-statute or cross-section incorporation by reference.
    /// Phrases: "shall be governed by/under", "subject to".
    Requires,
    /// Source clause defeats a target rule when an exception applies.
    /// Phrases: "Notwithstanding".
    AffirmativeDefenseTo,
    /// Source clause carves out an exclusion from a target rule.
    /// Phrases: "Except as", "except that".
    Excludes,
}

/// One detected relation candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationCandidate {
    /// The clause whose body contained the relation phrase.
    pub from_cite: PinpointCite,
    /// Which phrase pattern matched.
    pub kind: RelationKind,
    /// The exact text snippet that matched (verbatim from body).
    pub phrase: String,
    /// Byte offset of the match within the clause body.
    pub offset_in_body: usize,
    /// Trailing context: the text immediately after the matched
    /// phrase, up to the next sentence-ending punctuation or 200
    /// chars (whichever is shorter). Useful for downstream
    /// cross-reference resolution.
    pub target_text: String,
}

// ─────────────────────────────────────────────────────────────────────
// Phrase patterns
// ─────────────────────────────────────────────────────────────────────

/// Phrase patterns we recognize. Each entry is `(phrase, kind)`.
/// Matching is case-insensitive at word boundaries.
const PATTERNS: &[(&str, RelationKind)] = &[
    ("shall be governed by", RelationKind::Requires),
    ("shall be governed under", RelationKind::Requires),
    ("subject to", RelationKind::Requires),
    ("Notwithstanding", RelationKind::AffirmativeDefenseTo),
    ("Except as", RelationKind::Excludes),
    ("except that", RelationKind::Excludes),
];

// ─────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────

/// Walk the tree and extract relation candidates from every
/// non-root node's body text.
pub fn extract_relations(tree: &ClauseTree) -> Vec<RelationCandidate> {
    let mut out = Vec::new();
    extract_from_node(&tree.root, /* is_root */ true, &mut out);
    out
}

fn extract_from_node(node: &ClauseNode, is_root: bool, out: &mut Vec<RelationCandidate>) {
    if !is_root {
        detect_in_body(&node.id, &node.text.text, out);
    }
    for child in &node.children {
        extract_from_node(child, /* is_root */ false, out);
    }
}

/// Apply every pattern to `body` and emit candidates.
fn detect_in_body(cite: &PinpointCite, body: &str, out: &mut Vec<RelationCandidate>) {
    let body_lower = body.to_lowercase();
    for (phrase, kind) in PATTERNS {
        let phrase_lower = phrase.to_lowercase();
        let mut search_start = 0;
        while let Some(rel_pos) = body_lower[search_start..].find(&phrase_lower) {
            let abs_pos = search_start + rel_pos;
            // Word-boundary check on both sides.
            if !at_word_boundary(&body_lower, abs_pos, phrase.len()) {
                search_start = abs_pos + phrase.len();
                continue;
            }
            // Extract verbatim phrase from the original (case-preserving) body.
            let matched_phrase = body[abs_pos..abs_pos + phrase.len()].to_string();
            let target_start = abs_pos + phrase.len();
            let target_text = extract_target(&body[target_start..]);
            out.push(RelationCandidate {
                from_cite: cite.clone(),
                kind: *kind,
                phrase: matched_phrase,
                offset_in_body: abs_pos,
                target_text,
            });
            search_start = abs_pos + phrase.len();
        }
    }
}

/// True if the substring `body[pos..pos+len]` is bordered by a word
/// boundary on both sides (start-of-text / end-of-text / non-word
/// char).
fn at_word_boundary(body: &str, pos: usize, len: usize) -> bool {
    let before_ok = if pos == 0 {
        true
    } else {
        body.as_bytes()
            .get(pos - 1)
            .map(|b| !is_word_byte(*b))
            .unwrap_or(true)
    };
    let after_idx = pos + len;
    let after_ok = if after_idx >= body.len() {
        true
    } else {
        body.as_bytes()
            .get(after_idx)
            .map(|b| !is_word_byte(*b))
            .unwrap_or(true)
    };
    before_ok && after_ok
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Extract the trailing context after a matched phrase. Returns the
/// text up to the next sentence-terminating period (`.` followed by
/// whitespace or end-of-text), comma, semicolon, or 200 chars —
/// whichever comes first. Leading whitespace is trimmed.
fn extract_target(rest: &str) -> String {
    let trimmed = rest.trim_start();
    let max = 200.min(trimmed.len());
    let slice = &trimmed[..max];
    // Find a clause-ending terminator.
    let mut end = slice.len();
    for (i, c) in slice.char_indices() {
        if c == '.' || c == ';' {
            // Sentence-ish boundary — stop here.
            end = i;
            break;
        }
    }
    slice[..end].trim().to_string()
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::judicial::citation::ontology::PinpointCitationConcept;
    use crate::social::judicial::statute_structure::parse_statute_text;

    fn root_cite() -> PinpointCite {
        PinpointCite::new()
            .push(PinpointCitationConcept::Title, "TEST")
            .push(PinpointCitationConcept::Section, "1")
    }

    // ── Unit: word-boundary + target extraction ──────────────────────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn at_word_boundary_handles_string_start() {
        assert!(at_word_boundary("hello world", 0, 5));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn at_word_boundary_handles_string_end() {
        assert!(at_word_boundary("hello world", 6, 5));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn at_word_boundary_rejects_mid_word() {
        // "ell" inside "hello" — not a boundary on either side.
        assert!(!at_word_boundary("hello", 1, 3));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extract_target_stops_at_period() {
        assert_eq!(
            extract_target(" section 42121. More text."),
            "section 42121"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extract_target_stops_at_semicolon() {
        assert_eq!(extract_target(" foo bar; more"), "foo bar");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extract_target_truncates_long_text() {
        let long = "x".repeat(500);
        let result = extract_target(&long);
        assert_eq!(result.len(), 200);
    }

    // ── Unit: detect_in_body produces candidates ─────────────────────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn detects_shall_be_governed_by() {
        let mut out = Vec::new();
        detect_in_body(
            &root_cite(),
            "An action shall be governed by section 42121(b) of title 49.",
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, RelationKind::Requires);
        assert_eq!(out[0].phrase, "shall be governed by");
        assert_eq!(out[0].target_text, "section 42121(b) of title 49");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn detects_shall_be_governed_under() {
        let mut out = Vec::new();
        detect_in_body(
            &root_cite(),
            "An action under paragraph (1) shall be governed under the rules in section 42121(b).",
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, RelationKind::Requires);
        assert_eq!(out[0].phrase, "shall be governed under");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn detects_notwithstanding() {
        let mut out = Vec::new();
        detect_in_body(
            &root_cite(),
            "Notwithstanding a finding by the Secretary, no investigation shall proceed.",
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, RelationKind::AffirmativeDefenseTo);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn detects_except_as() {
        let mut out = Vec::new();
        detect_in_body(
            &root_cite(),
            "Except as otherwise provided, this section applies.",
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, RelationKind::Excludes);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn detects_multiple_phrases_in_one_body() {
        let mut out = Vec::new();
        detect_in_body(
            &root_cite(),
            "An action shall be governed by section 42121(b). Notwithstanding any finding, no investigation occurs.",
            &mut out,
        );
        assert_eq!(out.len(), 2);
        let kinds: alloc::vec::Vec<RelationKind> = out.iter().map(|c| c.kind).collect();
        assert!(kinds.contains(&RelationKind::Requires));
        assert!(kinds.contains(&RelationKind::AffirmativeDefenseTo));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn no_match_when_phrase_not_present() {
        let mut out = Vec::new();
        detect_in_body(
            &root_cite(),
            "Just some prose without trigger phrases.",
            &mut out,
        );
        assert!(out.is_empty());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn case_insensitive_match() {
        let mut out = Vec::new();
        detect_in_body(
            &root_cite(),
            "An action SHALL BE GOVERNED BY section 42121(b).",
            &mut out,
        );
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, RelationKind::Requires);
        assert_eq!(out[0].phrase, "SHALL BE GOVERNED BY");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn word_boundary_rejects_embedded_substring() {
        let mut out = Vec::new();
        // "subject toxic" has "subject to" as a prefix but the
        // following byte is alphabetic, so the boundary check fails.
        // Result: no match.
        detect_in_body(&root_cite(), "Make this subject toxic.", &mut out);
        assert!(out.is_empty());
    }

    // ── extract_relations tree-walking tests ─────────────────────────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extract_skips_root() {
        let tree = parse_statute_text(
            "Prefix prose with shall be governed by clause.\n(a) FOO.--body without trigger",
            root_cite(),
            "test://",
        )
        .unwrap();
        let rels = extract_relations(&tree);
        // Root has the trigger but is skipped; (a) doesn't. Empty.
        assert!(rels.is_empty(), "got: {rels:?}");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extract_walks_nested_children() {
        let text = "(a) outer body\n(1) inner body shall be governed by section 42121(b).";
        let tree = parse_statute_text(text, root_cite(), "test://").unwrap();
        let rels = extract_relations(&tree);
        assert_eq!(rels.len(), 1);
        // The relation should be attributed to (a)(1).
        assert_eq!(rels[0].from_cite.segments.last().unwrap().label, "1");
    }

    // ── Real-corpus tests ────────────────────────────────────────────

    const SOX_CANONICAL: &str =
        include_str!("../../../../data/test_fixtures/statute_shape/sox_1514a_shape.txt");
    const AIR21_CANONICAL: &str =
        include_str!("../../../../data/test_fixtures/statute_shape/air21_42121_shape.txt");

    fn sox_root() -> PinpointCite {
        PinpointCite::new()
            .push(PinpointCitationConcept::Title, "18")
            .push(PinpointCitationConcept::Section, "1514A")
    }

    fn air21_root() -> PinpointCite {
        PinpointCite::new()
            .push(PinpointCitationConcept::Title, "49")
            .push(PinpointCitationConcept::Section, "42121")
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extract_sox_finds_governed_by_cross_references() {
        let tree =
            parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
        let rels = extract_relations(&tree);
        // SOX § 1514A(b)(2)(A) "shall be governed under" + (b)(2)(C)
        // "shall be governed by" — at least two Requires matches.
        let governed: alloc::vec::Vec<_> = rels
            .iter()
            .filter(|r| {
                r.kind == RelationKind::Requires && r.phrase.to_lowercase().contains("governed")
            })
            .collect();
        assert!(
            governed.len() >= 2,
            "expected ≥2 'shall be governed' matches in SOX; got {} ({:?})",
            governed.len(),
            governed
                .iter()
                .map(|r| &r.phrase)
                .collect::<alloc::vec::Vec<_>>()
        );
        // Each match's target should reference section 42121.
        for r in &governed {
            assert!(
                r.target_text.contains("42121"),
                "expected 42121 in target: {}",
                r.target_text
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extract_sox_governed_by_attributed_to_b2a_and_b2c() {
        let tree =
            parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
        let rels = extract_relations(&tree);
        let governed_cites: alloc::collections::BTreeSet<String> = rels
            .iter()
            .filter(|r| r.phrase.to_lowercase().contains("governed"))
            .map(|r| r.from_cite.to_bluebook())
            .collect();
        // (b)(2)(A) and (b)(2)(C) should both be source clauses.
        assert!(
            governed_cites.iter().any(|c| c.ends_with("(b)(2)(A)")),
            "expected (b)(2)(A) in governed cites: {:?}",
            governed_cites
        );
        assert!(
            governed_cites.iter().any(|c| c.ends_with("(b)(2)(C)")),
            "expected (b)(2)(C) in governed cites: {:?}",
            governed_cites
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extract_air21_finds_notwithstanding_in_burden_clause_ii() {
        let tree = parse_statute_text(
            AIR21_CANONICAL,
            air21_root(),
            "praxis-lock://air21_42121@2010",
        )
        .unwrap();
        let rels = extract_relations(&tree);
        // AIR21 (b)(2)(B)(ii) "Notwithstanding a finding..." — the
        // canonical investigation-gate defense.
        let notwiths: alloc::vec::Vec<_> = rels
            .iter()
            .filter(|r| r.kind == RelationKind::AffirmativeDefenseTo)
            .collect();
        assert!(
            !notwiths.is_empty(),
            "expected at least one Notwithstanding match in AIR21"
        );
        // (b)(2)(B)(ii) should be among them.
        let has_b2bii = notwiths
            .iter()
            .any(|r| r.from_cite.to_bluebook().ends_with("(b)(2)(B)(ii)"));
        assert!(
            has_b2bii,
            "expected (b)(2)(B)(ii) to contain Notwithstanding; got cites: {:?}",
            notwiths
                .iter()
                .map(|r| r.from_cite.to_bluebook())
                .collect::<alloc::vec::Vec<_>>()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_extracted_relation_from_cite_is_findable() {
        // Property: every from_cite resolves to a node in the tree.
        let tree =
            parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
        let rels = extract_relations(&tree);
        for r in &rels {
            assert!(
                tree.find(&r.from_cite).is_some(),
                "from_cite {} not in tree",
                r.from_cite.to_bluebook()
            );
        }
    }

    #[pr4xis::praxis_value(Explainable)]
    #[test]
    fn print_relation_summary() {
        let sox =
            parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
        let air21 = parse_statute_text(
            AIR21_CANONICAL,
            air21_root(),
            "praxis-lock://air21_42121@2010",
        )
        .unwrap();
        let sox_rels = extract_relations(&sox);
        let air21_rels = extract_relations(&air21);

        eprintln!("\n=== Relation extraction summary ===");
        eprintln!("SOX § 1514A: {} candidates", sox_rels.len());
        for r in &sox_rels {
            eprintln!(
                "  {} {:?} \"{}\" → {}",
                r.from_cite.to_bluebook(),
                r.kind,
                r.phrase,
                if r.target_text.is_empty() {
                    "(no target)"
                } else {
                    &r.target_text
                }
            );
        }
        eprintln!("\nAIR21 § 42121: {} candidates", air21_rels.len());
        for r in &air21_rels {
            eprintln!(
                "  {} {:?} \"{}\" → {}",
                r.from_cite.to_bluebook(),
                r.kind,
                r.phrase,
                if r.target_text.is_empty() {
                    "(no target)"
                } else {
                    &r.target_text
                }
            );
        }
        eprintln!();
    }
}
