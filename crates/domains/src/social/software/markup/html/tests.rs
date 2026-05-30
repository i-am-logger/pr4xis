//! Three-layer test stack for the HTML5 ontology:
//!
//! 1. **Axioms (cited)** — every claim about the loaded inventory
//!    has an explicit citation tag.
//! 2. **Property tests (proptest)** — case-fold idempotence,
//!    load-idempotence, round-trip element-name ↔ loaded-set.
//! 3. **Functor laws** — `assert_category_laws` on `HtmlCategory`.

use super::loader::{
    XHTML_1_0_STRICT_XSD, attribute_names, element_names, is_html_attribute, is_html_element,
};
use super::ontology::{HtmlCategory, HtmlConcept};
use pr4xis::category::Concept;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::Ontology;
use proptest::prelude::*;

// =============================================================================
// Axiom tests (cited)
// =============================================================================

/// Every XHTML 1.0 Strict element from Pemberton et al. 2002 §A.1
/// appears in the loaded ontology's element set. Spot-check uses
/// the elements named in the schema's `<xs:element name=...>`
/// declarations.
#[test]
fn axiom_xhtml_1_0_strict_elements_in_loaded_inventory() {
    // Per Pemberton et al. 2002 §A.1, XHTML 1.0 Strict defines 77
    // top-level elements. The full list, verified from
    // www.w3.org/2002/08/xhtml/xhtml1-strict.xsd.
    let expected = [
        "html",
        "head",
        "title",
        "base",
        "meta",
        "link",
        "style",
        "script",
        "noscript",
        "body",
        "div",
        "p",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "ul",
        "ol",
        "li",
        "dl",
        "dt",
        "dd",
        "address",
        "hr",
        "pre",
        "blockquote",
        "ins",
        "del",
        "a",
        "span",
        "bdo",
        "br",
        "em",
        "strong",
        "dfn",
        "code",
        "samp",
        "kbd",
        "var",
        "cite",
        "abbr",
        "acronym",
        "q",
        "sub",
        "sup",
        "tt",
        "i",
        "b",
        "big",
        "small",
        "object",
        "param",
        "img",
        "map",
        "area",
        "form",
        "label",
        "input",
        "select",
        "optgroup",
        "option",
        "textarea",
        "fieldset",
        "legend",
        "button",
        "table",
        "caption",
        "thead",
        "tfoot",
        "tbody",
        "colgroup",
        "col",
        "tr",
        "th",
        "td",
    ];
    for el in expected {
        assert!(
            is_html_element(el),
            "Pemberton et al. 2002 §A.1 element {el:?} missing from loaded inventory"
        );
    }
}

/// Every XHTML 1.0 Strict attribute well-known from Pemberton et al.
/// 2002 §A.2 appears in the loaded ontology's attribute set.
#[test]
fn axiom_xhtml_1_0_strict_attributes_in_loaded_inventory() {
    for at in [
        "href", "src", "alt", "title", "id", "class", "name", "type", "value", "rel", "rev",
        "media", "lang", "charset", "colspan", "rowspan", "summary", "scope",
    ] {
        assert!(
            is_html_attribute(at),
            "Pemberton et al. 2002 §A.1 attribute {at:?} missing from loaded inventory"
        );
    }
}

/// Content categories are a closed enumeration per WHATWG HTML LS
/// §3.2.5 — exactly 7 leaves under HtmlContentCategory.
#[test]
fn axiom_content_categories_are_seven_per_whatwg_3_2_5() {
    use pr4xis::category::{Arrow, Category};
    let count = HtmlCategory::morphisms()
        .iter()
        .filter(|m| {
            m.target() == HtmlConcept::HtmlContentCategory
                && matches!(m.kind(), super::ontology::HtmlRelationKind::Subsumption)
                && m.source() != m.target()
        })
        .count();
    assert_eq!(
        count, 7,
        "WHATWG HTML LS §3.2.5 declares 7 content categories; found {count}"
    );
}

/// HTML name lookup is case-insensitive per WHATWG HTML LS §13.1.2
/// + HTML 4.01 §3.2.2.
#[test]
fn axiom_name_lookup_case_insensitive_per_whatwg_13_1_2() {
    for el in ["img", "IMG", "Img", "ImG", "iMg"] {
        assert!(is_html_element(el), "case fold failed on element {el:?}");
    }
    for at in ["href", "HREF", "Href", "HrEf"] {
        assert!(
            is_html_attribute(at),
            "case fold failed on attribute {at:?}"
        );
    }
}

