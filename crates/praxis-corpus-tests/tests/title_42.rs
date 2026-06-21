//! Full Title 42 (113 MB) — the LRC's largest U.S. Code title (civil-rights
//! statutes, Title VII, ADA, Medicare/Medicaid, Social Security, public health).
//!
//! The 113 MB XML is parsed ONCE into a process-shared [`LazyLock`] fixture;
//! every `#[test]` below borrows the same immutable [`UsCodeTitle`]. Run under
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

/// Title 42, parsed once. `None` if the giant is not on disk (fresh checkout
/// before `pr4xis update`); each test then HARD-FAILS via `corpus_or_fail!` (tests do not skip).
static TITLE_42: LazyLock<Option<UslmCorpus>> =
    LazyLock::new(|| load_uslm_corpus("legal/uscode/usc_title_42/usc_title_42-pl-119-90.xml"));

#[test]
fn parses_with_expected_section_count() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_42, "usc_title_42");
    assert_eq!(title.identifier, "/us/usc/t42");
    assert_eq!(title.number, 42);
    assert!(
        title
            .heading
            .to_ascii_uppercase()
            .contains("PUBLIC HEALTH AND WELFARE"),
        "got heading: {:?}",
        title.heading
    );
    // Title 42 is the LRC's largest title — the civil-rights statutes, Title
    // VII, ADA, Medicare, Medicaid, Social Security, public health (CDC/NIH/HHS),
    // and housing. The release point carries several thousand sections. Lower
    // bound is conservative.
    assert!(
        title.sections.len() >= 2000,
        "expected ≥2000 sections, got {}",
        title.sections.len()
    );
}

#[test]
fn every_section_has_unique_identifier() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_42, "usc_title_42");
    axiom_section_identifiers_unique(title).expect(
        "SectionIdentifiersUnique-or-LRC-documented-duplicates must hold for full Title 42",
    );
}

#[test]
fn every_section_satisfies_every_axiom() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_42, "usc_title_42");
    axiom_every_section_has_num(title).expect("EverySectionHasNum must hold for full Title 42");
    axiom_every_container_has_identifier(title)
        .expect("EveryContainerHasIdentifier must hold for full Title 42");
    axiom_child_identifier_extends_parent(title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 42");
    axiom_hierarchy_strictly_nested(title)
        .expect("HierarchyStrictlyNested must hold for full Title 42");
    axiom_section_identifiers_unique(title)
        .expect("SectionIdentifiersUnique must hold for full Title 42");
    axiom_ref_hrefs_well_formed(title).expect("RefHrefsWellFormed must hold for full Title 42");
}

#[test]
fn every_section_lifts_to_statute() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_42, "usc_title_42");
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
fn known_sections_present() {
    // Sentinel sections forming the harassment-timeline tree's
    // federal civil-rights backbone.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_42, "usc_title_42");
    let ids: std::collections::HashSet<&str> = title
        .sections
        .iter()
        .map(|s| s.identifier.as_str())
        .collect();

    // § 1981 — Equal rights under the law (Civil Rights Act of
    // 1866, R.S. § 1977). "All persons … shall have the same
    // right … to make and enforce contracts."
    assert!(
        ids.contains("/us/usc/t42/s1981"),
        "§ 1981 (equal rights / CRA 1866) missing"
    );

    // § 1981a — Damages in cases of intentional discrimination
    // in employment (Civil Rights Act of 1991). Adds
    // compensatory + punitive damages to Title VII claims.
    assert!(
        ids.contains("/us/usc/t42/s1981a"),
        "§ 1981a (CRA 1991 damages) missing"
    );

    // § 1983 — Civil action for deprivation of rights (Ku Klux
    // Klan Act of 1871, R.S. § 1979). The primary vehicle for
    // suing state actors for constitutional violations.
    assert!(
        ids.contains("/us/usc/t42/s1983"),
        "§ 1983 (deprivation of rights under color of law) missing"
    );

    // § 1985 — Conspiracy to interfere with civil rights;
    // § 1985(3) reaches private conspiracies motivated by
    // class-based animus.
    assert!(
        ids.contains("/us/usc/t42/s1985"),
        "§ 1985 (conspiracy to interfere with civil rights) missing"
    );

    // § 1988 — Proceedings in vindication of civil rights;
    // § 1988(b) is the attorney's-fees provision for the
    // foregoing civil-rights statutes.
    assert!(
        ids.contains("/us/usc/t42/s1988"),
        "§ 1988 (civil-rights attorney's fees) missing"
    );

    // § 2000e — Definitions (Title VII of the Civil Rights Act
    // of 1964). The introductory section of the employment-
    // discrimination framework; § 2000e-3 is its anti-
    // retaliation provision.
    assert!(
        ids.contains("/us/usc/t42/s2000e"),
        "§ 2000e (Title VII definitions) missing"
    );

    // § 12101 — Findings and purpose (Americans with
    // Disabilities Act of 1990). § 12203 is the ADA anti-
    // retaliation provision.
    assert!(
        ids.contains("/us/usc/t42/s12101"),
        "§ 12101 (ADA findings and purpose) missing"
    );

    // § 12203 — ADA prohibition against retaliation and
    // coercion.
    assert!(
        ids.contains("/us/usc/t42/s12203"),
        "§ 12203 (ADA anti-retaliation) missing"
    );
}

#[test]
fn codegen_and_runtime_agree_on_section_1983() {
    // § 1983 (deprivation of rights under color of state law,
    // the Ku Klux Klan Act of 1871) is THE load-bearing § for
    // the harassment-timeline tree's claims against state actors.
    // Runtime XML loading must produce the same term set the
    // build-time codegen path would have.
    let UslmCorpus { xml, title } = corpus_or_fail!(TITLE_42, "usc_title_42");
    let section = title
        .section("/us/usc/t42/s1983")
        .expect("§ 1983 must be present");
    let runtime_data = derive_structural("usc42_1983", section);
    let codegen_doc =
        parse_uslm_str(xml, "/us/usc/t42/s1983", "usc42_1983").expect("codegen parse");
    let runtime_ids: std::collections::HashSet<&str> =
        runtime_data.terms.iter().map(|t| t.id.as_str()).collect();
    let codegen_ids: std::collections::HashSet<&str> = codegen_doc
        .terms
        .iter()
        .filter(|t| t.id != "usc42_1983")
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        runtime_ids, codegen_ids,
        "codegen and runtime diverge on Title 42 § 1983"
    );
}
