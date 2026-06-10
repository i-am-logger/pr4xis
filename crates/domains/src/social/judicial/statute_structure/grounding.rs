//! Grounding statute prose into the English lexicon — the honest written-form
//! `denotes` floor at the statute level (the producer the USC codec will store).
//!
//! A span of statute text is scanned for content-word lemmas
//! ([`extract_lemmas`](super::term_extractor::extract_lemmas) — stopwords and
//! numerals filtered, deduped); each lemma that English knows as a written form
//! becomes a typed `denotes` pointer into the `english_wordnet` archive: a
//! [`Grounded`](pr4xis_runtime::definition::EdgeTarget::Grounded) edge targeting
//! the word's [`ontolex:Form`](crate::cognitive::linguistics::english::bridge::form_atom)
//! atom by content address.
//!
//! It is the WRITTEN-FORM FLOOR (the weakest adequate claim): the pointer lands
//! on a Form — "this written form occurred" — NEVER on a sense. Fine-grained
//! word-sense disambiguation is 59–82% accurate; a written-form anchor is ~0
//! error (Halpin & Hayes 2010; the design's written-form-floor decision). Sense
//! is licensed only by a statute's own definitions, a stronger kind deferred.
//!
//! This module COMPUTES the pointers (pure, no archive change). Persisting them
//! into the USC `.prx` codec — and re-minting the archive pins that act re-mints
//! — is the next, maintainer-coordinated slice; the source byte-exact path is
//! untouched by it.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis_runtime::definition::EdgeTarget;

use crate::cognitive::linguistics::english::English;
use crate::cognitive::linguistics::english::bridge::{ENGLISH_ONTOLOGY, form_atom};

use super::term_extractor::extract_lemmas;

/// One written-form `denotes` pointer: the surface `word` that occurred and the
/// [`Grounded`](EdgeTarget::Grounded) edge into its `ontolex:Form` atom in
/// `english_wordnet`. The edge is what a statute subdivision carries; resolving
/// it (via the runtime `AtomResolver`) yields the Form atom — never a sense.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenotesPointer {
    /// The written form (lowercased lemma) that occurred in the prose.
    pub word: String,
    /// The grounded edge — `denotes` into the word's Form atom by content address.
    pub target: EdgeTarget,
}

/// The written-form `denotes` pointers for a span of statute prose: one per
/// content-word lemma English knows as a written form, each pointing at that
/// word's [`form_atom`] by content address.
///
/// A word English does not know is left UNGROUNDED (no pointer) — the floor only
/// asserts forms that actually exist in the lexicon, never invents a target.
pub fn denotes_pointers(text: &str, english: &English) -> Vec<DenotesPointer> {
    extract_lemmas(text)
        .into_iter()
        // Only a written form English actually knows is grounded — the floor
        // never points at a non-existent atom.
        .filter(|form| !english.lookup(&form.written_rep).is_empty())
        .filter_map(|form| {
            let atom = form_atom(&form.written_rep).address().ok()?;
            Some(DenotesPointer {
                word: form.written_rep.clone(),
                target: EdgeTarget::Grounded {
                    ontology: ENGLISH_ONTOLOGY.to_string(),
                    atom,
                },
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;

    use pr4xis_runtime::grounding::{AtomResolver, ConnectedOntologies, ConnectedOntology};

    use crate::cognitive::linguistics::english::bridge::{FORM_KIND, project_archive_with_forms};

    #[test]
    fn grounds_only_the_content_words_english_knows() {
        // "a dog is an animal" — content words dog, animal (a/is/an are stopwords);
        // both are sample written forms, so both ground.
        let english = English::sample();
        let pointers = denotes_pointers("a dog is an animal", &english);
        let words: Vec<&str> = pointers.iter().map(|p| p.word.as_str()).collect();
        assert!(
            words.contains(&"dog"),
            "dog is a known content word; got {words:?}"
        );
        assert!(words.contains(&"animal"), "animal is a known content word");
        assert!(
            !words.iter().any(|w| ["a", "is", "an"].contains(w)),
            "stopwords are not grounded; got {words:?}"
        );
    }

    #[test]
    fn an_unknown_word_is_left_ungrounded() {
        // "dog" grounds; "xyzzy" is not a written form English knows → no pointer.
        let english = English::sample();
        let pointers = denotes_pointers("dog xyzzy", &english);
        assert_eq!(pointers.len(), 1);
        assert_eq!(pointers[0].word, "dog");
    }

    /// END-TO-END: statute prose → a `denotes` pointer → resolves (via the runtime
    /// `AtomResolver`) into the word's `ontolex:Form` atom in `english_wordnet`,
    /// and the resolved target IS a Form (never a sense). The producer + G3a
    /// resolver + G3b-1 Form layer, joined.
    #[test]
    fn a_produced_pointer_resolves_to_a_form_atom() {
        let english = English::sample();
        let archive = project_archive_with_forms(&english);
        let english_root = archive.root().unwrap();

        let pointer = denotes_pointers("the dog", &english)
            .into_iter()
            .find(|p| p.word == "dog")
            .expect("dog grounds");

        let mut peers = BTreeMap::new();
        peers.insert(ENGLISH_ONTOLOGY.to_string(), archive);
        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: ENGLISH_ONTOLOGY.to_string(),
            root: english_root,
            role: "denotes".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();

        let resolved = resolver
            .resolve(&pointer.target)
            .expect("the produced denotes pointer resolves by content address");
        assert_eq!(
            resolved.kind, FORM_KIND,
            "the floor pointer resolves to an ontolex:Form, never a sense"
        );
        assert_eq!(resolved.name, "dog");
    }
}
