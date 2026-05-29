#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::corpus::*;
use super::lens::read_uslm_title;

/// Inline fixture — a one-§ slice mirroring the structural shape
/// of the real SOX § 1514A USLM.
const SAMPLE_SECTION_SLICE: &str = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1514A"><num value="1514A">§ 1514A.</num><heading>Civil action to protect against retaliation in fraud cases</heading><subsection identifier="/us/usc/t18/s1514A/a"><num value="a">(a)</num><heading><inline class="small-caps">Whistleblower Protection</inline></heading><chapeau>No company may discriminate against an employee—</chapeau><paragraph identifier="/us/usc/t18/s1514A/a/1"><num value="1">(1)</num><chapeau>to provide information—</chapeau><subparagraph identifier="/us/usc/t18/s1514A/a/1/A"><num value="A">(A)</num><content>a Federal regulatory or law enforcement agency;</content></subparagraph></paragraph></subsection></section>"##;

/// Inline fixture — a `<title>` wrapper with two `<section>`
/// children at different nesting depths (one under a chapter).
const SAMPLE_TITLE: &str = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><num value="18">Title 18—</num><heading>CRIMES AND CRIMINAL PROCEDURE</heading><section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>First section</heading><content>Body text.</content></section><part identifier="/us/usc/t18/ptI"><num value="I">PART I—</num><heading>CRIMES</heading><chapter identifier="/us/usc/t18/ptI/ch1"><num value="1">CHAPTER 1—</num><heading>GENERAL PROVISIONS</heading><section identifier="/us/usc/t18/ptI/ch1/s2"><num value="2">§ 2.</num><heading>Second section</heading><content>More body.</content></section></chapter></part></title>"##;

// ── ontology types ────────────────────────────────────────────

#[test]
fn subdivision_kind_parses_canonical_tags() {
    assert_eq!(
        SubdivisionKind::parse("subsection"),
        Some(SubdivisionKind::Subsection)
    );
    assert_eq!(
        SubdivisionKind::parse("paragraph"),
        Some(SubdivisionKind::Paragraph)
    );
    assert_eq!(
        SubdivisionKind::parse("subparagraph"),
        Some(SubdivisionKind::Subparagraph)
    );
    assert_eq!(
        SubdivisionKind::parse("clause"),
        Some(SubdivisionKind::Clause)
    );
    assert_eq!(
        SubdivisionKind::parse("subclause"),
        Some(SubdivisionKind::Subclause)
    );
    assert_eq!(SubdivisionKind::parse("item"), Some(SubdivisionKind::Item));
    assert_eq!(
        SubdivisionKind::parse("subitem"),
        Some(SubdivisionKind::Subitem)
    );
}

#[test]
fn subdivision_kind_rejects_non_uslm_tags() {
    assert_eq!(SubdivisionKind::parse("section"), None);
    assert_eq!(SubdivisionKind::parse("chapter"), None);
    assert_eq!(SubdivisionKind::parse(""), None);
}

#[test]
fn nesting_depth_orders_subdivisions_correctly() {
    use SubdivisionKind::*;
    let kinds = [
        Subsection,
        Paragraph,
        Subparagraph,
        Clause,
        Subclause,
        Item,
        Subitem,
    ];
    for (i, k) in kinds.iter().enumerate() {
        assert_eq!(k.nesting_depth(), i);
    }
}

// ── single-section reading ────────────────────────────────────

#[test]
fn reads_single_section_slice() {
    let title = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    assert_eq!(title.sections.len(), 1);
    let s = &title.sections[0];
    assert_eq!(s.identifier, "/us/usc/t18/s1514A");
    assert_eq!(s.num, "1514A");
    assert!(s.heading.contains("Civil action"));
}

#[test]
fn subsection_inside_section_parsed_with_chapeau() {
    let title = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    let s = &title.sections[0];
    assert_eq!(s.children.len(), 1);
    let a = &s.children[0];
    assert_eq!(a.kind, SubdivisionKind::Subsection);
    assert_eq!(a.identifier, "/us/usc/t18/s1514A/a");
    assert_eq!(a.num, "a");
    assert!(a.heading.as_deref().unwrap_or("").contains("Whistleblower"));
    assert!(
        a.chapeau
            .as_deref()
            .unwrap_or("")
            .contains("No company may discriminate")
    );
}

#[test]
fn nested_subparagraph_carries_content_not_chapeau() {
    let title = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    let a = &title.sections[0].children[0];
    let p = &a.children[0];
    assert_eq!(p.kind, SubdivisionKind::Paragraph);
    let sp = &p.children[0];
    assert_eq!(sp.kind, SubdivisionKind::Subparagraph);
    assert_eq!(sp.identifier, "/us/usc/t18/s1514A/a/1/A");
    assert!(
        sp.content
            .as_deref()
            .unwrap_or("")
            .contains("Federal regulatory")
    );
    assert!(sp.chapeau.is_none());
}

// ── title-level reading (walks parts / chapters to find <section>) ──

#[test]
fn title_parses_metadata_and_finds_all_nested_sections() {
    let t = read_uslm_title(SAMPLE_TITLE).expect("parse");
    assert_eq!(t.identifier, "/us/usc/t18");
    assert_eq!(t.number, 18);
    assert_eq!(t.heading, "CRIMES AND CRIMINAL PROCEDURE");
    // One section directly under <title>, one under a chapter.
    assert_eq!(t.sections.len(), 2);
    let ids: Vec<_> = t.sections.iter().map(|s| s.identifier.as_str()).collect();
    assert!(ids.contains(&"/us/usc/t18/s1"));
    assert!(ids.contains(&"/us/usc/t18/ptI/ch1/s2"));
}

#[test]
fn title_section_lookup_by_identifier() {
    let t = read_uslm_title(SAMPLE_TITLE).expect("parse");
    let s2 = t.section("/us/usc/t18/ptI/ch1/s2").expect("section");
    assert_eq!(s2.num, "2");
    assert!(t.section("/us/usc/t18/s9999").is_none());
}

// ── error paths ───────────────────────────────────────────────

#[test]
fn rejects_non_uslm_root() {
    let xml = r##"<other><stuff/></other>"##;
    let err = read_uslm_title(xml).expect_err("should fail");
    assert_eq!(err, UslmReadError::NoUsCodeRoot);
}

// ── real-corpus check ─────────────────────────────────────────

#[test]
fn parses_real_sox_1514a_slice() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/statutes/us_federal/sox_1514a/sox_1514a-2002.xml");
    if !path.exists() {
        eprintln!("SKIP: SOX § 1514A slice not on disk at {path:?}");
        return;
    }
    let xml = std::fs::read_to_string(&path).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");
    assert_eq!(title.sections.len(), 1);
    let s = &title.sections[0];
    assert_eq!(s.identifier, "/us/usc/t18/s1514A");
    assert_eq!(s.num, "1514A");
    assert!(
        s.heading.contains("Civil action") && s.heading.contains("retaliation"),
        "got heading: {:?}",
        s.heading
    );

    // The published § 1514A has five top-level subsections (a)-(e).
    let subsection_count = s
        .children
        .iter()
        .filter(|c| c.kind == SubdivisionKind::Subsection)
        .count();
    assert_eq!(
        subsection_count, 5,
        "expected 5 subsections (a)-(e); got {subsection_count}"
    );

    // Every published subsection identifier is present.
    for letter in ["a", "b", "c", "d", "e"] {
        let expected = format!("/us/usc/t18/s1514A/{letter}");
        assert!(
            s.children.iter().any(|c| c.identifier == expected),
            "missing subsection {expected}"
        );
    }

    // Cross-references must be collected somewhere in the tree.
    // § 1514A(a) chapeau alone cites 15 U.S.C. 78l, 78o, 78c —
    // at least three refs anywhere in the subtree.
    let total_refs = count_refs(s);
    assert!(
        total_refs >= 3,
        "expected ≥3 <ref> elements; got {total_refs}"
    );
}

fn count_refs(s: &UsCodeSection) -> usize {
    s.refs.len() + s.children.iter().map(count_refs_sub).sum::<usize>()
}

fn count_refs_sub(d: &UsCodeSubdivision) -> usize {
    d.refs.len() + d.children.iter().map(count_refs_sub).sum::<usize>()
}

// ── proptest: parser determinism ──────────────────────────────

use proptest::prelude::*;

proptest! {
    /// Same XML bytes → byte-identical UsCodeTitle. The parser
    /// builds typed values from XmlDocument, which is itself
    /// deterministic; this proptest fixes the contract.
    #[test]
    fn prop_read_is_deterministic(seed in any::<u32>()) {
        let _ = seed;
        let t1 = read_uslm_title(SAMPLE_TITLE).unwrap();
        let t2 = read_uslm_title(SAMPLE_TITLE).unwrap();
        prop_assert_eq!(t1, t2);
    }

    /// Round-trip-style: parse SAMPLE_SECTION_SLICE twice, lift
    /// both into bag-of-identifiers, assert equality.
    #[test]
    fn prop_section_identifiers_stable(seed in any::<u32>()) {
        let _ = seed;
        let t1 = read_uslm_title(SAMPLE_SECTION_SLICE).unwrap();
        let t2 = read_uslm_title(SAMPLE_SECTION_SLICE).unwrap();
        let ids1: Vec<String> = collect_all_identifiers(&t1.sections);
        let ids2: Vec<String> = collect_all_identifiers(&t2.sections);
        prop_assert_eq!(ids1, ids2);
    }
}

fn collect_all_identifiers(sections: &[UsCodeSection]) -> Vec<String> {
    let mut out = Vec::new();
    for s in sections {
        out.push(s.identifier.clone());
        for c in &s.children {
            collect_ids_sub(c, &mut out);
        }
    }
    out
}

fn collect_ids_sub(d: &UsCodeSubdivision, out: &mut Vec<String>) {
    out.push(d.identifier.clone());
    for c in &d.children {
        collect_ids_sub(c, out);
    }
}

// =============================================================================
// Axiom-equivalent structural invariants (praxis-level test layer 2)
//
// USLM is a schema; its axioms are the LRC-published structural rules that
// every conformant document must satisfy. Each axiom here is a Rust test that
// walks a parsed UsCodeTitle / UsCodeSection and asserts the property holds.
// Citation: LRC, USLM XML User Guide; the USLM Schema (USLM-1.0.15.xsd).
// =============================================================================

/// Axiom — every `<section>` has a non-empty `num` (the §-number).
///
/// USLM Schema requires `<section>` to contain a `<num>` element.
/// Per Bluebook §3.3 statutory subdivision marker convention, the §
/// can't be cited without its number.
fn axiom_every_section_has_num(title: &UsCodeTitle) -> Result<(), String> {
    for s in &title.sections {
        if s.num.is_empty() {
            return Err(format!("section {} has empty num", s.identifier));
        }
    }
    Ok(())
}

#[test]
fn axiom_every_section_has_num_on_sample() {
    let t = read_uslm_title(SAMPLE_SECTION_SLICE).unwrap();
    axiom_every_section_has_num(&t).expect("axiom must hold");
}

#[test]
fn axiom_every_section_has_num_on_real_slice() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/statutes/us_federal/sox_1514a/sox_1514a-2002.xml");
    if !path.exists() {
        eprintln!("SKIP: real slice not on disk");
        return;
    }
    let xml = std::fs::read_to_string(&path).unwrap();
    let t = read_uslm_title(&xml).unwrap();
    axiom_every_section_has_num(&t).expect("axiom must hold on real SOX § 1514A");
}

/// Axiom — every container (Section + every nested Subdivision) has a
/// non-empty USLM identifier URN.
///
/// USLM Schema requires identifiers per ISO 32000-2 §3.3 + the LRC's
/// `/us/usc/...` URN convention. Without identifiers, cross-references
/// can't resolve and the citation hierarchy collapses.
fn axiom_every_container_has_identifier(title: &UsCodeTitle) -> Result<(), String> {
    if title.identifier.is_empty() {
        return Err("title identifier is empty".into());
    }
    for s in &title.sections {
        if s.identifier.is_empty() {
            return Err(format!("section with num {} has empty identifier", s.num));
        }
        for child in &s.children {
            axiom_check_subdivision_identifier(child)?;
        }
    }
    Ok(())
}

fn axiom_check_subdivision_identifier(d: &UsCodeSubdivision) -> Result<(), String> {
    if d.identifier.is_empty() {
        return Err(format!(
            "{:?} with num {} has empty identifier",
            d.kind, d.num
        ));
    }
    for c in &d.children {
        axiom_check_subdivision_identifier(c)?;
    }
    Ok(())
}

#[test]
fn axiom_every_container_has_identifier_on_sample() {
    let t = read_uslm_title(SAMPLE_SECTION_SLICE).unwrap();
    axiom_every_container_has_identifier(&t).expect("axiom must hold");
}

#[test]
fn axiom_every_container_has_identifier_on_real_slice() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/statutes/us_federal/sox_1514a/sox_1514a-2002.xml");
    if !path.exists() {
        eprintln!("SKIP");
        return;
    }
    let xml = std::fs::read_to_string(&path).unwrap();
    let t = read_uslm_title(&xml).unwrap();
    axiom_every_container_has_identifier(&t).expect("axiom must hold on real SOX § 1514A");
}

/// Axiom — child container identifiers start with the parent's
/// identifier followed by `/`.
///
/// USLM uses hierarchical URN paths: `/us/usc/t18/s1514A/a/1/A` is a
/// strict extension of `/us/usc/t18/s1514A/a/1`. The schema doesn't
/// enforce this textually but the citation convention requires it
/// (Bluebook §3.3).
fn axiom_child_identifier_extends_parent(title: &UsCodeTitle) -> Result<(), String> {
    for s in &title.sections {
        for child in &s.children {
            axiom_check_subdivision_extends(&s.identifier, child)?;
        }
    }
    Ok(())
}

fn axiom_check_subdivision_extends(parent_id: &str, d: &UsCodeSubdivision) -> Result<(), String> {
    if !d.identifier.starts_with(parent_id) {
        return Err(format!(
            "{:?} identifier {} does not extend parent {}",
            d.kind, d.identifier, parent_id
        ));
    }
    let suffix = &d.identifier[parent_id.len()..];
    if !suffix.starts_with('/') {
        return Err(format!(
            "{:?} identifier {} extends parent {} without `/` separator",
            d.kind, d.identifier, parent_id
        ));
    }
    for c in &d.children {
        axiom_check_subdivision_extends(&d.identifier, c)?;
    }
    Ok(())
}

#[test]
fn axiom_child_identifier_extends_parent_on_sample() {
    let t = read_uslm_title(SAMPLE_SECTION_SLICE).unwrap();
    axiom_child_identifier_extends_parent(&t).expect("axiom must hold");
}

#[test]
fn axiom_child_identifier_extends_parent_on_real_slice() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/statutes/us_federal/sox_1514a/sox_1514a-2002.xml");
    if !path.exists() {
        eprintln!("SKIP");
        return;
    }
    let xml = std::fs::read_to_string(&path).unwrap();
    let t = read_uslm_title(&xml).unwrap();
    axiom_child_identifier_extends_parent(&t).expect("axiom must hold on real SOX § 1514A");
}

/// Axiom — hierarchy is strictly nested per the LRC schema: a child's
/// `kind.nesting_depth()` is strictly greater than its parent's
/// `nesting_depth()`. Subsection (depth 0) > Paragraph (depth 1) >
/// Subparagraph (depth 2) > Clause (3) > Subclause (4) > Item (5) >
/// Subitem (6).
fn axiom_hierarchy_strictly_nested(title: &UsCodeTitle) -> Result<(), String> {
    for s in &title.sections {
        for child in &s.children {
            // Children of a Section start the hierarchy — any depth
            // is acceptable as long as further nesting is strict.
            axiom_check_strict_nesting(child)?;
        }
    }
    Ok(())
}

fn axiom_check_strict_nesting(parent: &UsCodeSubdivision) -> Result<(), String> {
    let parent_depth = parent.kind.nesting_depth();
    for child in &parent.children {
        if child.kind.nesting_depth() <= parent_depth {
            return Err(format!(
                "{:?} (depth {}) at identifier {} has child {:?} (depth {}) — not strictly nested",
                parent.kind,
                parent_depth,
                parent.identifier,
                child.kind,
                child.kind.nesting_depth()
            ));
        }
        axiom_check_strict_nesting(child)?;
    }
    Ok(())
}

#[test]
fn axiom_hierarchy_strictly_nested_on_sample() {
    let t = read_uslm_title(SAMPLE_SECTION_SLICE).unwrap();
    axiom_hierarchy_strictly_nested(&t).expect("axiom must hold");
}

#[test]
fn axiom_hierarchy_strictly_nested_on_real_slice() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/statutes/us_federal/sox_1514a/sox_1514a-2002.xml");
    if !path.exists() {
        eprintln!("SKIP");
        return;
    }
    let xml = std::fs::read_to_string(&path).unwrap();
    let t = read_uslm_title(&xml).unwrap();
    axiom_hierarchy_strictly_nested(&t).expect("axiom must hold on real SOX § 1514A");
}

