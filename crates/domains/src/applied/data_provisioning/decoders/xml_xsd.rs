//! XML XSD decoder — raw bytes → loaded [`XsdOntologyInstance`].
//!
//! This decoder is the `ContentType::XmlXsd` entry in the decoder
//! dispatch table. It takes raw bytes (the fetched XSD file contents)
//! and returns a typed
//! `XsdOntologyInstance`
//! — the praxis XSD ontology applied to the source text.
//!
//! **This decoder does not reimplement any XML or XSD parsing.** It
//! delegates to:
//!
//! - [`crate::social::software::markup::xml::reader::read_xml`] — the
//!   generic XML substrate, used only to verify that the document's
//!   root element is `xsd:schema` / `xs:schema` (W3C XSD 1.1 Part 1
//!   §3.1.2). On the happy path the parsed `XmlDocument` is discarded;
//!   it exists purely as a fail-closed gate.
//! - [`crate::formal::meta::xsd::from_xsd_parser::project_from_xsd_text`]
//!   — the praxis functor from XSD source text into the typed
//!   [`XsdOntologyInstance`]. The functor itself cites W3C XSD 1.1
//!   Part 1 §3.3 (Element Declarations) and §3.3.6 (Substitution
//!   Groups) for what it extracts.
//!
//! Build-time validity of the XSD (acyclic derivation, every
//! `substitutionGroup` head resolvable, …) is the job of
//! `pr4xis::codegen::uslm_schema` (via the `xsd-parser` 1.5.2 crate);
//! see Bergmann, S. *xsd-parser*, MIT, v1.5.2,
//! <https://github.com/Bergmann89/xsd-parser>. This runtime decoder is
//! the read-side dual — it doesn't generate Rust code, it returns the
//! ontology instance that downstream walkers (e.g. the USLM
//! `WellBehavedLens`) dispatch on.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::meta::xsd::XSD_NAMESPACE_URI;
use crate::formal::meta::xsd::from_xsd_parser::{XsdOntologyInstance, project_from_xsd_text};
use crate::social::software::markup::xml::reader::read_xml;

/// Decode raw bytes as a W3C XSD 1.1 schema document.
///
/// Expects UTF-8 XML text as bytes. The decode pipeline is:
///
/// 1. Validate UTF-8 → [`DecodeError::NotUtf8`] on failure.
/// 2. Parse the XML through the praxis XML reader; reject documents
///    whose root element is not `xsd:schema` / `xs:schema`
///    (W3C XSD 1.1 Part 1 §3.1.2) → [`DecodeError::Xsd`] on failure.
/// 3. Project through the praxis XSD ontology functor
///    [`project_from_xsd_text`] into an [`XsdOntologyInstance`].
///
/// # Errors
///
/// Returns a [`DecodeError`] if the bytes are not valid UTF-8, the
/// XML reader rejects the document as malformed, or the root element
/// is not an `xsd:schema` declaration.
/// The [`ContentType`](crate::applied::data_provisioning::ontology::ContentType)
/// this module realizes -- the single declaration of which content type
/// this file decodes, read by `super::has_decoder_for` (audit 2026-06-12 D-22).
pub const DECODES: crate::applied::data_provisioning::ontology::ContentType =
    crate::applied::data_provisioning::ontology::ContentType::XmlXsd;

pub fn decode(bytes: &[u8]) -> Result<XsdOntologyInstance, DecodeError> {
    let text = core::str::from_utf8(bytes).map_err(|_| DecodeError::NotUtf8)?;

    // Strip the UTF-8 byte-order mark if present (W3C XML 1.0 §F.1
    // "Detection Without External Encoding Information" recognises the
    // U+FEFF byte sequence `EF BB BF` as an optional leading marker on
    // UTF-8 XML documents). The praxis XML reader doesn't strip BOMs;
    // the USLM 1.0.18 schema bytes carry one.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);

    // Gate: the document must be an XSD schema (root local name =
    // `schema`, root namespace = the W3C XSD 1.1 URI; W3C XSD 1.1
    // Part 1 §3.1.1 — namespace-URI match, not prefix). We use the
    // praxis XML reader as a fail-closed XML well-formedness gate, then
    // verify the root identity through the parsed `XmlElement`.
    let doc = read_xml(text).map_err(|e| DecodeError::Xsd(e.to_string()))?;
    if doc.root.name.local != "schema" {
        return Err(DecodeError::Xsd(format!(
            "root element is `{}`, expected `schema` per W3C XSD 1.1 Part 1 §3.1.2",
            doc.root.name.qualified()
        )));
    }
    // The XML reader filters `xmlns*` declarations out of the attribute
    // list and surfaces only the *first* xmlns declaration in the
    // element's `namespace` field. That's lossy for the multi-xmlns
    // root we always see on a real XSD (the USLM schema declares the
    // USLM target namespace as the default and `xmlns:xsd=…` for the
    // XSD vocabulary). To enforce W3C XSD 1.1 Part 1 §3.1.1 namespace
    // identity we scan the raw root-tag attributes for any xmlns
    // declaration whose value is `XSD_NAMESPACE_URI` and whose prefix
    // matches the root element's prefix (or no prefix when the root
    // declares the default namespace).
    if !root_declares_xsd_namespace(text, doc.root.name.prefix.as_deref()) {
        return Err(DecodeError::Xsd(format!(
            "root element `{}` does not declare the W3C XSD 1.1 namespace `{}` (Part 1 §3.1.1)",
            doc.root.name.qualified(),
            XSD_NAMESPACE_URI,
        )));
    }

    Ok(project_from_xsd_text(text))
}

