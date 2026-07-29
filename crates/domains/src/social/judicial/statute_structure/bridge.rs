//! Bridge between `praxis.lock` structural data and a parsed
//! `ClauseTree` — produces a structured diff showing where the
//! hand-coded lock data and the parser's view of the canonical text
//! agree, paraphrase, or diverge.
//!
//! Closes the audit loop opened by `canonical_audit` modules: the
//! existing per-statute audits classify lock terms manually via
//! hand-curated `KNOWN_PARAPHRASES` and `KNOWN_GAPS` constants. The
//! bridge supplies the *machine* half — for every lock term, find
//! the matching parsed clause (or flag as unmatched); for every
//! parsed clause, find the covering lock term(s) (or flag as
//! uncovered). The per-statute audit interprets the diff against
//! its hand-curated classification.
//!
//! # API shape
//!
//! [`audit_lock_against_tree`] takes:
//! - `structural` — the lock's `StructuralData` (terms + relations).
//! - `tree` — the parsed `ClauseTree` from canonical text.
//! - `curie_mapper` — a per-statute function that maps a lock-term
//!   CURIE's local part to its subsection-path components. SOX's
//!   convention (`:1` → `["a", "1"]`) differs from AIR21's (`:a_1`
//!   → `["a", "1"]`) so each statute supplies its own mapper.
//!
//! Returns a [`BridgeReport`] with per-term and per-clause results.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::data_provisioning::registry::{StructuralData, StructuralRelation};
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::social::judicial::citation::PinpointCite;
use crate::social::judicial::statute_structure::parser::{ClauseNode, ClauseTree};
use crate::social::judicial::statute_structure::relation_extractor::{
    RelationCandidate, RelationKind,
};
use crate::social::judicial::statute_structure::term_extractor::extract_terms;

// ─────────────────────────────────────────────────────────────────────
// Result types
// ─────────────────────────────────────────────────────────────────────

/// The diff between a praxis.lock structural block and a parsed
/// canonical-text tree.
#[derive(Debug, Clone, Default)]
pub struct BridgeReport {
    pub by_lock_term: Vec<TermMatchResult>,
    pub by_clause: Vec<ClauseMatchResult>,
}

/// Per-lock-term result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermMatchResult {
    /// Lock term's CURIE maps to a clause in the parsed tree.
    Matched {
        lock_term_id: String,
        clause_cite: PinpointCite,
        text_match: TextMatch,
        /// Canonical heading detected at the matched clause via the
        /// term-extractor's `HEADING.--` recognition. `None` when no
        /// canonical heading was extracted at this node (typically
        /// inline list items at the deepest levels). When present,
        /// downstream audits can compare it against the lock-term
        /// name for tighter drift detection than the substring-based
        /// `TextMatch`.
        canonical_heading: Option<String>,
    },
    /// Lock term has no matching clause — flagged for review.
    Unmatched {
        lock_term_id: String,
        reason: UnmatchedReason,
    },
}

/// How the lock term's name/definition compares to the parsed clause's body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextMatch {
    /// Lock term's `name` appears verbatim as a substring of the
    /// clause's body text (case-insensitive). Strong evidence the
    /// hand-coded term tracks canonical text closely.
    NameInBody,
    /// Lock term's `name` doesn't appear in the clause body — the
    /// term is paraphrasing or summarising. Doesn't itself imply
    /// drift; many practitioner-shorthand names are intentional
    /// paraphrases (`Covered Employer`, `Prohibition on Retaliation`,
    /// etc.). The per-statute `KNOWN_PARAPHRASES` registry annotates
    /// these.
    Paraphrase,
}

/// Why a lock term was unmatched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnmatchedReason {
    /// The CURIE mapper returned `None` — the CURIE's local part
    /// doesn't parse into a subsection path.
    InvalidCurie,
    /// The mapped subsection path doesn't reach any node in the tree.
    SubsectionNotFoundInTree { attempted_path: Vec<String> },
}

/// Per-clause result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClauseMatchResult {
    /// One or more lock terms map to this clause.
    Covered {
        clause_cite: PinpointCite,
        lock_term_ids: Vec<String>,
    },
    /// No lock term maps to this clause. Could be intentional (the
    /// statute has detail not modeled in the lock) or a gap.
    Uncovered { clause_cite: PinpointCite },
}

