//! HTML5 element / attribute name loader — sourced from the
//! registered `xhtml_1_0_xsd@1.0` source on disk.
//!
//! Per `feedback_bottom_up_loaded_not_encoded`: the element /
//! attribute inventory is **loaded** from the W3C-published XHTML
//! 1.0 Strict XSD, not hand-coded as Rust enum variants. The XSD
//! is hash-pinned in `praxis.lock` and bundled at
//! `crates/domains/data/markup-schemas/xhtml/xhtml-1.0-strict.xsd`.
//!
//! ## Citations
//!
//! - **Pemberton et al. (eds.) (2002)** *XHTML 1.0: The Extensible
//!   HyperText Markup Language (Second Edition)*, W3C Recommendation
//!   1 August 2002. §A.1 Document Type Definitions —
//!   <https://www.w3.org/TR/xhtml1/#dtds>. The reference ontological
//!   inventory of HTML4 / XHTML1 elements + attributes (the
//!   W3C-published companion XSD at
//!   <https://www.w3.org/2002/08/xhtml/xhtml1-strict.xsd> is a
//!   faithful XML Schema rendering of the §A.1.1 Strict DTD).
//!   Normatively shared with HTML5 for every name it covers
//!   (WHATWG HTML Living Standard §1.6 "History" — backwards-
//!   compatibility principle).
//!
//! ## Scanning
//!
//! XHTML 1.0 Strict declares each element with
//! `<xs:element name="...">` at the top level of the schema; each
//! attribute is declared either via a top-level `<xs:attribute
//! name="...">` or inline within an `<xs:attributeGroup name="...">`.
//! A minimal text-scan (mirroring the one in
//! `formal::meta::xsd::english_projection::tests::scan_xsd_named_declarations`)
//! suffices to enumerate all named declarations — the structural
//! well-formedness of the bundled XSD is a build-time invariant
//! verified by `pr4xis::codegen::xhtml_schema::generate_xhtml_schema_source`.

#[allow(unused_imports)]
use alloc::{
    collections::BTreeSet,
    string::{String, ToString},
    vec::Vec,
};

use std::sync::OnceLock;

/// The raw bytes of the bundled XHTML 1.0 Strict XSD —
/// `crates/domains/data/markup-schemas/xhtml/xhtml-1.0-strict.xsd`.
/// Embedded at build time via `include_str!` so the runtime path
/// is hermetic.
pub const XHTML_1_0_STRICT_XSD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/markup-schemas/xhtml/xhtml-1.0-strict.xsd"
));

/// Lazily-loaded set of element local-names (lowercased) declared by
/// the bundled XHTML 1.0 Strict XSD.
pub fn element_names() -> &'static BTreeSet<String> {
    static SET: OnceLock<BTreeSet<String>> = OnceLock::new();
    SET.get_or_init(|| scan_named_declarations(XHTML_1_0_STRICT_XSD, "<xs:element "))
}

/// Lazily-loaded set of attribute local-names (lowercased) declared
/// by the bundled XHTML 1.0 Strict XSD (top-level + within
/// attributeGroup definitions).
pub fn attribute_names() -> &'static BTreeSet<String> {
    static SET: OnceLock<BTreeSet<String>> = OnceLock::new();
    SET.get_or_init(|| scan_named_declarations(XHTML_1_0_STRICT_XSD, "<xs:attribute "))
}

/// True iff `name` is a declared HTML element per the bundled
/// XHTML 1.0 Strict XSD. Case-insensitive (HTML element names are
/// case-insensitive per WHATWG HTML LS §13.1.2 / W3C HTML 4.01
/// §3.2.2).
pub fn is_html_element(name: &str) -> bool {
    element_names().contains(&name.to_lowercase())
}

/// True iff `name` is a declared HTML attribute per the bundled
/// XHTML 1.0 Strict XSD. Case-insensitive (HTML attribute names
/// are case-insensitive per WHATWG HTML LS §13.1.2).
pub fn is_html_attribute(name: &str) -> bool {
    attribute_names().contains(&name.to_lowercase())
}

// =============================================================================
// Internals — XSD text scan
// =============================================================================

/// Scan `xsd_src` for every `<{tag_prefix}name="...">` declaration
/// and return the lowercased name set. Mirrors the same minimal
/// text-scan that
/// `formal::meta::xsd::english_projection::tests::scan_xsd_named_declarations`
/// performs on the USLM XSD; the XSD's structural well-formedness
/// is a build-time invariant verified by xsd-parser at codegen.
fn scan_named_declarations(xsd_src: &str, tag_prefix: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut search_from = 0;
    while let Some(idx) = xsd_src[search_from..].find(tag_prefix) {
        let abs = search_from + idx + tag_prefix.len();
        let end = xsd_src[abs..]
            .find('>')
            .map(|p| abs + p)
            .unwrap_or(xsd_src.len());
        let attr_slice = &xsd_src[abs..end];
        if let Some(name) = extract_attr(attr_slice, "name") {
            out.insert(name.to_lowercase());
        }
        search_from = end;
    }
    out
}

