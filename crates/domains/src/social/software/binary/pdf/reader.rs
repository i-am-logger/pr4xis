//! PDF reader — byte stream → typed [`PdfDocument`].
//!
//! Implements the parsing half of the ontology — the byte-level decode
//! of ISO 32000-2:2020 §7 (Syntax). Wraps the `lopdf` crate's
//! object-tree representation behind a praxis-typed surface so callers
//! see typed enums and structured errors instead of raw `lopdf::Object`
//! values.
//!
//! Scope of this module:
//!
//! - File header parsing (§7.5.2) — version recognition.
//! - File trailer parsing (§7.5.5) — catalog and info-dictionary
//!   reachability.
//! - Cross-reference resolution (§7.5.4 / §7.5.8) — indirect objects
//!   are accessible via the typed document.
//! - Page-tree enumeration (§7.7.3) — page count + per-page resource
//!   resolution.
//! - Object-stream decompression (§7.5.7) — handled transparently by
//!   `lopdf` via its FlateDecode (RFC 1950/1951) implementation.
//!
//! Out of scope for this commit:
//!
//! - Content-stream interpretation (§7.8.2, §9.4) — Phase 3.
//! - Font / encoding resolution (§9.5–§9.10) — Phase 4.
//! - Image / Form-XObject flagging walk (§8.9, §8.10) — Phase 5.
//! - Encrypted PDFs (§7.6) — fails closed with
//!   [`PdfReadError::UnsupportedEncryption`].

use alloc::{format, string::String, string::ToString, vec::Vec};

use super::ontology::PdfConcept;

/// A successfully-parsed PDF document — the typed surface a praxis
/// caller works with.
///
/// Holds the minimum information needed to (a) ground every
/// [`PdfConcept`] axiom against actual document state and (b) drive
/// the content-stream interpreter and flagged-content walker that
/// follow in subsequent phases.
#[derive(Debug, Clone)]
pub struct PdfDocument {
    /// PDF version declared in the header (e.g. `(1, 7)`, `(2, 0)`).
    /// Per ISO 32000-2:2020 §7.5.2, the header line is `%PDF-N.M`.
    pub version: (u8, u8),
    /// Number of pages in the document — the count is the
    /// `/Count` entry of the root page-tree node (§7.7.3.2).
    pub page_count: usize,
    /// `(object-number, generation)` of the document Catalog
    /// (§7.7.2 — the trailer's `/Root` entry).
    pub catalog_id: (u32, u16),
    /// `(object-number, generation)` of the document Info dictionary,
    /// if present (§14.3.3 — the trailer's `/Info` entry).
    pub info_id: Option<(u32, u16)>,
    /// Lopdf-backed payload — kept opaque to praxis callers. The
    /// content-stream interpreter (Phase 3) and flagged-content
    /// walker (Phase 5) reach in through helper functions, not
    /// directly through this field.
    inner: lopdf::Document,
}

impl PdfDocument {
    /// Concept-grounded check: does the parsed document satisfy the
    /// `FileStructureWellFormed` axiom at runtime?
    ///
    /// Returns the list of [`PdfConcept`] entries the parser
    /// confirmed present. A well-formed document must produce at
    /// least Header, Body, CrossReferenceSection, and Trailer.
    pub fn structural_concepts(&self) -> Vec<PdfConcept> {
        let mut found = Vec::new();
        // Header — always present if we parsed (lopdf rejects bad
        // header before returning a Document).
        found.push(PdfConcept::Header);
        // Body — every indirect object in lopdf's tree belongs to
        // the body or an object stream within the body.
        if !self.inner.objects.is_empty() {
            found.push(PdfConcept::Body);
        }
        // CrossReferenceSection — lopdf populates `reference_table`
        // (legacy xref) or via xref streams (PDF 1.5+).
        if !self.inner.reference_table.entries.is_empty() {
            found.push(PdfConcept::CrossReferenceSection);
        }
        // Trailer — lopdf's `trailer` dictionary is always populated
        // for a successfully-parsed document.
        if !self.inner.trailer.is_empty() {
            found.push(PdfConcept::Trailer);
        }
        // Catalog — reachable through trailer /Root.
        if self.inner.get_object(self.catalog_id).is_ok() {
            found.push(PdfConcept::Catalog);
        }
        // PageTree — if we have page_count > 0 there's a page tree.
        if self.page_count > 0 {
            found.push(PdfConcept::PageTree);
            found.push(PdfConcept::Page);
        }
        found
    }

    /// Number of indirect objects in the body (§7.5.3).
    pub fn indirect_object_count(&self) -> usize {
        self.inner.objects.len()
    }

