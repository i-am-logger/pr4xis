//! OWL decoder — raw bytes → loaded [`OwlOntology`].
//!
//! This decoder is the `ContentType::Owl` entry in the decoder dispatch
//! table. It takes raw bytes (the fetched OWL / RDF-XML vocabulary
//! contents) and returns the typed
//! [`OwlOntology`]
//! — the praxis OWL ontology applied to the source text.
//!
//! **This decoder does not reimplement any XML or OWL parsing.** It
//! delegates to
//! [`crate::social::software::markup::xml::owl::reader::read_owl`], the
//! praxis OWL reader, which is built on the praxis XML 1.0 substrate.
//! Per the W3C OWL 2 Web Ontology Language Structural Specification
//! (Motik, Patel-Schneider & Parsia eds., W3C Recommendation 11 December
//! 2012), an OWL ontology serialised as RDF/XML (Gandon & Schreiber
//! eds., RDF 1.1 XML Syntax, W3C Recommendation 25 February 2014)
//! declares named classes, object properties, and their
//! `rdfs:subClassOf` / `rdfs:subPropertyOf` hierarchies.
//!
//! This runtime decoder is the read-side dual of the (separate, later)
//! OWL → Rust codegen pipeline — it doesn't generate Rust code, it
//! returns the ontology instance downstream consumers (e.g. the
//! CitationQuality ontology grounding from CiTO) dispatch on.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::social::software::markup::xml::owl::ontology::OwlOntology;
use crate::social::software::markup::xml::owl::reader::read_owl;

/// Decode raw bytes as a W3C OWL 2 / RDF-XML vocabulary.
///
/// Expects UTF-8 XML text as bytes. The decode pipeline is:
///
/// 1. Validate UTF-8 → [`DecodeError::NotUtf8`] on failure.
/// 2. Strip an optional leading UTF-8 byte-order mark (W3C XML 1.0
///    §F.1).
/// 3. Project through the praxis OWL reader
///    [`read_owl`] into an [`OwlOntology`] → [`DecodeError::Owl`] on
///    failure.
///
/// # Errors
///
/// Returns a [`DecodeError`] if the bytes are not valid UTF-8 or the OWL
/// reader rejects the document as malformed.
/// The [`ContentType`](crate::applied::data_provisioning::ontology::ContentType)
/// this module realizes -- the single declaration of which content type
/// this file decodes, read by `super::has_decoder_for` (audit 2026-06-12 D-22).
pub const DECODES: crate::applied::data_provisioning::ontology::ContentType =
    crate::applied::data_provisioning::ontology::ContentType::Owl;

pub fn decode(bytes: &[u8]) -> Result<OwlOntology, DecodeError> {
    let text = core::str::from_utf8(bytes).map_err(|_| DecodeError::NotUtf8)?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    read_owl(text).map_err(|e| DecodeError::Owl(e.to_string()))
}

/// Decoder errors. Flat shape — callers only need "decode failed" + a
/// reason; the underlying reader's typed error is stringified.
#[derive(Debug)]
pub enum DecodeError {
    /// The bytes are not valid UTF-8.
    NotUtf8,
    /// The OWL reader rejected the content.
    Owl(String),
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DecodeError::NotUtf8 => write!(f, "data-provisioning Owl decoder: not valid UTF-8"),
            DecodeError::Owl(msg) => {
                write!(f, "data-provisioning Owl decoder: owl read failed: {msg}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    const CITO_OWL: &str = include_str!("../../../../data/ontologies/cito-2.8.1.owl");

    #[test]
    fn decoder_round_trips_cito_bytes() {
        let ont = decode(CITO_OWL.as_bytes())
            .expect("the bundled CiTO OWL bytes must decode through the dispatcher");
        // Sanity: CiTO declares > 30 object properties (cito:cites +
        // its sub-properties and their cito:isCitedBy inverses). The
        // load-bearing exact-shape assertions live with the owl tests;
        // this test only proves the decoder dispatch wiring works.
        assert!(ont.property_count() > 30);
    }

    #[test]
    fn decoder_rejects_invalid_utf8() {
        let err = decode(&[0xFF, 0xFE, 0xFD]).expect_err("invalid UTF-8 must be rejected");
        match err {
            DecodeError::NotUtf8 => {}
            other => panic!("expected NotUtf8, got {other:?}"),
        }
    }
}
