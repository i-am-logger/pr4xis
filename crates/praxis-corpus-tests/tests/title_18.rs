//! Full Title 18 — the U.S. Code's federal criminal code (CRIMES AND
//! CRIMINAL PROCEDURE), home of the SOX whistleblower statute § 1514A and
//! the underlying mail/wire/bank/securities-fraud offenses it protects
//! reporting of.
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

/// Title 18, parsed once. `None` if the title is not on disk (fresh checkout
/// before `pr4xis update`); each test then skips gracefully.
static TITLE_18: LazyLock<Option<UslmCorpus>> =
    LazyLock::new(|| load_uslm_corpus("legal/uscode/usc_title_18/usc_title_18-pl-119-90.xml"));

/// Borrow the shared corpus, or return early with a SKIP note when absent.
macro_rules! corpus_or_skip {
    () => {
        match &*TITLE_18 {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Title 18 USLM not on disk (run `pr4xis update`)");
                return;
            }
        }
    };
}

#[test]
fn full_title_18_parses_with_expected_section_count() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    assert_eq!(title.identifier, "/us/usc/t18");
    assert_eq!(title.number, 18);
    assert!(
        title.heading.contains("CRIMES"),
        "got heading: {:?}",
        title.heading
    );
    // LRC publishes ≥1,000 sections at this release point (Title 18
    // had 1,399 at pl-119-90). Lower bound is conservative — the
    // exact count drifts as Congress adds/repeals sections.
    assert!(
        title.sections.len() >= 1_000,
        "expected ≥1,000 sections, got {}",
        title.sections.len()
    );
}

#[test]
fn full_title_18_every_section_has_unique_identifier() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let mut seen = std::collections::HashSet::new();
    for s in &title.sections {
        assert!(
            seen.insert(&s.identifier),
            "duplicate identifier: {}",
            s.identifier
        );
    }
}

#[test]
fn full_title_18_every_section_satisfies_every_axiom() {
    let UslmCorpus { title, .. } = corpus_or_skip!();

    axiom_every_section_has_num(title).expect("EverySectionHasNum must hold for full Title 18");
    axiom_every_container_has_identifier(title)
        .expect("EveryContainerHasIdentifier must hold for full Title 18");
    axiom_child_identifier_extends_parent(title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 18");
    axiom_hierarchy_strictly_nested(title)
        .expect("HierarchyStrictlyNested must hold for full Title 18");
    axiom_section_identifiers_unique(title)
        .expect("SectionIdentifiersUnique must hold for full Title 18");
    axiom_ref_hrefs_well_formed(title).expect("RefHrefsWellFormed must hold for full Title 18");
}

#[test]
fn full_title_18_every_section_lifts_to_statute() {
    let UslmCorpus { title, .. } = corpus_or_skip!();

    // The from_uslm_section functor must succeed on every published
    // section — no statute should fail validation (no dangling
    // Composes, no CURIE collisions, no unknown relation kinds).
    let mut failed = 0usize;
    let mut first_failure: Option<(String, String)> = None;
    for (idx, s) in title.sections.iter().enumerate() {
        // Derive a statute_name from the section identifier:
        // `/us/usc/t18/s1514A` → `usc_t18_s1514a`.
        // Praxis CURIE names must be `[a-z][a-z0-9_]*`, so lowercase
        // and replace `/` with `_`.
        let name = section_identifier_to_statute_name(&s.identifier);
        match from_uslm_section(&name, "pl-119-90", s) {
            Ok(_) => {}
            Err(e) => {
                failed += 1;
                if first_failure.is_none() {
                    first_failure = Some((name, format!("idx {idx}: {e}")));
                }
            }
        }
    }
    if let Some((n, msg)) = first_failure {
        panic!(
            "{failed} of {} sections failed to lift to Statute; first: {n}: {msg}",
            title.sections.len()
        );
    }
    assert_eq!(failed, 0);
}

#[test]
fn full_title_18_known_sections_present() {
    // Sentinel sections this case actually cites — failure here means
    // the corpus drifted at the LRC's release point or our slicing
    // is wrong.
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let ids: std::collections::HashSet<&str> = title
        .sections
        .iter()
        .map(|s| s.identifier.as_str())
        .collect();

    // SOX whistleblower (the case's core statute).
    assert!(ids.contains("/us/usc/t18/s1514A"), "§ 1514A missing");

    // SOX § 802 — destruction of records.
    assert!(ids.contains("/us/usc/t18/s1519"), "§ 1519 missing");
    assert!(ids.contains("/us/usc/t18/s1520"), "§ 1520 missing");

    // Underlying-fraud statutes § 1514A protects reporting of.
    assert!(
        ids.contains("/us/usc/t18/s1341"),
        "§ 1341 (mail fraud) missing"
    );
    assert!(
        ids.contains("/us/usc/t18/s1343"),
        "§ 1343 (wire fraud) missing"
    );
    assert!(
        ids.contains("/us/usc/t18/s1344"),
        "§ 1344 (bank fraud) missing"
    );
    assert!(
        ids.contains("/us/usc/t18/s1348"),
        "§ 1348 (securities fraud) missing"
    );

    // Witness retaliation (related to whistleblower protection).
    assert!(ids.contains("/us/usc/t18/s1513"), "§ 1513 missing");
}

#[test]
fn full_title_18_codegen_and_runtime_agree_on_sox_1514a() {
    let UslmCorpus { xml, title } = corpus_or_skip!();
    let sox_section = title
        .section("/us/usc/t18/s1514A")
        .expect("§ 1514A must be present");
    let runtime_data = derive_structural("sox_1514a", sox_section);

    let codegen_doc =
        parse_uslm_str(xml, "/us/usc/t18/s1514A", "sox_1514a").expect("codegen parse");

    let runtime_ids: std::collections::HashSet<&str> =
        runtime_data.terms.iter().map(|t| t.id.as_str()).collect();
    let codegen_ids: std::collections::HashSet<&str> = codegen_doc
        .terms
        .iter()
        .filter(|t| t.id != "sox_1514a") // codegen has root term, runtime drops it
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        runtime_ids, codegen_ids,
        "codegen and runtime diverge on Title 18 § 1514A"
    );
}