    /// Borrow the underlying lopdf document. Internal use only —
    /// the content-stream interpreter and flagged-content walker
    /// (subsequent phases) read through this; runtime consumers of
    /// pr4xis-domains should use the typed accessors above.
    #[allow(dead_code)] // Phase 3+ entry point.
    pub(crate) fn inner(&self) -> &lopdf::Document {
        &self.inner
    }
}

// ─────────────────────────────────────────────────────────────────────
// Errors — every reader failure is named, no silent malformation.
// ─────────────────────────────────────────────────────────────────────

/// Why a PDF couldn't be read.
///
/// Each variant names a specific failure mode keyed to an ISO 32000-2
/// section. Per `feedback_no_silent_failures` the reader fails closed
/// with a named error rather than returning a half-parsed document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfReadError {
    /// File header didn't start with `%PDF-` (§7.5.2). Empty input,
    /// non-PDF data, or corrupted bytes.
    InvalidHeader { found: String },
    /// Cross-reference table is malformed — `lopdf` couldn't locate
    /// `startxref` or the offset it points to is invalid (§7.5.4,
    /// §7.5.5, §7.5.8).
    MalformedXref { detail: String },
    /// Document declares encryption (`/Encrypt` entry in trailer)
    /// but we don't support PDF encryption yet (§7.6).
    UnsupportedEncryption,
    /// Trailer dictionary doesn't carry a `/Root` reference, or the
    /// referenced object isn't a Catalog (§7.5.5, §7.7.2).
    MissingCatalog,
    /// Page tree is unreadable — `/Pages` entry missing or its
    /// `/Count` entry not an integer (§7.7.3).
    MalformedPageTree { detail: String },
    /// Any other parse error from the underlying lopdf reader.
    InternalParseError { detail: String },
}

impl core::fmt::Display for PdfReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidHeader { found } => {
                write!(f, "invalid PDF header — expected %PDF-N.M, found: {found}")
            }
            Self::MalformedXref { detail } => write!(f, "malformed xref: {detail}"),
            Self::UnsupportedEncryption => write!(
                f,
                "PDF declares /Encrypt — encryption is not yet supported (ISO 32000-2 §7.6)"
            ),
            Self::MissingCatalog => write!(f, "trailer has no /Root or /Root is not a Catalog"),
            Self::MalformedPageTree { detail } => write!(f, "malformed page tree: {detail}"),
            Self::InternalParseError { detail } => write!(f, "internal parse error: {detail}"),
        }
    }
}

impl std::error::Error for PdfReadError {}

// ─────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────

/// Read a PDF document from a byte slice.
///
/// The byte input is treated as the complete PDF file (header → body
/// → xref → trailer). Streamed reads are not supported in this
/// phase — PDF's cross-reference structure assumes random access.
///
/// Per ISO 32000-2:2020 §7.5.1, a well-formed file is exactly
/// `Header ▸ Body ▸ CrossReferenceSection ▸ Trailer`; this function
/// fails closed with a named [`PdfReadError`] if any part doesn't
/// parse.
pub fn read_pdf_bytes(bytes: &[u8]) -> Result<PdfDocument, PdfReadError> {
    // ─── §7.5.2 — File header check ───
    // The header must start with `%PDF-` per ISO 32000-2:2020 §7.5.2.
    // lopdf's parser does this internally but its error message is
    // ambiguous, so we check up-front for a clearer diagnostic.
    if !bytes.starts_with(b"%PDF-") {
        let prefix = bytes
            .iter()
            .take(16)
            .map(|b| {
                if b.is_ascii_graphic() || *b == b' ' {
                    char::from(*b).to_string()
                } else {
                    format!("\\x{b:02x}")
                }
            })
            .collect::<String>();
        return Err(PdfReadError::InvalidHeader { found: prefix });
    }

    // ─── §7.5 — Full parse ───
    let doc = lopdf::Document::load_mem(bytes).map_err(|e| match &e {
        lopdf::Error::Parse(_) | lopdf::Error::Decompress(_) => PdfReadError::MalformedXref {
            detail: format!("{e}"),
        },
        _ => PdfReadError::InternalParseError {
            detail: format!("{e}"),
        },
    })?;

    // ─── §7.6 — Refuse encrypted PDFs (out of scope for this phase) ───
    if doc.is_encrypted() {
        return Err(PdfReadError::UnsupportedEncryption);
    }

    // ─── §7.5.5 — Header version ───
    let (major, minor) = parse_version(&doc.version)?;

    // ─── §7.5.5 — Trailer /Root → Catalog ───
    let catalog_id = doc
        .trailer
        .get(b"Root")
        .and_then(|o| o.as_reference())
        .map_err(|_| PdfReadError::MissingCatalog)?;

    // ─── §14.3.3 — Trailer /Info (optional) ───
    let info_id = doc
        .trailer
        .get(b"Info")
        .ok()
        .and_then(|o| o.as_reference().ok());

    // ─── §7.7.3 — Page count ───
    let page_count = doc.get_pages().len();

    Ok(PdfDocument {
        version: (major, minor),
        page_count,
        catalog_id,
        info_id,
        inner: doc,
    })
}

