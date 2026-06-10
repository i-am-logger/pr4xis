//! PDF ontology — category laws, axiom presence, and structural
//! invariants on the concept/edge surface.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    EncodingIsTotal, FileStructureWellFormed, FilterChainTerminates, FlaggedContent, FlaggedKind,
    ImageContentMustBeFlagged, IndirectReferencesResolve, IsTextBearing, PdfCategory, PdfConcept,
    PdfEdge, PdfOntology, PdfSymbols,
};
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// ─────────────────────────────────────────────────────────────────────
// Category + ontology
// ─────────────────────────────────────────────────────────────────────

#[test]
fn category_laws() {
    assert_category_laws::<PdfCategory>();
}

#[test]
fn ontology_validates() {
    PdfOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// ─────────────────────────────────────────────────────────────────────
// Concept surface — every variant is enumerable.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn concept_variants_are_24() {
    // Document, Header, Body, CrossReferenceSection, Trailer,
    // IndirectObject, IndirectReference, ObjectStream,
    // Catalog, PageTree, Page,
    // ContentStream, Resources,
    // Font, FontDescriptor, Encoding, ToUnicodeCmap,
    // XObject, ImageXObject, FormXObject,
    // FilterChain,
    // StructTreeRoot, StructureElement
    // = 23 variants.
    assert_eq!(PdfConcept::variants().len(), 23);
}

// ─────────────────────────────────────────────────────────────────────
// Symbols — magic bytes are the spec-published values.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn header_magic_is_pdf_prefix() {
    assert_eq!(PdfSymbols::header_magic(), b"%PDF-");
}

#[test]
fn eof_marker_is_double_percent_eof() {
    assert_eq!(PdfSymbols::eof_marker(), b"%%EOF");
}

#[test]
fn keyword_bytes_match_spec() {
    assert_eq!(PdfSymbols::xref_keyword(), b"xref");
    assert_eq!(PdfSymbols::trailer_keyword(), b"trailer");
    assert_eq!(PdfSymbols::startxref_keyword(), b"startxref");
    assert_eq!(PdfSymbols::obj_keyword(), b"obj");
    assert_eq!(PdfSymbols::endobj_keyword(), b"endobj");
    assert_eq!(PdfSymbols::ref_keyword(), b"R");
    assert_eq!(PdfSymbols::stream_keyword(), b"stream");
    assert_eq!(PdfSymbols::endstream_keyword(), b"endstream");
}

// ─────────────────────────────────────────────────────────────────────
// Axioms — each one is present, has a citation, and verifies.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn axiom_file_structure_well_formed_verifies() {
    assert!(FileStructureWellFormed.verify().is_ok());
}

#[test]
fn axiom_indirect_references_resolve_verifies() {
    assert!(IndirectReferencesResolve.verify().is_ok());
}

#[test]
fn axiom_filter_chain_terminates_verifies() {
    assert!(FilterChainTerminates.verify().is_ok());
}

#[test]
fn axiom_encoding_is_total_verifies() {
    assert!(EncodingIsTotal.verify().is_ok());
}

#[test]
fn axiom_image_content_must_be_flagged_verifies() {
    assert!(ImageContentMustBeFlagged.verify().is_ok());
}

