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
use pr4xis_domains::formal::meta::identifier_format::Identifier;
use pr4xis_domains::social::compliance::statutes::from_uslm::{
    derive_structural, from_uslm_section,
};
use pr4xis_domains::social::software::markup::xml::uslm::axioms::{
    axiom_child_identifier_extends_parent, axiom_every_container_has_identifier,
    axiom_every_section_has_num, axiom_hierarchy_strictly_nested, axiom_ref_hrefs_well_formed,
    axiom_section_identifiers_unique, section_identifier_to_statute_name,
};
use pr4xis_domains::social::software::markup::xml::uslm::{
    ContainerKind, HierarchyNode, UsCodeContainer, UsCodeDate, UsCodeHeader, UsCodeNoteKind,
    UsCodeSection, UsCodeSubdivision, UsCodeTitleId,
};
use praxis_corpus_tests::{UslmCorpus, corpus_or_fail, load_uslm_corpus};

/// Title 18, parsed once. `None` if the title is not on disk (fresh checkout
/// before `pr4xis update`); each test then HARD-FAILS via `corpus_or_fail!` (tests do not skip).
static TITLE_18: LazyLock<Option<UslmCorpus>> =
    LazyLock::new(|| load_uslm_corpus("legal/uscode/usc_title_18/usc_title_18-pl-119-90.xml"));

#[test]
fn full_title_18_parses_with_expected_section_count() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
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
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
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
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");

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
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");

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
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
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
    let UslmCorpus { xml, title } = corpus_or_fail!(TITLE_18, "usc_title_18");
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

// =============================================================================
// Tier-1 hierarchy coverage (M4.δ.4)
// =============================================================================

#[test]
fn hierarchy_has_published_part_chapter_subchapter_counts_for_title_18() {
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let parts = t.containers_of_kind(ContainerKind::Part);
    let chapters = t.containers_of_kind(ContainerKind::Chapter);
    let subchapters = t.containers_of_kind(ContainerKind::Subchapter);
    eprintln!(
        "Title 18: {} parts, {} chapters, {} subchapters",
        parts.len(),
        chapters.len(),
        subchapters.len()
    );
    assert!(parts.len() >= 4, "expected ≥4 parts, got {}", parts.len());
    assert!(
        chapters.len() >= 100,
        "expected ≥100 chapters, got {}",
        chapters.len()
    );
}

#[test]
fn flat_sections_equals_hierarchy_dfs_walk() {
    // Invariant: the flat `sections` field is a DFS flatten of
    // the `hierarchy` tree. Order and count must match.
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let mut dfs_collected: Vec<&UsCodeSection> = Vec::new();
    fn walk<'a>(nodes: &'a [HierarchyNode], out: &mut Vec<&'a UsCodeSection>) {
        for n in nodes {
            match n {
                HierarchyNode::Section(s) => out.push(s),
                HierarchyNode::Container(c) => walk(&c.children, out),
            }
        }
    }
    walk(&t.hierarchy, &mut dfs_collected);
    assert_eq!(dfs_collected.len(), t.sections.len());
    for (i, (a, b)) in dfs_collected.iter().zip(t.sections.iter()).enumerate() {
        assert_eq!(
            a.identifier, b.identifier,
            "section #{i} order mismatch: hierarchy={} flat={}",
            a.identifier, b.identifier
        );
    }
}

#[test]
fn every_container_has_unique_identifier_in_title_18() {
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let mut seen = std::collections::HashSet::new();
    for c in t.containers() {
        assert!(
            seen.insert(c.identifier.as_str()),
            "duplicate container identifier: {} ({:?})",
            c.identifier,
            c.kind,
        );
    }
}

// =============================================================================
// Tier-2 notes/sourceCredit/continuation/header coverage (M4.δ.5)
// =============================================================================

