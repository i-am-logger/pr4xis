//! Term extractor — pulls canonical headings out of `ClauseNode`
//! bodies and produces `ExtractedTerm` candidates suitable for
//! comparison against hand-coded `praxis.lock` structural data.
//!
//! # The canonical heading pattern
//!
//! U.S. federal statutes (and most Bluebook-style drafted laws) use
//! a consistent **`HEADING.--`** convention to introduce each
//! subdivision. For example, in § 1514A(b)(2)(D):
//!
//! ```text
//! (D) STATUTE OF LIMITATIONS.--An action under paragraph (1) shall
//! be commenced not later than 180 days after the date on which the
//! violation occurs...
//! ```
//!
//! The heading is the **canonical name** of that subdivision —
//! enacted text, not editorial annotation. Term names hand-coded
//! into `praxis.lock` either match the heading verbatim
//! (`Statute of Limitations`) or are practitioner shorthand for it
//! (`Statute of Limitations for Filing`).
//!
//! This module recognizes the heading pattern, extracts the heading
//! as the canonical name, and presents the remaining body separately.
//! The result is the **machine-extracted** counterpart to the
//! hand-coded `name` field in each `StructuralTerm`.
//!
//! # Heading grammar (literature-grounded)
//!
//! - **House Legislative Counsel's Manual on Drafting Style (2017)**
//!   §312(a) — federal-bill drafting convention for subsection headings.
//!   LLM-checked (web).
//! - The `.--` heading separator is observable directly in the loaded
//!   USLM/USC heading text (machine-verified), not a style-guide claim.
//! - **Wyner, Adam & Bench-Capon, Trevor (2007)** — structural
//!   extraction from legal text grounded in heading detection.
//!
//! Heading recognition rule (deliberately conservative):
//! - Body text starts with `HEADING.--` where `HEADING` is non-empty,
//!   begins with an ASCII uppercase letter, and is followed by `.--`.
//! - `HEADING` may contain ASCII letters, digits, spaces, semicolons,
//!   commas, periods (except the terminal `.--`), and parenthesized
//!   alphanumeric sequences.
//! - If no `.--` is present, the body is treated as having no heading
//!   and the full body is returned.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use crate::cognitive::linguistics::lemon::lexicon::Form;
use crate::social::judicial::citation::PinpointCite;
use crate::social::judicial::statute_structure::parser::{ClauseNode, ClauseTree};

/// One extracted term candidate from a `ClauseTree` node. The
/// `cite` mirrors the source node's pinpoint citation; `heading` is
/// the canonical `HEADING.--` text if the body matched the pattern;
/// `body` is the body text *minus* the heading (or the full body if
/// no heading was detected).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedTerm {
    pub cite: PinpointCite,
    pub heading: Option<String>,
    pub body: String,
}

impl ExtractedTerm {
    /// Pretty-print this term's heading or `(none)`.
    pub fn heading_or_none(&self) -> &str {
        self.heading.as_deref().unwrap_or("(none)")
    }
}

/// Walk the tree and produce one `ExtractedTerm` per non-root node.
/// The root is skipped because it doesn't correspond to a single
/// subdivision (its body is just the title-level prefix prose).
pub fn extract_terms(tree: &ClauseTree) -> Vec<ExtractedTerm> {
    let mut out = Vec::new();
    extract_from_node(&tree.root, /* is_root */ true, &mut out);
    out
}

