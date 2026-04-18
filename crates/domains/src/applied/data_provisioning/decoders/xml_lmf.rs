//! XML LMF decoder — wraps the existing `xml_reader → lmf::reader` pipeline.
//!
//! This decoder is the `ContentType::XmlLmf` entry in the decoder dispatch
//! table. It takes raw bytes (the fetched file contents), decodes them
//! through the existing XML pipeline, and returns a typed `WordNet`
//! instance. Callers that want a fully-loaded English ontology then feed
//! the `WordNet` through `English::from_wordnet`.
//!
//! **This decoder does not reimplement any XML parsing.** It delegates to:
//!
//! - `crates/domains/src/social/software/markup/xml/reader.rs`
//! - `crates/domains/src/social/software/markup/xml/lmf/reader.rs`
//!
//! The composition is explicit here so the data-provisioning layer is a
//! thin wrapper, not a second implementation of functionality that already
//! exists in `social/software/markup/xml/`.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::social::software::markup::xml::lmf::ontology::WordNet;
use crate::social::software::markup::xml::lmf::reader as lmf_reader;

/// Decode raw bytes as a WordNet LMF XML document.
///
/// Expects UTF-8 XML text as bytes. Delegates to the existing
/// `lmf::reader::read_wordnet` which itself delegates to
/// `xml_reader::read_xml` — both existing functors in the workspace.
///
/// # Errors
///
/// Returns a `DecodeError` if the bytes are not valid UTF-8 or if the XML
/// cannot be parsed as LMF.
///
/// # Example
///
/// ```
/// use pr4xis_domains::applied::data_provisioning::decoders::xml_lmf;
///
/// let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
/// <LexicalResource>
///   <Lexicon id="oewn" label="English WordNet" language="en" email="t@e" license="CC" version="2025" url="https://en-word.net/">
///     <LexicalEntry id="e1"><Lemma writtenForm="dog" partOfSpeech="n"/><Sense id="s1" synset="d1"/></LexicalEntry>
///     <Synset id="d1" ili="i1" partOfSpeech="n"><Definition>a dog</Definition></Synset>
///   </Lexicon>
/// </LexicalResource>"#;
///
/// let wordnet = xml_lmf::decode(xml).expect("valid LMF");
/// assert_eq!(wordnet.entries.len(), 1);
/// assert_eq!(wordnet.synsets.len(), 1);
/// ```
pub fn decode(bytes: &[u8]) -> Result<WordNet, DecodeError> {
    let text = core::str::from_utf8(bytes).map_err(|_| DecodeError::NotUtf8)?;
    lmf_reader::read_wordnet(text).map_err(|e| DecodeError::Lmf(e.to_string()))
}

/// Decoder errors. Minimal for now — the existing XML/LMF readers have
/// their own rich error types; we just flatten them to strings here
/// because callers only need to know "decode failed" + a reason.
#[derive(Debug)]
pub enum DecodeError {
    /// The bytes are not valid UTF-8.
    NotUtf8,
    /// The LMF reader (which wraps the XML reader) rejected the content.
    Lmf(String),
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DecodeError::NotUtf8 => write!(f, "data-provisioning XmlLmf decoder: not valid UTF-8"),
            DecodeError::Lmf(msg) => {
                write!(
                    f,
                    "data-provisioning XmlLmf decoder: lmf read failed: {msg}"
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}
