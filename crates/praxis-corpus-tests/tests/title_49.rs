//! Full Title 49 (Transportation) — home of AIR21 § 42121, the airline-
//! whistleblower protection provision (49 U.S.C. § 42121) that anchors the
//! AIR21 retaliation case.
//!
//! The giant XML is parsed ONCE into a process-shared [`LazyLock`] fixture;
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
use pr4xis_domains::social::software::markup::xml::uslm::{ContainerKind, UsCodeTitleId};
use praxis_corpus_tests::{UslmCorpus, load_uslm_corpus};

/// Title 49, parsed once. `None` if the giant is not on disk (fresh checkout
/// before `pr4xis update`); each test then skips gracefully.
static TITLE_49: LazyLock<Option<UslmCorpus>> =
    LazyLock::new(|| load_uslm_corpus("legal/uscode/usc_title_49/usc_title_49-pl-119-90.xml"));

/// Borrow the shared corpus, or return early with a SKIP note when absent.
macro_rules! corpus_or_skip {
    () => {
        match &*TITLE_49 {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Title 49 USLM not on disk (run `pr4xis update`)");
                return;
            }
        }
    };
}

#[test]
fn full_title_49_parses_with_expected_section_count() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    assert_eq!(title.identifier, "/us/usc/t49");
    assert_eq!(title.number, 49);
    assert!(
        title.heading.contains("TRANSPORTATION"),
        "got heading: {:?}",
        title.heading
    );
    assert!(
        title.sections.len() >= 1_000,
        "expected ≥1,000 sections, got {}",
        title.sections.len()
    );
}

#[test]
fn full_title_49_every_section_has_unique_identifier() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let mut seen = std::collections::HashSet::new();
    for s in &title.sections {
        assert!(seen.insert(&s.identifier), "duplicate: {}", s.identifier);
    }
}

#[test]
fn full_title_49_every_section_satisfies_every_axiom() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    axiom_every_section_has_num(title).expect("EverySectionHasNum");
    axiom_every_container_has_identifier(title).expect("EveryContainerHasIdentifier");
    axiom_child_identifier_extends_parent(title).expect("ChildIdentifierExtendsParent");
    axiom_hierarchy_strictly_nested(title).expect("HierarchyStrictlyNested");
    axiom_section_identifiers_unique(title).expect("SectionIdentifiersUnique");
    axiom_ref_hrefs_well_formed(title).expect("RefHrefsWellFormed");
}

#[test]
fn full_title_49_every_section_lifts_to_statute() {
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
            "{failed}/{} sections failed; first: {n}: {msg}",
            title.sections.len()
        );
    }
    assert_eq!(failed, 0);
}

#[test]
fn full_title_49_known_sections_present() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let ids: std::collections::HashSet<&str> = title
        .sections
        .iter()
        .map(|s| s.identifier.as_str())
        .collect();
    // AIR21 whistleblower — the case's load-bearing § in Title 49.
    assert!(ids.contains("/us/usc/t49/s42121"), "§ 42121 missing");
}

#[test]
fn full_title_49_codegen_and_runtime_agree_on_air21() {
    let UslmCorpus { xml, title } = corpus_or_skip!();
    let section = title
        .section("/us/usc/t49/s42121")
        .expect("§ 42121 must be present");
    let runtime_data = derive_structural("air21_42121", section);
    let codegen_doc =
        parse_uslm_str(xml, "/us/usc/t49/s42121", "air21_42121").expect("codegen parse");
    let runtime_ids: std::collections::HashSet<&str> =
        runtime_data.terms.iter().map(|t| t.id.as_str()).collect();
    let codegen_ids: std::collections::HashSet<&str> = codegen_doc
        .terms
        .iter()
        .filter(|t| t.id != "air21_42121")
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        runtime_ids, codegen_ids,
        "codegen and runtime diverge on Title 49 § 42121"
    );
}

