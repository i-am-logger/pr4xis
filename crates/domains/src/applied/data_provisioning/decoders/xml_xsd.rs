//! XML XSD decoder — wraps the `xml::xsd::reader` pipeline.
//!
//! This decoder is the `ContentType::XmlXsd` entry in the decoder
//! dispatch table. It takes raw bytes (the fetched XSD file contents),
//! decodes them through the existing XML pipeline, and returns a typed
//! [`XsdSchema`](crate::social::software::markup::xml::xsd::XsdSchema)
//! instance. The XSD AST is the substrate the USLM ontology codegen will
//! consume to derive its Rust types from the published schema rather than
//! by hand.
//!
//! **This decoder does not reimplement any XML parsing.** It delegates to:
//!
//! - `crates/domains/src/social/software/markup/xml/reader.rs` (XML parse)
//! - `crates/domains/src/social/software/markup/xml/xsd/reader.rs` (XSD
//!   schema-component extraction on top of the parsed XML)

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::social::software::markup::xml::xsd::ontology::XsdSchema;
use crate::social::software::markup::xml::xsd::reader as xsd_reader;

/// Decode raw bytes as a W3C XSD 1.1 schema document.
///
/// Expects UTF-8 XML text as bytes. Delegates to
/// [`xsd_reader::read_xsd`], which itself delegates to the praxis XML
/// reader — both existing functors in this workspace.
///
/// # Errors
///
/// Returns a [`DecodeError`] if the bytes are not valid UTF-8 or if the
/// XSD reader rejects the document (root not `xsd:schema`, unsupported
/// construct, etc.).
pub fn decode(bytes: &[u8]) -> Result<XsdSchema, DecodeError> {
    let text = core::str::from_utf8(bytes).map_err(|_| DecodeError::NotUtf8)?;
    xsd_reader::read_xsd(text).map_err(|e| DecodeError::Xsd(e.to_string()))
}

/// Decoder errors. Flat shape — callers only need "decode failed" + a
/// reason; the underlying XSD reader's typed errors are stringified.
#[derive(Debug)]
pub enum DecodeError {
    /// The bytes are not valid UTF-8.
    NotUtf8,
    /// The XSD reader rejected the content.
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

    const USLM_XSD: &str = include_str!("../../../../data/legal/uscode/schema/uslm-1.0.18.xsd");

    #[test]
    fn decoder_round_trips_uslm_xsd_bytes() {
        let schema = decode(USLM_XSD.as_bytes())
            .expect("the USLM XSD bytes must decode through the dispatcher");
        // Sanity: the USLM XSD declares > 100 elements. Don't reassert
        // the exact counts here — the load-bearing exact-count
        // assertions live in `xml::xsd::tests::loads_uslm_full_xsd`;
        // this test only proves the decoder dispatch wiring works.
        assert!(schema.elements.len() > 50);
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
}
