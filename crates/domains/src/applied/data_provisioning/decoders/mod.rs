//! Decoder functors — one per `ContentType`.
//!
//! Each decoder is a typed transformation `RawBytes → DomainOntology` for
//! a specific content type. The fetch pipeline is uniform; the decoders
//! are content-specific. Adding a new content type = adding one new file
//! here that declares a decoder, plus a new `ContentType` variant in
//! `ontology.rs`.
//!
//! The initial PR wires only `XmlLmf` (for WordNet), which reuses the
//! existing `xml_reader::read_xml → lmf::reader::read_wordnet` pipeline
//! and can be composed with `English::from_wordnet` by callers that want
//! a fully-loaded English ontology.

use super::ontology::ContentType;

pub mod xml_lmf;

/// Does a decoder exist for this content type? Used by the
/// `DecoderTotalityPerContentType` axiom.
pub fn has_decoder_for(content_type: ContentType) -> bool {
    matches!(content_type, ContentType::XmlLmf)
}