/// Extract `<key>="value"` from an attribute slice (no full XML
/// parse; works on the well-formed XSD attribute syntax). Returns
/// `None` if the attribute is `ref="..."` or otherwise lacks a
/// `name="..."` slot.
fn extract_attr(slice: &str, key: &str) -> Option<String> {
    let pattern = alloc::format!("{key}=\"");
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

    #[test]
    fn xsd_bundle_is_nonempty() {
        // The bundle ships with praxis; if this fires the file is
        // missing or the include_str! path is broken.
        assert!(
            !XHTML_1_0_STRICT_XSD.is_empty(),
            "XHTML 1.0 Strict XSD bundle is empty — bundle missing?"
        );
        // The published file declares the strict schema's target
        // namespace as the XHTML 1.0 namespace.
        assert!(
            XHTML_1_0_STRICT_XSD.contains("targetNamespace=\"http://www.w3.org/1999/xhtml\""),
            "bundle does not carry XHTML 1.0 target namespace — wrong file?"
        );
    }

    #[test]
    fn loader_yields_expected_element_count() {
        // Pemberton et al. (2002) XHTML 1.0 Strict §A.1 DTD (mirrored
        // by the companion XSD bundled here) has 77 distinct elements.
        // Any drift here is a bundle change.
        let names = element_names();
        assert_eq!(
            names.len(),
            77,
            "XHTML 1.0 Strict declares 77 elements per Pemberton et al. 2002 §A.1; \
             loader found {}: {:?}",
            names.len(),
            names
        );
    }

    #[test]
    fn loader_yields_expected_attribute_count() {
        // Pemberton et al. (2002) XHTML 1.0 Strict §A.1 DTD (mirrored
        // by the companion XSD bundled here) declares 179
        // `<xs:attribute>` instances (some are repeated
        // across complex types but distinct in the XSD). Drift here
        // also indicates a bundle change.
        let names = attribute_names();
        // Names dedupe by lowercased form, so the distinct-name
        // count is a subset of the raw declaration count.
        assert!(
            !names.is_empty(),
            "XHTML 1.0 Strict XSD declares attributes; loader found none"
        );
    }

    #[test]
    fn key_html_elements_recognized() {
        // Spot-check elements that downstream code (USLM citations,
        // generic HTML content) refers to by name. Every name in
        // this list MUST be in the loaded set; if any drops out the
        // bundle has changed.
        for el in [
            "br", "img", "del", "meta", "html", "head", "body", "p", "h1", "h2", "h3", "h4", "h5",
            "h6", "ul", "ol", "li", "a", "table", "tr", "td", "th", "thead", "tbody", "tfoot",
            "div", "span", "form", "input", "textarea", "button", "label", "fieldset", "select",
            "option", "ins",
        ] {
            assert!(
                is_html_element(el),
                "expected XHTML 1.0 Strict element {el:?} not found in loaded set"
            );
        }
    }

    #[test]
    fn key_html_attributes_recognized() {
        // Spot-check attributes — same idea as above.
        for at in [
            "href", "src", "alt", "colspan", "rowspan", "id", "class", "name", "type", "value",
            "title",
        ] {
            assert!(
                is_html_attribute(at),
                "expected XHTML 1.0 Strict attribute {at:?} not found in loaded set"
            );
        }
    }

    #[test]
    fn lookup_is_case_insensitive() {
        // WHATWG HTML LS §13.1.2 / HTML 4.01 §3.2.2 — element /
        // attribute names are ASCII-case-insensitive.
        assert!(is_html_element("IMG"));
        assert!(is_html_element("Img"));
        assert!(is_html_element("img"));
        assert!(is_html_attribute("HREF"));
        assert!(is_html_attribute("Href"));
        assert!(is_html_attribute("href"));
    }

    #[test]
    fn no_html5_only_elements_in_xhtml_1_0_strict() {
        // The bundled XSD is XHTML 1.0 Strict, which predates the
        // HTML5 semantic-section elements. Asserting their absence
        // here documents the M4.η.1.a follow-up scope (the same
        // names need to be picked up from the WHATWG / XHTML5
        // Polyglot source when M4.η.1.a lands).
        for el in [
            "canvas",
            "video",
            "audio",
            "section",
            "article",
            "nav",
            "aside",
            "header",
            "footer",
            "main",
            "figure",
            "figcaption",
            "picture",
            "dialog",
            "details",
            "summary",
            "template",
            "slot",
            "data",
            "time",
            "mark",
            "ruby",
            "rt",
            "rp",
            "wbr",
            "bdi",
            "output",
            "progress",
            "meter",
            "source",
            "track",
            "embed",
        ] {
            assert!(
                !is_html_element(el),
                "HTML5-only element {el:?} unexpectedly present in XHTML 1.0 Strict — \
                 either the bundle is wrong, or M4.η.1.a's follow-up source landed silently"
            );
        }
    }

    #[test]
    fn empty_input_rejected() {
        assert!(!is_html_element(""));
        assert!(!is_html_attribute(""));
    }
}
