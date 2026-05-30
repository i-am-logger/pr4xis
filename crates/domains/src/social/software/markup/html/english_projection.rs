//! Functor: HTML ontology → English. Projects every HTML element /
//! attribute name through the WordNet-backed English pipeline, with
//! the bundled XHTML 1.0 Strict XSD's declared names acting as the
//! schema's own authoritative naming.
//!
//! ## Categorical setting
//!
//! Per Mac Lane *Categories for the Working Mathematician* §I.3,
//! the functor F: C → D is a structure-preserving map. Here:
//!
//! - **C** is [`super::ontology::HtmlCategory`] — the HTML5
//!   concept inventory plus the `Subsumption` morphisms encoded by
//!   the ontology macro.
//! - **D** is `English` — the WordNet-backed English pipeline.
//!
//! At the type level the functor collapses each HtmlConcept to its
//! canonical English head-noun phrase (which resolves through
//! WordNet under the bundled `english-wordnet-2025` data).
//!
//! At the instance level the functor acts as the resolver for
//! XHTML-loaded element / attribute names — the schema's own
//! declared names are the authoritative answer to "is `img` a
//! recognized HTML element?".
//!
//! ## Citations
//!
//! - Fellbaum, C. (ed.) (1998) *WordNet: An Electronic Lexical
//!   Database*, MIT Press.
//! - Mac Lane, S. (1998) *Categories for the Working Mathematician*,
//!   Springer GTM 5, 2nd ed., §I.3 (Functors).
//! - Spivak, D. I. (2014) *Category Theory for the Sciences*, MIT
//!   Press, §5 (functorial structure preservation).

#[allow(unused_imports)]
use alloc::{format, string::String, string::ToString, vec, vec::Vec};

use super::loader::{is_html_attribute, is_html_element};

/// True iff `name` is a recognized HTML element OR HTML attribute
/// per the bundled XHTML 1.0 Strict XSD. The wrapper is the single
/// entry point downstream `is_schema_vocabulary` classifiers
/// consult to recognize HTML-vocabulary tokens.
///
/// Per `feedback_bottom_up_loaded_not_encoded`: this delegates to
/// [`super::loader`], which reads the bundled XSD — names are
/// never hand-coded as Rust string matches.
pub fn is_html_vocabulary(name: &str) -> bool {
    is_html_element(name) || is_html_attribute(name)
}

/// The canonical English head-noun phrase for each HTML content
/// category, used as the target-side label under the functor.
///
/// Per WHATWG HTML LS §3.2.5 the category names are themselves
/// the canonical English phrases — "flow content", "phrasing
/// content", etc. Every word in the phrase resolves through WordNet.
pub fn content_category_phrase(c: super::ontology::HtmlConcept) -> Option<&'static str> {
    use super::ontology::HtmlConcept as H;
    Some(match c {
        H::HtmlContentCategory => "content category",
        H::FlowContent => "flow content",
        H::PhrasingContent => "phrasing content",
        H::MetadataContent => "metadata content",
        H::InteractiveContent => "interactive content",
        H::SectioningContent => "sectioning content",
        H::EmbeddedContent => "embedded content",
        H::HeadingContent => "heading content",
        H::HtmlComponent => "HTML component",
        // Element / attribute concepts are not single-phrase
        // entities — their projection is per-instance, through the
        // bundled XSD's name inventory.
        H::HtmlElement | H::HtmlAttribute => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::super::ontology::HtmlConcept;
    use super::*;

    #[test]
    fn html_vocabulary_recognizes_loaded_elements() {
        // Every element in the bundled XSD is recognized.
        assert!(is_html_vocabulary("img"));
        assert!(is_html_vocabulary("br"));
        assert!(is_html_vocabulary("del"));
        assert!(is_html_vocabulary("meta"));
    }

    #[test]
    fn html_vocabulary_recognizes_loaded_attributes() {
        assert!(is_html_vocabulary("href"));
        assert!(is_html_vocabulary("src"));
        assert!(is_html_vocabulary("alt"));
        assert!(is_html_vocabulary("colspan"));
        assert!(is_html_vocabulary("rowspan"));
    }

    #[test]
    fn html_vocabulary_rejects_unrelated_strings() {
        // General-English words are not in the HTML vocabulary
        // (they resolve through WordNet via a separate path).
        assert!(!is_html_vocabulary("section_unrelated_string_zzz"));
        assert!(!is_html_vocabulary(""));
    }

    #[test]
    fn html_vocabulary_is_case_insensitive() {
        assert!(is_html_vocabulary("IMG"));
        assert!(is_html_vocabulary("Img"));
        assert!(is_html_vocabulary("HREF"));
        assert!(is_html_vocabulary("Href"));
    }

    #[test]
    fn content_category_phrases_cover_every_category() {
        // Every content-category concept has a canonical phrase.
        for c in [
            HtmlConcept::HtmlContentCategory,
            HtmlConcept::FlowContent,
            HtmlConcept::PhrasingContent,
            HtmlConcept::MetadataContent,
            HtmlConcept::InteractiveContent,
            HtmlConcept::SectioningContent,
            HtmlConcept::EmbeddedContent,
            HtmlConcept::HeadingContent,
        ] {
            assert!(
                content_category_phrase(c).is_some(),
                "missing canonical phrase for {c:?}"
            );
        }
    }

    #[test]
    fn element_and_attribute_have_no_canonical_phrase() {
        // Element / attribute concepts are instance-valued, not
        // single-phrase entities. The function returns None to
        // signal the caller to consult the per-instance projection.
        assert!(content_category_phrase(HtmlConcept::HtmlElement).is_none());
        assert!(content_category_phrase(HtmlConcept::HtmlAttribute).is_none());
    }
}