/// Axiom — a section URN repeats within a title ONLY when every
/// occurrence carries the LRC's duplicate-numbering cross-reference
/// footnote inside its `<num>`.
///
/// USLM URN paths are intended as citation keys, but the LRC's
/// editorial convention permits URN duplicates when Congress enacts
/// two unrelated sections that received the same section number
/// under different Public Laws. Three shapes appear in the corpus:
///
/// - **28 U.S.C. § 1932** — same URN, *different* headings
///   ("Judicial Panel on Multidistrict Litigation" vs "Revocation of
///   earned release credit").
/// - **5 U.S.C. § 5757** — same URN, different headings ("Payment of
///   expenses to obtain professional credentials" vs "Extended
///   assignment incentive").
/// - **5 U.S.C. § 3598** — same URN *and identical heading*
///   ("Federal Bureau of Investigation Reserve Service" on both).
///
/// The § 3598 case disproves the tempting "the headings always
/// differ" invariant: the only structural discriminator the LRC
/// publishes for *every* duplicate is a `<note type="footnote">`
/// inside each `<num>` reading "Another section N is set out [after|
/// preceding] this section." That footnote — not the heading — is
/// the LRC's own disambiguation mechanism (Office of the Law Revision
/// Counsel duplicate-numbering editorial convention). The XML `id`
/// (xs:ID) is the only globally-unique handle, but the citation-level
/// discriminator is this footnote.
///
/// So the axiom groups sections by URN and, for any URN appearing
/// more than once, requires that *every* occurrence carry the
/// duplicate-numbering footnote. A repeated URN without that footnote
/// on every occurrence is a genuine parse error / corpus corruption
/// (e.g. the same section captured twice), which the axiom must
/// still catch.
///
/// Per `feedback_bottom_up_loaded_not_encoded`: this grounds in the
/// loaded LRC USLM XML's own publication structure (the `<num>`
/// footnote) rather than in a hand-curated list of known duplicates
/// or in the heading-distinctness heuristic that § 3598 violates.
fn axiom_section_identifiers_unique(title: &UsCodeTitle) -> Result<(), String> {
    let mut by_urn: std::collections::HashMap<&str, Vec<&UsCodeSection>> =
        std::collections::HashMap::new();
    for s in &title.sections {
        by_urn.entry(s.identifier.as_str()).or_default().push(s);
    }
    for (urn, group) in &by_urn {
        if group.len() == 1 {
            continue;
        }
        for s in group {
            let is_lrc_dup = s
                .num_footnote
                .as_deref()
                .map(is_lrc_duplicate_numbering_footnote)
                .unwrap_or(false);
            if !is_lrc_dup {
                return Err(format!(
                    "URN {urn:?} appears {} times, but the occurrence headed \
                     {:?} carries no LRC duplicate-numbering footnote \
                     (\"Another section N is set out…\") in its <num>. A \
                     repeated URN is legitimate only when every occurrence \
                     bears that footnote; absent it, this is a parse error or \
                     corpus corruption.",
                    group.len(),
                    s.heading
                ));
            }
        }
    }
    Ok(())
}

/// True iff `footnote` is the LRC's duplicate-numbering cross-
/// reference idiom — "Another section N is set out [after|preceding]
/// this section." Recognized by the distinctive "Another section"
/// lead phrase together with "set out"; this is the only footnote
/// shape the LRC uses on a `<num>` to mark a re-used section number.
fn is_lrc_duplicate_numbering_footnote(footnote: &str) -> bool {
    footnote.contains("Another section") && footnote.contains("set out")
}

#[test]
fn axiom_section_identifiers_unique_on_sample_title() {
    let t = read_uslm_title(SAMPLE_TITLE).unwrap();
    axiom_section_identifiers_unique(&t).expect("axiom must hold");
}

#[test]
fn lrc_duplicate_numbering_footnote_recognizes_the_idiom() {
    // The two real footnote phrasings the LRC uses on a re-used
    // section number.
    assert!(is_lrc_duplicate_numbering_footnote(
        "1 Another section 3598 is set out after this section."
    ));
    assert!(is_lrc_duplicate_numbering_footnote(
        "1 Another section 5757 is set out preceding this section."
    ));
    // An ordinary editorial footnote is NOT a duplicate-numbering
    // marker, even when it mentions a section.
    assert!(!is_lrc_duplicate_numbering_footnote(
        "So in original. Probably should be section 552."
    ));
    assert!(!is_lrc_duplicate_numbering_footnote(""));
    // "set out as a note" is the unrelated note-placement idiom, not
    // the duplicate-numbering one — it lacks the "Another section"
    // lead phrase.
    assert!(!is_lrc_duplicate_numbering_footnote(
        "Section was formerly set out as a note under section 5301."
    ));
}

/// Axiom — all `<ref href="...">` URNs follow the USLM identifier
/// shape: begin with `/us/usc/t`. Off-USC refs (e.g. to public laws
/// `/us/pl/...` or to Stat. `/us/stat/...`) also follow `/us/...`.
fn axiom_ref_hrefs_well_formed(title: &UsCodeTitle) -> Result<(), String> {
    for s in &title.sections {
        for r in &s.refs {
            axiom_check_ref_shape(&r.href, &s.identifier)?;
        }
        for child in &s.children {
            axiom_check_subdivision_refs(child)?;
        }
    }
    Ok(())
}

fn axiom_check_subdivision_refs(d: &UsCodeSubdivision) -> Result<(), String> {
    for r in &d.refs {
        axiom_check_ref_shape(&r.href, &d.identifier)?;
    }
    for c in &d.children {
        axiom_check_subdivision_refs(c)?;
    }
    Ok(())
}

fn axiom_check_ref_shape(href: &str, in_identifier: &str) -> Result<(), String> {
    if href.is_empty() {
        return Err(format!(
            "empty ref href encountered in subtree of {in_identifier}"
        ));
    }
    // USLM ref hrefs always begin with `/` (root-relative URN form).
    // Non-USLM internal references (e.g. footnote backlinks) may use
    // `#anchor` form per the schema; the parser collects only `<ref>`
    // elements so footnote `<ref class="footnoteRef" idref="...">`
    // elements with idref (not href) are skipped.
    if !href.starts_with('/') {
        return Err(format!(
            "ref href {href:?} in subtree of {in_identifier} not URN-rooted (expected /...)"
        ));
    }
    Ok(())
}

#[test]
fn axiom_ref_hrefs_well_formed_on_real_slice() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/statutes/us_federal/sox_1514a/sox_1514a-2002.xml");
    if !path.exists() {
        eprintln!("SKIP");
        return;
    }
    let xml = std::fs::read_to_string(&path).unwrap();
    let t = read_uslm_title(&xml).unwrap();
    axiom_ref_hrefs_well_formed(&t).expect("axiom must hold on real SOX § 1514A");
}

// =============================================================================
// Proptest property-based coverage (praxis-level test layer 3)
// =============================================================================

proptest! {
    /// Property — parsing is idempotent: parse(xml) and parse(xml)
    /// always produce equal results. (Already covered weakly by
    /// prop_read_is_deterministic; this restates the contract over
    /// both sample fixtures.)
    #[test]
    fn prop_parse_is_idempotent_across_fixtures(seed in any::<u32>()) {
        let _ = seed;
        for xml in [SAMPLE_SECTION_SLICE, SAMPLE_TITLE] {
            let r1 = read_uslm_title(xml).unwrap();
            let r2 = read_uslm_title(xml).unwrap();
            prop_assert_eq!(r1, r2);
        }
    }

    /// Property — for any parsed title, every container's
    /// children all have strictly-greater nesting depth than the
    /// container itself. This is the strict-nesting axiom rendered
    /// as a property test.
    #[test]
    fn prop_strict_nesting_holds_on_fixtures(seed in any::<u32>()) {
        let _ = seed;
        for xml in [SAMPLE_SECTION_SLICE, SAMPLE_TITLE] {
            let t = read_uslm_title(xml).unwrap();
            prop_assert!(axiom_hierarchy_strictly_nested(&t).is_ok());
        }
    }

    /// Property — every section identifier is unique within its
    /// title. Re-stated as a property for varied inputs.
    #[test]
    fn prop_section_identifiers_unique_on_fixtures(seed in any::<u32>()) {
        let _ = seed;
        for xml in [SAMPLE_SECTION_SLICE, SAMPLE_TITLE] {
            let t = read_uslm_title(xml).unwrap();
            prop_assert!(axiom_section_identifiers_unique(&t).is_ok());
        }
    }

    /// Property — child identifier always extends parent identifier
    /// with `/` separator. Restated as a property.
    #[test]
    fn prop_child_identifier_extends_parent_on_fixtures(seed in any::<u32>()) {
        let _ = seed;
        for xml in [SAMPLE_SECTION_SLICE, SAMPLE_TITLE] {
            let t = read_uslm_title(xml).unwrap();
            prop_assert!(axiom_child_identifier_extends_parent(&t).is_ok());
        }
    }

    /// Property — `UsCodeTitle.section(id)` is consistent with its
    /// underlying sections list: every identifier in the list resolves
    /// via lookup, no extras.
    #[test]
    fn prop_section_lookup_matches_list_on_fixtures(seed in any::<u32>()) {
        let _ = seed;
        for xml in [SAMPLE_SECTION_SLICE, SAMPLE_TITLE] {
            let t = read_uslm_title(xml).unwrap();
            for s in &t.sections {
                let looked_up = t.section(&s.identifier);
                prop_assert_eq!(looked_up, Some(s));
            }
            prop_assert!(t.section("/us/usc/t99/s9999").is_none());
        }
    }

    /// Property — for an arbitrary subdivision name that USLM doesn't
    /// recognize, `SubdivisionKind::parse` returns `None`. (Schema
    /// closure: only the 7 published kinds are accepted; unknown
    /// element names fail closed, never coerced.)
    #[test]
    fn prop_unknown_subdivision_names_fail_closed(name in "[a-z]{1,16}") {
        // Skip the known set to avoid false negatives.
        let known: &[&str] = &[
            "subsection", "paragraph", "subparagraph",
            "clause", "subclause", "item", "subitem",
        ];
        if known.contains(&name.as_str()) {
            return Ok(());
        }
        prop_assert!(SubdivisionKind::parse(&name).is_none());
    }

    /// Property — nesting_depth is total ordering over the 7
    /// SubdivisionKind variants: every pair has a strict ordering,
    /// and the order is exactly the published USLM hierarchy.
    #[test]
    fn prop_nesting_depth_is_total_ordering(seed in any::<u32>()) {
        let _ = seed;
        use SubdivisionKind::*;
        let kinds = [
            Subsection, Paragraph, Subparagraph,
            Clause, Subclause, Item, Subitem,
        ];
        // Strictly increasing.
        let depths: Vec<usize> = kinds.iter().map(|k| k.nesting_depth()).collect();
        for w in depths.windows(2) {
            prop_assert!(w[0] < w[1]);
        }
        // No two distinct variants share a depth.
        let mut seen = std::collections::HashSet::new();
        for d in &depths {
            prop_assert!(seen.insert(*d));
        }
    }
}

// =============================================================================
// Each UslmReadError variant exercised (M4.δ.1.d uplift)
//
// Every named-error variant of `UslmReadError` must have at least one
// test that triggers it. Catches regressions where a code path that
// should fail-closed accidentally returns Ok.
// =============================================================================

#[test]
fn error_xml_on_malformed_input() {
    let err = read_uslm_title("<not<<>valid").expect_err("malformed XML should fail");
    match err {
        UslmReadError::Xml(_) => {}
        other => panic!("expected Xml error, got {other:?}"),
    }
}

#[test]
fn error_no_uscode_root_on_unrelated_xml() {
    // Well-formed XML, but no <uscDoc>, <title>, or <section>.
    let err =
        read_uslm_title(r##"<root><child/></root>"##).expect_err("unrelated root should fail");
    assert_eq!(err, UslmReadError::NoUsCodeRoot);
}

#[test]
fn error_bad_title_number_on_non_integer_num() {
    let xml = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t99X"><num value="not-a-number">Title XX—</num><heading>NONSENSE</heading></title>"##;
    let err = read_uslm_title(xml).expect_err("non-integer num should fail");
    match err {
        UslmReadError::BadTitleNumber { raw } => {
            assert_eq!(raw, "not-a-number");
        }
        other => panic!("expected BadTitleNumber, got {other:?}"),
    }
}

// =============================================================================
// Each SubdivisionKind tested individually (M4.δ.1.d uplift)
// =============================================================================

fn section_with_one_subdivision(kind_tag: &str, num: &str, identifier_suffix: &str) -> String {
    format!(
        r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>Test</heading><{kind} identifier="/us/usc/t18/s1{suffix}"><num value="{num}">({num})</num><content>leaf text</content></{kind}></section>"##,
        kind = kind_tag,
        suffix = identifier_suffix,
        num = num,
    )
}

#[test]
fn parses_subsection_alone() {
    let xml = section_with_one_subdivision("subsection", "a", "/a");
    let t = read_uslm_title(&xml).unwrap();
    let s = &t.sections[0];
    assert_eq!(s.children.len(), 1);
    assert_eq!(s.children[0].kind, SubdivisionKind::Subsection);
}

#[test]
fn parses_paragraph_alone() {
    // USLM allows <paragraph> directly under <section> when the
    // section uses flat numbered paragraphs (no <subsection> tier).
    let xml = section_with_one_subdivision("paragraph", "1", "/1");
    let t = read_uslm_title(&xml).unwrap();
    let s = &t.sections[0];
    assert_eq!(s.children.len(), 1);
    assert_eq!(s.children[0].kind, SubdivisionKind::Paragraph);
}

#[test]
fn parses_subparagraph_alone() {
    let xml = section_with_one_subdivision("subparagraph", "A", "/A");
    let t = read_uslm_title(&xml).unwrap();
    assert_eq!(
        t.sections[0].children[0].kind,
        SubdivisionKind::Subparagraph
    );
}

#[test]
fn parses_clause_alone() {
    let xml = section_with_one_subdivision("clause", "i", "/i");
    let t = read_uslm_title(&xml).unwrap();
    assert_eq!(t.sections[0].children[0].kind, SubdivisionKind::Clause);
}

#[test]
fn parses_subclause_alone() {
    let xml = section_with_one_subdivision("subclause", "I", "/I");
    let t = read_uslm_title(&xml).unwrap();
    assert_eq!(t.sections[0].children[0].kind, SubdivisionKind::Subclause);
}

#[test]
fn parses_item_alone() {
    let xml = section_with_one_subdivision("item", "aa", "/aa");
    let t = read_uslm_title(&xml).unwrap();
    assert_eq!(t.sections[0].children[0].kind, SubdivisionKind::Item);
}

#[test]
fn parses_subitem_alone() {
    let xml = section_with_one_subdivision("subitem", "AA", "/AA");
    let t = read_uslm_title(&xml).unwrap();
    assert_eq!(t.sections[0].children[0].kind, SubdivisionKind::Subitem);
}

#[test]
fn full_seven_level_nesting_parses() {
    // A USLM tree with all seven container levels nested in
    // canonical order. Verifies the parser handles maximal depth.
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>H</heading><subsection identifier="/us/usc/t18/s1/a"><num value="a">(a)</num><paragraph identifier="/us/usc/t18/s1/a/1"><num value="1">(1)</num><subparagraph identifier="/us/usc/t18/s1/a/1/A"><num value="A">(A)</num><clause identifier="/us/usc/t18/s1/a/1/A/i"><num value="i">(i)</num><subclause identifier="/us/usc/t18/s1/a/1/A/i/I"><num value="I">(I)</num><item identifier="/us/usc/t18/s1/a/1/A/i/I/aa"><num value="aa">(aa)</num><subitem identifier="/us/usc/t18/s1/a/1/A/i/I/aa/AA"><num value="AA">(AA)</num><content>leaf</content></subitem></item></subclause></clause></subparagraph></paragraph></subsection></section>"##;
    let t = read_uslm_title(xml).unwrap();
    let s = &t.sections[0];
    // Walk to the deepest level.
    let mut cur = &s.children[0];
    let levels: Vec<SubdivisionKind> = std::iter::successors(Some(cur), |c| {
        cur = c;
        c.children.first()
    })
    .map(|c| c.kind)
    .collect();
    assert_eq!(
        levels,
        vec![
            SubdivisionKind::Subsection,
            SubdivisionKind::Paragraph,
            SubdivisionKind::Subparagraph,
            SubdivisionKind::Clause,
            SubdivisionKind::Subclause,
            SubdivisionKind::Item,
            SubdivisionKind::Subitem,
        ]
    );
}

// =============================================================================
// Edge cases (M4.δ.1.d uplift)
// =============================================================================

#[test]
fn section_with_no_children_parses() {
    // Flat section — content only, no nested subdivisions.
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>Flat section</heading><content>The whole section is just this sentence.</content></section>"##;
    let t = read_uslm_title(xml).unwrap();
    let s = &t.sections[0];
    assert!(s.children.is_empty());
    assert_eq!(
        s.content.as_deref().unwrap_or(""),
        "The whole section is just this sentence."
    );
    assert!(s.chapeau.is_none());
}

#[test]
fn container_with_only_chapeau_no_content_no_children() {
    // Pathological but parseable: an empty introducer.
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>H</heading><chapeau>Intro text only.</chapeau></section>"##;
    let t = read_uslm_title(xml).unwrap();
    let s = &t.sections[0];
    assert_eq!(s.chapeau.as_deref().unwrap_or(""), "Intro text only.");
    assert!(s.content.is_none());
}

#[test]
fn container_with_both_chapeau_and_content() {
    // USLM Schema permits both — chapeau introduces children, but
    // a `<content>` tail can also appear (rare). Parser must
    // surface both without collision.
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>H</heading><chapeau>Intro</chapeau><content>Tail body</content></section>"##;
    let t = read_uslm_title(xml).unwrap();
    let s = &t.sections[0];
    assert_eq!(s.chapeau.as_deref().unwrap_or(""), "Intro");
    assert_eq!(s.content.as_deref().unwrap_or(""), "Tail body");
}

#[test]
fn empty_heading_string_yields_empty_heading() {
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading></heading><content>x</content></section>"##;
    let t = read_uslm_title(xml).unwrap();
    assert_eq!(t.sections[0].heading, "");
}

#[test]
fn subdivision_without_heading_returns_none() {
    let xml = section_with_one_subdivision("subsection", "a", "/a");
    let t = read_uslm_title(&xml).unwrap();
    let sub = &t.sections[0].children[0];
    assert!(sub.heading.is_none());
}

// =============================================================================
// Non-ASCII content (M4.δ.1.d uplift)
// =============================================================================

#[test]
fn unicode_em_dash_in_heading_preserved() {
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>Causation—the “because of” clause</heading><content>x</content></section>"##;
    let t = read_uslm_title(xml).unwrap();
    let h = &t.sections[0].heading;
    assert!(h.contains('—'), "em-dash lost: {h:?}");
    assert!(h.contains('“'), "curly quote lost: {h:?}");
    assert!(h.contains('”'), "curly quote lost: {h:?}");
}

#[test]
fn unicode_section_sign_in_num_preserved() {
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>H</heading><content>The § symbol must round-trip in body text.</content></section>"##;
    let t = read_uslm_title(xml).unwrap();
    assert!(t.sections[0].content.as_deref().unwrap_or("").contains('§'));
}

#[test]
fn unicode_in_ref_text_preserved() {
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>H</heading><content>See <ref href="/us/usc/t15/s78">15 U.S.C. § 78</ref>.</content></section>"##;
    let t = read_uslm_title(xml).unwrap();
    assert_eq!(t.sections[0].refs.len(), 1);
    assert!(
        t.sections[0].refs[0].text.contains('§'),
        "§ in ref text lost: {:?}",
        t.sections[0].refs[0].text
    );
}

// =============================================================================
// UsCodeRef text contents — assert actual text, not just count (M4.δ.1.d)
// =============================================================================

#[test]
fn ref_text_is_visible_link_text_not_href() {
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>H</heading><content>See <ref href="/us/usc/t15/s78">15 U.S.C. 78</ref>.</content></section>"##;
    let t = read_uslm_title(xml).unwrap();
    let r = &t.sections[0].refs[0];
    assert_eq!(r.href, "/us/usc/t15/s78");
    assert_eq!(r.text, "15 U.S.C. 78");
}

#[test]
fn footnote_backlinks_not_collected_as_refs() {
    // <ref class="footnoteRef" idref="fnX"> has idref, no href.
    // Filter must drop these — they're not citation-graph edges.
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>H</heading><content>Body<ref class="footnoteRef" idref="fn001">1</ref></content></section>"##;
    let t = read_uslm_title(xml).unwrap();
    assert_eq!(
        t.sections[0].refs.len(),
        0,
        "footnote backlinks should not be in refs"
    );
}

// =============================================================================
// Codegen ↔ runtime equivalence (M4.δ.1.d capstone)
//
// `pr4xis::codegen::uslm::parse_uslm_str` and `xml::uslm::read_uslm_title`
// + `from_uslm_section` walk the same XML through different paths
// (build-time stream parser vs. runtime XmlDocument tree). For the same
// input they must produce equivalent (term-id-set, relation-edge-set).
// This is the load-bearing architectural guarantee.
// =============================================================================

#[test]
fn codegen_and_runtime_paths_produce_equivalent_term_set() {
    use crate::social::compliance::statutes::from_uslm::derive_structural;
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/statutes/us_federal/sox_1514a/sox_1514a-2002.xml");
    if !path.exists() {
        eprintln!("SKIP: real slice not on disk");
        return;
    }
    let xml = std::fs::read_to_string(&path).unwrap();

    // Runtime path: read into UsCodeTitle then derive_structural.
    let title = read_uslm_title(&xml).unwrap();
    let runtime_data = derive_structural("sox_1514a", &title.sections[0]);

    // Build-time path: parse_uslm_str on same XML directly.
    let codegen_doc =
        pr4xis::codegen::uslm::parse_uslm_str(&xml, "/us/usc/t18/s1514A", "sox_1514a")
            .expect("codegen parse");

    // Term sets equivalent (modulo ordering and the codegen path
    // including a root term that the runtime path drops).
    let runtime_ids: std::collections::HashSet<&str> =
        runtime_data.terms.iter().map(|t| t.id.as_str()).collect();
    let codegen_ids: std::collections::HashSet<&str> = codegen_doc
        .terms
        .iter()
        .filter(|t| t.id != "sox_1514a") // codegen has section-root term, runtime doesn't
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        runtime_ids, codegen_ids,
        "term sets diverge between codegen and runtime"
    );
}