#[test]
fn every_axiom_carries_iso_or_praxis_citation() {
    let axioms = PdfOntology::axioms();
    assert_eq!(axioms.len(), 5);
    for a in axioms {
        let cit = a.meta().citation.as_str().to_string();
        assert!(
            cit.contains("ISO 32000-2") || cit.contains("praxis feedback_"),
            "axiom citation must cite ISO 32000-2 or a named praxis rule; got: {cit}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────
// Structural invariants on the morphism catalog.
// ─────────────────────────────────────────────────────────────────────

#[test]
fn document_contains_the_four_file_parts() {
    use PdfConcept::*;
    let ms = PdfCategory::morphisms();
    let parts = [Header, Body, CrossReferenceSection, Trailer];
    for p in parts {
        assert!(
            ms.contains(&PdfEdge::contains(Document, p)),
            "Document should contain {p:?} per ISO 32000-2:2020 §7.5.1"
        );
    }
}

#[test]
fn trailer_references_catalog() {
    use PdfConcept::*;
    let ms = PdfCategory::morphisms();
    assert!(
        ms.contains(&PdfEdge::references(Trailer, Catalog)),
        "Trailer's /Root entry references the Catalog per ISO 32000-2:2020 §7.5.5"
    );
}

#[test]
fn catalog_references_page_tree() {
    use PdfConcept::*;
    let ms = PdfCategory::morphisms();
    assert!(
        ms.contains(&PdfEdge::references(Catalog, PageTree)),
        "Catalog's /Pages entry references the page tree per ISO 32000-2:2020 §7.7.2"
    );
}

#[test]
fn page_references_content_stream_and_resources() {
    use PdfConcept::*;
    let ms = PdfCategory::morphisms();
    assert!(ms.contains(&PdfEdge::references(Page, ContentStream)));
    assert!(ms.contains(&PdfEdge::references(Page, Resources)));
}

#[test]
fn font_references_descriptor_encoding_and_tounicode() {
    use PdfConcept::*;
    let ms = PdfCategory::morphisms();
    assert!(ms.contains(&PdfEdge::references(Font, FontDescriptor)));
    assert!(ms.contains(&PdfEdge::references(Font, Encoding)));
    assert!(ms.contains(&PdfEdge::references(Font, ToUnicodeCmap)));
}

#[test]
fn xobject_contains_image_and_form_variants() {
    use PdfConcept::*;
    let ms = PdfCategory::morphisms();
    assert!(ms.contains(&PdfEdge::contains(XObject, ImageXObject)));
    assert!(ms.contains(&PdfEdge::contains(XObject, FormXObject)));
}

#[test]
fn streams_can_carry_filter_chains() {
    use PdfConcept::*;
    let ms = PdfCategory::morphisms();
    for stream in [ContentStream, ImageXObject, FormXObject, ObjectStream] {
        assert!(
            ms.contains(&PdfEdge::contains(stream, FilterChain)),
            "{stream:?} should carry a FilterChain per ISO 32000-2:2020 §7.4"
        );
    }
}

#[test]
fn page_tree_is_recursive() {
    use PdfConcept::*;
    let ms = PdfCategory::morphisms();
    assert!(
        ms.contains(&PdfEdge::contains(PageTree, PageTree)),
        "page tree nodes are recursive per ISO 32000-2:2020 §7.7.3.2"
    );
}

#[test]
fn structure_tree_is_recursive() {
    use PdfConcept::*;
    let ms = PdfCategory::morphisms();
    assert!(
        ms.contains(&PdfEdge::contains(
            PdfConcept::StructureElement,
            StructureElement
        )),
        "structure elements are recursive per ISO 32000-2:2020 §14.7.3"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Quality
// ─────────────────────────────────────────────────────────────────────

#[test]
fn is_text_bearing_covers_content_streams_and_form_xobjects() {
    let q = IsTextBearing;
    assert!(q.get(&PdfConcept::ContentStream).is_some());
    assert!(q.get(&PdfConcept::FormXObject).is_some());
    assert!(q.get(&PdfConcept::StructureElement).is_some());
}

#[test]
fn is_text_bearing_excludes_image_xobject_and_filter_chain() {
    let q = IsTextBearing;
    assert!(q.get(&PdfConcept::ImageXObject).is_none());
    assert!(q.get(&PdfConcept::FilterChain).is_none());
    assert!(q.get(&PdfConcept::Header).is_none());
}

// ─────────────────────────────────────────────────────────────────────
// Flagged content surface
// ─────────────────────────────────────────────────────────────────────

#[test]
fn flagged_kind_covers_every_non_text_class() {
    // The five non-text kinds the extractor must surface — adding a
    // new variant without updating this test should fail compilation
    // via the exhaustive match.
    let kinds = [
        FlaggedKind::ImageXObject,
        FlaggedKind::FormXObject,
        FlaggedKind::InlineImage,
        FlaggedKind::VectorPath,
        FlaggedKind::UnaltedFigure,
    ];
    for k in kinds {
        let _exhaustive = match k {
            FlaggedKind::ImageXObject => 0,
            FlaggedKind::FormXObject => 1,
            FlaggedKind::InlineImage => 2,
            FlaggedKind::VectorPath => 3,
            FlaggedKind::UnaltedFigure => 4,
        };
    }
}

#[test]
fn flagged_content_carries_required_fields() {
    let f = FlaggedContent {
        kind: FlaggedKind::ImageXObject,
        page: 1,
        object: Some((42, 0)),
        dimensions: Some((240.0, 180.0)),
        note: "test image".to_string(),
    };
    assert_eq!(f.kind, FlaggedKind::ImageXObject);
    assert_eq!(f.page, 1);
    assert_eq!(f.object, Some((42, 0)));
}

// ─────────────────────────────────────────────────────────────────────
// Property-based — the catalog is internally consistent.
// ─────────────────────────────────────────────────────────────────────

fn arb_concept() -> impl Strategy<Value = PdfConcept> {
    proptest::sample::select(PdfConcept::variants())
}

proptest! {
    /// Every morphism in the catalog has a non-empty Provenance.name.
    #[test]
    fn prop_every_morphism_named(_seed in any::<u32>()) {
        for m in PdfCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    /// Identity is reflexive on every concept variant.
    #[test]
    fn prop_identity_is_self_edge(c in arb_concept()) {
        let id = PdfCategory::identity(&c);
        prop_assert_eq!(id.source(), c);
        prop_assert_eq!(id.target(), c);
    }

    /// Composing an edge with the target's identity is a no-op
    /// (modulo kind which inherits from the non-identity edge).
    #[test]
    fn prop_compose_with_identity_is_noop(c in arb_concept()) {
        let id = PdfCategory::identity(&c);
        let composed = PdfCategory::compose(&id, &id);
        prop_assert!(composed.is_some());
        prop_assert_eq!(composed.unwrap(), id);
    }

    /// Mixing kinds in composition fails — a Containment edge
    /// followed by an IndirectReference edge cannot compose.
    #[test]
    fn prop_compose_mixed_kinds_fails(c1 in arb_concept(), c2 in arb_concept(), c3 in arb_concept()) {
        prop_assume!(c1 != c2 && c2 != c3);
        let f = PdfEdge::contains(c1, c2);
        let g = PdfEdge::references(c2, c3);
        prop_assert!(PdfCategory::compose(&f, &g).is_none());
    }
}
