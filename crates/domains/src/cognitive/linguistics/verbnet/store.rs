//! The loaded VerbNet class hierarchy, indexed for the corroboration query:
//! do two WordNet concepts' verbs share a VerbNet class family?
//!
//! Mirrors [`crate::cognitive::linguistics::english::ontology::english_loaded`]'s
//! shape: a process-wide cached, flattened, indexed view over the typed
//! [`super::ontology::VerbNet`] tree the reader produces — built once, queried
//! many times.
//!
//! ## The WordNet sense-key crosswalk
//!
//! VerbNet's `<MEMBER wn="cut%2:30:00">` attribute carries a Princeton
//! WordNet SENSE key: `lemma%ss_type:lex_filenum:lex_id[:head_word:head_id]`
//! (Fellbaum 1998, WNDB(5WN) `sense key` format; `ss_type` 1=noun, 2=verb,
//! 3=adjective, 4=adverb, 5=adjective-satellite — VerbNet members are always
//! `2`). Resolving that to this codebase's `ConceptId` (concepts are keyed by
//! SYNSET, not individual sense) takes two steps: sense key -> OEWN `Sense`
//! id (mechanical, [`oewn_sense_id_for_sense_key`]) -> synset id -> `ConceptId`
//! (needs the loaded WordNet data, since `English::from_wordnet` intentionally
//! discards the raw `Sense.id` string after construction — see that
//! function's own disposal comment).
//!
//! Both steps are PRECOMPUTED once, offline, by
//! `verbnet_class_collection::regenerate::regenerate_verbnet_archive`
//! (which can afford the one-time ~89 MB WordNet XML parse a normal runtime
//! load path is built to avoid), and bundled as one more entry
//! (`WORDNET_CROSSWALK_PATH`) inside the same archived VerbNet collection —
//! a small `sense_key\t<ConceptId numeric value>` TSV, resolved for only the
//! ~5,800 sense-keys VerbNet members actually reference.
//!
//! The crosswalk targets `ConceptId`'s numeric VALUE, not the OEWN synset-id
//! STRING — verified empirically (`ConceptView::original_id()` only returns
//! the real `"oewn-…"` string on a freshly-built, raw-XML-parsed `English`;
//! the compact/store-bundle archive `english_loaded()` normally serves at
//! runtime returns a SYNTHETIC placeholder instead, per
//! `lmf::compact::decode`'s own doc comment: "index-derived synthetic ids in
//! place of the original `oewn-…` strings ... the only difference is
//! `Concept::original_id`"). `ConceptId` itself, by that same doc's explicit
//! guarantee, IS identical between the raw and compact/store-bundle load
//! paths ("same `ConceptId`s, same relations") — so it, not the display
//! string, is the crosswalk's stable target.

#[allow(unused_imports)]
use alloc::{
    collections::{BTreeMap, BTreeSet},
    format,
    string::{String, ToString},
    vec::Vec,
};

use super::ontology::{VerbNet, VerbNetClass};
use crate::cognitive::linguistics::english::ConceptId;

/// Convert a Princeton WordNet sense key (VerbNet's `wn=` token, e.g.
/// `"cut%2:30:00"`) into the Open English WordNet 2025 `Sense` id it
/// corresponds to (e.g. `"oewn-cut__2.30.00.."`). `None` if the token isn't
/// shaped like a sense key at all (a defensive fail-closed return, not
/// expected on real VerbNet data — every `wn=` token this codebase has
/// observed is a well-formed sense key).
///
/// See the module doc for the citation and the byte-exact verification this
/// transform is grounded in.
pub fn oewn_sense_id_for_sense_key(sense_key: &str) -> Option<String> {
    let (lemma, rest) = sense_key.split_once('%')?;
    if lemma.is_empty() {
        return None;
    }
    let mut fields: Vec<&str> = rest.split(':').collect();
    if fields.is_empty() || fields.len() > 5 {
        return None;
    }
    while fields.len() < 5 {
        fields.push("");
    }
    Some(format!("oewn-{lemma}__{}", fields.join(".")))
}

/// Parse the precomputed `sense_key\t<ConceptId numeric value>` crosswalk TSV
/// (the module doc explains why this is loaded as precomputed data rather
/// than resolved live). Malformed lines (wrong column count, non-numeric
/// second column) are skipped, fail-closed, rather than panicking — the same
/// discipline every other TSV reader in this codebase applies.
fn parse_crosswalk_tsv(text: &str) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once('\t')
            && let Ok(concept_value) = value.parse::<u64>()
        {
            map.insert(key.to_string(), concept_value);
        }
    }
    map
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IndexedClass {
    parent: Option<String>,
}

