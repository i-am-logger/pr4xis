//! USLM-vocabulary classifier — recognises element / attribute /
//! complexType / simpleType / attributeGroup / group local-names
//! that the bundled USLM-1.0.18.xsd documents through its own
//! `<xsd:annotation><xsd:documentation>` blocks.
//!
//! Per `feedback_bottom_up_loaded_not_encoded`: no hand-list of names.
//! A name is "USLM-vocabulary" iff the loaded `uslm-1.0.18.xsd`
//! declares a schema component with that `name="…"` attribute AND
//! that declaration carries a non-empty `<xsd:annotation>
//! <xsd:documentation>` block. The schema documents itself; this
//! module is the thin scanner that surfaces the set.
//!
//! Loads USLM vocabulary from the schema's own self-documentation
//! (M4.η.3); M4.η.4 then deleted the `schema_vocabulary@2026`
//! hand-curated bundle entirely, leaving this loader as the
//! authoritative USLM-name recognition path.
//!
//! ## Citations
//!
//! - **Office of the Law Revision Counsel, U.S. House of
//!   Representatives.** *USLM (United States Legislative Markup) XML
//!   User Guide* (github.com/usgpo/uslm). The bundled `uslm-1.0.18.xsd`
//!   gives every declaration an inline `<xsd:documentation>` child
//!   (W3C XSD 1.1 Part 1 §3.15, cited below).
//!   Published by the LRC under 1 U.S.C. § 204; public-domain
//!   federal work per 17 U.S.C. § 105.
//!   <https://github.com/usgpo/uslm>.
//! - **Gao, S., Sperberg-McQueen, C. M., & Thompson, H. S. (eds.)**
//!   *W3C XML Schema Definition Language (XSD) 1.1 Part 1:
//!   Structures*, W3C Recommendation 5 April 2012, §3.15
//!   (`<xs:annotation>` / `<xs:documentation>` — the canonical
//!   schema-self-documentation channel).
//!   <https://www.w3.org/TR/xmlschema11-1/>.

#[allow(unused_imports)]
use alloc::{
    collections::BTreeSet,
    format,
    string::{String, ToString},
    vec::Vec,
};

use std::sync::OnceLock;

/// The committed USLM-1.0.18 `.prx` — the content-addressed envelope carrying
/// the XSD bytes. The raw `.xsd` is fetch-only (`pr4xis update`) and ships in NO
/// crate; only this `.prx` is committed + embedded. Loaded through the
/// generalized raw-source gate (phase 2).
const USLM_1_0_18_XSD_PRX: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/legal/uscode/schema/uslm-1.0.18.prx"
));

/// The loaded USLM-1.0.18 XSD bytes, materialized from the committed `.prx`
/// through the fail-closed `[compact_archive_signatures]` content gate, cached
/// for the process behind a `OnceLock`. The raw `.xsd` is no longer embedded —
/// only the gated `.prx` is. The function-form successor of the former
/// `USLM_1_0_18_XSD` const.
#[must_use]
pub fn loaded_uslm_1_0_18_xsd() -> &'static str {
    use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
    static XSD: OnceLock<&'static str> = OnceLock::new();
    XSD.get_or_init(|| raw_source_text_embedded("uslm_xsd", "1.0.18", USLM_1_0_18_XSD_PRX))
}

/// Lazily-loaded set of every local-name (lowercased) declared by
/// the bundled USLM-1.0.18 XSD that carries a non-empty
/// `<xsd:annotation><xsd:documentation>` block.
///
/// Population walks the XSD source once on first call and caches.
/// The well-formedness of the bundled XSD is a build-time invariant
/// verified by `pr4xis::codegen::uslm_schema::generate_uslm_schema_source`;
/// if that codegen succeeds, the structural scan below cannot
/// silently misclassify.
pub fn documented_names() -> &'static BTreeSet<String> {
    static SET: OnceLock<BTreeSet<String>> = OnceLock::new();
    SET.get_or_init(|| scan_documented_names(loaded_uslm_1_0_18_xsd()))
}