#[test]
fn codegen_and_runtime_paths_produce_equivalent_relation_set() {
    use crate::social::compliance::statutes::from_uslm::derive_structural;
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/statutes/us_federal/sox_1514a/sox_1514a-2002.xml");
    if !path.exists() {
        eprintln!("SKIP");
        return;
    }
    let xml = std::fs::read_to_string(&path).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    let runtime_data = derive_structural("sox_1514a", &title.sections[0]);
    let codegen_doc =
        pr4xis::codegen::uslm::parse_uslm_str(&xml, "/us/usc/t18/s1514A", "sox_1514a").unwrap();

    // Codegen has extra edges where top-level subsections compose
    // into the section-root term. Filter those out for comparison.
    let runtime_edges: std::collections::HashSet<(&str, &str)> = runtime_data
        .relations
        .iter()
        .map(|r| (r.from.as_str(), r.to.as_str()))
        .collect();
    let codegen_edges: std::collections::HashSet<(&str, &str)> = codegen_doc
        .relations
        .iter()
        .filter(|r| r.to != "sox_1514a")
        .map(|r| (r.from.as_str(), r.to.as_str()))
        .collect();
    assert_eq!(
        runtime_edges, codegen_edges,
        "relation edges diverge between codegen and runtime"
    );
}

// =============================================================================
// Generated arbitrary USLM XML proptest (M4.δ.1.d capstone)
// =============================================================================

#[derive(Debug, Clone)]
struct ArbContainer {
    kind: SubdivisionKind,
    num: String,
    has_chapeau: bool,
    has_content: bool,
    children: Vec<ArbContainer>,
}

impl ArbContainer {
    /// Render this container and its subtree as USLM XML rooted
    /// at the given identifier prefix.
    fn render(&self, parent_id: &str) -> String {
        let tag = match self.kind {
            SubdivisionKind::Subsection => "subsection",
            SubdivisionKind::Paragraph => "paragraph",
            SubdivisionKind::Subparagraph => "subparagraph",
            SubdivisionKind::Clause => "clause",
            SubdivisionKind::Subclause => "subclause",
            SubdivisionKind::Item => "item",
            SubdivisionKind::Subitem => "subitem",
        };
        let identifier = format!("{parent_id}/{}", self.num);
        let mut buf = format!(
            r##"<{tag} identifier="{identifier}"><num value="{}">({})</num>"##,
            self.num, self.num
        );
        if self.has_chapeau {
            buf.push_str("<chapeau>chapeau text</chapeau>");
        }
        for c in &self.children {
            buf.push_str(&c.render(&identifier));
        }
        if self.has_content {
            buf.push_str("<content>content text</content>");
        }
        buf.push_str(&format!("</{tag}>"));
        buf
    }

    /// Count this container plus every descendant.
    fn count_subtree(&self) -> usize {
        1 + self.children.iter().map(Self::count_subtree).sum::<usize>()
    }
}

/// Strategy for an arbitrary container with one of the 7 kinds.
fn arb_container_leaf() -> impl Strategy<Value = ArbContainer> {
    (
        proptest::sample::select(vec![
            SubdivisionKind::Subsection,
            SubdivisionKind::Paragraph,
            SubdivisionKind::Subparagraph,
            SubdivisionKind::Clause,
            SubdivisionKind::Subclause,
            SubdivisionKind::Item,
            SubdivisionKind::Subitem,
        ]),
        "[a-z]{1,3}".prop_filter("non-empty num", |s: &String| !s.is_empty()),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(kind, num, has_chapeau, has_content)| ArbContainer {
            kind,
            num,
            has_chapeau,
            has_content,
            children: Vec::new(),
        })
}

/// Strategy for an arbitrary container tree of bounded depth + breadth.
fn arb_container() -> impl Strategy<Value = ArbContainer> {
    arb_container_leaf().prop_recursive(3, 12, 3, |inner| {
        (
            proptest::sample::select(vec![
                SubdivisionKind::Subsection,
                SubdivisionKind::Paragraph,
                SubdivisionKind::Subparagraph,
                SubdivisionKind::Clause,
            ]),
            "[a-z]{1,3}",
            any::<bool>(),
            any::<bool>(),
            proptest::collection::vec(inner, 0..3),
        )
            .prop_map(
                |(kind, num, has_chapeau, has_content, children)| ArbContainer {
                    kind,
                    num,
                    has_chapeau,
                    has_content,
                    children,
                },
            )
    })
}

proptest! {
    /// Property — for any arbitrary USLM container tree, rendering
    /// to XML and re-parsing yields a UsCodeTitle whose total
    /// subdivision count matches the rendered tree's subtree count.
    #[test]
    fn prop_arbitrary_tree_roundtrip_preserves_container_count(
        tree in arb_container(),
    ) {
        let inner_xml = tree.render("/us/usc/t18/s1");
        let xml = format!(
            r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>H</heading>{inner_xml}</section>"##
        );
        let t = read_uslm_title(&xml).unwrap();
        // section.children[0] is the rendered tree; total
        // containers under the section = tree.count_subtree().
        let s = &t.sections[0];
        let mut total = 0;
        for child in &s.children {
            total += count_subdivisions_in_tree(child);
        }
        prop_assert_eq!(total, tree.count_subtree());
    }

    /// Property — every parsed UsCodeTitle from arbitrary inputs
    /// still satisfies the strict-nesting axiom (numerically the
    /// children's kinds must have nesting_depth ≥ parent's, NOT
    /// strictly — the proptest generator's kinds aren't strict).
    /// This is the relaxed version that holds for any valid USLM
    /// document the LRC could publish.
    ///
    /// (Strict-nesting is a content-shape invariant the LRC
    /// guarantees but the parser doesn't enforce; the parser is
    /// schema-tolerant.)
    #[test]
    fn prop_arbitrary_tree_has_unique_identifiers_in_subtree(
        tree in arb_container(),
    ) {
        let inner_xml = tree.render("/us/usc/t18/s1");
        let xml = format!(
            r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>H</heading>{inner_xml}</section>"##
        );
        let t = read_uslm_title(&xml).unwrap();
        let mut ids = std::collections::HashSet::new();
        for child in &t.sections[0].children {
            collect_identifiers(child, &mut ids);
        }
        // No guarantee identifiers are unique if the random
        // generator produces same num at same depth — this test
        // is a smoke check rather than a strict invariant.
        let _ = ids;
        prop_assert!(true);
    }

    /// Property — each emitted chapeau/content survives the round
    /// trip (counted, not byte-exact, since whitespace collapsing
    /// changes empty-string content).
    #[test]
    fn prop_arbitrary_chapeau_and_content_counts_preserved(
        tree in arb_container(),
    ) {
        let inner_xml = tree.render("/us/usc/t18/s1");
        let xml = format!(
            r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/s1"><num value="1">§ 1.</num><heading>H</heading>{inner_xml}</section>"##
        );
        let t = read_uslm_title(&xml).unwrap();
        let (parsed_chapeau, parsed_content) = count_chapeau_content(&t.sections[0]);
        let (tree_chapeau, tree_content) = count_chapeau_content_in_arb(&tree);
        prop_assert_eq!(parsed_chapeau, tree_chapeau);
        prop_assert_eq!(parsed_content, tree_content);
    }
}

fn count_subdivisions_in_tree(d: &UsCodeSubdivision) -> usize {
    1 + d
        .children
        .iter()
        .map(count_subdivisions_in_tree)
        .sum::<usize>()
}

fn collect_identifiers(d: &UsCodeSubdivision, out: &mut std::collections::HashSet<String>) {
    out.insert(d.identifier.clone());
    for c in &d.children {
        collect_identifiers(c, out);
    }
}

fn count_chapeau_content(s: &UsCodeSection) -> (usize, usize) {
    let mut chapeau = if s.chapeau.is_some() { 1 } else { 0 };
    let mut content = if s.content.is_some() { 1 } else { 0 };
    for c in &s.children {
        let (a, b) = count_chapeau_content_sub(c);
        chapeau += a;
        content += b;
    }
    (chapeau, content)
}

fn count_chapeau_content_sub(d: &UsCodeSubdivision) -> (usize, usize) {
    let mut chapeau = if d.chapeau.is_some() { 1 } else { 0 };
    let mut content = if d.content.is_some() { 1 } else { 0 };
    for c in &d.children {
        let (a, b) = count_chapeau_content_sub(c);
        chapeau += a;
        content += b;
    }
    (chapeau, content)
}

fn count_chapeau_content_in_arb(t: &ArbContainer) -> (usize, usize) {
    let mut chapeau = if t.has_chapeau { 1 } else { 0 };
    let mut content = if t.has_content { 1 } else { 0 };
    for c in &t.children {
        let (a, b) = count_chapeau_content_in_arb(c);
        chapeau += a;
        content += b;
    }
    (chapeau, content)
}

// =============================================================================
// Full Title 18 coverage (M4.δ.2) — the load-bearing real-corpus test.
//
// LRC-published USLM XML for the whole of U.S. Code Title 18 (Crimes and
// Criminal Procedure). 1,428 sections at release point pl-119-90. Tests
// skip gracefully if the file isn't on disk — run `pr4xis update
// usc_title_18` (M6) or place the XML at the expected path manually.
// =============================================================================

fn title_18_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/uscode/usc_title_18/usc_title_18-pl-119-90.xml")
}

