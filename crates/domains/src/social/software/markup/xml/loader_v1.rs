//! XML 1.0 grounding-source loader — exposes the two build-time
//! arrays as cached lookup functions:
//!
//! - `reserved_attribute_names()` — the four `xml:*`-namespace
//!   reserved attributes from the W3C xml.xsd.
//! - `information_items()` — the 11 information items from the W3C
//!   XML Information Set rec.
//!
//! Per `feedback_bottom_up_loaded_not_encoded`: every entry comes
//! from a registered authoritative source — never hand-coded.
//!
//! ## Citations
//!
//! See [`super::ontology_v1`] for the full citation list. The two
//! immediate sources are:
//!
//! - W3C xml.xsd (canonical xml-namespace schema).
//! - Cowan & Tobin (2004) XML Information Set (Second Edition), W3C
//!   Recommendation 4 February 2004.

#[allow(unused_imports)]
use alloc::{
    collections::BTreeSet,
    string::{String, ToString},
    vec::Vec,
};

use std::sync::OnceLock;

// =============================================================================
// Build-time generated data
// =============================================================================

mod generated_namespace {
    include!(concat!(
        env!("OUT_DIR"),
        "/xml_namespace_schema_generated.rs"
    ));
}

mod generated_infoset {
    include!(concat!(env!("OUT_DIR"), "/xml_infoset_generated.rs"));
}

pub use generated_infoset::InformationItemEntry;

// =============================================================================
// Reserved-attribute inventory (xml.xsd)
// =============================================================================

/// Lazily-loaded set of `xml:*`-namespace-reserved attribute local
/// names (lowercased) per the W3C xml.xsd bundled at
/// `crates/domains/data/markup-schemas/xml/xml.xsd`.
pub fn reserved_attribute_names() -> &'static BTreeSet<String> {
    static SET: OnceLock<BTreeSet<String>> = OnceLock::new();
    SET.get_or_init(|| {
        generated_namespace::XML_NAMESPACE_ATTRIBUTES
            .iter()
            .map(|s| s.to_lowercase())
            .collect()
    })
}

/// True iff `local_name` is an `xml:*`-namespace-reserved attribute
/// per the W3C xml.xsd. Case-insensitive (XML attribute names within
/// the reserved namespace are normatively lowercase per Bray et al.
/// 2009 §3, but the lookup is case-fold for parity with the HTML
/// loader contract).
pub fn is_xml_namespace_attribute(local_name: &str) -> bool {
    reserved_attribute_names().contains(&local_name.to_lowercase())
}

// =============================================================================
// Information-item inventory (XML Information Set rec)
// =============================================================================

/// The 11 information items per Cowan & Tobin 2004 §2, in section
/// order (2.1 Document, 2.2 Element, ..., 2.11 Namespace).
pub fn information_items() -> &'static [InformationItemEntry] {
    generated_infoset::XML_INFOSET_INFORMATION_ITEMS
}

/// Lazily-loaded set of canonical English head-noun phrases for the
/// 11 information items (e.g. `"document"`, `"element"`, ...). Used
/// by the English-projection functor.
pub fn information_item_phrases() -> &'static BTreeSet<String> {
    static SET: OnceLock<BTreeSet<String>> = OnceLock::new();
    SET.get_or_init(|| {
        information_items()
            .iter()
            .map(|e| e.english_name.to_lowercase())
            .collect()
    })
}

/// True iff `phrase` is the canonical English name of a loaded
/// information item, case-insensitive.
pub fn is_information_item_phrase(phrase: &str) -> bool {
    information_item_phrases().contains(&phrase.to_lowercase())
}

