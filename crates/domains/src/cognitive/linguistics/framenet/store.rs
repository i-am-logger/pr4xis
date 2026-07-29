//! The loaded FrameNet lexical-unit + frame-relation graph, indexed for the
//! corroboration query: do two WordNet concepts evoke the same frame, or
//! frames directly related by any of the 9 frame-to-frame relation types?
//!
//! ## Why lemma-indexed (and POS-checked), not `ConceptId`-indexed like VerbNet
//!
//! Same reasoning as
//! [`crate::cognitive::linguistics::conceptnet::store`]'s module doc:
//! FrameNet's lexical units carry NO native WordNet sense-key or synset
//! reference (confirmed 2026-07-13 against the official NLTK FrameNet
//! reader and corroborated by the existence of a whole separate research
//! literature on FrameNet-WordNet alignment, e.g. Ferrández et al. 2010
//! LREC "Aligning FrameNet and WordNet based on Semantic Neighborhoods" —
//! work that would be unnecessary if a native link existed). A
//! `ConceptId`-precise crosswalk the way VerbNet's regen builds one is
//! therefore not meaningful here either.
//!
//! Unlike ConceptNet's bare nodes, FrameNet lexical units DO carry real POS
//! information (`POS="V"` etc. on the source `<lexUnit>` element) — this
//! store uses it, indexing by `(lemma, LmfPos)` rather than lemma alone, so
//! a WordNet noun concept can never spuriously match a same-spelled verb's
//! FrameNet membership (e.g. "run" the noun vs. `run.v`).

#[allow(unused_imports)]
use alloc::{
    collections::{BTreeMap, BTreeSet},
    string::{String, ToString},
    vec::Vec,
};

use super::ontology::FrameNet;
use crate::cognitive::linguistics::english::{ConceptId, LexicalReasoner};
use crate::social::software::markup::xml::lmf::LmfPos;

/// Normalize a surface lemma for lookup: lowercase, spaces and hyphens
/// folded to `_`. Mirrors
/// [`crate::cognitive::linguistics::conceptnet::store::normalize_lemma`]'s
/// exact behavior — kept as a small local copy rather than a cross-source
/// dependency (VerbNet, ConceptNet and FrameNet are peer instance-data
/// loaders with no reason to import from one another; the transform itself
/// is a five-line generic surface-canonicalization, not source-specific
/// logic).
#[must_use]
pub fn normalize_lemma(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' | '-' => '_',
            c => c.to_ascii_lowercase(),
        })
        .collect()
}

/// The loaded, indexed FrameNet data — the corroboration mechanism's query
/// surface. `frame_neighbors` is a symmetric adjacency list (every relation
/// recorded in both directions): FrameNet's 9 relation types are mapped
/// generically onto the existing `Association` relation kind
/// (`crate::formal::relations::ontology::Association`, SKOS `related`),
/// itself Symmetric — the SAME generic-mapping discipline ConceptNet's
/// store applies (see that module's `shares_association` doc), rather than
/// distinguishing Inheritance's genuine hierarchy from the other 8 lateral
/// relation types.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrameNetStore {
    lu_frames: BTreeMap<(String, LmfPos), BTreeSet<String>>,
    frame_neighbors: BTreeMap<String, BTreeSet<String>>,
}

impl FrameNetStore {
    /// Build the indexed store from the typed, reader-produced [`FrameNet`]
    /// data.
    #[must_use]
    pub fn from_framenet(fd: &FrameNet) -> Self {
        let mut lu_frames: BTreeMap<(String, LmfPos), BTreeSet<String>> = BTreeMap::new();
        for lu in &fd.lexical_units {
            lu_frames
                .entry((normalize_lemma(&lu.lemma), lu.pos))
                .or_default()
                .insert(lu.frame.clone());
        }
        let mut frame_neighbors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for rel in &fd.relations {
            frame_neighbors
                .entry(rel.sub_frame.clone())
                .or_default()
                .insert(rel.super_frame.clone());
            frame_neighbors
                .entry(rel.super_frame.clone())
                .or_default()
                .insert(rel.sub_frame.clone());
        }
        Self {
            lu_frames,
            frame_neighbors,
        }
    }

