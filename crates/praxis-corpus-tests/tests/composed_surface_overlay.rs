//! FULL-CORPUS surface-overlay sweep — the strongest oracle for the
//! `ComposedReasoner`'s loaded-only overlay (`ComposedSurfaceUnionFaithful`):
//! over the REAL packed `WordIndex` (every one of English's ~131.8k words) and
//! a REAL loaded USC title, every word resolves to EXACTLY
//!
//! ```text
//! composed.lookup(word) == english.lookup(word) ++ overlay(word)
//! ```
//!
//! with the pinned order contract — English's ids first (the packed run order,
//! borrowed zero-copy, never re-owned by the reasoner), then the loaded ids in
//! MINT order (archive node order; per node the lowercased name surface, then
//! its `ontolex:Form` surfaces). The overlay expectation is re-derived here
//! INDEPENDENTLY by walking the loaded archive — never by reading the
//! reasoner's own index — so a union-order break, a dropped English
//! fall-through, or a lost loaded surface each fails this sweep.
//!
//! This sweep also PINS the dedup fix the overlay landed: the former eager
//! seeding iterated `known_words()` (function words ++ WordNet words) and
//! extended per occurrence, so the 78 words living in BOTH stores (e.g. "a",
//! "can", "will") carried their English ids TWICE. The overlay falls through to
//! `english.lookup()` itself, so `composed.lookup(w) == english.lookup(w)`
//! holds exactly — duplicates gone.

use std::collections::BTreeMap;
use std::rc::Rc;

use pr4xis::ontology::meta::OntologyName;
use pr4xis_domains::applied::data_provisioning::registry::data_sources;
use pr4xis_domains::cognitive::linguistics::composed::ComposedReasoner;
use pr4xis_domains::cognitive::linguistics::english::bridge::FORM_KIND;
use pr4xis_domains::cognitive::linguistics::english::english_loaded;
use pr4xis_domains::cognitive::linguistics::english::{ConceptId, LexicalReasoner};
use pr4xis_domains::cognitive::linguistics::language::Language;
use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis_domains::social::software::markup::xml::uslm::corpus::bridge::usc_runtime_ontology;
use pr4xis_domains::social::software::markup::xml::uslm::{UsCode, read_uslm_title};
use pr4xis_runtime::lens::archive_lens::archived_local_name;
use pr4xis_runtime::ontology::RuntimeOntology;
use praxis_corpus_tests::{require, workspace_root};

/// Load the first provisioned USC title as a `UsCode` (`None` routes through
/// [`require`] to hard-fail — tests do not skip).
fn first_provisioned_title() -> Option<UsCode> {
    let root = workspace_root();
    for entry in data_sources() {
        if entry.kind != SourceTaxonomyConcept::UsCodeTitle {
            continue;
        }
        let Ok(source) = std::fs::read(root.join(entry.local_path())) else {
            continue;
        };
        let text = core::str::from_utf8(&source).expect("USLM source is UTF-8");
        let title = read_uslm_title(text).expect("parse title");
        return Some(UsCode::from_uslm_titles_owned(vec![title]));
    }
    None
}

/// Re-derive the loaded-only overlay INDEPENDENTLY of the reasoner: walk the
/// loaded archives exactly as the seeding's mint order does — non-Form nodes in
/// archive order (each assigned the next disjoint id above `base`); per node
/// the lowercased name surface, then each Form-atom surface its edges denote.
fn independent_overlay(
    loaded: &[Rc<RuntimeOntology>],
    base: u64,
) -> BTreeMap<String, Vec<ConceptId>> {
    let mut overlay: BTreeMap<String, Vec<ConceptId>> = BTreeMap::new();
    let mut next = base;
    for onto in loaded {
        let form_names: std::collections::BTreeSet<&str> = onto
            .archive()
            .nodes
            .iter()
            .filter(|n| n.kind == FORM_KIND)
            .map(|n| n.name.as_str())
            .collect();
        for node in onto.archive().nodes.iter() {
            if node.kind == FORM_KIND {
                continue;
            }
            let id = ConceptId::new(next);
            next += 1;
            overlay
                .entry(node.name.to_lowercase())
                .or_default()
                .push(id);
            for edge in node.edges.iter() {
                if let Some(form) = archived_local_name(&edge.1)
                    && form_names.contains(form)
                {
                    overlay.entry(form.to_lowercase()).or_default().push(id);
                }
            }
        }
    }
    overlay
}

/// THE SWEEP: every REAL English word (the full packed `WordIndex` + the
/// function words) AND every loaded surface resolves through the composed
/// reasoner to exactly `english.lookup(word) ++ overlay(word)`, order pinned.
#[test]
fn every_english_word_resolves_to_the_ordered_union_over_the_real_corpus() {
    let usc = require(first_provisioned_title(), "usc");
    let english = english_loaded();
    let onto = usc_runtime_ontology(&usc, OntologyName::new_static("usc_title"))
        .expect("the provisioned title materializes");
    let loaded = vec![Rc::new(onto)];
    let overlay = independent_overlay(&loaded, english.concept_count() as u64);
    let composed = ComposedReasoner::new(english, loaded);

    let expected = |word: &str| -> Vec<ConceptId> {
        let mut v = english.lookup(word).to_vec();
        if let Some(ids) = overlay.get(word) {
            v.extend_from_slice(ids);
        }
        v
    };

    // Leg 1 — EVERY English word (the full WordIndex sweep, ~131.8k words, plus
    // the function words known_words adds): identical to English's own read,
    // extended by the overlay exactly where a loaded surface collides. This is
    // also the dedup pin: pre-overlay, 78 function∩WordNet words (e.g. "a")
    // returned their English ids twice.
    let mut english_words = 0usize;
    let mut collisions = 0usize;
    for word in english.known_words() {
        let want = expected(word);
        assert_eq!(
            composed.lookup(word),
            want.as_slice(),
            "resolve({word:?}) must equal english.lookup ++ overlay, order pinned"
        );
        english_words += 1;
        if overlay.contains_key(word) {
            collisions += 1;
        }
    }
    assert!(
        english_words > 100_000,
        "the sweep must cover the real WordIndex, not a sample (saw {english_words})"
    );

    // Leg 2 — EVERY loaded surface (the overlay keys): resolves to the union,
    // never shadowing English (a collision keeps English's ids as the prefix).
    for surface in overlay.keys() {
        let want = expected(surface);
        assert_eq!(
            composed.lookup(surface),
            want.as_slice(),
            "loaded surface {surface:?} must resolve to english ++ overlay, order pinned"
        );
        assert!(
            !composed.lookup(surface).is_empty(),
            "a loaded surface always resolves"
        );
    }
    eprintln!(
        "SURFACE-OVERLAY SWEEP: {english_words} English words + {} loaded surfaces \
         ({collisions} collisions) all resolve to the pinned ordered union",
        overlay.len()
    );

    // Leg 3 — the four classes were all really present in the sweep (Honest:
    // an empty overlay or a collision-free corpus would weaken the oracle).
    assert!(!overlay.is_empty(), "the loaded title mints surfaces");
    assert!(collisions > 0, "some loaded surface collides with English");
    let loaded_only = overlay
        .keys()
        .filter(|s| english.lookup(s).is_empty())
        .count();
    assert!(loaded_only > 0, "some loaded surface is loaded-only");
}
