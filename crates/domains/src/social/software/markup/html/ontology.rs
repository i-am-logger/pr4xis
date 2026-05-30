//! HTML5 ontology — element + attribute inventory loaded from
//! `xhtml_1_0_xsd@1.0`; content-category taxonomy from WHATWG HTML
//! Living Standard §3.2.5.
//!
//! ## Why XHTML 1.0 Strict as the grounding XSD
//!
//! XHTML 1.0 Strict is the closest reusable authoritative XSD to
//! Praxis's existing xsd-parser pipeline (M4.ε.5.a). It declares 77
//! elements + 179 attributes that are normatively shared with HTML5
//! for every name it covers (WHATWG HTML Living Standard §1.6
//! "History" — backwards-compatibility principle). HTML5-only
//! additions (canvas, video, audio, the semantic-sectioning family)
//! are queued for M4.η.1.a.
//!
//! The **element / attribute inventory** is loaded via
//! [`super::loader`] from the bundled XSD. The **content-category
//! taxonomy** is declared here because content categories are a
//! semantic overlay on top of the XSD's purely structural
//! declarations — WHATWG HTML LS §3.2.5 invented them as the
//! HTML5 successor to HTML4's `<!ENTITY % inline>` / `% block` DTD
//! parameter-entity carve-outs. The categories themselves are a
//! closed enumeration, so they live in the ontology macro; element
//! membership in each category will be loaded from the WHATWG
//! source data in M4.η.1.a (queued).
//!
//! ## Citations
//!
//! - **Pemberton et al. (eds.) (2002)** *XHTML 1.0: The Extensible
//!   HyperText Markup Language (Second Edition)*, W3C Recommendation
//!   1 August 2002. §A.1 Document Type Definitions —
//!   <https://www.w3.org/TR/xhtml1/#dtds>. The normative element /
//!   attribute inventory; the W3C-published companion XSD at
//!   <https://www.w3.org/2002/08/xhtml/xhtml1-strict.xsd> is a
//!   faithful XML Schema rendering of the §A.1.1 Strict DTD.
//! - **WHATWG (current edition)** *HTML Living Standard*.
//!   <https://html.spec.whatwg.org/>. §3.2.5 Content models;
//!   §3.2.5.1 The "nothing" content model; §3.2.5.2 Transparent
//!   content models. Content-category taxonomy.
//! - **Raggett, Le Hors & Jacobs (eds.) (1999)** *HTML 4.01
//!   Specification*, W3C Recommendation 24 December 1999. §3.2.2
//!   Names case-insensitivity. Historical baseline for case
//!   handling.
//! - **Smith, B. et al. (2005)** "Relations in biomedical
//!   ontologies", *Genome Biology* 6:R46 — OBO-RO relation-kind
//!   tagging used by the underlying `pr4xis::ontology!` macro.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// =============================================================================
// Concept inventory
// =============================================================================

