//! PDF ontology — the structural surface, ISO 32000-2:2020 grounded.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::logic::proof::{SimpleProof, Verdict};
use pr4xis::ontology::meta::{Citation, Label, ModulePath, OntologyName, Provenance};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ─────────────────────────────────────────────────────────────────────
// Concept enum — the named structural objects in a PDF document.
//
// ISO 32000-2:2020 §7 (Syntax) names every concept below. Variants
// stay close to the spec's terms so future readers can grep both
// directions.
// ─────────────────────────────────────────────────────────────────────

/// Every named structural object in a PDF file.
///
/// Variants map to ISO 32000-2:2020 section headings:
///
/// - File-level: `Document`, `Header`, `Body`, `CrossReferenceSection`,
///   `Trailer` (§7.5).
/// - Object-level: `IndirectObject`, `IndirectReference`, `ObjectStream`
///   (§7.3.10, §7.5.7).
/// - Document-level: `Catalog`, `PageTree`, `Page` (§7.7.2–§7.7.3).
/// - Page-content: `ContentStream`, `Resources` (§7.8.2, §7.7.3.3).
/// - Text: `Font`, `FontDescriptor`, `Encoding`, `ToUnicodeCmap`
///   (§9.5–§9.10).
/// - Graphics: `XObject`, `ImageXObject`, `FormXObject` (§8.8–§8.10).
/// - Encoding chain: `FilterChain` (§7.4).
/// - Logical structure: `StructTreeRoot`, `StructureElement` (§14.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum PdfConcept {
    /// The whole PDF document — header + body + xref + trailer.
    Document,
    /// File header — `%PDF-N.N` magic bytes (§7.5.2).
    Header,
    /// File body — sequence of indirect objects (§7.5.3).
    Body,
    /// Cross-reference section — either xref table (§7.5.4) or
    /// xref stream (§7.5.8). Maps `(object-number, generation)`
    /// pairs to byte offsets.
    CrossReferenceSection,
    /// File trailer — dictionary naming the catalog, info, and
    /// previous xref offset (§7.5.5).
    Trailer,
    /// An indirect object — addressed `N G obj … endobj` (§7.3.10).
    IndirectObject,
    /// An indirect reference — `N G R` form (§7.3.10).
    IndirectReference,
    /// Object stream — a compressed container holding many indirect
    /// objects, introduced in PDF 1.5 (§7.5.7).
    ObjectStream,
    /// Document catalog — the root object the trailer points at;
    /// names the page tree, structure tree, metadata, etc. (§7.7.2).
    Catalog,
    /// Page tree — the hierarchical container of all pages (§7.7.3).
    PageTree,
    /// A single page (§7.7.3.3).
    Page,
    /// A page's content stream — sequence of graphics operators that
    /// paint the page (§7.8.2, §8.2).
    ContentStream,
    /// Resource dictionary — fonts, XObjects, color spaces, …
    /// referenced by a page's content stream (§7.7.3.3).
    Resources,
    /// A font resource (§9.5).
    Font,
    /// A font descriptor — metrics, embedded program, flags (§9.8).
    FontDescriptor,
    /// A character encoding map — glyph code → glyph name (§9.6.5).
    Encoding,
    /// A ToUnicode CMap — glyph code → Unicode (Adobe Tech Note
    /// #5014; §9.10.2).
    ToUnicodeCmap,
    /// External object (XObject) parent concept (§8.8).
    XObject,
    /// Image XObject — raster image data (§8.9).
    ImageXObject,
    /// Form XObject — a content stream embeddable elsewhere (§8.10).
    FormXObject,
    /// Filter chain — `/Filter [ /A /B … ]` applied to a stream's
    /// bytes (§7.4).
    FilterChain,
    /// Structure tree root — the entry point for tagged-PDF logical
    /// structure (§14.7.2).
    StructTreeRoot,
    /// Structure element — a logical-structure node (§14.7.3).
    StructureElement,
}