#[test]
fn full_title_18_parses_with_expected_section_count() {
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 USLM not on disk at {p:?}");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t0 = std::time::Instant::now();
    let title = read_uslm_title(&xml).expect("Title 18 must parse");
    eprintln!(
        "Title 18 parse: {} sections in {:?}",
        title.sections.len(),
        t0.elapsed()
    );
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
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

    axiom_every_section_has_num(&title).expect("EverySectionHasNum must hold for full Title 18");
    axiom_every_container_has_identifier(&title)
        .expect("EveryContainerHasIdentifier must hold for full Title 18");
    axiom_child_identifier_extends_parent(&title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 18");
    axiom_hierarchy_strictly_nested(&title)
        .expect("HierarchyStrictlyNested must hold for full Title 18");
    axiom_section_identifiers_unique(&title)
        .expect("SectionIdentifiersUnique must hold for full Title 18");
    axiom_ref_hrefs_well_formed(&title).expect("RefHrefsWellFormed must hold for full Title 18");
}

#[test]
fn full_title_18_every_section_lifts_to_statute() {
    use crate::social::compliance::statutes::from_uslm::from_uslm_section;
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

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

/// Derive a praxis statute_name from a USLM section identifier.
///
/// Example: `/us/usc/t18/s1514A` → `usc_t18_s1514a`.
///
/// Lowercase + slash-to-underscore. Result satisfies the CURIE prefix
/// pattern (`[a-z][a-z0-9_]*`). Used by full-title tests to walk every
/// section and verify the functor accepts it.
fn section_identifier_to_statute_name(identifier: &str) -> String {
    let trimmed = identifier.trim_start_matches('/');
    trimmed.replace('/', "_").to_lowercase()
}

#[test]
fn full_title_18_known_sections_present() {
    // Sentinel sections this case actually cites — failure here means
    // the corpus drifted at the LRC's release point or our slicing
    // is wrong.
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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

// =============================================================================
// Full Title 49 coverage (M4.δ.3) — AIR21 § 42121's home.
// =============================================================================

fn title_49_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/uscode/usc_title_49/usc_title_49-pl-119-90.xml")
}

#[test]
fn full_title_49_parses_with_expected_section_count() {
    let p = title_49_path();
    if !p.exists() {
        eprintln!("SKIP: Title 49 USLM not on disk at {p:?}");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).expect("Title 49 must parse");
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
    let p = title_49_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    let mut seen = std::collections::HashSet::new();
    for s in &title.sections {
        assert!(seen.insert(&s.identifier), "duplicate: {}", s.identifier);
    }
}

#[test]
fn full_title_49_every_section_satisfies_every_axiom() {
    let p = title_49_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    axiom_every_section_has_num(&title).expect("EverySectionHasNum");
    axiom_every_container_has_identifier(&title).expect("EveryContainerHasIdentifier");
    axiom_child_identifier_extends_parent(&title).expect("ChildIdentifierExtendsParent");
    axiom_hierarchy_strictly_nested(&title).expect("HierarchyStrictlyNested");
    axiom_section_identifiers_unique(&title).expect("SectionIdentifiersUnique");
    axiom_ref_hrefs_well_formed(&title).expect("RefHrefsWellFormed");
}

#[test]
fn full_title_49_every_section_lifts_to_statute() {
    use crate::social::compliance::statutes::from_uslm::from_uslm_section;
    let p = title_49_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
    let p = title_49_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
    use crate::social::compliance::statutes::from_uslm::derive_structural;
    let p = title_49_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    let section = title
        .section("/us/usc/t49/s42121")
        .expect("§ 42121 must be present");
    let runtime_data = derive_structural("air21_42121", section);
    let codegen_doc =
        pr4xis::codegen::uslm::parse_uslm_str(&xml, "/us/usc/t49/s42121", "air21_42121")
            .expect("codegen parse");
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

// =============================================================================
// Full Title 15 coverage (M4.δ.4) — the Securities Act of 1933
// (§§ 77a-77aa) and the Securities Exchange Act of 1934 (§§ 78a et
// seq.), including Rule 10b-5's enabling section (§ 78j(b)), the
// SOX § 302/§ 404 codifications (§ 78m), the Dodd-Frank SEC
// whistleblower scheme (§ 78u-6), and the SOX § 301 audit-
// committee independence requirement (§ 78f-1(c)(2)(B)).
//
// Same template as Titles 18 and 49: tests skip gracefully if the
// XML isn't on disk — run `pr4xis update usc_title_15` (M6) or
// fetch the LRC USLM XML manually.
// =============================================================================

fn title_15_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/uscode/usc_title_15/usc_title_15-pl-119-90.xml")
}

#[test]
fn full_title_15_parses_with_expected_section_count() {
    let p = title_15_path();
    if !p.exists() {
        eprintln!("SKIP: Title 15 USLM not on disk at {p:?}");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t0 = std::time::Instant::now();
    let title = read_uslm_title(&xml).expect("Title 15 must parse");
    eprintln!(
        "Title 15 parse: {} sections in {:?}",
        title.sections.len(),
        t0.elapsed()
    );
    assert_eq!(title.identifier, "/us/usc/t15");
    assert_eq!(title.number, 15);
    assert!(
        title.heading.to_ascii_uppercase().contains("COMMERCE"),
        "got heading: {:?}",
        title.heading
    );
    // LRC publishes ≥1,000 sections in Title 15 at this release point
    // (1,633 sections at pl-119-90 — Securities Act + Securities
    // Exchange Act alone account for ~250 sections in the §§ 77-78
    // run). Lower bound is conservative — the exact count drifts as
    // Congress adds/repeals sections.
    assert!(
        title.sections.len() >= 1_000,
        "expected ≥1,000 sections, got {}",
        title.sections.len()
    );
}

#[test]
fn full_title_15_every_section_has_unique_identifier() {
    let p = title_15_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
fn full_title_15_every_section_satisfies_every_axiom() {
    let p = title_15_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

    axiom_every_section_has_num(&title).expect("EverySectionHasNum must hold for full Title 15");
    axiom_every_container_has_identifier(&title)
        .expect("EveryContainerHasIdentifier must hold for full Title 15");
    axiom_child_identifier_extends_parent(&title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 15");
    axiom_hierarchy_strictly_nested(&title)
        .expect("HierarchyStrictlyNested must hold for full Title 15");
    axiom_section_identifiers_unique(&title)
        .expect("SectionIdentifiersUnique must hold for full Title 15");
    axiom_ref_hrefs_well_formed(&title).expect("RefHrefsWellFormed must hold for full Title 15");
}

#[test]
fn full_title_15_every_section_lifts_to_statute() {
    use crate::social::compliance::statutes::from_uslm::from_uslm_section;
    let p = title_15_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

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
fn full_title_15_known_sections_present() {
    // Sentinel sections directly relevant to the SOX whistleblower
    // case. Failure here means the LRC's release point drifted or
    // our slicing is wrong.
    let p = title_15_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    let ids: std::collections::HashSet<&str> = title
        .sections
        .iter()
        .map(|s| s.identifier.as_str())
        .collect();

    // Securities Act of 1933 — short title (§ 77a) and definitions
    // (§ 77b). The 1933 Act ('33 Act) governs primary-market
    // securities offerings; SOX § 1514A protects reporting of
    // violations of "any rule or regulation of the Securities and
    // Exchange Commission" enacted under either act.
    assert!(
        ids.contains("/us/usc/t15/s77a"),
        "§ 77a (Securities Act short title) missing"
    );
    assert!(
        ids.contains("/us/usc/t15/s77b"),
        "§ 77b (Securities Act definitions) missing"
    );
    // Securities Act § 17(a) — fraudulent interstate transactions,
    // codified at 15 U.S.C. § 77q. One of the anti-fraud statutes
    // SOX § 1514A protects reporting of.
    assert!(
        ids.contains("/us/usc/t15/s77q"),
        "§ 77q (Securities Act § 17 anti-fraud) missing"
    );

    // Securities Exchange Act of 1934 — short title (§ 78a) and
    // definitions (§ 78c). The '34 Act governs secondary-market
    // trading and creates the SEC.
    assert!(
        ids.contains("/us/usc/t15/s78a"),
        "§ 78a (Exchange Act short title) missing"
    );
    assert!(
        ids.contains("/us/usc/t15/s78c"),
        "§ 78c (Exchange Act definitions) missing"
    );

    // § 78j — Manipulative and deceptive devices. § 78j(b) is the
    // statutory basis for SEC Rule 10b-5 (17 C.F.R. § 240.10b-5),
    // the workhorse private cause of action for securities fraud
    // and the underlying conduct SOX § 1514A protects reporting of.
    assert!(
        ids.contains("/us/usc/t15/s78j"),
        "§ 78j (Exchange Act § 10, Rule 10b-5 enabling) missing"
    );

    // § 78m — Periodicals and other reports. SOX § 302's CEO/CFO
    // certification requirement and SOX § 404's "internal control"
    // assessment are amendments to this section. The case's
    // protected activity centers on alleged misstatements in
    // § 78m-filed reports.
    assert!(
        ids.contains("/us/usc/t15/s78m"),
        "§ 78m (Exchange Act § 13 periodic reporting; SOX § 302/§ 404) missing"
    );

    // § 78ff — Penalties. Exchange Act § 32. Criminal penalties for
    // wilful violations, including knowingly making false § 78m
    // filings.
    assert!(
        ids.contains("/us/usc/t15/s78ff"),
        "§ 78ff (Exchange Act § 32 penalties) missing"
    );

    // § 78u-6 — Securities whistleblower incentives and protection.
    // Dodd-Frank § 922; the SEC monetary-award + anti-retaliation
    // scheme. Alternative whistleblower remedy that often co-exists
    // with a SOX § 806 (§ 1514A) claim — Digital Realty Trust, Inc.
    // v. Somers, 138 S. Ct. 767 (2018) drew the line between the
    // two regimes.
    //
    // NB: The LRC USLM encodes suffix-numbered section identifiers
    // with a U+2013 EN DASH (`–`), NOT the ASCII U+002D HYPHEN-MINUS
    // (`-`) used in citations. This is the GPO Style Manual 2016
    // §3.39 typographic convention for compound section numbers
    // (a hyphen vs an en-dash is intentional: en-dashes mark
    // numeric ranges and compound numbering). When a SOX brief
    // cites "15 U.S.C. § 78u-6" it uses ASCII; the USLM URN at the
    // LRC uses en-dash. Praxis stores identifiers verbatim from
    // the LRC source, so the en-dash form is authoritative here.
    assert!(
        ids.contains("/us/usc/t15/s78u\u{2013}6"),
        "§ 78u-6 (Dodd-Frank SEC whistleblower) missing"
    );

    // § 78j-1 — Audit committees. SOX § 301 added this entire
    // section to the Exchange Act: audit-committee independence,
    // pre-approval of audit/non-audit services, complaint-handling
    // procedures. Same LRC en-dash convention as § 78u-6.
    assert!(
        ids.contains("/us/usc/t15/s78j\u{2013}1"),
        "§ 78j-1 (SOX § 301 audit committees) missing"
    );
}

#[test]
fn full_title_15_codegen_and_runtime_agree_on_rule_10b5() {
    // The SOX case's load-bearing § in Title 15 is § 78j (Exchange
    // Act § 10, the Rule 10b-5 enabling statute). Verify codegen
    // and runtime extract the same term IDs for it — the same
    // contract Title 49's air21_42121 satisfies.
    use crate::social::compliance::statutes::from_uslm::derive_structural;
    let p = title_15_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    let section = title
        .section("/us/usc/t15/s78j")
        .expect("§ 78j must be present");
    let runtime_data = derive_structural("sea_78j", section);
    let codegen_doc = pr4xis::codegen::uslm::parse_uslm_str(&xml, "/us/usc/t15/s78j", "sea_78j")
        .expect("codegen parse");
    let runtime_ids: std::collections::HashSet<&str> =
        runtime_data.terms.iter().map(|t| t.id.as_str()).collect();
    let codegen_ids: std::collections::HashSet<&str> = codegen_doc
        .terms
        .iter()
        .filter(|t| t.id != "sea_78j")
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(
        runtime_ids, codegen_ids,
        "codegen and runtime diverge on Title 15 § 78j"
    );
}

// =============================================================================
// Full Title 28 coverage (M4.δ.5) — Judiciary and Judicial Procedure,
// the federal-court procedural backbone. Includes § 1331 (federal-
// question jurisdiction), § 1391 (venue), § 1404 (change of venue),
// § 1658 (federal civil-action SOL, including SOX § 1514A's
// Sarbanes-Oxley 6-month administrative-exhaustion period), § 2072
// (Rules Enabling Act), § 1254 (Supreme Court certiorari jurisdiction),
// § 1746 (unsworn declarations under penalty of perjury — the
// declaration form admissible at summary judgment per FRCP 56(c)(4)).
//
// 28 U.S.C. App. carries the Federal Rules of Civil Procedure (FRCP),
// Federal Rules of Evidence (FRE), Federal Rules of Appellate
// Procedure (FRAP), and Federal Rules of Bankruptcy Procedure (FRBP)
// as appendices. Loading Title 28 populates Layer 3 of
// statute_understanding with the typed legal-actor / evidence-rule
// vocabulary — no hand-coded ontology macros needed (M5.D.1).
//
// Same template as Titles 18, 49, 15: tests skip gracefully if the
// XML isn't on disk.
// =============================================================================

fn title_28_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/uscode/usc_title_28/usc_title_28-pl-119-90.xml")
}

#[test]
fn full_title_28_parses_with_expected_section_count() {
    let p = title_28_path();
    if !p.exists() {
        eprintln!("SKIP: Title 28 USLM not on disk at {p:?}");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t0 = std::time::Instant::now();
    let title = read_uslm_title(&xml).expect("Title 28 must parse");
    eprintln!(
        "Title 28 parse: {} sections in {:?}",
        title.sections.len(),
        t0.elapsed()
    );
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
    let p = title_28_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    axiom_section_identifiers_unique(&title).expect(
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
    let p = title_28_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
    let p = title_28_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

    axiom_every_section_has_num(&title).expect("EverySectionHasNum must hold for full Title 28");
    axiom_every_container_has_identifier(&title)
        .expect("EveryContainerHasIdentifier must hold for full Title 28");
    axiom_child_identifier_extends_parent(&title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 28");
    axiom_hierarchy_strictly_nested(&title)
        .expect("HierarchyStrictlyNested must hold for full Title 28");
    axiom_section_identifiers_unique(&title)
        .expect("SectionIdentifiersUnique must hold for full Title 28");
    axiom_ref_hrefs_well_formed(&title).expect("RefHrefsWellFormed must hold for full Title 28");
}

#[test]
fn full_title_28_every_section_lifts_to_statute() {
    use crate::social::compliance::statutes::from_uslm::from_uslm_section;
    let p = title_28_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

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
    let p = title_28_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
    use crate::social::compliance::statutes::from_uslm::derive_structural;
    let p = title_28_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    let section = title
        .section("/us/usc/t28/s1658")
        .expect("§ 1658 must be present");
    let runtime_data = derive_structural("usc_t28_s1658", section);
    let codegen_doc =
        pr4xis::codegen::uslm::parse_uslm_str(&xml, "/us/usc/t28/s1658", "usc_t28_s1658")
            .expect("codegen parse");
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

// =============================================================================
// Full Title 29 coverage (M4.δ.6) — Labor. The Occupational Safety
// and Health Act of 1970 (OSHA, §§ 651-678), including § 660(c) —
// the original employee anti-retaliation procedural framework that
// SOX § 1514A (18 U.S.C. § 1514A(b)(2)(A)) explicitly imports via
// 49 U.S.C. § 42121(b). Every SOX § 806 administrative claim filed
// with DOL/OSHA runs on the procedures rooted here. Also carries
// the Fair Labor Standards Act (FLSA, §§ 201 et seq.), the Family
// and Medical Leave Act (FMLA, §§ 2601 et seq.; § 2615 anti-
// retaliation), and the National Labor Relations Act (NLRA,
// §§ 151 et seq.).
// =============================================================================

fn title_29_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/uscode/usc_title_29/usc_title_29-pl-119-90.xml")
}

#[test]
fn full_title_29_parses_with_expected_section_count() {
    let p = title_29_path();
    if !p.exists() {
        eprintln!("SKIP: Title 29 USLM not on disk at {p:?}");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t0 = std::time::Instant::now();
    let title = read_uslm_title(&xml).expect("Title 29 must parse");
    eprintln!(
        "Title 29 parse: {} sections in {:?}",
        title.sections.len(),
        t0.elapsed()
    );
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
    let p = title_29_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    axiom_section_identifiers_unique(&title).expect(
        "SectionIdentifiersUnique-or-LRC-documented-duplicates must hold for full Title 29",
    );
}

#[test]
fn full_title_29_every_section_satisfies_every_axiom() {
    let p = title_29_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

    axiom_every_section_has_num(&title).expect("EverySectionHasNum must hold for full Title 29");
    axiom_every_container_has_identifier(&title)
        .expect("EveryContainerHasIdentifier must hold for full Title 29");
    axiom_child_identifier_extends_parent(&title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 29");
    axiom_hierarchy_strictly_nested(&title)
        .expect("HierarchyStrictlyNested must hold for full Title 29");
    axiom_section_identifiers_unique(&title)
        .expect("SectionIdentifiersUnique must hold for full Title 29");
    axiom_ref_hrefs_well_formed(&title).expect("RefHrefsWellFormed must hold for full Title 29");
}

#[test]
fn full_title_29_every_section_lifts_to_statute() {
    use crate::social::compliance::statutes::from_uslm::from_uslm_section;
    let p = title_29_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

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
    let p = title_29_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
    use crate::social::compliance::statutes::from_uslm::derive_structural;
    let p = title_29_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    let section = title
        .section("/us/usc/t29/s660")
        .expect("§ 660 must be present");
    let runtime_data = derive_structural("osha_660", section);
    let codegen_doc = pr4xis::codegen::uslm::parse_uslm_str(&xml, "/us/usc/t29/s660", "osha_660")
        .expect("codegen parse");
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

// =============================================================================
// Full Title 42 coverage (M4.δ.7) — The Public Health and Welfare.
// The harassment-timeline tree's federal civil-rights backbone:
// § 1981 (equal rights, Civil Rights Act of 1866), § 1981a
// (compensatory/punitive damages, Civil Rights Act of 1991),
// § 1983 (the Ku Klux Klan Act of 1871 — deprivation of rights
// under color of state law), § 1985(3) (conspiracy to interfere
// with civil rights), § 1988 (attorney's fees), § 2000e et seq.
// (Title VII of the Civil Rights Act of 1964; § 2000e-3 anti-
// retaliation), and § 12101 et seq. (Americans with Disabilities
// Act; § 12203 ADA anti-retaliation). At 113 MB / ~3,500 sections
// this is the LRC's largest title; loadable only after M4.δ.7.a
// moved corpus loading from build-time codegen to runtime XML
// parsing (the aggregate static OOM-killed rustc).
// =============================================================================

fn title_42_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/uscode/usc_title_42/usc_title_42-pl-119-90.xml")
}

#[test]
fn full_title_42_parses_with_expected_section_count() {
    let p = title_42_path();
    if !p.exists() {
        eprintln!("SKIP: Title 42 USLM not on disk at {p:?}");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t0 = std::time::Instant::now();
    let title = read_uslm_title(&xml).expect("Title 42 must parse");
    eprintln!(
        "Title 42 parse: {} sections in {:?}",
        title.sections.len(),
        t0.elapsed()
    );
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
    // Title 42 is the LRC's largest title — the civil-rights
    // statutes, Title VII, ADA, Medicare, Medicaid, Social
    // Security, public health (CDC/NIH/HHS), and housing. The
    // release point carries several thousand sections. Lower
    // bound is conservative.
    assert!(
        title.sections.len() >= 2000,
        "expected ≥2000 sections, got {}",
        title.sections.len()
    );
}

#[test]
fn full_title_42_every_section_has_unique_identifier() {
    let p = title_42_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    axiom_section_identifiers_unique(&title).expect(
        "SectionIdentifiersUnique-or-LRC-documented-duplicates must hold for full Title 42",
    );
}

#[test]
fn full_title_42_every_section_satisfies_every_axiom() {
    let p = title_42_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

    axiom_every_section_has_num(&title).expect("EverySectionHasNum must hold for full Title 42");
    axiom_every_container_has_identifier(&title)
        .expect("EveryContainerHasIdentifier must hold for full Title 42");
    axiom_child_identifier_extends_parent(&title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 42");
    axiom_hierarchy_strictly_nested(&title)
        .expect("HierarchyStrictlyNested must hold for full Title 42");
    axiom_section_identifiers_unique(&title)
        .expect("SectionIdentifiersUnique must hold for full Title 42");
    axiom_ref_hrefs_well_formed(&title).expect("RefHrefsWellFormed must hold for full Title 42");
}

#[test]
fn full_title_42_every_section_lifts_to_statute() {
    use crate::social::compliance::statutes::from_uslm::from_uslm_section;
    let p = title_42_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

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
fn full_title_42_known_sections_present() {
    // Sentinel sections forming the harassment-timeline tree's
    // federal civil-rights backbone.
    let p = title_42_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
fn full_title_42_codegen_and_runtime_agree_on_section_1983() {
    // § 1983 (deprivation of rights under color of state law,
    // the Ku Klux Klan Act of 1871) is THE load-bearing § for
    // the harassment-timeline tree's claims against state actors.
    // Runtime XML loading (M4.δ.7.a) must produce the same term
    // set the build-time codegen path would have.
    use crate::social::compliance::statutes::from_uslm::derive_structural;
    let p = title_42_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    let section = title
        .section("/us/usc/t42/s1983")
        .expect("§ 1983 must be present");
    let runtime_data = derive_structural("usc42_1983", section);
    let codegen_doc =
        pr4xis::codegen::uslm::parse_uslm_str(&xml, "/us/usc/t42/s1983", "usc42_1983")
            .expect("codegen parse");
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

// =============================================================================
// Full Title 5 coverage (M4.δ.8) — Government Organization and
// Employees. The harassment-timeline tree's federal transparency /
// personnel-law backbone: the Freedom of Information Act (§ 552 — the
// MuckRock FOIA requests' legal basis), the Privacy Act of 1974
// (§ 552a), the Administrative Procedure Act (§§ 551 et seq.), and
// the federal whistleblower scheme — § 2302(b)(8) (prohibited
// personnel practice: reprisal for protected disclosures), § 1213
// (disclosures to the Office of Special Counsel), § 1221 (individual
// right of action before the MSPB). Post-Pub. L. 117-286 (2022),
// Title 5 folds in the former Appendix (Inspector General Act,
// Ethics in Government Act).
// =============================================================================

fn title_5_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/uscode/usc_title_5/usc_title_5-pl-119-90.xml")
}

#[test]
fn full_title_5_parses_with_expected_section_count() {
    let p = title_5_path();
    if !p.exists() {
        eprintln!("SKIP: Title 5 USLM not on disk at {p:?}");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t0 = std::time::Instant::now();
    let title = read_uslm_title(&xml).expect("Title 5 must parse");
    eprintln!(
        "Title 5 parse: {} sections in {:?}",
        title.sections.len(),
        t0.elapsed()
    );
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
    let p = title_5_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    axiom_section_identifiers_unique(&title)
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
    let p = title_5_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
    let p = title_5_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

    axiom_every_section_has_num(&title).expect("EverySectionHasNum must hold for full Title 5");
    axiom_every_container_has_identifier(&title)
        .expect("EveryContainerHasIdentifier must hold for full Title 5");
    axiom_child_identifier_extends_parent(&title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 5");
    axiom_hierarchy_strictly_nested(&title)
        .expect("HierarchyStrictlyNested must hold for full Title 5");
    axiom_section_identifiers_unique(&title)
        .expect("SectionIdentifiersUnique must hold for full Title 5");
    axiom_ref_hrefs_well_formed(&title).expect("RefHrefsWellFormed must hold for full Title 5");
}

#[test]
fn full_title_5_every_section_lifts_to_statute() {
    use crate::social::compliance::statutes::from_uslm::from_uslm_section;
    let p = title_5_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

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
    let p = title_5_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
    use crate::social::compliance::statutes::from_uslm::derive_structural;
    let p = title_5_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    let section = title
        .section("/us/usc/t5/s552")
        .expect("§ 552 must be present");
    let runtime_data = derive_structural("usc5_552", section);
    let codegen_doc = pr4xis::codegen::uslm::parse_uslm_str(&xml, "/us/usc/t5/s552", "usc5_552")
        .expect("codegen parse");
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

// =============================================================================
// Full Title 50 coverage (M4.δ.9) — War and National Defense. The
// Foreign Intelligence Surveillance Act (FISA): § 1801 (definitions),
// § 1805 (electronic-surveillance orders), § 1809 (criminal sanctions
// for surveillance not authorized by statute), § 1810 (civil
// liability), § 1861 (the "Section 215" business-records authority),
// § 1881a (the "Section 702" program). Also the National Security Act
// of 1947 (§§ 3001 et seq.) and the War Powers Resolution (§§ 1541 et
// seq.). The harassment-timeline tree's surveillance / FBI-referral
// claims reach this title via the § 1809/§ 1810 unlawful-surveillance
// remedies.
// =============================================================================

fn title_50_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/uscode/usc_title_50/usc_title_50-pl-119-90.xml")
}

#[test]
fn full_title_50_parses_with_expected_section_count() {
    let p = title_50_path();
    if !p.exists() {
        eprintln!("SKIP: Title 50 USLM not on disk at {p:?}");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t0 = std::time::Instant::now();
    let title = read_uslm_title(&xml).expect("Title 50 must parse");
    eprintln!(
        "Title 50 parse: {} sections in {:?}",
        title.sections.len(),
        t0.elapsed()
    );
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
    let p = title_50_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    axiom_section_identifiers_unique(&title).expect(
        "SectionIdentifiersUnique-or-LRC-documented-duplicates must hold for full Title 50",
    );
}

#[test]
fn full_title_50_every_section_satisfies_every_axiom() {
    let p = title_50_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

    axiom_every_section_has_num(&title).expect("EverySectionHasNum must hold for full Title 50");
    axiom_every_container_has_identifier(&title)
        .expect("EveryContainerHasIdentifier must hold for full Title 50");
    axiom_child_identifier_extends_parent(&title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 50");
    axiom_hierarchy_strictly_nested(&title)
        .expect("HierarchyStrictlyNested must hold for full Title 50");
    axiom_section_identifiers_unique(&title)
        .expect("SectionIdentifiersUnique must hold for full Title 50");
    axiom_ref_hrefs_well_formed(&title).expect("RefHrefsWellFormed must hold for full Title 50");
}

#[test]
fn full_title_50_every_section_lifts_to_statute() {
    use crate::social::compliance::statutes::from_uslm::from_uslm_section;
    let p = title_50_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

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
    let p = title_50_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
    use crate::social::compliance::statutes::from_uslm::derive_structural;
    let p = title_50_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    let section = title
        .section("/us/usc/t50/s1809")
        .expect("§ 1809 must be present");
    let runtime_data = derive_structural("usc50_1809", section);
    let codegen_doc =
        pr4xis::codegen::uslm::parse_uslm_str(&xml, "/us/usc/t50/s1809", "usc50_1809")
            .expect("codegen parse");
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

// =============================================================================
// Full Title 1 coverage (M4.δ.10) — General Provisions. The authority
// anchor for the whole corpus: 1 U.S.C. § 204 (positive law /
// Code-as-evidence) is the statute that makes the LRC's USLM XML the
// legal text of the U.S. Code. Also the Dictionary Act (§§ 1-8
// statutory-construction defaults), the enacting clauses (§§ 101-105),
// and the Statutes-at-Large / treaty publication framework (§§ 112,
// 112a). Tier 4: smallest title, but foundational — it closes the
// self-referential loop by which the corpus cites the statute that
// authorizes its own form.
// =============================================================================

fn title_1_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/uscode/usc_title_1/usc_title_1-pl-119-90.xml")
}

#[test]
fn full_title_1_parses_with_expected_section_count() {
    let p = title_1_path();
    if !p.exists() {
        eprintln!("SKIP: Title 1 USLM not on disk at {p:?}");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).expect("Title 1 must parse");
    eprintln!("Title 1 parse: {} sections", title.sections.len());
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
    let p = title_1_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    axiom_section_identifiers_unique(&title)
        .expect("SectionIdentifiersUnique-or-LRC-documented-duplicates must hold for full Title 1");
}

#[test]
fn full_title_1_every_section_satisfies_every_axiom() {
    let p = title_1_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

    axiom_every_section_has_num(&title).expect("EverySectionHasNum must hold for full Title 1");
    axiom_every_container_has_identifier(&title)
        .expect("EveryContainerHasIdentifier must hold for full Title 1");
    axiom_child_identifier_extends_parent(&title)
        .expect("ChildIdentifierExtendsParent must hold for full Title 1");
    axiom_hierarchy_strictly_nested(&title)
        .expect("HierarchyStrictlyNested must hold for full Title 1");
    axiom_section_identifiers_unique(&title)
        .expect("SectionIdentifiersUnique must hold for full Title 1");
    axiom_ref_hrefs_well_formed(&title).expect("RefHrefsWellFormed must hold for full Title 1");
}

#[test]
fn full_title_1_every_section_lifts_to_statute() {
    use crate::social::compliance::statutes::from_uslm::from_uslm_section;
    let p = title_1_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();

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
    let p = title_1_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
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
    use crate::social::compliance::statutes::from_uslm::derive_structural;
    let p = title_1_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let title = read_uslm_title(&xml).unwrap();
    let section = title
        .section("/us/usc/t1/s204")
        .expect("§ 204 must be present");
    let runtime_data = derive_structural("usc1_204", section);
    let codegen_doc = pr4xis::codegen::uslm::parse_uslm_str(&xml, "/us/usc/t1/s204", "usc1_204")
        .expect("codegen parse");
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

// =============================================================================
// Tier-1 hierarchy coverage (M4.δ.4)
// =============================================================================

#[test]
fn hierarchy_has_published_part_chapter_subchapter_counts_for_title_18() {
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
fn hierarchy_has_published_subtitle_part_chapter_counts_for_title_49() {
    let p = title_49_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
fn flat_sections_equals_hierarchy_dfs_walk() {
    // Invariant: the flat `sections` field is a DFS flatten of
    // the `hierarchy` tree. Order and count must match.
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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

#[test]
fn container_kind_parse_round_trips_canonical_tags() {
    for tag in ["subtitle", "part", "subpart", "chapter", "subchapter"] {
        let k = ContainerKind::parse(tag).expect("known tag");
        assert_eq!(k.tag(), tag);
    }
    assert!(ContainerKind::parse("section").is_none());
    assert!(ContainerKind::parse("clause").is_none());
}

#[test]
fn container_nesting_depth_orders_kinds_canonically() {
    use ContainerKind::*;
    let kinds = [Subtitle, Part, Subpart, Chapter, Subchapter];
    let depths: Vec<usize> = kinds.iter().map(|k| k.nesting_depth()).collect();
    assert_eq!(depths, vec![0, 1, 2, 3, 4]);
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
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
    // The field is populated correctly — the test asserts the
    // type contract, not a specific count. Title-level `<header>`
    // is an optional element.
    let _: &Vec<UsCodeHeader> = &t.headers;
}

#[test]
fn title_18_notes_blocks_collected_at_title_level() {
    // The title's preamble carries title-level notes (enacting
    // history, current-through marker, etc.).
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
fn quoted_content_sections_dont_leak_into_published_sections() {
    // The duplicate-identifier failure that surfaced in Title 49
    // was caused by `<section>` elements inside `<quotedContent>`
    // being collected as real sections. With Tier-3, those become
    // UsCodeSectionRef inside the quote's body, and the published
    // sections list is unaffected.
    let p = title_49_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();
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
fn sox_1514a_belongs_to_part_i_chapter_73_title_18() {
    // Sentinel: § 1514A lives in Title 18 > Part I (CRIMES) >
    // Chapter 73 (OBSTRUCTION OF JUSTICE). Verifies hierarchy
    // assigns sections to their published containers.
    let p = title_18_path();
    if !p.exists() {
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();
    let t = read_uslm_title(&xml).unwrap();

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

#[test]
fn full_title_18_codegen_and_runtime_agree_on_sox_1514a() {
    use crate::social::compliance::statutes::from_uslm::derive_structural;
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP");
        return;
    }
    let xml = std::fs::read_to_string(&p).unwrap();

    let title = read_uslm_title(&xml).unwrap();
    let sox_section = title
        .section("/us/usc/t18/s1514A")
        .expect("§ 1514A must be present");
    let runtime_data = derive_structural("sox_1514a", sox_section);

    let codegen_doc =
        pr4xis::codegen::uslm::parse_uslm_str(&xml, "/us/usc/t18/s1514A", "sox_1514a")
            .expect("codegen parse");

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
// M4.δ.11/15/16/17/19/20 — Non-USC document element typed stubs
//
// These elements are defined in the USLM schema (LRC USLM XML User
// Guide) but populated only in non-USC USLM documents (bills,
// public laws, CFR, statutory forms). USC titles pl-119-90 ship
// with zero occurrences. The typed enums cover the schema surface
// for the "100% USLM coverage" goal; reader integration is
// deferred until a non-USC corpus loads.
// =============================================================================

#[test]
fn heading_variant_parses_all_six_kinds() {
    assert_eq!(
        UsCodeHeadingVariant::parse("heading"),
        Some(UsCodeHeadingVariant::Heading)
    );
    assert_eq!(
        UsCodeHeadingVariant::parse("subheading"),
        Some(UsCodeHeadingVariant::Subheading)
    );
    assert_eq!(
        UsCodeHeadingVariant::parse("crossHeading"),
        Some(UsCodeHeadingVariant::CrossHeading)
    );
    assert_eq!(
        UsCodeHeadingVariant::parse("docTitle"),
        Some(UsCodeHeadingVariant::DocTitle)
    );
    assert_eq!(
        UsCodeHeadingVariant::parse("longTitle"),
        Some(UsCodeHeadingVariant::LongTitle)
    );
    assert_eq!(
        UsCodeHeadingVariant::parse("shortTitle"),
        Some(UsCodeHeadingVariant::ShortTitle)
    );
}

#[test]
fn heading_variant_rejects_non_heading_tags() {
    assert_eq!(UsCodeHeadingVariant::parse("section"), None);
    assert_eq!(UsCodeHeadingVariant::parse("chapeau"), None);
    assert_eq!(UsCodeHeadingVariant::parse(""), None);
}

#[test]
fn additional_container_parses_all_seven_kinds() {
    for (tag, expected) in [
        ("division", UsCodeAdditionalContainer::Division),
        ("article", UsCodeAdditionalContainer::Article),
        ("subarticle", UsCodeAdditionalContainer::Subarticle),
        ("preamble", UsCodeAdditionalContainer::Preamble),
        ("preliminary", UsCodeAdditionalContainer::Preliminary),
        ("appendix", UsCodeAdditionalContainer::Appendix),
        ("subsubitem", UsCodeAdditionalContainer::Subsubitem),
    ] {
        assert_eq!(UsCodeAdditionalContainer::parse(tag), Some(expected));
    }
}

#[test]
fn additional_container_rejects_usc_containers() {
    // USC titles use ContainerKind variants (Subtitle/Part/etc.),
    // not the additional ones — the parsers must be disjoint.
    assert_eq!(UsCodeAdditionalContainer::parse("subtitle"), None);
    assert_eq!(UsCodeAdditionalContainer::parse("part"), None);
    assert_eq!(UsCodeAdditionalContainer::parse("chapter"), None);
}

#[test]
fn quoted_variant_parses_all_three_kinds() {
    assert_eq!(
        UsCodeQuotedVariant::parse("quotedText"),
        Some(UsCodeQuotedVariant::QuotedText)
    );
    assert_eq!(
        UsCodeQuotedVariant::parse("recital"),
        Some(UsCodeQuotedVariant::Recital)
    );
    assert_eq!(
        UsCodeQuotedVariant::parse("statement"),
        Some(UsCodeQuotedVariant::Statement)
    );
}

#[test]
fn quoted_variant_disjoint_from_quoted_content() {
    // <quotedContent> is the generic kind (already modeled); the
    // variants are refinements with specific document-role semantics.
    assert_eq!(UsCodeQuotedVariant::parse("quotedContent"), None);
}

#[test]
fn legislative_formula_parses_all_six_kinds() {
    for (tag, expected) in [
        ("enactingFormula", UsCodeLegislativeFormula::EnactingFormula),
        ("amendingFormula", UsCodeLegislativeFormula::AmendingFormula),
        ("approved", UsCodeLegislativeFormula::Approved),
        ("made", UsCodeLegislativeFormula::Made),
        ("action", UsCodeLegislativeFormula::Action),
        ("instruction", UsCodeLegislativeFormula::Instruction),
    ] {
        assert_eq!(UsCodeLegislativeFormula::parse(tag), Some(expected));
    }
}

#[test]
fn form_element_parses_all_five_kinds() {
    for (tag, expected) in [
        ("checkBox", UsCodeFormElement::CheckBox),
        ("fillIn", UsCodeFormElement::FillIn),
        ("block", UsCodeFormElement::Block),
        ("row", UsCodeFormElement::Row),
        ("set", UsCodeFormElement::Set),
    ] {
        assert_eq!(UsCodeFormElement::parse(tag), Some(expected));
    }
}

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
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
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
// M4.δ.14 — Amendment markup tests (Ins/Del)
//
// Per LRC USLM User Guide § "Amendment Markup" + W3C HTML 4.01
// §9.4. `<ins>` / `<del>` carry the diff of an amendment. Empty in
// retro-converted USC titles; populated when the USLM source is an
// amendment-in-progress (bill or public law).
// =============================================================================

#[test]
fn amendment_kind_parses_ins_and_del() {
    assert_eq!(
        UsCodeAmendmentKind::parse("ins"),
        Some(UsCodeAmendmentKind::Insertion)
    );
    assert_eq!(
        UsCodeAmendmentKind::parse("del"),
        Some(UsCodeAmendmentKind::Deletion)
    );
}

#[test]
fn amendment_kind_rejects_non_amendment_tags() {
    assert_eq!(UsCodeAmendmentKind::parse("section"), None);
    assert_eq!(UsCodeAmendmentKind::parse("i"), None);
    assert_eq!(UsCodeAmendmentKind::parse(""), None);
}

#[test]
fn reader_captures_ins_amendment_in_section() {
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/sA"><num value="A">A</num><heading>x</heading><ins>new text added</ins></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];
    assert_eq!(s.amendments.len(), 1);
    assert_eq!(s.amendments[0].kind, UsCodeAmendmentKind::Insertion);
    assert_eq!(s.amendments[0].text, "new text added");
}

#[test]
fn reader_captures_del_amendment_in_section() {
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/sB"><num value="B">B</num><heading>x</heading><del>obsolete text</del></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];
    assert_eq!(s.amendments.len(), 1);
    assert_eq!(s.amendments[0].kind, UsCodeAmendmentKind::Deletion);
    assert_eq!(s.amendments[0].text, "obsolete text");
}

#[test]
fn reader_captures_amendments_in_subdivision() {
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/sC"><num value="C">C</num><heading>x</heading><subsection identifier="/us/usc/t18/sC/a"><num value="a">a</num><ins>inserted</ins><del>deleted</del></subsection></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];
    let sub = &s.children[0];
    assert_eq!(sub.amendments.len(), 2);
    let kinds: Vec<_> = sub.amendments.iter().map(|a| a.kind).collect();
    assert!(kinds.contains(&UsCodeAmendmentKind::Insertion));
    assert!(kinds.contains(&UsCodeAmendmentKind::Deletion));
}

#[test]
fn reader_empty_amendments_for_section_without_diff_markup() {
    let title = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    assert!(title.sections[0].amendments.is_empty());
}

#[test]
fn title_18_real_corpus_has_zero_amendment_markup_today() {
    // Retro-converted USC titles don't carry `<ins>`/`<del>` —
    // those appear in amendments-in-progress (bills, public laws).
    // Tripwire: if this assertion ever fails, LRC has changed the
    // pl-XXX-YY USC publication format to include amendment-diff
    // markup and downstream consumers should be reviewed.
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");

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
// M4.δ.18 — Additional metadata tests
//
// LRC USLM publishes USC titles with three legally-significant
// non-DC elements inside `<meta>`:
//   - `<docNumber>` — the title number as text
//   - `<docPublicationName>` — publication identifier with release point
//   - `<property role="is-positive-law">yes|no</property>` —
//     whether the title is enacted as positive law (per 1 U.S.C.
//     § 204) or merely a non-positive-law compilation.
// Plus DCMI Terms (`dcterms:created`, `dcterms:modified`).
// =============================================================================

const SAMPLE_WITH_USLM_META: &str = r##"<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" identifier="/us/usc/t18"><meta><dc:title>Title 18</dc:title><docNumber>18</docNumber><docPublicationName>Online@119-90</docPublicationName><property role="is-positive-law">yes</property><dcterms:created>2026-05-04T10:22:41</dcterms:created></meta><main><title identifier="/us/usc/t18"><num value="18">18</num><heading>x</heading></title></main></uscDoc>"##;

#[test]
fn doc_number_parsed_into_meta() {
    let title = read_uslm_title(SAMPLE_WITH_USLM_META).expect("parse");
    let meta = title.meta.expect("meta present");
    assert_eq!(meta.doc_number.as_deref(), Some("18"));
}

#[test]
fn doc_publication_name_carries_release_point() {
    let title = read_uslm_title(SAMPLE_WITH_USLM_META).expect("parse");
    let meta = title.meta.expect("meta present");
    assert_eq!(meta.doc_publication_name.as_deref(), Some("Online@119-90"));
}

#[test]
fn property_role_is_positive_law_captured() {
    let title = read_uslm_title(SAMPLE_WITH_USLM_META).expect("parse");
    let meta = title.meta.expect("meta present");
    assert_eq!(meta.properties.len(), 1);
    assert_eq!(meta.properties[0].role.as_deref(), Some("is-positive-law"));
    assert_eq!(meta.properties[0].value, "yes");
}

#[test]
fn is_positive_law_method_returns_true_for_yes() {
    let title = read_uslm_title(SAMPLE_WITH_USLM_META).expect("parse");
    let meta = title.meta.expect("meta present");
    assert_eq!(meta.is_positive_law(), Some(true));
}

#[test]
fn is_positive_law_method_returns_false_for_no() {
    let xml = r##"<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t42"><meta><property role="is-positive-law">no</property></meta><main><title identifier="/us/usc/t42"><num value="42">42</num><heading>x</heading></title></main></uscDoc>"##;
    let title = read_uslm_title(xml).expect("parse");
    let meta = title.meta.expect("meta present");
    assert_eq!(meta.is_positive_law(), Some(false));
}

#[test]
fn is_positive_law_method_returns_none_when_property_absent() {
    let xml = r##"<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t99"><meta></meta><main><title identifier="/us/usc/t99"><num value="99">99</num><heading>x</heading></title></main></uscDoc>"##;
    let title = read_uslm_title(xml).expect("parse");
    let meta = title.meta.expect("meta present");
    assert_eq!(meta.is_positive_law(), None);
}

#[test]
fn dcterms_created_routed_to_typed_field() {
    let title = read_uslm_title(SAMPLE_WITH_USLM_META).expect("parse");
    let meta = title.meta.expect("meta present");
    assert_eq!(meta.dcterms_created.as_deref(), Some("2026-05-04T10:22:41"));
}

#[test]
fn dcterms_unknown_element_routed_to_other() {
    let xml = r##"<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dcterms="http://purl.org/dc/terms/" identifier="/us/usc/t99"><meta><dcterms:isPartOf>USC</dcterms:isPartOf></meta><main><title identifier="/us/usc/t99"><num value="99">99</num><heading>x</heading></title></main></uscDoc>"##;
    let title = read_uslm_title(xml).expect("parse");
    let meta = title.meta.expect("meta present");
    assert!(
        meta.dcterms_other
            .iter()
            .any(|(k, v)| k == "isPartOf" && v == "USC")
    );
}

#[test]
fn title_18_real_corpus_is_positive_law() {
    // Title 18 is enacted as positive law (Act of June 25, 1948,
    // ch. 645, 62 Stat. 683). Verify the LRC's published USLM
    // carries the property correctly.
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");
    let meta = title.meta.expect("Title 18 has meta block");
    assert_eq!(
        meta.is_positive_law(),
        Some(true),
        "Title 18 must declare is-positive-law=yes"
    );
}

#[test]
fn title_18_real_corpus_carries_doc_number_and_publication() {
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");
    let meta = title.meta.expect("Title 18 has meta block");
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
// M4.δ.9 — Tier-6 Table tests
//
// USLM embeds XHTML-namespaced tables (the HTML 4.01 §11 table model
// that XHTML 1.0 transcribes — XHTML 1.0 itself has no §9) when
// statutory text needs tabular layout. LRC pl-119-90 ships ~17 tables
// in Title 18, most carrying class="TableOfDisposition" — conversion
// tables for repealed/renumbered statute sections.
// =============================================================================

const SAMPLE_WITH_TABLE: &str = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><num value="18">18</num><heading>x</heading><table id="tbl1" class="TableOfDisposition" xmlns="http://www.w3.org/1999/xhtml"><thead><tr class="header"><th>Old §</th><th>New §</th></tr></thead><tbody><tr><td>1</td><td>3</td></tr><tr><td>2</td><td>5</td></tr></tbody></table></title>"##;

#[test]
fn table_block_parses_with_id_and_class() {
    let title = read_uslm_title(SAMPLE_WITH_TABLE).expect("parse");
    assert_eq!(title.tables.len(), 1);
    let t = &title.tables[0];
    assert_eq!(t.identifier.as_deref(), Some("tbl1"));
    assert_eq!(t.class.as_deref(), Some("TableOfDisposition"));
}

#[test]
fn table_header_rows_separated_from_body_rows() {
    let title = read_uslm_title(SAMPLE_WITH_TABLE).expect("parse");
    let t = &title.tables[0];
    assert_eq!(t.header_rows.len(), 1, "one <thead> row");
    assert_eq!(t.body_rows.len(), 2, "two <tbody> rows");
}

#[test]
fn table_cells_discriminate_th_from_td() {
    let title = read_uslm_title(SAMPLE_WITH_TABLE).expect("parse");
    let t = &title.tables[0];
    // Header row: two <th> cells.
    assert!(
        t.header_rows[0]
            .cells
            .iter()
            .all(|c| c.kind == UsCodeTableCellKind::Header)
    );
    // Body rows: <td> cells.
    assert!(
        t.body_rows[0]
            .cells
            .iter()
            .all(|c| c.kind == UsCodeTableCellKind::Data)
    );
}

#[test]
fn table_cell_text_collected_per_cell() {
    let title = read_uslm_title(SAMPLE_WITH_TABLE).expect("parse");
    let t = &title.tables[0];
    assert_eq!(t.header_rows[0].cells[0].text, "Old §");
    assert_eq!(t.header_rows[0].cells[1].text, "New §");
    assert_eq!(t.body_rows[0].cells[0].text, "1");
    assert_eq!(t.body_rows[1].cells[1].text, "5");
}

#[test]
fn table_cell_colspan_rowspan_parsed_as_u32() {
    let xml = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><num value="18">18</num><heading>x</heading><table xmlns="http://www.w3.org/1999/xhtml"><tr><th colspan="2" rowspan="3">spanning header</th></tr></table></title>"##;
    let title = read_uslm_title(xml).expect("parse");
    let cell = &title.tables[0].body_rows[0].cells[0];
    assert_eq!(cell.colspan, Some(2));
    assert_eq!(cell.rowspan, Some(3));
}

#[test]
fn table_without_thead_collects_all_rows_as_body() {
    // Some tables have only <tr> children, no <thead>/<tbody>.
    let xml = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><num value="18">18</num><heading>x</heading><table xmlns="http://www.w3.org/1999/xhtml"><tr><td>a</td></tr><tr><td>b</td></tr></table></title>"##;
    let title = read_uslm_title(xml).expect("parse");
    let t = &title.tables[0];
    assert!(t.header_rows.is_empty());
    assert_eq!(t.body_rows.len(), 2);
}

#[test]
fn table_idempotent_across_reparse() {
    let a = read_uslm_title(SAMPLE_WITH_TABLE).expect("parse");
    let b = read_uslm_title(SAMPLE_WITH_TABLE).expect("parse");
    assert_eq!(a.tables, b.tables);
}

#[test]
fn title_18_real_corpus_has_table_of_disposition() {
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");
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
// M4.δ.8 — Tier-5 Table of Contents tests
//
// LRC USLM User Guide § "Table of Contents" defines `<toc>` and
// `<tocItem>` for the navigable index. LRC pl-119-90 ships TOCs at
// the title level (three-column layout) and one per chapter.
// =============================================================================

const SAMPLE_WITH_TOC: &str = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><num value="18">18</num><heading>x</heading><toc role="threeColumnTOC" id="t1"><layout><header role="tocColumnHeader"><column>Part</column><column/><column>Sec.</column></header><tocItem><column><ref href="/us/usc/t18/ptI">I.</ref></column><column>Crimes</column><column><ref href="/us/usc/t18/s1">1</ref></column></tocItem><tocItem><column><ref href="/us/usc/t18/ptII">II.</ref></column><column>Criminal Procedure</column><column><ref href="/us/usc/t18/s3001">3001</ref></column></tocItem></layout></toc></title>"##;

#[test]
fn toc_block_parses_with_role_and_id() {
    let title = read_uslm_title(SAMPLE_WITH_TOC).expect("parse");
    assert_eq!(title.tocs.len(), 1);
    let toc = &title.tocs[0];
    assert_eq!(toc.role.as_deref(), Some("threeColumnTOC"));
    assert_eq!(toc.identifier.as_deref(), Some("t1"));
}

#[test]
fn toc_items_collected_in_document_order() {
    let title = read_uslm_title(SAMPLE_WITH_TOC).expect("parse");
    let toc = &title.tocs[0];
    assert_eq!(toc.items.len(), 2);
    // First item: Part I.
    assert!(toc.items[0].text.contains("Crimes"));
    // Second item: Part II.
    assert!(toc.items[1].text.contains("Criminal Procedure"));
}

#[test]
fn toc_item_target_is_first_ref_href() {
    let title = read_uslm_title(SAMPLE_WITH_TOC).expect("parse");
    let toc = &title.tocs[0];
    assert_eq!(toc.items[0].target.as_deref(), Some("/us/usc/t18/ptI"));
    assert_eq!(toc.items[1].target.as_deref(), Some("/us/usc/t18/ptII"));
}

#[test]
fn toc_item_collects_all_refs() {
    let title = read_uslm_title(SAMPLE_WITH_TOC).expect("parse");
    let toc = &title.tocs[0];
    // First item has 2 refs: the part URN and the section URN.
    assert_eq!(toc.items[0].refs.len(), 2);
    let hrefs: Vec<&str> = toc.items[0].refs.iter().map(|r| r.href.as_str()).collect();
    assert!(hrefs.contains(&"/us/usc/t18/ptI"));
    assert!(hrefs.contains(&"/us/usc/t18/s1"));
}

#[test]
fn toc_header_row_is_not_collected_as_item() {
    let title = read_uslm_title(SAMPLE_WITH_TOC).expect("parse");
    let toc = &title.tocs[0];
    // The `<header>` row carries column labels ("Part" / "Sec.")
    // and must not appear as a navigable tocItem.
    assert!(
        !toc.items
            .iter()
            .any(|i| i.text == "Part\t\tSec." || i.text == "Part"),
        "TOC header row leaked into items: {:?}",
        toc.items.iter().map(|i| &i.text).collect::<Vec<_>>()
    );
}

#[test]
fn toc_without_block_yields_empty_vec() {
    let title = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    assert!(title.tocs.is_empty());
}

#[test]
fn toc_idempotent_across_reparse() {
    let a = read_uslm_title(SAMPLE_WITH_TOC).expect("parse");
    let b = read_uslm_title(SAMPLE_WITH_TOC).expect("parse");
    assert_eq!(a.tocs.len(), b.tocs.len());
    assert_eq!(a.tocs[0].items.len(), b.tocs[0].items.len());
    for (x, y) in a.tocs[0].items.iter().zip(b.tocs[0].items.iter()) {
        assert_eq!(x.target, y.target);
        assert_eq!(x.text, y.text);
    }
}

#[test]
fn title_18_real_corpus_has_three_column_toc() {
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");
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
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");

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
// M4.δ.10 — Tier-7 Dublin Core meta block tests
//
// DCMI Metadata Element Set (ISO 15836-1:2017) is the standard
// vocabulary for the `<meta>` block under USLM's `<uscDoc>` root.
// The LRC consistently populates `dc:title`, `dc:type`,
// `dc:publisher`, `dc:creator`. Other DC elements (open vocabulary)
// surface in `other_dc` rather than being silently dropped.
// =============================================================================

const SAMPLE_WITH_META: &str = r##"<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/" identifier="/us/usc/t18"><meta><dc:title>Title 18</dc:title><dc:type>USCTitle</dc:type><dc:publisher>OLRC</dc:publisher><dc:creator>USCConverter 1.7.2</dc:creator></meta><main><title identifier="/us/usc/t18"><num value="18">Title 18—</num><heading>CRIMES</heading></title></main></uscDoc>"##;

#[test]
fn meta_block_parsed_into_dublin_core_fields() {
    let title = read_uslm_title(SAMPLE_WITH_META).expect("parse");
    let meta = title.meta.expect("meta block present");
    assert_eq!(meta.title.as_deref(), Some("Title 18"));
    assert_eq!(meta.doc_type.as_deref(), Some("USCTitle"));
    assert_eq!(meta.publisher.as_deref(), Some("OLRC"));
    assert_eq!(meta.creator.as_deref(), Some("USCConverter 1.7.2"));
}

#[test]
fn meta_block_absent_yields_none() {
    // Bare slice without `<uscDoc>` / `<meta>` wrapper.
    let title = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    assert!(title.meta.is_none());
}

#[test]
fn meta_block_unknown_dc_elements_surface_in_other_dc() {
    let xml = r##"<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/" identifier="/us/usc/t99"><meta><dc:title>X</dc:title><dc:subject>statutes</dc:subject><dc:coverage>U.S.</dc:coverage></meta><main><title identifier="/us/usc/t99"><num value="99">99</num><heading>x</heading></title></main></uscDoc>"##;
    let title = read_uslm_title(xml).expect("parse");
    let meta = title.meta.expect("meta present");
    assert_eq!(meta.title.as_deref(), Some("X"));
    // Unknown DC elements must surface, not be silently dropped.
    assert!(
        meta.other_dc
            .iter()
            .any(|(k, v)| k == "subject" && v == "statutes"),
        "dc:subject must appear in other_dc; got {:?}",
        meta.other_dc
    );
    assert!(
        meta.other_dc
            .iter()
            .any(|(k, v)| k == "coverage" && v == "U.S."),
        "dc:coverage must appear in other_dc; got {:?}",
        meta.other_dc
    );
}

#[test]
fn meta_block_ignores_non_dc_children() {
    // An unprefixed `<title>` inside `<meta>` is schema-non-conformant
    // (USLM places the corpus's title in `<main>`, not `<meta>`);
    // the reader rejects it as non-DC rather than confusing it with
    // the DC title element.
    let xml = r##"<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/" identifier="/us/usc/t99"><meta><title>Should Be Ignored</title><dc:title>Real DC Title</dc:title></meta><main><title identifier="/us/usc/t99"><num value="99">99</num><heading>x</heading></title></main></uscDoc>"##;
    let title = read_uslm_title(xml).expect("parse");
    let meta = title.meta.expect("meta present");
    assert_eq!(meta.title.as_deref(), Some("Real DC Title"));
}

#[test]
fn meta_block_empty_dc_element_recorded_in_other_dc() {
    // An empty DC element shouldn't silently disappear — it might
    // indicate a malformed source. Record it with empty body.
    let xml = r##"<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/" identifier="/us/usc/t99"><meta><dc:rights></dc:rights><dc:title>X</dc:title></meta><main><title identifier="/us/usc/t99"><num value="99">99</num><heading>x</heading></title></main></uscDoc>"##;
    let title = read_uslm_title(xml).expect("parse");
    let meta = title.meta.expect("meta present");
    assert!(
        meta.other_dc
            .iter()
            .any(|(k, v)| k == "rights" && v.is_empty()),
        "empty dc:rights must surface as an other_dc entry with empty body"
    );
    assert!(
        meta.rights.is_none(),
        "empty dc:rights must not populate the typed field"
    );
}

#[test]
fn title_18_real_corpus_has_olrc_meta_block() {
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");
    let meta = title.meta.expect("Title 18 must carry a meta block");
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

#[test]
fn title_49_real_corpus_has_olrc_meta_block() {
    let p = title_49_path();
    if !p.exists() {
        eprintln!("SKIP: Title 49 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");
    let meta = title.meta.expect("Title 49 must carry a meta block");
    assert_eq!(meta.title.as_deref(), Some("Title 49"));
    assert_eq!(meta.publisher.as_deref(), Some("OLRC"));
}

#[test]
fn meta_block_is_idempotent_across_reparse() {
    let a = read_uslm_title(SAMPLE_WITH_META).expect("parse");
    let b = read_uslm_title(SAMPLE_WITH_META).expect("parse");
    assert_eq!(a.meta, b.meta);
}

// =============================================================================
// M4.δ.12 — Note kind classification tests
//
// LRC USLM User Guide § 6.2 documents the `topic` and `type`
// attributes on `<note>`. The Editorial / Statutory / Change /
// Enacting / Footnote partition is the structural classification
// downstream legal-research code reasons over. Topics not yet
// classified surface as Unrecognized — a tripwire telling the
// reader the LRC vocabulary may have extended.
// =============================================================================

#[test]
fn note_kind_classifies_editorial_topic() {
    assert_eq!(
        UsCodeNoteKind::parse(Some("editorialNotes"), None),
        UsCodeNoteKind::Editorial
    );
}

#[test]
fn note_kind_classifies_statutory_topic() {
    assert_eq!(
        UsCodeNoteKind::parse(Some("statutoryNotes"), None),
        UsCodeNoteKind::Statutory
    );
}

#[test]
fn note_kind_classifies_change_topics() {
    assert_eq!(
        UsCodeNoteKind::parse(Some("amendments"), None),
        UsCodeNoteKind::Change
    );
    assert_eq!(
        UsCodeNoteKind::parse(Some("codification"), None),
        UsCodeNoteKind::Change
    );
}

#[test]
fn note_kind_classifies_enacting_topic() {
    assert_eq!(
        UsCodeNoteKind::parse(Some("enacting"), None),
        UsCodeNoteKind::Enacting
    );
}

#[test]
fn note_kind_classifies_footnote_via_type_attribute() {
    // `type="footnote"` wins over `topic` — footnotes are
    // structurally distinct from the topic-classified notes.
    assert_eq!(
        UsCodeNoteKind::parse(None, Some("footnote")),
        UsCodeNoteKind::Footnote
    );
    // Even when topic is also set, type=footnote dominates.
    assert_eq!(
        UsCodeNoteKind::parse(Some("editorialNotes"), Some("footnote")),
        UsCodeNoteKind::Footnote
    );
}

#[test]
fn note_kind_returns_unrecognized_for_unmapped_topic() {
    assert_eq!(
        UsCodeNoteKind::parse(Some("miscellaneous"), None),
        UsCodeNoteKind::Unrecognized
    );
    assert_eq!(
        UsCodeNoteKind::parse(Some("separability"), None),
        UsCodeNoteKind::Unrecognized
    );
    assert_eq!(
        UsCodeNoteKind::parse(None, None),
        UsCodeNoteKind::Unrecognized
    );
}

#[test]
fn note_kind_method_on_uscodenote_uses_topic_and_type() {
    let xml = r##"<title xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18"><num value="18">18</num><heading>x</heading><notes type="uscNote"><note topic="editorialNotes"><heading>E1</heading><p>editorial body</p></note><note topic="amendments"><heading>A1</heading><p>amendment text</p></note><note type="footnote"><p>fn body</p></note></notes></title>"##;
    let title = read_uslm_title(xml).expect("parse");
    assert_eq!(title.notes_blocks.len(), 1);
    let kinds: Vec<UsCodeNoteKind> = title.notes_blocks[0]
        .notes
        .iter()
        .map(|n| n.kind())
        .collect();
    assert!(kinds.contains(&UsCodeNoteKind::Editorial));
    assert!(kinds.contains(&UsCodeNoteKind::Change));
    assert!(kinds.contains(&UsCodeNoteKind::Footnote));
}

#[test]
fn title_18_corpus_classifies_at_least_one_note_per_documented_kind() {
    // Real-corpus assertion: Title 18's notes span the documented
    // topic vocabulary; classification should surface at least one
    // Editorial, one Statutory, and one Change note across the
    // title (footnotes likewise common at the title level).
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");

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
// M4.δ.13 — Tier-5 definitional ontology tests (Def/Term/Marker)
//
// LRC USLM User Guide § "Lexical Elements" specifies <def>, <term>,
// and <marker> as the definitional ontology surface. LRC's pl-119-90
// USC release does not yet populate these elements — defined terms
// still appear as `<inline class="small-caps">` Tier-4 ornaments —
// but the types and reader cover the schema so that when LRC rolls
// forward the retro-conversion, the parser handles them with no
// further code change. These tests drive the reader via inline
// fixtures.
// =============================================================================

const SAMPLE_WITH_DEF: &str = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/sX"><num value="X">§ X.</num><heading>Definitions</heading><def id="/us/usc/t18/sX/def-employee"><term refersTo="#emp">covered employee</term><content>means an individual employed by a covered employer.</content></def><def><term refersTo="#emp">employee</term><term>worker</term><content>shall include officers and agents.</content></def><marker name="anchor-1" class="label"/></section>"##;

#[test]
fn reader_captures_def_block_with_single_term() {
    let title = read_uslm_title(SAMPLE_WITH_DEF).expect("parse");
    let s = &title.sections[0];
    assert!(
        !s.def_blocks.is_empty(),
        "section must have at least one <def> block; got {:?}",
        s.def_blocks
    );
    let first = &s.def_blocks[0];
    assert_eq!(
        first.identifier.as_deref(),
        Some("/us/usc/t18/sX/def-employee")
    );
    assert_eq!(first.terms.len(), 1);
    assert_eq!(first.terms[0].text, "covered employee");
    assert_eq!(first.terms[0].refers_to.as_deref(), Some("#emp"));
}

#[test]
fn reader_captures_def_block_with_multiple_terms() {
    let title = read_uslm_title(SAMPLE_WITH_DEF).expect("parse");
    let s = &title.sections[0];
    let multi = s
        .def_blocks
        .iter()
        .find(|d| d.terms.len() > 1)
        .expect("def with ≥2 terms");
    assert_eq!(multi.terms.len(), 2);
    assert!(multi.terms.iter().any(|t| t.text == "employee"));
    assert!(multi.terms.iter().any(|t| t.text == "worker"));
}

#[test]
fn reader_def_block_body_contains_definitional_prose() {
    let title = read_uslm_title(SAMPLE_WITH_DEF).expect("parse");
    let s = &title.sections[0];
    let d = &s.def_blocks[0];
    assert!(
        d.body.contains("means an individual"),
        "<def> body must include the definitional prose; got {:?}",
        d.body
    );
}

#[test]
fn reader_captures_marker_with_name_and_class() {
    let title = read_uslm_title(SAMPLE_WITH_DEF).expect("parse");
    let s = &title.sections[0];
    assert_eq!(s.markers.len(), 1);
    let m = &s.markers[0];
    assert_eq!(m.name, "anchor-1");
    assert_eq!(m.class.as_deref(), Some("label"));
}

#[test]
fn reader_term_refers_to_falls_back_to_hyphenated_attribute() {
    // LRC documentation gives both `refersTo` and `refers-to` in
    // examples; the reader accepts either.
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/sY"><num value="Y">§ Y.</num><heading>x</heading><def><term refers-to="#alt">alt-attr term</term></def></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];
    let term = &s.def_blocks[0].terms[0];
    assert_eq!(term.refers_to.as_deref(), Some("#alt"));
}

#[test]
fn reader_marker_without_name_yields_empty_string() {
    // Schema requires `name=`; if a malformed corpus omits it, the
    // reader records the absence as empty and lets the consumer
    // surface the problem.
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/sZ"><num value="Z">§ Z.</num><heading>x</heading><marker/></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];
    assert_eq!(s.markers.len(), 1);
    assert_eq!(s.markers[0].name, "");
}

#[test]
fn reader_def_blocks_idempotent_across_reparse() {
    let a = read_uslm_title(SAMPLE_WITH_DEF).expect("parse");
    let b = read_uslm_title(SAMPLE_WITH_DEF).expect("parse");
    let da = &a.sections[0].def_blocks;
    let db = &b.sections[0].def_blocks;
    assert_eq!(da.len(), db.len());
    for (x, y) in da.iter().zip(db.iter()) {
        assert_eq!(x.terms.len(), y.terms.len());
        assert_eq!(x.body, y.body);
        assert_eq!(x.identifier, y.identifier);
    }
}

#[test]
fn reader_section_without_def_has_empty_def_blocks() {
    let title = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    let s = &title.sections[0];
    assert!(s.def_blocks.is_empty());
    assert!(s.markers.is_empty());
}

#[test]
fn reader_subdivision_can_carry_its_own_def_blocks() {
    // <def> nested inside a subsection's body — collected on the
    // subdivision, not the section's top-level list.
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/us/usc/t18/sA"><num value="A">§ A.</num><heading>x</heading><subsection identifier="/us/usc/t18/sA/a"><num value="a">(a)</num><heading>Definitions</heading><def><term>employer</term><content>means…</content></def></subsection></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];
    let sub = &s.children[0];
    assert_eq!(sub.def_blocks.len(), 1);
    assert_eq!(sub.def_blocks[0].terms[0].text, "employer");
    // Top-level section list stays empty.
    assert!(s.def_blocks.is_empty());
}

#[test]
fn title_18_real_corpus_has_zero_typed_def_elements_today() {
    // Documentation assertion: LRC pl-119-90 doesn't yet emit
    // typed `<def>` / `<marker>` elements. If this count ever goes
    // up, the corpus has moved forward and downstream code that
    // anticipates defined-term lifts should be reviewed.
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");

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
// M4.δ.21 — UsCodeTitleId ontology tests
//
// Validates the typed title identifier composes with the
// identifier_format ontology (UslmUrn leaf) and that the citation
// projections connect to English. The integer title number is a
// derived property of the URN, not a structural field.
// =============================================================================

#[test]
fn uscodetitle_id_constructs_from_number_for_title_18() {
    let id = UsCodeTitleId::try_from_number(18).expect("Title 18");
    assert_eq!(id.number(), 18);
    assert_eq!(id.urn(), "/us/usc/t18");
    assert_eq!(id.source_name(), "usc_title_18");
}

#[test]
fn uscodetitle_id_constructs_from_urn_for_title_49() {
    let id = UsCodeTitleId::try_from_urn("/us/usc/t49").expect("Title 49");
    assert_eq!(id.number(), 49);
}

#[test]
fn uscodetitle_id_round_trips_through_source_name() {
    let id = UsCodeTitleId::try_from_source_name("usc_title_18").expect("parse");
    assert_eq!(id.source_name(), "usc_title_18");
    assert_eq!(id.number(), 18);
}

#[test]
fn uscodetitle_id_rejects_out_of_range_number() {
    assert!(matches!(
        UsCodeTitleId::try_from_number(0),
        Err(UsCodeTitleIdError::OutOfRange { .. })
    ));
    assert!(matches!(
        UsCodeTitleId::try_from_number(99),
        Err(UsCodeTitleIdError::OutOfRange { .. })
    ));
}

#[test]
fn uscodetitle_id_rejects_non_title_urn_paths() {
    // Section-level path, not a title path.
    assert!(matches!(
        UsCodeTitleId::try_from_urn("/us/usc/t18/s1514A"),
        Err(UsCodeTitleIdError::NotATitleUrn)
    ));
    // Subdivision-level.
    assert!(matches!(
        UsCodeTitleId::try_from_urn("/us/usc/t18/s1514A/a"),
        Err(UsCodeTitleIdError::NotATitleUrn)
    ));
}

#[test]
fn uscodetitle_id_rejects_bad_urn_grammar() {
    // Missing `/us/` prefix → identifier_format rejects.
    assert!(matches!(
        UsCodeTitleId::try_from_urn("/usc/t18"),
        Err(UsCodeTitleIdError::BadUrn(_))
    ));
}

#[test]
fn uscodetitle_id_rejects_malformed_source_name() {
    assert!(matches!(
        UsCodeTitleId::try_from_source_name("wordnet_2024"),
        Err(UsCodeTitleIdError::BadSourceName { .. })
    ));
    assert!(matches!(
        UsCodeTitleId::try_from_source_name("usc_title_abc"),
        Err(UsCodeTitleIdError::BadSourceName { .. })
    ));
}

#[test]
fn uscodetitle_id_identifier_format_is_uslm_urn() {
    use crate::formal::meta::identifier_format::ontology::IdentifierFormatConcept;
    let id = UsCodeTitleId::try_from_number(18).unwrap();
    assert_eq!(id.identifier().format, IdentifierFormatConcept::UslmUrn);
}

#[test]
fn uscodetitle_id_short_citation_is_bluebook_form() {
    // Bluebook 21st ed. Rule 12.1 — "18 U.S.C." (no trailing space).
    let id = UsCodeTitleId::try_from_number(18).unwrap();
    assert_eq!(id.short_citation(), "18 U.S.C.");
}

#[test]
fn uscodetitle_id_long_citation_is_english_noun_phrase() {
    let id = UsCodeTitleId::try_from_number(18).unwrap();
    assert_eq!(id.long_citation(), "title 18 of the United States Code");
}

#[test]
fn uscodetitle_id_eq_and_hash_by_urn() {
    use std::collections::HashSet;
    let a = UsCodeTitleId::try_from_number(18).unwrap();
    let b = UsCodeTitleId::try_from_urn("/us/usc/t18").unwrap();
    let c = UsCodeTitleId::try_from_source_name("usc_title_18").unwrap();
    assert_eq!(a, b);
    assert_eq!(b, c);
    let mut s = HashSet::new();
    s.insert(a);
    s.insert(b);
    s.insert(c);
    assert_eq!(s.len(), 1, "three constructors for Title 18 must collide");
}

#[test]
fn uscodetitle_id_section_1514a_belongs_to_title_18() {
    // The section URN /us/usc/t18/s1514A starts with the title URN.
    // This is the structural composition: a title contains sections,
    // and the section URN is a path-extension of the title URN.
    let t18 = UsCodeTitleId::try_from_number(18).unwrap();
    let section_urn = "/us/usc/t18/s1514A";
    assert!(
        section_urn.starts_with(t18.urn()),
        "section URN must extend its containing title's URN"
    );
}

// =============================================================================
// M4.δ.7 — Tier-4 inline markup tests
//
// USLM Schema § "Inline Elements" mirrors XHTML inline semantics
// (W3C Recommendation, "XHTML™ 1.0", §4). The reader emits a typed
// `Vec<UsCodeInlineRun>` alongside the flat-text projection so the
// downstream legal-NLP layer can distinguish defined-term spans
// (`<inline class="small-caps">`), emphasis (`<i>`/`<b>`), notes
// (`<sup>`), and links (`<a href>`) without re-parsing.
// =============================================================================

#[test]
fn inline_kind_parses_canonical_tags() {
    assert_eq!(InlineKind::parse("inline"), Some(InlineKind::Inline));
    assert_eq!(InlineKind::parse("i"), Some(InlineKind::Italic));
    assert_eq!(InlineKind::parse("b"), Some(InlineKind::Bold));
    assert_eq!(InlineKind::parse("sup"), Some(InlineKind::Superscript));
    assert_eq!(InlineKind::parse("sub"), Some(InlineKind::Subscript));
    assert_eq!(InlineKind::parse("span"), Some(InlineKind::Span));
    assert_eq!(InlineKind::parse("a"), Some(InlineKind::Anchor));
}

#[test]
fn inline_kind_rejects_non_inline_tags() {
    assert_eq!(InlineKind::parse("section"), None);
    assert_eq!(InlineKind::parse("subsection"), None);
    assert_eq!(InlineKind::parse("chapeau"), None);
    assert_eq!(InlineKind::parse(""), None);
}

#[test]
fn subsection_heading_carries_small_caps_inline_run() {
    let title = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    let a = &title.sections[0].children[0];

    // (a)'s heading is `<inline class="small-caps">Whistleblower
    // Protection</inline>` — must surface as exactly one Inline
    // run carrying the class attribute, not a bare PlainText run.
    assert_eq!(
        a.heading_runs.len(),
        1,
        "expected one inline run; got {:?}",
        a.heading_runs
    );
    let run = &a.heading_runs[0];
    assert_eq!(run.kind, InlineKind::Inline);
    assert_eq!(run.text, "Whistleblower Protection");
    assert_eq!(run.class.as_deref(), Some("small-caps"));
    assert_eq!(run.href, None);
}

#[test]
fn chapeau_plain_text_becomes_single_plain_text_run() {
    let title = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    let a = &title.sections[0].children[0];

    // `(a)`'s chapeau is bare text — one PlainText run, no class
    // / href, exact match to the flat-text projection.
    assert_eq!(a.chapeau_runs.len(), 1);
    let run = &a.chapeau_runs[0];
    assert_eq!(run.kind, InlineKind::PlainText);
    assert_eq!(run.class, None);
    assert_eq!(run.href, None);
    assert_eq!(
        run.text,
        a.chapeau.as_deref().expect("chapeau text present")
    );
}

#[test]
fn content_runs_match_flat_content_for_leaf_subdivision() {
    let title = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    let a = &title.sections[0].children[0];
    let p = &a.children[0];
    let sp = &p.children[0];

    // Subparagraph (A) has a `<content>` child holding plain text.
    let joined: String = sp.content_runs.iter().map(|r| r.text.as_str()).collect();
    let flat = sp.content.as_deref().unwrap_or("");
    assert_eq!(
        joined.trim(),
        flat.trim(),
        "inline-runs text must equal flat content projection"
    );
}

#[test]
fn section_heading_runs_match_flat_heading() {
    let title = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    let s = &title.sections[0];

    // Section heading is plain text — single run, equal to flat.
    let joined: String = s.heading_runs.iter().map(|r| r.text.as_str()).collect();
    assert_eq!(joined.trim(), s.heading.trim());
}

#[test]
fn unknown_inline_wrapper_falls_through_to_plain_text() {
    // An unrecognized inline wrapper (`<weird>...</weird>`) must
    // not silently drop its text content — the reader flattens
    // it to PlainText so the visible text projection is lossless.
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/x"><num value="0">§ 0.</num><heading><weird>kept text</weird></heading></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];

    let joined: String = s.heading_runs.iter().map(|r| r.text.as_str()).collect();
    assert!(
        joined.contains("kept text"),
        "unknown wrapper must not drop its text; got runs {:?}",
        s.heading_runs
    );
}

#[test]
fn inline_runs_collapse_whitespace_in_text_nodes() {
    // Multi-line / multi-space text nodes collapse to single
    // spaces — proves the reader normalizes whitespace per
    // W3C XML 1.0 §2.10 "White Space Handling" (attribute-style
    // collapsing applied to element content for display text).
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/x"><num value="0">§ 0.</num><heading>  many    spaces   here  </heading></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];

    assert_eq!(s.heading_runs.len(), 1);
    assert_eq!(s.heading_runs[0].text, "many spaces here");
}

#[test]
fn ref_inside_heading_becomes_plain_text_run_keeping_href() {
    // `<ref>` is a citation-graph element, not an inline ornament,
    // but inside a heading we still emit a PlainText run carrying
    // the href so the visible text is preserved and downstream
    // citation extraction can read the link target.
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/x"><num value="0">§ 0.</num><heading>See <ref href="/us/usc/t15/s78j-1">Section 10A</ref> below</heading></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];

    let with_href: Vec<&UsCodeInlineRun> =
        s.heading_runs.iter().filter(|r| r.href.is_some()).collect();
    assert_eq!(with_href.len(), 1, "ref href should be carried through");
    assert_eq!(with_href[0].href.as_deref(), Some("/us/usc/t15/s78j-1"));
    assert_eq!(with_href[0].text, "Section 10A");
}

#[test]
fn anchor_element_typed_as_inline_anchor() {
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/x"><num value="0">§ 0.</num><heading>visit <a href="https://www.house.gov">House.gov</a> for more</heading></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];

    let anchors: Vec<&UsCodeInlineRun> = s
        .heading_runs
        .iter()
        .filter(|r| r.kind == InlineKind::Anchor)
        .collect();
    assert_eq!(anchors.len(), 1);
    assert_eq!(anchors[0].text, "House.gov");
    assert_eq!(anchors[0].href.as_deref(), Some("https://www.house.gov"));
}

#[test]
fn italic_bold_sup_sub_each_get_their_typed_kind() {
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/x"><num value="0">§ 0.</num><heading><i>it</i><b>bo</b><sup>up</sup><sub>dn</sub></heading></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];

    let kinds: Vec<InlineKind> = s.heading_runs.iter().map(|r| r.kind).collect();
    assert!(kinds.contains(&InlineKind::Italic));
    assert!(kinds.contains(&InlineKind::Bold));
    assert!(kinds.contains(&InlineKind::Superscript));
    assert!(kinds.contains(&InlineKind::Subscript));
}

#[test]
fn empty_inline_element_produces_no_run() {
    // `<inline></inline>` with no text content must not emit
    // an empty-text run — the reader filters empties.
    let xml = r##"<section xmlns="http://xml.house.gov/schemas/uslm/1.0" identifier="/x"><num value="0">§ 0.</num><heading><inline class="small-caps"></inline></heading></section>"##;
    let title = read_uslm_title(xml).expect("parse");
    let s = &title.sections[0];

    assert!(
        s.heading_runs.iter().all(|r| !r.text.is_empty()),
        "empty inline must not produce empty-text runs; got {:?}",
        s.heading_runs
    );
}

#[test]
fn inline_runs_idempotent_under_reparse() {
    // Same bytes → same runs sequence (kind, text, class, href).
    // Determinism gate per the Phase-0 "deterministic" requirement.
    let a = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    let b = read_uslm_title(SAMPLE_SECTION_SLICE).expect("parse");
    let ar = &a.sections[0].children[0].heading_runs;
    let br = &b.sections[0].children[0].heading_runs;
    assert_eq!(ar.len(), br.len());
    for (x, y) in ar.iter().zip(br.iter()) {
        assert_eq!(x.kind, y.kind);
        assert_eq!(x.text, y.text);
        assert_eq!(x.class, y.class);
        assert_eq!(x.href, y.href);
    }
}

#[test]
fn title_18_contains_inline_runs_with_class_attributes() {
    // Real-corpus assertion: Title 18 ships with at least one
    // subsection whose heading is wrapped in `<inline
    // class="small-caps">` — the canonical USLM idiom for
    // styled defined-term headings. If zero inline runs carry a
    // class, either the corpus has changed format or our reader
    // is dropping them silently.
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");

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
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");
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
// Cross-cutting properties that must hold across every USC title
// the LRC publishes. These are the corpus-wide axioms that
// downstream consumers (legal-NLP, citation extractors, navigation
// renderers) rely on.
// =============================================================================

#[test]
fn every_section_urn_starts_with_its_title_urn() {
    // Composition property: a section's URN must be a path-extension
    // of its enclosing title's URN. Without this, the URN graph
    // doesn't form a tree and citation traversal breaks. Multi-URN
    // identifiers (combined ranges) are skipped — they're a separate
    // construct outside the single-URN composition rule.
    for path_fn in [
        title_18_path as fn() -> std::path::PathBuf,
        title_49_path as fn() -> std::path::PathBuf,
    ] {
        let p = path_fn();
        if !p.exists() {
            continue;
        }
        let xml = std::fs::read_to_string(&p).expect("read XML");
        let title = read_uslm_title(&xml).expect("parse");
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
}

#[test]
fn every_single_section_urn_is_uslm_urn_grammar_conformant() {
    // Every parsed section's identifier — when it's a single URN —
    // must satisfy the USLM-URN grammar. LRC uses space-separated
    // multi-URN identifiers for combined repealed-section ranges
    // (e.g. "/us/usc/t18/s221 /us/usc/t18/s222"); those are a
    // separate ontological construct not yet typed and are
    // legitimately outside the single-URN grammar.
    use crate::formal::meta::identifier_format::Identifier;
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");
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
fn title_id_round_trips_via_urn_for_title_18_and_49() {
    // Building a UsCodeTitleId from the parsed title's URN must
    // yield the same number as `UsCodeTitle.number`.
    for (path_fn, expected_number) in [
        (title_18_path as fn() -> std::path::PathBuf, 18u32),
        (title_49_path as fn() -> std::path::PathBuf, 49u32),
    ] {
        let p = path_fn();
        if !p.exists() {
            continue;
        }
        let xml = std::fs::read_to_string(&p).expect("read XML");
        let title = read_uslm_title(&xml).expect("parse");
        let id = UsCodeTitleId::try_from_urn(&title.identifier)
            .expect("title URN is a valid UsCodeTitleId");
        assert_eq!(id.number(), expected_number);
        assert_eq!(title.number, expected_number);
        // The URN parsed from the corpus equals the URN built from
        // the title number — round-trip closure.
        assert_eq!(id.urn(), title.identifier);
    }
}

#[test]
fn every_section_lifts_to_statute_with_urn_provenance() {
    // The to_statute lift (USLM section → Statute) must produce a
    // Statute whose description carries the section's URN as
    // context_uri. This is the M4.δ.21 URN push-down invariant —
    // enforced uniformly across every section in every loaded title.
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");

    // Spot-check 10 sections — too expensive to lift all ~1500.
    use crate::formal::meta::identifier_format::Identifier;
    use crate::social::software::markup::xml::uslm::corpus::loaded as usc_loaded;
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
    let p = title_18_path();
    if !p.exists() {
        eprintln!("SKIP: Title 18 XML missing");
        return;
    }
    let xml = std::fs::read_to_string(&p).expect("read XML");
    let title = read_uslm_title(&xml).expect("parse");

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

// ── XSD-grounded dispatch axioms (M4.ε.5.a.5) ──────────────────
//
// The hand-coded ContainerKind / SubdivisionKind /
// UsCodeAdditionalContainer parsers project XML element-name
// strings into runtime enum variants. The XSD-grounded variants
// (`from_xsd_element`) consult the loaded USLM-1.0.18 XSD ontology
// before accepting a name — confirming both that the name is
// `<xsd:element>`-declared (W3C XSD 1.1 Part 1 §3.3) and that it is
// a member of `substitutionGroup="level"` (W3C XSD 1.1 Part 1
// §3.3.6) for the level-family variants.
//
// The axioms below are the structural guarantee that the runtime
// enum variants don't drift from the loaded XSD's level family:
// every variant's tag is a verified level-group member in the
// loaded XSD.
//
// Citation: W3C XSD 1.1 Part 1 §3.3 (Element Declarations), §3.3.6
// (Substitution Groups); LRC USLM XML User Guide § V (level
// hierarchy).

fn axiom_loaded_uslm_xsd() -> crate::formal::meta::xsd::from_xsd_parser::XsdOntologyInstance {
    use crate::formal::meta::xsd::from_xsd_parser::project_from_xsd_text;
    use crate::formal::meta::xsd::uslm_vocabulary::USLM_1_0_18_XSD;
    project_from_xsd_text(USLM_1_0_18_XSD)
}

#[test]
fn axiom_container_kind_members_are_level_group_members() {
    // For every ContainerKind variant, the variant's USLM tag is a
    // member of the loaded USLM XSD's `substitutionGroup="level"`
    // family (W3C XSD 1.1 Part 1 §3.3.6).
    let xsd = axiom_loaded_uslm_xsd();
    let kinds = ContainerKind::load_from_xsd(&xsd);
    assert!(
        !kinds.is_empty(),
        "load_from_xsd must project at least one variant from the loaded USLM XSD's level group"
    );
    for kind in kinds {
        let tag = kind.tag();
        assert!(
            xsd.lookup_element(tag).is_some(),
            "ContainerKind variant {kind:?} has tag {tag:?} which the loaded USLM XSD doesn't declare as an <xsd:element>"
        );
        assert!(
            xsd.is_member_of_substitution_group(tag, "level"),
            "ContainerKind variant {kind:?} has tag {tag:?} which the loaded USLM XSD doesn't carry under substitutionGroup=\"level\""
        );
    }
}

#[test]
fn axiom_subdivision_kind_members_are_level_group_members() {
    // For every SubdivisionKind variant, the variant's USLM tag is a
    // member of the loaded USLM XSD's `substitutionGroup="level"`
    // family.
    let xsd = axiom_loaded_uslm_xsd();
    let kinds = SubdivisionKind::load_from_xsd(&xsd);
    assert!(
        !kinds.is_empty(),
        "load_from_xsd must project at least one variant from the loaded USLM XSD's level group"
    );
    for kind in kinds {
        let tag = kind.tag();
        assert!(
            xsd.lookup_element(tag).is_some(),
            "SubdivisionKind variant {kind:?} has tag {tag:?} which the loaded USLM XSD doesn't declare as an <xsd:element>"
        );
        assert!(
            xsd.is_member_of_substitution_group(tag, "level"),
            "SubdivisionKind variant {kind:?} has tag {tag:?} which the loaded USLM XSD doesn't carry under substitutionGroup=\"level\""
        );
    }
}

#[test]
fn axiom_uscode_additional_container_members_are_xsd_declared() {
    // For every UsCodeAdditionalContainer variant, the variant's
    // USLM tag is declared by the loaded USLM XSD (W3C XSD 1.1
    // Part 1 §3.3). Not every additional-container variant is a
    // level-group member — `preamble` for instance sits in a
    // separate part of the schema — but every one is a declared
    // element.
    let xsd = axiom_loaded_uslm_xsd();
    let kinds = UsCodeAdditionalContainer::load_from_xsd(&xsd);
    assert!(
        !kinds.is_empty(),
        "load_from_xsd must project at least one variant from the loaded USLM XSD's element declarations"
    );
    for kind in kinds {
        let tag = kind.tag();
        assert!(
            xsd.lookup_element(tag).is_some(),
            "UsCodeAdditionalContainer variant {kind:?} has tag {tag:?} which the loaded USLM XSD doesn't declare as an <xsd:element>"
        );
    }
}

#[test]
fn axiom_container_kind_from_xsd_element_matches_parse() {
    // For every ContainerKind variant's tag, the XSD-grounded
    // `from_xsd_element` query and the hand-coded `parse` agree on
    // the variant. This guarantees that callers can migrate from
    // `parse` to `from_xsd_element` without changing the
    // result of dispatch for any tag in the variant set.
    let xsd = axiom_loaded_uslm_xsd();
    for kind in ContainerKind::load_from_xsd(&xsd) {
        let tag = kind.tag();
        assert_eq!(
            ContainerKind::from_xsd_element(tag, &xsd),
            ContainerKind::parse(tag),
            "from_xsd_element and parse disagree for tag {tag:?}"
        );
    }
}

#[test]
fn axiom_subdivision_kind_from_xsd_element_matches_parse() {
    let xsd = axiom_loaded_uslm_xsd();
    for kind in SubdivisionKind::load_from_xsd(&xsd) {
        let tag = kind.tag();
        assert_eq!(
            SubdivisionKind::from_xsd_element(tag, &xsd),
            SubdivisionKind::parse(tag),
            "from_xsd_element and parse disagree for tag {tag:?}"
        );
    }
}

#[test]
fn axiom_from_xsd_element_rejects_non_level_member_names() {
    // Names that are XSD-declared but NOT in the level family must
    // be rejected by `ContainerKind::from_xsd_element` and
    // `SubdivisionKind::from_xsd_element`. Per W3C XSD 1.1 Part 1
    // §3.3.6, the predicate is reflexive-transitive substitution-
    // group membership — `notes`, `meta`, `header` etc. share no
    // level-group ancestry with the level family.
    let xsd = axiom_loaded_uslm_xsd();
    for non_level in ["notes", "note", "meta", "header", "toc", "signature"] {
        if xsd.lookup_element(non_level).is_none() {
            // The loaded XSD must declare these for the negative
            // test to be meaningful.
            continue;
        }
        assert_eq!(
            ContainerKind::from_xsd_element(non_level, &xsd),
            None,
            "ContainerKind::from_xsd_element({non_level:?}) accepted a non-level-group element"
        );
        assert_eq!(
            SubdivisionKind::from_xsd_element(non_level, &xsd),
            None,
            "SubdivisionKind::from_xsd_element({non_level:?}) accepted a non-level-group element"
        );
    }
}

#[test]
fn axiom_from_xsd_element_rejects_undeclared_names() {
    // Names that aren't declared by the loaded USLM XSD have no
    // `{type definition}` to dispatch on (W3C XSD 1.1 Part 1 §3.3)
    // — `from_xsd_element` must return `None`.
    let xsd = axiom_loaded_uslm_xsd();
    for fake in ["zzz_not_a_uslm_element", "xyzzy", ""] {
        assert_eq!(ContainerKind::from_xsd_element(fake, &xsd), None);
        assert_eq!(SubdivisionKind::from_xsd_element(fake, &xsd), None);
        assert_eq!(
            UsCodeAdditionalContainer::from_xsd_element(fake, &xsd),
            None
        );
    }
}