    /// The frame set `concept` evokes, keyed by its own `(lemma, pos)`
    /// pairs (a synset can have several lemmas; each lemma at this
    /// concept's POS may independently carry FrameNet membership).
    fn frames_for(&self, en: &dyn LexicalReasoner, concept: ConceptId) -> BTreeSet<&String> {
        let Some(view) = en.concept(concept) else {
            return BTreeSet::new();
        };
        let pos = view.pos();
        view.lemmas()
            .filter_map(|lemma| self.lu_frames.get(&(normalize_lemma(lemma), pos)))
            .flatten()
            .collect()
    }

    /// Does `concept` have ANY FrameNet lexical-unit membership at all
    /// (under any of its synset's lemmas, at its own POS)? The epistemic
    /// distinction [`FrameNetStore::shares_frame_family`]'s `false` alone
    /// can't make — mirrors
    /// [`crate::cognitive::linguistics::verbnet::store::VerbNetStore::has_coverage`]
    /// and
    /// [`crate::cognitive::linguistics::conceptnet::store::ConceptNetStore::has_coverage`]'s
    /// same rationale.
    #[must_use]
    pub fn has_coverage(&self, en: &dyn LexicalReasoner, concept: ConceptId) -> bool {
        !self.frames_for(en, concept).is_empty()
    }

    /// Does `concept_a` share a frame family with `concept_b` — i.e. do
    /// they evoke the SAME frame, or frames connected by a direct
    /// frame-to-frame relation edge (any of the 9 types, one hop, no
    /// transitive closure — mirrors ConceptNet's flat "any assertion"
    /// latitude, not VerbNet's nested-class-tree ancestor walk, since
    /// FrameNet's own relation structure is a general graph, not a clean
    /// single-parent hierarchy)?
    #[must_use]
    pub fn shares_frame_family(
        &self,
        en: &dyn LexicalReasoner,
        a: ConceptId,
        b: ConceptId,
    ) -> bool {
        let frames_a = self.frames_for(en, a);
        let frames_b = self.frames_for(en, b);
        if frames_a.iter().any(|f| frames_b.contains(f)) {
            return true;
        }
        frames_a.iter().any(|fa| {
            self.frame_neighbors
                .get(*fa)
                .is_some_and(|neighbors| frames_b.iter().any(|fb| neighbors.contains(*fb)))
        })
    }
}

