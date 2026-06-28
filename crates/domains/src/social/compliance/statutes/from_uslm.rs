//! USLM → Statute functor.
//!
//! Mirror of [`crate::cognitive::linguistics::english::English::from_wordnet`][english]
//! for legal statutes: given a parsed [`UsCodeSection`] (the runtime
//! USLM value), produce a typed [`Statute`].
//!
//! The functor is a thin adapter — it walks the section + every
//! nested subdivision, builds a
//! [`StructuralData`] in-memory, then routes
//! through the existing [`Statute::from_structural_with_context`]
//! validation pipeline with the section URN as the provenance URI.
//! This guarantees that XML-derived statutes pass the same CURIE-
//! validity / dangling-relation / unknown-relation-kind checks.
//!
//! [english]: crate::cognitive::linguistics::english::English::from_wordnet
//! [structural_data]: crate::applied::data_provisioning::registry::StructuralData
//!
//! # Term + relation derivation
//!
//! - **Statute identity** — the `<section>` itself is the statute,
//!   carried by [`Statute::name()`] and [`Statute::version()`].
//!   It is **not** a term — praxis CURIEs require a `prefix:local`
//!   shape, and the statute name alone has no local part. Top-
//!   level subsections compose into nothing within the statute.
//! - **Subdivision terms** — every `<subsection>` / `<paragraph>` /
//!   `<subparagraph>` / `<clause>` / `<subclause>` / `<item>` /
//!   `<subitem>` becomes a term whose CURIE is
//!   `statute_name:<path>` where `<path>` is the USLM identifier
//!   suffix below the section, joined with underscores.
//! - **Composes relations** — every nested subdivision (anything
//!   below the top level) emits one `Composes` relation pointing
//!   at its immediate parent subdivision (mereological: the parent
//!   has-a the child as a textual component). Top-level
//!   subsections have no Composes edge — they sit at the root of
//!   the statute's subdivision forest.
//!
//! Cross-references (`<ref href=...>`) are not lifted into typed
//! relations at this layer — the legal-ontology's `RelationType`
//! lacks a generic "references" variant. They remain accessible
//! through the source [`UsCodeSection::refs`] / per-subdivision
//! refs fields for downstream consumers that want them.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use super::statute::{Statute, StatuteConstructError};
use crate::applied::data_provisioning::registry::{
    StructuralData, StructuralRelation, StructuralTerm,
};
use crate::social::software::markup::xml::uslm::{UsCodeSection, UsCodeSubdivision};

/// Construct a [`Statute`] from a parsed [`UsCodeSection`].
///
/// `name` is the praxis-registry statute name (`"sox_1514a"`) used
/// as the CURIE prefix. `version` mirrors the praxis.toml version
/// field. Errors propagate from
/// [`Statute::from_structural_with_context`] — dangling Composes,
/// malformed CURIEs, or unknown relation kinds fail closed. The
/// section's URN is used as the provenance `context_uri` for every
/// derived `SourceTextRef`.
pub fn from_uslm_section(
    name: &str,
    version: &str,
    section: &UsCodeSection,
) -> Result<Statute, StatuteConstructError> {
    let data = derive_structural(name, section);
    let context_uri = section.identifier.as_str();
    Statute::from_structural_with_context(name, version, &data, context_uri)
}

/// Derive a [`StructuralData`] from a UsCodeSection. Exposed for
/// tests that want to inspect the intermediate shape without
/// rebuilding the validated [`Statute`].
pub fn derive_structural(name: &str, section: &UsCodeSection) -> StructuralData {
    let mut terms = Vec::new();
    let mut relations = Vec::new();
    let section_prefix = section.identifier.as_str();

    // Top-level subdivisions ((a), (b), …) have no parent within
    // the statute — they sit at the root of the subdivision forest.
    // The § itself is carried by Statute::name() / Statute::version().
    for child in &section.children {
        collect(
            child,
            name,
            section_prefix,
            None,
            &mut terms,
            &mut relations,
        );
    }

    StructuralData {
        description: format!("USLM source: {}", section.identifier),
        terms,
        relations,
    }
}

