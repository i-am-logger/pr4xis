//! Full Title 28 — Judiciary and Judicial Procedure. Carries federal-court
//! jurisdiction (§§ 1331/1391/1404), the federal statute of limitations
//! (§ 1658), the Rules Enabling Act (§ 2072), and the FRCP / FRE / FRAP / FRBP
//! appendices (28 U.S.C. App.) that populate Layer 3 of statute_understanding
//! with the typed legal-actor / evidence-rule vocabulary.
//!
//! The XML is parsed ONCE into a process-shared [`LazyLock`] fixture; every
//! `#[test]` below borrows the same immutable [`UsCodeTitle`]. Run under
//! `cargo test` (one process, thread-parallel), so the parse is paid once for
//! the whole file regardless of how many assertions touch it.

use std::sync::LazyLock;

use pr4xis::codegen::uslm::parse_uslm_str;
use pr4xis_domains::social::compliance::statutes::from_uslm::{
    derive_structural, from_uslm_section,
};
use pr4xis_domains::social::software::markup::xml::uslm::axioms::{
    axiom_child_identifier_extends_parent, axiom_every_container_has_identifier,
    axiom_every_section_has_num, axiom_hierarchy_strictly_nested, axiom_ref_hrefs_well_formed,
    axiom_section_identifiers_unique, section_identifier_to_statute_name,
};
use praxis_corpus_tests::{UslmCorpus, corpus_or_fail, load_uslm_corpus};

/// Title 28, parsed once. `None` if the giant is not on disk (fresh checkout
/// before `pr4xis update`); each test then HARD-FAILS via `corpus_or_fail!` (tests do not skip).
static TITLE_28: LazyLock<Option<UslmCorpus>> =
    LazyLock::new(|| load_uslm_corpus("legal/uscode/usc_title_28/usc_title_28-pl-119-90.xml"));

#[test]
fn full_title_28_parses_with_expected_section_count() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_28, "usc_title_28");
    assert_eq!(title.identifier, "/us/usc/t28");
    assert_eq!(title.number, 28);
    assert!(
        title.heading.to_ascii_uppercase().contains("JUDICIARY"),
        "got heading: {:?}",
        title.heading
    );
    // Title 28's core (chapters 1-180) carries ~700 numbered
    // sections; the FRCP / FRE / FRAP / FRBP appendices contribute
    // a few hundred more (each rule is a section in the USLM
    // projection). Lower bound is conservative.
    assert!(
        title.sections.len() >= 500,
        "expected ≥500 sections, got {}",
        title.sections.len()
    );
}

#[test]
fn full_title_28_every_section_has_unique_identifier() {
    // Title 28 carries one LRC-documented URN duplicate at § 1932
    // (Judicial Panel on Multidistrict Litigation AND Revocation of
    // earned release credit, both published under the same URN per
    // LRC editorial convention — see `axiom_section_identifiers_unique`).
    // Delegate the LRC-documented-duplicate accommodation to that
    // axiom rather than duplicating the logic here.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_28, "usc_title_28");
    axiom_section_identifiers_unique(title).expect(
        "SectionIdentifiersUnique-or-LRC-documented-duplicates must hold for full Title 28",
    );
}

#[test]
fn full_title_28_has_exactly_one_lrc_documented_duplicate_at_section_1932() {
    // Empirical confirmation of the LRC's editorial convention.
    // Title 28 at release point pl-119-90 carries exactly ONE URN
    // duplicate: § 1932 (Judicial Panel on Multidistrict Litigation
    // AND Revocation of earned release credit). Both occurrences
    // carry the LRC's "Another section 1932 is set out..." footnote.
    //
    // If a future LRC release adds more URN duplicates, this test
    // surfaces it so the audit doc can update — duplicates are
    // visible, not silent.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_28, "usc_title_28");
    let mut by_urn: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for s in &title.sections {
        *by_urn.entry(s.identifier.as_str()).or_insert(0) += 1;
    }
    let duplicates: Vec<(&&str, &usize)> = by_urn.iter().filter(|(_, n)| **n > 1).collect();
    assert_eq!(
        duplicates.len(),
        1,
        "expected exactly one LRC-documented URN duplicate (§ 1932), got {duplicates:?}"
    );
    let (dup_urn, dup_count) = duplicates[0];
    assert_eq!(*dup_urn, "/us/usc/t28/s1932");
    assert_eq!(*dup_count, 2);
}

#[test]
fn full_title_28_every_section_satisfies_every_axiom() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_28, "usc_title_28");

    axiom_every_section_has_num(title).expect("EverySectionHasNum must hold for full Title 28");
    axiom_every_container_has_identifier(title)
        .expect("EveryContainerHasIdentifier must hold for full Title 28");
    axiom_child_identifier_extends_parent(title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 28");
    axiom_hierarchy_strictly_nested(title)
        .expect("HierarchyStrictlyNested must hold for full Title 28");
    axiom_section_identifiers_unique(title)
        .expect("SectionIdentifiersUnique must hold for full Title 28");
    axiom_ref_hrefs_well_formed(title).expect("RefHrefsWellFormed must hold for full Title 28");
}