/// The process-wide loaded FrameNet store — the committed `framenet@1.7`
/// `.prx` decoded, parsed, and indexed once. Mirrors
/// [`crate::cognitive::linguistics::conceptnet::store::conceptnet_loaded`]'s
/// caching shape: built lazily on first use, reused for the process
/// lifetime.
#[cfg(feature = "std")]
pub fn framenet_loaded() -> &'static FrameNetStore {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<FrameNetStore> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        use crate::applied::data_provisioning::decoders::plaintext_tsv;
        use crate::applied::data_provisioning::raw_source_prx::raw_source_bytes_embedded;

        const FRAMENET_PRX: &[u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/framenet/framenet-1.7.prx"
        ));

        let bytes = raw_source_bytes_embedded("framenet", "1.7", FRAMENET_PRX);
        let records = plaintext_tsv::decode(&bytes)
            .unwrap_or_else(|e| panic!("framenet committed .prx archive failed to decode: {e}"));
        let fd = super::reader::read_framenet(&records);
        FrameNetStore::from_framenet(&fd)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::english::English;
    use crate::cognitive::linguistics::framenet::ontology::{
        FrameNetLexicalUnit, FrameNetRelation,
    };
    use crate::social::software::markup::xml::lmf::reader::read_wordnet;

    fn lu(lemma: &str, pos: LmfPos, frame: &str) -> FrameNetLexicalUnit {
        FrameNetLexicalUnit {
            lemma: lemma.to_string(),
            pos,
            frame: frame.to_string(),
        }
    }

    fn rel(relation: &str, sub: &str, sup: &str) -> FrameNetRelation {
        FrameNetRelation {
            relation: relation.to_string(),
            sub_frame: sub.to_string(),
            super_frame: sup.to_string(),
        }
    }

    fn fixture_reasoner() -> English {
        const LMF: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<LexicalResource>
  <Lexicon id="t" label="T" language="en" version="1.0">
    <LexicalEntry id="e-cause-v"><Lemma writtenForm="cause" partOfSpeech="v"/><Sense id="cause-v-1" synset="s-cause"/></LexicalEntry>
    <LexicalEntry id="e-bring-about-v"><Lemma writtenForm="bring_about" partOfSpeech="v"/><Sense id="bring-about-v-1" synset="s-bring-about"/></LexicalEntry>
    <LexicalEntry id="e-eat-v"><Lemma writtenForm="eat" partOfSpeech="v"/><Sense id="eat-v-1" synset="s-eat"/></LexicalEntry>
    <LexicalEntry id="e-run-n"><Lemma writtenForm="run" partOfSpeech="n"/><Sense id="run-n-1" synset="s-run-n"/></LexicalEntry>
    <Synset id="s-cause" ili="i1" partOfSpeech="v"><Definition>bring about</Definition></Synset>
    <Synset id="s-bring-about" ili="i2" partOfSpeech="v"><Definition>cause to happen</Definition></Synset>
    <Synset id="s-eat" ili="i3" partOfSpeech="v"><Definition>consume food</Definition></Synset>
    <Synset id="s-run-n" ili="i4" partOfSpeech="n"><Definition>an act of running</Definition></Synset>
  </Lexicon>
</LexicalResource>"#;
        English::from_wordnet(&read_wordnet(LMF).expect("LMF parses"))
    }

    fn fixture_framenet_same_frame() -> FrameNetStore {
        FrameNetStore::from_framenet(&FrameNet {
            lexical_units: alloc::vec![
                lu("cause", LmfPos::Verb, "Causation"),
                lu("bring_about", LmfPos::Verb, "Causation"),
            ],
            relations: Vec::new(),
        })
    }

    fn fixture_framenet_related_frames() -> FrameNetStore {
        FrameNetStore::from_framenet(&FrameNet {
            lexical_units: alloc::vec![
                lu("cause", LmfPos::Verb, "Causation"),
                lu("bring_about", LmfPos::Verb, "Cause_to_happen"),
            ],
            relations: alloc::vec![rel("Inheritance", "Cause_to_happen", "Causation")],
        })
    }

    fn fixture_framenet_unrelated() -> FrameNetStore {
        FrameNetStore::from_framenet(&FrameNet {
            lexical_units: alloc::vec![
                lu("cause", LmfPos::Verb, "Causation"),
                lu("eat", LmfPos::Verb, "Ingestion"),
                lu("bring_about", LmfPos::Verb, "Unrelated_frame"),
            ],
            relations: Vec::new(),
        })
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn normalize_folds_space_and_hyphen_to_underscore_and_lowercases() {
        assert_eq!(normalize_lemma("bring about"), "bring_about");
        assert_eq!(normalize_lemma("Well-Known"), "well_known");
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn shares_frame_family_finds_direct_same_frame_membership() {
        let store = fixture_framenet_same_frame();
        let en = fixture_reasoner();
        let cause = en.lookup("cause")[0];
        let bring_about = en.lookup("bring_about")[0];
        assert!(store.shares_frame_family(&en, cause, bring_about));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn shares_frame_family_finds_a_one_hop_relation() {
        let store = fixture_framenet_related_frames();
        let en = fixture_reasoner();
        let cause = en.lookup("cause")[0];
        let bring_about = en.lookup("bring_about")[0];
        assert!(store.shares_frame_family(&en, cause, bring_about));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn two_concepts_with_no_shared_or_related_frame_return_false() {
        let store = fixture_framenet_unrelated();
        let en = fixture_reasoner();
        let cause = en.lookup("cause")[0];
        let eat = en.lookup("eat")[0];
        assert!(!store.shares_frame_family(&en, cause, eat));
        assert!(store.has_coverage(&en, cause));
        assert!(store.has_coverage(&en, eat));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn pos_mismatch_never_spuriously_matches() {
        // "run" the NOUN must not pick up "cause"'s VERB-keyed FrameNet
        // membership just because both happen to be in the same store.
        let store = fixture_framenet_unrelated();
        let en = fixture_reasoner();
        let run_n = en.lookup("run")[0];
        assert!(!store.has_coverage(&en, run_n));
    }
}
