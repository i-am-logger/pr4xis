//! Statute-report generator — produces a single markdown document
//! consolidating all three audit layers (terms, lock relations,
//! composition cross-references) into a reviewable "praxis-
//! understanding" summary for a registered statute.
//!
//! Intended for human review (Praxis-validation step) and inclusion
//! in PR / commit narratives. The report renders:
//!
//! 1. Statute identification (name, version, canonical-text SHA-256,
//!    provenance).
//! 2. Structural-data counts (terms, relations).
//! 3. Bridge audit summary (matched / unmatched terms,
//!    covered / uncovered / orphan clauses).
//! 4. Relation-side bridge summary (lock-backed extracted candidates,
//!    composition-layer references for cross-statute phrases).
//! 5. Text-match breakdown (verbatim-in-body vs paraphrase) +
//!    heading-relation classification.
//! 6. Per-paraphrase + per-gap documentation pulled from the
//!    statute's `KNOWN_PARAPHRASES` / `KNOWN_GAPS` registries.
//!
//! The generator is statute-agnostic — caller supplies the canonical
//! text, root cite, structural data, CURIE mapper, and the
//! paraphrase/gap registries via a [`ReportContext`].

#[allow(unused_imports)]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::applied::data_provisioning::registry::StructuralData;
use crate::social::judicial::citation::PinpointCite;
use crate::social::judicial::statute_structure::bridge::{
    HeadingRelation, TermMatchResult, TextMatch, audit_lock_against_tree, classify_heading_vs_name,
};
use crate::social::judicial::statute_structure::parse_statute_text;
use crate::social::judicial::statute_structure::relation_extractor::extract_relations;
use crate::social::judicial::statute_structure::term_extractor::extract_terms;

// ─────────────────────────────────────────────────────────────────────
// Inputs
// ─────────────────────────────────────────────────────────────────────

/// One paraphrase registry entry. Mirrors per-statute
/// `KnownParaphrase` structs; the report-generator takes a flat
/// `&[ReportParaphrase]` slice so it stays statute-agnostic.
#[derive(Debug, Clone, Copy)]
pub struct ReportParaphrase {
    pub term_id: &'static str,
    pub canonical_subsection: &'static str,
    pub rationale: &'static str,
}

/// One known-gap registry entry. Mirrors per-statute `KnownGap`.
#[derive(Debug, Clone, Copy)]
pub struct ReportGap {
    pub term_id: &'static str,
    pub kind_name: &'static str,
    pub canonical_subsection: &'static str,
    pub note: &'static str,
    pub resolution_blocker: &'static str,
}

/// All inputs the report generator needs.
pub struct ReportContext<'a, F: Fn(&str) -> Option<Vec<String>>> {
    pub statute_name: &'a str,
    pub statute_version: &'a str,
    pub canonical_text: &'a str,
    pub canonical_sha256: &'a str,
    pub canonical_provenance: &'a str,
    pub root_cite: PinpointCite,
    pub structural: &'a StructuralData,
    pub curie_mapper: F,
    pub paraphrases: &'a [ReportParaphrase],
    pub gaps: &'a [ReportGap],
}

// ─────────────────────────────────────────────────────────────────────
// Generator
// ─────────────────────────────────────────────────────────────────────

