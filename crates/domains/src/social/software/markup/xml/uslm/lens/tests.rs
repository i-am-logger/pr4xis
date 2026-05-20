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
    let err =
        UslmXmlLens::get(b"<other-root>nope</other-root>").expect_err("must reject non-USLM root");
    assert!(matches!(
        err,
        UslmLensError::Read(UslmReadError::NoUsCodeRoot)
    ));
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
// Layer 2.5 — real-corpus check (skip when XML not present)
// =========================================================

/// Real-corpus PutGet — SOX § 1514A USLM slice (if on disk).
/// Skips when the data isn't present so CI works without the LRC
/// USLM bundle.
#[test]
fn axiom_put_get_law_on_real_sox_1514a_slice() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data/legal/statutes/us_federal/sox_1514a/sox_1514a-2002.xml");
    if !path.exists() {
        eprintln!(
            "SKIP: real SOX 1514A slice not on disk at {}",
            path.display()
        );
        return;
    }
    let bytes = std::fs::read(&path).expect("read real slice");
    UslmXmlLens::assert_put_get_law(&bytes).expect("PutGet on real SOX slice");
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