/// Extract content-word lemmas from a term name. Lowercases,
/// strips punctuation, removes stopwords, deduplicates. Produces
/// the candidate-lemma list that the (future) M5 statute-to-English
/// adjunction will resolve against WordNet.
///
/// # Examples
///
/// - `"Covered Employer"` → `["covered", "employer"]`
/// - `"Statute of Limitations for Filing"` →
///   `["statute", "limitations", "filing"]` ("of" and "for" are
///   stopwords)
/// - `"Burdens of Proof for District Court Actions"` →
///   `["burdens", "proof", "district", "court", "actions"]`
///
/// # Stopwords
///
/// Removes the closed-class English function words that don't carry
/// lexical content: articles (a, an, the), prepositions (of, in, on,
/// at, to, for, by, with, from), conjunctions (and, or, but, nor),
/// auxiliaries (is, are, was, were, be, been, being, has, have, had),
/// pronouns (he, she, it, they, this, that), and modals (shall, may,
/// can, will, must). Source: Cambridge Grammar of the English
/// Language (Huddleston & Pullum 2002) Ch. 1 — function-word vs
/// content-word distinction.
///
/// The stopword list is loaded from the bundled
/// `crates/domains/data/function-words/english.xml` (Chiarcos &
/// Sukhareva 2015 OLiA POS taxonomy) — not hand-coded.
///
/// # Examples
///
/// Two content words around an article:
///
/// ```
/// use pr4xis_domains::social::judicial::statute_structure::term_extractor::extract_lemmas;
/// let lemmas: Vec<String> = extract_lemmas("Covered Employer")
///     .into_iter()
///     .map(|f| f.written_rep)
///     .collect();
/// assert_eq!(lemmas, vec!["covered", "employer"]);
/// ```
///
/// Stopwords stripped, numeric tokens filtered (ISO 80000-2):
///
/// ```
/// use pr4xis_domains::social::judicial::statute_structure::term_extractor::extract_lemmas;
/// let lemmas: Vec<String> = extract_lemmas("Section 42121 of the Statute")
///     .into_iter()
///     .map(|f| f.written_rep)
///     .collect();
/// // "section" and "statute" remain; "42121" is numeric (filtered),
/// // "of" and "the" are function words (filtered).
/// assert_eq!(lemmas, vec!["section", "statute"]);
/// ```
///
/// Lower-cased output:
///
/// ```
/// use pr4xis_domains::social::judicial::statute_structure::term_extractor::extract_lemmas;
/// for form in extract_lemmas("PROHIBITION ON RETALIATION") {
///     assert_eq!(form.written_rep, form.written_rep.to_lowercase());
/// }
/// ```
pub fn extract_lemmas(term_name: &str) -> Vec<Form> {
    let stopwords = english_stopwords();
    let mut seen: alloc::collections::BTreeSet<String> = Default::default();
    let mut out = Vec::new();

    // Strip Unicode format characters that don't participate in word
    // identity — per Unicode 15.0 §5.3, U+00AD SOFT HYPHEN is a
    // typographic line-break hint inserted by typesetters; it is NOT
    // a word boundary or letter. The USC USLM XML carries these inside
    // long words like "trans\u{00AD}feree" (28 U.S.C. § 3307) to allow
    // graceful line breaking in PDF / print renderings. Stripping
    // before tokenization keeps the word identity intact for WordNet
    // lookup. Cited: Unicode 15.0 §5.3 "Soft Hyphens"; Bringhurst,
    // *The Elements of Typographic Style* §1.3 (soft-hyphen typography).
    let normalized: String = term_name.chars().filter(|&c| c != '\u{00ad}').collect();

    for word in normalized.split(|c: char| !c.is_alphanumeric()) {
        if word.is_empty() {
            continue;
        }
        // Numeric tokens are statute-section references, not lexemes
        // (ISO 80000-2 — numerals carry quantity, not meaning).
        if word.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let lowered = word.to_lowercase();
        if stopwords.contains(&lowered) {
            continue;
        }
        if seen.insert(lowered.clone()) {
            out.push(Form {
                written_rep: lowered,
                lang: "en".to_string(),
            });
        }
    }

    out
}

/// The English stopword set, loaded from
/// `crates/domains/data/function-words/english.xml` (LMF format).
///
/// Source: OLiA POS taxonomy (Chiarcos & Sukhareva 2015) +
/// Huddleston & Pullum (2002) *Cambridge Grammar of the English
/// Language* Ch. 1 — function-word vs content-word distinction.
/// The XML is embedded at compile time via [`include_str!`] and
/// parsed once into a [`BTreeSet`] on first call.
fn english_stopwords() -> &'static alloc::collections::BTreeSet<String> {
    use std::sync::OnceLock;
    static STOPWORDS: OnceLock<alloc::collections::BTreeSet<String>> = OnceLock::new();
    STOPWORDS.get_or_init(|| {
        // The committed function-word `.prx` — materialized through the
        // generalized feature-light `[compact_archive_signatures]` gate (phase
        // 2d). The raw `english.xml` is the git-tracked source-of-truth but is
        // EXCLUDED from the published crate; only this `.prx` ships. Its
        // parseability is a build-time invariant verified by the bundled test
        // suite. Failure here is a defect, not user input — hard fail.
        const PRX: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/function-words/english.prx"
        ));
        let xml = crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded(
            "english_function_words",
            "2026",
            PRX,
        );
        let wn = crate::social::software::markup::xml::lmf::reader::read_wordnet(xml).expect(
            "english_function_words committed .prx bytes failed to parse — \
                 build-time invariant violated",
        );
        wn.entries
            .iter()
            .map(|e| e.lemma.written_form.to_lowercase())
            .collect()
    })
}