// ─────────────────────────────────────────────────────────────────────
// Kinded relations — per OBO-RO (Smith et al. 2005) every edge in a
// praxis ontology carries a Kind tag. PDF has two:
//
// - `Containment` — structural parent-child in the document model
//   (Document contains Header, Page contains ContentStream, …).
// - `IndirectReference` — `N G R` pointer between indirect objects.
//   Distinct from containment because the same target object can
//   be referenced from many sources; containment is unique.
// ─────────────────────────────────────────────────────────────────────

/// Relation kinds tracked by PDF edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PdfRelationKind {
    /// Structural parent-child containment in the document model.
    Containment,
    /// `N G R` indirect-object reference (§7.3.10).
    IndirectReference,
}

/// A kinded edge between two PDF concepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PdfEdge {
    pub from: PdfConcept,
    pub to: PdfConcept,
    pub kind: PdfRelationKind,
}

impl PdfEdge {
    pub const fn contains(from: PdfConcept, to: PdfConcept) -> Self {
        Self {
            from,
            to,
            kind: PdfRelationKind::Containment,
        }
    }

    pub const fn references(from: PdfConcept, to: PdfConcept) -> Self {
        Self {
            from,
            to,
            kind: PdfRelationKind::IndirectReference,
        }
    }
}

impl Arrow for PdfEdge {
    type Object = PdfConcept;
    type Kind = PdfRelationKind;

