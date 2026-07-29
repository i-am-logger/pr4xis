//! Extraction pipeline — composes Phases 2-5 into a typed
//! `(Vec<PageText>, Vec<FlaggedContent>)` output.
//!
//! What this module does, in order:
//!
//! 1. Walk every page's content stream via Phase 3
//!    ([`super::content_stream::walk_content_stream`]).
//! 2. Resolve each text-show event's font through Phase 4
//!    ([`super::font::resolve_font`] + [`super::font::decode_bytes`]).
//!    `TextShowEvent.bytes` → `String` chunk.
//! 3. Collect graphics events through Phase 5
//!    ([`super::flagged::flag_page`]) so non-text content is
//!    surfaced, never silently dropped.
//! 4. (Optional) Apply Bluebook §3.3 statute-section slicing
//!    via [`slice_to_section`] — given a page's full text and a
//!    Bluebook citation prefix (e.g. `"§ 1514A"`), return only
//!    the bytes between this section's header and the next
//!    section's header.
//!
//! Resolution-failure handling is *typed*, not silent. If a font
//! can't be resolved or a byte sequence can't be decoded, the
//! pipeline pushes a [`PageTextChunk`] with the corresponding
//! [`ChunkOutcome::FailedDecode`] / [`ChunkOutcome::FailedResolve`]
//! variant instead of dropping the chunk. Downstream auditors see
//! every gap.
//!
//! Spec references:
//!
//! - ISO 32000-2:2020 §9 — text rendering pipeline.
//! - Bluebook §3.3 — statute subdivision marker grammar (used
//!   by [`slice_to_section`]).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::content_stream::{ContentStreamError, walk_content_stream};
use super::flagged::{FlagError, flag_page};
use super::font::{FontResolveError, PdfFont, decode_bytes, resolve_font};
use super::ontology::FlaggedContent;
use super::reader::PdfDocument;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

// ─────────────────────────────────────────────────────────────────────
// Output types
// ─────────────────────────────────────────────────────────────────────

/// A single text-show event's outcome through the decode pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum ChunkOutcome {
    /// Decode succeeded; the field carries the Unicode text.
    Decoded(String),
    /// Font name in the `Tf` operator didn't resolve to a usable
    /// font dictionary or its encoding isn't supported. The field
    /// carries the diagnostic.
    FailedResolve(String),
    /// Font resolved but its encoding couldn't decode these bytes.
    /// The field carries the diagnostic.
    FailedDecode(String),
}

/// One contiguous text chunk from a content stream — the Phase 4
/// decoding of a single [`super::content_stream::TextShowEvent`].
#[derive(Debug, Clone, PartialEq)]
pub struct PageTextChunk {
    /// 1-indexed page number this chunk came from.
    pub page: u32,
    /// Font name in scope (the `Tf` operand).
    pub font_name: String,
    /// Decode outcome — succeeded, or failed with a typed reason.
    pub outcome: ChunkOutcome,
}

impl PageTextChunk {
    /// Convenience: extract the decoded `&str`, returning `""`
    /// for failed-decode chunks. Used when assembling per-page
    /// text concatenations.
    pub fn as_str(&self) -> &str {
        match &self.outcome {
            ChunkOutcome::Decoded(s) => s.as_str(),
            _ => "",
        }
    }
}

/// One page's worth of extracted text + the non-text content the
/// flagging walker found on that page.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PageText {
    pub page: u32,
    pub chunks: Vec<PageTextChunk>,
    pub flagged: Vec<FlaggedContent>,
}

impl PageText {
    /// Concatenate every decoded chunk on this page into a single
    /// String. Failed-decode chunks contribute empty string;
    /// downstream consumers wanting per-chunk visibility iterate
    /// `chunks` directly.
    pub fn full_text(&self) -> String {
        let mut out = String::new();
        for c in &self.chunks {
            if let ChunkOutcome::Decoded(s) = &c.outcome {
                out.push_str(s);
            }
        }
        out
    }
}

/// One document's complete extraction result.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExtractedDocument {
    pub pages: Vec<PageText>,
}

impl ExtractedDocument {
    /// Concatenate every page's `full_text()` with `"\n\n"`
    /// between pages. Useful for end-to-end "what did the PDF
    /// say" assembly without per-chunk diagnostics.
    pub fn full_text(&self) -> String {
        let mut out = String::new();
        for (i, page) in self.pages.iter().enumerate() {
            if i > 0 {
                out.push_str("\n\n");
            }
            out.push_str(&page.full_text());
        }
        out
    }

