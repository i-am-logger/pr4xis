//! Full Title 50 — War and National Defense. The Foreign Intelligence
//! Surveillance Act (FISA): § 1801 (definitions), § 1805 (electronic-
//! surveillance orders), § 1809 (criminal sanctions for surveillance not
//! authorized by statute), § 1810 (civil liability), § 1861 (the "Section
//! 215" business-records authority), § 1881a (the "Section 702" program).
//! Also the National Security Act of 1947 (§§ 3001 et seq.) and the War
//! Powers Resolution (§§ 1541 et seq.). The harassment-timeline tree's
//! surveillance / FBI-referral claims reach this title via the § 1809 /
//! § 1810 unlawful-surveillance remedies.
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
use praxis_corpus_tests::{UslmCorpus, load_uslm_corpus};

/// Title 50, parsed once. `None` if the title is not on disk (fresh checkout
/// before `pr4xis update`); each test then skips gracefully.
static TITLE_50: LazyLock<Option<UslmCorpus>> =
    LazyLock::new(|| load_uslm_corpus("legal/uscode/usc_title_50/usc_title_50-pl-119-90.xml"));

/// Borrow the shared corpus, or return early with a SKIP note when absent.
macro_rules! corpus_or_skip {
    () => {
        match &*TITLE_50 {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Title 50 USLM not on disk (run `pr4xis update`)");
                return;
            }
        }
    };
}

#[test]
fn full_title_50_parses_with_expected_section_count() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    assert_eq!(title.identifier, "/us/usc/t50");
    assert_eq!(title.number, 50);
    assert!(
        title
            .heading
            .to_ascii_uppercase()
            .contains("WAR AND NATIONAL DEFENSE"),
        "got heading: {:?}",
        title.heading
    );
    // Title 50 carries FISA, the National Security Act, the War Powers
    // Resolution, and the intelligence-community chapters. Lower bound
    // is conservative.
    assert!(
        title.sections.len() >= 400,
        "expected ≥400 sections, got {}",
        title.sections.len()
    );
}

#[test]
fn full_title_50_every_section_has_unique_identifier() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    axiom_section_identifiers_unique(title).expect(
        "SectionIdentifiersUnique-or-LRC-documented-duplicates must hold for full Title 50",
    );
}

#[test]
fn full_title_50_every_section_satisfies_every_axiom() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    axiom_every_section_has_num(title).expect("EverySectionHasNum must hold for full Title 50");
    axiom_every_container_has_identifier(title)
        .expect("EveryContainerHasIdentifier must hold for full Title 50");
    axiom_child_identifier_extends_parent(title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 50");
    axiom_hierarchy_strictly_nested(title)
        .expect("HierarchyStrictlyNested must hold for full Title 50");
    axiom_section_identifiers_unique(title)
        .expect("SectionIdentifiersUnique must hold for full Title 50");
    axiom_ref_hrefs_well_formed(title).expect("RefHrefsWellFormed must hold for full Title 50");
}

#[test]
fn full_title_50_every_section_lifts_to_statute() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
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
fn full_title_50_known_sections_present() {
    // Sentinel sections forming the harassment-timeline tree's
    // foreign-intelligence-surveillance backbone.
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let ids: std::collections::HashSet<&str> = title
        .sections
        .iter()
        .map(|s| s.identifier.as_str())
        .collect();

    // § 1801 — Definitions (FISA). Anchors the entire electronic-
    // surveillance framework.
    assert!(
        ids.contains("/us/usc/t50/s1801"),
        "§ 1801 (FISA definitions) missing"
    );

    // § 1805 — Issuance of order (FISA electronic-surveillance
    // warrant procedure before the FISA Court).
    assert!(
        ids.contains("/us/usc/t50/s1805"),
        "§ 1805 (FISA surveillance orders) missing"
    );

    // § 1809 — Criminal sanctions: it is an offense to engage in
    // electronic surveillance "under color of law except as
    // authorized" by statute. The provision a victim of unlawful
    // surveillance invokes.
    assert!(
        ids.contains("/us/usc/t50/s1809"),
        "§ 1809 (FISA criminal sanctions for unlawful surveillance) missing"
    );

    // § 1810 — Civil liability for an aggrieved person subjected to
    // unlawful electronic surveillance.
    assert!(
        ids.contains("/us/usc/t50/s1810"),
        "§ 1810 (FISA civil liability) missing"
    );

    // § 1861 — Access to certain business records and other tangible
    // things (the "Section 215" authority).
    assert!(
        ids.contains("/us/usc/t50/s1861"),
        "§ 1861 (FISA \"Section 215\" business records) missing"
    );

    // § 1881a — Procedures for targeting non-U.S. persons reasonably
    // believed to be outside the United States (the "Section 702"
    // program).
    assert!(
        ids.contains("/us/usc/t50/s1881a"),
        "§ 1881a (FISA \"Section 702\" program) missing"
    );

    // § 3001 — Short title (National Security Act of 1947). Heads the
    // chapter that created the CIA, NSC, and DNI.
    assert!(
        ids.contains("/us/usc/t50/s3001"),
        "§ 3001 (National Security Act of 1947) missing"
    );

    // § 1541 — Purpose and policy (War Powers Resolution).
    assert!(
        ids.contains("/us/usc/t50/s1541"),
        "§ 1541 (War Powers Resolution) missing"
    );
}

#[test]
fn full_title_50_codegen_and_runtime_agree_on_section_1809() {
    // § 1809 (FISA criminal sanctions for surveillance not authorized
    // by statute) is THE load-bearing § for the harassment-timeline
    // tree's unlawful-surveillance theory. Runtime XML loading
    // (M4.δ.7.a) must produce the same term set the build-time codegen
    // path would have.
    let UslmCorpus { xml, title } = corpus_or_skip!();
    let section = title
        .section("/us/usc/t50/s1809")
        .expect("§ 1809 must be present");
    let runtime_data = derive_structural("usc50_1809", section);
    let codegen_doc =
        parse_uslm_str(xml, "/us/usc/t50/s1809", "usc50_1809").expect("codegen parse");
    let runtime_ids: std::collections::HashSet<&str> =
        runtime_data.terms.iter().map(|t| t.id.as_str()).collect();
    let codegen_ids: std::collections::HashSet<&str> = codegen_doc
        .terms
        .iter()
        .filter(|t| t.id != "usc50_1809")
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        runtime_ids, codegen_ids,
        "codegen and runtime diverge on Title 50 § 1809"
    );
}
