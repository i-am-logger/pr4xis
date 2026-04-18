//! `XmlElementAttribute` extractor — real implementation.
//!
//! The `SelfDescribingMetadata::XmlElementAttribute` leaf. Parses XML bytes
//! using pr4xis's existing XML reader and confirms a named element has an
//! attribute matching the declared expected value.
//!
//! WordNet LMF uses this: `<Lexicon id="..." version="2025" ...>`.
//!
//! This extractor **reuses the existing XML pipeline** at
//! `crates/domains/src/social/software/markup/xml/reader.rs`. No new XML
//! parsing; we delegate to the ontology-grounded parser that the rest of the
//! workspace already depends on.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::super::ontology::{ClaimData, IdentityClaim, VerificationResult};
use crate::social::software::markup::xml::reader as xml_reader;

/// Verify an `XmlElementAttribute` claim against raw bytes.
///
/// Expects `claim.data` to be `ClaimData::XmlAttribute { element, attribute,
/// expected }`. Parses the bytes as XML via `xml_reader::read_xml`, searches
/// for the first element matching `element`, reads its `attribute`, and
/// compares to `expected`.
pub fn verify(claim: &IdentityClaim, bytes: &[u8]) -> VerificationResult {
    let (element_name, attribute_name, expected) = match &claim.data {
        ClaimData::XmlAttribute {
            element,
            attribute,
            expected,
        } => (*element, *attribute, expected.clone()),
        _ => {
            return VerificationResult::Unverifiable {
                reason: "XmlElementAttribute extractor expected XmlAttribute ClaimData".into(),
            };
        }
    };

    // Decode bytes as UTF-8 text so the existing xml_reader can parse it.
    let text = match core::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(_) => {
            return VerificationResult::Unverifiable {
                reason: "artifact bytes are not valid UTF-8 XML".into(),
            };
        }
    };

    let doc = match xml_reader::read_xml(text) {
        Ok(d) => d,
        Err(e) => {
            return VerificationResult::Unverifiable {
                reason: format!("xml parse failed: {}", e.message),
            };
        }
    };

    // Walk every element in the parsed tree looking for the first element
    // whose name matches `element_name`. `find_all` returns a Vec<&XmlElement>
    // (through its existing API); we iterate that vec.
    let matching = doc.find_all(element_name);
    let first = match matching.first() {
        Some(e) => e,
        None => {
            return VerificationResult::Unverifiable {
                reason: format!("element <{}> not found in xml", element_name),
            };
        }
    };

    let actual = first
        .attributes
        .iter()
        .find(|a| a.name.local == attribute_name)
        .map(|a| a.value.clone());

    match actual {
        Some(value) if value == expected => VerificationResult::Verified(claim.clone()),
        Some(value) => VerificationResult::Mismatch {
            expected,
            actual: value,
        },
        None => VerificationResult::Unverifiable {
            reason: format!(
                "element <{}> has no `{}` attribute",
                element_name, attribute_name
            ),
        },
    }
}
