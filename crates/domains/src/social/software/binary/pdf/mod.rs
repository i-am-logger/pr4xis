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

#[cfg(test)]
mod tests;

pub use ontology::*;