#[test]
fn hierarchy_has_published_subtitle_part_chapter_counts_for_title_49() {
    let UslmCorpus { title: t, .. } = corpus_or_skip!();
    let subtitles = t.containers_of_kind(ContainerKind::Subtitle);
    let parts = t.containers_of_kind(ContainerKind::Part);
    let subparts = t.containers_of_kind(ContainerKind::Subpart);
    let chapters = t.containers_of_kind(ContainerKind::Chapter);
    let subchapters = t.containers_of_kind(ContainerKind::Subchapter);
    eprintln!(
        "Title 49: {} subtitles, {} parts, {} subparts, {} chapters, {} subchapters",
        subtitles.len(),
        parts.len(),
        subparts.len(),
        chapters.len(),
        subchapters.len()
    );
    assert!(
        subtitles.len() >= 5,
        "expected ≥5 subtitles, got {}",
        subtitles.len()
    );
    assert!(parts.len() >= 5, "expected ≥5 parts, got {}", parts.len());
    assert!(
        chapters.len() >= 50,
        "expected ≥50 chapters, got {}",
        chapters.len()
    );
}

#[test]
fn quoted_content_sections_dont_leak_into_published_sections() {
    // The duplicate-identifier failure that surfaced in Title 49
    // was caused by `<section>` elements inside `<quotedContent>`
    // being collected as real sections. With Tier-3, those become
    // UsCodeSectionRef inside the quote's body, and the published
    // sections list is unaffected.
    let UslmCorpus { title: t, .. } = corpus_or_skip!();
    // Every published section has a non-empty identifier.
    for s in &t.sections {
        assert!(
            !s.identifier.is_empty(),
            "published section with empty identifier"
        );
    }
    // Sections inside quoted content are accessible through note
    // quoted_contents — collected as UsCodeSectionRef.
    let mut quoted_section_count = 0usize;
    for s in &t.sections {
        for note in &s.bare_notes {
            for qc in &note.quoted_contents {
                quoted_section_count += qc.section_refs.len();
            }
        }
        for block in &s.notes_blocks {
            for note in &block.notes {
                for qc in &note.quoted_contents {
                    quoted_section_count += qc.section_refs.len();
                }
            }
        }
    }
    eprintln!("Title 49 section-level quoted-section refs: {quoted_section_count}");
    assert!(quoted_section_count >= 1, "expected ≥1 quoted section ref");
}

#[test]
fn title_49_real_corpus_has_olrc_meta_block() {
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let meta = title
        .meta
        .as_ref()
        .expect("Title 49 must carry a meta block");
    assert_eq!(meta.title.as_deref(), Some("Title 49"));
    assert_eq!(meta.publisher.as_deref(), Some("OLRC"));
}

#[test]
fn every_section_urn_starts_with_its_title_urn_t49() {
    // Composition property: a section's URN must be a path-extension
    // of its enclosing title's URN. Without this, the URN graph
    // doesn't form a tree and citation traversal breaks. Multi-URN
    // identifiers (combined ranges) are skipped — they're a separate
    // construct outside the single-URN composition rule.
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let title_urn = title.identifier.as_str();
    for s in &title.sections {
        if s.identifier.contains(' ') {
            continue;
        }
        assert!(
            s.identifier.starts_with(&format!("{title_urn}/")),
            "section URN {} doesn't extend title URN {title_urn}",
            s.identifier
        );
    }
}

#[test]
fn title_id_round_trips_via_urn_for_title_18_and_49_t49() {
    // Building a UsCodeTitleId from the parsed title's URN must
    // yield the same number as `UsCodeTitle.number`.
    let UslmCorpus { title, .. } = corpus_or_skip!();
    let expected_number = 49u32;
    let id =
        UsCodeTitleId::try_from_urn(&title.identifier).expect("title URN is a valid UsCodeTitleId");
    assert_eq!(id.number(), expected_number);
    assert_eq!(title.number, expected_number);
    // The URN parsed from the corpus equals the URN built from
    // the title number — round-trip closure.
    assert_eq!(id.urn(), title.identifier);
}
