//! Parser types + the `parse_statute_text` entry point.
//!
//! See `mod.rs` for the literature and pipeline position. The parser
//! is a single-pass stack-based algorithm: scan the input for
//! Bluebook §3.3 subdivision markers, infer each marker's kind +
//! depth, and assemble a tree where each marker becomes a clause
//! node and text between markers becomes the body of the most
//! recently opened node.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use crate::social::judicial::citation::{
    PinpointCite, PinpointSegment, ontology::PinpointCitationConcept,
};
use crate::social::judicial::source_text::SourceTextRef;

// ─────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────

/// One node in the parsed statute-structure tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClauseNode {
    /// The pinpoint citation path identifying this clause from the
    /// statute's root (e.g., `(b)(2)(B)(i)` ↔ a 4-segment cite).
    pub id: PinpointCite,
    /// Body text — the prose between this marker and the next
    /// sibling marker (or the next deeper-level child marker,
    /// whichever comes first). May be empty if a clause has only
    /// children with no introductory body.
    pub text: SourceTextRef,
    /// Child clauses, in canonical Bluebook order.
    pub children: Vec<ClauseNode>,
}

/// The full parsed tree. The `root` node always carries the input's
/// root citation (e.g., the statute section like § 1514A); its
/// children are the top-level subsections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClauseTree {
    pub root: ClauseNode,
}

/// Bluebook §3.3 label kinds, in canonical order of depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelKind {
    /// `(a)`, `(b)`, ... — depth 1 — `Subsection`.
    LowercaseLetter,
    /// `(1)`, `(2)`, `(99)` — depth 2 — `Paragraph`.
    ArabicNumeral,
    /// `(A)`, `(B)`, ... — depth 3 — `Subparagraph`.
    UppercaseLetter,
    /// `(i)`, `(ii)`, `(iii)`, ... — depth 4 — `Clause`.
    LowercaseRoman,
    /// `(I)`, `(II)`, ... — depth 5 — sub-clause (rare).
    UppercaseRoman,
}

impl LabelKind {
    /// Canonical depth (1-indexed, matches Bluebook nesting depth).
    pub fn depth(&self) -> usize {
        match self {
            Self::LowercaseLetter => 1,
            Self::ArabicNumeral => 2,
            Self::UppercaseLetter => 3,
            Self::LowercaseRoman => 4,
            Self::UppercaseRoman => 5,
        }
    }

    /// The [`PinpointCitationConcept`] this kind maps to.
    pub fn pinpoint_level(&self) -> PinpointCitationConcept {
        match self {
            Self::LowercaseLetter => PinpointCitationConcept::Subsection,
            Self::ArabicNumeral => PinpointCitationConcept::Paragraph,
            Self::UppercaseLetter => PinpointCitationConcept::Subparagraph,
            // Bluebook collapses depths ≥4 into the `Clause` concept;
            // the structural depth is still preserved in the tree.
            Self::LowercaseRoman | Self::UppercaseRoman => PinpointCitationConcept::Clause,
        }
    }

