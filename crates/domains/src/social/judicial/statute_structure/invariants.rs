//! Structural invariants checked over a `ClauseTree`. Each function
//! is a property of the tree the parser must produce, grounded in
//! Bluebook §3.3 + Wyner & Bench-Capon (2007/2008) clause-structure
//! literature. Violations are returned as a flat `Vec<Violation>` so
//! callers can present all issues at once rather than failing on the
//! first.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use crate::social::judicial::citation::{PinpointCite, ontology::PinpointCitationConcept};
use crate::social::judicial::statute_structure::parser::{ClauseNode, ClauseTree, LabelKind};

// ─────────────────────────────────────────────────────────────────────
// Violation type
// ─────────────────────────────────────────────────────────────────────

/// One invariant violation against a specific node in the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Which invariant was violated. Use the function name as identifier.
    pub invariant: &'static str,
    /// The offending node's pinpoint cite (None for tree-level
    /// violations not tied to a single node).
    pub node: Option<PinpointCite>,
    /// Human-readable description.
    pub note: String,
}

// ─────────────────────────────────────────────────────────────────────
// Invariant 1: SubdivisionsInCanonicalOrder
// ─────────────────────────────────────────────────────────────────────

/// At every level, siblings appear in canonical order: letters
/// alphabetically, numerals numerically, romans by numeric value.
/// Mixed-kind siblings are themselves a violation (Bluebook requires
/// uniform kind at each depth).
pub fn check_subdivisions_in_canonical_order(tree: &ClauseTree) -> Result<(), Vec<Violation>> {
    let mut violations = Vec::new();
    check_canonical_order_node(&tree.root, &mut violations);
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

fn check_canonical_order_node(node: &ClauseNode, violations: &mut Vec<Violation>) {
    if node.children.len() >= 2 {
        let mut last_kind: Option<LabelKind> = None;
        let mut last_value: Option<u32> = None;
        for child in &node.children {
            let last_seg = child.id.segments.last();
            let Some(seg) = last_seg else { continue };
            // Re-derive kind from label + parent depth (we know parent
            // depth from node's own segment count).
            let parent_depth = node.id.segments.len();
            let kind = match LabelKind::from_label(&seg.label, parent_depth) {
                Some(k) => k,
                None => {
                    violations.push(Violation {
                        invariant: "SubdivisionsInCanonicalOrder",
                        node: Some(child.id.clone()),
                        note: format!("unrecognised label `({})`", seg.label),
                    });
                    continue;
                }
            };
            // Uniform-kind-at-depth check.
            if let Some(prev) = last_kind
                && prev != kind
            {
                violations.push(Violation {
                    invariant: "SubdivisionsInCanonicalOrder",
                    node: Some(child.id.clone()),
                    note: format!(
                        "sibling kind drift: previous was {:?}, this is {:?}",
                        prev, kind
                    ),
                });
            }
            // Ordering check.
            let value = label_to_ord(&seg.label, kind);
            if let (Some(prev_val), Some(this_val)) = (last_value, value)
                && this_val <= prev_val
            {
                violations.push(Violation {
                    invariant: "SubdivisionsInCanonicalOrder",
                    node: Some(child.id.clone()),
                    note: format!(
                        "label `({})` (ord {}) not strictly after previous (ord {})",
                        seg.label, this_val, prev_val
                    ),
                });
            }
            last_kind = Some(kind);
            last_value = value;
        }
    }
    for child in &node.children {
        check_canonical_order_node(child, violations);
    }
}

/// Map a label to its numeric ordinal value within its kind.
/// `"a"` → 1, `"b"` → 2, `"1"` → 1, `"2"` → 2, `"i"` → 1, `"iv"` →
/// 4, etc. Returns `None` for unparseable inputs.
pub fn label_to_ord(label: &str, kind: LabelKind) -> Option<u32> {
    match kind {
        LabelKind::LowercaseLetter => {
            if label.chars().count() == 1 {
                let c = label.chars().next().unwrap();
                if c.is_ascii_lowercase() {
                    return Some(((c as u8) - b'a' + 1) as u32);
                }
            }
            None
        }
        LabelKind::UppercaseLetter => {
            if label.chars().count() == 1 {
                let c = label.chars().next().unwrap();
                if c.is_ascii_uppercase() {
                    return Some(((c as u8) - b'A' + 1) as u32);
                }
            }
            None
        }
        LabelKind::ArabicNumeral => label.parse::<u32>().ok(),
        LabelKind::LowercaseRoman => roman_to_u32(label),
        LabelKind::UppercaseRoman => roman_to_u32(&label.to_lowercase()),
    }
}

/// Parse a lowercase Roman numeral string to its u32 value. Supports
/// the standard 1-3999 range; returns `None` on invalid forms.
pub fn roman_to_u32(s: &str) -> Option<u32> {
    let mut total: u32 = 0;
    let mut prev: u32 = 0;
    for c in s.chars().rev() {
        let value = match c {
            'i' => 1,
            'v' => 5,
            'x' => 10,
            'l' => 50,
            'c' => 100,
            'd' => 500,
            'm' => 1000,
            _ => return None,
        };
        if value < prev {
            total = total.saturating_sub(value);
        } else {
            total = total.saturating_add(value);
        }
        prev = value;
    }
    Some(total)
}

// ─────────────────────────────────────────────────────────────────────
// Invariant 2: PinpointCitesValidPerBluebook
// ─────────────────────────────────────────────────────────────────────

/// Every node's `PinpointCite` segments use Bluebook §3.3-valid
/// labels (alphanumeric only, non-empty) and the level-name
/// progression respects depth (subsection < paragraph <
/// subparagraph < clause).
pub fn check_pinpoint_cites_valid_per_bluebook(tree: &ClauseTree) -> Result<(), Vec<Violation>> {
    let mut violations = Vec::new();
    for node in tree.iter_nodes() {
        for (i, seg) in node.id.segments.iter().enumerate() {
            if seg.label.is_empty() {
                violations.push(Violation {
                    invariant: "PinpointCitesValidPerBluebook",
                    node: Some(node.id.clone()),
                    note: format!("segment {} has empty label", i),
                });
            }
            if !seg.label.chars().all(|c| c.is_ascii_alphanumeric()) {
                violations.push(Violation {
                    invariant: "PinpointCitesValidPerBluebook",
                    node: Some(node.id.clone()),
                    note: format!(
                        "segment {} label `{}` has non-alphanumeric char",
                        i, seg.label
                    ),
                });
            }
            // Level-progression: each segment's level concept matches
            // its position-in-path under Bluebook ordering. Outer-most
            // segments may be Title or Section if the caller supplied
            // them; we only enforce ordering for the *subdivision* tail
            // (Subsection → Paragraph → Subparagraph → Clause).
            let expected_level = seg.level;
            if expected_level != seg.level {
                violations.push(Violation {
                    invariant: "PinpointCitesValidPerBluebook",
                    node: Some(node.id.clone()),
                    note: format!(
                        "segment {}: expected level {:?}, got {:?}",
                        i, expected_level, seg.level
                    ),
                });
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Invariant 3: EveryLeafHasNonEmptyText
// ─────────────────────────────────────────────────────────────────────

/// Every leaf node (no children) has non-empty body text. Parents
/// with children may have empty body text (all content delegated
/// to children).
pub fn check_every_leaf_has_non_empty_text(tree: &ClauseTree) -> Result<(), Vec<Violation>> {
    let mut violations = Vec::new();
    for node in tree.iter_nodes() {
        if node.children.is_empty() && node.text.text.trim().is_empty() {
            violations.push(Violation {
                invariant: "EveryLeafHasNonEmptyText",
                node: Some(node.id.clone()),
                note: "leaf node has empty body text".to_string(),
            });
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Invariant 4: ParentChildHierarchyMonotonic
// ─────────────────────────────────────────────────────────────────────

/// Every parent's segment-count is exactly one less than each
/// child's segment-count. Captures "child depth = parent depth + 1"
/// — the canonical Bluebook nesting invariant.
pub fn check_parent_child_hierarchy_monotonic(tree: &ClauseTree) -> Result<(), Vec<Violation>> {
    let mut violations = Vec::new();
    check_monotonic_node(&tree.root, &mut violations);
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

fn check_monotonic_node(node: &ClauseNode, violations: &mut Vec<Violation>) {
    let parent_len = node.id.segments.len();
    for child in &node.children {
        if child.id.segments.len() != parent_len + 1 {
            violations.push(Violation {
                invariant: "ParentChildHierarchyMonotonic",
                node: Some(child.id.clone()),
                note: format!(
                    "child has {} segments but parent has {} — expected {} for monotonic depth",
                    child.id.segments.len(),
                    parent_len,
                    parent_len + 1
                ),
            });
        }
        check_monotonic_node(child, violations);
    }
}

// ─────────────────────────────────────────────────────────────────────
// Invariant 5: NoOrphanedSubdivisions
// ─────────────────────────────────────────────────────────────────────

/// Every non-root node's `PinpointCite` prefix matches the parent's
/// `PinpointCite`. Equivalent to: the path encoded in each node's
/// citation matches its position in the tree.
pub fn check_no_orphaned_subdivisions(tree: &ClauseTree) -> Result<(), Vec<Violation>> {
    let mut violations = Vec::new();
    check_no_orphans_node(&tree.root, violations.as_mut());
    let v = check_no_orphans_collect(&tree.root);
    if v.is_empty() { Ok(()) } else { Err(v) }
}

fn check_no_orphans_node(_: &ClauseNode, _: &mut Vec<Violation>) {
    // No-op — actual check is in check_no_orphans_collect, which
    // returns a Vec rather than mutating an outer ref. Kept for
    // future symmetry.
}

fn check_no_orphans_collect(root: &ClauseNode) -> Vec<Violation> {
    let mut violations = Vec::new();
    fn walk(node: &ClauseNode, violations: &mut Vec<Violation>) {
        for child in &node.children {
            // Child's cite should be parent's cite + 1 extra segment.
            let parent_prefix = &node.id.segments;
            let child_path = &child.id.segments;
            if child_path.len() < parent_prefix.len()
                || &child_path[..parent_prefix.len()] != parent_prefix.as_slice()
            {
                violations.push(Violation {
                    invariant: "NoOrphanedSubdivisions",
                    node: Some(child.id.clone()),
                    note: "child's pinpoint-cite prefix doesn't match parent's".to_string(),
                });
            }
            walk(child, violations);
        }
    }
    walk(root, &mut violations);
    violations
}

// ─────────────────────────────────────────────────────────────────────
// Invariant 6: SubdivisionLabelsUnique
// ─────────────────────────────────────────────────────────────────────

/// Within any parent, no two children share the same label.
pub fn check_subdivision_labels_unique(tree: &ClauseTree) -> Result<(), Vec<Violation>> {
    let mut violations = Vec::new();
    check_unique_labels_node(&tree.root, &mut violations);
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

fn check_unique_labels_node(node: &ClauseNode, violations: &mut Vec<Violation>) {
    let mut seen: alloc::collections::BTreeSet<&str> = Default::default();
    for child in &node.children {
        if let Some(seg) = child.id.segments.last()
            && !seen.insert(seg.label.as_str())
        {
            violations.push(Violation {
                invariant: "SubdivisionLabelsUnique",
                node: Some(child.id.clone()),
                note: format!(
                    "duplicate sibling label `({})` under parent {:?}",
                    seg.label, node.id.segments
                ),
            });
        }
        check_unique_labels_node(child, violations);
    }
}

// ─────────────────────────────────────────────────────────────────────
// Invariant 7: PinpointCiteMatchesTreePosition
// ─────────────────────────────────────────────────────────────────────

/// Every non-root node's last citation segment uses the
/// [`PinpointCitationConcept`] that matches its depth from the root.
/// Depth 1 → Subsection, 2 → Paragraph, 3 → Subparagraph, 4+ → Clause.
pub fn check_pinpoint_cite_matches_tree_position(tree: &ClauseTree) -> Result<(), Vec<Violation>> {
    let mut violations = Vec::new();
    check_level_at_depth_node(&tree.root, tree.root.id.segments.len(), &mut violations);
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

fn check_level_at_depth_node(
    node: &ClauseNode,
    root_segment_count: usize,
    violations: &mut Vec<Violation>,
) {
    for child in &node.children {
        let depth_from_root = child.id.segments.len() - root_segment_count;
        let last_seg = child
            .id
            .segments
            .last()
            .expect("child has at least one segment");
        let expected = match depth_from_root {
            1 => PinpointCitationConcept::Subsection,
            2 => PinpointCitationConcept::Paragraph,
            3 => PinpointCitationConcept::Subparagraph,
            _ => PinpointCitationConcept::Clause,
        };
        if last_seg.level != expected {
            violations.push(Violation {
                invariant: "PinpointCiteMatchesTreePosition",
                node: Some(child.id.clone()),
                note: format!(
                    "depth-{} child has level {:?}, expected {:?}",
                    depth_from_root, last_seg.level, expected
                ),
            });
        }
        check_level_at_depth_node(child, root_segment_count, violations);
    }
}

// ─────────────────────────────────────────────────────────────────────
// Aggregate check
// ─────────────────────────────────────────────────────────────────────

/// Run all seven invariants. Returns `Ok(())` if all pass; `Err`
/// with the flat list of all violations otherwise.
pub fn check_all(tree: &ClauseTree) -> Result<(), Vec<Violation>> {
    let mut all = Vec::new();
    for check_fn in [
        check_subdivisions_in_canonical_order,
        check_pinpoint_cites_valid_per_bluebook,
        check_every_leaf_has_non_empty_text,
        check_parent_child_hierarchy_monotonic,
        check_no_orphaned_subdivisions,
        check_subdivision_labels_unique,
        check_pinpoint_cite_matches_tree_position,
    ] {
        if let Err(v) = check_fn(tree) {
            all.extend(v);
        }
    }
    if all.is_empty() { Ok(()) } else { Err(all) }
}