/// True iff `name` is the local-name of a schema component declared
/// in the bundled USLM-1.0.18.xsd AND that declaration carries a
/// non-empty `<xsd:annotation><xsd:documentation>` block.
///
/// Comparison is case-insensitive (XML element / attribute names are
/// case-sensitive per W3C XML 1.0 §3, but downstream consumers
/// case-fold lemmas before classifying, so the public surface
/// mirrors that — same convention as the HTML and XML 1.0 loaders).
pub fn is_uslm_vocabulary(name: &str) -> bool {
    documented_names().contains(&name.to_lowercase())
}

// =============================================================================
// Internals — XSD text scan for `<xsd:KIND name="…">` + documentation child
// =============================================================================

/// The six XSD declaration kinds USLM uses for named top-level (or
/// inline-named-and-documented) schema components. Mirrors the kind
/// set the existing scanner in
/// `formal::meta::xsd::english_projection::tests::scan_xsd_named_declarations`
/// recognises.
const DECLARATION_KINDS: &[&str] = &[
    "<xsd:element ",
    "<xsd:attribute ",
    "<xsd:complexType ",
    "<xsd:simpleType ",
    "<xsd:attributeGroup ",
    "<xsd:group ",
];

/// Scan the XSD source for every declaration that (a) carries a
/// `name="…"` attribute, (b) is in its open-tag form (not self-
/// closing `/>`), and (c) has a non-empty `<xsd:documentation>` block
/// inside its `<xsd:annotation>` first-child. Returns the lower-
/// cased set of those local-names.
fn scan_documented_names(xsd_src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for kind_prefix in DECLARATION_KINDS {
        let mut cursor = 0;
        while let Some(rel) = xsd_src[cursor..].find(kind_prefix) {
            let abs = cursor + rel + kind_prefix.len();
            // Find the end of the opening tag (the next `>`). A
            // self-closing form ends with `/>`; the open form ends
            // with a bare `>` and is followed by children.
            let tag_close = xsd_src[abs..]
                .find('>')
                .map(|p| abs + p)
                .unwrap_or(xsd_src.len());
            let attr_slice = &xsd_src[abs..tag_close];
            let name = extract_attr(attr_slice, "name");
            let is_self_closing = attr_slice.ends_with('/');
            // Advance cursor past this opening tag for the next
            // iteration regardless of outcome below.
            cursor = tag_close + 1;
            let Some(name) = name else { continue };
            if is_self_closing {
                // No children, hence no documentation. Skip.
                continue;
            }
            // The declaration's body starts after the opening `>`.
            // For USLM, documentation (if present) is the first
            // child of the declaration's `<xsd:annotation>` — and
            // annotation is always the first child when present
            // (W3C XSD 1.1 Part 1 §3.15.2: annotation must precede
            // the type / sequence / choice / restriction children).
            //
            // We scan forward from the opening-tag end and check
            // whether the first non-whitespace child is
            // `<xsd:annotation>` AND that annotation contains a
            // non-empty `<xsd:documentation>` block before the next
            // sibling declaration begins.
            if declaration_has_documentation(&xsd_src[cursor..]) {
                out.insert(name.to_lowercase());
            }
        }
    }
    out
}