    /// Infer the label kind from the raw label string + current
    /// stack-depth context.
    ///
    /// Context is needed only for the lowercase ambiguous cases
    /// `(i)`, `(v)`, `(x)`: when the current open parent is a
    /// `Subparagraph` (depth 3), these resolve to `LowercaseRoman`
    /// (depth 4); otherwise they're treated as the rarer top-level
    /// `LowercaseLetter`.
    pub fn from_label(label: &str, parent_depth: usize) -> Option<Self> {
        if label.is_empty() {
            return None;
        }
        let chars: Vec<char> = label.chars().collect();

        // All digits → ArabicNumeral.
        if chars.iter().all(|c| c.is_ascii_digit()) {
            return Some(Self::ArabicNumeral);
        }

        // All uppercase letters: distinguish UppercaseRoman from
        // UppercaseLetter. Romans use I/V/X/L/C/D/M; single letters
        // outside that set are UppercaseLetter. Multi-char strictly
        // made of Roman digits in a valid Roman pattern is
        // UppercaseRoman.
        if chars.iter().all(|c| c.is_ascii_uppercase()) {
            if chars.len() == 1 {
                let c = chars[0];
                // Only I, V, X are ambiguous single-character romans
                // at SOX-statute depth; resolve by parent depth.
                if matches!(c, 'I' | 'V' | 'X') && parent_depth >= 3 {
                    return Some(Self::UppercaseRoman);
                }
                return Some(Self::UppercaseLetter);
            } else if chars.iter().all(is_roman_char_upper) {
                return Some(Self::UppercaseRoman);
            } else {
                return Some(Self::UppercaseLetter);
            }
        }

        // All lowercase letters: distinguish LowercaseRoman from
        // LowercaseLetter.
        if chars.iter().all(|c| c.is_ascii_lowercase()) {
            if chars.len() == 1 {
                let c = chars[0];
                if matches!(c, 'i' | 'v' | 'x') && parent_depth >= 3 {
                    return Some(Self::LowercaseRoman);
                }
                return Some(Self::LowercaseLetter);
            } else if chars.iter().all(is_roman_char_lower) {
                return Some(Self::LowercaseRoman);
            } else {
                return Some(Self::LowercaseLetter);
            }
        }

        // Mixed case or other → unrecognised.
        None
    }
}

/// A `(` is treated as a subdivision-marker opener only if it appears
/// at the start of a line — i.e., at byte offset 0, or preceded by
/// whitespace back to a newline (or start of text). This avoids
/// treating prose references like "subsection (a)" as markers; the
/// canonical statutory text always puts real subdivision markers at
/// line start.
fn is_line_leading(bytes: &[u8], idx: usize) -> bool {
    let mut k = idx;
    while k > 0 {
        k -= 1;
        let c = bytes[k];
        if c == b'\n' || c == b'\r' {
            return true;
        }
        if !c.is_ascii_whitespace() {
            return false;
        }
    }
    // Reached start of text without finding non-whitespace.
    true
}

fn is_roman_char_lower(c: &char) -> bool {
    matches!(c, 'i' | 'v' | 'x' | 'l' | 'c' | 'd' | 'm')
}

fn is_roman_char_upper(c: &char) -> bool {
    matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M')
}

/// Errors from `parse_statute_text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// A `(...)` marker contained an unrecognised label format (mixed
    /// case, special characters, or empty).
    InvalidLabel { offset: usize, label: String },
    /// A marker's inferred depth doesn't fit anywhere in the current
    /// stack — would skip a level without a parent. E.g., a `(A)` at
    /// position 0 with no prior `(letter)`/`(digit)` to nest under.
    DepthSkip {
        offset: usize,
        label: String,
        expected_parent_depth_at_most: usize,
    },
    /// An unbalanced `(` without a matching `)`.
    UnbalancedParen { offset: usize },
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidLabel { offset, label } => {
                write!(f, "invalid label `({})` at offset {}", label, offset)
            }
            Self::DepthSkip {
                offset,
                label,
                expected_parent_depth_at_most,
            } => write!(
                f,
                "label `({})` at offset {} would skip a level (no parent at depth ≤ {})",
                label, offset, expected_parent_depth_at_most
            ),
            Self::UnbalancedParen { offset } => {
                write!(f, "unbalanced `(` at offset {}", offset)
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Scanner: text → flat list of (label, body) entries
// ─────────────────────────────────────────────────────────────────────

/// One marker found in the source text, paired with the body text
/// that follows it (up to the next marker or EOF).
#[derive(Debug, Clone)]
struct ScannedMarker {
    label: String,
    /// Byte offset of the opening `(` of this marker.
    offset: usize,
    /// Byte range of the body text *after* this marker's `)` and
    /// before the next marker's `(` (or EOF).
    body: core::ops::Range<usize>,
}

/// Find all `(label)` subdivision markers in the text. Returns a
/// flat list in document order. Body ranges are computed so each
/// marker owns the text up to (but not including) the next marker
/// or EOF.
///
/// Returns the prefix text (before any marker) separately so the
/// caller can attach it to the root.
fn scan_markers(text: &str) -> Result<(String, Vec<ScannedMarker>), ParseError> {
    let bytes = text.as_bytes();
    let mut markers = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'(' && is_line_leading(bytes, i) {
            // Scan to closing `)`.
            let start = i;
            let mut j = i + 1;
            let mut label = String::new();
            while j < bytes.len() && bytes[j] != b')' {
                let c = bytes[j] as char;
                if !c.is_ascii_alphanumeric() {
                    // Not a subdivision marker — treat as ordinary text.
                    label.clear();
                    break;
                }
                label.push(c);
                j += 1;
            }
            // Must terminate with `)` and have a non-empty alnum label.
            if j < bytes.len() && bytes[j] == b')' && !label.is_empty() {
                // Accept as a marker.
                markers.push(ScannedMarker {
                    label,
                    offset: start,
                    body: (j + 1)..bytes.len(), // tentatively to EOF; fixed up below
                });
                i = j + 1;
            } else {
                // Not a marker — skip the `(` and continue.
                i = start + 1;
            }
        } else {
            i += 1;
        }
    }

    // Fix up body ranges: each marker's body ends where the next
    // marker's `(` begins.
    let n = markers.len();
    for k in 0..n {
        let body_start = markers[k].body.start;
        let body_end = if k + 1 < n {
            markers[k + 1].offset
        } else {
            bytes.len()
        };
        markers[k].body = body_start..body_end;
    }

    // Prefix text: everything before the first marker (or all text
    // if no markers).
    let prefix_end = markers.first().map(|m| m.offset).unwrap_or(bytes.len());
    let prefix = String::from_utf8_lossy(&bytes[..prefix_end])
        .trim()
        .to_string();

    Ok((prefix, markers))
}