/// Render the comprehensive markdown audit report.
pub fn generate_statute_report<F>(ctx: &ReportContext<F>) -> String
where
    F: Fn(&str) -> Option<Vec<String>>,
{
    let mut out = String::new();

    // Header.
    out.push_str(&format!(
        "# Praxis-understanding report: `{}@{}`\n\n",
        ctx.statute_name, ctx.statute_version
    ));
    out.push_str(&format!(
        "- Canonical text SHA-256: `{}`\n",
        ctx.canonical_sha256
    ));
    out.push_str(&format!("- Provenance: `{}`\n", ctx.canonical_provenance));
    out.push_str(&format!("- Lock terms: {}\n", ctx.structural.terms.len()));
    out.push_str(&format!(
        "- Lock relations: {}\n",
        ctx.structural.relations.len()
    ));
    out.push_str(&format!(
        "- Canonical text length: {} chars\n\n",
        ctx.canonical_text.len()
    ));

    // Parse + extract.
    let tree = match parse_statute_text(
        ctx.canonical_text,
        ctx.root_cite.clone(),
        &format!("praxis-lock://{}@{}", ctx.statute_name, ctx.statute_version),
    ) {
        Ok(t) => t,
        Err(e) => {
            out.push_str(&format!("**Parse error:** {e:?}\n"));
            return out;
        }
    };

    let extracted_terms = extract_terms(&tree);
    let extracted_relations = extract_relations(&tree);

    out.push_str(&format!("- Parsed clause nodes: {}\n", tree.node_count()));
    out.push_str(&format!("- Max parse depth: {}\n", tree.max_depth()));
    out.push_str(&format!(
        "- Extracted headings: {}\n",
        extracted_terms
            .iter()
            .filter(|t| t.heading.is_some())
            .count()
    ));
    out.push_str(&format!(
        "- Extracted relation candidates: {}\n\n",
        extracted_relations.len()
    ));

    // Bridge audit.
    let report = audit_lock_against_tree(ctx.structural, &tree, &ctx.curie_mapper);

    out.push_str("## Term-side bridge audit\n\n");
    out.push_str(&format!(
        "- Lock terms matched: {}\n",
        report.matched_term_count()
    ));
    out.push_str(&format!(
        "- Lock terms unmatched: {}\n",
        report.unmatched_term_count()
    ));
    out.push_str(&format!(
        "- Canonical clauses covered: {}\n",
        report.covered_clause_count()
    ));
    out.push_str(&format!(
        "- Canonical clauses uncovered: {}\n",
        report.uncovered_clause_count()
    ));
    out.push_str(&format!(
        "- Orphan clauses (no subtree coverage): {}\n\n",
        report.uncovered_orphan_clauses().len()
    ));

    // Text-match breakdown.
    let mut name_in_body = 0;
    let mut paraphrase = 0;
    let mut heading_agrees = 0;
    let mut heading_diverges = 0;
    let mut no_heading = 0;
    let mut name_by_id: alloc::collections::BTreeMap<String, String> = Default::default();
    for term in &ctx.structural.terms {
        name_by_id.insert(term.id.clone(), term.name.clone());
    }
    for r in &report.by_lock_term {
        if let TermMatchResult::Matched {
            text_match,
            canonical_heading,
            lock_term_id,
            ..
        } = r
        {
            match text_match {
                TextMatch::NameInBody => name_in_body += 1,
                TextMatch::Paraphrase => paraphrase += 1,
            }
            if let Some(lock_name) = name_by_id.get(lock_term_id) {
                match classify_heading_vs_name(lock_name, canonical_heading.as_deref()) {
                    HeadingRelation::HeadingAgrees => heading_agrees += 1,
                    HeadingRelation::HeadingDiverges => heading_diverges += 1,
                    HeadingRelation::NoHeading => no_heading += 1,
                }
            }
        }
    }
    out.push_str("### Text-match breakdown\n\n");
    out.push_str(&format!(
        "- Lock-name in body (substring): {}\n",
        name_in_body
    ));
    out.push_str(&format!(
        "- Paraphrase (no substring match): {}\n",
        paraphrase
    ));
    out.push_str("\n### Heading-relation breakdown\n\n");
    out.push_str(&format!(
        "- Heading agrees with lock-name: {}\n",
        heading_agrees
    ));
    out.push_str(&format!(
        "- Heading diverges from lock-name: {}\n",
        heading_diverges
    ));
    out.push_str(&format!("- No heading at clause: {}\n\n", no_heading));

    // Uncovered orphan clauses (real gaps).
    let orphans = report.uncovered_orphan_clauses();
    if !orphans.is_empty() {
        out.push_str("### Orphan canonical clauses (no subtree coverage)\n\n");
        for cite in &orphans {
            out.push_str(&format!("- `{}`\n", cite.to_bluebook()));
        }
        out.push('\n');
    }

    // Unmatched lock terms.
    let unmatched_ids = report.unmatched_lock_term_ids();
    if !unmatched_ids.is_empty() {
        out.push_str("### Unmatched lock terms\n\n");
        for id in &unmatched_ids {
            out.push_str(&format!("- `{id}`\n"));
        }
        out.push('\n');
    }

    // Relation-side audit summary.
    out.push_str("## Relation-side audit\n\n");
    out.push_str(&format!(
        "- Extracted relation candidates: {}\n",
        extracted_relations.len()
    ));
    for c in &extracted_relations {
        out.push_str(&format!(
            "  - `{}` *{:?}* `\"{}\"` → `{}`\n",
            c.from_cite.to_bluebook(),
            c.kind,
            c.phrase,
            if c.target_text.is_empty() {
                "(no target)"
            } else {
                &c.target_text
            }
        ));
    }
    out.push('\n');

    // Known paraphrases.
    if !ctx.paraphrases.is_empty() {
        out.push_str("## Documented paraphrases\n\n");
        out.push_str(&format!(
            "{} entries — practitioner shorthand for canonical subsections:\n\n",
            ctx.paraphrases.len()
        ));
        for p in ctx.paraphrases {
            out.push_str(&format!(
                "- **`{}`** @ `{}` — {}\n",
                p.term_id, p.canonical_subsection, p.rationale
            ));
        }
        out.push('\n');
    }

    // Known gaps.
    if !ctx.gaps.is_empty() {
        out.push_str("## Documented gaps requiring resolution\n\n");
        out.push_str(&format!("{} entries:\n\n", ctx.gaps.len()));
        for g in ctx.gaps {
            out.push_str(&format!(
                "- **`{}`** @ `{}` [{}]\n  - {}\n  - **blocker:** {}\n",
                g.term_id, g.canonical_subsection, g.kind_name, g.note, g.resolution_blocker
            ));
        }
        out.push('\n');
    }

    // Footer.
    out.push_str("---\n");
    out.push_str(&format!(
        "Generated by `pr4xis-domains::social::judicial::statute_structure::statute_report` for `{}@{}`.\n",
        ctx.statute_name, ctx.statute_version
    ));

    out
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applied::data_provisioning::registry::{
        StructuralData, StructuralRelation, StructuralTerm,
    };
    use crate::social::judicial::citation::ontology::PinpointCitationConcept;

    fn sox_root() -> PinpointCite {
        PinpointCite::new()
            .push(PinpointCitationConcept::Title, "18")
            .push(PinpointCitationConcept::Section, "1514A")
    }

    fn sox_curie_mapper(local: &str) -> Option<Vec<String>> {
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

    const SOX_CANONICAL: &str =
        include_str!("../../../../data/test_fixtures/statute_shape/sox_1514a_shape.txt");

    /// Minimal hand-built `StructuralData` for report-generator tests
    /// — three terms, one Composes relation. The report generator is
    /// statute-agnostic; this fixture exercises every section of the
    /// rendered markdown without depending on any specific statute's
    /// term count or wording.
    fn fixture_structural() -> StructuralData {
        StructuralData {
            description: "report-generator test fixture".into(),
            terms: vec![
                StructuralTerm {
                    id: "sox_1514a:a".into(),
                    name: "Covered Employer".into(),
                    definition: "Definition (a).".into(),
                    lemmas: Vec::new(),
                },
                StructuralTerm {
                    id: "sox_1514a:b2b".into(),
                    name: "Procedure".into(),
                    definition: "Definition (b)(2)(B).".into(),
                    lemmas: Vec::new(),
                },
                StructuralTerm {
                    id: "sox_1514a:b2b_iii".into(),
                    name: "Merits clause".into(),
                    definition: "Definition (b)(2)(B)(iii).".into(),
                    lemmas: Vec::new(),
                },
            ],
            relations: vec![StructuralRelation {
                from: "sox_1514a:b2b_iii".into(),
                to: "sox_1514a:b2b".into(),
                relation: "Composes".into(),
            }],
        }
    }

    #[test]
    fn report_renders_every_section() {
        let structural = fixture_structural();
        let ctx = ReportContext {
            statute_name: "sox_1514a",
            statute_version: "2002",
            canonical_text: SOX_CANONICAL,
            canonical_sha256: "a1a53fd9576443c176ac33dca7c88d8257a708c3c9c4b2680dff21ff76cf5d12",
            canonical_provenance: "training_reconstructed_2026-05-15",
            root_cite: sox_root(),
            structural: &structural,
            curie_mapper: sox_curie_mapper,
            paraphrases: &[ReportParaphrase {
                term_id: "sox_1514a:a",
                canonical_subsection: "(a)",
                rationale: "Covered Employer shorthand.",
            }],
            gaps: &[ReportGap {
                term_id: "sox_1514a:b2b",
                kind_name: "DefinitionDrift",
                canonical_subsection: "(b)(2)(B)",
                note: "Hand-coded definition drifts.",
                resolution_blocker: "PDF loader.",
            }],
        };

        let report = generate_statute_report(&ctx);

        assert!(report.contains("# Praxis-understanding report: `sox_1514a@2002`"));
        assert!(report.contains("Canonical text SHA-256"));
        assert!(report.contains("## Term-side bridge audit"));
        assert!(report.contains("### Text-match breakdown"));
        assert!(report.contains("### Heading-relation breakdown"));
        assert!(report.contains("## Relation-side audit"));
        assert!(report.contains("## Documented paraphrases"));
        assert!(report.contains("## Documented gaps requiring resolution"));
        // Counts come from the fixture (3 terms, 1 relation).
        assert!(report.contains("Lock terms: 3"));
        assert!(report.contains("Lock relations: 1"));
        assert!(report.contains("Covered Employer shorthand"));
        assert!(report.contains("DefinitionDrift"));
    }

    #[test]
    fn report_with_no_paraphrases_omits_section() {
        let structural = fixture_structural();
        let ctx = ReportContext {
            statute_name: "sox_1514a",
            statute_version: "2002",
            canonical_text: SOX_CANONICAL,
            canonical_sha256: "test",
            canonical_provenance: "test",
            root_cite: sox_root(),
            structural: &structural,
            curie_mapper: sox_curie_mapper,
            paraphrases: &[],
            gaps: &[],
        };
        let report = generate_statute_report(&ctx);
        assert!(!report.contains("## Documented paraphrases"));
        assert!(!report.contains("## Documented gaps requiring resolution"));
    }
}