pr4xis::ontology! {
    name: "Html",
    source: "Pemberton et al. (eds.) (2002) XHTML 1.0: The Extensible HyperText Markup Language (Second Edition), W3C Recommendation 1 August 2002, §A.1 Document Type Definitions; WHATWG (current edition) HTML Living Standard, §3.2.5 Content models; Raggett, Le Hors & Jacobs (eds.) (1999) HTML 4.01 Specification, W3C Recommendation 24 December 1999, §3.2.2",

    concepts: [
        HtmlComponent,
        HtmlElement,
        HtmlAttribute,
        HtmlContentCategory,
        FlowContent,
        PhrasingContent,
        MetadataContent,
        InteractiveContent,
        SectioningContent,
        EmbeddedContent,
        HeadingContent,
    ],

    labels: {
        HtmlComponent: ("en", "HTML component",
            "Top of the HTML schema-component partition: every named or structural piece of an HTML document (element name, attribute name, content-category membership) is an HtmlComponent. Pemberton et al. 2002 §A.1 (Strict DTD)."),
        HtmlElement: ("en", "HTML element",
            "A named element declared in the XHTML 1.0 Strict XSD (`<xs:element name=\"...\">`). Loaded from `xhtml_1_0_xsd@1.0` at runtime; case-insensitive per WHATWG HTML LS §13.1.2 / HTML 4.01 §3.2.2."),
        HtmlAttribute: ("en", "HTML attribute",
            "A named attribute declared in the XHTML 1.0 Strict XSD (`<xs:attribute name=\"...\">`). Loaded from `xhtml_1_0_xsd@1.0` at runtime; case-insensitive per WHATWG HTML LS §13.1.2 / HTML 4.01 §3.2.2."),
        HtmlContentCategory: ("en", "HTML content category",
            "A semantic grouping of HTML elements per WHATWG HTML LS §3.2.5. Content categories partition the element inventory along orthogonal axes (flow / phrasing / metadata / interactive / sectioning / embedded / heading) for purposes of allowed-children rules."),
        FlowContent: ("en", "Flow content",
            "WHATWG HTML LS §3.2.5.2.2: elements that can appear in the body of an HTML document. Most XHTML 1.0 Strict block + inline elements belong to this category."),
        PhrasingContent: ("en", "Phrasing content",
            "WHATWG HTML LS §3.2.5.2.5: text of an HTML document plus elements that mark up that text at the intra-paragraph level (a, span, em, strong, code, br, ...)."),
        MetadataContent: ("en", "Metadata content",
            "WHATWG HTML LS §3.2.5.2.1: elements that set the presentation or behavior of the rest of the document or otherwise carry information that doesn't render in the page itself (link, meta, title, style, script, base)."),
        InteractiveContent: ("en", "Interactive content",
            "WHATWG HTML LS §3.2.5.2.7: elements that are specifically intended for user interaction (a, button, input, select, textarea, label)."),
        SectioningContent: ("en", "Sectioning content",
            "WHATWG HTML LS §3.2.5.2.3: elements that define the scope of headings and footers (section, article, nav, aside in HTML5; not present in XHTML 1.0 Strict — populated only when the M4.η.1.a follow-up source lands)."),
        EmbeddedContent: ("en", "Embedded content",
            "WHATWG HTML LS §3.2.5.2.6: elements that import another resource into the document (img, object, picture, audio, video, iframe; XHTML 1.0 Strict covers img, object)."),
        HeadingContent: ("en", "Heading content",
            "WHATWG HTML LS §3.2.5.2.4: elements that define the heading of a section (h1, h2, h3, h4, h5, h6; HTML5 adds hgroup)."),
    },

    // is_a hierarchy:
    //   HtmlComponent (root)
    //   ├── HtmlElement      (instances loaded from XSD)
    //   ├── HtmlAttribute    (instances loaded from XSD)
    //   └── HtmlContentCategory (closed: 7 leaves per WHATWG §3.2.5)
    //       ├── FlowContent          (§3.2.5.2.2)
    //       ├── PhrasingContent      (§3.2.5.2.5)
    //       ├── MetadataContent      (§3.2.5.2.1)
    //       ├── InteractiveContent   (§3.2.5.2.7)
    //       ├── SectioningContent    (§3.2.5.2.3)
    //       ├── EmbeddedContent      (§3.2.5.2.6)
    //       └── HeadingContent       (§3.2.5.2.4)
    is_a: [
        (HtmlElement,         HtmlComponent),
        (HtmlAttribute,       HtmlComponent),
        (HtmlContentCategory, HtmlComponent),

        (FlowContent,        HtmlContentCategory),
        (PhrasingContent,    HtmlContentCategory),
        (MetadataContent,    HtmlContentCategory),
        (InteractiveContent, HtmlContentCategory),
        (SectioningContent,  HtmlContentCategory),
        (EmbeddedContent,    HtmlContentCategory),
        (HeadingContent,     HtmlContentCategory),
    ],
}

// =============================================================================
// Quality: which spec edition is the primary normative source for a concept.
// =============================================================================

