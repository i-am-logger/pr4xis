//! Tests for the USLM-derived `Statute` materialization of
//! 18 U.S.C. § 1514A. The `statute()` accessor looks up the section
//! by URN against the unified `UsCode::loaded()` corpus; these tests
//! verify the resulting `Statute` has the structural shape SOX § 806
//! actually has, with provenance pinned to the section URN.

use crate::social::compliance::statutes::sox_1514a::statute;

#[test]
fn returns_a_statute() {
    let s = statute();
    assert_eq!(s.name(), "sox_1514a");
    assert_eq!(s.version(), "2002");
}

#[test]
fn description_references_uslm_source() {
    let s = statute();
    let desc = s.description().text.as_str();
    assert!(
        desc.contains("/us/usc/t18/s1514A"),
        "USLM-derived description should name the URN, got: {desc:?}"
    );
}

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

#[test]
fn idempotent_across_calls() {
    // The OnceLock guarantees the same instance pointer.
    let a = statute() as *const _;
    let b = statute() as *const _;
    assert_eq!(a, b);
}

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

#[test]
fn every_term_has_non_empty_id() {
    for t in statute().terms() {
        assert!(!t.id.value().is_empty(), "USLM-derived term has empty id");
    }
}

#[test]
fn description_uri_pins_to_uslm_source() {
    let s = statute();
    assert!(s.description().text.contains("/us/usc/t18/s1514A"));
}

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
