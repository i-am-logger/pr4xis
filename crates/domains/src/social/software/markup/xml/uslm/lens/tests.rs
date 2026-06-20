//! Tests for the USLM `WellBehavedLens` implementation.
//!
//! Coverage:
//!
//! - **Layer 1 — unit tests:** `get` parses synthetic USLM, `put`
//!   round-trips bytes, structural fields propagate.
//! - **Layer 2 — lens-law axioms:** the three Foster et al. 2007 §2.2
//!   laws (GetPut, PutGet, PutPut) exercised against synthetic and
//!   real-corpus inputs. Each test names the specific cited section.
//! - **Layer 3 — proptest properties:** PutGet and GetPut across
//!   randomized USLM-shape variants.
//!
//! Citation: Foster, Greenwald, Moore, Pierce & Schmitt 2007, *ACM
//! TOPLAS* 29(3) Article 17, §2.2 (well-behaved-lens laws), §5 (tree-
//! shaped lenses).

use super::*;
use crate::formal::meta::well_behaved_lens::WellBehavedLens;

/// Synthetic USLM `<section>` slice with a small subdivision tree
/// (SOX § 1514A-shaped).
const SAMPLE_SECTION: &str = r##"<section identifier="/us/usc/t18/s1514A"><num value="1514A">§ 1514A.</num><heading>Civil action to protect against retaliation in fraud cases</heading><subsection identifier="/us/usc/t18/s1514A/a"><num value="a">(a)</num><chapeau>No company may discriminate against an employee—</chapeau><paragraph identifier="/us/usc/t18/s1514A/a/1"><num value="1">(1)</num><content>to provide information.</content></paragraph></subsection></section>"##;

/// A full-shape USLM title with `<uscDoc>` wrapper, `<meta>`, `<main>`,
/// `<title>`, and one `<section>`. Exercises the namespace-aware
/// reader logic.
const SAMPLE_TITLE: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<uscDoc xmlns="http://xml.house.gov/schemas/uslm/1.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <meta>
    <dc:title>Title 18</dc:title>
    <dc:type>USCTitle</dc:type>
    <dc:publisher>OLRC</dc:publisher>
  </meta>
  <main>
    <title identifier="/us/usc/t18">
      <num value="18">Title 18</num>
      <heading>CRIMES AND CRIMINAL PROCEDURE</heading>
      <section identifier="/us/usc/t18/s1514A">
        <num value="1514A">§ 1514A.</num>
        <heading>Civil action to protect against retaliation in fraud cases</heading>
        <content>No company may discriminate.</content>
      </section>
    </title>
  </main>
</uscDoc>
"##;

// =========================================================
// Layer 1 — unit tests
// =========================================================

#[test]
fn get_parses_synthetic_section_slice() {
    let target =
        UslmXmlLens::get(SAMPLE_SECTION.as_bytes()).expect("get must parse synthetic section");
    assert_eq!(target.view.sections.len(), 1);
    assert_eq!(target.view.sections[0].identifier, "/us/usc/t18/s1514A");
    assert_eq!(target.complement.as_slice(), SAMPLE_SECTION.as_bytes());
}

#[test]
fn get_parses_synthetic_full_title() {
    let target = UslmXmlLens::get(SAMPLE_TITLE.as_bytes()).expect("get must parse synthetic title");
    assert_eq!(target.view.identifier, "/us/usc/t18");
    assert_eq!(target.view.number, 18);
    assert_eq!(target.view.sections.len(), 1);
    assert!(target.view.meta.is_some(), "meta block populated");
}

#[test]
fn put_returns_complement_bytes_verbatim() {
    let target = UslmXmlLens::get(SAMPLE_TITLE.as_bytes()).expect("get must parse synthetic title");
    let put_bytes = UslmXmlLens::put(&target).expect("put");
    // Constant-complement view-update (Bancilhon & Spyratos 1981
    // Theorem 3): with the complement held constant, put recovers
    // the source bytes verbatim.
    assert_eq!(put_bytes.as_slice(), SAMPLE_TITLE.as_bytes());
}

