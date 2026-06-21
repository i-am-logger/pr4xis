//! Full Title 5 — Government Organization and Employees. The
//! harassment-timeline tree's federal transparency / personnel-law
//! backbone: the Freedom of Information Act (§ 552 — the MuckRock FOIA
//! requests' legal basis), the Privacy Act of 1974 (§ 552a), the
//! Administrative Procedure Act (§§ 551 et seq.), and the federal
//! whistleblower scheme — § 2302(b)(8) (prohibited personnel practice:
//! reprisal for protected disclosures), § 1213 (disclosures to the Office
//! of Special Counsel), § 1221 (individual right of action before the
//! MSPB). Post-Pub. L. 117-286 (2022), Title 5 folds in the former
//! Appendix (Inspector General Act, Ethics in Government Act).
//!
//! The XML is parsed ONCE into a process-shared [`LazyLock`] fixture;
//! every `#[test]` below borrows the same immutable [`UsCodeTitle`]. Run
//! under `cargo test` (one process, thread-parallel), so the parse is paid
//! once for the whole file regardless of how many assertions touch it.

use std::sync::LazyLock;

use pr4xis::codegen::uslm::parse_uslm_str;
use pr4xis_domains::social::compliance::statutes::from_uslm::{
    derive_structural, from_uslm_section,
};
use pr4xis_domains::social::software::markup::xml::uslm::UsCodeSection;
use pr4xis_domains::social::software::markup::xml::uslm::axioms::{
    axiom_child_identifier_extends_parent, axiom_every_container_has_identifier,
    axiom_every_section_has_num, axiom_hierarchy_strictly_nested, axiom_ref_hrefs_well_formed,
    axiom_section_identifiers_unique, is_lrc_duplicate_numbering_footnote,
    section_identifier_to_statute_name,
};
use praxis_corpus_tests::{UslmCorpus, corpus_or_fail, load_uslm_corpus};

/// Title 5, parsed once. `None` if the giant is not on disk (fresh checkout
/// before `pr4xis update`); each test then HARD-FAILS via `corpus_or_fail!` (tests do not skip).
static TITLE_5: LazyLock<Option<UslmCorpus>> =
    LazyLock::new(|| load_uslm_corpus("legal/uscode/usc_title_5/usc_title_5-pl-119-90.xml"));

#[test]
fn full_title_5_parses_with_expected_section_count() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_5, "usc_title_5");
    assert_eq!(title.identifier, "/us/usc/t5");
    assert_eq!(title.number, 5);
    assert!(
        title
            .heading
            .to_ascii_uppercase()
            .contains("GOVERNMENT ORGANIZATION AND EMPLOYEES"),
        "got heading: {:?}",
        title.heading
    );
    // Title 5 carries the APA, FOIA/Privacy Act, the entire federal
    // civil-service personnel system, and (post-2022 reorganization)
    // the former Appendix. Lower bound is conservative.
    assert!(
        title.sections.len() >= 800,
        "expected ≥800 sections, got {}",
        title.sections.len()
    );
}

#[test]
fn full_title_5_every_section_has_unique_identifier() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_5, "usc_title_5");
    axiom_section_identifiers_unique(title)
        .expect("SectionIdentifiersUnique-or-LRC-documented-duplicates must hold for full Title 5");
}

#[test]
fn full_title_5_section_3598_is_identical_heading_lrc_duplicate() {
    // 5 U.S.C. § 3598 is the corpus's proof that the LRC publishes
    // duplicate section numbers with IDENTICAL headings — both
    // occurrences are headed "Federal Bureau of Investigation Reserve
    // Service". The only structural discriminator is the "Another
    // section 3598 is set out [after|preceding] this section" footnote
    // inside each <num>. This is why `axiom_section_identifiers_unique`
    // grounds on that footnote rather than on heading-distinctness
    // (which 28 U.S.C. § 1932 and 5 U.S.C. § 5757 satisfy but § 3598
    // does not). Regression guard for the M4.δ.8 finding.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_5, "usc_title_5");
    let occ: Vec<&UsCodeSection> = title
        .sections
        .iter()
        .filter(|s| s.identifier == "/us/usc/t5/s3598")
        .collect();
    assert_eq!(
        occ.len(),
        2,
        "expected exactly two § 3598 occurrences, got {}",
        occ.len()
    );
    // The heading does NOT disambiguate them — both are identical.
    assert_eq!(
        occ[0].heading, occ[1].heading,
        "the two § 3598 sections are expected to carry the identical heading"
    );
    assert!(
        occ[0]
            .heading
            .contains("Federal Bureau of Investigation Reserve Service"),
        "got heading {:?}",
        occ[0].heading
    );
    // The <num> footnote IS the disambiguator — present on both, with
    // one "after" and one "preceding".
    for s in &occ {
        let fnote = s.num_footnote.as_deref().unwrap_or("");
        assert!(
            is_lrc_duplicate_numbering_footnote(fnote),
            "§ 3598 occurrence missing LRC duplicate-numbering footnote; got {:?}",
            s.num_footnote
        );
    }
    let after = occ
        .iter()
        .any(|s| s.num_footnote.as_deref().unwrap_or("").contains("after"));
    let preceding = occ.iter().any(|s| {
        s.num_footnote
            .as_deref()
            .unwrap_or("")
            .contains("preceding")
    });
    assert!(
        after && preceding,
        "expected one 'after' and one 'preceding' duplicate-numbering footnote"
    );

    // A non-duplicated section carries NO num footnote — confirms the
    // footnote is specific to the duplicate-numbering convention and
    // not a generic artifact.
    let foia = title
        .section("/us/usc/t5/s552")
        .expect("§ 552 must be present");
    assert_eq!(
        foia.num_footnote, None,
        "ordinary (non-duplicated) § 552 should carry no <num> footnote"
    );
}

