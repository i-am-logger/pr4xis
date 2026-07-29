//! FULL-CORPUS surface-overlay sweep — the strongest oracle for the
//! `ComposedReasoner`'s loaded-only overlay: over the REAL packed `WordIndex`
//! (every one of English's ~131.8k words) and a REAL loaded USC title, every
//! word resolves to EXACTLY
//!
//! ```text
//! composed.lookup(word) == english.lookup(word) ++ overlay(word)
//! ```
//!
//! with English's ids first (packed run order), then the loaded ids in mint
//! order. This is the `#[test]` driver for the registered, cited
//! [`ComposedSurfaceUnionFaithfulOnRealCorpus`] axiom — its `verify()` runs the
//! sweep, re-deriving the overlay expectation INDEPENDENTLY of the reasoner, so
//! a union-order break, a dropped English fall-through, or a lost loaded surface
//! each fails it. The test `require()`-gates on a provisioned USC title, so an
//! unprovisioned checkout hard-fails with the `pr4xis update usc` hint (the
//! crate's "tests do not skip" contract), never a silent pass.

use pr4xis::ontology::Axiom;
use pr4xis_domains::applied::data_provisioning::registry::data_sources;
use pr4xis_domains::cognitive::linguistics::composed::ComposedSurfaceUnionFaithfulOnRealCorpus;
use pr4xis_domains::formal::meta::source_taxonomy::ontology::SourceTaxonomyConcept;
use pr4xis_domains::social::software::markup::xml::uslm::{UsCode, read_uslm_title};
use praxis_corpus_tests::{require, workspace_root};

/// The first provisioned USC title (`None` on a fresh checkout routes through
/// [`require`] to hard-fail — tests do not skip). The same title
/// `ComposedSurfaceUnionFaithfulOnRealCorpus::verify()` loads internally.
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

#[test]
fn every_english_word_resolves_to_the_ordered_union_over_the_real_corpus() {
    require(first_provisioned_title(), "usc");
    assert!(ComposedSurfaceUnionFaithfulOnRealCorpus.verify().is_ok());
}
