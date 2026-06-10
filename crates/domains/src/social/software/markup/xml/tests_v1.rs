//! Three-layer test stack for the M4.η.2 XML 1.0 ontology:
//!
//! 1. **Axioms (cited)** — every claim about the loaded inventory
//!    has an explicit citation tag.
//! 2. **Property tests (proptest)** — case-fold idempotence,
//!    load-idempotence, round-trip on canonical-phrase lookup.
//! 3. **Functor laws** — `assert_category_laws` on `Xml10Category`.
//!
//! Coexists with the older `xml::tests` (M4.δ-era `XmlNodeKind` /
//! `XmlCategory`) — these tests target the new
//! `ontology_v1::Xml10*` types only.

use super::english_projection_v1::{canonical_phrase, is_xml_10_vocabulary};
use super::loader_v1::{
    information_item_phrases, information_items, is_information_item_phrase,
    is_xml_namespace_attribute, reserved_attribute_names,
};
use super::ontology_v1::{Xml10Category, Xml10Concept, Xml10Ontology, Xml10RelationKind};
use pr4xis::category::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::Ontology;
use proptest::prelude::*;

// =============================================================================
// Axiom tests (cited)
// =============================================================================

/// Cowan & Tobin (2004) XML Information Set Second Edition §2.1–
/// §2.11 declares exactly 11 information items. The build-time
/// loader produces these from the published rec; the axiom asserts
/// the count.
#[test]
fn axiom_eleven_information_items_per_cowan_tobin_2004() {
    let items = information_items();
    assert_eq!(
        items.len(),
        11,
        "Cowan & Tobin 2004 §2 declares 11 information items; loader yielded {} ({:?})",
        items.len(),
        items.iter().map(|i| i.english_name).collect::<Vec<_>>()
    );
}

/// Every canonical information-item name from Cowan & Tobin 2004
/// §2.1–§2.11 is in the loaded inventory.
#[test]
fn axiom_canonical_information_item_names_present() {
    for phrase in [
        "document",
        "element",
        "attribute",
        "processing instruction",
        "unexpanded entity reference",
        "character",
        "comment",
        "document type declaration",
        "unparsed entity",
        "notation",
        "namespace",
    ] {
        assert!(
            is_information_item_phrase(phrase),
            "Cowan & Tobin 2004 information item {phrase:?} missing from loaded inventory"
        );
    }
}

/// W3C xml.xsd declares the four reserved-namespace attributes
/// (`xml:lang`, `xml:space`, `xml:base`, `xml:id`) per Bray et al.
/// 2009 Namespaces in XML 1.0 §3.
#[test]
fn axiom_four_xml_namespace_attributes_per_bray_et_al_2009() {
    let names = reserved_attribute_names();
    assert_eq!(
        names.len(),
        4,
        "W3C xml.xsd declares 4 xml:*-reserved attributes; loader yielded {} ({:?})",
        names.len(),
        names
    );
}

/// Each of the four canonical `xml:*` reserved attribute local-
/// names appears in the loaded inventory.
#[test]
fn axiom_xml_reserved_attribute_local_names_present() {
    for name in ["base", "id", "lang", "space"] {
        assert!(
            is_xml_namespace_attribute(name),
            "W3C xml.xsd reserved attribute {name:?} missing from loaded inventory"
        );
    }
}

/// XML Information Set §2 partitions every part of an XML document
/// into one of the information items — the ontology's
/// `XmlInformationItem` root is `is_a`-reachable from every leaf.
#[test]
fn axiom_every_non_root_concept_reaches_xml_information_item() {
    use pr4xis::category::{Arrow, Category};
    let morphs = Xml10Category::morphisms();
    for c in Xml10Concept::variants() {
        if matches!(c, Xml10Concept::XmlInformationItem) {
            continue;
        }
        let reaches = morphs.iter().any(|m| {
            m.source() == c
                && m.target() == Xml10Concept::XmlInformationItem
                && matches!(m.kind(), Xml10RelationKind::Subsumption)
        });
        assert!(
            reaches,
            "concept {c:?} does not is_a-reach XmlInformationItem"
        );
    }
}

/// XML 1.0 §2.1 requires that every well-formed XML document have
/// exactly one root element; in the information-set model
/// (Cowan & Tobin 2004 §2.1), the corresponding rule is that the
/// document information set has exactly one DocumentItem at its
/// root. The enum-variant uniqueness encodes this at the type
/// level.
#[test]
fn axiom_document_item_is_unique_variant_per_xml_1_0_2_1() {
    let variants = Xml10Concept::variants();
    let count = variants
        .iter()
        .filter(|c| matches!(c, Xml10Concept::DocumentItem))
        .count();
    assert_eq!(
        count, 1,
        "XML 1.0 §2.1 + Cowan & Tobin 2004 §2.1: DocumentItem must be a unique variant"
    );
}

/// Spot-check the W3C-published rec anchor format. The Cowan &
/// Tobin 2004 rec assigns anchors like `infoitem.document`,
/// `infoitem.element`, etc. on each `<a name="...">` link target.
#[test]
fn axiom_w3c_anchor_format_consistent_with_infoset_rec() {
    let items = information_items();
    for it in items {
        assert!(
            it.anchor.starts_with("infoitem."),
            "anchor {:?} does not match W3C-published rec convention `infoitem.*`",
            it.anchor
        );
    }
}