impl BridgeReport {
    /// Count of lock terms that matched a clause.
    ///
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), not a bare
    /// `usize` — a count, the same typing discipline as
    /// `formal::mereology::counting::ontology::cardinality`.
    pub fn matched_term_count(&self) -> Quantity {
        let count = self
            .by_lock_term
            .iter()
            .filter(|r| matches!(r, TermMatchResult::Matched { .. }))
            .count();
        Quantity::from_unit(count as f64, &unit::UNITLESS)
    }

    /// Count of lock terms with no matching clause — see
    /// [`Self::matched_term_count`] for the typing note.
    pub fn unmatched_term_count(&self) -> Quantity {
        let count = self
            .by_lock_term
            .iter()
            .filter(|r| matches!(r, TermMatchResult::Unmatched { .. }))
            .count();
        Quantity::from_unit(count as f64, &unit::UNITLESS)
    }

    /// Count of clauses with at least one covering lock term — see
    /// [`Self::matched_term_count`] for the typing note.
    pub fn covered_clause_count(&self) -> Quantity {
        let count = self
            .by_clause
            .iter()
            .filter(|r| matches!(r, ClauseMatchResult::Covered { .. }))
            .count();
        Quantity::from_unit(count as f64, &unit::UNITLESS)
    }

    /// Count of clauses with no covering lock term — see
    /// [`Self::matched_term_count`] for the typing note.
    pub fn uncovered_clause_count(&self) -> Quantity {
        let count = self
            .by_clause
            .iter()
            .filter(|r| matches!(r, ClauseMatchResult::Uncovered { .. }))
            .count();
        Quantity::from_unit(count as f64, &unit::UNITLESS)
    }

    /// All unmatched lock-term CURIEs.
    pub fn unmatched_lock_term_ids(&self) -> Vec<&str> {
        self.by_lock_term
            .iter()
            .filter_map(|r| match r {
                TermMatchResult::Unmatched { lock_term_id, .. } => Some(lock_term_id.as_str()),
                _ => None,
            })
            .collect()
    }

    /// All uncovered clause cites.
    pub fn uncovered_clause_cites(&self) -> Vec<&PinpointCite> {
        self.by_clause
            .iter()
            .filter_map(|r| match r {
                ClauseMatchResult::Uncovered { clause_cite } => Some(clause_cite),
                _ => None,
            })
            .collect()
    }

    /// Returns true if `clause_cite` OR any clause descended from
    /// it has at least one covering lock term. Used to distinguish
    /// "umbrella" clauses (no direct lock term but child subsections
    /// are covered — usually fine) from genuine orphans (entire
    /// subtree has no lock-term coverage — usually a gap).
    pub fn is_subtree_covered(&self, clause_cite: &PinpointCite) -> bool {
        let prefix = &clause_cite.segments;
        for r in &self.by_clause {
            if let ClauseMatchResult::Covered {
                clause_cite: cc, ..
            } = r
            {
                if cc.segments.len() < prefix.len() {
                    continue;
                }
                let prefix_matches = cc
                    .segments
                    .iter()
                    .zip(prefix.iter())
                    .all(|(a, b)| a.label == b.label && a.level == b.level);
                if prefix_matches {
                    return true;
                }
            }
        }
        false
    }

    /// Uncovered clauses where NO clause in the subtree has a lock
    /// term — the real "orphan" gaps. Excludes umbrella clauses
    /// whose children are covered.
    pub fn uncovered_orphan_clauses(&self) -> Vec<&PinpointCite> {
        self.by_clause
            .iter()
            .filter_map(|r| match r {
                ClauseMatchResult::Uncovered { clause_cite } => {
                    if self.is_subtree_covered(clause_cite) {
                        None // umbrella — children covered
                    } else {
                        Some(clause_cite)
                    }
                }
                _ => None,
            })
            .collect()
    }

    /// Lock-term match result by CURIE.
    pub fn lock_term(&self, curie: &str) -> Option<&TermMatchResult> {
        self.by_lock_term.iter().find(|r| match r {
            TermMatchResult::Matched { lock_term_id, .. }
            | TermMatchResult::Unmatched { lock_term_id, .. } => lock_term_id == curie,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────
// Main entry point
// ─────────────────────────────────────────────────────────────────────

/// Run the bridge audit. See module docs for the parameter shapes.
pub fn audit_lock_against_tree(
    structural: &StructuralData,
    tree: &ClauseTree,
    curie_mapper: impl Fn(&str) -> Option<Vec<String>>,
) -> BridgeReport {
    // Collect every (lock_term_id, parsed_path_or_none) pair so we
    // can build both per-term and per-clause views.
    let root_segment_count = tree.root.id.segments.len();

    // Pre-compute extracted headings per clause cite (Bluebook form
    // of the cite) for the new canonical_heading field on Matched
    // results.
    let extracted = extract_terms(tree);
    let heading_by_cite: alloc::collections::BTreeMap<String, String> = extracted
        .into_iter()
        .filter_map(|t| t.heading.map(|h| (t.cite.to_bluebook(), h)))
        .collect();

    let mut term_results: Vec<TermMatchResult> = Vec::with_capacity(structural.terms.len());
    // Map of clause-path → list of lock-term CURIEs that map there.
    let mut clause_to_terms: alloc::collections::BTreeMap<Vec<String>, Vec<String>> =
        Default::default();

    for term in &structural.terms {
        let term_id = &term.id;
        // Extract the CURIE local part (everything after the first `:`).
        let local = term_id
            .split_once(':')
            .map(|(_, local)| local)
            .unwrap_or(term_id.as_str());

        let path = match curie_mapper(local) {
            Some(p) => p,
            None => {
                term_results.push(TermMatchResult::Unmatched {
                    lock_term_id: term_id.clone(),
                    reason: UnmatchedReason::InvalidCurie,
                });
                continue;
            }
        };

        // Navigate the tree by the path.
        let found = navigate_path(&tree.root, &path);
        match found {
            Some(node) => {
                let text_match = if lock_name_in_body(&term.name, &node.text.text) {
                    TextMatch::NameInBody
                } else {
                    TextMatch::Paraphrase
                };
                let canonical_heading = heading_by_cite.get(&node.id.to_bluebook()).cloned();
                term_results.push(TermMatchResult::Matched {
                    lock_term_id: term_id.clone(),
                    clause_cite: node.id.clone(),
                    text_match,
                    canonical_heading,
                });
                clause_to_terms
                    .entry(path)
                    .or_default()
                    .push(term_id.clone());
            }
            None => {
                term_results.push(TermMatchResult::Unmatched {
                    lock_term_id: term_id.clone(),
                    reason: UnmatchedReason::SubsectionNotFoundInTree {
                        attempted_path: path,
                    },
                });
            }
        }
    }

    // Build per-clause results: walk the tree and check coverage.
    let mut clause_results: Vec<ClauseMatchResult> = Vec::new();
    walk_clauses(
        &tree.root,
        root_segment_count,
        &mut Vec::new(),
        &clause_to_terms,
        &mut clause_results,
    );

    BridgeReport {
        by_lock_term: term_results,
        by_clause: clause_results,
    }
}

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

/// Navigate a `ClauseTree` from the root by matching subsection
/// labels at each level. Returns the matching node or `None`.
fn navigate_path<'a>(root: &'a ClauseNode, path: &[String]) -> Option<&'a ClauseNode> {
    let mut current = root;
    for label in path {
        let next = current
            .children
            .iter()
            .find(|c| c.id.segments.last().map(|s| s.label.as_str()) == Some(label.as_str()))?;
        current = next;
    }
    Some(current)
}

/// Walk every non-root clause and record whether it has lock-term
/// coverage.
fn walk_clauses(
    node: &ClauseNode,
    root_segment_count: usize,
    current_path: &mut Vec<String>,
    clause_to_terms: &alloc::collections::BTreeMap<Vec<String>, Vec<String>>,
    results: &mut Vec<ClauseMatchResult>,
) {
    for child in &node.children {
        let last_label = child
            .id
            .segments
            .last()
            .map(|s| s.label.clone())
            .unwrap_or_default();
        current_path.push(last_label);

        // Skip the root in per-clause results — only emit for real
        // subdivisions.
        if child.id.segments.len() > root_segment_count {
            let result = match clause_to_terms.get(current_path.as_slice()) {
                Some(ids) => ClauseMatchResult::Covered {
                    clause_cite: child.id.clone(),
                    lock_term_ids: ids.clone(),
                },
                None => ClauseMatchResult::Uncovered {
                    clause_cite: child.id.clone(),
                },
            };
            results.push(result);
        }

        walk_clauses(
            child,
            root_segment_count,
            current_path,
            clause_to_terms,
            results,
        );
        current_path.pop();
    }
}

/// Case-insensitive substring check.
fn lock_name_in_body(lock_name: &str, body: &str) -> bool {
    let needle = lock_name.to_lowercase();
    let haystack = body.to_lowercase();
    haystack.contains(&needle)
}

// ─────────────────────────────────────────────────────────────────────
// Relation-side bridge: extracted phrase candidates vs lock relations
// ─────────────────────────────────────────────────────────────────────

/// Per-extracted-candidate result of the relation-side bridge audit.
/// Mirrors `TermMatchResult` but for relations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractedRelationResult {
    /// Extracted phrase has a corresponding lock relation: same
    /// from-clause subsection path, and lock-relation kind
    /// corresponds to the extractor's `RelationKind`.
    LockBacked {
        candidate_index: usize,
        lock_relation: LockRelationRef,
    },
    /// Extracted phrase found a clause but no matching lock relation
    /// — potential gap (lock might be missing a relation visible in
    /// canonical text) OR an intentional non-modeled phrase.
    NoLockMatch {
        candidate_index: usize,
        from_path: Vec<String>,
        kind: RelationKind,
    },
}