/// The loaded, indexed VerbNet class hierarchy — the corroboration
/// mechanism's query surface. Construction flattens the recursive
/// [`VerbNetClass`] tree into a lookup table (class id → parent id) plus a
/// reverse index (`ConceptId` numeric value → the class ids some sense-key
/// resolving to that concept is a DIRECT member of), so
/// [`VerbNetStore::shares_class_family`] is O(ancestors) per query, not a
/// tree walk.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerbNetStore {
    classes: BTreeMap<String, IndexedClass>,
    /// `ConceptId::value()` -> the class ids it's a direct member of
    /// (usually one, but a member can be declared in more than one class, or
    /// a concept can be reached by more than one sense-key, in real
    /// VerbNet/WordNet data — kept as a `Vec`, not assumed unique).
    concept_to_classes: BTreeMap<u64, Vec<String>>,
}

impl VerbNetStore {
    /// Build the indexed store from the typed, reader-produced [`VerbNet`]
    /// tree plus the precomputed sense-key -> `ConceptId`-value crosswalk
    /// table (see module doc) — the one place the recursive class structure
    /// is flattened and each member's sense-keys are resolved to concepts.
    #[must_use]
    pub fn from_verbnet_and_crosswalk(vn: &VerbNet, crosswalk: &BTreeMap<String, u64>) -> Self {
        let mut classes = BTreeMap::new();
        let mut concept_to_classes: BTreeMap<u64, Vec<String>> = BTreeMap::new();

        for top in &vn.classes {
            index_class(top, None, crosswalk, &mut classes, &mut concept_to_classes);
        }

        Self {
            classes,
            concept_to_classes,
        }
    }

    /// This class and every ancestor above it (nearest-first, self
    /// included) — `stop-55.4-1-1` -> `["stop-55.4-1-1", "stop-55.4-1",
    /// "stop-55.4"]`. Terminates at a class with no recorded parent (a
    /// top-level `<VNCLASS>`); a cyclic parent chain (never observed in real
    /// VerbNet data, and structurally impossible from a tree-shaped reader
    /// output) would loop forever, so this also guards with a visited set,
    /// fail-closed rather than trusting the invariant silently.
    fn self_and_ancestors(&self, class_id: &str) -> Vec<String> {
        let mut chain = Vec::new();
        let mut seen = BTreeSet::new();
        let mut current = Some(class_id.to_string());
        while let Some(id) = current {
            if !seen.insert(id.clone()) {
                break; // defensive: a cycle would otherwise loop forever
            }
            let parent = self.classes.get(&id).and_then(|c| c.parent.clone());
            chain.push(id);
            current = parent;
        }
        chain
    }

    /// Does `concept_a` share a VerbNet class family with `concept_b` — i.e.
    /// does a class either is a DIRECT member of have an ancestor-or-self in
    /// common with a class the other is a direct member of? Returns the
    /// shared (nearest, if several) common ancestor class id as the citable
    /// witness, or `None` if the two share no class family (which includes
    /// the case where one or both concepts have NO VerbNet membership at all
    /// — callers distinguish "no common class" from "no VerbNet data for
    /// this concept" via [`VerbNetStore::has_coverage`], not via this
    /// method's `None`).
    #[must_use]
    pub fn shares_class_family(
        &self,
        concept_a: ConceptId,
        concept_b: ConceptId,
    ) -> Option<String> {
        let classes_a = self.concept_to_classes.get(&concept_a.value())?;
        let classes_b = self.concept_to_classes.get(&concept_b.value())?;

        let mut best: Option<String> = None;
        for class_a in classes_a {
            let ancestors_a = self.self_and_ancestors(class_a);
            for class_b in classes_b {
                let ancestors_b = self.self_and_ancestors(class_b);
                // Nearest common ancestor: walk A's chain nearest-first,
                // take the first entry that also appears in B's chain.
                if let Some(shared) = ancestors_a.iter().find(|a| ancestors_b.contains(a)) {
                    best = Some(shared.clone());
                    break;
                }
            }
            if best.is_some() {
                break;
            }
        }
        best
    }

    /// Does `concept` have ANY VerbNet class membership at all? The
    /// epistemic distinction [`VerbNetStore::shares_class_family`]'s `None`
    /// alone can't make: a concept absent from VerbNet entirely ("no
    /// coverage") is a different fact than a concept that IS in VerbNet but
    /// shares no class with the other concept ("queried, no connection") —
    /// the composition rule the corroboration mechanism (tasks #37/#38)
    /// consults needs both states distinguished, not collapsed into one
    /// `Option`.
    #[must_use]
    pub fn has_coverage(&self, concept: ConceptId) -> bool {
        self.concept_to_classes.contains_key(&concept.value())
    }
}