/// Check whether the document's root opening tag declares the W3C XSD
/// 1.1 namespace URI bound to `root_prefix` (or as the default
/// namespace when `root_prefix` is `None`). Reads the raw XML text to
/// recover the xmlns declarations that the praxis XML reader drops on
/// the floor.
///
/// W3C XML Namespaces 1.0 §2.2 — namespace declarations are attribute
/// values on element start tags; binding is by exact-URI match.
fn root_declares_xsd_namespace(text: &str, root_prefix: Option<&str>) -> bool {
    // Locate the start of the root element by scanning past the XML
    // prolog (declaration, comments, PIs, DOCTYPE). The `XmlDocument`
    // succeeded, so the structure is well-formed — we just need to
    // find the `<` that opens the root tag and read attributes up to
    // its matching `>`.
    let mut cursor = 0;
    while cursor < text.len() {
        let remaining = &text[cursor..];
        if remaining.starts_with("<?") {
            if let Some(end) = remaining.find("?>") {
                cursor += end + 2;
                continue;
            }
            return false;
        }
        if remaining.starts_with("<!--") {
            if let Some(end) = remaining.find("-->") {
                cursor += end + 3;
                continue;
            }
            return false;
        }
        if remaining.starts_with("<!DOCTYPE") {
            // Skip past `>` at depth 0 — DOCTYPE may have nested `[…]`.
            if let Some(end) = remaining.find('>') {
                cursor += end + 1;
                continue;
            }
            return false;
        }
        if remaining.starts_with('<') {
            break;
        }
        // Whitespace between prolog pieces.
        cursor += remaining
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(text.len() - cursor);
    }
    let tag_end = match text[cursor..].find('>') {
        Some(end) => cursor + end,
        None => return false,
    };
    let tag_slice = &text[cursor..=tag_end];

    // The xmlns binding pattern we need to find.
    let needle = match root_prefix {
        Some(p) => format!("xmlns:{p}=\""),
        None => "xmlns=\"".to_string(),
    };
    let Some(idx) = tag_slice.find(needle.as_str()) else {
        return false;
    };
    let value_start = idx + needle.len();
    let Some(value_end_rel) = tag_slice[value_start..].find('"') else {
        return false;
    };
    let value = &tag_slice[value_start..value_start + value_end_rel];
    value == XSD_NAMESPACE_URI
}

/// Decoder errors. Flat shape — callers only need "decode failed" + a
/// reason; the underlying readers' typed errors are stringified.
#[derive(Debug)]
pub enum DecodeError {
    /// The bytes are not valid UTF-8.
    NotUtf8,
    /// The XML/XSD reader rejected the content.
    Xsd(String),
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DecodeError::NotUtf8 => write!(f, "data-provisioning XmlXsd decoder: not valid UTF-8"),
            DecodeError::Xsd(msg) => {
                write!(
                    f,
                    "data-provisioning XmlXsd decoder: xsd read failed: {msg}"
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// The committed USLM-1.0.18 `.prx`. The raw `.xsd` is fetch-only
    /// (`pr4xis update`) and ships in NO crate; the XSD bytes are materialized
    /// from this committed `.prx` through the fail-closed
    /// `[compact_archive_signatures]` gate — the SAME load path the runtime
    /// `formal::meta::xsd::uslm_vocabulary::loaded_uslm_1_0_18_xsd()` uses, so a
    /// clean checkout (no `pr4xis update`) still compiles + runs this test.
    const USLM_XSD_PRX: &[u8] =
        include_bytes!("../../../../data/legal/uscode/schema/uslm-1.0.18.prx");

    #[test]
    fn decoder_round_trips_uslm_xsd_bytes() {
        use crate::applied::data_provisioning::raw_source_prx::raw_source_text_embedded;
        let uslm_xsd = raw_source_text_embedded("uslm_xsd", "1.0.18", USLM_XSD_PRX);
        let instance = decode(uslm_xsd.as_bytes())
            .expect("the USLM XSD bytes must decode through the dispatcher");
        // Sanity: the USLM XSD declares > 100 `<xs:element>` declarations.
        // Don't reassert exact counts here — the load-bearing exact-count
        // assertions live with the XSD ontology tests under
        // `formal::meta::xsd`; this test only proves the decoder dispatch
        // wiring works.
        assert!(instance.elements.len() > 50);
    }

    #[test]
    fn decoder_rejects_invalid_utf8() {
        let err = decode(&[0xFF, 0xFE, 0xFD]).expect_err("invalid UTF-8 must be rejected");
        match err {
            DecodeError::NotUtf8 => {}
            other => panic!("expected NotUtf8, got {other:?}"),
        }
    }

    #[test]
    fn decoder_rejects_non_schema_xml() {
        let bytes = br#"<?xml version="1.0"?><not-schema xmlns="http://example.com"/>"#;
        let err = decode(bytes).expect_err("non-schema XML must be rejected");
        match err {
            DecodeError::Xsd(_) => {}
            other => panic!("expected Xsd error, got {other:?}"),
        }
    }

    #[test]
    fn decoder_rejects_schema_with_wrong_namespace() {
        // Root is `schema` but the declared namespace is not the W3C
        // XSD 1.1 URI — must be rejected per Part 1 §3.1.1.
        let bytes = br#"<?xml version="1.0"?><schema xmlns="http://example.com/not-xsd"></schema>"#;
        let err = decode(bytes).expect_err("wrong-namespace schema must be rejected");
        match err {
            DecodeError::Xsd(_) => {}
            other => panic!("expected Xsd error, got {other:?}"),
        }
    }
}