/// A reference to a specific lock relation by `(from, to, kind)`.
/// Kind is the praxis.lock string spelling (PascalCase).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockRelationRef {
    pub from: String,
    pub to: String,
    pub kind: String,
}

/// The relation-side bridge audit report.
#[derive(Debug, Clone)]
pub struct RelationBridgeReport {
    pub by_extracted: Vec<ExtractedRelationResult>,
}

impl RelationBridgeReport {
    /// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), not a bare
    /// `usize` — see [`BridgeReport::matched_term_count`]'s note.
    pub fn lock_backed_count(&self) -> Quantity {
        let count = self
            .by_extracted
            .iter()
            .filter(|r| matches!(r, ExtractedRelationResult::LockBacked { .. }))
            .count();
        Quantity::from_unit(count as f64, &unit::UNITLESS)
    }

    /// See [`BridgeReport::matched_term_count`]'s note.
    pub fn no_match_count(&self) -> Quantity {
        let count = self
            .by_extracted
            .iter()
            .filter(|r| matches!(r, ExtractedRelationResult::NoLockMatch { .. }))
            .count();
        Quantity::from_unit(count as f64, &unit::UNITLESS)
    }

    /// Indices of extracted candidates that found no lock match.
    pub fn unmatched_candidate_indices(&self) -> Vec<usize> {
        self.by_extracted
            .iter()
            .filter_map(|r| match r {
                ExtractedRelationResult::NoLockMatch {
                    candidate_index, ..
                } => Some(*candidate_index),
                _ => None,
            })
            .collect()
    }
}