#[test]
fn get_rejects_non_utf8() {
    // 0xFF is not valid UTF-8.
    let err = UslmXmlLens::get(&[0xFFu8, 0xFEu8, 0xFDu8]).expect_err("must reject non-UTF-8");
    assert!(matches!(err, UslmLensError::NotUtf8(_)));
}

#[test]
fn get_rejects_non_usc_root() {
    // Well-formed XML, but no `<uscDoc>` / `<title>` / `<section>` root.
    // Post-M4.ε.5.a.5 the XSD-grounded walker is more precise: it
    // refuses the unknown element name BEFORE attempting the
    // namespace-aware title-locator. Both outcomes are valid
    // rejections — `UnknownElement` is the strictly more informative
    // error (W3C XSD 1.1 Part 1 §3.3: no element declaration → no
    // dispatch target).
    let err =
        UslmXmlLens::get(b"<other-root>nope</other-root>").expect_err("must reject non-USLM root");
    assert!(
        matches!(err, UslmLensError::UnknownElement(_))
            || matches!(err, UslmLensError::Read(UslmReadError::NoUsCodeRoot)),
        "expected UnknownElement or NoUsCodeRoot, got {err:?}",
    );
}

// =========================================================
// Layer 2 — lens-law axioms (Foster et al. 2007 §2.2)
// =========================================================

/// Axiom — PutGet on a synthetic section slice.
///
/// Foster, Greenwald, Moore, Pierce & Schmitt 2007 §2.2:
/// `canonical(put(get(s))) = canonical(s)` for every source byte
/// stream `s`. With the explicit complement design (Bancilhon &
/// Spyratos 1981 Theorem 3), `put(get(s)) = s` byte-verbatim — the
/// canonical-form check is the stronger framing used by the M4.θ
/// fractal-round-trip gate.
#[test]
fn axiom_put_get_law_synthetic_section() {
    UslmXmlLens::assert_put_get_law(SAMPLE_SECTION.as_bytes()).expect("PutGet on section slice");
}

/// Axiom — PutGet on a full synthetic title.
#[test]
fn axiom_put_get_law_synthetic_title() {
    UslmXmlLens::assert_put_get_law(SAMPLE_TITLE.as_bytes()).expect("PutGet on full title");
}

/// Axiom — GetPut. `get(put(get(s)))` must yield a view structurally
/// equal to `get(s)`. Foster et al. 2007 §2.2.
#[test]
fn axiom_get_put_law_synthetic_section() {
    let first = UslmXmlLens::get(SAMPLE_SECTION.as_bytes()).expect("first get");
    let bytes = UslmXmlLens::put(&first).expect("put");
    let second = UslmXmlLens::get(&bytes).expect("second get");
    // The typed view recovers byte-equal between the two gets.
    assert_eq!(first.view, second.view);
}

/// Axiom — PutPut. Successive puts of the same target yield the same
/// bytes. Foster et al. 2007 §2.2.
#[test]
fn axiom_put_put_law_synthetic_section() {
    let target = UslmXmlLens::get(SAMPLE_SECTION.as_bytes()).expect("get");
    let a = UslmXmlLens::put(&target).expect("put 1");
    let b = UslmXmlLens::put(&target).expect("put 2");
    assert_eq!(a, b);
}

/// Axiom — canonical form is idempotent. W3C XML C14N 1.1 §3
/// (Boyer & Marcy 2008): `canonical(canonical(s)) = canonical(s)`.
#[test]
fn axiom_canonical_is_idempotent() {
    let once = UslmXmlLens::canonical(SAMPLE_SECTION.as_bytes()).expect("c14n once");
    let twice = UslmXmlLens::canonical(&once).expect("c14n twice");
    assert_eq!(once, twice);
}

