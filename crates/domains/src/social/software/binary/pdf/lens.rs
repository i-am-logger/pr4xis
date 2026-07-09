//! `PdfLens` — the [`WellBehavedLens`] binding the praxis PDF
//! reader's byte hop into the round-trip harness. Parallel to
//! [`crate::formal::meta::xsd::lens::XsdSchemaLens`] (for XSD) and
//! [`crate::formal::meta::dtd::lens::DtdLens`] (for DTD).
//!
//! Per ISO 32000-2:2020 §7.5, a PDF document is a stream of indirect
//! objects, a cross-reference table, and a trailer. This lens composes
//! [`super::reader::read_pdf_bytes`] (the indirect-object decoder) with
//! [`super::extract::extract_document`] (the §7.8 content-stream → §9
//! text-show event pipeline) to read raw PDF bytes into a
//! [`PdfExtraction`] = `(extraction, complement)` pair. Per Bancilhon
//! & Spyratos 1981 Theorem 3 the complement carrying the original
//! bytes makes put-get hold byte-canonically — same shape as
//! [`crate::social::software::markup::xml::uslm::UslmXmlLens`].
//!
//! ## Scope
//!
//! Text-only extraction. Per
//! `feedback_pdf_text_only_until_image_understanding`, image
//! content surfaces as `FlaggedContent` items inside the
//! [`super::extract::ExtractedDocument`] view (never silently
//! dropped or paraphrased); the actual image bytes await the
//! Phase 6 image-understanding seam.
//!
//! ## Citation
//!
//! - **ISO 32000-2:2020** *Document management — Portable Document
//!   Format — Part 2: PDF 2.0* — the byte-format substrate this lens
//!   reads.
//! - **Foster, Greenwald, Moore, Pierce & Schmitt (2007)** "Combinators
//!   for Bidirectional Tree Transformations", *ACM TOPLAS* 29(3) §3, Definition 3.2
//!   — the well-behaved-lens laws.
//! - **Bancilhon & Spyratos (1981)** "Update Semantics of Relational
//!   Views", *ACM TODS* 6(4) Theorem 3 — constant-complement view
//!   update.

#[allow(unused_imports)]
use alloc::{format, string::String, vec::Vec};
use core::fmt;

use super::extract::{ExtractError, ExtractedDocument, extract_document};
use super::reader::{PdfReadError, read_pdf_bytes};
use crate::formal::meta::well_behaved_lens::WellBehavedLens;

/// The byte-anchored view of a PDF document — the typed
/// [`ExtractedDocument`] (text per page + flagged non-text
/// content) plus the original bytes as the constant complement
/// (Bancilhon & Spyratos 1981 Theorem 3).
#[derive(Debug, Clone, PartialEq)]
pub struct PdfExtraction {
    /// The decoded text + flagged content for every page (ISO
    /// 32000-2:2020 §7.8 content-stream → §9 text-show event
    /// pipeline).
    pub extraction: ExtractedDocument,
    /// The complement: the original source bytes from which
    /// `extraction` was derived. Per Bancilhon & Spyratos 1981
    /// Theorem 3, holding the complement constant recovers the
    /// source verbatim on put-without-modification.
    pub complement: Vec<u8>,
}

impl PdfExtraction {
    /// Convenience: concatenate every decoded text chunk across
    /// every page into a single `String`. Flagged content is
    /// reachable via `self.extraction.pages[i].flagged`.
    pub fn full_text(&self) -> String {
        self.extraction.full_text()
    }
}

/// Error of [`PdfLens::get`] / [`PdfLens::put`].
#[derive(Debug, Clone)]
pub enum PdfLensError {
    /// The PDF byte-format parser rejected the bytes (header,
    /// trailer, cross-reference table, or object decoding failure
    /// per ISO 32000-2:2020 §7.5).
    Read(String),
    /// The content-stream extractor rejected the parsed document
    /// (page-tree or content-stream malformation per §7.7 / §7.8).
    Extract(String),
}

impl fmt::Display for PdfLensError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfLensError::Read(m) => write!(f, "pdf lens: byte-format read failed: {m}"),
            PdfLensError::Extract(m) => write!(f, "pdf lens: extraction failed: {m}"),
        }
    }
}

impl From<PdfReadError> for PdfLensError {
    fn from(e: PdfReadError) -> Self {
        PdfLensError::Read(format!("{e}"))
    }
}

impl From<ExtractError> for PdfLensError {
    fn from(e: ExtractError) -> Self {
        PdfLensError::Extract(format!("{e}"))
    }
}