/// True iff `name` is a recognized XML 1.0 vocabulary token — either
/// an `xml:*`-reserved attribute name (`lang`, `space`, `base`,
/// `id`) OR a canonical English head-noun phrase of an information
/// item (`document`, `element`, `attribute`, ...). This is the
/// entry point the schema-vocabulary classifier chain consults to
/// recognize XML 1.0 ontology tokens.
pub fn is_xml_10_vocabulary(name: &str) -> bool {
    let lowered = name.to_lowercase();
    reserved_attribute_names().contains(&lowered) || information_item_phrases().contains(&lowered)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn loader_yields_four_xml_namespace_attributes() {
        // The bundled W3C xml.xsd declares `xml:lang`, `xml:space`,
        // `xml:base`, `xml:id` — four reserved attributes. Per
        // Bray et al. 2009 Namespaces in XML 1.0 §3.
        let names = reserved_attribute_names();
        assert_eq!(
            names.len(),
            4,
            "W3C xml.xsd declares 4 xml:*-reserved attributes; loader found {}: {:?}",
            names.len(),
            names
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn loader_yields_the_four_canonical_xml_reserved_names() {
        for n in ["base", "id", "lang", "space"] {
            assert!(
                is_xml_namespace_attribute(n),
                "expected xml:{n} reserved attribute missing from loaded set"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn loader_yields_eleven_information_items() {
        // Cowan & Tobin 2004 §2.1–§2.11 — exactly 11 information
        // items.
        let items = information_items();
        assert_eq!(
            items.len(),
            11,
            "Cowan & Tobin 2004 §2 declares 11 information items; loader found {}: {:?}",
            items.len(),
            items.iter().map(|i| i.english_name).collect::<Vec<_>>()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn loader_yields_every_named_information_item() {
        // The 11 canonical English head-noun phrases per Cowan &
        // Tobin 2004 §2.1–§2.11.
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
                "expected information item {phrase:?} not found in loaded set"
            );
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn loader_anchors_match_w3c_published_rec() {
        // Spot-check anchors from the published rec at
        // https://www.w3.org/TR/xml-infoset/. The W3C uses
        // `infoitem.document`, `infoitem.element`, `infoitem.attribute`,
        // `infoitem.pi`, ... — full list per §2.
        let items = information_items();
        let anchors: Vec<&str> = items.iter().map(|i| i.anchor).collect();
        assert!(anchors.contains(&"infoitem.document"));
        assert!(anchors.contains(&"infoitem.element"));
        assert!(anchors.contains(&"infoitem.attribute"));
        assert!(anchors.contains(&"infoitem.pi"));
        assert!(anchors.contains(&"infoitem.character"));
        assert!(anchors.contains(&"infoitem.comment"));
        assert!(anchors.contains(&"infoitem.doctype"));
        assert!(anchors.contains(&"infoitem.notation"));
        assert!(anchors.contains(&"infoitem.namespace"));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lookup_is_case_insensitive() {
        // Both halves of the loader are case-fold to keep parity
        // with the HTML loader contract.
        assert!(is_xml_namespace_attribute("LANG"));
        assert!(is_xml_namespace_attribute("Lang"));
        assert!(is_xml_namespace_attribute("lang"));
        assert!(is_information_item_phrase("DOCUMENT"));
        assert!(is_information_item_phrase("Document"));
        assert!(is_information_item_phrase("document"));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn empty_input_rejected() {
        assert!(!is_xml_namespace_attribute(""));
        assert!(!is_information_item_phrase(""));
        assert!(!is_xml_10_vocabulary(""));
    }

    #[pr4xis::praxis_value(Verifiable, Honest)]
    #[test]
    fn is_xml_10_vocabulary_unifies_both_halves() {
        // Reserved attribute side.
        assert!(is_xml_10_vocabulary("lang"));
        assert!(is_xml_10_vocabulary("space"));
        // Information-item phrase side.
        assert!(is_xml_10_vocabulary("element"));
        assert!(is_xml_10_vocabulary("document"));
        // Outside both.
        assert!(!is_xml_10_vocabulary("zzz_definitely_not_in_xml_1_0"));
    }
}
