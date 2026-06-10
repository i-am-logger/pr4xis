//! Full Title 29 — Labor. Carries OSHA (§§ 651 et seq.; § 660(c) employee
//! anti-retaliation, the procedural backbone SOX § 1514A imports by reference),
//! the Fair Labor Standards Act (FLSA, §§ 201 et seq.), the Family and Medical
//! Leave Act (FMLA, §§ 2601 et seq.; § 2615 anti-retaliation), and the National
//! Labor Relations Act (NLRA, §§ 151 et seq.).
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

/// Title 29, parsed once. `None` if the title is not on disk (fresh checkout
/// before `pr4xis update`); each test then skips gracefully.
static TITLE_29: LazyLock<Option<UslmCorpus>> =
    LazyLock::new(|| load_uslm_corpus("legal/uscode/usc_title_29/usc_title_29-pl-119-90.xml"));

/// Borrow the shared corpus, or return early with a SKIP note when absent.
macro_rules! corpus_or_skip {
    () => {
        match &*TITLE_29 {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Title 29 USLM not on disk (run `pr4xis update`)");
                return;
            }
        }
    };
}

#[test]
fn full_title_29_parses_with_expected_section_count() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    assert_eq!(title.identifier, "/us/usc/t29");
    assert_eq!(title.number, 29);
    assert!(
        title.heading.to_ascii_uppercase().contains("LABOR"),
        "got heading: {:?}",
        title.heading
    );
    // Title 29 carries ~500-800 sections at this release point —
    // OSHA, FLSA, FMLA, NLRA, ERISA-labor, plus chapters on
    // labor-management relations, training, mine safety, and
    // related programs. Lower bound is conservative.
    assert!(
        title.sections.len() >= 400,
        "expected ≥400 sections, got {}",
        title.sections.len()
    );
}

#[test]
fn full_title_29_every_section_has_unique_identifier() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    axiom_section_identifiers_unique(title).expect(
        "SectionIdentifiersUnique-or-LRC-documented-duplicates must hold for full Title 29",
    );
}

#[test]
fn full_title_29_every_section_satisfies_every_axiom() {
    let UslmCorpus { title, .. } = corpus_or_skip!();

    axiom_every_section_has_num(title).expect("EverySectionHasNum must hold for full Title 29");
    axiom_every_container_has_identifier(title)
        .expect("EveryContainerHasIdentifier must hold for full Title 29");
    axiom_child_identifier_extends_parent(title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 29");
    axiom_hierarchy_strictly_nested(title)
        .expect("HierarchyStrictlyNested must hold for full Title 29");
    axiom_section_identifiers_unique(title)
        .expect("SectionIdentifiersUnique must hold for full Title 29");
    axiom_ref_hrefs_well_formed(title).expect("RefHrefsWellFormed must hold for full Title 29");
}

#[test]
fn full_title_29_every_section_lifts_to_statute() {
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
fn full_title_29_known_sections_present() {
    // Sentinel sections directly relevant to the SOX whistleblower
    // case's labor-law backbone.
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let ids: std::collections::HashSet<&str> = title
        .sections
        .iter()
        .map(|s| s.identifier.as_str())
        .collect();

    // § 651 — OSHA "Congressional statement of findings and
    // declaration of purpose and policy". The introductory section
    // of the Occupational Safety and Health Act.
    assert!(
        ids.contains("/us/usc/t29/s651"),
        "§ 651 (OSHA congressional findings) missing"
    );

    // § 652 — OSHA definitions.
    assert!(
        ids.contains("/us/usc/t29/s652"),
        "§ 652 (OSHA definitions) missing"
    );

    // § 654 — OSHA general duty clause: "Each employer shall
    // furnish to each of his employees employment and a place of
    // employment which are free from recognized hazards". The
    // statutory anchor for OSH-Act-based hazard claims.
    assert!(
        ids.contains("/us/usc/t29/s654"),
        "§ 654 (OSHA general duty clause) missing"
    );

    // § 660 — OSHA judicial review. § 660(c) is the original
    // employee anti-retaliation procedural framework — the one
    // SOX § 1514A(b)(2)(A) imports by reference via 49 U.S.C.
    // § 42121(b). Every SOX § 806 administrative complaint filed
    // with OSHA / OALJ runs on this procedural backbone.
    assert!(
        ids.contains("/us/usc/t29/s660"),
        "§ 660 (OSHA judicial review; § 660(c) anti-retaliation) missing"
    );

    // § 158 — NLRA § 8, unfair labor practices. The statutory
    // anti-retaliation framework that pre-dates OSHA § 11(c) and
    // shares its administrative-burden-shifting model.
    assert!(
        ids.contains("/us/usc/t29/s158"),
        "§ 158 (NLRA § 8 unfair labor practices) missing"
    );

    // FLSA § 15(a)(3) — anti-retaliation provision (29 U.S.C.
    // § 215(a)(3)). Predecessor anti-retaliation model to OSHA's.
    assert!(
        ids.contains("/us/usc/t29/s215"),
        "§ 215 (FLSA anti-retaliation) missing"
    );

    // § 2611 — FMLA definitions. FMLA's procedural framework is
    // structurally similar to OSHA's and frequently consolidated
    // with retaliation claims.
    assert!(
        ids.contains("/us/usc/t29/s2611"),
        "§ 2611 (FMLA definitions) missing"
    );

    // § 2615 — FMLA anti-retaliation ("Prohibited acts").
    assert!(
        ids.contains("/us/usc/t29/s2615"),
        "§ 2615 (FMLA prohibited acts) missing"
    );
}

#[test]
fn full_title_29_codegen_and_runtime_agree_on_section_660() {
    // § 660 (OSHA judicial review, including § 660(c) anti-
    // retaliation) is THE load-bearing § for the SOX § 1514A
    // procedural-framework-by-reference chain. SOX § 806 imports
    // OSHA's procedures via 49 U.S.C. § 42121(b), which references
    // back to OSHA's administrative scheme rooted here.
    let UslmCorpus { xml, title } = corpus_or_skip!();
    let section = title
        .section("/us/usc/t29/s660")
        .expect("§ 660 must be present");
    let runtime_data = derive_structural("osha_660", section);
    let codegen_doc = parse_uslm_str(xml, "/us/usc/t29/s660", "osha_660").expect("codegen parse");
    let runtime_ids: std::collections::HashSet<&str> =
        runtime_data.terms.iter().map(|t| t.id.as_str()).collect();
    let codegen_ids: std::collections::HashSet<&str> = codegen_doc
        .terms
        .iter()
        .filter(|t| t.id != "osha_660")
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        runtime_ids, codegen_ids,
        "codegen and runtime diverge on Title 29 § 660"
    );
}
