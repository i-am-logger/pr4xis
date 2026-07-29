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

pub mod adobe_glyph_list;
pub mod file_collection;
pub mod owl;
pub mod plaintext_tsv;
pub mod propbank_frameset_collection;
pub mod tar_gz_archive;
pub mod theme_collection;
pub mod verbnet_class_collection;
pub mod xhtml;
pub mod xml_dtd;
pub mod xml_lmf;
pub mod xml_xsd;
pub mod zip_archive;

#[cfg(test)]
mod totality_proptests;

/// Does a decoder exist for this content type? Used by the
/// `DecoderTotalityPerKind` axiom.
///
/// EXHAUSTIVE — no wildcard arm (audit 2026-06-12 D-22): adding a `ContentType`
/// variant is a COMPILE ERROR here until its decoder status is decided, which a
/// `matches!` (treating every unlisted variant as silently `false`) cannot give.
/// Each `true` arm is owned by a decoder module's `DECODES` const — the single
/// declaration of which content type that file realizes (verified by
/// `has_decoder_for_agrees_with_each_module_const`). `UslmXml` is the one decoder
/// realized outside this directory (`xml::uslm::lens::read_uslm_title`), so it has
/// no module const to hang on — the genuine reason this is exhaustive-but-still-
/// enumerated rather than a collected set.
pub fn has_decoder_for(content_type: ContentType) -> bool {
    use ContentType as CT;
    match content_type {
        CT::XmlLmf => true, // xml_lmf::DECODES
        // Closed-class function-word / legal lexica are WN-LMF-shaped, so the
        // SAME `xml_lmf` reader (`lmf::reader::read_wordnet`) decodes their
        // materialized raw bytes. One reader, two content types — no dedicated
        // module/const, so it has no `DECODES` to hang on.
        CT::XmlLmfLexicon => true,
        CT::Owl => true,                        // owl::DECODES
        CT::XmlXsd => true,                     // xml_xsd::DECODES
        CT::Xhtml => true,                      // xhtml::DECODES
        CT::AdobeGlyphList => true,             // adobe_glyph_list::DECODES
        CT::XmlDtd => true,                     // xml_dtd::DECODES
        CT::TarGzArchive => true,               // tar_gz_archive::DECODES
        CT::ZipArchive => true,                 // zip_archive::DECODES
        CT::Plaintext => true,                  // plaintext_tsv::DECODES
        CT::ThemeCollection => true,            // theme_collection::DECODES
        CT::VerbNetClassCollection => true,     // verbnet_class_collection::DECODES
        CT::PropBankFramesetCollection => true, // propbank_frameset_collection::DECODES
        // Realized outside decoders/ — by xml::uslm::lens::read_uslm_title.
        CT::UslmXml => true,
        // The math-operator vocabulary is WN-LMF-shaped, so the SAME
        // `xml_lmf` reader (`lmf::reader::read_wordnet`) decodes its
        // materialized raw bytes — it just projects to an `OperatorVocabulary`
        // downstream instead of an `English`. No dedicated decoder module/const
        // (one reader, two consumers), so it has no `DECODES` to hang on.
        CT::MathOperatorLmf => true,
        // No decoder yet — fail-closed for DecoderTotalityPerKind.
        CT::Pdf | CT::Json | CT::Video | CT::Audio | CT::Binary => false,
    }
}