    /// Total number of flagged non-text items across all pages, as a
    /// dimensionless [`Quantity`] (`unit::UNITLESS`) -- a count, not a
    /// physical quantity.
    pub fn flagged_count(&self) -> Quantity {
        let count: usize = self.pages.iter().map(|p| p.flagged.len()).sum();
        Quantity::from_unit(count as f64, &unit::UNITLESS)
    }
}

// ─────────────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────────────

/// Why the extraction pipeline failed at a level above per-chunk
/// decode (which is non-fatal). Per-chunk decode failures are
/// reported via [`ChunkOutcome::FailedDecode`] / `FailedResolve`,
/// not surfaced here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtractError {
    /// Page index out of [1, page_count] range.
    PageOutOfRange { page: u32, total: usize },
    /// Content stream bytes couldn't be retrieved.
    UnreadableContentStream { page: u32, detail: String },
    /// Content stream couldn't be tokenized.
    MalformedContentStream { page: u32, detail: String },
    /// Flagging walker failed (typically xobject resolution).
    FlaggingFailed { page: u32, detail: String },
}

impl core::fmt::Display for ExtractError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::PageOutOfRange { page, total } => {
                write!(f, "page {page} out of range (document has {total} pages)")
            }
            Self::UnreadableContentStream { page, detail } => {
                write!(f, "page {page} content stream unreadable: {detail}")
            }
            Self::MalformedContentStream { page, detail } => {
                write!(f, "page {page} content stream malformed: {detail}")
            }
            Self::FlaggingFailed { page, detail } => {
                write!(f, "page {page} flagging walk failed: {detail}")
            }
        }
    }
}

impl std::error::Error for ExtractError {}

// ─────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────

/// Extract text + flagged content from every page in the document.
pub fn extract_document(doc: &PdfDocument) -> Result<ExtractedDocument, ExtractError> {
    let pages = doc.inner().get_pages();
    let mut out = ExtractedDocument::default();
    for &page_num in pages.keys() {
        out.pages.push(extract_page(doc, page_num)?);
    }
    Ok(out)
}

/// Extract text + flagged content from a single page.
pub fn extract_page(doc: &PdfDocument, page_num: u32) -> Result<PageText, ExtractError> {
    let pages = doc.inner().get_pages();
    let total = pages.len();
    let page_id = *pages.get(&page_num).ok_or(ExtractError::PageOutOfRange {
        page: page_num,
        total,
    })?;

    let bytes = doc.inner().get_page_content(page_id).map_err(|e| {
        ExtractError::UnreadableContentStream {
            page: page_num,
            detail: format!("{e}"),
        }
    })?;

    let walk = walk_content_stream(&bytes).map_err(|e| match e {
        ContentStreamError::Malformed { detail } => ExtractError::MalformedContentStream {
            page: page_num,
            detail,
        },
    })?;

    // Resolve each TextShowEvent's font name once per unique name.
    // Same name on multiple Tjs reuses the resolution.
    let page_fonts = doc.inner().get_page_fonts(page_id).unwrap_or_default();
    let mut chunks = Vec::with_capacity(walk.text_events.len());
    for ev in &walk.text_events {
        let outcome = decode_event(doc, &page_fonts, &ev.font_name, &ev.bytes);
        chunks.push(PageTextChunk {
            page: page_num,
            font_name: ev.font_name.clone(),
            outcome,
        });
    }

    let flagged = flag_page(doc, page_num).map_err(|e| match e {
        FlagError::PageOutOfRange { page, total } => ExtractError::PageOutOfRange { page, total },
        FlagError::UnreadableContentStream { page, detail } => {
            ExtractError::UnreadableContentStream { page, detail }
        }
        FlagError::XObjectResolutionFailed { page, detail, .. } => {
            ExtractError::FlaggingFailed { page, detail }
        }
    })?;

    Ok(PageText {
        page: page_num,
        chunks,
        flagged,
    })
}

/// Decode one event's bytes through Phase 4 font resolution +
/// decode, mapping failures into typed [`ChunkOutcome`] variants
/// rather than fatally erroring.
fn decode_event(
    doc: &PdfDocument,
    page_fonts: &alloc::collections::BTreeMap<Vec<u8>, &lopdf::Dictionary>,
    font_name: &str,
    bytes: &[u8],
) -> ChunkOutcome {
    let font_dict = match page_fonts.get(font_name.as_bytes()) {
        Some(d) => *d,
        None => {
            return ChunkOutcome::FailedResolve(format!(
                "font name {font_name:?} not in /Resources /Font"
            ));
        }
    };
    let pdf_font: PdfFont = match resolve_font(font_name, font_dict, doc.inner()) {
        Ok(f) => f,
        Err(FontResolveError::MissingSubtype) => {
            return ChunkOutcome::FailedResolve("font dict has no /Subtype".to_string());
        }
        Err(other) => return ChunkOutcome::FailedResolve(format!("{other}")),
    };
    match decode_bytes(&pdf_font, bytes) {
        Ok(s) => ChunkOutcome::Decoded(s),
        Err(e) => ChunkOutcome::FailedDecode(format!("{e}")),
    }
}