/// Quality: which published spec is the primary normative source for
/// an HtmlConcept. Mirrors the XSD ontology's `PartSpec` pattern.
///
/// - **Xhtml10Strict** — Pemberton et al. 2002 (the XSD grounding
///   source).
/// - **WhatwgHtmlLs** — WHATWG HTML Living Standard (the
///   content-category semantics).
#[derive(Debug, Clone)]
pub struct SpecSource;

/// Which published HTML/XHTML spec defines a concept's primary
/// semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlSpec {
    /// Pemberton et al. (2002) XHTML 1.0 (Second Edition) §A.1.
    Xhtml10Strict,
    /// WHATWG HTML Living Standard §3.2.5.
    WhatwgHtmlLs,
}

impl Quality for SpecSource {
    type Individual = HtmlConcept;
    type Value = HtmlSpec;

    fn get(&self, c: &HtmlConcept) -> Option<HtmlSpec> {
        use HtmlConcept as H;
        match c {
            // The XSD grounding source — element / attribute names.
            H::HtmlElement | H::HtmlAttribute => Some(HtmlSpec::Xhtml10Strict),
            // WHATWG semantics — content categories.
            H::HtmlContentCategory
            | H::FlowContent
            | H::PhrasingContent
            | H::MetadataContent
            | H::InteractiveContent
            | H::SectioningContent
            | H::EmbeddedContent
            | H::HeadingContent => Some(HtmlSpec::WhatwgHtmlLs),
            // Abstract root — both specs apply.
            H::HtmlComponent => None,
        }
    }
}

// =============================================================================
// Axioms
// =============================================================================

impl Ontology for HtmlOntology {
    type Cat = HtmlCategory;
    type Qual = SpecSource;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(HtmlComponentPartitioned));
        axioms.push(Box::new(ContentCategoryClosedEnumeration));
        axioms.push(Box::new(NameCaseInsensitivity));
        axioms.push(Box::new(ElementInventoryNonEmpty));
        axioms.push(Box::new(AttributeInventoryNonEmpty));
        axioms.push(Box::new(EveryConceptHasSpecSource));
        axioms
    }
}

/// Axiom: every non-root HTML concept is `is_a`-reachable from
/// `HtmlComponent`. WHATWG HTML LS §1.6 (Compliance classes)
/// partitions every named HTML construct into typed kinds; nothing
/// is left dangling outside the partition.
pub struct HtmlComponentPartitioned;

