//! Tests for the USLM-derived `Statute` materialization of
//! 49 U.S.C. § 42121.

use crate::social::compliance::statutes::air21_42121::statute;

#[test]
fn returns_a_statute() {
    let s = statute();
    assert_eq!(s.name(), "air21_42121");
    assert_eq!(s.version(), "2010");
}

#[test]
fn description_references_uslm_source() {
    let s = statute();
    assert!(
        s.description().text.contains("/us/usc/t49/s42121"),
        "got: {:?}",
        s.description().text
    );
}

#[test]
fn terms_include_published_subsections() {
    let s = statute();
    // § 42121 has subsections (a)–(e) per the published statute.
    for sub in ["a", "b", "c", "d", "e"] {
        let curie = format!("air21_42121:{sub}");
        assert!(
            s.term_by_curie(&curie).is_some(),
            "subsection {curie} missing"
        );
    }
}

#[test]
fn burden_shifting_clauses_present() {
    // § 42121(b)(2)(B)(i)-(iv) — the four-clause burden-shifting
    // framework SOX § 1514A(b)(2)(C) imports by reference.
    let s = statute();
    for clause in ["i", "ii", "iii", "iv"] {
        let curie = format!("air21_42121:b_2_B_{clause}");
        assert!(
            s.term_by_curie(&curie).is_some(),
            "burden-shifting clause {curie} missing"
        );
    }
}

#[test]
fn idempotent_across_calls() {
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
    assert_eq!(ids.len(), n);
}

#[test]
fn all_relation_endpoints_resolve_to_terms() {
    let s = statute();
    let term_ids: hashbrown::HashSet<&str> = s.terms().iter().map(|t| t.id.value()).collect();
    for (i, r) in s.relations().iter().enumerate() {
        assert!(
            term_ids.contains(r.from.value()),
            "relation #{i}: from `{}` is not a term",
            r.from.value()
        );
        assert!(
            term_ids.contains(r.to.value()),
            "relation #{i}: to `{}` is not a term",
            r.to.value()
        );
    }
}

#[test]
fn all_term_curies_use_air21_prefix() {
    for t in statute().terms() {
        assert!(
            t.id.value().starts_with("air21_42121:"),
            "term id `{}` is not in air21_42121: namespace",
            t.id.value()
        );
    }
}

#[test]
fn every_term_has_non_empty_name_and_definition() {
    for t in statute().terms() {
        assert!(
            !t.name.text.is_empty(),
            "term {} has empty name",
            t.id.value()
        );
        assert!(
            !t.definition.text.is_empty(),
            "term {} has empty definition",
            t.id.value()
        );
    }
}

#[test]
fn every_term_carries_urn_provenance() {
    let s = statute();
    for t in s.terms() {
        let name_ctx = t.name.context_uri.as_deref().unwrap_or("");
        let def_ctx = t.definition.context_uri.as_deref().unwrap_or("");
        assert_eq!(name_ctx, "/us/usc/t49/s42121");
        assert_eq!(def_ctx, "/us/usc/t49/s42121");
    }
}

#[test]
fn term_count_meets_published_subsection_floor() {
    let n = statute().terms().len();
    assert!(
        n >= 5,
        "USLM-derived Statute should expose >=5 terms (one per subsection); got {n}"
    );
}