/// The byte-anchored [`WellBehavedLens`] binding `bytes ⇆ PdfExtraction`.
///
/// `get(bytes)` runs the PDF byte-format parser ([`read_pdf_bytes`])
/// followed by the content-stream extractor ([`extract_document`]),
/// retaining the original bytes as the complement. `put(target)`
/// returns the complement — constant-complement PutGet (Bancilhon
/// & Spyratos 1981 Theorem 3). `canonical` returns the bytes
/// unchanged: ISO 32000-2 does not publish a canonical form for
/// PDF, so round-trip identity = byte equality on the source bytes.
pub struct PdfLens;

impl WellBehavedLens for PdfLens {
    type Target = PdfExtraction;
    type Error = PdfLensError;

    fn get(bytes: &[u8]) -> Result<Self::Target, Self::Error> {
        let doc = read_pdf_bytes(bytes)?;
        let extraction = extract_document(&doc)?;
        Ok(PdfExtraction {
            extraction,
            complement: bytes.to_vec(),
        })
    }

    fn put(target: &Self::Target) -> Result<Vec<u8>, Self::Error> {
        Ok(target.complement.clone())
    }

    fn canonical(bytes: &[u8]) -> Result<Vec<u8>, Self::Error> {
        // ISO 32000-2 does not publish a canonical form for PDF;
        // round-trip identity is therefore byte-equality on the
        // source bytes themselves (same convention as
        // [`crate::formal::meta::dtd::lens::DtdLens`] for DTDs).
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hermetic single-page PDF carrying a text content stream, built
    /// programmatically via `lopdf` (correct xref offsets) — the same
    /// approach as `super::super::reader`'s `minimal_pdf` fixture. It
    /// exercises the read + extract pipeline end-to-end with NO committed
    /// binary: praxis ships no PDFs in its crates, and real statute
    /// corpora are fetched + compiled to `.prx` in CI, not embedded.
    fn sample_pdf_with_text() -> Vec<u8> {
        use lopdf::{Document, Object, Stream, dictionary};
        let mut doc = Document::with_version("1.4");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });
        let content_id = doc.add_object(Stream::new(
            dictionary! {},
            b"BT /F1 24 Tf 72 720 Td (Praxis PDF lens fixture) Tj ET".to_vec(),
        ));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
            "Resources" => dictionary! { "Font" => dictionary! { "F1" => font_id } },
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => vec![page_id.into()],
                "Count" => 1,
            }),
        );
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).expect("serialize sample pdf");
        bytes
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn get_then_put_returns_original_bytes() {
        let bytes = sample_pdf_with_text();
        let target = <PdfLens as WellBehavedLens>::get(&bytes).expect("parse + extract");
        let back = <PdfLens as WellBehavedLens>::put(&target).expect("put");
        assert_eq!(back, bytes);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn get_extracts_some_text() {
        let bytes = sample_pdf_with_text();
        let target = <PdfLens as WellBehavedLens>::get(&bytes).expect("parse + extract");
        // The fixture's content stream shows "Praxis PDF lens fixture";
        // any text-shaped output proves the content-stream pipeline ran.
        let text = target.full_text();
        assert!(
            !text.trim().is_empty(),
            "extracted PDF text was empty — content-stream pipeline failed"
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn get_emits_one_page_text_per_page() {
        let bytes = sample_pdf_with_text();
        let target = <PdfLens as WellBehavedLens>::get(&bytes).expect("parse + extract");
        // The extractor produces one PageText per page; the fixture has
        // one page, so the count is positive.
        assert!(!target.extraction.pages.is_empty());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn put_get_law_holds() {
        let bytes = sample_pdf_with_text();
        assert!(<PdfLens as WellBehavedLens>::assert_put_get_law(&bytes).is_ok());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn rejects_non_pdf_bytes() {
        let bytes = b"not a pdf".to_vec();
        assert!(matches!(
            <PdfLens as WellBehavedLens>::get(&bytes),
            Err(PdfLensError::Read(_))
        ));
    }

    proptest::proptest! {
        /// Robustness: for arbitrary byte streams, `get` either
        /// returns a `PdfExtraction` (ISO 32000-2 §7.5 parse +
        /// §7.8 + §9 extract succeed) or a typed [`PdfLensError`].
        /// Never panics on malformed input — the byte-format parser
        /// fails closed via the typed error surface.
        #[test]
        fn prop_get_never_panics_on_arbitrary_bytes(
            bytes in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..256)
        ) {
            let _ = <PdfLens as WellBehavedLens>::get(&bytes);
        }
    }

    pr4xis::register_praxis_value!(prop_get_never_panics_on_arbitrary_bytes, Honest);
}
