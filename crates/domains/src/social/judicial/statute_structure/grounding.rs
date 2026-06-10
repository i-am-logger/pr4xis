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
//! # The ontological, general way
//!
//! `denotes` is ONE grounding lens. [`denotes_lens`] adapts the producer to the
//! generic [`ground`](pr4xis_runtime::grounding::ground): any content
//! [`Archive`](pr4xis_runtime::archive::Archive) — a USC title projected by
//! `uslm::corpus::bridge`, English itself, anything — grounds the same way,
//! gaining typed [`EdgeTarget::Grounded`] edges in the GENERIC substrate that
//! resolve through the generic `AtomResolver`. English is confined to the lens;
//! `cites` / `defines` are other lenses of the same shape. There is no bespoke
//! string side-channel and no per-source codec.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use pr4xis_runtime::definition::{Definition, EdgeTarget};

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

/// The lexical `denotes` grounding LENS — the lens form of [`denotes_pointers`],
/// for the generic [`ground`](pr4xis_runtime::grounding::ground).
///
/// It grounds ANY archive node (a statute provision, …) into the English
/// `ontolex:Form` atoms its lexical prose denotes, producing typed
/// `(denotes, `[`EdgeTarget::Grounded`]`)` edges resolved by the generic
/// `AtomResolver`. English is confined to THIS lens — `ground` itself is
/// source-agnostic, and `cites` / `defines` are other lenses of the same shape.
pub fn denotes_lens(english: &English) -> impl Fn(&Definition) -> Vec<(String, EdgeTarget)> + '_ {
    move |node| {
        node.lexical.as_deref().map_or_else(Vec::new, |text| {
            denotes_pointers(text, english)
                .into_iter()
                .map(|p| ("denotes".to_string(), p.target))
                .collect()
        })
    }
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

    /// THE GENERIC LOOP: a content archive grounds via `ground(denotes_lens)` —
    /// adding typed `EdgeTarget::Grounded` edges to its nodes — and those edges
    /// resolve through the GENERIC `AtomResolver` to `ontolex:Form` atoms. No
    /// English-hardcoding outside the lens; the same `ground` would carry a `cites`
    /// lens over the same substrate. This is the ontological replacement for the
    /// reverted string side-channel.
    #[test]
    fn a_content_archive_grounds_via_the_lens_and_resolves_to_forms() {
        use pr4xis_runtime::archive::Archive;
        use pr4xis_runtime::definition::Definition;
        use pr4xis_runtime::grounding::ground;

        let english = English::sample();

        // A content archive — e.g. a statute provision node carrying prose. (The
        // USC bridge produces exactly such Definitions; here a bare one isolates
        // the grounding loop.)
        let content = Archive {
            nodes: alloc::vec![Definition {
                kind: "Provision".to_string(),
                name: "/us/usc/t1/s1/a".to_string(),
                edges: alloc::vec![],
                axioms: alloc::vec![],
                lexical: Some("the dog is an animal".to_string()),
            }],
            connections: alloc::vec![],
        };

        // Ground it with the lexical denotes lens — typed Grounded edges added.
        let grounded = ground(&content, denotes_lens(&english));
        let provision = &grounded.nodes[0];
        let denotes: Vec<&str> = provision
            .edges
            .iter()
            .filter(|(k, _)| k == "denotes")
            .filter_map(|(_, t)| match t {
                EdgeTarget::Grounded { .. } => Some("denotes"),
                EdgeTarget::Local(_) => None,
            })
            .collect();
        assert!(
            !denotes.is_empty(),
            "the provision grounds its content words"
        );

        // Resolve every grounded edge through the GENERIC resolver — each lands on
        // a Form atom (never a sense).
        let archive = project_archive_with_forms(&english);
        let english_root = archive.root().unwrap();
        let mut peers = BTreeMap::new();
        peers.insert(ENGLISH_ONTOLOGY.to_string(), archive);
        let manifest = ConnectedOntologies(alloc::vec![ConnectedOntology {
            name: ENGLISH_ONTOLOGY.to_string(),
            root: english_root,
            role: "denotes".to_string(),
        }]);
        let resolver = AtomResolver::new(&manifest, &peers).unwrap();

        let mut resolved_forms = Vec::new();
        for (_, target) in provision.edges.iter().filter(|(k, _)| k == "denotes") {
            let form = resolver.resolve(target).expect("a grounded edge resolves");
            assert_eq!(
                form.kind, FORM_KIND,
                "grounds to an ontolex:Form, never a sense"
            );
            resolved_forms.push(form.name.clone());
        }
        assert!(resolved_forms.contains(&"dog".to_string()));
        assert!(resolved_forms.contains(&"animal".to_string()));
    }
}