    fn source(&self) -> PdfConcept {
        self.from
    }
    fn target(&self) -> PdfConcept {
        self.to
    }
    fn kind(&self) -> PdfRelationKind {
        self.kind
    }
    fn meta(&self) -> Provenance {
        Provenance {
            name: OntologyName::new_static("PdfEdge"),
            description: Label::new_static(
                "PDF structural or referential edge — ISO 32000-2:2020 §7",
            ),
            citation: Citation::parse_static("ISO 32000-2:2020 §7"),
            module_path: ModulePath::new_static(module_path!()),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Category — the structural relations PDF enforces between concepts.
// ─────────────────────────────────────────────────────────────────────

/// The PDF category — ISO 32000-2:2020 structural rules as
/// category morphisms.
pub struct PdfCategory;

impl Category for PdfCategory {
    type Object = PdfConcept;
    type Morphism = PdfEdge;

    fn identity(obj: &PdfConcept) -> PdfEdge {
        PdfEdge {
            from: *obj,
            to: *obj,
            kind: PdfRelationKind::Containment,
        }
    }

    fn compose(f: &PdfEdge, g: &PdfEdge) -> Option<PdfEdge> {
        if f.to != g.from {
            return None;
        }
        // Composition is well-defined only when both edges share
        // the same kind, or when one is an identity (self-edge).
        // Mixing Containment with IndirectReference would conflate
        // distinct relation kinds, so we refuse.
        let f_id = f.from == f.to;
        let g_id = g.from == g.to;
        let kind = if f_id {
            g.kind
        } else if g_id || f.kind == g.kind {
            f.kind
        } else {
            return None;
        };
        Some(PdfEdge {
            from: f.from,
            to: g.to,
            kind,
        })
    }

    fn morphisms() -> Vec<PdfEdge> {
        use PdfConcept::*;

        // Generators: the directly-asserted edges from the spec.
        // Transitive closure (same-kind paths) is computed below so
        // the catalog satisfies the Category closure law.
        let generators: Vec<PdfEdge> = vec![
            // ─── File-level containment (ISO 32000-2 §7.5) ───
            PdfEdge::contains(Document, Header),
            PdfEdge::contains(Document, Body),
            PdfEdge::contains(Document, CrossReferenceSection),
            PdfEdge::contains(Document, Trailer),
            // Body holds indirect objects (§7.5.3) and object streams
            // (§7.5.7); an object stream itself holds indirect objects.
            PdfEdge::contains(Body, IndirectObject),
            PdfEdge::contains(Body, ObjectStream),
            PdfEdge::contains(ObjectStream, IndirectObject),
            // ─── Document logical structure (§7.7) ───
            // Trailer's /Root entry references the Catalog (§7.5.5).
            PdfEdge::references(Trailer, Catalog),
            // Catalog's /Pages entry references the page tree (§7.7.2).
            PdfEdge::references(Catalog, PageTree),
            PdfEdge::contains(PageTree, Page),
            PdfEdge::contains(PageTree, PageTree),
            // Catalog optionally references the structure tree (§14.7.2).
            PdfEdge::references(Catalog, StructTreeRoot),
            PdfEdge::contains(StructTreeRoot, StructureElement),
            PdfEdge::contains(StructureElement, StructureElement),
            // ─── Page level (§7.7.3.3) ───
            PdfEdge::references(Page, ContentStream),
            PdfEdge::references(Page, Resources),
            // ─── Resources (§7.7.3.3) ───
            PdfEdge::references(Resources, Font),
            PdfEdge::references(Resources, XObject),
            // ─── Fonts (§9.5–§9.10) ───
            PdfEdge::references(Font, FontDescriptor),
            PdfEdge::references(Font, Encoding),
            PdfEdge::references(Font, ToUnicodeCmap),
            // ─── XObjects (§8.8–§8.10) ───
            PdfEdge::contains(XObject, ImageXObject),
            PdfEdge::contains(XObject, FormXObject),
            // Form XObjects contain a content stream (§8.10).
            PdfEdge::references(FormXObject, ContentStream),
            // ─── Filters (§7.4) ───
            // Any stream object may carry a filter chain.
            PdfEdge::contains(ContentStream, FilterChain),
            PdfEdge::contains(ImageXObject, FilterChain),
            PdfEdge::contains(FormXObject, FilterChain),
            PdfEdge::contains(ObjectStream, FilterChain),
        ];

        // ─── Transitive closure per relation kind ───────────────
        // The Category closure law requires that whenever f: A → B
        // and g: B → C are both in the catalog with the same kind,
        // their composition f▸g: A → C is also in the catalog.
        let kinds = [
            PdfRelationKind::Containment,
            PdfRelationKind::IndirectReference,
        ];
        let mut closure: Vec<PdfEdge> = Vec::new();
        for kind in kinds {
            let generators_for_kind: Vec<&PdfEdge> =
                generators.iter().filter(|e| e.kind == kind).collect();
            for c in PdfConcept::variants() {
                let mut reachable: Vec<PdfConcept> = Vec::new();
                let mut stack: Vec<PdfConcept> = vec![c];
                while let Some(curr) = stack.pop() {
                    for e in &generators_for_kind {
                        if e.from == curr && !reachable.contains(&e.to) {
                            reachable.push(e.to);
                            stack.push(e.to);
                        }
                    }
                }
                for t in reachable {
                    let edge = PdfEdge {
                        from: c,
                        to: t,
                        kind,
                    };
                    if !closure.contains(&edge) {
                        closure.push(edge);
                    }
                }
            }
        }

        // ─── Identity morphisms ───
        let mut m: Vec<PdfEdge> = PdfConcept::variants()
            .into_iter()
            .map(|c| PdfEdge {
                from: c,
                to: c,
                kind: PdfRelationKind::Containment,
            })
            .collect();
        m.extend(closure);
        m
    }
}

// ─────────────────────────────────────────────────────────────────────
// Symbols — PDF's byte-level magic markers (§7.5).
// ─────────────────────────────────────────────────────────────────────

/// PDF byte-level markers that the lexer keys on.
///
/// Per ISO 32000-2:2020 §7.5, PDF's file-level grammar is anchored on
/// these byte sequences: the header magic, the EOF marker, and the
/// keywords that introduce the cross-reference table and trailer.
pub struct PdfSymbols;

impl PdfSymbols {
    /// File-header magic prefix — every PDF starts with `%PDF-N.N`
    /// (§7.5.2). The bytes here are the prefix; the version follows.
    pub fn header_magic() -> &'static [u8] {
        b"%PDF-"
    }

    /// End-of-file marker — every PDF ends with `%%EOF` (§7.5.5).
    pub fn eof_marker() -> &'static [u8] {
        b"%%EOF"
    }

    /// Cross-reference table keyword (§7.5.4).
    pub fn xref_keyword() -> &'static [u8] {
        b"xref"
    }

