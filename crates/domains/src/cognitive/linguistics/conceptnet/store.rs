//! The loaded ConceptNet association table, indexed for the corroboration
//! query: do two WordNet concepts share ANY ConceptNet assertion?
//!
//! ## Why lemma-indexed, not `ConceptId`-indexed like VerbNet
//!
//! [`crate::cognitive::linguistics::verbnet::store`]'s crosswalk resolves to
//! an exact `ConceptId` at bundling time because VerbNet's `wn=` attribute is
//! a Princeton WordNet SENSE key — unambiguous, one sense-key names exactly
//! one synset. ConceptNet's `/c/en/word` node URIs are, in the overwhelming
//! common case, NOT sense-disambiguated (Speer, Chin & Havasi 2017 §3.1: node
//! merging is deliberately lemma-level, not sense-level, to keep the graph
//! dense) — a single ConceptNet node can correspond to several WordNet
//! synsets for a polysemous word. Precomputing a sense-exact crosswalk the
//! way VerbNet's regen does is therefore not meaningful here: there is no
//! single correct target sense to resolve to.
//!
//! Instead, the store indexes by NORMALIZED SURFACE LEMMA (the same key
//! space the committed `.assoc` TSV already uses, since the WordNet-crosswalk
//! filter that produced it is itself lemma-level — see
//! `super::regenerate`). A query resolves its `ConceptId` to its synset's
//! lemma set live, via the already-loaded [`LexicalReasoner`], and checks
//! whether ANY of that synset's lemmas has a ConceptNet edge to ANY of the
//! other concept's lemmas — the same "any synonym counts" latitude ConceptNet
//! itself takes when merging nodes.

#[allow(unused_imports)]
use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec::Vec,
};

use super::ontology::ConceptNet;
use crate::cognitive::linguistics::english::{ConceptId, LexicalReasoner};

/// Normalize a surface lemma into ConceptNet's own `/c/en/…` node-key
/// convention: lowercase, with spaces and hyphens folded to `_` (verified
/// byte-exact against the real fetched corpus — `well-being` and `well being`
/// both surface as the ConceptNet node `well_being`; see
/// `super::regenerate`'s module doc for the verification command). The
/// SAME function bundling time (`super::regenerate::regenerate_conceptnet_archive`)
/// and query time (this module) both call — a single canonical
/// implementation, so the two can never drift apart.
#[must_use]
pub fn normalize_lemma(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' | '-' => '_',
            c => c.to_ascii_lowercase(),
        })
        .collect()
}

/// The loaded, indexed ConceptNet association table — the corroboration
/// mechanism's query surface. A symmetric adjacency list (every edge
/// recorded in both directions): ConceptNet relations are mapped generically
/// onto the existing `Association` relation kind
/// (`crate::formal::relations::ontology::Association`, SKOS `related`),
/// which is itself Symmetric — so an undirected "is there ANY assertion
/// between these two lemmas" is the query this store answers, not "in which
/// direction".
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConceptNetStore {
    associations: BTreeMap<String, BTreeSet<String>>,
}

impl ConceptNetStore {
    /// Build the indexed store from the typed, reader-produced [`ConceptNet`]
    /// edge list.
    #[must_use]
    pub fn from_conceptnet(cn: &ConceptNet) -> Self {
        let mut associations: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for edge in &cn.edges {
            associations
                .entry(edge.start_lemma.clone())
                .or_default()
                .insert(edge.end_lemma.clone());
            associations
                .entry(edge.end_lemma.clone())
                .or_default()
                .insert(edge.start_lemma.clone());
        }
        Self { associations }
    }

    /// Does `concept` have ANY ConceptNet node at all (under any of its
    /// synset's lemmas)? The epistemic distinction
    /// [`ConceptNetStore::shares_association`]'s `false` alone can't make —
    /// mirrors [`crate::cognitive::linguistics::verbnet::store::VerbNetStore::has_coverage`]'s
    /// same rationale: "no data for this concept" and "queried, no
    /// connection" are different facts the corroboration composition rule
    /// needs told apart.
    #[must_use]
    pub fn has_coverage(&self, en: &dyn LexicalReasoner, concept: ConceptId) -> bool {
        let Some(view) = en.concept(concept) else {
            return false;
        };
        view.lemmas()
            .any(|lemma| self.associations.contains_key(&normalize_lemma(lemma)))
    }