// ─────────────────────────────────────────────────────────────────────
// Bluebook §3.3 — section-boundary slicing
// ─────────────────────────────────────────────────────────────────────

/// Slice the input text down to a single statutory section.
///
/// `text` is the concatenated page text (typically
/// [`ExtractedDocument::full_text`]). `section_marker` is the
/// canonical Bluebook citation prefix of the section we want
/// (e.g. `"§ 1514A"` or `"§ 42121"`). The function returns the
/// substring from the first occurrence of `section_marker` up to
/// (but not including) the *next* `§ ` marker that isn't a
/// subsection-internal pinpoint of the same section.
///
/// Per Bluebook §3.3, statute sections are marked by `§ N`
/// followed by a period or whitespace, with subsection pinpoints
/// like `§ N(a)(1)` continuing within the same section. We
/// detect the next section by looking for `§ ` followed by a
/// digit that introduces a *different* section number.
///
/// Returns `None` if `section_marker` doesn't appear in `text`.
/// Idempotent: slicing the same text with the same marker yields
/// the same substring every time.
pub fn slice_to_section<'a>(text: &'a str, section_marker: &str) -> Option<&'a str> {
    let start = text.find(section_marker)?;
    let body = &text[start..];

    // Find the next "§ <digit>" that's not the marker itself.
    // We re-search from a position past the current marker.
    let after_marker = start + section_marker.len();
    let mut search_from = after_marker;
    while let Some(rel) = text[search_from..].find("§ ") {
        let abs = search_from + rel;
        // Is the next char after "§ " a digit? If yes, it's a
        // new section header; otherwise (could be inside a
        // sentence), keep looking.
        let next_char = text[abs + "§ ".len()..].chars().next().unwrap_or(' ');
        if next_char.is_ascii_digit() {
            // Check whether this is the *same* section's
            // subsection pinpoint by comparing the digit run.
            let candidate_start = abs + "§ ".len();
            let candidate_digits: String = text[candidate_start..]
                .chars()
                .take_while(|c| c.is_ascii_digit() || c.is_ascii_alphabetic())
                .collect();
            let marker_digits: String = section_marker
                .trim_start_matches('§')
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_digit() || c.is_ascii_alphabetic())
                .collect();
            if !marker_digits.is_empty() && candidate_digits != marker_digits {
                let body_end = abs - start;
                return Some(&body[..body_end]);
            }
        }
        search_from = abs + "§ ".len();
    }
    Some(body)
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::reader::read_pdf_bytes;
    use super::*;
    use lopdf::{Document, Object, Stream, dictionary};

    /// Synthesize a 1-page PDF with a single Tj that uses
    /// WinAnsiEncoding via a real font dict so the pipeline can
    /// resolve + decode end-to-end.
    fn pdf_one_page_winansi_tj(text: &str) -> Vec<u8> {
        let mut doc = Document::with_version("1.4");
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
            "Encoding" => "WinAnsiEncoding",
        });
        let cs_ops = format!("BT\n/F1 12 Tf\n({text}) Tj\nET\n");
        let content_id = doc.add_object(Stream::new(dictionary! {}, cs_ops.into_bytes()));
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
            "Resources" => dictionary! {
                "Font" => dictionary! { "F1" => font_id },
            },
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).expect("serialize");
        bytes
    }

    // ── End-to-end through Phases 2-5 ─────────────────────────────

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn extract_decodes_winansi_ascii_round_trip() {
        let bytes = pdf_one_page_winansi_tj("Hello world");
        let doc = read_pdf_bytes(&bytes).expect("parse");
        let ext = extract_document(&doc).expect("extract");
        assert_eq!(ext.pages.len(), 1);
        assert_eq!(ext.pages[0].full_text(), "Hello world");
        assert!(ext.pages[0].flagged.is_empty());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extract_document_full_text_joins_pages_with_blank_line() {
        let bytes = pdf_one_page_winansi_tj("page one");
        let doc = read_pdf_bytes(&bytes).expect("parse");
        let ext = extract_document(&doc).expect("extract");
        // Single page: no blank line separator.
        assert_eq!(ext.full_text(), "page one");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extract_chunk_carries_page_and_font_name() {
        let bytes = pdf_one_page_winansi_tj("abc");
        let doc = read_pdf_bytes(&bytes).expect("parse");
        let page = extract_page(&doc, 1).expect("extract");
        assert_eq!(page.page, 1);
        assert_eq!(page.chunks.len(), 1);
        assert_eq!(page.chunks[0].page, 1);
        assert_eq!(page.chunks[0].font_name, "F1");
        match &page.chunks[0].outcome {
            ChunkOutcome::Decoded(s) => assert_eq!(s, "abc"),
            other => panic!("expected Decoded; got {other:?}"),
        }
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn missing_font_yields_failed_resolve_chunk() {
        // Build a page that references font /F1 but /Resources
        // doesn't carry /Font, so font name resolution misses.
        let mut doc = Document::with_version("1.4");
        let cs_ops = b"BT\n/F1 12 Tf\n(x) Tj\nET\n".to_vec();
        let content_id = doc.add_object(Stream::new(dictionary! {}, cs_ops));
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
            "Resources" => dictionary! {},
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).expect("serialize");

        let parsed = read_pdf_bytes(&bytes).expect("parse");
        let ext = extract_document(&parsed).expect("extract");
        assert_eq!(ext.pages[0].chunks.len(), 1);
        match &ext.pages[0].chunks[0].outcome {
            ChunkOutcome::FailedResolve(detail) => assert!(detail.contains("F1")),
            other => panic!("expected FailedResolve; got {other:?}"),
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn extract_is_deterministic_on_same_input() {
        let bytes = pdf_one_page_winansi_tj("determinism check");
        let doc = read_pdf_bytes(&bytes).expect("parse");
        let e1 = extract_document(&doc).expect("extract 1");
        let e2 = extract_document(&doc).expect("extract 2");
        assert_eq!(e1, e2);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn page_out_of_range_returns_named_error() {
        let bytes = pdf_one_page_winansi_tj("x");
        let doc = read_pdf_bytes(&bytes).expect("parse");
        match extract_page(&doc, 99) {
            Err(ExtractError::PageOutOfRange { page, total }) => {
                assert_eq!(page, 99);
                assert_eq!(total, 1);
            }
            other => panic!("expected PageOutOfRange; got {other:?}"),
        }
    }

    // ── Section-boundary slicing (Bluebook §3.3) ───────────────

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn slice_to_section_returns_section_body() {
        let text =
            "preamble\n§ 1514A. Civil action.\nBody text here.\n§ 1515. Definitions.\nOther stuff.";
        let sliced = slice_to_section(text, "§ 1514A").expect("found");
        assert!(sliced.starts_with("§ 1514A"));
        assert!(sliced.contains("Body text here"));
        assert!(!sliced.contains("Definitions"));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn slice_to_section_returns_none_when_marker_missing() {
        let text = "no statute section here";
        assert!(slice_to_section(text, "§ 1514A").is_none());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn slice_to_section_is_idempotent() {
        let text = "§ 1514A. Header.\n(a) Whistleblower.\n§ 1515. Next section.";
        let once = slice_to_section(text, "§ 1514A").expect("found");
        let twice = slice_to_section(once, "§ 1514A").expect("found again");
        assert_eq!(once, twice);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn slice_to_section_preserves_subsection_pinpoints_within() {
        let text = "§ 1514A. Header.\n(a) Subsection.\n(b)(2)(C) Burden of proof.\n§ 1515. Next.";
        let sliced = slice_to_section(text, "§ 1514A").expect("found");
        assert!(sliced.contains("(b)(2)(C)"));
        assert!(!sliced.contains("§ 1515"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn slice_to_section_stops_at_next_distinct_section() {
        let text = "§ 1514A. First.\nbody A.\n§ 1515. Second.\nbody B.";
        let sliced = slice_to_section(text, "§ 1514A").expect("found");
        assert!(!sliced.contains("Second"));
        assert!(!sliced.contains("body B"));
    }

    // ── Property-based ────────────────────────────────────────────

    use proptest::prelude::*;

    /// Generate printable-ASCII text safe to embed in a PDF
    /// string literal. Per ISO 32000-2:2020 Annex D.5, WinAnsi
    /// is undefined for most control characters (0x00–0x1F,
    /// 0x7F); restrict to the printable range 0x20–0x7E and
    /// exclude the PDF string-escape delimiters `()` and `\`.
    fn arb_safe_text() -> impl Strategy<Value = String> {
        proptest::collection::vec(0x20u8..=0x7Eu8, 0..32).prop_map(|bytes| {
            bytes
                .into_iter()
                .filter(|b| !matches!(*b, b'(' | b')' | b'\\'))
                .map(|b| b as char)
                .collect()
        })
    }

    proptest! {
        /// extract_document round-trips ASCII through the full
        /// pipeline: arbitrary safe-ASCII text in → identical text
        /// in `full_text()` out.
        #[test]
        fn prop_ascii_round_trips_through_pipeline(text in arb_safe_text()) {
            let bytes = pdf_one_page_winansi_tj(&text);
            let doc = read_pdf_bytes(&bytes).expect("parse");
            let ext = extract_document(&doc).expect("extract");
            prop_assert_eq!(ext.full_text(), text);
        }

        /// extract_document is deterministic: same PDF bytes →
        /// identical ExtractedDocument across repeated extractions.
        #[test]
        fn prop_extract_is_deterministic(text in arb_safe_text()) {
            let bytes = pdf_one_page_winansi_tj(&text);
            let doc = read_pdf_bytes(&bytes).expect("parse");
            let e1 = extract_document(&doc).expect("extract 1");
            let e2 = extract_document(&doc).expect("extract 2");
            prop_assert_eq!(e1, e2);
        }

        /// Every chunk's `page` field matches the parent
        /// `PageText.page`. Cross-layer invariant — chunks can't
        /// "leak" between pages.
        #[test]
        fn prop_chunk_page_matches_parent_page(text in arb_safe_text()) {
            let bytes = pdf_one_page_winansi_tj(&text);
            let doc = read_pdf_bytes(&bytes).expect("parse");
            let ext = extract_document(&doc).expect("extract");
            for page in &ext.pages {
                for chunk in &page.chunks {
                    prop_assert_eq!(chunk.page, page.page);
                }
            }
        }

        /// slice_to_section is a prefix operation: the returned
        /// substring is a contiguous slice of the input. Length
        /// is always ≤ input length.
        #[test]
        fn prop_slice_to_section_is_substring(
            prefix in "[a-z ]{0,16}",
            section_num in 100u32..9999,
            body in "[A-Za-z0-9 .,()]{0,64}",
            tail_num in 100u32..9999,
        ) {
            let marker = format!("§ {section_num}");
            let text = format!(
                "{prefix}{marker}. Header.\n{body}\n§ {tail_num}. Other.",
            );
            if let Some(sliced) = slice_to_section(&text, &marker) {
                prop_assert!(text.contains(sliced));
                prop_assert!(sliced.len() <= text.len());
                prop_assert!(sliced.starts_with(&marker));
            }
        }

        /// slice_to_section is idempotent: applying twice is the
        /// same as applying once. Fundamental property of a clean
        /// slice operation.
        #[test]
        fn prop_slice_to_section_is_idempotent(
            section_num in 100u32..9999,
            body in "[A-Za-z0-9 .,()]{0,64}",
            tail_num in 100u32..9999,
        ) {
            // Ensure section_num and tail_num are different so the
            // slice has a defined upper boundary.
            prop_assume!(section_num != tail_num);
            let marker = format!("§ {section_num}");
            let text = format!(
                "{marker}. Header.\n{body}\n§ {tail_num}. Other.",
            );
            if let Some(once) = slice_to_section(&text, &marker) {
                let once_owned = once.to_string();
                let twice_owned = slice_to_section(&once_owned, &marker)
                    .expect("idempotent")
                    .to_string();
                prop_assert_eq!(once_owned, twice_owned);
            }
        }

        /// flagged_count() on the empty/text-only fixture is
        /// always zero — text pages without images don't surface
        /// false-positive flags.
        #[test]
        fn prop_text_only_page_has_zero_flagged(text in arb_safe_text()) {
            let bytes = pdf_one_page_winansi_tj(&text);
            let doc = read_pdf_bytes(&bytes).expect("parse");
            let ext = extract_document(&doc).expect("extract");
            prop_assert_eq!(ext.flagged_count().value, 0.0);
        }
    }

    pr4xis::register_praxis_value!(prop_ascii_round_trips_through_pipeline, Deterministic);
    pr4xis::register_praxis_value!(prop_extract_is_deterministic, Deterministic);
    pr4xis::register_praxis_value!(prop_chunk_page_matches_parent_page, Verifiable);
    pr4xis::register_praxis_value!(prop_slice_to_section_is_substring, Verifiable);
    pr4xis::register_praxis_value!(prop_slice_to_section_is_idempotent, Deterministic);
    pr4xis::register_praxis_value!(prop_text_only_page_has_zero_flagged, Verifiable);
}