    /// Trailer dictionary keyword (§7.5.5).
    pub fn trailer_keyword() -> &'static [u8] {
        b"trailer"
    }

    /// Cross-reference start offset keyword (§7.5.5).
    pub fn startxref_keyword() -> &'static [u8] {
        b"startxref"
    }

    /// Indirect-object open keyword (§7.3.10).
    pub fn obj_keyword() -> &'static [u8] {
        b"obj"
    }

    /// Indirect-object close keyword (§7.3.10).
    pub fn endobj_keyword() -> &'static [u8] {
        b"endobj"
    }

    /// Indirect-reference suffix (§7.3.10 — `N G R`).
    pub fn ref_keyword() -> &'static [u8] {
        b"R"
    }

    /// Stream open keyword (§7.3.8).
    pub fn stream_keyword() -> &'static [u8] {
        b"stream"
    }

    /// Stream close keyword (§7.3.8).
    pub fn endstream_keyword() -> &'static [u8] {
        b"endstream"
    }
}

// ─────────────────────────────────────────────────────────────────────
// Image-flagging — the type that carries content praxis refuses to
// silently drop per `feedback_pdf_text_only_until_image_understanding`.
// ─────────────────────────────────────────────────────────────────────

/// Non-text content encountered during extraction that praxis refuses
/// to silently drop or paraphrase.
///
/// Per the internal rule `feedback_pdf_text_only_until_image_understanding`:
/// *"PDF parsing extracts text only; images/diagrams/figures must be
/// flagged explicitly, never silently dropped or paraphrased. Waits
/// for Phase 6 image-understanding."* The text-extractor emits a
/// `Vec<FlaggedContent>` alongside its text output; downstream
/// consumers see exactly what wasn't extracted.
///
/// `Eq` is not derived because `dimensions` carries `f32`. PDF user
/// units are real-valued (§8.3.2.3); exact-bit equality on flagged
/// dimensions is not semantically meaningful.
#[derive(Debug, Clone, PartialEq)]
pub struct FlaggedContent {
    /// What kind of non-text content was found.
    pub kind: FlaggedKind,
    /// Page (1-indexed) where it was encountered.
    pub page: u32,
    /// `(object-number, generation)` of the source PDF object, when
    /// the content is referenced via an indirect object.
    pub object: Option<(u32, u16)>,
    /// Reported width × height in PDF user units, when available.
    pub dimensions: Option<(f32, f32)>,
    /// Human-readable description (e.g. `"inline image, 240×180 px"`).
    pub note: String,
}

/// Kinds of non-text content the extractor flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlaggedKind {
    /// Image XObject — raster image referenced by the page (§8.9).
    ImageXObject,
    /// Form XObject — embedded content stream possibly painting
    /// images, vector graphics, or text (§8.10). Flagged because we
    /// cannot statically rule out non-text content without rendering.
    FormXObject,
    /// Inline image — `BI…ID…EI` operator triple in a content
    /// stream (§8.9.7).
    InlineImage,
    /// Vector path content (`m`/`l`/`c`/`re`/`S`/`f`/`B` operators)
    /// — drawing primitives that praxis does not extract.
    VectorPath,
    /// A figure marked in the structure tree (§14.7.4
    /// `/StructTreeRoot` with `/S /Figure`) but not associated with
    /// a textual alt-description.
    UnaltedFigure,
}

// ─────────────────────────────────────────────────────────────────────
// Axioms — structural invariants grounded in ISO 32000-2 sections
// (and one praxis-internal rule for image flagging).
// ─────────────────────────────────────────────────────────────────────