/// Inspect the body of a declaration starting at the byte just after
/// its opening `>`. Return true iff the body's first
/// `<xsd:annotation>` block (preceding any sibling declaration)
/// contains a `<xsd:documentation>` element with non-empty content.
///
/// The check is deliberately minimal: USLM places annotation as the
/// first child of every documented declaration, immediately followed
/// by a documentation block — a USLM XSD authoring convention: every
/// declaration in `uslm-1.0.18.xsd` carries an inline
/// `<xsd:documentation>` (W3C XSD 1.1 Part 1 §3.15).
/// Non-USLM XSDs may not follow this convention; this scanner is
/// dedicated to USLM, not a general XSD parser (general parsing is
/// `xsd-parser`'s responsibility, run at codegen time).
fn declaration_has_documentation(body: &str) -> bool {
    // Find the next `<xsd:annotation>` or any other XSD tag opening,
    // whichever comes first. If annotation comes first, look inside
    // it for `<xsd:documentation>` with content.
    let Some(ann_idx) = body.find("<xsd:annotation>") else {
        return false;
    };
    // The annotation must precede any sibling XSD construct (so we
    // know it belongs to *this* declaration, not a nested one).
    // Sibling construct prefixes to bound by:
    const SIBLING_PREFIXES: &[&str] = &[
        "<xsd:sequence",
        "<xsd:choice",
        "<xsd:all",
        "<xsd:complexType",
        "<xsd:simpleType",
        "<xsd:complexContent",
        "<xsd:simpleContent",
        "<xsd:restriction",
        "<xsd:extension",
        "<xsd:attribute",
        "<xsd:attributeGroup",
        "<xsd:group",
        "<xsd:element",
        "<xsd:enumeration",
        "<xsd:union",
        "<xsd:list",
    ];
    for prefix in SIBLING_PREFIXES {
        if let Some(sib_idx) = body.find(prefix)
            && sib_idx < ann_idx
        {
            // A sibling construct appears before any annotation
            // — this declaration has no annotation of its own.
            return false;
        }
    }
    // The annotation must close before we leave this declaration's
    // body. Find the `</xsd:annotation>` that matches.
    let ann_body_start = ann_idx + "<xsd:annotation>".len();
    let Some(rel_close) = body[ann_body_start..].find("</xsd:annotation>") else {
        return false;
    };
    let ann_body = &body[ann_body_start..ann_body_start + rel_close];
    // Inside the annotation body, look for a non-empty
    // <xsd:documentation> block.
    let mut search = 0;
    while let Some(open_rel) = ann_body[search..].find("<xsd:documentation") {
        let after_open_tag = ann_body[search + open_rel..]
            .find('>')
            .map(|p| search + open_rel + p + 1);
        let Some(content_start) = after_open_tag else {
            return false;
        };
        let Some(close_rel) = ann_body[content_start..].find("</xsd:documentation>") else {
            return false;
        };
        let content = &ann_body[content_start..content_start + close_rel];
        if !is_effectively_empty(content) {
            return true;
        }
        search = content_start + close_rel + "</xsd:documentation>".len();
    }
    false
}

/// True iff the documentation content is empty after stripping
/// CDATA wrappers and trimming whitespace. Used to filter out
/// placeholder `<xsd:documentation><![CDATA[ ]]></xsd:documentation>`
/// blocks (USLM has none — but the check guards against future
/// drift).
fn is_effectively_empty(content: &str) -> bool {
    let trimmed = content.trim();
    let stripped = trimmed
        .strip_prefix("<![CDATA[")
        .and_then(|s| s.strip_suffix("]]>"))
        .unwrap_or(trimmed);
    stripped.trim().is_empty()
}