/// Match an extracted `RelationKind` to one or more lock-relation
/// kind strings. Returns the list of acceptable lock-kind strings
/// the extracted variant can correspond to.
pub fn lock_kinds_for(extracted: RelationKind) -> &'static [&'static str] {
    match extracted {
        // "shall be governed by/under" / "subject to" can manifest as
        // either Requires or ExhaustionRequiredFor in the lock.
        RelationKind::Requires => &["Requires", "ExhaustionRequiredFor"],
        RelationKind::AffirmativeDefenseTo => &["AffirmativeDefenseTo"],
        // Excludes has no clear 1:1 lock-relation analogue in the
        // current RelationType taxonomy; the closest is
        // AlternativeTo (mutual exclusion) or Negates.
        RelationKind::Excludes => &["AlternativeTo", "Negates"],
    }
}

/// Audit extracted relation candidates against the lock's
/// `StructuralRelation`s. For each candidate, find a lock relation
/// where:
/// - The lock relation's `from` CURIE maps (via `curie_mapper`) to
///   the candidate's `from_cite` subsection path.
/// - The lock relation's `kind` string is in
///   [`lock_kinds_for`]`(candidate.kind)`.
///
/// `root_segment_count` is the number of leading `PinpointCite`
/// segments (typically Title + Section) that come BEFORE the
/// subdivision-path portion. These are stripped before comparison
/// with the curie-mapped paths.
pub fn audit_extracted_relations_against_lock(
    extracted: &[RelationCandidate],
    structural: &StructuralData,
    curie_mapper: impl Fn(&str) -> Option<Vec<String>>,
    root_segment_count: usize,
) -> RelationBridgeReport {
    // Pre-compute (lock_from_subsection_path, lock_relation) pairs
    // for quick lookup.
    let lock_indexed: Vec<(Vec<String>, &StructuralRelation)> = structural
        .relations
        .iter()
        .filter_map(|rel| {
            let local = rel
                .from
                .split_once(':')
                .map(|(_, l)| l)
                .unwrap_or(rel.from.as_str());
            curie_mapper(local).map(|path| (path, rel))
        })
        .collect();

    let mut results: Vec<ExtractedRelationResult> = Vec::with_capacity(extracted.len());
    for (i, candidate) in extracted.iter().enumerate() {
        // Build the subsection path from the candidate's from_cite,
        // stripping the leading Title/Section segments.
        let candidate_path: Vec<String> = candidate
            .from_cite
            .segments
            .iter()
            .skip(root_segment_count)
            .map(|s| s.label.clone())
            .collect();

        let acceptable_lock_kinds = lock_kinds_for(candidate.kind);

        let matched = lock_indexed.iter().find(|(path, rel)| {
            paths_equal_case_insensitive(path, &candidate_path)
                && acceptable_lock_kinds.contains(&rel.relation.as_str())
        });

        match matched {
            Some((_, rel)) => results.push(ExtractedRelationResult::LockBacked {
                candidate_index: i,
                lock_relation: LockRelationRef {
                    from: rel.from.clone(),
                    to: rel.to.clone(),
                    kind: rel.relation.clone(),
                },
            }),
            None => results.push(ExtractedRelationResult::NoLockMatch {
                candidate_index: i,
                from_path: candidate_path,
                kind: candidate.kind,
            }),
        }
    }

    RelationBridgeReport {
        by_extracted: results,
    }
}