fn collect(
    d: &UsCodeSubdivision,
    statute_name: &str,
    section_prefix: &str,
    parent_curie: Option<&str>,
    terms: &mut Vec<StructuralTerm>,
    relations: &mut Vec<StructuralRelation>,
) {
    let id = identifier_to_curie(&d.identifier, section_prefix, statute_name);
    let name = d
        .heading
        .as_ref()
        .filter(|h| !h.is_empty())
        .cloned()
        .unwrap_or_else(|| derive_name_from_id(&id));
    // Definition falls back from <chapeau> → <content> → <heading> →
    // URN-derived name so it's never empty. A subsection with only
    // nested children (no chapeau / content) is structurally
    // characterized by its heading + child enumeration; using the
    // heading as the definition preserves that semantic anchor.
    let definition = {
        let raw = pick_definition_text(&d.chapeau, &d.content);
        if raw.is_empty() { name.clone() } else { raw }
    };

    terms.push(StructuralTerm {
        id: id.clone(),
        name,
        definition,
        lemmas: Vec::new(),
    });
    if let Some(parent) = parent_curie {
        relations.push(StructuralRelation {
            from: id.clone(),
            to: parent.to_string(),
            relation: "Composes".to_string(),
        });
    }

    for child in &d.children {
        collect(
            child,
            statute_name,
            section_prefix,
            Some(&id),
            terms,
            relations,
        );
    }
}

/// CURIE derivation from a USLM identifier URN.
///
/// `/us/usc/t18/s1514A/a/1/A` with prefix `/us/usc/t18/s1514A`
/// and statute name `sox_1514a` → `"sox_1514a:a_1_A"`.
///
/// Subdivisions only — the section root URN never reaches this
/// function because the § itself is represented by
/// `Statute::name()`, not by a term.
fn identifier_to_curie(identifier: &str, section_prefix: &str, statute_name: &str) -> String {
    let local = identifier
        .strip_prefix(section_prefix)
        .and_then(|s| s.strip_prefix('/'))
        .unwrap_or("");
    let joined = local.replace('/', "_");
    format!("{statute_name}:{joined}")
}

/// Fallback name when a container has no `<heading>`. Uses the
/// CURIE's local part formatted as a subdivision marker, e.g.
/// `sox_1514a:a_1_A` → `(a)(1)(A)`. For the root CURIE returns it
/// as-is — the statute's praxis name, not a subdivision marker.
fn derive_name_from_id(curie: &str) -> String {
    let Some(local) = curie.split(':').nth(1) else {
        return curie.to_string();
    };
    if local.is_empty() {
        return curie.to_string();
    }
    local
        .split('_')
        .map(|seg| format!("({seg})"))
        .collect::<Vec<_>>()
        .join("")
}