fn index_class(
    class: &VerbNetClass,
    parent: Option<&str>,
    crosswalk: &BTreeMap<String, u64>,
    classes: &mut BTreeMap<String, IndexedClass>,
    concept_to_classes: &mut BTreeMap<u64, Vec<String>>,
) {
    classes.insert(
        class.id.clone(),
        IndexedClass {
            parent: parent.map(str::to_string),
        },
    );
    for member in &class.members {
        for sense_key in &member.wn_sense_keys {
            let Some(&concept_value) = crosswalk.get(sense_key) else {
                continue;
            };
            concept_to_classes
                .entry(concept_value)
                .or_default()
                .push(class.id.clone());
        }
    }
    for sub in &class.subclasses {
        index_class(
            sub,
            Some(class.id.as_str()),
            crosswalk,
            classes,
            concept_to_classes,
        );
    }
}

/// Decode the committed `verbnet@3.3` `.prx` archive into its raw
/// `path -> bytes` collection — the shared first step
/// [`verbnet_loaded`] and [`verbnet_classes_loaded`] both build on, factored
/// out so the embed-and-decode is written once.
#[cfg(feature = "std")]
fn decode_verbnet_archive()
-> crate::applied::data_provisioning::decoders::verbnet_class_collection::VerbNetClassCollection {
    use crate::applied::data_provisioning::decoders::verbnet_class_collection;
    use crate::applied::data_provisioning::raw_source_prx::raw_source_bytes_embedded;

    const VERBNET_PRX: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/verbnet/verbnet-3.3.prx"
    ));

    let bytes = raw_source_bytes_embedded("verbnet", "3.3", VERBNET_PRX);
    verbnet_class_collection::decode(&bytes)
        .unwrap_or_else(|e| panic!("verbnet committed .prx archive failed to decode: {e}"))
}

/// The process-wide loaded VerbNet store — the committed `verbnet@3.3` `.prx`
/// decoded, parsed, and indexed once (class hierarchy plus the bundled
/// sense-key -> `ConceptId` crosswalk, see module doc). Mirrors
/// [`crate::cognitive::linguistics::english::ontology::english_loaded`]'s
/// caching shape: built lazily on first use, reused for the process lifetime.
#[cfg(feature = "std")]
pub fn verbnet_loaded() -> &'static VerbNetStore {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<VerbNetStore> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        use crate::applied::data_provisioning::decoders::verbnet_class_collection::WORDNET_CROSSWALK_PATH;

        let collection = decode_verbnet_archive();

        let crosswalk_text = collection
            .iter()
            .find(|f| f.path == WORDNET_CROSSWALK_PATH)
            .map(|f| {
                core::str::from_utf8(&f.content)
                    .expect("verbnet wordnet crosswalk is UTF-8")
                    .to_string()
            })
            .unwrap_or_else(|| {
                panic!("verbnet committed .prx archive is missing {WORDNET_CROSSWALK_PATH}")
            });
        let crosswalk = parse_crosswalk_tsv(&crosswalk_text);

        let class_files: Vec<_> = collection
            .into_iter()
            .filter(|f| f.path != WORDNET_CROSSWALK_PATH)
            .collect();
        let vn = super::reader::read_verbnet(&class_files);
        VerbNetStore::from_verbnet_and_crosswalk(&vn, &crosswalk)
    })
}