#[test]
fn full_title_28_every_section_lifts_to_statute() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_28, "usc_title_28");

    let mut failed = 0usize;
    let mut first_failure: Option<(String, String)> = None;
    for s in &title.sections {
        let name = section_identifier_to_statute_name(&s.identifier);
        if let Err(e) = from_uslm_section(&name, "pl-119-90", s) {
            failed += 1;
            if first_failure.is_none() {
                first_failure = Some((name, format!("{e}")));
            }
        }
    }
    if let Some((n, msg)) = first_failure {
        panic!(
            "{failed}/{} sections failed to lift to Statute; first: {n}: {msg}",
            title.sections.len()
        );
    }
    assert_eq!(failed, 0);
}

#[test]
fn full_title_28_known_sections_present() {
    // Sentinel sections directly relevant to the SOX whistleblower
    // case's federal-court procedural posture.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_28, "usc_title_28");
    let ids: std::collections::HashSet<&str> = title
        .sections
        .iter()
        .map(|s| s.identifier.as_str())
        .collect();

    // § 1331 — federal-question jurisdiction. The Texas-2026 SOX
    // case is in federal court under § 1331 (Sarbanes-Oxley § 806
    // / 18 U.S.C. § 1514A creates a federal cause of action;
    // jurisdiction lies under § 1331).
    assert!(
        ids.contains("/us/usc/t28/s1331"),
        "§ 1331 (federal-question jurisdiction) missing"
    );

    // § 1391 — venue. Determines where the SOX action could be
    // brought; the Texas-2026 case is in the Southern District of
    // Texas because the alleged violations occurred there (Aptiv
    // facility in Hutchins, TX) per § 1391(b)(2).
    assert!(ids.contains("/us/usc/t28/s1391"), "§ 1391 (venue) missing");

    // § 1404 — change of venue. The pattern of post-2020 SOX cases
    // includes defendant motions to transfer; § 1404(a) gives the
    // standard.
    assert!(
        ids.contains("/us/usc/t28/s1404"),
        "§ 1404 (change of venue) missing"
    );

    // § 1658 — federal statute of limitations. § 1658(a) is the
    // four-year catch-all SOL; § 1658(b) is the special SOL for
    // private-securities-fraud claims (2 years from discovery / 5
    // years from violation). SOX § 1514A has its own internal
    // 180-day administrative SOL at 18 U.S.C. § 1514A(b)(2)(D),
    // but § 1658(b) applies to the related Rule 10b-5 claim
    // (Merck & Co. v. Reynolds, 559 U.S. 633 (2010)).
    assert!(
        ids.contains("/us/usc/t28/s1658"),
        "§ 1658 (federal statute of limitations) missing"
    );

    // § 2072 — Rules Enabling Act. Authorizes the Supreme Court to
    // promulgate the Federal Rules of Civil Procedure / Evidence /
    // Appellate Procedure / Bankruptcy Procedure (which the LRC
    // codifies as 28 U.S.C. App.). The legal-actor / evidence-rule
    // vocabulary M5.D.1 loaded for Layer 3 comes from those rules.
    assert!(
        ids.contains("/us/usc/t28/s2072"),
        "§ 2072 (Rules Enabling Act) missing"
    );

    // § 1254 — Supreme Court certiorari jurisdiction. The route
    // a SOX § 1514A case takes to SCOTUS (e.g. Lawson v. FMR LLC,
    // 571 U.S. 429 (2014); Murray v. UBS Securities, LLC, 601 U.S.
    // 23 (2024) — both 28 U.S.C. § 1254(1) certiorari grants).
    assert!(
        ids.contains("/us/usc/t28/s1254"),
        "§ 1254 (SCOTUS certiorari) missing"
    );

    // § 1746 — unsworn declarations under penalty of perjury. The
    // form admissible at summary judgment per FRCP 56(c)(4) — the
    // mechanism by which the complainant submits sworn testimony
    // without a notary.
    assert!(
        ids.contains("/us/usc/t28/s1746"),
        "§ 1746 (unsworn declarations) missing"
    );

    // § 2112 — record on appeal from agencies (cross-referenced by
    // 15 U.S.C. § 77i for SEC appeals). Title 15's § 77i quotes
    // § 2112 directly; this confirms the cross-title reference
    // target loads.
    assert!(
        ids.contains("/us/usc/t28/s2112"),
        "§ 2112 (record on appeal) missing"
    );
}

#[test]
fn full_title_28_codegen_and_runtime_agree_on_section_1658() {
    // § 1658 (federal SOL) is the load-bearing § for SOX
    // collateral-statute-of-limitations analysis — Lawson v. FMR
    // LLC's discussion of which SOL applies to a § 1514A claim
    // turned on § 1658 vs SOX § 1514A's internal 180-day clock.
    let UslmCorpus { xml, title } = corpus_or_fail!(TITLE_28, "usc_title_28");
    let section = title
        .section("/us/usc/t28/s1658")
        .expect("§ 1658 must be present");
    let runtime_data = derive_structural("usc_t28_s1658", section);
    let codegen_doc =
        parse_uslm_str(xml, "/us/usc/t28/s1658", "usc_t28_s1658").expect("codegen parse");
    let runtime_ids: std::collections::HashSet<&str> =
        runtime_data.terms.iter().map(|t| t.id.as_str()).collect();
    let codegen_ids: std::collections::HashSet<&str> = codegen_doc
        .terms
        .iter()
        .filter(|t| t.id != "usc_t28_s1658")
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        runtime_ids, codegen_ids,
        "codegen and runtime diverge on Title 28 § 1658"
    );
}