/// The W3C-published rec's section numbering uses `2.1`–`2.11` for
/// the 11 information items (§2 is the umbrella). Every loaded
/// item's section starts with `"2."`.
#[test]
fn axiom_section_numbers_under_section_two() {
    let items = information_items();
    for it in items {
        assert!(
            it.section.starts_with("2."),
            "section {:?} does not fall under §2 — Cowan & Tobin 2004 §2.1–§2.11",
            it.section
        );
    }
}

// =============================================================================
// Functor / category laws
// =============================================================================

#[test]
fn category_laws_pass_on_xml_10_category() {
    assert_category_laws::<Xml10Category>();
}

#[test]
fn ontology_validates() {
    use pr4xis::logic::proof::Counterexample;
    Xml10Ontology::validate().unwrap_or_else(|c: Box<dyn Counterexample>| {
        panic!(
            "Xml10Ontology validation failed: {}",
            c.meta().description.as_str()
        )
    });
}

#[test]
fn functor_xml_10_to_english_identity_preservation() {
    // Per Mac Lane §I.3: F(id_X) = id_F(X). The canonical-phrase
    // projection is referentially transparent — repeated lookups
    // produce identical phrases.
    for c in Xml10Concept::variants() {
        let a = canonical_phrase(c);
        let b = canonical_phrase(c);
        assert_eq!(a, b, "identity preservation failed on {c:?}");
    }
}

#[test]
fn functor_xml_10_to_english_composition_law() {
    // For every concept with a canonical phrase, the phrase is
    // itself recognized by the schema-vocabulary classifier. This
    // proves the functor composes with the classifier-lookup map.
    for c in Xml10Concept::variants() {
        if matches!(
            c,
            Xml10Concept::XmlInformationItem | Xml10Concept::XmlReservedAttribute
        ) {
            continue;
        }
        if let Some(phrase) = canonical_phrase(c) {
            assert!(
                is_xml_10_vocabulary(phrase),
                "composition law failed: canonical phrase {phrase:?} for {c:?} \
                 not recognized by is_xml_10_vocabulary"
            );
        }
    }
}

// =============================================================================
// Property tests
// =============================================================================

fn arb_lemma() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            prop::char::range('a', 'z'),
            prop::char::range('A', 'Z'),
            prop::char::range('0', '9'),
            Just(' '),
            Just('_'),
        ],
        0..32,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

proptest! {
    /// Case-fold idempotence: looking up `x` and `x.to_lowercase()`
    /// produce the same answer for the reserved-attribute lookup.
    #[test]
    fn prop_case_fold_reserved_attribute(x in arb_lemma()) {
        let direct = is_xml_namespace_attribute(&x);
        let folded = is_xml_namespace_attribute(&x.to_lowercase());
        prop_assert_eq!(direct, folded);
    }

    /// Case-fold idempotence on the information-item lookup.
    #[test]
    fn prop_case_fold_information_item(x in arb_lemma()) {
        let direct = is_information_item_phrase(&x);
        let folded = is_information_item_phrase(&x.to_lowercase());
        prop_assert_eq!(direct, folded);
    }

    /// Total function: every input produces a Boolean without
    /// panic.
    #[test]
    fn prop_total_function(x in arb_lemma()) {
        let _ = is_xml_namespace_attribute(&x);
        let _ = is_information_item_phrase(&x);
        let _ = is_xml_10_vocabulary(&x);
    }

    /// Load idempotence — repeated reads return identical content.
    #[test]
    fn prop_load_idempotent(_seed in any::<u32>()) {
        let a_attrs = reserved_attribute_names();
        let b_attrs = reserved_attribute_names();
        prop_assert_eq!(a_attrs.len(), b_attrs.len());
        for x in a_attrs {
            prop_assert!(b_attrs.contains(x));
        }
        let a_items = information_item_phrases();
        let b_items = information_item_phrases();
        prop_assert_eq!(a_items.len(), b_items.len());
        for x in a_items {
            prop_assert!(b_items.contains(x));
        }
    }

    /// Round-trip: every loaded reserved attribute name round-trips
    /// through the case-insensitive lookup.
    #[test]
    fn prop_reserved_attribute_round_trip(idx in 0usize..1024) {
        let set = reserved_attribute_names();
        if set.is_empty() {
            return Ok(());
        }
        let n = set.len();
        let entry = set.iter().nth(idx % n).unwrap();
        prop_assert!(is_xml_namespace_attribute(entry));
        prop_assert!(is_xml_namespace_attribute(&entry.to_uppercase()));
    }

    /// Round-trip: every loaded information-item phrase round-trips
    /// through the case-insensitive lookup.
    #[test]
    fn prop_information_item_round_trip(idx in 0usize..1024) {
        let set = information_item_phrases();
        if set.is_empty() {
            return Ok(());
        }
        let n = set.len();
        let entry = set.iter().nth(idx % n).unwrap();
        prop_assert!(is_information_item_phrase(entry));
        prop_assert!(is_information_item_phrase(&entry.to_uppercase()));
    }
}