/// File-level structural axiom: every PDF is exactly Header ▸ Body ▸
/// CrossReferenceSection ▸ Trailer, in that order.
///
/// ISO 32000-2:2020 §7.5.1 ("File structure"): *"A PDF file shall be
/// organized as four parts: a one-line header… a body containing
/// indirect objects… a cross-reference table containing information
/// about the indirect objects in the file, and a trailer giving the
/// location of the cross-reference table and of certain special
/// objects within the body of the file."*
///
/// Structural — enforced at the type level by `PdfDocument` (Phase 2)
/// holding exactly these four fields.
pub struct FileStructureWellFormed;

impl Axiom for FileStructureWellFormed {
    fn verify(&self) -> Verdict {
        // Structural — the `PdfDocument` runtime type (Phase 2) will
        // hold the four parts as named fields, making any other shape
        // unrepresentable. The axiom records the invariant at the
        // ontology layer.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "FileStructureWellFormed",
        "every PDF is Header ▸ Body ▸ CrossReferenceSection ▸ Trailer in that order",
        "ISO 32000-2:2020 §7.5.1"
    );
}
pr4xis::register_axiom!(FileStructureWellFormed, "ISO 32000-2:2020 §7.5.1");

/// Indirect-reference resolution axiom: every `N G R` reference in
/// the document body must resolve to an indirect object present in
/// either the body or an object stream.
///
/// ISO 32000-2:2020 §7.3.10 ("Indirect objects"): *"An indirect
/// reference… shall consist of the object number of the referenced
/// indirect object, followed by white space, the generation number,
/// white space, and the keyword R."* The cross-reference section
/// (§7.5.4 / §7.5.8) maps every `(N, G)` pair to a byte offset.
/// Dangling references — pairs not in the xref — violate this axiom.
///
/// Structural — the reader (Phase 2) returns `Result<…, ReaderError>`
/// for dangling references; the typed `PdfDocument` carries only
/// resolved objects by construction.
pub struct IndirectReferencesResolve;

impl Axiom for IndirectReferencesResolve {
    fn verify(&self) -> Verdict {
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "IndirectReferencesResolve",
        "every indirect reference resolves to a known indirect object",
        "ISO 32000-2:2020 §7.3.10, §7.5.4, §7.5.8"
    );
}
pr4xis::register_axiom!(
    IndirectReferencesResolve,
    "ISO 32000-2:2020 §7.3.10, §7.5.4, §7.5.8"
);

/// Filter-chain termination axiom: every filter chain on a stream
/// terminates in interpretable bytes — the last filter's output is
/// the stream's logical content.
///
/// ISO 32000-2:2020 §7.4.1 ("General"): *"A stream filter is a
/// transformation that converts a stream's data from one form to
/// another."* Filters compose left-to-right; the supported filters
/// (FlateDecode, LZWDecode, ASCII85Decode, ASCIIHexDecode,
/// RunLengthDecode, CCITTFaxDecode, JBIG2Decode, DCTDecode,
/// JPXDecode, Crypt) are enumerated in §7.4.2 and Tables 6–8.
///
/// Structural — the reader (Phase 2) calls the registered decoder
/// for each filter in order; if any filter in the chain is
/// unsupported the reader fails closed with `UnsupportedFilter`
/// rather than silently returning encoded bytes.
pub struct FilterChainTerminates;

impl Axiom for FilterChainTerminates {
    fn verify(&self) -> Verdict {
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "FilterChainTerminates",
        "every filter chain on a stream terminates in interpretable bytes",
        "ISO 32000-2:2020 §7.4.1, §7.4.2"
    );
}
pr4xis::register_axiom!(FilterChainTerminates, "ISO 32000-2:2020 §7.4.1, §7.4.2");