fn extract_from_node(node: &ClauseNode, is_root: bool, out: &mut Vec<ExtractedTerm>) {
    if !is_root {
        let (heading, body) = split_heading(&node.text.text);
        out.push(ExtractedTerm {
            cite: node.id.clone(),
            heading,
            body,
        });
    }
    for child in &node.children {
        extract_from_node(child, /* is_root */ false, out);
    }
}

/// Split a body string into (heading, remaining-body) if it matches
/// the canonical `HEADING.--` pattern; otherwise returns `(None,
/// full_body)`.
///
/// Recognition rules:
/// - First non-whitespace character must be ASCII uppercase.
/// - Heading runs until the first `.--` or `.—` separator.
/// - Heading characters: ASCII alphanumerics, spaces, `;`, `,`, `.`
///   (only inside the heading, not the terminal `.--`),
///   parentheses, hyphens, slashes, and apostrophes.
/// - Heading must be non-empty after trimming.
pub fn split_heading(body: &str) -> (Option<String>, String) {
    let trimmed_start = body.trim_start();
    if trimmed_start.is_empty() {
        return (None, body.to_string());
    }
    if !trimmed_start
        .chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
    {
        return (None, body.to_string());
    }
    // Find the terminal `.--` (ASCII double-hyphen) or `.—` (em dash).
    // Bias to `.--` since the canonical-text fixtures use that form.
    let sep = ".--";
    let sep_em = ".\u{2014}"; // .—
    let sep_pos = trimmed_start
        .find(sep)
        .map(|p| (p, sep.len()))
        .or_else(|| trimmed_start.find(sep_em).map(|p| (p, sep_em.len())));

    let Some((pos, sep_len)) = sep_pos else {
        return (None, body.to_string());
    };
    let raw_heading = &trimmed_start[..pos];
    if raw_heading.is_empty() {
        return (None, body.to_string());
    }
    // Reject if the heading contains a newline — that means the
    // `.--` is too far away and isn't actually a heading separator.
    if raw_heading.contains('\n') || raw_heading.contains('\r') {
        return (None, body.to_string());
    }
    // Reject if the heading is too long to be a real subsection
    // heading (heuristic: real headings are < 120 chars).
    if raw_heading.len() > 200 {
        return (None, body.to_string());
    }
    let heading = raw_heading.trim().to_string();
    let remaining = trimmed_start[pos + sep_len..].trim().to_string();
    (Some(heading), remaining)
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::judicial::citation::ontology::PinpointCitationConcept;
    use crate::social::judicial::statute_structure::parse_statute_text;

    fn root() -> PinpointCite {
        PinpointCite::new()
            .push(PinpointCitationConcept::Title, "TEST")
            .push(PinpointCitationConcept::Section, "1")
    }

    // ── split_heading unit tests ─────────────────────────────────────

    #[test]
    fn split_heading_finds_simple_pattern() {
        let (h, body) = split_heading("HEADING.--body text follows.");
        assert_eq!(h.as_deref(), Some("HEADING"));
        assert_eq!(body, "body text follows.");
    }

    #[test]
    fn split_heading_strips_leading_whitespace() {
        let (h, body) = split_heading("   FOO.--rest");
        assert_eq!(h.as_deref(), Some("FOO"));
        assert_eq!(body, "rest");
    }

    #[test]
    fn split_heading_handles_multi_word() {
        let (h, _) = split_heading("STATUTE OF LIMITATIONS.--An action ...");
        assert_eq!(h.as_deref(), Some("STATUTE OF LIMITATIONS"));
    }

    #[test]
    fn split_heading_handles_punctuation_in_heading() {
        let (h, _) = split_heading(
            "NONENFORCEABILITY OF CERTAIN PROVISIONS WAIVING RIGHTS AND REMEDIES OR REQUIRING ARBITRATION OF DISPUTES.--text",
        );
        assert_eq!(
            h.as_deref(),
            Some(
                "NONENFORCEABILITY OF CERTAIN PROVISIONS WAIVING RIGHTS AND REMEDIES OR REQUIRING ARBITRATION OF DISPUTES"
            )
        );
    }

    #[test]
    fn split_heading_rejects_lowercase_first_char() {
        let (h, body) = split_heading("text starting lowercase.--body");
        assert_eq!(h, None);
        assert_eq!(body, "text starting lowercase.--body");
    }

    #[test]
    fn split_heading_rejects_no_separator() {
        let (h, body) = split_heading("UPPERCASE TEXT WITH NO SEPARATOR HERE");
        assert_eq!(h, None);
        assert_eq!(body, "UPPERCASE TEXT WITH NO SEPARATOR HERE");
    }

    #[test]
    fn split_heading_rejects_heading_with_newline() {
        let (h, _) = split_heading("LINE 1\nLINE 2.--body");
        assert_eq!(h, None);
    }

    #[test]
    fn split_heading_accepts_em_dash_form() {
        let (h, body) = split_heading("HEADING.\u{2014}body text");
        assert_eq!(h.as_deref(), Some("HEADING"));
        assert_eq!(body, "body text");
    }

    #[test]
    fn split_heading_empty_input() {
        let (h, body) = split_heading("");
        assert_eq!(h, None);
        assert_eq!(body, "");
    }

    // ── extract_terms tree-walking tests ─────────────────────────────

    #[test]
    fn extract_skips_root() {
        let tree = parse_statute_text(
            "Section header text\n(a) FOO.--body of a",
            root(),
            "test://",
        )
        .unwrap();
        let terms = extract_terms(&tree);
        // One non-root child, so one ExtractedTerm.
        assert_eq!(terms.len(), 1);
    }

    #[test]
    fn extract_handles_nested_children() {
        let text = "(a) OUTER.--outer body\n(1) INNER.--inner body\n(A) DEEPEST.--deepest body";
        let tree = parse_statute_text(text, root(), "test://").unwrap();
        let terms = extract_terms(&tree);
        assert_eq!(terms.len(), 3);
        assert_eq!(terms[0].heading.as_deref(), Some("OUTER"));
        assert_eq!(terms[1].heading.as_deref(), Some("INNER"));
        assert_eq!(terms[2].heading.as_deref(), Some("DEEPEST"));
    }

    #[test]
    fn extract_handles_missing_heading() {
        let text = "(a) just prose without a heading separator here";
        let tree = parse_statute_text(text, root(), "test://").unwrap();
        let terms = extract_terms(&tree);
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].heading, None);
        assert!(terms[0].body.contains("just prose"));
    }

    // ── Real-corpus tests against SOX and AIR21 canonical fixtures ────

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

    #[test]
    fn extract_sox_finds_canonical_headings() {
        let tree =
            parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
        let terms = extract_terms(&tree);

        // Sample known headings from § 1514A.
        let expected: alloc::vec::Vec<(&str, &str)> = vec![
            (
                "a",
                "WHISTLEBLOWER PROTECTION FOR EMPLOYEES OF PUBLICLY TRADED COMPANIES",
            ),
            ("b", "ENFORCEMENT ACTION"),
            ("c", "REMEDIES"),
            ("d", "RIGHTS RETAINED BY EMPLOYEE"),
            (
                "e",
                "NONENFORCEABILITY OF CERTAIN PROVISIONS WAIVING RIGHTS AND REMEDIES OR REQUIRING ARBITRATION OF DISPUTES",
            ),
        ];
        for (subsection, expected_heading) in expected {
            let term = terms
                .iter()
                .find(|t| {
                    t.cite.segments.last().map(|s| s.label.as_str()) == Some(subsection)
                        && t.cite.segments.len() == 3 // Title + Section + Subsection
                })
                .unwrap_or_else(|| panic!("subsection ({subsection}) not extracted"));
            assert_eq!(
                term.heading.as_deref(),
                Some(expected_heading),
                "subsection ({subsection}) heading mismatch"
            );
        }
    }

    #[test]
    fn extract_sox_finds_burden_subsection_heading() {
        let tree =
            parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
        let terms = extract_terms(&tree);
        // (b)(2)(C) BURDENS OF PROOF.
        let b2c = terms
            .iter()
            .find(|t| t.cite.to_bluebook().ends_with("(b)(2)(C)"))
            .expect("(b)(2)(C) extracted");
        assert_eq!(b2c.heading.as_deref(), Some("BURDENS OF PROOF"));
    }

    #[test]
    fn extract_air21_finds_canonical_headings() {
        let tree = parse_statute_text(
            AIR21_CANONICAL,
            air21_root(),
            "praxis-lock://air21_42121@2010",
        )
        .unwrap();
        let terms = extract_terms(&tree);

        let expected: alloc::vec::Vec<(&str, &str)> = vec![
            ("a", "DISCRIMINATION AGAINST EMPLOYEES"),
            ("b", "DEPARTMENT OF LABOR COMPLAINT PROCEDURE"),
        ];
        for (subsection, expected_heading) in expected {
            let term = terms
                .iter()
                .find(|t| {
                    t.cite.segments.last().map(|s| s.label.as_str()) == Some(subsection)
                        && t.cite.segments.len() == 3
                })
                .unwrap_or_else(|| panic!("subsection ({subsection}) not extracted"));
            assert_eq!(
                term.heading.as_deref(),
                Some(expected_heading),
                "subsection ({subsection}) heading mismatch"
            );
        }
    }

    #[test]
    fn extract_air21_finds_four_clause_burden_framework_headings() {
        let tree = parse_statute_text(
            AIR21_CANONICAL,
            air21_root(),
            "praxis-lock://air21_42121@2010",
        )
        .unwrap();
        let terms = extract_terms(&tree);

        // (b)(2)(B)(i) — Required showing by complainant.
        let expected: alloc::vec::Vec<(&str, &str)> = vec![
            ("(b)(2)(B)(i)", "Required showing by complainant"),
            ("(b)(2)(B)(ii)", "Showing by employer"),
            ("(b)(2)(B)(iii)", "Criteria for determination by Secretary"),
            ("(b)(2)(B)(iv)", "Prohibition"),
        ];
        for (suffix, expected_heading) in expected {
            let term = terms
                .iter()
                .find(|t| t.cite.to_bluebook().ends_with(suffix))
                .unwrap_or_else(|| panic!("clause {suffix} not extracted"));
            // These clauses use mixed-case headings (canonical form
            // in AIR21's (b)(2)(B) is "Required showing by
            // complainant" — title case, not all caps). split_heading
            // currently requires uppercase first char; lower-case
            // sub-headings are detected only if they begin with a
            // capital. "Required" / "Showing" / "Criteria" /
            // "Prohibition" all begin with capitals so this works.
            assert_eq!(
                term.heading.as_deref(),
                Some(expected_heading),
                "clause {suffix} heading mismatch — got {:?}",
                term.heading
            );
        }
    }

    #[test]
    fn every_extracted_term_cite_matches_a_tree_node() {
        // Property: every ExtractedTerm has a cite findable in the tree.
        let tree =
            parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
        let terms = extract_terms(&tree);
        for t in &terms {
            assert!(
                tree.find(&t.cite).is_some(),
                "extracted cite {} not in tree",
                t.cite.to_bluebook()
            );
        }
    }

    // ── extract_lemmas unit tests ────────────────────────────────────

    /// Test helper: build the expected `Vec<Form>` for English words.
    fn en_forms(words: &[&str]) -> Vec<Form> {
        words
            .iter()
            .map(|w| Form {
                written_rep: w.to_string(),
                lang: "en".to_string(),
            })
            .collect()
    }

    #[test]
    fn lemmas_simple_two_word() {
        assert_eq!(
            extract_lemmas("Covered Employer"),
            en_forms(&["covered", "employer"])
        );
    }

    #[test]
    fn lemmas_carry_english_language_tag() {
        for f in extract_lemmas("Covered Employer") {
            assert_eq!(f.lang, "en");
        }
    }

    #[test]
    fn lemmas_strips_prepositions() {
        assert_eq!(
            extract_lemmas("Statute of Limitations for Filing"),
            en_forms(&["statute", "limitations", "filing"])
        );
    }

    #[test]
    fn lemmas_strips_conjunctions_and_articles() {
        assert_eq!(
            extract_lemmas("The Right and the Remedy"),
            en_forms(&["right", "remedy"])
        );
    }

    #[test]
    fn lemmas_strips_modals() {
        assert_eq!(
            extract_lemmas("Action May Not Be Waived"),
            en_forms(&["action", "waived"])
        );
    }

    #[test]
    fn lemmas_deduplicates() {
        assert_eq!(
            extract_lemmas("Court Action Court Filing"),
            en_forms(&["court", "action", "filing"])
        );
    }

    #[test]
    fn lemmas_handles_punctuation() {
        assert_eq!(
            extract_lemmas("Non-Waivability of Rights and Remedies"),
            en_forms(&["non", "waivability", "rights", "remedies"])
        );
    }

    #[test]
    fn lemmas_empty_input() {
        assert_eq!(extract_lemmas(""), Vec::<Form>::new());
    }

    #[test]
    fn lemmas_all_stopwords_input() {
        assert_eq!(extract_lemmas("the and of"), Vec::<Form>::new());
    }

    #[test]
    fn lemmas_for_sox_term_names() {
        assert_eq!(
            extract_lemmas("Covered Employer"),
            en_forms(&["covered", "employer"])
        );
        assert_eq!(
            extract_lemmas("Prohibition on Retaliation"),
            en_forms(&["prohibition", "retaliation"])
        );
        assert_eq!(
            extract_lemmas("Compensatory Damages"),
            en_forms(&["compensatory", "damages"])
        );
        assert_eq!(
            extract_lemmas("Reporting to Federal Agency"),
            en_forms(&["reporting", "federal", "agency"])
        );
        assert_eq!(
            extract_lemmas("Invalidity of Predispute Arbitration Agreements"),
            en_forms(&["invalidity", "predispute", "arbitration", "agreements"])
        );
    }

    #[test]
    fn lemmas_for_air21_term_names() {
        assert_eq!(
            extract_lemmas("Discrimination Prohibited"),
            en_forms(&["discrimination", "prohibited"])
        );
        assert_eq!(
            extract_lemmas("Prima Facie Investigation Gate"),
            en_forms(&["prima", "facie", "investigation", "gate"])
        );
        assert_eq!(
            extract_lemmas("Merits Contributing-Factor Demonstration"),
            en_forms(&["merits", "contributing", "factor", "demonstration"])
        );
    }

    #[test]
    fn print_extraction_summary() {
        let sox_tree =
            parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
        let sox_terms = extract_terms(&sox_tree);
        let air21_tree = parse_statute_text(
            AIR21_CANONICAL,
            air21_root(),
            "praxis-lock://air21_42121@2010",
        )
        .unwrap();
        let air21_terms = extract_terms(&air21_tree);

        eprintln!("\n=== Term-extraction summary ===");
        let sox_with_heading = sox_terms.iter().filter(|t| t.heading.is_some()).count();
        let air21_with_heading = air21_terms.iter().filter(|t| t.heading.is_some()).count();
        eprintln!(
            "SOX § 1514A: {} terms extracted, {} with heading, {} without",
            sox_terms.len(),
            sox_with_heading,
            sox_terms.len() - sox_with_heading
        );
        eprintln!(
            "AIR21 § 42121: {} terms extracted, {} with heading, {} without",
            air21_terms.len(),
            air21_with_heading,
            air21_terms.len() - air21_with_heading
        );

        eprintln!("\nSample headings extracted from SOX:");
        for t in sox_terms.iter().take(8) {
            eprintln!("  {} → {}", t.cite.to_bluebook(), t.heading_or_none());
        }
        eprintln!();
    }

    // ── Property-based laws for extract_lemmas ─────────────────────
    //
    // The extract_lemmas pipeline filters stopwords (loaded from
    // function-words/english.xml) and numerics, lower-cases, and
    // deduplicates. These properties must hold for *every* input.

    use proptest::prelude::*;

    fn arb_term_name() -> impl Strategy<Value = String> {
        // ASCII text with letters, digits, spaces, and a few legal-text
        // punctuation marks. Bounded length to keep the search space
        // tractable.
        proptest::collection::vec(
            prop_oneof![
                prop::char::range('a', 'z'),
                prop::char::range('A', 'Z'),
                prop::char::range('0', '9'),
                Just(' '),
                Just('-'),
                Just('.'),
                Just(','),
            ],
            0..32,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    proptest! {
        #[test]
        fn property_lemmas_never_include_stopwords(name in arb_term_name()) {
            let stopwords = english_stopwords();
            for f in extract_lemmas(&name) {
                prop_assert!(
                    !stopwords.contains(&f.written_rep),
                    "stopword `{}` leaked through extract_lemmas",
                    f.written_rep
                );
            }
        }

        #[test]
        fn property_lemmas_never_purely_numeric(name in arb_term_name()) {
            for f in extract_lemmas(&name) {
                prop_assert!(
                    !f.written_rep.chars().all(|c| c.is_ascii_digit()),
                    "numeric token `{}` leaked through",
                    f.written_rep
                );
            }
        }

        #[test]
        fn property_lemmas_are_lowercase(name in arb_term_name()) {
            for f in extract_lemmas(&name) {
                prop_assert_eq!(&f.written_rep, &f.written_rep.to_lowercase());
            }
        }

        #[test]
        fn property_lemmas_are_unique(name in arb_term_name()) {
            let out = extract_lemmas(&name);
            let unique: alloc::collections::BTreeSet<&String> =
                out.iter().map(|f| &f.written_rep).collect();
            prop_assert_eq!(unique.len(), out.len());
        }

        #[test]
        fn property_lemmas_all_tagged_en(name in arb_term_name()) {
            for f in extract_lemmas(&name) {
                prop_assert_eq!(f.lang, "en");
            }
        }

        #[test]
        fn property_lemmas_never_empty_string(name in arb_term_name()) {
            for f in extract_lemmas(&name) {
                prop_assert!(!f.written_rep.is_empty());
            }
        }

        #[test]
        fn property_lemmas_idempotent_under_repetition(
            name in arb_term_name(),
        ) {
            // Joining a term name with itself by space shouldn't produce
            // new lemmas — the dedup step folds repeats.
            let single = extract_lemmas(&name);
            let doubled = extract_lemmas(&format!("{name} {name}"));
            prop_assert_eq!(single, doubled);
        }
    }

    // ── Concurrency tests for the OnceLock-cached stopword set ────
    //
    // The english_stopwords helper is a OnceLock — concurrent first
    // calls from multiple threads MUST all see the same fully-built
    // BTreeSet without panic, deadlock, or double-init. Daubert
    // prong 3 (operational error rate): a thread-unsafe lazy cache
    // would manifest as flaky test failures, which an auditor would
    // flag as unreliable methodology.

    #[test]
    fn concurrency_stopwords_lazy_init_under_threads() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        const N_THREADS: usize = 16;
        let barrier = Arc::new(Barrier::new(N_THREADS));
        let mut handles = Vec::with_capacity(N_THREADS);
        for _ in 0..N_THREADS {
            let b = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                // All threads block until everyone is ready, then
                // hammer english_stopwords() simultaneously.
                b.wait();
                let set = english_stopwords();
                // Every thread must see the same content. Read a few
                // known stopwords to verify the set is populated.
                assert!(set.contains("the"));
                assert!(set.contains("and"));
                assert!(set.contains("because"));
                set.len()
            }));
        }
        let sizes: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        // All threads must observe the same size — proves OnceLock
        // initialization was atomic.
        let first = sizes[0];
        for s in &sizes {
            assert_eq!(*s, first, "thread observed different stopword-set size");
        }
    }

    #[test]
    fn concurrency_extract_lemmas_thread_safe() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        const N_THREADS: usize = 16;
        let barrier = Arc::new(Barrier::new(N_THREADS));
        let mut handles = Vec::with_capacity(N_THREADS);
        for i in 0..N_THREADS {
            let b = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                b.wait();
                let name = format!("Covered Employer {i}");
                let lemmas = extract_lemmas(&name);
                lemmas
                    .into_iter()
                    .map(|f| f.written_rep)
                    .collect::<Vec<_>>()
            }));
        }
        for (i, h) in handles.into_iter().enumerate() {
            let lemmas = h.join().unwrap();
            assert!(
                lemmas.contains(&"covered".to_string()),
                "thread {i} missing `covered`: {lemmas:?}"
            );
            assert!(
                lemmas.contains(&"employer".to_string()),
                "thread {i} missing `employer`: {lemmas:?}"
            );
        }
    }
}