/// The bundled XSD is the W3C-published XHTML 1.0 Strict schema —
/// targets the canonical XHTML 1.0 namespace.
#[test]
fn axiom_bundled_xsd_targets_xhtml_1_0_namespace() {
    assert!(
        XHTML_1_0_STRICT_XSD.contains("targetNamespace=\"http://www.w3.org/1999/xhtml\""),
        "bundled XSD does not target http://www.w3.org/1999/xhtml — wrong file?"
    );
}

/// HTML5-only sectioning + media elements are NOT in the loaded
/// inventory — documenting the M4.η.1.a follow-up scope.
#[test]
fn axiom_html5_only_elements_absent_per_m4_eta_1_a_scope() {
    for el in [
        "canvas", "video", "audio", "section", "article", "nav", "aside", "header", "footer",
        "main", "figure", "picture",
    ] {
        assert!(
            !is_html_element(el),
            "HTML5-only element {el:?} unexpectedly present — bundle drift or M4.η.1.a leak?"
        );
    }
}

// =============================================================================
// Functor / category laws
// =============================================================================

#[test]
fn category_laws_pass_on_html_category() {
    assert_category_laws::<HtmlCategory>();
}

#[test]
fn ontology_validates() {
    use pr4xis::logic::proof::Counterexample;
    super::ontology::HtmlOntology::validate().unwrap_or_else(|c: Box<dyn Counterexample>| {
        panic!(
            "HtmlOntology validation failed: {}",
            c.meta().description.as_str()
        )
    });
}

#[test]
fn every_concept_reaches_root_via_is_a() {
    // Mac Lane §I.3 functor identity preservation flavor: every
    // non-root concept reaches HtmlComponent through Subsumption.
    use super::ontology::HtmlRelationKind;
    use pr4xis::category::{Arrow, Category};
    let morphs = HtmlCategory::morphisms();
    for c in HtmlConcept::variants() {
        if matches!(c, HtmlConcept::HtmlComponent) {
            continue;
        }
        let reaches = morphs.iter().any(|m| {
            m.source() == c
                && m.target() == HtmlConcept::HtmlComponent
                && matches!(m.kind(), HtmlRelationKind::Subsumption)
        });
        assert!(reaches, "concept {c:?} does not is_a-reach HtmlComponent");
    }
}

// =============================================================================
// Property tests
// =============================================================================

fn arb_html_name() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            prop::char::range('a', 'z'),
            prop::char::range('A', 'Z'),
            prop::char::range('0', '9'),
            Just('-'),
            Just('_'),
        ],
        0..16,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

proptest! {
    /// Case-fold idempotence: looking up `x` and `x.to_lowercase()`
    /// produce the same answer for any name.
    #[test]
    fn prop_case_fold_factors_through_element(x in arb_html_name()) {
        let direct = is_html_element(&x);
        let folded = is_html_element(&x.to_lowercase());
        prop_assert_eq!(direct, folded);
    }

    #[test]
    fn prop_case_fold_factors_through_attribute(x in arb_html_name()) {
        let direct = is_html_attribute(&x);
        let folded = is_html_attribute(&x.to_lowercase());
        prop_assert_eq!(direct, folded);
    }

    /// Total function: every input produces a Boolean without panic.
    #[test]
    fn prop_total_function(x in arb_html_name()) {
        let _ = is_html_element(&x);
        let _ = is_html_attribute(&x);
    }

    /// Load idempotence: repeated reads return identical content.
    /// OnceLock guarantees object identity; this checks logical
    /// equality so a future refactor preserves the contract.
    #[test]
    fn prop_load_idempotent(_seed in any::<u32>()) {
        let a_el = element_names();
        let b_el = element_names();
        prop_assert_eq!(a_el.len(), b_el.len());
        for x in a_el {
            prop_assert!(b_el.contains(x));
        }
        let a_at = attribute_names();
        let b_at = attribute_names();
        prop_assert_eq!(a_at.len(), b_at.len());
    }

    /// Round-trip: every loaded element name round-trips through
    /// the case-insensitive lookup.
    #[test]
    fn prop_loaded_element_round_trip(idx in 0usize..1024) {
        let set = element_names();
        if set.is_empty() {
            return Ok(());
        }
        let n = set.len();
        let entry = set.iter().nth(idx % n).unwrap();
        prop_assert!(is_html_element(entry));
        prop_assert!(is_html_element(&entry.to_uppercase()));
    }

    /// Round-trip: every loaded attribute name round-trips through
    /// the case-insensitive lookup.
    #[test]
    fn prop_loaded_attribute_round_trip(idx in 0usize..1024) {
        let set = attribute_names();
        if set.is_empty() {
            return Ok(());
        }
        let n = set.len();
        let entry = set.iter().nth(idx % n).unwrap();
        prop_assert!(is_html_attribute(entry));
        prop_assert!(is_html_attribute(&entry.to_uppercase()));
    }
}