/// The process-wide loaded raw VerbNet class TREE (not the flattened,
/// crosswalk-indexed [`VerbNetStore`]) — carries `theme_roles`/`frames`,
/// which the corroboration store's flattening step does not preserve. A
/// second cache over the SAME embedded archive bytes
/// [`verbnet_loaded`] decodes, so a caller needing the raw hierarchy (the
/// `defines` grounding lens's `basic_transitive_theme_order` query) does not
/// pay for `VerbNetStore`'s crosswalk indexing it does not need.
#[cfg(feature = "std")]
pub fn verbnet_classes_loaded() -> &'static VerbNet {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<VerbNet> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        use crate::applied::data_provisioning::decoders::verbnet_class_collection::WORDNET_CROSSWALK_PATH;

        let collection = decode_verbnet_archive();
        let class_files: Vec<_> = collection
            .into_iter()
            .filter(|f| f.path != WORDNET_CROSSWALK_PATH)
            .collect();
        super::reader::read_verbnet(&class_files)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::verbnet::ontology::VerbNetMember;

    fn member(name: &str, wn: &[&str]) -> VerbNetMember {
        VerbNetMember {
            name: name.to_string(),
            wn_sense_keys: wn.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn stop_55_4_fixture() -> VerbNet {
        VerbNet {
            classes: alloc::vec![VerbNetClass {
                id: "stop-55.4".into(),
                members: alloc::vec![member("cut", &["cut%2:30:00"])],
                subclasses: alloc::vec![VerbNetClass {
                    id: "stop-55.4-1".into(),
                    members: alloc::vec![member("halt", &["halt%2:38:05"])],
                    subclasses: alloc::vec![VerbNetClass {
                        id: "stop-55.4-1-1".into(),
                        members: alloc::vec![member("end", &["end%2:30:01", "end%2:36:13"])],
                        subclasses: Vec::new(),
                        theme_roles: Vec::new(),
                        frames: Vec::new(),
                    }],
                    theme_roles: Vec::new(),
                    frames: Vec::new(),
                }],
                theme_roles: Vec::new(),
                frames: Vec::new(),
            }],
        }
    }

    /// Test-only crosswalk: sense key -> a synthetic-but-realistic
    /// `ConceptId` numeric value (arbitrary small integers, not derived from
    /// real data — the real crosswalk is exercised separately by
    /// `crates/praxis-corpus-tests/tests/scratch_probe.rs`'s real-data probe).
    fn fixture_crosswalk() -> BTreeMap<String, u64> {
        [
            ("cut%2:30:00", 1001),
            ("halt%2:38:05", 1002),
            ("end%2:30:01", 1003),
            ("end%2:36:13", 1004),
            ("eat%2:34:00", 1005),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect()
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sense_key_crosswalk_matches_the_real_oewn_id_shape() {
        // Byte-exact-verified against crates/domains/data/wordnet/
        // english-wordnet-2025.xml: `cut%2:30:00` -> `oewn-cut__2.30.00..`
        // (Sense id whose synset is oewn-00293269-v).
        assert_eq!(
            oewn_sense_id_for_sense_key("cut%2:30:00").as_deref(),
            Some("oewn-cut__2.30.00..")
        );
        assert_eq!(
            oewn_sense_id_for_sense_key("kill%2:30:08").as_deref(),
            Some("oewn-kill__2.30.08..")
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn crosswalk_rejects_a_non_sense_key_without_panicking() {
        assert_eq!(oewn_sense_id_for_sense_key("not-a-sense-key"), None);
        assert_eq!(oewn_sense_id_for_sense_key(""), None);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn cut_and_end_share_the_top_level_class_as_common_ancestor() {
        // The exact real-world case this corroboration mechanism exists
        // for: cut (stop-55.4, top level) and end (stop-55.4-1-1, two
        // subclass levels deeper) share stop-55.4 as their nearest common
        // ancestor.
        let crosswalk = fixture_crosswalk();
        let store = VerbNetStore::from_verbnet_and_crosswalk(&stop_55_4_fixture(), &crosswalk);
        assert_eq!(
            store
                .shares_class_family(ConceptId::new(1001), ConceptId::new(1003))
                .as_deref(),
            Some("stop-55.4")
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn a_concept_with_no_verbnet_membership_has_no_coverage() {
        let crosswalk = fixture_crosswalk();
        let store = VerbNetStore::from_verbnet_and_crosswalk(&stop_55_4_fixture(), &crosswalk);
        assert!(store.has_coverage(ConceptId::new(1001)));
        assert!(!store.has_coverage(ConceptId::new(9999)));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn two_concepts_with_no_shared_class_return_none_not_a_false_positive() {
        let mut vn = stop_55_4_fixture();
        vn.classes.push(VerbNetClass {
            id: "eat-39.1".into(),
            members: alloc::vec![member("eat", &["eat%2:34:00"])],
            subclasses: Vec::new(),
            theme_roles: Vec::new(),
            frames: Vec::new(),
        });
        let crosswalk = fixture_crosswalk();
        let store = VerbNetStore::from_verbnet_and_crosswalk(&vn, &crosswalk);
        assert_eq!(
            store.shares_class_family(ConceptId::new(1001), ConceptId::new(1005)),
            None
        );
        // Both DO have coverage — it's a real "queried, no connection", not
        // a "no data" case.
        assert!(store.has_coverage(ConceptId::new(1001)));
        assert!(store.has_coverage(ConceptId::new(1005)));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn parse_crosswalk_tsv_skips_malformed_lines_without_panicking() {
        let text = "cut%2:30:00\t1001\nmalformed-line-no-tab\nend%2:30:01\tnot-a-number\nhalt%2:38:05\t1002\n\n";
        let map = parse_crosswalk_tsv(text);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("cut%2:30:00").copied(), Some(1001));
    }

    /// `verbnet_classes_loaded()` decodes the SAME committed archive
    /// `verbnet_loaded()` does, but returns the raw class tree (theme roles
    /// and frames intact) rather than the flattened, crosswalk-indexed
    /// `VerbNetStore` — proven by confirming the real `representation-110.1`
    /// class's "mean" Basic Transitive frame through it, byte-exact against
    /// the same archive `reader::tests::REPRESENTATION_110_1` verifies
    /// against directly.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn verbnet_classes_loaded_confirms_mean_via_the_real_archive() {
        let vn = verbnet_classes_loaded();
        assert_eq!(
            vn.basic_transitive_theme_order("mean"),
            Some("representation-110.1")
        );
    }
}