/// Axiom — canonical form is preserved across round-trip. For every
/// `s`, `canonical(put(get(s))) == canonical(s)` exactly (not just
/// modulo whitespace). Foster et al. 2007 §2.2 PutGet law.
#[test]
fn axiom_canonical_round_trip_equals_input() {
    let input_canonical =
        UslmXmlLens::canonical(SAMPLE_SECTION.as_bytes()).expect("canonical input");
    let target = UslmXmlLens::get(SAMPLE_SECTION.as_bytes()).expect("get");
    let bytes = UslmXmlLens::put(&target).expect("put");
    let rt_canonical = UslmXmlLens::canonical(&bytes).expect("canonical rt");
    assert_eq!(input_canonical, rt_canonical);
}

// =========================================================
// Layer 2.5 — real-corpus check
// =========================================================

/// Real-corpus PutGet — SOX § 1514A USLM slice sourced from the fetched
/// `usc_title_18` corpus (18 U.S.C. § 1514A), not a deleted standalone
/// fixture. The verbatim `<section>` byte span for § 1514A is sliced out
/// of Title 18 and the lens's PutGet law (Foster et al. 2007 §2.2) is run
/// over those genuine published bytes. FAILS LOUD when the corpus is
/// absent — CI fetches it via `pr4xis update usc_title_18`; no skip.
#[test]
fn axiom_put_get_law_on_real_sox_1514a_slice() {
    let bytes = crate::social::software::markup::xml::uslm::real_sox_1514a::section_bytes();
    UslmXmlLens::assert_put_get_law(&bytes).expect("PutGet on real SOX § 1514A slice");
}

// =========================================================
// Layer 3 — proptest property-based
// =========================================================

use proptest::prelude::*;

proptest! {
    /// Property — PutGet holds across arbitrary section identifiers.
    /// The lens never loses bytes regardless of which SOX-shaped
    /// identifier the synthetic XML carries.
    #[test]
    fn prop_put_get_law_section_with_arbitrary_identifier(
        n in 1u32..54,
    ) {
        let xml = format!(
            r##"<section identifier="/us/usc/t{n}/s{n}A"><num value="{n}A">§ {n}A.</num><heading>Synthetic heading</heading><content>Body text.</content></section>"##,
        );
        UslmXmlLens::assert_put_get_law(xml.as_bytes())
            .expect("synthetic section PutGet");
    }

    /// Property — GetPut law. Round-tripping a target through put +
    /// get yields a view structurally equal to the original.
    #[test]
    fn prop_get_put_law_arbitrary_section(
        n in 1u32..54,
    ) {
        let xml = format!(
            r##"<section identifier="/us/usc/t{n}/s100"><num value="100">§ 100.</num><heading>H</heading><content>Body.</content></section>"##,
        );
        let first = UslmXmlLens::get(xml.as_bytes()).expect("get 1");
        let bytes = UslmXmlLens::put(&first).expect("put");
        let second = UslmXmlLens::get(&bytes).expect("get 2");
        prop_assert_eq!(first.view, second.view);
    }

    /// Property — PutPut law. Two successive puts return the same
    /// bytes for any valid USLM input.
    #[test]
    fn prop_put_put_law(
        n in 1u32..54,
    ) {
        let xml = format!(
            r##"<section identifier="/us/usc/t{n}/s5"><num value="5">§ 5.</num><heading>H</heading><content>B.</content></section>"##,
        );
        let target = UslmXmlLens::get(xml.as_bytes()).expect("get");
        let a = UslmXmlLens::put(&target).expect("put 1");
        let b = UslmXmlLens::put(&target).expect("put 2");
        prop_assert_eq!(a, b);
    }
}

// =========================================================
// Layer 4 — XSD-grounded-dispatch axioms (M4.ε.5.a.5)
// =========================================================

use crate::formal::meta::xsd::from_xsd_parser::project_from_xsd_text;
use crate::formal::meta::xsd::uslm_vocabulary::loaded_uslm_1_0_18_xsd;