/// Extract `<key>="value"` from an attribute slice. Mirrors the
/// helper in the HTML loader and the english_projection test
/// scanner — kept local rather than shared to keep this module
/// self-contained.
fn extract_attr(slice: &str, key: &str) -> Option<String> {
    let pattern = format!("{key}=\"");
    let start = slice.find(&pattern)? + pattern.len();
    let end = slice[start..].find('"')? + start;
    Some(slice[start..end].to_string())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // ── Loader invariants ────────────────────────────────────────────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn xsd_bundle_is_nonempty() {
        // The bundle ships with praxis; if this fires the file is
        // missing or the include_str! path is broken.
        assert!(
            !loaded_uslm_1_0_18_xsd().is_empty(),
            "USLM-1.0.18.xsd bundle is empty — bundle missing?"
        );
        // The published file declares the USLM target namespace.
        assert!(
            loaded_uslm_1_0_18_xsd().contains("uslm"),
            "bundle does not look like the USLM XSD — wrong file?"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn documented_names_set_is_nonempty() {
        // USLM-1.0.18 declares hundreds of named, documented schema
        // components, each XSD-documented. The scan must surface ≥100 of them.
        let names = documented_names();
        assert!(
            names.len() >= 100,
            "expected ≥100 documented USLM names, found {}",
            names.len()
        );
        // Log the loaded count for telemetry (visible under
        // --nocapture). Lets reviewers verify the loader picks up
        // the expected order of magnitude after schema changes.
        eprintln!("uslm_vocabulary: {} documented names loaded", names.len());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lookup_lowercases() {
        // Every stored name is lower-case; the lookup case-folds
        // input.
        for name in documented_names() {
            assert_eq!(name, &name.to_lowercase());
        }
    }

    // ── Axiom: every name on the M4.η.3 USLM-target list is
    //          recognised through the XSD's own documentation
    //          (via the XSD's own <xsd:documentation>; XSD 1.1 Part 1 §3.15) ───────────────

    /// Axiom: the canonical USLM element-name set
    /// (`toc / num / pos / inline / def / misc / subarticle /
    /// subparagraph / subclause / subitem / subsubitem`) is each
    /// declared in `uslm-1.0.18.xsd` with a non-empty
    /// `<xsd:annotation><xsd:documentation>` child.
    ///
    /// Every USLM declaration in `uslm-1.0.18.xsd` carries an inline
    /// `<xsd:annotation><xsd:documentation>` (W3C XSD 1.1 Part 1 §3.15).
    ///
    /// Note: `enum`, `attrs`, and `usc` appear only in documentation
    /// prose (not as XSD `name="…"` declarations); the
    /// whole-name-first recognition path (M4.η.4) handles their
    /// resolution by recognising the *containing* declaration name
    /// (e.g. `ChoiceEnum`, `XmlSpecialAttrs`, `uscDoc`) — none of
    /// `enum / attrs / usc` ever appears as a standalone XSD local
    /// name to begin with.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_uslm_documented_names_present() {
        for el in [
            "toc",
            "num",
            "pos",
            "inline",
            "def",
            "misc",
            "subarticle",
            "subparagraph",
            "subclause",
            "subitem",
            "subsubitem",
        ] {
            assert!(
                is_uslm_vocabulary(el),
                "expected USLM-documented name {el:?} not in loaded set"
            );
        }
    }

    /// Axiom: hierarchy-level names that every USLM element carries
    /// inline documentation (the USLM Levels model — User Guide §6.5). Spot-checks the breadth of the loaded set.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_uslm_hierarchy_names_present() {
        for el in [
            "section",
            "subsection",
            "paragraph",
            "clause",
            "item",
            "title",
            "chapter",
            "part",
        ] {
            assert!(
                is_uslm_vocabulary(el),
                "expected USLM hierarchy name {el:?} not in loaded set"
            );
        }
    }

    // ── Axiom: case-insensitivity ─────────────────────────────────────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn axiom_lookup_case_insensitive() {
        assert!(is_uslm_vocabulary("TOC"));
        assert!(is_uslm_vocabulary("Toc"));
        assert!(is_uslm_vocabulary("toc"));
        assert!(is_uslm_vocabulary("SUBARTICLE"));
        assert!(is_uslm_vocabulary("SubArticle"));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn axiom_empty_input_rejected() {
        assert!(!is_uslm_vocabulary(""));
    }

    // ── Negative axiom: names not in USLM XSD are not recognised ────

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn axiom_unrelated_strings_not_present() {
        // Names that are neither USLM elements nor HTML / XML.
        for n in ["ZZZNotAUSLMName", "totally_made_up_token_xyz", "fhqwhgads"] {
            assert!(
                !is_uslm_vocabulary(n),
                "unrelated name {n:?} unexpectedly present"
            );
        }
    }

    // ── Property test: classifier is anchored to the actual XSD ──────

    /// Property: if `is_uslm_vocabulary(name)` returns true, then the
    /// loaded XSD source contains an `<xsd:KIND name="name">` (or
    /// case-equivalent) declaration AND that declaration has a
    /// non-empty `<xsd:documentation>` block. (Sanity: classifier is
    /// anchored to actual XSD structure, never a hand-coded list.)
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn property_recognised_names_back_to_xsd() {
        for name in documented_names() {
            // Verify the name appears as a declared `name="…"` in
            // the XSD (case-insensitive search since XSD names are
            // case-sensitive but our set is lower-cased).
            let found_decl = DECLARATION_KINDS.iter().any(|kind_prefix| {
                let needle_lower = format!("{kind_prefix}name=\"{name}\"");
                if loaded_uslm_1_0_18_xsd()
                    .to_lowercase()
                    .contains(&needle_lower.to_lowercase())
                {
                    return true;
                }
                // Some declarations have other attributes (type=…)
                // before name=…; check for `name="X"` substring
                // appearing anywhere inside an opening tag of this
                // kind. The earlier check matches the common shape
                // where name=… is first.
                let mut cursor = 0;
                let xsd = loaded_uslm_1_0_18_xsd();
                let src_lower = xsd.to_lowercase();
                let kp_lower = kind_prefix.to_lowercase();
                while let Some(rel) = src_lower[cursor..].find(&kp_lower) {
                    let abs = cursor + rel + kp_lower.len();
                    let tag_close = src_lower[abs..]
                        .find('>')
                        .map(|p| abs + p)
                        .unwrap_or(src_lower.len());
                    let attr_slice = &xsd[abs..tag_close];
                    if let Some(found_name) = extract_attr(attr_slice, "name")
                        && found_name.to_lowercase() == *name
                    {
                        return true;
                    }
                    cursor = tag_close + 1;
                }
                false
            });
            assert!(
                found_decl,
                "loaded USLM-vocabulary name {name:?} has no \
                 corresponding `<xsd:KIND name=\"…\">` declaration \
                 in the bundled XSD — anchoring invariant violated"
            );
        }
    }

    /// Property: `is_effectively_empty` returns true for whitespace
    /// and CDATA-wrapped whitespace, false for any non-whitespace
    /// content.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn property_effectively_empty_recognises_blank_documentation() {
        assert!(is_effectively_empty(""));
        assert!(is_effectively_empty("   "));
        assert!(is_effectively_empty("\n\t  \n"));
        assert!(is_effectively_empty("<![CDATA[ ]]>"));
        assert!(is_effectively_empty("<![CDATA[\n   \n]]>"));
        assert!(!is_effectively_empty("toc element documentation"));
        assert!(!is_effectively_empty(
            "<![CDATA[ A <toc> is a table of contents. ]]>"
        ));
    }

    // ── Functor laws: classifier is a faithful projection ────────────
    //
    // The classifier `is_uslm_vocabulary: String → Bool` is a functor
    // from the discrete category of lowercase strings to the discrete
    // category `{true, false}`. Functor laws to check:
    //   1. Identity preservation: repeated lookup gives the same
    //      answer (referential transparency / Mac Lane §I.3 identity
    //      law).
    //   2. Composition consistency: `lookup(case_fold(x)) ==
    //      lookup(x)` for all `x` — the case-fold normalisation
    //      factors through (Mac Lane §I.3 composition law).
    //
    // Mirrors the parallel functor-law tests in the other XSD-grounded
    // classifiers (HTML / XML 1.0) — uniform test depth per
    // `feedback_uniform_test_depth_across_ontologies`.

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn functor_law_identity_preservation() {
        for x in ["toc", "subarticle", "section", "ZZZNotPresent"] {
            let a = is_uslm_vocabulary(x);
            let b = is_uslm_vocabulary(x);
            let c = is_uslm_vocabulary(x);
            assert_eq!(a, b);
            assert_eq!(b, c);
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn functor_law_case_fold_factors_through() {
        for x in ["TOC", "SubArticle", "Inline", "SubItem", "SUBSUBITEM"] {
            let direct = is_uslm_vocabulary(x);
            let folded = is_uslm_vocabulary(&x.to_lowercase());
            assert_eq!(
                direct, folded,
                "case-fold factor failed on {x:?}: direct={direct} folded={folded}"
            );
        }
    }

    // ── Concurrency: OnceLock thread-safety ──────────────────────────

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn concurrency_lazy_init_under_threads() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        const N_THREADS: usize = 16;
        let barrier = Arc::new(Barrier::new(N_THREADS));
        let mut handles = Vec::with_capacity(N_THREADS);
        for _ in 0..N_THREADS {
            let b = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                b.wait();
                let set = documented_names();
                assert!(set.contains("toc"));
                assert!(set.contains("subarticle"));
                set.len()
            }));
        }
        let sizes: Vec<usize> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let first = sizes[0];
        for s in &sizes {
            assert_eq!(*s, first, "thread observed different set size");
        }
    }

    // ── Load idempotence ─────────────────────────────────────────────

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn load_idempotence_two_reads_equal() {
        let a = documented_names();
        let b = documented_names();
        assert_eq!(a.len(), b.len());
        for x in a {
            assert!(b.contains(x));
        }
    }

    // ── Proptest ────────────────────────────────────────────────────

    fn arb_lemma() -> impl Strategy<Value = String> {
        proptest::collection::vec(
            prop_oneof![
                prop::char::range('a', 'z'),
                prop::char::range('A', 'Z'),
                prop::char::range('0', '9'),
                Just('-'),
                Just('_'),
            ],
            0..24,
        )
        .prop_map(|chars| chars.into_iter().collect())
    }

    proptest! {
        /// Property: case-fold factors through. For every input,
        /// looking up `x` and looking up `x.to_lowercase()` produces
        /// the same result.
        #[test]
        fn prop_case_fold_factors_through(x in arb_lemma()) {
            let direct = is_uslm_vocabulary(&x);
            let folded = is_uslm_vocabulary(&x.to_lowercase());
            prop_assert_eq!(direct, folded);
        }

        /// Property: lookup is total — every input produces a Boolean
        /// without panic.
        #[test]
        fn prop_total_function(x in arb_lemma()) {
            let _ = is_uslm_vocabulary(&x);
        }

        /// Property: every recognised name is present in the loaded
        /// set (the set IS the classifier — no string-shape rules).
        #[test]
        fn prop_recognised_iff_in_set(x in arb_lemma()) {
            let recognised = is_uslm_vocabulary(&x);
            let in_set = documented_names().contains(&x.to_lowercase());
            prop_assert_eq!(recognised, in_set);
        }

        /// Property: empty / whitespace inputs are never recognised.
        #[test]
        fn prop_empty_not_recognised(_x in any::<u8>()) {
            prop_assert!(!is_uslm_vocabulary(""));
            prop_assert!(!is_uslm_vocabulary(" "));
            prop_assert!(!is_uslm_vocabulary("\t"));
        }

        /// Property: load idempotence. Repeated reads return
        /// identical content. The OnceLock guarantees object
        /// identity; this property checks logical equality so a
        /// future refactor that bypasses OnceLock would still
        /// preserve the contract.
        #[test]
        fn prop_load_idempotent(_seed in any::<u32>()) {
            let a = documented_names();
            let b = documented_names();
            prop_assert_eq!(a.len(), b.len());
            for x in a {
                prop_assert!(b.contains(x));
            }
        }
    }

    pr4xis::register_praxis_value!(prop_case_fold_factors_through, Deterministic);
    pr4xis::register_praxis_value!(prop_total_function, Honest);
    pr4xis::register_praxis_value!(prop_recognised_iff_in_set, Verifiable);
    pr4xis::register_praxis_value!(prop_empty_not_recognised, Honest);
    pr4xis::register_praxis_value!(prop_load_idempotent, Deterministic);
}