/// A container may carry either a `<chapeau>` (intro text before
/// children) or a `<content>` (leaf body), per USLM Schema. Pick
/// the first non-empty one as the definition.
fn pick_definition_text(chapeau: &Option<String>, content: &Option<String>) -> String {
    chapeau
        .clone()
        .filter(|s| !s.is_empty())
        .or_else(|| content.clone().filter(|s| !s.is_empty()))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::software::markup::xml::uslm::read_uslm_title;

    const SAMPLE_USLM: &str = r##"<section identifier="/us/usc/t18/s1514A"><num value="1514A">§ 1514A.</num><heading>Civil action to protect against retaliation in fraud cases</heading><subsection identifier="/us/usc/t18/s1514A/a"><num value="a">(a)</num><heading><inline class="small-caps">Whistleblower Protection</inline></heading><chapeau>No company may discriminate against an employee—</chapeau><paragraph identifier="/us/usc/t18/s1514A/a/1"><num value="1">(1)</num><chapeau>to provide information—</chapeau><subparagraph identifier="/us/usc/t18/s1514A/a/1/A"><num value="A">(A)</num><content>a Federal regulatory or law enforcement agency;</content></subparagraph><subparagraph identifier="/us/usc/t18/s1514A/a/1/B"><num value="B">(B)</num><content>any Member of Congress;</content></subparagraph></paragraph><paragraph identifier="/us/usc/t18/s1514A/a/2"><num value="2">(2)</num><content>to file a proceeding.</content></paragraph></subsection><subsection identifier="/us/usc/t18/s1514A/b"><num value="b">(b)</num><heading><inline class="small-caps">Enforcement Action</inline></heading><content>A person who alleges discharge may seek relief.</content></subsection></section>"##;

    fn sample_section() -> UsCodeSection {
        let title = read_uslm_title(SAMPLE_USLM).expect("parse");
        title.sections.into_iter().next().expect("one section")
    }

    /// Count the subdivisions in a section's subtree — every
    /// container below the § itself. The § is not counted (it's
    /// the statute identity, not a term).
    fn count_subdivisions_in(s: &UsCodeSection) -> usize {
        s.children
            .iter()
            .map(count_subdivisions_in_sub)
            .sum::<usize>()
    }

    fn count_subdivisions_in_sub(d: &UsCodeSubdivision) -> usize {
        1 + d
            .children
            .iter()
            .map(count_subdivisions_in_sub)
            .sum::<usize>()
    }

    /// Count subdivisions that are nested below the top level
    /// (i.e. have a parent within the statute's forest). These are
    /// the only ones that emit Composes edges.
    fn count_nested_subdivisions_in(s: &UsCodeSection) -> usize {
        s.children.iter().map(count_descendants).sum::<usize>()
    }

    fn count_descendants(d: &UsCodeSubdivision) -> usize {
        d.children
            .iter()
            .map(count_subdivisions_in_sub)
            .sum::<usize>()
    }

    // =========================================================
    // Layer 1 — unit tests
    // =========================================================

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn from_uslm_section_succeeds_on_sample() {
        let s = sample_section();
        let st = from_uslm_section("sox_1514a", "2002", &s).expect("functor");
        assert_eq!(st.name(), "sox_1514a");
        assert_eq!(st.version(), "2002");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn term_count_equals_subdivision_count() {
        let s = sample_section();
        let st = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        let expected = count_subdivisions_in(&s);
        assert_eq!(st.terms().len(), expected);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn relation_count_equals_nested_subdivision_count() {
        // Every subdivision below the top level emits one
        // Composes (its parent). Top-level subsections have no
        // parent within the statute.
        let s = sample_section();
        let st = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        let expected = count_nested_subdivisions_in(&s);
        assert_eq!(st.relations().len(), expected);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn no_root_term_only_subdivisions() {
        // The § itself isn't a term — `Statute::name()` carries
        // the identity. The bare statute name as a CURIE has no
        // local part and would be invalid.
        let s = sample_section();
        let st = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        assert!(st.term_by_curie("sox_1514a").is_none());
        for t in st.terms() {
            assert!(
                t.id.value().contains(':'),
                "term CURIE {} has no local part",
                t.id.value()
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn subdivision_curies_follow_path_convention() {
        let s = sample_section();
        let st = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        for curie in [
            "sox_1514a:a",
            "sox_1514a:b",
            "sox_1514a:a_1",
            "sox_1514a:a_2",
            "sox_1514a:a_1_A",
            "sox_1514a:a_1_B",
        ] {
            assert!(st.term_by_curie(curie).is_some(), "missing CURIE: {curie}");
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn fallback_name_used_when_heading_absent() {
        let s = sample_section();
        let st = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        let a_1_a = st.term_by_curie("sox_1514a:a_1_A").expect("(a)(1)(A) term");
        // No <heading> on the subparagraph in fixture; falls back
        // to derived subdivision marker.
        assert_eq!(a_1_a.name.text, "(a)(1)(A)");
    }

    // =========================================================
    // Layer 2 — axiom-equivalent invariants
    // =========================================================

    /// Axiom — every relation endpoint resolves to an existing
    /// term. `from_structural` enforces this; restated here to make
    /// the contract explicit.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_no_dangling_relations() {
        let s = sample_section();
        let st = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        let curies: std::collections::HashSet<&str> =
            st.terms().iter().map(|t| t.id.value()).collect();
        for r in st.relations() {
            assert!(
                curies.contains(r.from.value()),
                "dangling from {:?}",
                r.from
            );
            assert!(curies.contains(r.to.value()), "dangling to {:?}", r.to);
        }
    }

    /// Axiom — every term CURIE is unique within the statute.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_term_curies_unique() {
        let s = sample_section();
        let st = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        let mut seen = std::collections::HashSet::new();
        for t in st.terms() {
            assert!(
                seen.insert(t.id.value()),
                "duplicate CURIE: {}",
                t.id.value()
            );
        }
    }

    /// Axiom — Composes relations form a forest. Each non-root
    /// term has at most one outgoing Composes edge (its parent),
    /// and the root has zero.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_composes_is_forest_at_most_one_parent_per_term() {
        let s = sample_section();
        let st = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        let mut parent_count: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();
        for r in st.relations() {
            *parent_count.entry(r.from.value()).or_default() += 1;
        }
        for (curie, count) in &parent_count {
            assert!(
                *count <= 1,
                "term {curie} has {count} parents — Composes is not a forest"
            );
        }
    }

    /// Axiom — Composes graph is a forest where every chain
    /// terminates at a top-level subsection (one with no outgoing
    /// Composes edge). No term escapes its subdivision-tree root.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_every_term_reaches_a_top_level_subsection() {
        let s = sample_section();
        let st = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        let parent_of: std::collections::HashMap<&str, &str> = st
            .relations()
            .iter()
            .map(|r| (r.from.value(), r.to.value()))
            .collect();
        let top_level: std::collections::HashSet<&str> = st
            .terms()
            .iter()
            .filter(|t| {
                let local = t.id.value().split(':').nth(1).unwrap_or("");
                !local.contains('_')
            })
            .map(|t| t.id.value())
            .collect();
        for t in st.terms() {
            let curie = t.id.value();
            let mut cur = curie;
            let mut hops = 0;
            while let Some(p) = parent_of.get(cur).copied() {
                cur = p;
                hops += 1;
                if hops > 32 {
                    panic!("Composes chain from {curie} did not terminate in 32 hops");
                }
            }
            assert!(
                top_level.contains(cur),
                "term {curie} chain terminated at {cur} which is not a top-level subsection"
            );
        }
    }

    /// Axiom — for every Composes relation, the child CURIE
    /// local-part strictly extends the parent's by exactly one new
    /// underscore-separated path segment.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_composes_curie_is_strict_extension() {
        let s = sample_section();
        let st = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        for r in st.relations() {
            let from = r.from.value();
            let to = r.to.value();
            let from_local = from.split(':').nth(1).unwrap_or("");
            let to_local = to.split(':').nth(1).unwrap_or("");
            assert!(!to_local.is_empty(), "{to} has no local part");
            assert!(
                from_local.starts_with(to_local),
                "{from} → {to}: from local-part must extend to local-part"
            );
            let suffix = &from_local[to_local.len()..];
            assert!(
                suffix.starts_with('_'),
                "{from} → {to}: extension must add `_<segment>`"
            );
            assert_eq!(
                suffix.matches('_').count(),
                1,
                "{from} → {to}: more than one new path segment"
            );
        }
    }

    /// Axiom — the functor is deterministic. Same input → byte-
    /// identical term & relation lists.
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn axiom_functor_is_deterministic() {
        let s = sample_section();
        let a = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        let b = from_uslm_section("sox_1514a", "2002", &s).unwrap();
        let a_ids: Vec<&str> = a.terms().iter().map(|t| t.id.value()).collect();
        let b_ids: Vec<&str> = b.terms().iter().map(|t| t.id.value()).collect();
        assert_eq!(a_ids, b_ids);
        let a_rels: Vec<(&str, &str)> = a
            .relations()
            .iter()
            .map(|r| (r.from.value(), r.to.value()))
            .collect();
        let b_rels: Vec<(&str, &str)> = b
            .relations()
            .iter()
            .map(|r| (r.from.value(), r.to.value()))
            .collect();
        assert_eq!(a_rels, b_rels);
    }

    // =========================================================
    // Layer 3 — proptest property-based
    // =========================================================

    use proptest::prelude::*;

    proptest! {
        /// Property — for any subset of arbitrary statute names,
        /// the functor returns the requested name verbatim in
        /// `Statute::name()` and `Statute::version()`.
        #[test]
        fn prop_name_version_round_trip(
            name in "[a-z][a-z0-9_]{0,15}",
            version in "[0-9]{4}",
        ) {
            let s = sample_section();
            let st = from_uslm_section(&name, &version, &s).unwrap();
            prop_assert_eq!(st.name(), &name);
            prop_assert_eq!(st.version(), &version);
        }

        /// Property — term count is stable across runs.
        #[test]
        fn prop_term_count_stable(seed in any::<u32>()) {
            let _ = seed;
            let s = sample_section();
            let a = from_uslm_section("sox_1514a", "2002", &s).unwrap();
            let b = from_uslm_section("sox_1514a", "2002", &s).unwrap();
            prop_assert_eq!(a.terms().len(), b.terms().len());
        }

        /// Property — adding statute_name as the CURIE prefix
        /// changes only the prefix, not the local-part shape.
        #[test]
        fn prop_curie_prefix_swappable(
            name1 in "[a-z][a-z0-9_]{0,15}",
            name2 in "[a-z][a-z0-9_]{0,15}",
        ) {
            let s = sample_section();
            let st1 = from_uslm_section(&name1, "2002", &s).unwrap();
            let st2 = from_uslm_section(&name2, "2002", &s).unwrap();
            // Same number of terms / relations.
            prop_assert_eq!(st1.terms().len(), st2.terms().len());
            prop_assert_eq!(st1.relations().len(), st2.relations().len());
            // Each local-part (after the `:`) matches between the two.
            for (t1, t2) in st1.terms().iter().zip(st2.terms()) {
                let l1 = t1.id.value().split(':').nth(1).unwrap_or("");
                let l2 = t2.id.value().split(':').nth(1).unwrap_or("");
                prop_assert_eq!(l1, l2);
            }
        }

        /// Property — no dangling relations. Restated across many
        /// (name, version) seeds.
        #[test]
        fn prop_no_dangling_relations(
            name in "[a-z][a-z0-9_]{0,15}",
        ) {
            let s = sample_section();
            let st = from_uslm_section(&name, "2002", &s).unwrap();
            let curies: std::collections::HashSet<&str> =
                st.terms().iter().map(|t| t.id.value()).collect();
            for r in st.relations() {
                prop_assert!(curies.contains(r.from.value()));
                prop_assert!(curies.contains(r.to.value()));
            }
        }

        /// Property — Composes is a forest (≤1 parent per node).
        #[test]
        fn prop_composes_is_forest(name in "[a-z][a-z0-9_]{0,15}") {
            let s = sample_section();
            let st = from_uslm_section(&name, "2002", &s).unwrap();
            let mut count: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for r in st.relations() {
                *count.entry(r.from.value().to_string()).or_default() += 1;
            }
            for (curie, n) in count {
                prop_assert!(n <= 1, "term {curie} has {n} parents");
            }
        }

        /// Property — every term's Composes chain terminates at
        /// a top-level subsection (a term whose CURIE local-part
        /// has no underscore).
        #[test]
        fn prop_every_chain_terminates_at_top_level(
            name in "[a-z][a-z0-9_]{0,15}",
        ) {
            let s = sample_section();
            let st = from_uslm_section(&name, "2002", &s).unwrap();
            let parent_of: std::collections::HashMap<String, String> = st
                .relations()
                .iter()
                .map(|r| (r.from.value().to_string(), r.to.value().to_string()))
                .collect();
            for t in st.terms() {
                let mut cur = t.id.value().to_string();
                let mut hops = 0;
                while let Some(p) = parent_of.get(&cur).cloned() {
                    cur = p;
                    hops += 1;
                    if hops > 32 {
                        return Err(proptest::test_runner::TestCaseError::fail(
                            format!("Composes chain from {} didn't terminate", t.id.value()),
                        ));
                    }
                }
                let local = cur.split(':').nth(1).unwrap_or("");
                prop_assert!(
                    !local.contains('_'),
                    "chain from {} terminated at {} which is not top-level",
                    t.id.value(), cur
                );
            }
        }
    }

    pr4xis::register_praxis_value!(prop_name_version_round_trip, Deterministic);
    pr4xis::register_praxis_value!(prop_term_count_stable, Deterministic);
    pr4xis::register_praxis_value!(prop_curie_prefix_swappable, Deterministic);
    pr4xis::register_praxis_value!(prop_no_dangling_relations, Verifiable);
    pr4xis::register_praxis_value!(prop_composes_is_forest, Verifiable);
    pr4xis::register_praxis_value!(prop_every_chain_terminates_at_top_level, Verifiable);

    // =========================================================
    // Real-corpus check — actual SOX § 1514A USLM slice
    // =========================================================

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn from_uslm_section_on_real_sox_1514a_slice() {
        // § 1514A is sliced out of the fetched `usc_title_18` corpus
        // (18 U.S.C. § 1514A), not a deleted standalone fixture. FAILS LOUD
        // when the corpus is absent — CI fetches it; tests do not skip.
        let section = crate::social::software::markup::xml::uslm::real_sox_1514a::section();
        let st = from_uslm_section("sox_1514a", "2002", &section).expect("functor");

        // Term count: 5 subsections + every paragraph,
        // subparagraph, clause beneath them. Real § 1514A has
        // ≥19 subdivisions total below the § itself (per the
        // M4.δ.1 USLM codegen test's ≥20 containers minus the §).
        assert!(
            st.terms().len() >= 19,
            "expected ≥19 subdivision terms; got {}",
            st.terms().len()
        );

        // The published subsections must all be present.
        for sub in ["a", "b", "c", "d", "e"] {
            assert!(
                st.term_by_curie(&format!("sox_1514a:{sub}")).is_some(),
                "missing {sub}"
            );
        }

        // Top-level subsections have no outgoing Composes edge
        // (they sit at the root of the subdivision forest).
        for sub in ["a", "b", "c", "d", "e"] {
            let curie = format!("sox_1514a:{sub}");
            let outgoing = st
                .relations()
                .iter()
                .filter(|r| r.from.value() == curie)
                .count();
            assert_eq!(outgoing, 0, "{curie} should have 0 outgoing Composes");
        }

        // A nested subdivision like (a)(1)(A) must compose into
        // its immediate parent (a)(1).
        let edge_a_1_a_to_a_1 = st.relations().iter().any(|r| {
            r.from.value() == "sox_1514a:a_1_A"
                && r.to.value() == "sox_1514a:a_1"
                && matches!(
                    r.relation,
                    crate::social::judicial::ontology::RelationType::Composes { .. }
                )
        });
        assert!(edge_a_1_a_to_a_1, "(a)(1)(A) → (a)(1) edge missing");
    }
}
