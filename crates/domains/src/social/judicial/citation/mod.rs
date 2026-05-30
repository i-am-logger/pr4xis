//! Pinpoint citation — the hierarchical structure of a statutory or
//! regulatory cite (Title / Section / Subsection / Paragraph / Clause /
//! Subclause).
//!
//! U.S. statutory citation follows a stable hierarchy: a typical
//! whistleblower cite reads "18 U.S.C. § 1514A(b)(2)(D)" — Title 18,
//! Section 1514A, subsection (b), paragraph (2), subparagraph (D). The
//! The subdivision level hierarchy is defined by the loaded USLM schema
//! (the `<level>` element family); the Bluebook §3.3 codifies the
//! parenthesized citation-rendering convention.
//!
//! # Concept partition
//!
//! ```text
//! PinpointCitation
//!   ├── Title         — Title of the U.S. Code (e.g., "Title 18")
//!   ├── Section       — § N (e.g., "§ 1514A")
//!   ├── Subsection    — (a), (b), (c), ... (statutory)
//!   ├── Paragraph     — (1), (2), (3), ... (within a subsection)
//!   ├── Subparagraph  — (A), (B), (C), ... (within a paragraph)
//!   └── Clause        — (i), (ii), (iii), ... (within a subparagraph)
//! ```
//!
//! These are *not* opposed — they compose hierarchically. A full
//! citation is a `Vec<PinpointSegment>` where each segment carries a
//! typed concept level and a label (e.g., `Subsection` + `"b"`).
//!
//! # Literature
//!
//! - **USLM schema (Office of the Law Revision Counsel)** — the
//!   subdivision `<level>` element hierarchy (subsection / paragraph /
//!   subparagraph / clause / subclause / item). The machine-loaded
//!   praxis source for the structure (machine-verified).
//! - **The Bluebook: A Uniform System of Citation, 21st ed. (2020)**
//!   §3.2 (pinpoint citations), §3.3 (multiple subdivisions) — the
//!   parenthesized citation-rendering convention. LLM-checked (web).
//! - **House Office of the Legislative Counsel** *Manual on Drafting
//!   Style* §312(a) — Congressional drafting convention for nesting
//!   subsections / paragraphs / subparagraphs / clauses. LLM-checked (web).
//! - **ALWD Guide to Legal Citation, 7th ed. (2021)** ch. 14 — alternate
//!   practitioner citation standard. LLM-checked (web).

pub mod ontology;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
use alloc::{string::String, string::ToString, vec::Vec};

use self::ontology::PinpointCitationConcept;

/// One segment of a pinpoint citation: a typed level + its label string.
/// E.g., `(Subsection, "b")` means "subsection (b)".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PinpointSegment {
    pub level: PinpointCitationConcept,
    pub label: String,
}

/// A complete pinpoint citation: an ordered sequence of segments,
/// outermost-first. For "§ 1514A(b)(2)(D)" the segments are:
/// `[(Section, "1514A"), (Subsection, "b"), (Paragraph, "2"), (Subparagraph, "D")]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PinpointCite {
    pub segments: Vec<PinpointSegment>,
}

impl PinpointCite {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: push a segment.
    pub fn push(mut self, level: PinpointCitationConcept, label: impl Into<String>) -> Self {
        self.segments.push(PinpointSegment {
            level,
            label: label.into(),
        });
        self
    }

    /// Parse a Bluebook-§3.3 subdivision string like "(b)(2)(D)" or
    /// "(a)(1)(A)(ii)" into a citation. Returns segments tagged
    /// `Subsection` / `Paragraph` / `Subparagraph` / `Clause` according
    /// to position (outermost = Subsection).
    ///
    /// Returns `None` if the input contains characters outside
    /// `()_a-zA-Z0-9_` or has unmatched parentheses.
    pub fn parse_subdivisions(s: &str) -> Option<Self> {
        let mut cite = Self::new();
        let mut chars = s.chars().peekable();
        let mut levels = [
            PinpointCitationConcept::Subsection,
            PinpointCitationConcept::Paragraph,
            PinpointCitationConcept::Subparagraph,
            PinpointCitationConcept::Clause,
        ]
        .into_iter();
        while let Some(&c) = chars.peek() {
            if c == '(' {
                chars.next();
                let mut label = String::new();
                let mut closed = false;
                while let Some(&c) = chars.peek() {
                    if c == ')' {
                        chars.next();
                        closed = true;
                        break;
                    }
                    if c.is_alphanumeric() {
                        label.push(c);
                        chars.next();
                    } else {
                        return None;
                    }
                }
                if !closed || label.is_empty() {
                    return None;
                }
                let level = levels.next()?;
                cite.segments.push(PinpointSegment { level, label });
            } else {
                return None;
            }
        }
        Some(cite)
    }

    /// Render the citation in Bluebook §3.2 form (parenthesized subdivisions).
    /// E.g., `[(Subsection, "b"), (Paragraph, "2"), (Subparagraph, "D")]`
    /// renders as `"(b)(2)(D)"`.
    pub fn to_bluebook(&self) -> String {
        let mut out = String::new();
        for seg in &self.segments {
            out.push('(');
            out.push_str(&seg.label);
            out.push(')');
        }
        out
    }
}
