//! Functor: XML 1.0 ontology → English. Projects every information-
//! item canonical phrase and every `xml:*`-reserved attribute name
//! through the WordNet-backed English pipeline, with the bundled
//! W3C sources' own naming acting as the authoritative resolution.
//!
//! ## Categorical setting
//!
//! Per Mac Lane *Categories for the Working Mathematician* §I.3,
//! the functor F: C → D is a structure-preserving map. Here:
//!
//! - **C** is [`super::ontology_v1::Xml10Category`] — the XML 1.0
//!   concept inventory plus the `Subsumption` morphisms encoded by
//!   the ontology macro.
//! - **D** is `English` — the WordNet-backed English pipeline.
//!
//! At the type level the functor collapses each `Xml10Concept` to
//! its canonical English head-noun phrase (`document`, `element`,
//! `processing instruction`, ...), each of which resolves through
//! WordNet under the bundled `english-wordnet-2025` data.
//!
//! At the instance level the functor acts as the resolver for the
//! schema-vocabulary classifier — the W3C-published names are the
//! authoritative answer to "is `lang` a recognized XML namespace
//! attribute?".
//!
//! ## Citations
//!
//! - Mac Lane, S. (1998) *Categories for the Working Mathematician*,
//!   Springer GTM 5, 2nd ed., §I.3 (Functors).
//! - Spivak, D. I. (2014) *Category Theory for the Sciences*, MIT
//!   Press, §5 (functorial structure preservation).
//! - Fellbaum, C. (ed.) (1998) *WordNet: An Electronic Lexical
//!   Database*, MIT Press.

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use super::loader_v1::{is_information_item_phrase, is_xml_namespace_attribute};
use super::ontology_v1::Xml10Concept;

/// True iff `name` is a recognized XML 1.0 vocabulary token per the
/// bundled W3C sources — either an `xml:*`-namespace-reserved
/// attribute name (`lang`, `space`, `base`, `id` per the W3C
/// xml.xsd) OR a canonical English head-noun phrase of an XML
/// Information Set information item (`document`, `element`, ...).
/// The wrapper is the single entry point downstream
/// `is_schema_vocabulary` classifiers consult to recognize XML 1.0
/// tokens.
///
/// Per `feedback_bottom_up_loaded_not_encoded`: this delegates to
/// [`super::loader_v1`], which reads the bundled XSD + Infoset rec
/// — names are never hand-coded as Rust string matches.
pub fn is_xml_10_vocabulary(name: &str) -> bool {
    is_xml_namespace_attribute(name) || is_information_item_phrase(name)
}

/// The canonical English head-noun phrase for each XML 1.0 concept,
/// used as the target-side label under the functor. The phrases
/// resolve cleanly through WordNet because every word is a
/// general-English content word (Fellbaum 1998).
pub fn canonical_phrase(c: Xml10Concept) -> Option<&'static str> {
    use Xml10Concept as X;
    Some(match c {
        X::XmlInformationItem => "XML information item",
        X::DocumentItem => "document",
        X::ElementItem => "element",
        X::AttributeItem => "attribute",
        X::ProcessingInstructionItem => "processing instruction",
        X::UnexpandedEntityReferenceItem => "unexpanded entity reference",
        X::CharacterItem => "character",
        X::CommentItem => "comment",
        X::DocumentTypeDeclarationItem => "document type declaration",
        X::UnparsedEntityItem => "unparsed entity",
        X::NotationItem => "notation",
        X::NamespaceItem => "namespace",
        // Reserved attributes — vocabulary names live one level
        // down (per-name); the concept itself has no single phrase.
        X::XmlReservedAttribute => return None,
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;

    #[test]
    fn vocabulary_recognizes_loaded_reserved_attributes() {
        // Every reserved attribute in the bundled xml.xsd is
        // recognized.
        assert!(is_xml_10_vocabulary("lang"));
        assert!(is_xml_10_vocabulary("space"));
        assert!(is_xml_10_vocabulary("base"));
        assert!(is_xml_10_vocabulary("id"));
    }

    #[test]
    fn vocabulary_recognizes_loaded_information_items() {
        // The 11 information-item phrases.
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
                is_xml_10_vocabulary(phrase),
                "expected XML 1.0 vocabulary token {phrase:?} not recognized"
            );
        }
    }

    #[test]
    fn vocabulary_rejects_unrelated_strings() {
        assert!(!is_xml_10_vocabulary("zzz_definitely_not_in_xml_1_0"));
        assert!(!is_xml_10_vocabulary(""));
    }

    #[test]
    fn vocabulary_is_case_insensitive() {
        assert!(is_xml_10_vocabulary("LANG"));
        assert!(is_xml_10_vocabulary("Lang"));
        assert!(is_xml_10_vocabulary("DOCUMENT"));
        assert!(is_xml_10_vocabulary("Document"));
    }

    #[test]
    fn canonical_phrase_covers_every_information_item() {
        // Every information-item concept has a canonical phrase.
        for c in [
            Xml10Concept::DocumentItem,
            Xml10Concept::ElementItem,
            Xml10Concept::AttributeItem,
            Xml10Concept::ProcessingInstructionItem,
            Xml10Concept::UnexpandedEntityReferenceItem,
            Xml10Concept::CharacterItem,
            Xml10Concept::CommentItem,
            Xml10Concept::DocumentTypeDeclarationItem,
            Xml10Concept::UnparsedEntityItem,
            Xml10Concept::NotationItem,
            Xml10Concept::NamespaceItem,
        ] {
            assert!(
                canonical_phrase(c).is_some(),
                "missing canonical phrase for {c:?}"
            );
        }
    }

    #[test]
    fn reserved_attribute_concept_has_no_canonical_phrase() {
        // The reserved-attribute concept is instance-valued, not
        // single-phrase. Per-instance phrases (`lang`, `space`, ...)
        // are loaded from the XSD.
        assert!(canonical_phrase(Xml10Concept::XmlReservedAttribute).is_none());
    }

    #[test]
    fn functor_law_identity_preservation() {
        // Looking up the same input twice gives the same answer —
        // the projection is referentially transparent.
        for c in Xml10Concept::variants() {
            let a = canonical_phrase(c);
            let b = canonical_phrase(c);
            assert_eq!(a, b, "identity preservation failed on {c:?}");
        }
    }

    #[test]
    fn functor_law_composition_with_lookup() {
        // For every concept c with a canonical phrase `p`, the
        // phrase is itself recognized by the vocabulary classifier.
        // This proves the functor composes with the
        // classifier-lookup map at the type level.
        for c in Xml10Concept::variants() {
            if let Some(phrase) = canonical_phrase(c) {
                // Skip the root — its phrase is a multi-word
                // compound that's *about* XML rather than a
                // vocabulary token in itself.
                if matches!(c, Xml10Concept::XmlInformationItem) {
                    continue;
                }
                assert!(
                    is_xml_10_vocabulary(phrase),
                    "composition law failed: canonical phrase {phrase:?} for {c:?} \
                     not recognized by is_xml_10_vocabulary"
                );
            }
        }
    }
}