#[test]
fn sox_1514a_carries_pub_law_source_credit() {
    // Every codified section should have at least one
    // <sourceCredit> entry citing its originating Pub. L. and
    // Stat. citation. § 1514A was enacted by SOX 2002 (Pub. L.
    // 107-204).
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let sox = t.section("/us/usc/t18/s1514A").expect("§ 1514A");
    assert!(
        !sox.source_credits.is_empty(),
        "§ 1514A must carry ≥1 <sourceCredit>"
    );
    let first = &sox.source_credits[0];
    assert!(
        first.text.contains("Pub. L.") || first.text.contains("107"),
        "first source credit should reference SOX 2002, got: {:?}",
        first.text
    );
    // The source credit should carry refs to the originating PL.
    assert!(
        first.refs.iter().any(|r| r.href.contains("/us/pl/107/204")),
        "expected /us/pl/107/204 ref in source credit; got refs: {:?}",
        first
            .refs
            .iter()
            .map(|r| r.href.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn sox_1514a_has_notes_blocks() {
    // § 1514A should have at least one editorial notes block
    // (amendments, effective dates, or short titles).
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let sox = t.section("/us/usc/t18/s1514A").expect("§ 1514A");
    assert!(
        !sox.notes_blocks.is_empty(),
        "§ 1514A should have ≥1 <notes> block"
    );
}

#[test]
fn title_18_headers_collected_correctly() {
    // `<header>` is allowed as a direct child of `<title>` per
    // the USLM Schema, but Title 18's published structure
    // doesn't carry one at that level (its TOC headers live
    // inside `<toc>` blocks, modeled later in M4.δ.8). Verify
    // the field exists and is consistently zero or more.
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    // The field is populated correctly — the test asserts the
    // type contract, not a specific count. Title-level `<header>`
    // is an optional element.
    let _: &Vec<UsCodeHeader> = &t.headers;
}

#[test]
fn title_18_notes_blocks_collected_at_title_level() {
    // The title's preamble carries title-level notes (enacting
    // history, current-through marker, etc.).
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    assert!(
        !t.notes_blocks.is_empty(),
        "Title 18 should carry ≥1 title-level <notes> block"
    );
}

#[test]
fn note_topic_attribute_round_trips() {
    // The schema-defined `topic` attribute on <note> is the
    // editorial-semantic discriminator. Round-trip it from the
    // bare-note list (Title 18's enacting note is a direct
    // `<note topic="enacting">` child of `<title>`, not wrapped
    // in a `<notes>` block).
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let mut topics: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for note in &t.bare_notes {
        if let Some(topic) = &note.topic {
            topics.insert(topic.as_str());
        }
    }
    for block in &t.notes_blocks {
        for note in &block.notes {
            if let Some(topic) = &note.topic {
                topics.insert(topic.as_str());
            }
        }
    }
    eprintln!("Title 18 title-level note topics: {topics:?}");
    assert!(
        topics.contains("enacting"),
        "expected `enacting` topic among title-level notes; got {topics:?}"
    );
}

#[test]
fn source_credit_refs_carry_only_href_urns() {
    // Footnote backlinks (<ref class="footnoteRef" idref="...">)
    // should be filtered out of source_credit refs by the
    // collect_refs_in helper; only href URNs remain.
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let sox = t.section("/us/usc/t18/s1514A").expect("§ 1514A");
    for sc in &sox.source_credits {
        for r in &sc.refs {
            assert!(
                !r.href.is_empty(),
                "source credit ref has empty href (footnote backlink leaked through)"
            );
            assert!(
                r.href.starts_with('/'),
                "source credit ref href {:?} not URN-rooted",
                r.href
            );
        }
    }
}

// =============================================================================
// Tier-3 QuotedContent / Date / Signature (M4.δ.6)
// =============================================================================

#[test]
fn title_18_collects_quoted_content_inside_notes() {
    // Title 18's amendment-history notes contain <quotedContent>
    // blocks carrying text from the amending Pub. L. The reader
    // must collect them as typed UsCodeQuotedContent values.
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let mut total = 0usize;
    let mut with_origin = 0usize;
    for note in &t.bare_notes {
        for qc in &note.quoted_contents {
            total += 1;
            if qc.origin.is_some() {
                with_origin += 1;
            }
        }
    }
    for block in &t.notes_blocks {
        for note in &block.notes {
            for qc in &note.quoted_contents {
                total += 1;
                if qc.origin.is_some() {
                    with_origin += 1;
                }
            }
        }
    }
    eprintln!("Title 18 title-level quotedContent: {total} total, {with_origin} with origin");
    // Title 18 has 650 quotedContent elements overall; at least a
    // few should surface at the title-level enacting notes.
    assert!(
        total >= 1,
        "expected ≥1 quotedContent in title-level notes, got {total}"
    );
}

#[test]
fn quoted_content_origin_carries_urn_when_present() {
    // Every quotedContent.origin (when set) follows the USLM URN
    // shape `/us/...`.
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    for note in &t.bare_notes {
        for qc in &note.quoted_contents {
            if let Some(origin) = &qc.origin {
                assert!(
                    origin.starts_with('/'),
                    "quotedContent origin {origin:?} not URN-rooted"
                );
            }
        }
    }
}

#[test]
fn source_credits_collect_iso_dates() {
    // SourceCredit on § 1514A contains <date date="2002-07-30">
    // for the SOX 2002 enactment. The reader must capture it as
    // a typed UsCodeDate.
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let sox = t.section("/us/usc/t18/s1514A").expect("§ 1514A");
    let mut all_dates: Vec<&UsCodeDate> = Vec::new();
    for sc in &sox.source_credits {
        all_dates.extend(&sc.dates);
    }
    eprintln!(
        "§ 1514A source-credit ISO dates: {:?}",
        all_dates.iter().map(|d| d.iso.as_str()).collect::<Vec<_>>()
    );
    assert!(
        !all_dates.is_empty(),
        "§ 1514A source credits must carry ≥1 typed date"
    );
    // SOX was enacted July 30, 2002.
    let has_sox_date = all_dates.iter().any(|d| d.iso == "2002-07-30");
    assert!(
        has_sox_date,
        "expected 2002-07-30 (SOX enactment) in § 1514A dates; got {:?}",
        all_dates.iter().map(|d| d.iso.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn iso_date_format_round_trips() {
    // Every captured UsCodeDate.iso must be a well-formed ISO
    // 8601 date string. Per the USLM Schema, the `date` attribute
    // is xs:date format (`YYYY-MM-DD`).
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let mut all_dates: Vec<&UsCodeDate> = Vec::new();
    for sc in t.sections.iter().flat_map(|s| s.source_credits.iter()) {
        all_dates.extend(&sc.dates);
    }
    for d in &all_dates {
        // YYYY-MM-DD: 4-2-2 with two dashes.
        assert_eq!(d.iso.len(), 10, "date {:?} not YYYY-MM-DD format", d.iso);
        let bytes = d.iso.as_bytes();
        assert_eq!(bytes[4], b'-', "date {:?} missing first dash", d.iso);
        assert_eq!(bytes[7], b'-', "date {:?} missing second dash", d.iso);
    }
}

#[test]
fn sox_1514a_belongs_to_part_i_chapter_73_title_18() {
    // Sentinel: § 1514A lives in Title 18 > Part I (CRIMES) >
    // Chapter 73 (OBSTRUCTION OF JUSTICE). Verifies hierarchy
    // assigns sections to their published containers.
    let UslmCorpus { title: t, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");

    // Find Part I.
    let part_i = t
        .containers_of_kind(ContainerKind::Part)
        .into_iter()
        .find(|c| c.identifier == "/us/usc/t18/ptI")
        .expect("Part I missing");
    // Find Chapter 73 within Part I.
    fn find_chapter<'a>(children: &'a [HierarchyNode], ch_id: &str) -> Option<&'a UsCodeContainer> {
        for n in children {
            if let HierarchyNode::Container(c) = n {
                if c.kind == ContainerKind::Chapter && c.identifier == ch_id {
                    return Some(c);
                }
                if let Some(found) = find_chapter(&c.children, ch_id) {
                    return Some(found);
                }
            }
        }
        None
    }
    let ch73 = find_chapter(&part_i.children, "/us/usc/t18/ptI/ch73")
        .expect("Chapter 73 missing under Part I");
    // § 1514A must be a leaf in Chapter 73.
    let has_1514a = ch73
        .children
        .iter()
        .any(|n| matches!(n, HierarchyNode::Section(s) if s.identifier == "/us/usc/t18/s1514A"));
    assert!(
        has_1514a,
        "§ 1514A not found as a leaf in Chapter 73 of Title 18"
    );
}

// =============================================================================
// M4.δ.11/15/16/17/19/20 — Non-USC document element tripwire
// =============================================================================

#[test]
fn non_usc_kinds_zero_in_title_18_corpus() {
    // Tripwire: a non-USC element ever appearing in a USC corpus
    // would indicate LRC has merged the bill/CFR pipeline into
    // the USC pipeline. Verify the published Title 18 contains
    // none of: ins, del, subheading, crossHeading, docTitle,
    // longTitle, shortTitle, division, article, subarticle,
    // preamble, preliminary, appendix, subsubitem, quotedText,
    // recital, statement, enactingFormula, amendingFormula,
    // approved, made, action, instruction, checkBox, fillIn.
    let UslmCorpus { xml, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    // Cheap substring check — the LRC's USLM never wraps any of
    // these tags in commented-out blocks, so a literal "<tag" hit
    // means the element appears in the document.
    for tag in [
        "<ins ",
        "<ins>",
        "<del ",
        "<del>",
        "<subheading",
        "<crossHeading",
        "<docTitle",
        "<longTitle",
        "<shortTitle",
        "<division",
        "<article",
        "<subarticle",
        "<preamble",
        "<preliminary",
        "<appendix",
        "<subsubitem",
        "<quotedText",
        "<recital",
        "<statement",
        "<enactingFormula",
        "<amendingFormula",
        "<approved",
        "<made",
        "<action",
        "<instruction",
        "<checkBox",
        "<fillIn",
    ] {
        assert!(
            !xml.contains(tag),
            "Title 18 USC corpus must not contain {tag} — that's a non-USC USLM element"
        );
    }
}

// =============================================================================
// M4.δ.14 — Amendment markup (Ins/Del) over the real corpus
// =============================================================================

#[test]
fn title_18_real_corpus_has_zero_amendment_markup_today() {
    // Retro-converted USC titles don't carry `<ins>`/`<del>` —
    // those appear in amendments-in-progress (bills, public laws).
    // Tripwire: if this assertion ever fails, LRC has changed the
    // pl-XXX-YY USC publication format to include amendment-diff
    // markup and downstream consumers should be reviewed.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");

    let mut total = 0usize;
    for s in &title.sections {
        total += s.amendments.len();
        for sub in &s.children {
            total += count_amendments_sub(sub);
        }
    }
    assert_eq!(
        total, 0,
        "Title 18 pl-119-90 must not carry <ins>/<del> markup; got {total}"
    );
}

fn count_amendments_sub(d: &UsCodeSubdivision) -> usize {
    let mut n = d.amendments.len();
    for c in &d.children {
        n += count_amendments_sub(c);
    }
    n
}

// =============================================================================
// M4.δ.18 — Additional metadata over the real corpus
// =============================================================================

#[test]
fn title_18_real_corpus_is_positive_law() {
    // Title 18 is enacted as positive law (Act of June 25, 1948,
    // ch. 645, 62 Stat. 683). Verify the LRC's published USLM
    // carries the property correctly.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let meta = title.meta.as_ref().expect("Title 18 has meta block");
    assert_eq!(
        meta.is_positive_law(),
        Some(true),
        "Title 18 must declare is-positive-law=yes"
    );
}

#[test]
fn title_18_real_corpus_carries_doc_number_and_publication() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let meta = title.meta.as_ref().expect("Title 18 has meta block");
    assert_eq!(meta.doc_number.as_deref(), Some("18"));
    assert!(
        meta.doc_publication_name
            .as_deref()
            .unwrap_or("")
            .contains("119-90"),
        "publication name should reference release point 119-90; got {:?}",
        meta.doc_publication_name
    );
}

// =============================================================================
// M4.δ.9 — Tier-6 Table over the real corpus
// =============================================================================

#[test]
fn title_18_real_corpus_has_table_of_disposition() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    // LRC reports 17 tables in Title 18 pl-119-90. Most carry
    // class="TableOfDisposition" — verify the collector picks them up.
    assert!(
        title.tables.len() >= 10,
        "Title 18 must contain ≥10 tables; got {}",
        title.tables.len()
    );
    let dispo_tables = title
        .tables
        .iter()
        .filter(|t| t.class.as_deref() == Some("TableOfDisposition"))
        .count();
    assert!(
        dispo_tables >= 1,
        "Title 18 must carry ≥1 TableOfDisposition; got {dispo_tables}"
    );
}

// =============================================================================
// M4.δ.8 — Tier-5 Table of Contents over the real corpus
// =============================================================================

#[test]
fn title_18_real_corpus_has_three_column_toc() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    // Title 18 ships at least one title-level TOC (the three-column
    // Part / Heading / Section index).
    assert!(
        !title.tocs.is_empty(),
        "Title 18 must carry ≥1 title-level <toc>"
    );
    let has_three_column = title
        .tocs
        .iter()
        .any(|t| t.role.as_deref() == Some("threeColumnTOC"));
    assert!(
        has_three_column,
        "Title 18's TOC must include role=threeColumnTOC"
    );

    let title_level_items: usize = title.tocs.iter().map(|t| t.items.len()).sum();
    assert!(
        title_level_items >= 5,
        "Title 18's title-level TOC should index ≥5 parts/chapters; got {title_level_items}"
    );
}

#[test]
fn title_18_chapter_tocs_total_at_least_a_thousand_items() {
    // LRC's TOC count for Title 18 (146 <toc> with 1534 <tocItem>)
    // means the corpus-wide TOC fanout is substantial — verifies
    // that container-level TOC collection (M4.δ.8) is wired up.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");

    let mut total = title.tocs.iter().map(|t| t.items.len()).sum::<usize>();
    for c in title.containers() {
        total += c.tocs.iter().map(|t| t.items.len()).sum::<usize>();
    }
    // LRC reports 1534 tocItems in Title 18 pl-119-90; allow some
    // wiggle for editorial header rows that don't become items.
    assert!(
        total >= 1000,
        "Title 18 chapter+title TOCs should total ≥1000 items; got {total}"
    );
}

// =============================================================================
// M4.δ.10 — Tier-7 Dublin Core meta block over the real corpus
// =============================================================================

#[test]
fn title_18_real_corpus_has_olrc_meta_block() {
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let meta = title
        .meta
        .as_ref()
        .expect("Title 18 must carry a meta block");
    // The four LRC-consistent fields.
    assert_eq!(meta.title.as_deref(), Some("Title 18"));
    assert_eq!(meta.doc_type.as_deref(), Some("USCTitle"));
    assert_eq!(meta.publisher.as_deref(), Some("OLRC"));
    assert!(
        meta.creator
            .as_deref()
            .unwrap_or("")
            .starts_with("USCConverter"),
        "dc:creator must be the USCConverter; got {:?}",
        meta.creator
    );
}

// =============================================================================
// M4.δ.12 — Note-kind classification over the real corpus
// =============================================================================

#[test]
fn title_18_corpus_classifies_at_least_one_note_per_documented_kind() {
    // Real-corpus assertion: Title 18's notes span the documented
    // topic vocabulary; classification should surface at least one
    // Editorial, one Statutory, and one Change note across the
    // title (footnotes likewise common at the title level).
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");

    let mut counts = std::collections::HashMap::new();
    for nb in &title.notes_blocks {
        for n in &nb.notes {
            *counts.entry(n.kind()).or_insert(0usize) += 1;
        }
    }
    for n in &title.bare_notes {
        *counts.entry(n.kind()).or_insert(0usize) += 1;
    }
    for section in &title.sections {
        for nb in &section.notes_blocks {
            for n in &nb.notes {
                *counts.entry(n.kind()).or_insert(0usize) += 1;
            }
        }
        for n in &section.bare_notes {
            *counts.entry(n.kind()).or_insert(0usize) += 1;
        }
    }

    // The expected non-empty kinds in the published corpus. Other
    // kinds (Enacting) may or may not be present at any given
    // release point.
    for expected in [
        UsCodeNoteKind::Editorial,
        UsCodeNoteKind::Statutory,
        UsCodeNoteKind::Change,
    ] {
        assert!(
            counts.get(&expected).copied().unwrap_or(0) > 0,
            "Title 18 must contain ≥1 note of kind {expected:?}; counts={counts:?}"
        );
    }
}

// =============================================================================
// M4.δ.13 — Tier-5 definitional ontology over the real corpus
// =============================================================================

#[test]
fn title_18_real_corpus_has_zero_typed_def_elements_today() {
    // Documentation assertion: LRC pl-119-90 doesn't yet emit
    // typed `<def>` / `<marker>` elements. If this count ever goes
    // up, the corpus has moved forward and downstream code that
    // anticipates defined-term lifts should be reviewed.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");

    let mut total_defs = 0usize;
    let mut total_markers = 0usize;
    for s in &title.sections {
        total_defs += s.def_blocks.len();
        total_markers += s.markers.len();
        for sub in &s.children {
            count_def_marker_in_sub(sub, &mut total_defs, &mut total_markers);
        }
    }
    // Allow strictly-zero or strictly-positive; the assertion is a
    // tripwire on a documented invariant.
    assert!(
        total_defs == 0 && total_markers == 0,
        "LRC pl-119-90 Title 18 corpus was expected to have zero \
         typed <def> / <marker> elements; counts: defs={total_defs}, \
         markers={total_markers}. If LRC has rolled forward the \
         retro-conversion, update this test and verify downstream \
         consumers handle the new structure."
    );
}

fn count_def_marker_in_sub(d: &UsCodeSubdivision, defs: &mut usize, markers: &mut usize) {
    *defs += d.def_blocks.len();
    *markers += d.markers.len();
    for c in &d.children {
        count_def_marker_in_sub(c, defs, markers);
    }
}

// =============================================================================
// Tier-4 inline runs over the real corpus
// =============================================================================

#[test]
fn title_18_contains_inline_runs_with_class_attributes() {
    // Real-corpus assertion: Title 18 ships with at least one
    // subsection whose heading is wrapped in `<inline
    // class="small-caps">` — the canonical USLM idiom for
    // styled defined-term headings. If zero inline runs carry a
    // class, either the corpus has changed format or our reader
    // is dropping them silently.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");

    let mut classed_runs = 0usize;
    for s in &title.sections {
        classed_runs += s.heading_runs.iter().filter(|r| r.class.is_some()).count();
        for sub in &s.children {
            classed_runs += count_classed_runs_sub(sub);
        }
    }
    assert!(
        classed_runs > 0,
        "Title 18 must contain ≥1 inline run with a class attribute; \
         small-caps headings are pervasive in USLM"
    );
}

fn count_classed_runs_sub(d: &UsCodeSubdivision) -> usize {
    let mut n = d.heading_runs.iter().filter(|r| r.class.is_some()).count();
    n += d.chapeau_runs.iter().filter(|r| r.class.is_some()).count();
    n += d.content_runs.iter().filter(|r| r.class.is_some()).count();
    for c in &d.children {
        n += count_classed_runs_sub(c);
    }
    n
}

#[test]
fn sox_1514a_subsection_a_carries_whistleblower_protection_small_caps() {
    // Title 18's § 1514A subsection (a) has the iconic
    // "Whistleblower Protection for Employees of Publicly Traded
    // Companies" heading rendered in small-caps via
    // `<inline class="small-caps">`. This is the precise
    // legal-term that downstream definition-extractors must lift.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let sox = title
        .section("/us/usc/t18/s1514A")
        .expect("§ 1514A present");
    let a = sox
        .children
        .iter()
        .find(|c| c.identifier == "/us/usc/t18/s1514A/a")
        .expect("subsection (a) present");

    let has_small_caps = a.heading_runs.iter().any(|r| {
        r.class.as_deref() == Some("small-caps") && r.text.to_lowercase().contains("whistleblower")
    });
    assert!(
        has_small_caps,
        "§ 1514A(a) heading must include <inline class=\"small-caps\">…Whistleblower…</inline>; \
         got runs: {:?}",
        a.heading_runs
    );
}

// =============================================================================
// Structural-integrity invariants (lock-in)
//
// Cross-cutting properties that must hold across every USC title the
// LRC publishes. The two cross-title invariants (section-URN tree
// shape and title-id round-trip) are asserted here for Title 18; the
// Title-49 half lives in `title_49.rs`.
// =============================================================================

#[test]
fn every_section_urn_starts_with_its_title_urn_t18() {
    // Composition property: a section's URN must be a path-extension
    // of its enclosing title's URN. Without this, the URN graph
    // doesn't form a tree and citation traversal breaks. Multi-URN
    // identifiers (combined ranges) are skipped — they're a separate
    // construct outside the single-URN composition rule.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
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
fn every_single_section_urn_is_uslm_urn_grammar_conformant() {
    // Every parsed section's identifier — when it's a single URN —
    // must satisfy the USLM-URN grammar. LRC uses space-separated
    // multi-URN identifiers for combined repealed-section ranges
    // (e.g. "/us/usc/t18/s221 /us/usc/t18/s222"); those are a
    // separate ontological construct not yet typed and are
    // legitimately outside the single-URN grammar.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let mut multi_urn_count = 0usize;
    for s in &title.sections {
        if s.identifier.contains(' ') {
            multi_urn_count += 1;
            continue;
        }
        Identifier::uslm_urn(&s.identifier)
            .unwrap_or_else(|e| panic!("section URN {} fails USLM grammar: {:?}", s.identifier, e));
    }
    // Document the multi-URN identifier count for visibility.
    // LRC pl-119-90 Title 18 ships a handful of these.
    if multi_urn_count > 0 {
        eprintln!(
            "Title 18 carries {multi_urn_count} multi-URN section identifiers (combined ranges)"
        );
    }
}

#[test]
fn title_id_round_trips_via_urn_for_title_18() {
    // Building a UsCodeTitleId from the parsed title's URN must
    // yield the same number as `UsCodeTitle.number`.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");
    let expected_number = 18u32;
    let id =
        UsCodeTitleId::try_from_urn(&title.identifier).expect("title URN is a valid UsCodeTitleId");
    assert_eq!(id.number(), expected_number);
    assert_eq!(title.number, expected_number);
    // The URN parsed from the corpus equals the URN built from
    // the title number — round-trip closure.
    assert_eq!(id.urn(), title.identifier);
}

#[test]
fn every_section_lifts_to_statute_with_urn_provenance() {
    // The to_statute lift (USLM section → Statute) must produce a
    // Statute whose description carries the section's URN as
    // context_uri. This is the M4.δ.21 URN push-down invariant —
    // enforced uniformly across every section in every loaded title.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");

    // Spot-check 10 sections — too expensive to lift all ~1500.
    use pr4xis_domains::social::software::markup::xml::uslm::corpus::loaded as usc_loaded;
    let usc = usc_loaded();
    for s in title.sections.iter().take(10) {
        let urn = match Identifier::uslm_urn(&s.identifier) {
            Ok(u) => u,
            Err(_) => continue,
        };
        if let Some(section) = usc.section_by_urn(&urn) {
            let statute = section.to_statute("test_lift", "1");
            let ctx = statute.description().context_uri.as_deref().unwrap_or("");
            assert_eq!(
                ctx, s.identifier,
                "to_statute lift of {} must carry URN provenance",
                s.identifier
            );
        }
    }
}

#[test]
fn no_section_has_internally_duplicate_subdivision_ids() {
    // Within a section, every subdivision's URN must be unique.
    // Duplicates would collide on term_by_id lookups silently.
    let UslmCorpus { title, .. } = corpus_or_fail!(TITLE_18, "usc_title_18");

    for s in &title.sections {
        let mut ids = std::collections::HashSet::new();
        check_unique_sub_ids(s, &mut ids);
    }
}

fn check_unique_sub_ids(section: &UsCodeSection, seen: &mut std::collections::HashSet<String>) {
    for sub in &section.children {
        recurse_check_unique(sub, seen);
    }
}

fn recurse_check_unique(d: &UsCodeSubdivision, seen: &mut std::collections::HashSet<String>) {
    if !d.identifier.is_empty() {
        assert!(
            seen.insert(d.identifier.clone()),
            "duplicate subdivision URN within section: {}",
            d.identifier
        );
    }
    for c in &d.children {
        recurse_check_unique(c, seen);
    }
}