// ─────────────────────────────────────────────────────────────────────
// Tree builder: flat scanned markers → ClauseTree
// ─────────────────────────────────────────────────────────────────────

/// Internal stack frame during tree construction. Holds the node
/// being built + its depth.
#[derive(Debug)]
struct Frame {
    node: ClauseNode,
    depth: usize,
}

/// Build a `ClauseTree` from scanned markers + their bodies.
fn build_tree(
    full_text: &str,
    prefix: String,
    markers: Vec<ScannedMarker>,
    root: PinpointCite,
    context_uri: &str,
) -> Result<ClauseTree, ParseError> {
    let root_node = ClauseNode {
        id: root,
        text: SourceTextRef::with_context(prefix, context_uri),
        children: Vec::new(),
    };

    let mut stack: Vec<Frame> = vec![Frame {
        node: root_node,
        depth: 0,
    }];

    for marker in markers {
        let parent_depth = stack.last().map(|f| f.depth).unwrap_or(0);
        let kind = LabelKind::from_label(&marker.label, parent_depth).ok_or_else(|| {
            ParseError::InvalidLabel {
                offset: marker.offset,
                label: marker.label.clone(),
            }
        })?;
        let new_depth = kind.depth();

        // Pop the stack until the top has depth `new_depth - 1`.
        // Each pop attaches the popped node as a child of the new
        // top.
        while let Some(top) = stack.last() {
            if top.depth < new_depth {
                break;
            }
            let popped = stack.pop().expect("stack non-empty in loop");
            let parent = stack.last_mut().ok_or(ParseError::DepthSkip {
                offset: marker.offset,
                label: marker.label.clone(),
                expected_parent_depth_at_most: new_depth - 1,
            })?;
            parent.node.children.push(popped.node);
        }

        // Now `stack.last().depth == new_depth - 1` (or stack is empty
        // — but we always keep the root at depth 0). If popping
        // emptied the stack we have a depth-skip error.
        let parent = stack.last().ok_or(ParseError::DepthSkip {
            offset: marker.offset,
            label: marker.label.clone(),
            expected_parent_depth_at_most: new_depth - 1,
        })?;

        // Skip if depth jumped (e.g., (A) without (a)(1) above).
        if parent.depth + 1 != new_depth {
            return Err(ParseError::DepthSkip {
                offset: marker.offset,
                label: marker.label.clone(),
                expected_parent_depth_at_most: new_depth - 1,
            });
        }

        // Build the new node's PinpointCite by extending parent's.
        let mut new_cite = parent.node.id.clone();
        new_cite.segments.push(PinpointSegment {
            level: kind.pinpoint_level(),
            label: marker.label.clone(),
        });

        let body_text = full_text[marker.body.start..marker.body.end]
            .trim()
            .to_string();

        let new_node = ClauseNode {
            id: new_cite,
            text: SourceTextRef::with_context(body_text, context_uri),
            children: Vec::new(),
        };

        stack.push(Frame {
            node: new_node,
            depth: new_depth,
        });
    }

    // Drain remaining stack: pop everything back into the root.
    while stack.len() > 1 {
        let popped = stack.pop().expect("stack has more than 1 frame");
        let parent = stack.last_mut().expect("root frame remains");
        parent.node.children.push(popped.node);
    }

    let root_frame = stack.pop().expect("root frame present");
    Ok(ClauseTree {
        root: root_frame.node,
    })
}