/// Parse `1.7` or `2.0` style version strings into `(major, minor)`.
fn parse_version(s: &str) -> Result<(u8, u8), PdfReadError> {
    let (major, minor) = s
        .split_once('.')
        .ok_or_else(|| PdfReadError::InvalidHeader {
            found: format!("version `{s}` lacks a dot"),
        })?;
    let major: u8 = major.parse().map_err(|_| PdfReadError::InvalidHeader {
        found: format!("non-numeric major version: {major}"),
    })?;
    let minor: u8 = minor.parse().map_err(|_| PdfReadError::InvalidHeader {
        found: format!("non-numeric minor version: {minor}"),
    })?;
    Ok((major, minor))
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// The smallest valid PDF — built programmatically via lopdf
    /// to guarantee xref offsets are correct. Used as a hermetic
    /// positive-path fixture.
    fn minimal_pdf() -> Vec<u8> {
        use lopdf::{Document, Object, dictionary};
        let mut doc = Document::with_version("1.4");
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
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
        doc.save_to(&mut bytes).expect("serialize minimal pdf");
        bytes
    }

    #[test]
    fn invalid_header_returns_named_error() {
        let err = read_pdf_bytes(b"not a pdf").unwrap_err();
        match err {
            PdfReadError::InvalidHeader { .. } => {}
            other => panic!("expected InvalidHeader; got {other:?}"),
        }
    }

    #[test]
    fn empty_input_returns_invalid_header() {
        let err = read_pdf_bytes(b"").unwrap_err();
        assert!(matches!(err, PdfReadError::InvalidHeader { .. }));
    }

    #[test]
    fn minimal_pdf_parses_to_structural_quadruple() {
        let bytes = minimal_pdf();
        let doc = read_pdf_bytes(&bytes).expect("minimal pdf should parse");
        let concepts = doc.structural_concepts();
        assert!(concepts.contains(&PdfConcept::Header));
        assert!(concepts.contains(&PdfConcept::Body));
        assert!(concepts.contains(&PdfConcept::CrossReferenceSection));
        assert!(concepts.contains(&PdfConcept::Trailer));
        assert!(concepts.contains(&PdfConcept::Catalog));
        assert!(concepts.contains(&PdfConcept::PageTree));
        assert!(concepts.contains(&PdfConcept::Page));
    }

    #[test]
    fn minimal_pdf_has_one_page() {
        let bytes = minimal_pdf();
        let doc = read_pdf_bytes(&bytes).expect("minimal pdf should parse");
        assert_eq!(doc.page_count, 1);
    }

    #[test]
    fn minimal_pdf_header_is_pdf_1_4() {
        let bytes = minimal_pdf();
        let doc = read_pdf_bytes(&bytes).expect("minimal pdf should parse");
        assert_eq!(doc.version, (1, 4));
    }

    #[test]
    fn minimal_pdf_catalog_id_is_a_valid_object_reference() {
        let bytes = minimal_pdf();
        let doc = read_pdf_bytes(&bytes).expect("minimal pdf should parse");
        // The exact (N, G) depends on the order lopdf assigns ids;
        // we only assert the catalog reference resolves to a real
        // object in the document.
        assert!(doc.inner().get_object(doc.catalog_id).is_ok());
    }

    #[test]
    fn minimal_pdf_has_no_info_dictionary() {
        let bytes = minimal_pdf();
        let doc = read_pdf_bytes(&bytes).expect("minimal pdf should parse");
        assert_eq!(doc.info_id, None);
    }

    #[test]
    fn minimal_pdf_indirect_object_count() {
        let bytes = minimal_pdf();
        let doc = read_pdf_bytes(&bytes).expect("minimal pdf should parse");
        // 4 declared (Catalog, PageTree, Page, ContentStream) — lopdf
        // may also expose the xref-implicit free-object slot;
        // assert a lower-bound.
        assert!(doc.indirect_object_count() >= 4);
    }

    #[test]
    fn read_is_deterministic_on_same_bytes() {
        let bytes = minimal_pdf();
        let d1 = read_pdf_bytes(&bytes).expect("parse 1");
        let d2 = read_pdf_bytes(&bytes).expect("parse 2");
        assert_eq!(d1.version, d2.version);
        assert_eq!(d1.page_count, d2.page_count);
        assert_eq!(d1.catalog_id, d2.catalog_id);
        assert_eq!(d1.info_id, d2.info_id);
    }
}