#[test]
fn full_title_5_every_section_satisfies_every_axiom() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_5, "usc_title_5");

    axiom_every_section_has_num(title).expect("EverySectionHasNum must hold for full Title 5");
    axiom_every_container_has_identifier(title)
        .expect("EveryContainerHasIdentifier must hold for full Title 5");
    axiom_child_identifier_extends_parent(title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 5");
    axiom_hierarchy_strictly_nested(title)
        .expect("HierarchyStrictlyNested must hold for full Title 5");
    axiom_section_identifiers_unique(title)
        .expect("SectionIdentifiersUnique must hold for full Title 5");
    axiom_ref_hrefs_well_formed(title).expect("RefHrefsWellFormed must hold for full Title 5");
}

#[test]
fn full_title_5_every_section_lifts_to_statute() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_5, "usc_title_5");

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
fn full_title_5_known_sections_present() {
    // Sentinel sections forming the harassment-timeline tree's
    // transparency + federal-whistleblower backbone.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_5, "usc_title_5");
    let ids: std::collections::HashSet<&str> = title
        .sections
        .iter()
        .map(|s| s.identifier.as_str())
        .collect();

    // § 552 — Public information; agency rules, opinions, orders,
    // records, and proceedings (the Freedom of Information Act). The
    // statutory basis for the harassment-timeline tree's 18 MuckRock
    // FOIA requests.
    assert!(ids.contains("/us/usc/t5/s552"), "§ 552 (FOIA) missing");

    // § 552a — Records maintained on individuals (the Privacy Act of
    // 1974). Governs agency records — directly relevant to the
    // surveillance / records-disclosure claims.
    assert!(
        ids.contains("/us/usc/t5/s552a"),
        "§ 552a (Privacy Act) missing"
    );

    // § 551 — Definitions (the Administrative Procedure Act). Anchors
    // the APA's judicial-review framework.
    assert!(
        ids.contains("/us/usc/t5/s551"),
        "§ 551 (APA definitions) missing"
    );

    // § 2301 — Merit system principles. The values the federal
    // personnel system must uphold; § 2302 enumerates the prohibited
    // practices that violate them.
    assert!(
        ids.contains("/us/usc/t5/s2301"),
        "§ 2301 (merit system principles) missing"
    );

    // § 2302 — Prohibited personnel practices. § 2302(b)(8) is the
    // core federal whistleblower-reprisal prohibition (protected
    // disclosures of violations of law, gross mismanagement, abuse of
    // authority, or substantial danger to public health/safety).
    assert!(
        ids.contains("/us/usc/t5/s2302"),
        "§ 2302 (prohibited personnel practices; (b)(8) whistleblower) missing"
    );

    // § 1213 — Provisions relating to disclosures of violations of
    // law, mismanagement, and danger to public health or safety. The
    // Office of Special Counsel disclosure channel.
    assert!(
        ids.contains("/us/usc/t5/s1213"),
        "§ 1213 (OSC disclosures) missing"
    );

    // § 1221 — Individual right of action in certain reprisal cases.
    // The whistleblower's IRA before the Merit Systems Protection
    // Board.
    assert!(
        ids.contains("/us/usc/t5/s1221"),
        "§ 1221 (MSPB individual right of action) missing"
    );
}

#[test]
fn full_title_5_codegen_and_runtime_agree_on_foia() {
    // § 552 (the Freedom of Information Act) is THE load-bearing §
    // for the harassment-timeline tree's FOIA work. Runtime XML
    // loading (M4.δ.7.a) must produce the same term set the build-
    // time codegen path would have.
    let UslmCorpus { xml, title } = corpus_or_fail!(TITLE_5, "usc_title_5");
    let section = title
        .section("/us/usc/t5/s552")
        .expect("§ 552 must be present");
    let runtime_data = derive_structural("usc5_552", section);
    let codegen_doc = parse_uslm_str(xml, "/us/usc/t5/s552", "usc5_552").expect("codegen parse");
    let runtime_ids: std::collections::HashSet<&str> =
        runtime_data.terms.iter().map(|t| t.id.as_str()).collect();
    let codegen_ids: std::collections::HashSet<&str> = codegen_doc
        .terms
        .iter()
        .filter(|t| t.id != "usc5_552")
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        runtime_ids, codegen_ids,
        "codegen and runtime diverge on Title 5 § 552"
    );
}