fn paths_equal_case_insensitive(a: &[String], b: &[String]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| x.eq_ignore_ascii_case(y))
}

/// Classification of how a lock-term name relates to its matched
/// clause's canonical heading. Returned by
/// [`classify_heading_vs_name`]; used by per-statute audits to
/// detect drift more precisely than the substring-based [`TextMatch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingRelation {
    /// Canonical heading and lock-name are equivalent
    /// (case-insensitive equality OR one is a substring of the
    /// other). Strongest match — the lock name is grounded in
    /// enacted statutory text.
    HeadingAgrees,
    /// Canonical heading exists but doesn't agree with the lock
    /// name (neither contains the other). Real divergence — lock
    /// name is fully paraphrased relative to enacted text.
    HeadingDiverges,
    /// No canonical heading detected at the matched clause. The
    /// audit falls back to body-substring matching for these.
    NoHeading,
}

/// Compare a lock-term name to a canonical heading and classify
/// their relationship. Case-insensitive; matches if either string
/// contains the other.
pub fn classify_heading_vs_name(
    lock_name: &str,
    canonical_heading: Option<&str>,
) -> HeadingRelation {
    let Some(heading) = canonical_heading else {
        return HeadingRelation::NoHeading;
    };
    let h = heading.to_lowercase();
    let n = lock_name.to_lowercase();
    let h_t = h.trim();
    let n_t = n.trim();
    if h_t == n_t || h_t.contains(n_t) || n_t.contains(h_t) {
        HeadingRelation::HeadingAgrees
    } else {
        HeadingRelation::HeadingDiverges
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applied::data_provisioning::registry::{StructuralRelation, StructuralTerm};
    use crate::social::judicial::citation::ontology::PinpointCitationConcept;
    use crate::social::judicial::statute_structure::parser::parse_statute_text;

    /// A dimensionless UNITLESS count quantity, for comparing against the
    /// `*_count` methods' typed return values in these tests.
    fn q(n: u32) -> Quantity {
        Quantity::from_unit(f64::from(n), &unit::UNITLESS)
    }

    fn root_cite() -> PinpointCite {
        PinpointCite::new()
            .push(PinpointCitationConcept::Title, "TEST")
            .push(PinpointCitationConcept::Section, "1")
    }

    fn make_term(id: &str, name: &str) -> StructuralTerm {
        StructuralTerm {
            id: id.to_string(),
            name: name.to_string(),
            definition: format!("def of {name}"),
            lemmas: Vec::new(),
        }
    }

    fn make_data(terms: Vec<StructuralTerm>) -> StructuralData {
        StructuralData {
            description: "test".to_string(),
            terms,
            relations: Vec::new() as Vec<StructuralRelation>,
        }
    }

    /// SOX-style mapper: ":a" → ["a"], ":1" → ["a", "1"], ":1a" → ["a", "1", "A"]
    fn sox_like_mapper(local: &str) -> Option<Vec<String>> {
        let stripped = local.find("_v").map(|i| &local[..i]).unwrap_or(local);
        let mut path = Vec::new();
        let chars: Vec<char> = stripped.chars().collect();
        let mut i = 0;
        if let Some(&c) = chars.first() {
            if c.is_ascii_alphabetic() {
                path.push(c.to_ascii_lowercase().to_string());
                i = 1;
            } else if c.is_ascii_digit() {
                path.push("a".to_string());
            }
        }
        while i < chars.len() {
            let c = chars[i];
            if c.is_ascii_digit() {
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
        Some(path)
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn matched_term_when_path_exists_in_tree() {
        let tree =
            parse_statute_text("(a) foo body text\n(b) bar body", root_cite(), "test://").unwrap();
        let data = make_data(vec![make_term("test:a", "foo")]);
        let report = audit_lock_against_tree(&data, &tree, sox_like_mapper);

        assert_eq!(report.matched_term_count(), q(1));
        let r = report.lock_term("test:a").expect("term in report");
        if let TermMatchResult::Matched {
            text_match: TextMatch::NameInBody,
            ..
        } = r
        {
            // "foo" appears in "foo body text"
        } else {
            panic!("expected NameInBody match, got {r:?}");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn paraphrase_when_name_not_in_body() {
        let tree = parse_statute_text("(a) prose without keyword", root_cite(), "test://").unwrap();
        let data = make_data(vec![make_term("test:a", "Practitioner Shorthand")]);
        let report = audit_lock_against_tree(&data, &tree, sox_like_mapper);

        let r = report.lock_term("test:a").expect("term in report");
        if let TermMatchResult::Matched {
            text_match: TextMatch::Paraphrase,
            ..
        } = r
        {
        } else {
            panic!("expected Paraphrase, got {r:?}");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn unmatched_when_path_not_in_tree() {
        let tree = parse_statute_text("(a) some text", root_cite(), "test://").unwrap();
        let data = make_data(vec![make_term("test:b", "missing")]);
        let report = audit_lock_against_tree(&data, &tree, sox_like_mapper);

        assert_eq!(report.unmatched_term_count(), q(1));
        let r = report.lock_term("test:b").expect("term in report");
        if let TermMatchResult::Unmatched {
            reason: UnmatchedReason::SubsectionNotFoundInTree { .. },
            ..
        } = r
        {
        } else {
            panic!("expected SubsectionNotFoundInTree, got {r:?}");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn uncovered_clause_when_no_lock_term_maps_to_it() {
        let tree = parse_statute_text("(a) first\n(b) second", root_cite(), "test://").unwrap();
        let data = make_data(vec![make_term("test:a", "first")]);
        let report = audit_lock_against_tree(&data, &tree, sox_like_mapper);

        // (a) covered, (b) uncovered.
        assert_eq!(report.covered_clause_count(), q(1));
        assert_eq!(report.uncovered_clause_count(), q(1));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn multiple_lock_terms_can_map_to_same_clause() {
        let tree = parse_statute_text("(a) body text", root_cite(), "test://").unwrap();
        let data = make_data(vec![
            make_term("test:a", "First aspect"),
            make_term("test:a_v2", "Second aspect"),
        ]);
        let report = audit_lock_against_tree(&data, &tree, sox_like_mapper);

        assert_eq!(report.matched_term_count(), q(2));
        // Single clause (a) covered by both terms.
        assert_eq!(report.covered_clause_count(), q(1));
        if let Some(ClauseMatchResult::Covered { lock_term_ids, .. }) = report.by_clause.first() {
            assert_eq!(lock_term_ids.len(), 2);
        } else {
            panic!("expected first clause to be Covered with 2 term IDs");
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn invalid_curie_mapper_returns_invalid_curie_reason() {
        let tree = parse_statute_text("(a) body", root_cite(), "test://").unwrap();
        let data = make_data(vec![make_term("test:weird", "Whatever")]);
        // Reject any CURIE that contains "weird".
        let mapper = |local: &str| -> Option<Vec<String>> {
            if local.contains("weird") {
                None
            } else {
                sox_like_mapper(local)
            }
        };
        let report = audit_lock_against_tree(&data, &tree, mapper);
        let r = report.lock_term("test:weird").expect("term in report");
        assert!(matches!(
            r,
            TermMatchResult::Unmatched {
                reason: UnmatchedReason::InvalidCurie,
                ..
            }
        ));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn deep_path_navigation() {
        // (a)(1)(A) path.
        let text = "(a) outer\n(1) middle\n(A) inner content";
        let tree = parse_statute_text(text, root_cite(), "test://").unwrap();
        let data = make_data(vec![make_term("test:1a", "content")]);
        let report = audit_lock_against_tree(&data, &tree, sox_like_mapper);

        let r = report.lock_term("test:1a").expect("term in report");
        if let TermMatchResult::Matched { clause_cite, .. } = r {
            // Path should be (a)(1)(A), 3 levels deep + root section.
            assert_eq!(clause_cite.segments.len(), 5); // Title + Section + a + 1 + A
            assert_eq!(clause_cite.segments.last().unwrap().label, "A");
        } else {
            panic!("expected match, got {r:?}");
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn empty_lock_data_produces_empty_term_results() {
        let tree = parse_statute_text("(a) body", root_cite(), "test://").unwrap();
        let data = make_data(Vec::new());
        let report = audit_lock_against_tree(&data, &tree, sox_like_mapper);
        assert!(report.by_lock_term.is_empty());
        assert_eq!(report.uncovered_clause_count(), q(1));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn tree_with_no_subdivisions_produces_no_clause_results() {
        let tree = parse_statute_text("Just prose, no markers.", root_cite(), "test://").unwrap();
        let data = make_data(vec![make_term("test:a", "Anything")]);
        let report = audit_lock_against_tree(&data, &tree, sox_like_mapper);
        // (a) doesn't exist in tree → unmatched.
        assert_eq!(report.unmatched_term_count(), q(1));
        // No subdivisions → no clause results.
        assert!(report.by_clause.is_empty());
    }

    // ── Canonical-heading population + classification ─────────────────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn canonical_heading_populated_when_heading_pattern_present() {
        let tree = parse_statute_text(
            "(a) MY HEADING.--body text follows.",
            root_cite(),
            "test://",
        )
        .unwrap();
        let data = make_data(vec![make_term("test:a", "Term Name")]);
        let report = audit_lock_against_tree(&data, &tree, sox_like_mapper);
        let r = report.lock_term("test:a").expect("term in report");
        if let TermMatchResult::Matched {
            canonical_heading, ..
        } = r
        {
            assert_eq!(canonical_heading.as_deref(), Some("MY HEADING"));
        } else {
            panic!("expected Matched, got {r:?}");
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn canonical_heading_none_when_no_pattern() {
        let tree =
            parse_statute_text("(a) just prose without separator", root_cite(), "test://").unwrap();
        let data = make_data(vec![make_term("test:a", "Whatever")]);
        let report = audit_lock_against_tree(&data, &tree, sox_like_mapper);
        let r = report.lock_term("test:a").expect("term in report");
        if let TermMatchResult::Matched {
            canonical_heading, ..
        } = r
        {
            assert_eq!(*canonical_heading, None);
        } else {
            panic!("expected Matched, got {r:?}");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn classify_heading_agrees_on_equal_case_insensitive() {
        assert_eq!(
            classify_heading_vs_name("Foo", Some("foo")),
            HeadingRelation::HeadingAgrees
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn classify_heading_agrees_when_heading_contains_lock_name() {
        assert_eq!(
            classify_heading_vs_name("Burdens of Proof", Some("BURDENS OF PROOF")),
            HeadingRelation::HeadingAgrees
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn classify_heading_agrees_when_lock_name_contains_heading() {
        assert_eq!(
            classify_heading_vs_name(
                "Burdens of Proof for District Court Actions",
                Some("BURDENS OF PROOF"),
            ),
            HeadingRelation::HeadingAgrees
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn classify_heading_diverges_when_disjoint() {
        assert_eq!(
            classify_heading_vs_name("Covered Employer", Some("WHISTLEBLOWER PROTECTION")),
            HeadingRelation::HeadingDiverges
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn classify_heading_no_heading_when_none() {
        assert_eq!(
            classify_heading_vs_name("Anything", None),
            HeadingRelation::NoHeading
        );
    }

    // ── Relation-side bridge tests ───────────────────────────────────

    use crate::social::judicial::citation::PinpointSegment;
    use crate::social::judicial::statute_structure::relation_extractor::RelationCandidate;

    fn make_rel(from: &str, to: &str, kind: &str) -> StructuralRelation {
        StructuralRelation {
            from: from.to_string(),
            to: to.to_string(),
            relation: kind.to_string(),
        }
    }

    fn make_candidate(path_labels: &[&str], kind: RelationKind, phrase: &str) -> RelationCandidate {
        let mut cite = PinpointCite::new()
            .push(PinpointCitationConcept::Title, "TEST")
            .push(PinpointCitationConcept::Section, "1");
        for (i, label) in path_labels.iter().enumerate() {
            let level = match i {
                0 => PinpointCitationConcept::Subsection,
                1 => PinpointCitationConcept::Paragraph,
                2 => PinpointCitationConcept::Subparagraph,
                _ => PinpointCitationConcept::Clause,
            };
            cite.segments.push(PinpointSegment {
                level,
                label: label.to_string(),
            });
        }
        RelationCandidate {
            from_cite: cite,
            kind,
            phrase: phrase.to_string(),
            offset_in_body: 0,
            target_text: "test target".to_string(),
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extracted_requires_matches_lock_requires() {
        let extracted = vec![make_candidate(
            &["b", "2", "A"],
            RelationKind::Requires,
            "shall be governed by",
        )];
        let mut data = make_data(Vec::new());
        data.relations = vec![make_rel("test:b2a", "test:other", "Requires")];

        let report = audit_extracted_relations_against_lock(
            &extracted,
            &data,
            sox_like_mapper,
            2, // Title + Section
        );
        assert_eq!(report.lock_backed_count(), q(1));
        assert_eq!(report.no_match_count(), q(0));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extracted_affirmative_defense_matches_lock_kind() {
        // Use SOX-style CURIEs (avoid roman-numeral ambiguity in the
        // mock sox_like_mapper). The AIR21 (b)(2)(B)(ii) case is
        // covered by the real-corpus test in canonical_audit.
        let extracted = vec![make_candidate(
            &["b", "1", "A"],
            RelationKind::AffirmativeDefenseTo,
            "Notwithstanding",
        )];
        let mut data = make_data(Vec::new());
        data.relations = vec![make_rel("test:b1a", "test:b1b", "AffirmativeDefenseTo")];

        let report = audit_extracted_relations_against_lock(&extracted, &data, sox_like_mapper, 2);
        assert_eq!(report.lock_backed_count(), q(1));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extracted_requires_no_match_when_path_differs() {
        let extracted = vec![make_candidate(
            &["b", "2", "A"],
            RelationKind::Requires,
            "shall be governed by",
        )];
        let mut data = make_data(Vec::new());
        // Lock relation FROM a different clause.
        data.relations = vec![make_rel("test:c1", "test:other", "Requires")];

        let report = audit_extracted_relations_against_lock(&extracted, &data, sox_like_mapper, 2);
        assert_eq!(report.no_match_count(), q(1));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extracted_requires_matches_exhaustion_required_for() {
        // lock_kinds_for(Requires) includes ExhaustionRequiredFor.
        let extracted = vec![make_candidate(
            &["b", "1"],
            RelationKind::Requires,
            "subject to",
        )];
        let mut data = make_data(Vec::new());
        data.relations = vec![make_rel("test:b1", "test:other", "ExhaustionRequiredFor")];

        let report = audit_extracted_relations_against_lock(&extracted, &data, sox_like_mapper, 2);
        assert_eq!(report.lock_backed_count(), q(1));
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn lock_kinds_for_requires() {
        let kinds = lock_kinds_for(RelationKind::Requires);
        assert!(kinds.contains(&"Requires"));
        assert!(kinds.contains(&"ExhaustionRequiredFor"));
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn lock_kinds_for_affirmative_defense_to() {
        let kinds = lock_kinds_for(RelationKind::AffirmativeDefenseTo);
        assert_eq!(kinds, &["AffirmativeDefenseTo"]);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn paths_equal_case_insensitive_basic() {
        assert!(paths_equal_case_insensitive(
            &["a".to_string(), "B".to_string()],
            &["A".to_string(), "b".to_string()]
        ));
        assert!(!paths_equal_case_insensitive(
            &["a".to_string()],
            &["a".to_string(), "1".to_string()]
        ));
    }
}
