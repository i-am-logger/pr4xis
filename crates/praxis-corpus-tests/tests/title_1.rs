//! Full Title 1 — the U.S. Code's smallest, foundational title: the Dictionary
//! Act (§§ 1-8 statutory-construction defaults), the enacting clauses
//! (§§ 101-105), the Statutes-at-Large / treaty publication framework
//! (§§ 112, 112a), and § 204 — the authority that makes the LRC's USLM XML the
//! legal text of the U.S. Code, closing the self-referential loop by which the
//! corpus cites the statute that authorizes its own form.
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

/// Title 1, parsed once. `None` if the giant is not on disk (fresh checkout
/// before `pr4xis update`); each test then skips gracefully.
static TITLE_1: LazyLock<Option<UslmCorpus>> =
    LazyLock::new(|| load_uslm_corpus("legal/uscode/usc_title_1/usc_title_1-pl-119-90.xml"));

/// Borrow the shared corpus, or return early with a SKIP note when absent.
macro_rules! corpus_or_skip {
    () => {
        match &*TITLE_1 {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Title 1 USLM not on disk (run `pr4xis update`)");
                return;
            }
        }
    };
}

#[test]
fn full_title_1_parses_with_expected_section_count() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    assert_eq!(title.identifier, "/us/usc/t1");
    assert_eq!(title.number, 1);
    assert!(
        title
            .heading
            .to_ascii_uppercase()
            .contains("GENERAL PROVISIONS"),
        "got heading: {:?}",
        title.heading
    );
    // Title 1 is the smallest title — the Dictionary Act, enacting
    // clauses, the Code-authority section, and the publication
    // framework. Lower bound is conservative.
    assert!(
        title.sections.len() >= 30,
        "expected ≥30 sections, got {}",
        title.sections.len()
    );
}

#[test]
fn full_title_1_every_section_has_unique_identifier() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    axiom_section_identifiers_unique(title)
        .expect("SectionIdentifiersUnique-or-LRC-documented-duplicates must hold for full Title 1");
}

#[test]
fn full_title_1_every_section_satisfies_every_axiom() {
    let UslmCorpus { title, .. } = corpus_or_skip!();

    axiom_every_section_has_num(title).expect("EverySectionHasNum must hold for full Title 1");
    axiom_every_container_has_identifier(title)
        .expect("EveryContainerHasIdentifier must hold for full Title 1");
    axiom_child_identifier_extends_parent(title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 1");
    axiom_hierarchy_strictly_nested(title)
        .expect("HierarchyStrictlyNested must hold for full Title 1");
    axiom_section_identifiers_unique(title)
        .expect("SectionIdentifiersUnique must hold for full Title 1");
    axiom_ref_hrefs_well_formed(title).expect("RefHrefsWellFormed must hold for full Title 1");
}

#[test]
fn full_title_1_every_section_lifts_to_statute() {
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
fn full_title_1_known_sections_present() {
    // Sentinel sections — the corpus-authority anchor + the
    // statutory-construction defaults every other title inherits.
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let ids: std::collections::HashSet<&str> = title
        .sections
        .iter()
        .map(|s| s.identifier.as_str())
        .collect();

    // § 1 — Words denoting number, gender, and so forth (the
    // Dictionary Act's lead section: "words importing the singular
    // include and apply to several persons…"). The default rules of
    // statutory construction every other title is read against.
    assert!(
        ids.contains("/us/usc/t1/s1"),
        "§ 1 (Dictionary Act) missing"
    );

    // § 101 — Enacting clause ("Be it enacted by the Senate and
    // House of Representatives…").
    assert!(
        ids.contains("/us/usc/t1/s101"),
        "§ 101 (enacting clause) missing"
    );

    // § 112 — Statutes at Large; contents; admissibility in
    // evidence. The companion publication authority to § 204.
    assert!(
        ids.contains("/us/usc/t1/s112"),
        "§ 112 (Statutes at Large) missing"
    );

    // § 204 — Codes and Supplements as evidence of the laws of
    // United States and District of Columbia; citation of Codes and
    // Supplements. THE authority that makes the LRC's USLM XML the
    // legal text of the U.S. Code — the statute every other
    // registered title's provenance line cites.
    assert!(
        ids.contains("/us/usc/t1/s204"),
        "§ 204 (Code-as-evidence / positive-law authority) missing"
    );
}

#[test]
fn full_title_1_codegen_and_runtime_agree_on_section_204() {
    // § 204 (Codes and Supplements as evidence; positive-law
    // authority) is THE self-referential anchor — the statute that
    // authorizes the corpus's own USLM form. Runtime XML loading
    // (M4.δ.7.a) must produce the same term set the build-time codegen
    // path would have.
    let UslmCorpus { xml, title } = corpus_or_skip!();
    let section = title
        .section("/us/usc/t1/s204")
        .expect("§ 204 must be present");
    let runtime_data = derive_structural("usc1_204", section);
    let codegen_doc = parse_uslm_str(xml, "/us/usc/t1/s204", "usc1_204").expect("codegen parse");
    let runtime_ids: std::collections::HashSet<&str> =
        runtime_data.terms.iter().map(|t| t.id.as_str()).collect();
    let codegen_ids: std::collections::HashSet<&str> = codegen_doc
        .terms
        .iter()
        .filter(|t| t.id != "usc1_204")
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        runtime_ids, codegen_ids,
        "codegen and runtime diverge on Title 1 § 204"
    );
}