/// Encoding-totality axiom: every glyph code emitted by a content
/// stream's text-showing operator (`Tj`, `TJ`, `'`, `"`) must have a
/// defined mapping to Unicode through the font's `/ToUnicode` CMap,
/// `/Encoding` dictionary, or one of the standard encodings.
///
/// ISO 32000-2:2020 §9.10.2 ("Mapping character codes to Unicode
/// values"): *"To make extraction of textual content meaningful,
/// PDF processors should map every character code… to a Unicode
/// value."* Adobe Tech Note #5014 (*ToUnicode Mapping File Tutorial*,
/// 2003) specifies the CMap format. Standard encodings
/// (PDFDocEncoding, WinAnsiEncoding, MacRomanEncoding,
/// MacExpertEncoding) are enumerated in §9.6.5 and Annex D.
///
/// Structural — the text-extractor (Phase 3) calls a per-font
/// `code_to_unicode(code) -> Option<char>`; unmapped codes are
/// reported as `FlaggedContent::UnmappedGlyph` rather than silently
/// emitted as replacement characters.
pub struct EncodingIsTotal;

impl Axiom for EncodingIsTotal {
    fn verify(&self) -> Verdict {
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EncodingIsTotal",
        "every glyph code has a defined Unicode mapping or is flagged",
        "ISO 32000-2:2020 §9.10.2, §9.6.5; Adobe Tech Note #5014"
    );
}
pr4xis::register_axiom!(
    EncodingIsTotal,
    "ISO 32000-2:2020 §9.10.2, §9.6.5; Adobe Tech Note #5014"
);

/// Image-content axiom: non-text content (image XObjects, form
/// XObjects, inline images, vector paths, unalted figures) MUST be
/// emitted as `FlaggedContent` by the extractor. Silent dropping is
/// forbidden.
///
/// Praxis-internal rule `feedback_pdf_text_only_until_image_understanding`:
/// *"PDF parsing extracts text only; images/diagrams/figures must be
/// flagged explicitly, never silently dropped or paraphrased. Waits
/// for Phase 6 image-understanding."*
///
/// Structural — the extractor's return type is `(Vec<PageText>,
/// Vec<FlaggedContent>)`; the second element is non-optional, so
/// downstream consumers cannot ignore that flagged content exists.
pub struct ImageContentMustBeFlagged;

impl Axiom for ImageContentMustBeFlagged {
    fn verify(&self) -> Verdict {
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ImageContentMustBeFlagged",
        "non-text content must be returned as FlaggedContent, never silently dropped",
        "praxis feedback_pdf_text_only_until_image_understanding (2026-04)"
    );
}
pr4xis::register_axiom!(
    ImageContentMustBeFlagged,
    "praxis feedback_pdf_text_only_until_image_understanding (2026-04)"
);

// ─────────────────────────────────────────────────────────────────────
// Quality — does a concept legitimately carry text content?
// ─────────────────────────────────────────────────────────────────────

/// Quality: which PDF concepts are text-bearing.
///
/// Used by the extractor pipeline to decide which content streams to
/// run the text-operator state machine on (every `ContentStream`
/// reached from a `Page`) and which to flag rather than parse (every
/// `ImageXObject`).
#[derive(Debug, Clone)]
pub struct IsTextBearing;

impl Quality for IsTextBearing {
    type Individual = PdfConcept;
    type Value = ();

    fn get(&self, c: &PdfConcept) -> Option<()> {
        match c {
            PdfConcept::ContentStream | PdfConcept::FormXObject | PdfConcept::StructureElement => {
                Some(())
            }
            _ => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Ontology
// ─────────────────────────────────────────────────────────────────────

/// The PDF ontology.
pub struct PdfOntology;

impl Ontology for PdfOntology {
    type Cat = PdfCategory;
    type Qual = IsTextBearing;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(FileStructureWellFormed),
            Box::new(IndirectReferencesResolve),
            Box::new(FilterChainTerminates),
            Box::new(EncodingIsTotal),
            Box::new(ImageContentMustBeFlagged),
        ]
    }
}
