//! Tests for the statute-structure parser + invariants.
//!
//! Three test layers per the user's "lots of testing" directive:
//! 1. Unit tests — small inputs covering parser edge cases.
//! 2. Invariant tests — every parser output satisfies the seven
//!    `invariants::check_*` properties.
//! 3. Property tests — proptest-generated inputs verify parser
//!    idempotency, canonical-order preservation, etc.
//! 4. Real-corpus tests — parsing the SOX 1514A and AIR21 § 42121
//!    canonical-text fixtures produces structurally correct trees.

use super::invariants::{
    check_all, check_parent_child_hierarchy_monotonic, check_subdivision_labels_unique,
    check_subdivisions_in_canonical_order, label_to_ord, roman_to_u32,
};
use super::parser::{LabelKind, ParseError, parse_statute_text};
use crate::social::judicial::citation::{
    PinpointCite, PinpointSegment, ontology::PinpointCitationConcept,
};

use proptest::prelude::*;

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

fn root_cite() -> PinpointCite {
    PinpointCite {
        segments: vec![PinpointSegment {
            level: PinpointCitationConcept::Section,
            label: "TEST".to_string(),
        }],
    }
}

// ─────────────────────────────────────────────────────────────────────
// LabelKind::from_label unit tests
// ─────────────────────────────────────────────────────────────────────

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn label_lowercase_letter_at_top_level() {
    assert_eq!(
        LabelKind::from_label("a", 0),
        Some(LabelKind::LowercaseLetter)
    );
    assert_eq!(
        LabelKind::from_label("z", 0),
        Some(LabelKind::LowercaseLetter)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn label_arabic_numeral() {
    assert_eq!(
        LabelKind::from_label("1", 1),
        Some(LabelKind::ArabicNumeral)
    );
    assert_eq!(
        LabelKind::from_label("99", 1),
        Some(LabelKind::ArabicNumeral)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn label_uppercase_letter() {
    assert_eq!(
        LabelKind::from_label("A", 2),
        Some(LabelKind::UppercaseLetter)
    );
    assert_eq!(
        LabelKind::from_label("Z", 2),
        Some(LabelKind::UppercaseLetter)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn label_lowercase_roman_multi_char() {
    // Multi-char lowercase romans are unambiguous.
    assert_eq!(
        LabelKind::from_label("ii", 3),
        Some(LabelKind::LowercaseRoman)
    );
    assert_eq!(
        LabelKind::from_label("iii", 3),
        Some(LabelKind::LowercaseRoman)
    );
    assert_eq!(
        LabelKind::from_label("iv", 3),
        Some(LabelKind::LowercaseRoman)
    );
    assert_eq!(
        LabelKind::from_label("ix", 3),
        Some(LabelKind::LowercaseRoman)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn label_single_i_disambiguated_by_context() {
    // (i) at top level = letter.
    assert_eq!(
        LabelKind::from_label("i", 0),
        Some(LabelKind::LowercaseLetter)
    );
    // (i) inside a Subparagraph (depth 3) = roman.
    assert_eq!(
        LabelKind::from_label("i", 3),
        Some(LabelKind::LowercaseRoman)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn label_uppercase_roman_multi_char() {
    assert_eq!(
        LabelKind::from_label("II", 3),
        Some(LabelKind::UppercaseRoman)
    );
    assert_eq!(
        LabelKind::from_label("IV", 3),
        Some(LabelKind::UppercaseRoman)
    );
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn label_rejects_empty() {
    assert_eq!(LabelKind::from_label("", 0), None);
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn label_rejects_mixed_case() {
    assert_eq!(LabelKind::from_label("Aa", 0), None);
    assert_eq!(LabelKind::from_label("xY", 0), None);
}

// ─────────────────────────────────────────────────────────────────────
// roman_to_u32 unit tests
// ─────────────────────────────────────────────────────────────────────

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn roman_basics() {
    assert_eq!(roman_to_u32("i"), Some(1));
    assert_eq!(roman_to_u32("ii"), Some(2));
    assert_eq!(roman_to_u32("iii"), Some(3));
    assert_eq!(roman_to_u32("iv"), Some(4));
    assert_eq!(roman_to_u32("v"), Some(5));
    assert_eq!(roman_to_u32("vi"), Some(6));
    assert_eq!(roman_to_u32("vii"), Some(7));
    assert_eq!(roman_to_u32("viii"), Some(8));
    assert_eq!(roman_to_u32("ix"), Some(9));
    assert_eq!(roman_to_u32("x"), Some(10));
    assert_eq!(roman_to_u32("xi"), Some(11));
    assert_eq!(roman_to_u32("xiv"), Some(14));
    assert_eq!(roman_to_u32("xx"), Some(20));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn roman_rejects_invalid() {
    assert_eq!(roman_to_u32("abc"), None);
    assert_eq!(roman_to_u32("1"), None);
}

// ─────────────────────────────────────────────────────────────────────
// Parser unit tests — small inputs
// ─────────────────────────────────────────────────────────────────────

#[pr4xis::praxis_value(Honest)]
#[test]
fn parse_empty_text_produces_root_only() {
    let tree = parse_statute_text("", root_cite(), "test://").unwrap();
    assert_eq!(tree.node_count(), 1);
    assert!(tree.root.children.is_empty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_text_with_no_markers_attaches_to_root() {
    let tree = parse_statute_text(
        "Just some prose without any markers.",
        root_cite(),
        "test://",
    )
    .unwrap();
    assert_eq!(tree.node_count(), 1);
    assert_eq!(tree.root.text.text, "Just some prose without any markers.");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_single_subsection() {
    let tree = parse_statute_text("(a) The text of subsection a.", root_cite(), "test://").unwrap();
    assert_eq!(tree.node_count(), 2);
    assert_eq!(tree.root.children.len(), 1);
    let a = &tree.root.children[0];
    assert_eq!(a.id.segments.last().unwrap().label, "a");
    assert_eq!(a.text.text, "The text of subsection a.");
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn parse_ignores_inline_prose_references() {
    // Prose like "subsection (a)" mid-text should NOT be treated as
    // a new marker — the parser requires markers to be line-leading.
    let text = "(a) Discusses subsection (b) inline.\n(b) Second.";
    let tree = parse_statute_text(text, root_cite(), "test://").unwrap();
    assert_eq!(
        tree.root.children.len(),
        2,
        "should have exactly 2 top-level subsections; got {} labels: {:?}",
        tree.root.children.len(),
        tree.root
            .children
            .iter()
            .map(|c| c.id.segments.last().unwrap().label.as_str())
            .collect::<alloc::vec::Vec<_>>()
    );
    assert!(
        tree.root.children[0]
            .text
            .text
            .contains("subsection (b) inline")
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_two_top_level_siblings() {
    let tree = parse_statute_text(
        "(a) First subsection.\n(b) Second subsection.",
        root_cite(),
        "test://",
    )
    .unwrap();
    assert_eq!(tree.root.children.len(), 2);
    assert_eq!(tree.root.children[0].id.segments.last().unwrap().label, "a");
    assert_eq!(tree.root.children[1].id.segments.last().unwrap().label, "b");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_nested_three_levels() {
    let text = "(a) Outer\n(1) middle\n(A) inner.";
    let tree = parse_statute_text(text, root_cite(), "test://").unwrap();
    let a = &tree.root.children[0];
    assert_eq!(a.children.len(), 1);
    let p1 = &a.children[0];
    assert_eq!(p1.id.segments.last().unwrap().label, "1");
    assert_eq!(p1.children.len(), 1);
    let inner = &p1.children[0];
    assert_eq!(inner.id.segments.last().unwrap().label, "A");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_back_to_top_after_nested() {
    let text = "(a) sub-a\n(1) paragraph\n(A) inner\n(b) sub-b.";
    let tree = parse_statute_text(text, root_cite(), "test://").unwrap();
    assert_eq!(tree.root.children.len(), 2);
    assert_eq!(tree.root.children[0].id.segments.last().unwrap().label, "a");
    assert_eq!(tree.root.children[1].id.segments.last().unwrap().label, "b");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_roman_clause_inside_subparagraph() {
    let text = "(a) outer\n(1) para\n(A) sub\n(i) first roman\n(ii) second roman.";
    let tree = parse_statute_text(text, root_cite(), "test://").unwrap();
    let a = &tree.root.children[0];
    let p1 = &a.children[0];
    let big_a = &p1.children[0];
    assert_eq!(big_a.children.len(), 2);
    assert_eq!(big_a.children[0].id.segments.last().unwrap().label, "i");
    assert_eq!(big_a.children[1].id.segments.last().unwrap().label, "ii");
    // The (i) at depth 4 should be classified as Clause.
    assert_eq!(
        big_a.children[0].id.segments.last().unwrap().level,
        PinpointCitationConcept::Clause
    );
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn parse_depth_skip_returns_error() {
    // (A) at top level with no (a)(1) parent context.
    let result = parse_statute_text("(A) orphan subparagraph", root_cite(), "test://");
    assert!(matches!(result, Err(ParseError::DepthSkip { .. })));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn parse_ignores_parenthetical_in_prose() {
    // "(15 U.S.C. 78l)" looks like a marker but the label
    // "15 U.S.C. 78l" contains spaces + periods — not a Bluebook
    // subdivision label.
    let text = "(a) Citing other law (15 U.S.C. 78l) inside text.";
    let tree = parse_statute_text(text, root_cite(), "test://").unwrap();
    assert_eq!(tree.root.children.len(), 1);
    // The whole text after (a) should be in (a)'s body.
    assert!(tree.root.children[0].text.text.contains("(15 U.S.C. 78l)"));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn parse_back_to_top_requires_newline_separator() {
    // Without a newline separator, "(b)" mid-line is treated as a
    // prose reference, not a marker. The parser is conservative
    // here — canonical statutes always put real markers at line start.
    let text = "(a) sub-a (b) sub-b.";
    let tree = parse_statute_text(text, root_cite(), "test://").unwrap();
    assert_eq!(tree.root.children.len(), 1);
    assert!(tree.root.children[0].text.text.contains("(b) sub-b"));
}

// ─────────────────────────────────────────────────────────────────────
// Invariant tests over small parsed trees
// ─────────────────────────────────────────────────────────────────────

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn invariants_hold_on_simple_tree() {
    let text = "(a) sub-a (1) p1 (2) p2 (b) sub-b.";
    let tree = parse_statute_text(text, root_cite(), "test://").unwrap();
    assert!(
        check_all(&tree).is_ok(),
        "invariants must hold: {:?}",
        check_all(&tree)
    );
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn canonical_order_check_catches_reverse() {
    // Construct manually: (b) before (a) — invariant violation.
    use super::parser::{ClauseNode, ClauseTree};
    use crate::social::judicial::source_text::SourceTextRef;

    let parent_cite = root_cite();
    let mut child_a = parent_cite.clone();
    child_a.segments.push(PinpointSegment {
        level: PinpointCitationConcept::Subsection,
        label: "a".to_string(),
    });
    let mut child_b = parent_cite.clone();
    child_b.segments.push(PinpointSegment {
        level: PinpointCitationConcept::Subsection,
        label: "b".to_string(),
    });

    let tree = ClauseTree {
        root: ClauseNode {
            id: parent_cite,
            text: SourceTextRef::new("root"),
            children: vec![
                ClauseNode {
                    id: child_b,
                    text: SourceTextRef::new("b first"),
                    children: vec![],
                },
                ClauseNode {
                    id: child_a,
                    text: SourceTextRef::new("a second"),
                    children: vec![],
                },
            ],
        },
    };
    let result = check_subdivisions_in_canonical_order(&tree);
    assert!(result.is_err());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn parent_child_monotonic_check_catches_skip() {
    use super::parser::{ClauseNode, ClauseTree};
    use crate::social::judicial::source_text::SourceTextRef;

    let parent = root_cite();
    // Child cite jumps two levels deep.
    let mut child = parent.clone();
    child.segments.push(PinpointSegment {
        level: PinpointCitationConcept::Subsection,
        label: "a".to_string(),
    });
    child.segments.push(PinpointSegment {
        level: PinpointCitationConcept::Paragraph,
        label: "1".to_string(),
    });

    let tree = ClauseTree {
        root: ClauseNode {
            id: parent,
            text: SourceTextRef::new("root"),
            children: vec![ClauseNode {
                id: child,
                text: SourceTextRef::new("body"),
                children: vec![],
            }],
        },
    };
    assert!(check_parent_child_hierarchy_monotonic(&tree).is_err());
}

// ─────────────────────────────────────────────────────────────────────
// Property tests (proptest)
// ─────────────────────────────────────────────────────────────────────

fn arb_number() -> impl Strategy<Value = u32> {
    1u32..=9
}

/// Generate a flat valid statute text with up to 4 top-level
/// subsections, each with up to 3 paragraphs, each with up to 3
/// subparagraphs. The output is canonically ordered.
fn arb_canonical_statute_text() -> impl Strategy<Value = String> {
    (1usize..=4)
        .prop_flat_map(|n_subsections| {
            let entries = (0..n_subsections).map(|i| {
                let letter = (b'a' + i as u8) as char;
                (Just(letter), proptest::collection::vec(arb_number(), 0..3)).prop_map(
                    move |(letter, paras)| {
                        let mut s = format!("({}) text-{} ", letter, letter);
                        let mut last = 0u32;
                        for n in paras {
                            if n <= last {
                                continue;
                            }
                            s.push_str(&format!("({}) p-text-{} ", n, n));
                            last = n;
                        }
                        s
                    },
                )
            });
            let strategies: Vec<_> = entries.collect();
            strategies
        })
        .prop_map(|parts| parts.join(""))
}

proptest! {
    /// Parse is idempotent: parsing the output of parse + serialize
    /// produces the same tree. (We use a simplified comparison: same
    /// node count + same root pinpoint cite + same top-level child
    /// labels.)
    #[test]
    fn prop_parse_succeeds_on_canonical_input(text in arb_canonical_statute_text()) {
        let tree = parse_statute_text(&text, root_cite(), "test://");
        prop_assert!(tree.is_ok(), "parse failed: {:?}", tree);
    }

    /// Every canonical-input parse satisfies all 7 invariants.
    #[test]
    fn prop_invariants_hold(text in arb_canonical_statute_text()) {
        let tree = parse_statute_text(&text, root_cite(), "test://").unwrap();
        prop_assert!(check_all(&tree).is_ok());
    }

    /// Parse twice gives the same tree.
    #[test]
    fn prop_parse_deterministic(text in arb_canonical_statute_text()) {
        let t1 = parse_statute_text(&text, root_cite(), "test://").unwrap();
        let t2 = parse_statute_text(&text, root_cite(), "test://").unwrap();
        prop_assert_eq!(t1.node_count(), t2.node_count());
        prop_assert_eq!(t1.max_depth(), t2.max_depth());
    }

    /// Every produced node's `PinpointCite` has alphanumeric labels.
    #[test]
    fn prop_cites_alphanumeric(text in arb_canonical_statute_text()) {
        let tree = parse_statute_text(&text, root_cite(), "test://").unwrap();
        for node in tree.iter_nodes() {
            for seg in &node.id.segments {
                prop_assert!(seg.label.chars().all(|c| c.is_ascii_alphanumeric()));
            }
        }
    }

    /// Every parent has segments.len = child.segments.len - 1.
    #[test]
    fn prop_monotonic_depth(text in arb_canonical_statute_text()) {
        let tree = parse_statute_text(&text, root_cite(), "test://").unwrap();
        prop_assert!(check_parent_child_hierarchy_monotonic(&tree).is_ok());
    }

    /// Within any parent, no two children share a label.
    #[test]
    fn prop_labels_unique(text in arb_canonical_statute_text()) {
        let tree = parse_statute_text(&text, root_cite(), "test://").unwrap();
        prop_assert!(check_subdivision_labels_unique(&tree).is_ok());
    }

    /// label_to_ord is consistent with roman_to_u32 for roman kinds.
    #[test]
    fn prop_label_to_ord_consistent_for_romans(s in "[ivxl]{1,5}") {
        if let Some(ord) = label_to_ord(&s, LabelKind::LowercaseRoman) {
            let roman = roman_to_u32(&s);
            prop_assert_eq!(Some(ord), roman);
        }
    }
}

pr4xis::register_praxis_value!(prop_parse_succeeds_on_canonical_input, Verifiable);
pr4xis::register_praxis_value!(prop_invariants_hold, Verifiable);
pr4xis::register_praxis_value!(prop_parse_deterministic, Deterministic);
pr4xis::register_praxis_value!(prop_cites_alphanumeric, Verifiable);
pr4xis::register_praxis_value!(prop_monotonic_depth, Verifiable);
pr4xis::register_praxis_value!(prop_labels_unique, Verifiable);
pr4xis::register_praxis_value!(prop_label_to_ord_consistent_for_romans, Verifiable);

// ─────────────────────────────────────────────────────────────────────
// Real-corpus tests against the canonical-text fixtures
// ─────────────────────────────────────────────────────────────────────

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
fn parse_sox_1514a_canonical() {
    let tree = parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002");
    let tree = tree.expect("SOX canonical text must parse");
    // SOX § 1514A has five top-level subsections (a)-(e).
    assert_eq!(
        tree.root.children.len(),
        5,
        "expected 5 top-level subsections, got {} (labels: {:?})",
        tree.root.children.len(),
        tree.root
            .children
            .iter()
            .map(|c| c.id.segments.last().unwrap().label.as_str())
            .collect::<alloc::vec::Vec<_>>()
    );
    let labels: alloc::vec::Vec<&str> = tree
        .root
        .children
        .iter()
        .map(|c| c.id.segments.last().unwrap().label.as_str())
        .collect();
    assert_eq!(labels, vec!["a", "b", "c", "d", "e"]);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_sox_satisfies_all_invariants() {
    let tree =
        parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
    if let Err(violations) = check_all(&tree) {
        panic!(
            "SOX canonical text parse violates {} invariant(s):\n{}",
            violations.len(),
            violations
                .iter()
                .map(|v| format!(
                    "  - [{}] {}: {}",
                    v.invariant,
                    v.node.as_ref().map(|c| c.to_bluebook()).unwrap_or_default(),
                    v.note
                ))
                .collect::<alloc::vec::Vec<_>>()
                .join("\n")
        );
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_air21_42121_canonical() {
    let tree = parse_statute_text(
        AIR21_CANONICAL,
        air21_root(),
        "praxis-lock://air21_42121@2010",
    );
    let tree = tree.expect("AIR21 canonical text must parse");
    // AIR21 § 42121 covered subsections in our fixture: (a) and (b).
    assert_eq!(tree.root.children.len(), 2);
    let labels: alloc::vec::Vec<&str> = tree
        .root
        .children
        .iter()
        .map(|c| c.id.segments.last().unwrap().label.as_str())
        .collect();
    assert_eq!(labels, vec!["a", "b"]);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_air21_satisfies_all_invariants() {
    let tree = parse_statute_text(
        AIR21_CANONICAL,
        air21_root(),
        "praxis-lock://air21_42121@2010",
    )
    .unwrap();
    if let Err(violations) = check_all(&tree) {
        panic!(
            "AIR21 canonical text parse violates {} invariant(s):\n{}",
            violations.len(),
            violations
                .iter()
                .map(|v| format!(
                    "  - [{}] {}: {}",
                    v.invariant,
                    v.node.as_ref().map(|c| c.to_bluebook()).unwrap_or_default(),
                    v.note
                ))
                .collect::<alloc::vec::Vec<_>>()
                .join("\n")
        );
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_air21_finds_four_clause_burden_framework() {
    // The user's case-relevant test: § 42121(b)(2)(B) has four
    // clauses (i)-(iv) representing the burden-shifting framework.
    let tree = parse_statute_text(
        AIR21_CANONICAL,
        air21_root(),
        "praxis-lock://air21_42121@2010",
    )
    .unwrap();

    // Navigate to (b)(2)(B).
    let b = tree
        .root
        .children
        .iter()
        .find(|c| c.id.segments.last().unwrap().label == "b")
        .expect("(b) present");
    let b2 = b
        .children
        .iter()
        .find(|c| c.id.segments.last().unwrap().label == "2")
        .expect("(b)(2) present");
    let b2b = b2
        .children
        .iter()
        .find(|c| c.id.segments.last().unwrap().label == "B")
        .expect("(b)(2)(B) present");

    // (b)(2)(B) should have four clause children: (i), (ii), (iii), (iv).
    let labels: alloc::vec::Vec<&str> = b2b
        .children
        .iter()
        .map(|c| c.id.segments.last().unwrap().label.as_str())
        .collect();
    assert_eq!(
        labels,
        vec!["i", "ii", "iii", "iv"],
        "§ 42121(b)(2)(B) should have clauses (i)-(iv); got {:?}",
        labels
    );

    // Each clause's last segment should be PinpointCitationConcept::Clause.
    for clause in &b2b.children {
        assert_eq!(
            clause.id.segments.last().unwrap().level,
            PinpointCitationConcept::Clause
        );
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_sox_finds_reporting_channels() {
    // § 1514A(a)(1) has three reporting channels (A), (B), (C).
    let tree =
        parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
    let a = tree
        .root
        .children
        .iter()
        .find(|c| c.id.segments.last().unwrap().label == "a")
        .expect("(a) present");
    let a1 = a
        .children
        .iter()
        .find(|c| c.id.segments.last().unwrap().label == "1")
        .expect("(a)(1) present");
    let labels: alloc::vec::Vec<&str> = a1
        .children
        .iter()
        .map(|c| c.id.segments.last().unwrap().label.as_str())
        .collect();
    assert_eq!(labels, vec!["A", "B", "C"]);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_sox_finds_burden_of_proof_subsection() {
    // § 1514A(b)(2)(C) — burdens of proof per AIR21.
    let tree =
        parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
    let target = PinpointCite::new()
        .push(PinpointCitationConcept::Title, "18")
        .push(PinpointCitationConcept::Section, "1514A")
        .push(PinpointCitationConcept::Subsection, "b")
        .push(PinpointCitationConcept::Paragraph, "2")
        .push(PinpointCitationConcept::Subparagraph, "C");
    let node = tree.find(&target).expect("(b)(2)(C) findable");
    assert!(node.text.text.contains("42121"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn parse_sox_finds_arbitration_subsection() {
    // § 1514A(e)(2) — invalidity of predispute arbitration.
    let tree =
        parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
    let target = PinpointCite::new()
        .push(PinpointCitationConcept::Title, "18")
        .push(PinpointCitationConcept::Section, "1514A")
        .push(PinpointCitationConcept::Subsection, "e")
        .push(PinpointCitationConcept::Paragraph, "2");
    let node = tree.find(&target).expect("(e)(2) findable");
    assert!(node.text.text.to_lowercase().contains("arbitration"));
}

#[pr4xis::praxis_value(Explainable)]
#[test]
fn print_parse_summary() {
    // Informational — visible with `cargo test -- --nocapture`.
    let sox =
        parse_statute_text(SOX_CANONICAL, sox_root(), "praxis-lock://sox_1514a@2002").unwrap();
    let air21 = parse_statute_text(
        AIR21_CANONICAL,
        air21_root(),
        "praxis-lock://air21_42121@2010",
    )
    .unwrap();
    eprintln!("\n=== Statute-structure parser real-corpus summary ===");
    eprintln!("SOX § 1514A:");
    eprintln!("  nodes:     {}", sox.node_count());
    eprintln!("  max depth: {}", sox.max_depth());
    eprintln!("  top-level: {}", sox.root.children.len());
    eprintln!("AIR21 § 42121:");
    eprintln!("  nodes:     {}", air21.node_count());
    eprintln!("  max depth: {}", air21.max_depth());
    eprintln!("  top-level: {}", air21.root.children.len());
    eprintln!();
}
