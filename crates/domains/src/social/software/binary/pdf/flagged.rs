//! Flagged-content walker — per-page enumeration of non-text content.
//!
//! Per `feedback_pdf_text_only_until_image_understanding`: *"PDF
//! parsing extracts text only; images/diagrams/figures must be
//! flagged explicitly, never silently dropped or paraphrased."*
//!
//! Phase 3's [`super::content_stream::walk_content_stream`] already
//! detects every graphics-flag-worthy operator inside a single
//! content stream and emits a [`super::content_stream::GraphicsEvent`].
//! This module composes that walker across a document:
//!
//! 1. Walks every page's content stream.
//! 2. Resolves each `Do <name>` operator's `<name>` against the
//!    page's `/Resources /XObject` dictionary, looks up the
//!    referenced XObject's `/Subtype`, and **reclassifies** the
//!    [`super::ontology::FlaggedKind`] correctly — Phase 3 conservatively
//!    flagged every `Do` as [`super::ontology::FlaggedKind::FormXObject`]
//!    because the operator alone doesn't carry the subtype.
//! 3. Extracts `/Width` × `/Height` from Image XObjects so the
//!    resulting [`super::ontology::FlaggedContent::dimensions`] is
//!    populated.
//! 4. Recursively descends into Form XObjects' inner content streams
//!    (with a depth bound) — Form XObjects can themselves contain
//!    text-show events and additional images.
//!
//! Out of scope for this phase:
//!
//! - Recursing the text-show events from inner Form XObject content
//!   streams back into the parent page's extracted text. That cross-
//!   layer wiring lands in Phase 6 (extraction pipeline).
//! - Walking the document's logical structure tree (§14.7) for
//!   `/Figure` elements without `/Alt` text — supported as the
//!   [`super::ontology::FlaggedKind::UnaltedFigure`] variant but
//!   the walker is forward work.
//!
//! Spec references:
//!
//! - ISO 32000-2:2020 §7.7.3.3 — *Page objects* (the `/Resources`
//!   entry that holds XObjects).
//! - §7.8.3 — *Resource dictionaries*.
//! - §8.8 — *External Objects (XObjects)*.
//! - §8.9 — *Images*.
//! - §8.10 — *Form XObjects*.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::content_stream::{ContentStreamError, GraphicsEvent, walk_content_stream};
use super::ontology::{FlaggedContent, FlaggedKind};
use super::reader::PdfDocument;

/// Why the flagged-content walk failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlagError {
    /// Requested page number isn't in the document.
    PageOutOfRange { page: u32, total: usize },
    /// Page's content stream couldn't be retrieved or decoded.
    UnreadableContentStream { page: u32, detail: String },
    /// `Do <name>` operator references an XObject that couldn't
    /// be resolved against the page's resource dictionary.
    XObjectResolutionFailed {
        page: u32,
        name: String,
        detail: String,
    },
}

impl core::fmt::Display for FlagError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::PageOutOfRange { page, total } => {
                write!(f, "page {page} out of range (document has {total} pages)")
            }
            Self::UnreadableContentStream { page, detail } => {
                write!(f, "content stream on page {page} unreadable: {detail}")
            }
            Self::XObjectResolutionFailed { page, name, detail } => write!(
                f,
                "XObject {name:?} referenced from page {page} couldn't be resolved: {detail}"
            ),
        }
    }
}

impl std::error::Error for FlagError {}

/// Maximum recursion depth for nested Form XObjects. PDF doesn't
/// require Form XObject DAGs to be acyclic; we cap the walk so a
/// pathological input can't blow the stack.
const MAX_FORM_DEPTH: u8 = 8;

// ─────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────

/// Walk every page in the document, returning every flagged piece
/// of non-text content found. Stable order: page 1 first, then
/// page 2, etc.; within a page, content-stream-order is preserved
/// (with reclassification not changing order).
pub fn flag_document(doc: &PdfDocument) -> Result<Vec<FlaggedContent>, FlagError> {
    let pages = doc.inner().get_pages();
    let mut out = Vec::new();
    for &page_num in pages.keys() {
        out.extend(flag_page(doc, page_num)?);
    }
    Ok(out)
}