    /// Does `concept_a` share ANY ConceptNet assertion with `concept_b` —
    /// i.e. does some lemma of `a`'s synset have a recorded edge to some
    /// lemma of `b`'s synset?
    #[must_use]
    pub fn shares_association(
        &self,
        en: &dyn LexicalReasoner,
        concept_a: ConceptId,
        concept_b: ConceptId,
    ) -> bool {
        let (Some(view_a), Some(view_b)) = (en.concept(concept_a), en.concept(concept_b)) else {
            return false;
        };
        let lemmas_b: BTreeSet<String> = view_b.lemmas().map(normalize_lemma).collect();
        view_a.lemmas().map(normalize_lemma).any(|lemma_a| {
            self.associations
                .get(&lemma_a)
                .is_some_and(|neighbors| neighbors.iter().any(|n| lemmas_b.contains(n)))
        })
    }
}

/// The process-wide loaded ConceptNet store — the committed `conceptnet@5.7.0`
/// `.prx` decoded, parsed, and indexed once. Mirrors
/// [`crate::cognitive::linguistics::verbnet::store::verbnet_loaded`]'s caching
/// shape: built lazily on first use, reused for the process lifetime.
#[cfg(feature = "std")]
pub fn conceptnet_loaded() -> &'static ConceptNetStore {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<ConceptNetStore> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        use crate::applied::data_provisioning::decoders::plaintext_tsv;
        use crate::applied::data_provisioning::raw_source_prx::raw_source_bytes_embedded;

        const CONCEPTNET_PRX: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/conceptnet/conceptnet-5.7.0.prx"
        ));

        let bytes = raw_source_bytes_embedded("conceptnet", "5.7.0", CONCEPTNET_PRX);
        let records = plaintext_tsv::decode(&bytes)
            .unwrap_or_else(|e| panic!("conceptnet committed .prx archive failed to decode: {e}"));
        let cn = super::reader::read_conceptnet(&records);
        ConceptNetStore::from_conceptnet(&cn)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::conceptnet::ontology::ConceptNetEdge;
    use crate::cognitive::linguistics::english::English;
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;

    fn edge(rel: &str, a: &str, b: &str, w: f32) -> ConceptNetEdge {
        ConceptNetEdge {
            relation: rel.to_string(),
            start_lemma: a.to_string(),
            end_lemma: b.to_string(),
            weight: w,
        }
    }

    fn fixture_conceptnet() -> ConceptNet {
        ConceptNet {
            edges: alloc::vec![
                edge("RelatedTo", "cut", "sever", 1.0),
                edge("IsA", "cut", "action", 2.0),
            ],
        }
    }

    fn fixture_reasoner() -> English {
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-cut-v"><Lemma writtenForm="cut" partOfSpeech="v"/><Sense id="cut-v-1" synset="s-cut"/></LexicalEntry>
    <LexicalEntry id="e-sever-v"><Lemma writtenForm="sever" partOfSpeech="v"/><Sense id="sever-v-1" synset="s-sever"/></LexicalEntry>
    <LexicalEntry id="e-eat-v"><Lemma writtenForm="eat" partOfSpeech="v"/><Sense id="eat-v-1" synset="s-eat"/></LexicalEntry>
    <Synset id="s-cut" ili="i1" partOfSpeech="v"><Definition>cease, stop</Definition></Synset>
    <Synset id="s-sever" ili="i2" partOfSpeech="v"><Definition>cut off</Definition></Synset>
    <Synset id="s-eat" ili="i3" partOfSpeech="v"><Definition>consume food</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        English::from_wordnet(&read_wordnet(LMF).expect("LMF parses"))
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn normalize_folds_space_and_hyphen_to_underscore_and_lowercases() {
        assert_eq!(normalize_lemma("well-being"), "well_being");
        assert_eq!(normalize_lemma("well being"), "well_being");
        assert_eq!(normalize_lemma("Turkish bath"), "turkish_bath");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn shares_association_finds_a_direct_edge() {
        let store = ConceptNetStore::from_conceptnet(&fixture_conceptnet());
        let en = fixture_reasoner();
        let cut = en.lookup("cut")[0];
        let sever = en.lookup("sever")[0];
        assert!(store.shares_association(&en, cut, sever));
        // Symmetric: order doesn't matter.
        assert!(store.shares_association(&en, sever, cut));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn two_concepts_with_no_shared_edge_return_false_not_a_false_positive() {
        let store = ConceptNetStore::from_conceptnet(&fixture_conceptnet());
        let en = fixture_reasoner();
        let cut = en.lookup("cut")[0];
        let eat = en.lookup("eat")[0];
        assert!(!store.shares_association(&en, cut, eat));
        // "cut" has coverage; "eat" does not — a real "queried, no
        // connection", not a "no data" case, for the covered side.
        assert!(store.has_coverage(&en, cut));
        assert!(!store.has_coverage(&en, eat));
    }
}