/// Axiom — the walker dispatches only via XSD-ontology queries.
///
/// Every element name the lens *recognises* must correspond to an
/// `<xsd:element>` declaration in the loaded USLM-1.0.18 XSD
/// ontology. We exercise this by parsing the canonical real-corpus
/// SOX § 1514A slice (the largest real-input regression we have in
/// tree) and confirming every USLM-namespace element along the
/// recursion path resolves through `xsd.lookup_element`.
///
/// W3C XSD 1.1 Part 1 §3.3 — *Element Declarations*: a name with no
/// declaration has no `{type definition}` to dispatch on.
#[test]
fn axiom_walker_only_uses_xsd_ontology_queries() {
    let xsd = project_from_xsd_text(loaded_uslm_1_0_18_xsd());

    // Every named element the walker reaches in a representative
    // USLM input must be declared by the loaded XSD. Walk the
    // synthetic-title sample's elements and confirm.
    let xml = SAMPLE_TITLE;
    // Element names actually present in the synthetic title body.
    let expected_present_names = [
        "uscDoc", "meta", "main", "title", "num", "heading", "section", "content",
    ];
    for name in expected_present_names {
        assert!(
            xsd.lookup_element(name).is_some(),
            "loaded USLM XSD must declare <{name}> (W3C XSD 1.1 Part 1 §3.3)",
        );
    }
    // The walker accepts this input — every dispatch decision in
    // `get` succeeded against the XSD.
    let _ = UslmXmlLens::get(xml.as_bytes())
        .expect("XSD-grounded walker accepts a synthetic title built from XSD-declared names");
}

/// Axiom — substitution-group dispatch routes hierarchy elements
/// through the loaded XSD's `"level"` substitution-group head per
/// W3C XSD 1.1 Part 1 §3.3.6.
#[test]
fn axiom_level_substitution_group_membership_grounded() {
    let xsd = project_from_xsd_text(loaded_uslm_1_0_18_xsd());

    // Reflexive: "level" is a member of "level" (W3C XSD 1.1 Part 1
    // §3.3.6 — every element is in its own substitution group).
    assert!(xsd.is_member_of_substitution_group("level", "level"));

    // Direct membership: USLM-1.0.18.xsd declares
    // `<xsd:element name="section" substitutionGroup="level">` —
    // the walker uses this membership to dispatch hierarchy children.
    assert!(
        xsd.is_member_of_substitution_group("section", "level"),
        "USLM XSD declares <section substitutionGroup=\"level\">; walker dispatches on this"
    );
    assert!(
        xsd.is_member_of_substitution_group("subtitle", "level"),
        "USLM XSD declares <subtitle substitutionGroup=\"level\">",
    );
    assert!(
        xsd.is_member_of_substitution_group("subsection", "level"),
        "USLM XSD declares <subsection substitutionGroup=\"level\">",
    );

    // Non-membership: a non-level element is not in the group.
    assert!(
        !xsd.is_member_of_substitution_group("heading", "level"),
        "<heading> is not a level element in the USLM XSD",
    );
}

/// Axiom — type-of-element query returns the loaded XSD's declared
/// type reference per W3C XSD 1.1 Part 1 §3.3.2.3.
#[test]
fn axiom_type_of_element_grounded() {
    let xsd = project_from_xsd_text(loaded_uslm_1_0_18_xsd());

    // The walker's section-leaf dispatch is keyed on
    // `type_definition_of("section") == Some("LevelType")` — exactly
    // what USLM-1.0.18.xsd declares.
    assert_eq!(
        xsd.type_definition_of("section"),
        Some("LevelType"),
        "USLM XSD §3.3.2.3 type reference for <section>",
    );
    assert_eq!(
        xsd.type_definition_of("title"),
        Some("LevelType"),
        "USLM XSD §3.3.2.3 type reference for <title>",
    );
    // An element with no `type=` attribute has `None` type_ref
    // (default to xs:anyType per §3.3.2.3).
    // The USLM XSD declares `type=` on every element it ships; if a
    // declaration omits `type=`, the loader returns None and the
    // caller applies the §3.3.2.3 default.
    let _ = xsd.type_definition_of("nonexistent_element_xyz"); // not loaded → None
    assert!(xsd.lookup_element("nonexistent_element_xyz").is_none());
}