/// Walk a single page (1-indexed). See module docs for the
/// classification semantics.
pub fn flag_page(doc: &PdfDocument, page_num: u32) -> Result<Vec<FlaggedContent>, FlagError> {
    let pages = doc.inner().get_pages();
    let total = pages.len();
    let page_id = *pages.get(&page_num).ok_or(FlagError::PageOutOfRange {
        page: page_num,
        total,
    })?;

    let bytes =
        doc.inner()
            .get_page_content(page_id)
            .map_err(|e| FlagError::UnreadableContentStream {
                page: page_num,
                detail: format!("{e}"),
            })?;
    let walk = walk_content_stream(&bytes).map_err(|e| match e {
        ContentStreamError::Malformed { detail } => FlagError::UnreadableContentStream {
            page: page_num,
            detail,
        },
    })?;

    let mut out = Vec::new();
    for event in &walk.graphics_events {
        reclassify_and_emit(doc, page_num, event, 0, &mut out)?;
    }
    Ok(out)
}

// ─────────────────────────────────────────────────────────────────────
// Internal — reclassification + recursion
// ─────────────────────────────────────────────────────────────────────

fn reclassify_and_emit(
    doc: &PdfDocument,
    page_num: u32,
    event: &GraphicsEvent,
    depth: u8,
    out: &mut Vec<FlaggedContent>,
) -> Result<(), FlagError> {
    match event.kind {
        // Inline images and vector-path painting don't need
        // resource resolution — Phase 3 classified them correctly.
        FlaggedKind::InlineImage | FlaggedKind::VectorPath => {
            out.push(FlaggedContent {
                kind: event.kind,
                page: page_num,
                object: None,
                dimensions: None,
                note: format!("operator {} in content stream", event.operator),
            });
            Ok(())
        }
        // `Do <name>` — resolve the XObject against page resources
        // to learn whether it's an Image or a Form, and pull
        // dimensions from /Width × /Height when present.
        FlaggedKind::FormXObject => {
            let resolution = resolve_xobject(doc, page_num, &event.detail).map_err(|detail| {
                FlagError::XObjectResolutionFailed {
                    page: page_num,
                    name: event.detail.clone(),
                    detail,
                }
            })?;
            match resolution {
                ResolvedXObject::Image {
                    object_id,
                    width,
                    height,
                } => {
                    out.push(FlaggedContent {
                        kind: FlaggedKind::ImageXObject,
                        page: page_num,
                        object: Some(object_id),
                        dimensions: width.zip(height).map(|(w, h)| (w as f32, h as f32)),
                        note: format!("/XObject {} → /Subtype /Image", event.detail),
                    });
                }
                ResolvedXObject::Form {
                    object_id,
                    inner_stream,
                } => {
                    out.push(FlaggedContent {
                        kind: FlaggedKind::FormXObject,
                        page: page_num,
                        object: Some(object_id),
                        dimensions: None,
                        note: format!("/XObject {} → /Subtype /Form", event.detail),
                    });
                    // Recurse into the form's inner content stream
                    // (bounded depth) to surface nested images.
                    if depth + 1 < MAX_FORM_DEPTH
                        && !inner_stream.is_empty()
                        && let Ok(inner_walk) = walk_content_stream(&inner_stream)
                    {
                        for nested in &inner_walk.graphics_events {
                            reclassify_and_emit(doc, page_num, nested, depth + 1, out)?;
                        }
                    }
                }
                ResolvedXObject::UnknownSubtype { object_id, subtype } => {
                    // Fail-flagged, not fail-error: we recognized
                    // the resource but its /Subtype isn't Image or
                    // Form. Surface as FormXObject (the more
                    // permissive of the two) with a descriptive note.
                    out.push(FlaggedContent {
                        kind: FlaggedKind::FormXObject,
                        page: page_num,
                        object: Some(object_id),
                        dimensions: None,
                        note: format!(
                            "/XObject {} has unrecognized /Subtype {:?}",
                            event.detail, subtype
                        ),
                    });
                }
            }
            Ok(())
        }
        // ImageXObject won't appear as an input event from Phase 3
        // (it always emits FormXObject for `Do`); UnaltedFigure
        // comes from the structure tree, not the content stream,
        // and isn't surfaced here. Both cases are passthrough.
        FlaggedKind::ImageXObject | FlaggedKind::UnaltedFigure => {
            out.push(FlaggedContent {
                kind: event.kind,
                page: page_num,
                object: None,
                dimensions: None,
                note: event.detail.clone(),
            });
            Ok(())
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// XObject resolution
// ─────────────────────────────────────────────────────────────────────

enum ResolvedXObject {
    Image {
        object_id: (u32, u16),
        width: Option<i64>,
        height: Option<i64>,
    },
    Form {
        object_id: (u32, u16),
        inner_stream: Vec<u8>,
    },
    UnknownSubtype {
        object_id: (u32, u16),
        subtype: String,
    },
}

/// Resolve `Do <name>` against the page's resource dictionary,
/// then look up the referenced XObject and classify it.
fn resolve_xobject(
    doc: &PdfDocument,
    page_num: u32,
    name: &str,
) -> Result<ResolvedXObject, String> {
    let pages = doc.inner().get_pages();
    let page_id = pages
        .get(&page_num)
        .ok_or_else(|| format!("page {page_num} not in document"))?;
    let resources_dict = page_resources(doc, *page_id)?;

    let xobject_dict = resources_dict
        .get(b"XObject")
        .map_err(|_| "page /Resources has no /XObject".to_string())?;
    let xobject_dict = deref_dict(doc, xobject_dict)?;

    let entry = xobject_dict
        .get(name.as_bytes())
        .map_err(|_| format!("/XObject has no entry {name:?}"))?;

    let (object_id, stream) = deref_stream_with_id(doc, entry)?;

    let subtype = stream
        .dict
        .get(b"Subtype")
        .ok()
        .and_then(|o| o.as_name().ok())
        .map(|n| String::from_utf8_lossy(n).into_owned())
        .unwrap_or_default();

    match subtype.as_str() {
        "Image" => {
            let width = stream.dict.get(b"Width").ok().and_then(|o| o.as_i64().ok());
            let height = stream
                .dict
                .get(b"Height")
                .ok()
                .and_then(|o| o.as_i64().ok());
            Ok(ResolvedXObject::Image {
                object_id,
                width,
                height,
            })
        }
        "Form" => {
            let inner_stream = stream
                .decompressed_content()
                .unwrap_or_else(|_| stream.content.clone());
            Ok(ResolvedXObject::Form {
                object_id,
                inner_stream,
            })
        }
        _ => Ok(ResolvedXObject::UnknownSubtype { object_id, subtype }),
    }
}

/// Walk up to the page's `/Resources` dictionary, honoring the
/// PDF inheritance rule (a page without `/Resources` inherits from
/// its parent page-tree node per §7.7.3.4).
fn page_resources(doc: &PdfDocument, page_id: (u32, u16)) -> Result<&lopdf::Dictionary, String> {
    let mut cursor = page_id;
    loop {
        let page = doc.inner().get_object(cursor).map_err(|e| format!("{e}"))?;
        let dict = page.as_dict().map_err(|e| format!("{e}"))?;
        if let Ok(res) = dict.get(b"Resources") {
            return deref_dict(doc, res);
        }
        if let Ok(parent) = dict.get(b"Parent")
            && let Ok(parent_ref) = parent.as_reference()
        {
            cursor = parent_ref;
            continue;
        }
        return Err("walked to page-tree root without finding /Resources".to_string());
    }
}

fn deref_dict<'a>(
    doc: &'a PdfDocument,
    obj: &'a lopdf::Object,
) -> Result<&'a lopdf::Dictionary, String> {
    let resolved = match obj {
        lopdf::Object::Reference(id) => doc.inner().get_object(*id).map_err(|e| format!("{e}"))?,
        other => other,
    };
    resolved.as_dict().map_err(|e| format!("{e}"))
}

fn deref_stream_with_id<'a>(
    doc: &'a PdfDocument,
    obj: &'a lopdf::Object,
) -> Result<((u32, u16), &'a lopdf::Stream), String> {
    let (id, resolved) = match obj {
        lopdf::Object::Reference(id) => (
            *id,
            doc.inner().get_object(*id).map_err(|e| format!("{e}"))?,
        ),
        // Inline (non-indirect) stream — no addressable id.
        other => ((0, 0), other),
    };
    let stream = resolved.as_stream().map_err(|e| format!("{e}"))?;
    Ok((id, stream))
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::reader::read_pdf_bytes;
    use super::*;
    use lopdf::{Document, Object, Stream, dictionary};

    /// Build a synthetic 1-page PDF whose page references the
    /// given XObject (subtype = `Image` or `Form`). Returns the
    /// serialized bytes.
    fn pdf_with_xobject(subtype: &str, content_stream_ops: &[u8]) -> Vec<u8> {
        let mut doc = Document::with_version("1.4");

        // The XObject itself.
        let mut xobj_dict = dictionary! {
            "Type" => "XObject",
            "Subtype" => subtype,
        };
        if subtype == "Image" {
            xobj_dict.set("Width", 240);
            xobj_dict.set("Height", 180);
            xobj_dict.set("ColorSpace", "DeviceRGB");
            xobj_dict.set("BitsPerComponent", 8);
        } else if subtype == "Form" {
            xobj_dict.set("BBox", vec![0.into(), 0.into(), 100.into(), 100.into()]);
        }
        let xobj_id = doc.add_object(Stream::new(xobj_dict, b"".to_vec()));

        // Page content stream that invokes the XObject.
        let content_id = doc.add_object(Stream::new(dictionary! {}, content_stream_ops.to_vec()));

        // Pages tree + single page.
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
            "Resources" => dictionary! {
                "XObject" => dictionary! { "Im0" => xobj_id },
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

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn do_referencing_image_xobject_reclassifies_to_image() {
        let bytes = pdf_with_xobject("Image", b"/Im0 Do\n");
        let doc = read_pdf_bytes(&bytes).expect("parse");
        let flagged = flag_document(&doc).expect("flag");
        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].kind, FlaggedKind::ImageXObject);
        assert_eq!(flagged[0].page, 1);
        assert!(flagged[0].object.is_some());
        assert_eq!(flagged[0].dimensions, Some((240.0, 180.0)));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn do_referencing_form_xobject_keeps_form_classification() {
        let bytes = pdf_with_xobject("Form", b"/Im0 Do\n");
        let doc = read_pdf_bytes(&bytes).expect("parse");
        let flagged = flag_document(&doc).expect("flag");
        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].kind, FlaggedKind::FormXObject);
        assert!(flagged[0].note.contains("/Form"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn vector_path_painting_flagged() {
        // Build a page with a fill-rectangle op. No XObject ref.
        let mut doc = Document::with_version("1.4");
        let content_id = doc.add_object(Stream::new(
            dictionary! {},
            b"100 100 50 50 re\nf\n".to_vec(),
        ));
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
        let flagged = flag_document(&parsed).expect("flag");
        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].kind, FlaggedKind::VectorPath);
        assert!(flagged[0].note.contains("operator f"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn page_with_only_text_yields_empty_flagged_list() {
        // Page with a text-only content stream, no /Do, no /re.
        let mut doc = Document::with_version("1.4");
        let content_id = doc.add_object(Stream::new(
            dictionary! {},
            b"BT\n/F1 12 Tf\n(Hello) Tj\nET\n".to_vec(),
        ));
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
        let flagged = flag_document(&parsed).expect("flag");
        assert!(flagged.is_empty());
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn page_out_of_range_returns_named_error() {
        let bytes = pdf_with_xobject("Image", b"");
        let doc = read_pdf_bytes(&bytes).expect("parse");
        let err = flag_page(&doc, 99).unwrap_err();
        match err {
            FlagError::PageOutOfRange { page, total } => {
                assert_eq!(page, 99);
                assert_eq!(total, 1);
            }
            other => panic!("expected PageOutOfRange; got {other:?}"),
        }
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn flag_document_is_deterministic() {
        let bytes = pdf_with_xobject("Image", b"/Im0 Do\n100 100 50 50 re\nf\n/Im0 Do\n");
        let doc1 = read_pdf_bytes(&bytes).expect("parse");
        let doc2 = read_pdf_bytes(&bytes).expect("parse");
        let f1 = flag_document(&doc1).expect("flag");
        let f2 = flag_document(&doc2).expect("flag");
        assert_eq!(f1.len(), f2.len());
        for (a, b) in f1.iter().zip(f2.iter()) {
            assert_eq!(a.kind, b.kind);
            assert_eq!(a.page, b.page);
            assert_eq!(a.dimensions, b.dimensions);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn image_dimensions_are_captured_from_xobject_dict() {
        let bytes = pdf_with_xobject("Image", b"/Im0 Do\n");
        let doc = read_pdf_bytes(&bytes).expect("parse");
        let flagged = flag_document(&doc).expect("flag");
        // Test fixture sets Width=240, Height=180.
        assert_eq!(flagged[0].dimensions, Some((240.0, 180.0)));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn unknown_subtype_is_flagged_with_descriptive_note() {
        let bytes = pdf_with_xobject("PS", b"/Im0 Do\n");
        let doc = read_pdf_bytes(&bytes).expect("parse");
        let flagged = flag_document(&doc).expect("flag");
        assert_eq!(flagged.len(), 1);
        assert!(flagged[0].note.contains("unrecognized"));
        assert!(flagged[0].note.contains("\"PS\""));
    }

    // ── Adversarial fixtures ─────────────────────────────────────

    /// `Do <name>` referencing an XObject name that doesn't exist
    /// in `/Resources /XObject` must return the typed
    /// `XObjectResolutionFailed` error, never silently emit an
    /// empty flag or panic.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn do_with_missing_xobject_resource_returns_named_error() {
        // Build a page whose content stream invokes /Missing but
        // whose Resources/XObject dict doesn't carry that name.
        use lopdf::{Document, Object, Stream, dictionary};
        let mut doc = Document::with_version("1.4");
        // The /XObject dict is present but only carries /Other,
        // not /Missing.
        let other_xobj = doc.add_object(Stream::new(
            dictionary! { "Type" => "XObject", "Subtype" => "Image",
            "Width" => 1, "Height" => 1,
            "ColorSpace" => "DeviceGray", "BitsPerComponent" => 8, },
            b"".to_vec(),
        ));
        let content_id = doc.add_object(Stream::new(dictionary! {}, b"/Missing Do\n".to_vec()));
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
            "Resources" => dictionary! {
                "XObject" => dictionary! { "Other" => other_xobj },
            },
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog);
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).expect("serialize");

        let parsed = read_pdf_bytes(&bytes).expect("parse");
        match flag_document(&parsed) {
            Err(FlagError::XObjectResolutionFailed { name, .. }) => {
                assert_eq!(name, "Missing");
            }
            other => panic!("expected XObjectResolutionFailed; got {other:?}"),
        }
    }

    /// A page with `/Resources` but no `/XObject` dictionary, and
    /// a content stream that doesn't reference any XObjects, must
    /// flag without panicking — the missing dict is a normal
    /// case for text-only pages.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn page_with_resources_but_no_xobject_dict_handles_gracefully() {
        use lopdf::{Document, Object, Stream, dictionary};
        let mut doc = Document::with_version("1.4");
        let content_id = doc.add_object(Stream::new(
            dictionary! {},
            b"100 100 50 50 re\nf\n".to_vec(),
        ));
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => content_id,
            // /Resources present but no /XObject.
            "Resources" => dictionary! { "Font" => dictionary! {} },
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog);
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).expect("serialize");

        let parsed = read_pdf_bytes(&bytes).expect("parse");
        // Vector path is flagged; no XObject lookup attempted.
        let flagged = flag_document(&parsed).expect("flag");
        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].kind, FlaggedKind::VectorPath);
    }

    // ── Property-based ────────────────────────────────────────────

    use proptest::prelude::*;

    proptest! {
        /// flag_document is deterministic — same input bytes yield
        /// identical output across repeated parses, regardless of
        /// how many graphics operators (vector fills) the input
        /// content stream contains.
        #[test]
        fn prop_flag_is_deterministic_across_fill_counts(n in 0u32..16) {
            let mut ops = String::new();
            for i in 0..n {
                ops.push_str(&format!("{} 0 10 10 re\nf\n", i * 20));
            }
            let bytes = pdf_with_xobject("Image", ops.as_bytes());
            let d1 = read_pdf_bytes(&bytes).expect("parse 1");
            let d2 = read_pdf_bytes(&bytes).expect("parse 2");
            let f1 = flag_document(&d1).expect("flag 1");
            let f2 = flag_document(&d2).expect("flag 2");
            prop_assert_eq!(f1.len(), f2.len());
            for (a, b) in f1.iter().zip(f2.iter()) {
                prop_assert_eq!(a.kind, b.kind);
                prop_assert_eq!(a.page, b.page);
                prop_assert_eq!(a.dimensions, b.dimensions);
            }
        }

        /// flag_document's output cardinality equals the number of
        /// painting operators in the input — every fill produces
        /// exactly one VectorPath flag. Crosses Phase 3 + Phase 5
        /// composition: no events are silently dropped or
        /// duplicated.
        #[test]
        fn prop_flag_count_equals_painting_op_count(n in 0u32..16) {
            let mut ops = String::new();
            for i in 0..n {
                ops.push_str(&format!("{} 0 10 10 re\nf\n", i * 20));
            }
            let bytes = pdf_with_xobject("Form", ops.as_bytes());
            let doc = read_pdf_bytes(&bytes).expect("parse");
            let flagged = flag_document(&doc).expect("flag");
            // No /Do invocations in the content stream, so the
            // only flagged events are vector paths.
            prop_assert_eq!(flagged.len() as u32, n);
            for f in &flagged {
                prop_assert_eq!(f.kind, FlaggedKind::VectorPath);
            }
        }

        /// Image dimensions round-trip from the XObject dictionary
        /// through resolution. (Bounds chosen to fit PDF user units
        /// and to keep the test fast.)
        #[test]
        fn prop_image_dimensions_round_trip(w in 1i64..2000, h in 1i64..2000) {
            use lopdf::{Document, Object, Stream, dictionary};
            let mut doc = Document::with_version("1.4");
            let img_id = doc.add_object(Stream::new(
                dictionary! {
                    "Type" => "XObject",
                    "Subtype" => "Image",
                    "Width" => w,
                    "Height" => h,
                    "ColorSpace" => "DeviceGray",
                    "BitsPerComponent" => 8,
                },
                b"".to_vec(),
            ));
            let content_id = doc.add_object(Stream::new(dictionary! {}, b"/Im0 Do\n".to_vec()));
            let pages_id = doc.new_object_id();
            let page_id = doc.add_object(dictionary! {
                "Type" => "Page",
                "Parent" => pages_id,
                "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
                "Contents" => content_id,
                "Resources" => dictionary! {
                    "XObject" => dictionary! { "Im0" => img_id },
                },
            });
            let pages = dictionary! {
                "Type" => "Pages",
                "Kids" => vec![page_id.into()],
                "Count" => 1,
            };
            doc.objects.insert(pages_id, Object::Dictionary(pages));
            let catalog = doc.add_object(dictionary! {
                "Type" => "Catalog",
                "Pages" => pages_id,
            });
            doc.trailer.set("Root", catalog);
            let mut bytes = Vec::new();
            doc.save_to(&mut bytes).expect("serialize");

            let parsed = read_pdf_bytes(&bytes).expect("parse");
            let flagged = flag_document(&parsed).expect("flag");
            prop_assert_eq!(flagged.len(), 1);
            prop_assert_eq!(flagged[0].kind, FlaggedKind::ImageXObject);
            prop_assert_eq!(flagged[0].dimensions, Some((w as f32, h as f32)));
        }

        /// Every flagged event from flag_document has page ∈ [1, N]
        /// where N is the document's page count. No off-by-one
        /// indexing, no zero-indexed leaks.
        #[test]
        fn prop_flagged_page_numbers_are_one_indexed(n in 1u32..8) {
            use lopdf::{Document, Object, Stream, dictionary};
            let mut doc = Document::with_version("1.4");
            let pages_id = doc.new_object_id();
            let mut kids = Vec::new();
            for _ in 0..n {
                let content_id = doc.add_object(Stream::new(
                    dictionary! {},
                    b"0 0 10 10 re\nf\n".to_vec(),
                ));
                let p = doc.add_object(dictionary! {
                    "Type" => "Page",
                    "Parent" => pages_id,
                    "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
                    "Contents" => content_id,
                    "Resources" => dictionary! {},
                });
                kids.push(Object::Reference(p));
            }
            let pages = dictionary! {
                "Type" => "Pages",
                "Kids" => kids,
                "Count" => n as i64,
            };
            doc.objects.insert(pages_id, Object::Dictionary(pages));
            let catalog = doc.add_object(dictionary! {
                "Type" => "Catalog",
                "Pages" => pages_id,
            });
            doc.trailer.set("Root", catalog);
            let mut bytes = Vec::new();
            doc.save_to(&mut bytes).expect("serialize");

            let parsed = read_pdf_bytes(&bytes).expect("parse");
            let flagged = flag_document(&parsed).expect("flag");
            prop_assert_eq!(flagged.len(), n as usize);
            for f in &flagged {
                prop_assert!(f.page >= 1, "page index must be 1-indexed");
                prop_assert!(f.page <= n, "page must not exceed page count");
            }
        }
    }

    pr4xis::register_praxis_value!(prop_flag_is_deterministic_across_fill_counts, Deterministic);
    pr4xis::register_praxis_value!(prop_flag_count_equals_painting_op_count, Verifiable);
    pr4xis::register_praxis_value!(prop_image_dimensions_round_trip, Deterministic);
    pr4xis::register_praxis_value!(prop_flagged_page_numbers_are_one_indexed, Verifiable);

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn multiple_pages_walked_in_order() {
        // Build a 2-page doc; page 1 has an image, page 2 has a
        // vector-fill rectangle.
        let mut doc = Document::with_version("1.4");
        // Image XObject (shared).
        let img_id = doc.add_object(Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => 10,
                "Height" => 10,
                "ColorSpace" => "DeviceGray",
                "BitsPerComponent" => 8,
            },
            b"".to_vec(),
        ));
        let p1_content = doc.add_object(Stream::new(dictionary! {}, b"/Im0 Do\n".to_vec()));
        let p2_content = doc.add_object(Stream::new(dictionary! {}, b"0 0 50 50 re\nf\n".to_vec()));
        let pages_id = doc.new_object_id();
        let p1 = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => p1_content,
            "Resources" => dictionary! {
                "XObject" => dictionary! { "Im0" => img_id },
            },
        });
        let p2 = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
            "Contents" => p2_content,
            "Resources" => dictionary! {},
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![p1.into(), p2.into()],
            "Count" => 2,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog);

        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).expect("serialize");

        let parsed = read_pdf_bytes(&bytes).expect("parse");
        let flagged = flag_document(&parsed).expect("flag");

        // Two events total — image on page 1, vector path on page 2.
        assert_eq!(flagged.len(), 2);
        assert_eq!(flagged[0].page, 1);
        assert_eq!(flagged[0].kind, FlaggedKind::ImageXObject);
        assert_eq!(flagged[1].page, 2);
        assert_eq!(flagged[1].kind, FlaggedKind::VectorPath);
    }
}