impl Axiom for HtmlComponentPartitioned {
    fn verify(&self) -> Verdict {
        use pr4xis::category::{Arrow, Category};
        let all_morphs = HtmlCategory::morphisms();
        for v in HtmlConcept::variants() {
            if matches!(v, HtmlConcept::HtmlComponent) {
                continue;
            }
            let reaches = all_morphs.iter().any(|m| {
                m.source() == v
                    && m.target() == HtmlConcept::HtmlComponent
                    && matches!(m.kind(), HtmlRelationKind::Subsumption)
            });
            if !reaches {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "HtmlComponentPartitioned",
        "every non-root HTML concept is is_a-reachable from HtmlComponent",
        "WHATWG HTML Living Standard §1.6 (Compliance classes)"
    );
}

pr4xis::register_axiom!(
    HtmlComponentPartitioned,
    "WHATWG HTML Living Standard §1.6 (Compliance classes)"
);

/// Axiom: `HtmlContentCategory` partitions into exactly seven
/// content-category leaves per WHATWG HTML LS §3.2.5
/// (Flow / Phrasing / Metadata / Interactive / Sectioning / Embedded
/// / Heading). The enumeration is closed: WHATWG does not introduce
/// new content categories without versioning the LS itself.
pub struct ContentCategoryClosedEnumeration;

impl Axiom for ContentCategoryClosedEnumeration {
    fn verify(&self) -> Verdict {
        use pr4xis::category::{Arrow, Category};
        let count = HtmlCategory::morphisms()
            .iter()
            .filter(|m| {
                m.target() == HtmlConcept::HtmlContentCategory
                    && matches!(m.kind(), HtmlRelationKind::Subsumption)
                    && m.source() != m.target()
            })
            .count();
        if count == 7 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ContentCategoryClosedEnumeration",
        "HtmlContentCategory has exactly 7 leaves (Flow/Phrasing/Metadata/Interactive/Sectioning/Embedded/Heading)",
        "WHATWG HTML Living Standard §3.2.5"
    );
}

pr4xis::register_axiom!(
    ContentCategoryClosedEnumeration,
    "WHATWG HTML Living Standard §3.2.5"
);

/// Axiom: HTML element + attribute names are case-insensitive.
/// WHATWG HTML LS §13.1.2 (and historically W3C HTML 4.01 §3.2.2)
/// — the parser case-folds element / attribute names to lowercase
/// during parsing.
pub struct NameCaseInsensitivity;

impl Axiom for NameCaseInsensitivity {
    fn verify(&self) -> Verdict {
        // The lookup contract is documented by [`super::loader`]; this
        // axiom asserts the contract at the ontology level. Concrete
        // case-fold property is tested in `super::loader::tests`.
        use super::loader::{is_html_attribute, is_html_element};
        let element_ok = is_html_element("IMG") == is_html_element("img");
        let attribute_ok = is_html_attribute("HREF") == is_html_attribute("href");
        if element_ok && attribute_ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NameCaseInsensitivity",
        "HTML element + attribute names are case-insensitive",
        "WHATWG HTML Living Standard §13.1.2; W3C HTML 4.01 §3.2.2"
    );
}

pr4xis::register_axiom!(
    NameCaseInsensitivity,
    "WHATWG HTML Living Standard §13.1.2; W3C HTML 4.01 §3.2.2"
);

/// Axiom: the loaded element inventory is non-empty. The bundle's
/// presence is a build-time invariant (hash-pinned in praxis.lock);
/// an empty load signals a configuration regression.
pub struct ElementInventoryNonEmpty;

impl Axiom for ElementInventoryNonEmpty {
    fn verify(&self) -> Verdict {
        if super::loader::element_names().is_empty() {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        } else {
            Ok(Box::new(SimpleProof::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "ElementInventoryNonEmpty",
        "the XHTML 1.0 Strict XSD declares at least one element",
        "Pemberton et al. 2002 XHTML 1.0 §A.1 (Strict DTD)"
    );
}

pr4xis::register_axiom!(
    ElementInventoryNonEmpty,
    "Pemberton et al. 2002 XHTML 1.0 §A.1 (Strict DTD)"
);

/// Axiom: the loaded attribute inventory is non-empty.
pub struct AttributeInventoryNonEmpty;

impl Axiom for AttributeInventoryNonEmpty {
    fn verify(&self) -> Verdict {
        if super::loader::attribute_names().is_empty() {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        } else {
            Ok(Box::new(SimpleProof::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AttributeInventoryNonEmpty",
        "the XHTML 1.0 Strict XSD declares at least one attribute",
        "Pemberton et al. 2002 XHTML 1.0 §A.1 (Strict DTD)"
    );
}

pr4xis::register_axiom!(
    AttributeInventoryNonEmpty,
    "Pemberton et al. 2002 XHTML 1.0 §A.1 (Strict DTD)"
);

/// Axiom: every non-root concept has a primary spec source under
/// the `SpecSource` quality. Totality on the non-root partition.
pub struct EveryConceptHasSpecSource;

impl Axiom for EveryConceptHasSpecSource {
    fn verify(&self) -> Verdict {
        let q = SpecSource;
        for c in HtmlConcept::variants() {
            if matches!(c, HtmlConcept::HtmlComponent) {
                continue;
            }
            if q.get(&c).is_none() {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EveryConceptHasSpecSource",
        "SpecSource is total on every non-root HtmlConcept",
        "WHATWG HTML Living Standard §1.6; Pemberton et al. 2002"
    );
}

pr4xis::register_axiom!(
    EveryConceptHasSpecSource,
    "WHATWG HTML Living Standard §1.6; Pemberton et al. 2002"
);