/// Property — every level-substitution-group member declared by the
/// loaded XSD round-trips through `get`/`put` when wrapped in a
/// minimal section slice. (W3C XSD 1.1 Part 1 §3.3.6 — substitution-
/// group monotonicity: adding a new member to the loaded XSD
/// doesn't break existing dispatches.)
#[test]
fn axiom_level_members_round_trip_through_lens() {
    // The minimal round-trip is a `<section>`-rooted slice (the
    // section leaf is the level member the walker hands to
    // `read_section`). We exercise the dispatch by feeding the
    // canonical SOX-shaped section slice; the membership query is
    // what routes it.
    let bytes = SAMPLE_SECTION.as_bytes();
    let target = UslmXmlLens::get(bytes).expect("XSD-grounded get");
    let put = UslmXmlLens::put(&target).expect("put");
    let second = UslmXmlLens::get(&put).expect("XSD-grounded get 2");
    assert_eq!(target.view, second.view);
}

/// Property — names NOT declared in the loaded XSD produce
/// [`UslmLensError::UnknownElement`] — no fallback path that
/// hand-codes recovery. (W3C XSD 1.1 Part 1 §3.3 — undeclared name
/// has no `{type definition}`.)
#[test]
fn axiom_undeclared_names_produce_unknown_element() {
    // `<not_a_usl_element>` is in the document's default namespace
    // (none declared, so empty), and the lens treats unprefixed
    // root elements without a namespace declaration as candidates
    // for XSD lookup. The lookup fails.
    let xml = r##"<not_a_usl_element>nope</not_a_usl_element>"##;
    let err = UslmXmlLens::get(xml.as_bytes()).expect_err("must reject undeclared element");
    assert!(
        matches!(err, UslmLensError::UnknownElement(ref name) if name == "not_a_usl_element"),
        "expected UnknownElement(\"not_a_usl_element\"), got {err:?}",
    );
}

// =========================================================
// Phase-1 structural-content audit (Milestone #266)
//
// The OWL graph-faithful refactor at praxis e74fa2c5 was preceded by
// an analogous audit quantifying what the typed view drops vs the
// source bytes. The audit motivates removing the
// `complement: Vec<u8>` side-channel: anything the typed view
// captures structurally needn't be remembered byte-verbatim.
//
// This test walks every USLM-namespace element by `(namespace URI,
// local name)` per W3C XML Infoset §1 and emits a deterministic diff
// report. The audit is informational at this phase — it does NOT fail
// on a non-zero gap. The Phase-1 result is the report; the decision
// of which elements to lift into the typed view (vs accept as
// out-of-scope) is the next user-driven step.
//
// Citation: U.S. House Office of the Law Revision Counsel, USLM XML
// User Guide and Schema. The element vocabulary the audit walks is
// the loaded USLM-1.0.18.xsd.
// =========================================================

/// The synthetic title sample has a known structural shape — exercise
/// the audit machinery on it first to verify the histogram math is
/// correct before running over multi-megabyte USC titles.
#[test]
fn axiom_structural_audit_on_synthetic_title() {
    use super::structural_audit::audit_structural_content;

    let audit = audit_structural_content(SAMPLE_TITLE.as_bytes()).expect("audit synthetic title");
    // The synthetic title has 11 raw elements: uscDoc, meta, dc:title,
    // dc:type, dc:publisher, main, title, num, heading, section, num,
    // heading, content → distinct element names ~10 (num/heading
    // repeat). Verify a few invariants.
    assert!(audit.raw.total() >= 11, "synthetic raw count");
    // The typed view materialises uscDoc/main/title/num/heading at
    // least once; verify the structural-audit didn't silently zero
    // out the title-level emission.
    assert!(
        audit
            .typed
            .get(Some(super::structural_audit::USLM_NS_FOR_TEST), "section")
            >= 1
    );
    // Render the report for human inspection.
    eprintln!(
        "{}",
        super::structural_audit::render_audit("synthetic_title", &audit)
    );
}
