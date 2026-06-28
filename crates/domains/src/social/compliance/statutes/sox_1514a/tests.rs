//! Tests for the USLM-derived `Statute` materialization of
//! 18 U.S.C. § 1514A. The `statute()` accessor looks up the section
//! by URN against the unified `UsCode::loaded()` corpus; these tests
//! verify the resulting `Statute` has the structural shape SOX § 806
//! actually has, with provenance pinned to the section URN.

use crate::social::compliance::statutes::sox_1514a::statute;

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn returns_a_statute() {
    let s = statute();
    assert_eq!(s.name(), "sox_1514a");
    assert_eq!(s.version(), "2002");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn description_references_uslm_source() {
    let s = statute();
    let desc = s.description().text.as_str();
    assert!(
        desc.contains("/us/usc/t18/s1514A"),
        "USLM-derived description should name the URN, got: {desc:?}"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn terms_include_published_subsections() {
    // Top-level subsections (a)–(e) per the published statute.
    let s = statute();
    for sub in ["a", "b", "c", "d", "e"] {
        let curie = format!("sox_1514a:{sub}");
        assert!(
            s.term_by_curie(&curie).is_some(),
            "subsection {curie} missing from USLM-derived Statute"
        );
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn nested_subdivision_present() {
    // (a)(1)(A) — first protected-activity subparagraph.
    let s = statute();
    assert!(
        s.term_by_curie("sox_1514a:a_1_A").is_some(),
        "(a)(1)(A) missing"
    );
}

// Forest / no-dangling-relations / unique-CURIE invariants are
// already verified at the functor level — see
// `social::compliance::statutes::from_uslm::tests::axiom_*`.

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn idempotent_across_calls() {
    // The OnceLock guarantees the same instance pointer.
    let a = statute() as *const _;
    let b = statute() as *const _;
    assert_eq!(a, b);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn all_term_ids_are_unique() {
    let s = statute();
    let mut ids: alloc::vec::Vec<&str> = s.terms().iter().map(|t| t.id.value()).collect();
    ids.sort_unstable();
    let n = ids.len();
    ids.dedup();
    assert_eq!(
        ids.len(),
        n,
        "found duplicate term CURIEs in USLM-derived Statute"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn all_relation_endpoints_resolve_to_terms() {
    let s = statute();
    let term_ids: hashbrown::HashSet<&str> = s.terms().iter().map(|t| t.id.value()).collect();
    for (i, r) in s.relations().iter().enumerate() {
        assert!(
            term_ids.contains(r.from.value()),
            "USLM relation #{i}: from `{}` is not a term",
            r.from.value()
        );
        assert!(
            term_ids.contains(r.to.value()),
            "USLM relation #{i}: to `{}` is not a term",
            r.to.value()
        );
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn all_term_curies_use_sox_1514a_prefix() {
    for t in statute().terms() {
        assert!(
            t.id.value().starts_with("sox_1514a:"),
            "USLM-derived term id `{}` is not in sox_1514a: namespace",
            t.id.value()
        );
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn every_term_has_non_empty_name_and_definition() {
    // The from_uslm conversion falls back
    // <chapeau> → <content> → <heading> → URN-derived name, so
    // no term should ever have an empty name or definition.
    // Subsections that contain only nested children (no flat
    // chapeau/content) still produce a meaningful definition
    // anchored on their heading.
    for t in statute().terms() {
        assert!(
            !t.name.text.is_empty(),
            "USLM-derived term {} has empty name",
            t.id.value()
        );
        assert!(
            !t.definition.text.is_empty(),
            "USLM-derived term {} has empty definition",
            t.id.value()
        );
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn every_term_has_non_empty_id() {
    for t in statute().terms() {
        assert!(!t.id.value().is_empty(), "USLM-derived term has empty id");
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn description_uri_pins_to_uslm_source() {
    let s = statute();
    assert!(s.description().text.contains("/us/usc/t18/s1514A"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn every_term_carries_urn_provenance() {
    // URN provenance pushdown: every USLM-derived term's name and
    // definition cite the section URN as context_uri so downstream
    // consumers can trace each term back to the LRC source.
    let s = statute();
    for t in s.terms() {
        let name_ctx = t.name.context_uri.as_deref().unwrap_or("");
        let def_ctx = t.definition.context_uri.as_deref().unwrap_or("");
        assert_eq!(
            name_ctx,
            "/us/usc/t18/s1514A",
            "term {} name context_uri must be the section URN; got {name_ctx:?}",
            t.id.value()
        );
        assert_eq!(
            def_ctx,
            "/us/usc/t18/s1514A",
            "term {} definition context_uri must be the section URN; got {def_ctx:?}",
            t.id.value()
        );
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn description_carries_uslm_urn_context() {
    // Statute-level description's context_uri is the URN.
    let ctx = statute()
        .description()
        .context_uri
        .as_deref()
        .expect("USLM-derived description carries a context URI");
    assert_eq!(ctx, "/us/usc/t18/s1514A");
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn term_count_meets_published_subsection_floor() {
    // SOX § 1514A has five top-level subsections (a)–(e) plus
    // nested paragraphs/subparagraphs/clauses. The USLM-derived
    // count varies with the corpus's exact depth; at minimum,
    // we should have 5 subsection terms + a few nested ones.
    let n = statute().terms().len();
    assert!(
        n >= 5,
        "USLM-derived Statute should expose >=5 terms (one per subsection); got {n}"
    );
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn idempotent_across_construct_and_lookup() {
    // The functor's deterministic property: same source →
    // same term ordering, byte-for-byte.
    let a = statute();
    let b = statute();
    assert_eq!(a.terms().len(), b.terms().len());
    for (ta, tb) in a.terms().iter().zip(b.terms().iter()) {
        assert_eq!(ta.id.value(), tb.id.value());
    }
}

// =============================================================================
// Statute query-API invariants. These are statute-agnostic — every
// USLM-derived statute should satisfy them — but tested here against
// the SOX § 1514A data so we get coverage on a real corpus instance.
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn relations_from_returns_only_outgoing() {
    let s = statute();
    for r in s.relations() {
        let outgoing: alloc::vec::Vec<_> = s.relations_from(&r.from).collect();
        assert!(
            outgoing.iter().any(|other| core::ptr::eq(*other, r)),
            "relations_from({}) missing its own outgoing relation to {}",
            r.from.value(),
            r.to.value()
        );
        // Every outgoing relation must actually have `from == r.from`.
        for o in &outgoing {
            assert_eq!(
                o.from.value(),
                r.from.value(),
                "relations_from({}) returned relation with from={}",
                r.from.value(),
                o.from.value()
            );
        }
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn relations_to_returns_only_incoming() {
    let s = statute();
    for r in s.relations() {
        let incoming: alloc::vec::Vec<_> = s.relations_to(&r.to).collect();
        assert!(
            incoming.iter().any(|other| core::ptr::eq(*other, r)),
            "relations_to({}) missing its own incoming relation from {}",
            r.to.value(),
            r.from.value()
        );
        for i in &incoming {
            assert_eq!(
                i.to.value(),
                r.to.value(),
                "relations_to({}) returned relation with to={}",
                r.to.value(),
                i.to.value()
            );
        }
    }
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn relation_iteration_is_partition_consistent() {
    // Summing |relations_from(t)| over every term must equal the
    // total relation count. Same for relations_to. These are
    // partition checks on the relation set.
    let s = statute();
    let total = s.relations().len();
    let by_from: usize = s
        .terms()
        .iter()
        .map(|t| s.relations_from(&t.id).count())
        .sum();
    let by_to: usize = s
        .terms()
        .iter()
        .map(|t| s.relations_to(&t.id).count())
        .sum();
    assert_eq!(
        by_from, total,
        "sum of relations_from over terms ({by_from}) != total relations ({total})"
    );
    assert_eq!(
        by_to, total,
        "sum of relations_to over terms ({by_to}) != total relations ({total})"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn term_by_curie_finds_existing_terms() {
    // Pick a known existing CURIE and verify lookup succeeds.
    let s = statute();
    for sub in ["a", "b", "c", "d", "e"] {
        let curie = format!("sox_1514a:{sub}");
        let t = s
            .term_by_curie(&curie)
            .unwrap_or_else(|| panic!("term_by_curie({curie}) returned None"));
        assert_eq!(t.id.value(), curie);
    }
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn term_by_curie_returns_none_for_unknown() {
    let s = statute();
    assert!(s.term_by_curie("sox_1514a:zzz_not_real").is_none());
    assert!(s.term_by_curie("other_statute:a").is_none());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn term_by_id_and_term_by_curie_agree() {
    // For every term, looking it up by Identifier and by raw CURIE
    // string yields the same `&LegalTerm`.
    let s = statute();
    for t in s.terms() {
        let by_id = s.term_by_id(&t.id).expect("term resolves by id");
        let by_curie = s
            .term_by_curie(t.id.value())
            .expect("term resolves by curie");
        assert!(
            core::ptr::eq(by_id, by_curie),
            "term_by_id and term_by_curie disagree for {}",
            t.id.value()
        );
    }
}
