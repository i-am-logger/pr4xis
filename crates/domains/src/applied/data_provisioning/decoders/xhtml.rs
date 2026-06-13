//! XHTML decoder — wraps the praxis XML reader to accept W3C-published
//! XHTML 1.0 Transitional text-form recommendations.
//!
//! This decoder is the `ContentType::Xhtml` entry in the decoder
//! dispatch table. It takes raw bytes (the fetched XHTML file
//! contents), decodes them through the existing XML pipeline, and
//! returns a typed [`XmlDocument`] instance. The downstream M4.η.2
//! XML 1.0 ontology codegen consumes this through a per-section
//! text-scan, so the decoder's job is bounded to "parse as XML, hand
//! back a typed tree".
//!
//! Cited:
//! - **Cowan & Tobin (eds.) (2004)** *XML Information Set (Second
//!   Edition)*, W3C Recommendation 4 February 2004 — the published
//!   recommendation is delivered as XHTML 1.0 Transitional.
//! - **Pemberton et al. (eds.) (2002)** *XHTML 1.0: The Extensible
//!   HyperText Markup Language (Second Edition)*, W3C Recommendation
//!   1 August 2002 — the publication format.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::social::software::markup::xml::ontology::XmlDocument;
use crate::social::software::markup::xml::reader as xml_reader;

/// Decode raw bytes as a W3C-published XHTML 1.0 document.
///
/// Expects UTF-8 XHTML text as bytes. Delegates to
/// [`xml_reader::read_xml`] — the same XML reader used for every
/// other XML-based ContentType.
///
/// # Errors
///
/// Returns a [`DecodeError`] if the bytes are not valid UTF-8 or if
/// the XML reader rejects the document.
/// The [`ContentType`](crate::applied::data_provisioning::ontology::ContentType)
/// this module realizes -- the single declaration of which content type
/// this file decodes, read by `super::has_decoder_for` (audit 2026-06-12 D-22).
pub const DECODES: crate::applied::data_provisioning::ontology::ContentType =
    crate::applied::data_provisioning::ontology::ContentType::Xhtml;

pub fn decode(bytes: &[u8]) -> Result<XmlDocument, DecodeError> {
    let text = core::str::from_utf8(bytes).map_err(|_| DecodeError::NotUtf8)?;
    xml_reader::read_xml(text).map_err(|e| DecodeError::Xml(e.to_string()))
}

/// Decoder errors.
#[derive(Debug)]
pub enum DecodeError {
    /// The bytes are not valid UTF-8.
    NotUtf8,
    /// The XML reader rejected the content.
    Xml(String),
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DecodeError::NotUtf8 => {
                write!(f, "data-provisioning Xhtml decoder: not valid UTF-8")
            }
            DecodeError::Xml(msg) => {
                write!(f, "data-provisioning Xhtml decoder: xml read failed: {msg}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bundled W3C XML Information Set rec (XHTML 1.0
    /// Transitional). Decodes through the praxis XML reader.
    const XML_INFOSET: &str = include_str!("../../../../data/markup-schemas/xml/xml-infoset.xhtml");

    #[test]
    fn decoder_round_trips_xml_infoset_bytes() {
        let doc = decode(XML_INFOSET.as_bytes())
            .expect("the XML Information Set XHTML bytes must decode through the dispatcher");
        // Sanity: the W3C XHTML root element is `<html>` with the
        // XHTML 1.0 namespace.
        assert_eq!(doc.root.name.local, "html");
    }

    #[test]
    fn decoder_rejects_invalid_utf8() {
        let bad_bytes = [0xff, 0xfe, 0xfd, 0xfc];
        let result = decode(&bad_bytes);
        assert!(matches!(result, Err(DecodeError::NotUtf8)));
    }
}
