//! PDF — Portable Document Format ontology (ISO 32000-2:2020).
//!
//! The praxis-way layer for reading PDF documents. Mirrors the
//! `software::markup::xml` pattern: a concept enum naming the
//! structural elements of the format, a kinded category over them,
//! axioms grounded in the spec, runtime types that the decoder
//! materializes from byte streams.
//!
//! Scope of this module:
//!
//! - **Document structure** (ISO 32000-2 §7.5) — header, body,
//!   cross-reference, trailer.
//! - **Indirect objects and references** (§7.3.10) — the addressing
//!   model that lets objects refer to each other by `(object-number,
//!   generation)` pairs.
//! - **Page tree and resources** (§7.7) — how a document's pages and
//!   their fonts/images are organized.
//! - **Content streams** (§7.8.2 / §8.2) — the operator stream that
//!   draws each page.
//! - **Filters** (§7.4) — the transformation chain that decodes
//!   compressed/encoded streams.
//! - **Text** (§9) — fonts, encodings, text-showing operators.
//! - **Image flagging** (§8.9 Images, §8.10 Form XObjects) — per
//!   `feedback_pdf_text_only_until_image_understanding`, image
//!   content MUST be enumerated as `FlaggedContent`, never silently
//!   dropped or paraphrased.
//!
//! Out of scope (Phase 6 image-understanding seam):
//!
//! - Pixel-level image extraction or OCR.
//! - Rendered visual layout (we extract logical text, not painted
//!   pixel positions).
//! - Encrypted PDFs (current implementation fails closed with a
//!   named error; future work adds the AES-256 / public-key
//!   handlers per §7.6).
//!
//! Spec references:
//!
//! - ISO 32000-2:2020 — *Document management — Portable document
//!   format — Part 2: PDF 2.0* (ISO, 2020).
//! - ISO 32000-1:2008 — *PDF 1.7* (kept as the legacy citation for
//!   features that predate PDF 2.0).
//! - Adobe Tech Note #5014 — *ToUnicode Mapping File Tutorial*
//!   (Adobe Systems, 2003), the authoritative ToUnicode CMap spec.
//! - IETF RFC 1950 / RFC 1951 — zlib container and DEFLATE algorithm
//!   used by `FlateDecode`.

pub mod ontology;

/// PDF byte-stream reader — opt-in behind the `pdf` feature. The
/// ontology surface in [`ontology`] is available unconditionally;
/// only the lopdf-backed parser lives here.
#[cfg(feature = "pdf")]
pub mod reader;

/// Content-stream interpreter — operator sequence → typed events.
/// Phase 3 of M4.γ; consumes the byte streams the reader exposes
/// and produces text-show events + flagged-graphics events.
#[cfg(feature = "pdf")]
pub mod content_stream;

/// Font + encoding pipeline — byte → Unicode resolution per
/// ISO 32000-2:2020 §9.10.2 (ToUnicode CMap > /Encoding > built-in).
/// Phase 4 of M4.γ; consumes the TextShowEvents emitted by
/// `content_stream` and produces decoded `String`s.
#[cfg(feature = "pdf")]
pub mod font;

/// Adobe Glyph List — glyph name → Unicode codepoint resolver
/// for `/Differences` array overrides per ISO 32000-2:2020
/// §9.6.5.4. Loaded from a verbatim copy of Adobe's published
/// `glyphlist.txt`; sha256 pinned and verified at test time.
#[cfg(feature = "pdf")]
pub mod agl;

/// Image / Form-XObject / inline-image flagging walker — emits
/// `Vec<FlaggedContent>` for every non-text content piece per
/// `feedback_pdf_text_only_until_image_understanding`. Phase 5
/// of M4.γ; resolves the `Do <name>` events from
/// `content_stream` against page resources to reclassify
/// `FlaggedKind::FormXObject` → `ImageXObject` when warranted.
#[cfg(feature = "pdf")]
pub mod flagged;

/// Extraction pipeline — composes Phases 2-5 into a typed
/// `(Vec<PageText>, Vec<FlaggedContent>)` output plus Bluebook
/// §3.3 statute-section slicing. Phase 6 of M4.γ.
#[cfg(feature = "pdf")]
pub mod extract;

/// `PdfLens : WellBehavedLens` — the byte-anchored lens binding
/// `bytes ⇆ PdfExtraction` via [`read_pdf_bytes`](reader::read_pdf_bytes)
/// + [`extract_document`](extract::extract_document). Closes
/// M4.γ.completion: the praxis-native PDF text-only path is now a
/// first-class registered lens, parallel to
/// [`crate::formal::meta::xsd::lens::XsdSchemaLens`] (XSD) and
/// [`crate::formal::meta::dtd::lens::DtdLens`] (DTD).
#[cfg(feature = "pdf")]
pub mod lens;

#[cfg(test)]
mod tests;

pub use ontology::*;