// ─────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────

/// Parse statute text into a typed `ClauseTree`. The `root`
/// `PinpointCite` identifies the section being parsed (e.g., a
/// 2-segment cite `(Title, "18") (Section, "1514A")`); the parsed
/// tree's root carries this citation, and each child clause extends
/// it with one additional segment per nesting level.
///
/// `context_uri` is attached to every `SourceTextRef` the parser
/// produces — typically a praxis-lock or local-file URI identifying
/// the canonical text fixture.
///
/// # Errors
///
/// - `InvalidLabel` if a `(...)` contains a label that isn't a
///   recognised Bluebook subdivision form.
/// - `DepthSkip` if a marker's depth would skip a level (e.g., `(A)`
///   without a `(letter)` and `(digit)` parent path).
/// - `UnbalancedParen` if an opening `(` has no matching `)`.
pub fn parse_statute_text(
    text: &str,
    root: PinpointCite,
    context_uri: &str,
) -> Result<ClauseTree, ParseError> {
    let (prefix, markers) = scan_markers(text)?;
    build_tree(text, prefix, markers, root, context_uri)
}

// ─────────────────────────────────────────────────────────────────────
// Tree helpers
// ─────────────────────────────────────────────────────────────────────

impl ClauseTree {
    /// Total node count, including the root.
    pub fn node_count(&self) -> usize {
        self.root.subtree_size()
    }

    /// Iterate every node in document (depth-first pre-order).
    pub fn iter_nodes(&self) -> impl Iterator<Item = &ClauseNode> {
        self.root.iter_subtree()
    }

    /// Find a node by its `PinpointCite`.
    pub fn find(&self, target: &PinpointCite) -> Option<&ClauseNode> {
        self.iter_nodes().find(|n| &n.id == target)
    }

    /// Maximum depth in the tree (root = 0, top-level subsections = 1, …).
    pub fn max_depth(&self) -> usize {
        self.root.max_depth_in_subtree(0)
    }
}

impl ClauseNode {
    fn subtree_size(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|c| c.subtree_size())
            .sum::<usize>()
    }

    fn iter_subtree(&self) -> ClauseSubtreeIter<'_> {
        ClauseSubtreeIter { stack: vec![self] }
    }

    fn max_depth_in_subtree(&self, mine: usize) -> usize {
        if self.children.is_empty() {
            mine
        } else {
            self.children
                .iter()
                .map(|c| c.max_depth_in_subtree(mine + 1))
                .max()
                .unwrap_or(mine)
        }
    }
}

/// Depth-first pre-order iterator over a `ClauseNode` subtree.
pub struct ClauseSubtreeIter<'a> {
    stack: Vec<&'a ClauseNode>,
}

impl<'a> Iterator for ClauseSubtreeIter<'a> {
    type Item = &'a ClauseNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        // Push children in reverse so they emit in document order.
        for child in node.children.iter().rev() {
            self.stack.push(child);
        }
        Some(node)
    }
}
