//! Every non-taxonomy WordNet relation English holds — opposition, mereology, and
//! the ~25 `WordnetRelations` sub-maps — as ONE immutable, zero-copy family of
//! labelled CSRs.
//!
//! `English` historically owned these as ~27 separate `HashMap`s. Every one of
//! them is keyed by a *dense* id assigned at
//! [`from_wordnet`](super::ontology::English::from_wordnet) time — a
//! [`SenseId`](super::ontology::SenseId) or a [`ConceptId`],
//! both the same `repr(transparent)` single-`u64` [`Reference<4>`](crate::formal::information::ontology::Reference).
//! So the whole family is ONE instance of the shared
//! [`PackedCsrFamily`]: a
//! [`DenseId`](crate::formal::meta::packed_csr::DenseId)-indexed family of
//! [`PodRun`]`<ConceptId>` columns
//! labelled by [`RelationKind`], packed into one buffer at load. The zero-copy
//! `&[ConceptId]` cast, the CSR reader, the little-endian invariant, and the
//! owned fallback all live in that one hand-audited generic; [`rel`](RelationStore::rel)
//! is the labelled-column read.

use alloc::vec::Vec;

use hashbrown::HashMap;

use super::ontology::{ConceptId, WordnetRelations};
use crate::formal::meta::packed_csr::{LabelKind, PackedCsrFamily, PodRun};

/// The labelled family of relations `English` holds — the tag on each CSR. The
/// discriminant order IS the layout order ([`ALL`](Self::ALL)); `kind as usize`
/// indexes the per-relation metadata, so the enum order and `ALL` MUST agree with
/// the `normalize` bundle below.
///
/// Literature: the relation identities are the Global WordNet Association LMF
/// relation set (Fellbaum 1998; Fellbaum-Osherson-Clark 2009 for `derivation`;
/// Bentivogli & Pianta 2004 for the domain pointers) — see [`WordnetRelations`].
///
/// # Why this projection is Rust, not `.prx` functor data
///
/// The relType→kind mapping this enum realizes is NOT the `english_functor`
/// case (a cross-ontology object-to-object projection carried as
/// content-addressed `.prx` data and applied by one interpreter): it maps the
/// loaded GWA-LMF relation vocabulary onto THIS store's internal dense-CSR
/// column layout — `kind as usize` IS the column index — so it is a storage
/// layout of one store, with no functor codomain ontology to address. Until
/// the functor-as-data machinery can carry store-layout projections, the
/// coupling is guarded in code: the const discriminant↔`ALL` pin below, the
/// exhaustive `WordnetRelations` destructure in `normalize` (a new field is a
/// compile error), the per-kind `every_kind_returns_its_own_distinct_edge`
/// fixture, and the loaded-DTD conformance test
/// (`every_relation_kind_grounds_in_the_loaded_wn_lmf_reltype_enumeration`),
/// which walks this enum against the registered `wn_lmf_dtd` relType
/// enumeration so the vocabulary itself stays LOADED, not encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationKind {
    /// Antonym opposition (sense-keyed): the `big ↔ small` pair.
    Opposition,
    /// Mereology whole → parts (concept-keyed): the `is_mereology` aggregate
    /// (holo/mero part + member + substance), in source order.
    MereologyParts,
    /// Derivation (sense-keyed): `compensate ↔ compensation`.
    Derivation,
    /// Pertainym (sense-keyed): relational adjective → noun base.
    Pertainym,
    /// Sense-level `similar`.
    SimilarSense,
    /// Sense-level `also`.
    AlsoSense,
    /// Sense-level `exemplifies` (instance-of).
    ExemplifiesSense,
    /// Sense-level `is_exemplified_by`.
    IsExemplifiedBySense,
    /// Sense-level `participle`.
    ParticipleSense,
    /// Synset-level `similar` (adjective satellites).
    SimilarSynset,
    /// Synset-level `also` (SKOS-style cross-reference; the codegen path's
    /// `RAW_REFERENCES` land here).
    AlsoSynset,
    /// Verb causation: `kill → die`.
    Causes,
    /// Inverse causation: `die ← kill`.
    IsCausedBy,
    /// Verb entailment: `walk → move`.
    Entails,
    /// Inverse entailment: `move ← walk`.
    IsEntailedBy,
    /// Attribute: adjective ↔ noun-attribute (`hot ↔ heat`).
    Attribute,
    /// Synset-level `exemplifies`.
    Exemplifies,
    /// Synset-level `is_exemplified_by`.
    IsExemplifiedBy,
    /// Topic-domain (`patent → law`).
    HasDomainTopic,
    /// Inverse topic-domain (`law → patent`).
    DomainTopic,
    /// Region-domain (`kangaroo → Australia`).
    HasDomainRegion,
    /// Inverse region-domain.
    DomainRegion,
    /// Synset-level `participle`.
    ParticipleSynset,
    /// HoloMember: collective → member (`forest ← tree`).
    HoloMember,
    /// HoloSubstance: substance-whole → constituent (`cake ← flour`).
    HoloSubstance,
    /// MeroMember: member → collective (`tree → forest`).
    MeroMember,
    /// MeroSubstance: constituent → substance-whole.
    MeroSubstance,
}

impl RelationKind {
    /// Every relation, in layout order — the SAME order the `normalize` bundle
    /// emits and the buffer packs. `kind as usize` is the index into it.
    pub const ALL: [RelationKind; 27] = [
        RelationKind::Opposition,
        RelationKind::MereologyParts,
        RelationKind::Derivation,
        RelationKind::Pertainym,
        RelationKind::SimilarSense,
        RelationKind::AlsoSense,
        RelationKind::ExemplifiesSense,
        RelationKind::IsExemplifiedBySense,
        RelationKind::ParticipleSense,
        RelationKind::SimilarSynset,
        RelationKind::AlsoSynset,
        RelationKind::Causes,
        RelationKind::IsCausedBy,
        RelationKind::Entails,
        RelationKind::IsEntailedBy,
        RelationKind::Attribute,
        RelationKind::Exemplifies,
        RelationKind::IsExemplifiedBy,
        RelationKind::HasDomainTopic,
        RelationKind::DomainTopic,
        RelationKind::HasDomainRegion,
        RelationKind::DomainRegion,
        RelationKind::ParticipleSynset,
        RelationKind::HoloMember,
        RelationKind::HoloSubstance,
        RelationKind::MeroMember,
        RelationKind::MeroSubstance,
    ];
}

/// The number of labelled relations (`= RelationKind::ALL.len()`).
const REL_COUNT: usize = 27;

/// Compile-time guard pinning the discriminant↔[`ALL`](RelationKind::ALL) leg of
/// the ordering coupling: [`LabelKind::index`] returns `self as usize`, which
/// indexes the family's per-column metadata, so the enum discriminant order MUST
/// equal the `ALL` layout order that drives the build/pack loops. A mid-enum
/// insertion that appended to `ALL` at the end would shift every following
/// discriminant one slot past its `ALL` entry — this loop diverges the two orders
/// and FAILS TO COMPILE. (The [`normalize`] bundle↔kind leg is pinned by the
/// per-kind labelling test.)
const _: () = {
    let mut i = 0;
    while i < REL_COUNT {
        assert!(RelationKind::ALL[i] as usize == i);
        i += 1;
    }
};

impl LabelKind for RelationKind {
    const COUNT: usize = REL_COUNT;
    fn index(self) -> usize {
        self as usize
    }
    fn all() -> &'static [Self] {
        &RelationKind::ALL
    }
}

/// Bundle the owned input maps into one `(row_count, map)` vector in the fixed
/// [`RelationKind::ALL`] order — the SINGLE place that couples a relation's tag to
/// its owned source map and its key space (sense- vs concept-keyed → row count).
/// Consumes every owned map.
///
/// Note `SenseId` and `ConceptId` are the same `Reference<4>`, so the sense-keyed
/// maps need no conversion — they are already `HashMap<ConceptId, Vec<ConceptId>>`.
fn normalize(
    opposition: HashMap<ConceptId, Vec<ConceptId>>,
    mereology_parts: HashMap<ConceptId, Vec<ConceptId>>,
    relations: WordnetRelations,
    sense_count: usize,
    concept_count: usize,
) -> Vec<(usize, HashMap<ConceptId, Vec<ConceptId>>)> {
    let WordnetRelations {
        derivation,
        pertainym,
        similar_sense,
        also_sense,
        exemplifies_sense,
        is_exemplified_by_sense,
        participle_sense,
        similar_synset,
        also_synset,
        causes,
        is_caused_by,
        entails,
        is_entailed_by,
        attribute,
        exemplifies,
        is_exemplified_by,
        has_domain_topic,
        domain_topic,
        has_domain_region,
        domain_region,
        participle_synset,
        holo_member,
        holo_substance,
        mero_member,
        mero_substance,
    } = relations;
    let s = sense_count;
    let c = concept_count;
    let bundle = alloc::vec![
        (s, opposition),
        (c, mereology_parts),
        (s, derivation),
        (s, pertainym),
        (s, similar_sense),
        (s, also_sense),
        (s, exemplifies_sense),
        (s, is_exemplified_by_sense),
        (s, participle_sense),
        (c, similar_synset),
        (c, also_synset),
        (c, causes),
        (c, is_caused_by),
        (c, entails),
        (c, is_entailed_by),
        (c, attribute),
        (c, exemplifies),
        (c, is_exemplified_by),
        (c, has_domain_topic),
        (c, domain_topic),
        (c, has_domain_region),
        (c, domain_region),
        (c, participle_synset),
        (c, holo_member),
        (c, holo_substance),
        (c, mero_member),
        (c, mero_substance),
    ];
    // One entry per relation, in `ALL` order — fires only on a construction bug.
    assert_eq!(
        bundle.len(),
        REL_COUNT,
        "relation bundle must have exactly one entry per RelationKind"
    );
    bundle
}

/// Every labelled relation as one dense-indexed, zero-copy CSR family. All
/// representation is the shared [`PackedCsrFamily`]; [`rel`](Self::rel) is the
/// labelled-column read.
pub struct RelationStore(PackedCsrFamily<RelationKind, PodRun<ConceptId>>);

impl RelationStore {
    /// Transcode the owned maps into the packed CSR family ONCE, consuming and
    /// freeing them. Each relation's per-id run is written in dense-id order, each
    /// id in its owned-map order, so [`rel`](Self::rel) returns byte-identical
    /// slices to the owned fallback.
    pub fn build(
        opposition: HashMap<ConceptId, Vec<ConceptId>>,
        mereology_parts: HashMap<ConceptId, Vec<ConceptId>>,
        relations: WordnetRelations,
        sense_count: usize,
        concept_count: usize,
    ) -> Self {
        let bundle = normalize(
            opposition,
            mereology_parts,
            relations,
            sense_count,
            concept_count,
        );
        Self(PackedCsrFamily::build(bundle))
    }

    /// The targets of `id` under `kind` (empty slice if none or out of range).
    pub fn rel(&self, kind: RelationKind, id: ConceptId) -> &[ConceptId] {
        self.0.column(kind, id)
    }

    /// Total number of edges of `kind`.
    pub fn edge_count(&self, kind: RelationKind) -> usize {
        self.0.edge_count(kind)
    }

    /// The dense row count (key space) of `kind`.
    pub fn row_count(&self, kind: RelationKind) -> usize {
        self.0.row_count(kind)
    }

    /// The total number of edges across every relation — for the [`Debug`] summary
    /// and coarse gap checks.
    pub fn total_edge_count(&self) -> usize {
        RelationKind::ALL.iter().map(|&k| self.edge_count(k)).sum()
    }
}

impl core::fmt::Debug for RelationStore {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RelationStore")
            .field(
                "opposition_edges",
                &self.edge_count(RelationKind::Opposition),
            )
            .field(
                "mereology_edges",
                &self.edge_count(RelationKind::MereologyParts),
            )
            .field("total_edges", &self.total_edge_count())
            .finish()
    }
}

// ── CSR family fixture + per-kind labelling test ─────────────────────────────
//
// The generic zero-copy CSR laws prove the build + the `&[ConceptId]` cast +
// archived-equals-owned + FamilyLabelFaithful (per-label distinctness) once,
// generically. These fixtures pin the RELATION instance's concrete `rel` results
// and — critically — the `normalize` bundle↔kind coupling that no generic law can
// see, since `normalize` is this store's own wiring.
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod fixture_tests {
    use super::*;

    fn cid(i: u64) -> ConceptId {
        ConceptId::new(i)
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn rel_slices_match_the_known_relations() {
        let mut opposition: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        opposition.insert(cid(0), alloc::vec![cid(3)]);
        opposition.insert(cid(2), alloc::vec![cid(3), cid(1)]);
        let mut mereology: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        mereology.insert(cid(0), alloc::vec![cid(1), cid(2)]);
        mereology.insert(cid(1), alloc::vec![cid(2)]);
        let mut relations = WordnetRelations::default();
        relations.derivation.insert(cid(3), alloc::vec![cid(0)]);
        let store = RelationStore::build(opposition, mereology, relations, 4, 3);
        let empty: &[ConceptId] = &[];

        assert_eq!(store.rel(RelationKind::Opposition, cid(0)), &[cid(3)]);
        assert_eq!(store.rel(RelationKind::Opposition, cid(1)), empty);
        // Multi-target — order preserved through the cast.
        assert_eq!(
            store.rel(RelationKind::Opposition, cid(2)),
            &[cid(3), cid(1)]
        );
        assert_eq!(store.edge_count(RelationKind::Opposition), 3);
        assert_eq!(store.row_count(RelationKind::Opposition), 4);
        assert_eq!(
            store.rel(RelationKind::MereologyParts, cid(0)),
            &[cid(1), cid(2)]
        );
        assert_eq!(store.rel(RelationKind::MereologyParts, cid(1)), &[cid(2)]);
        assert_eq!(store.rel(RelationKind::Derivation, cid(3)), &[cid(0)]);
        // An unpopulated relation is entirely empty.
        assert_eq!(store.rel(RelationKind::Pertainym, cid(0)), empty);
        // Out-of-range id → empty slice.
        assert_eq!(store.rel(RelationKind::Opposition, cid(99)), empty);
    }

    /// Pins the [`normalize`] bundle↔kind coupling: give EVERY [`RelationKind`] its
    /// own distinct known edge — relation at `ALL`-index `k` gets the single edge
    /// `k → 100 + k`, keyed at id `k` and inserted into THAT relation's own source
    /// map — then assert `rel(kind, k)` returns exactly it, for all 27 kinds. Each
    /// key id `k` is unique to one kind, so a mislabelled bundle slot would look
    /// `k` up in some OTHER relation's map and return `&[]`, failing here.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_kind_returns_its_own_distinct_edge() {
        let n = RelationKind::ALL.len();
        let mut opposition: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        let mut mereology: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        let mut r = WordnetRelations::default();

        for (k, &kind) in RelationKind::ALL.iter().enumerate() {
            let map: &mut HashMap<ConceptId, Vec<ConceptId>> = match kind {
                RelationKind::Opposition => &mut opposition,
                RelationKind::MereologyParts => &mut mereology,
                RelationKind::Derivation => &mut r.derivation,
                RelationKind::Pertainym => &mut r.pertainym,
                RelationKind::SimilarSense => &mut r.similar_sense,
                RelationKind::AlsoSense => &mut r.also_sense,
                RelationKind::ExemplifiesSense => &mut r.exemplifies_sense,
                RelationKind::IsExemplifiedBySense => &mut r.is_exemplified_by_sense,
                RelationKind::ParticipleSense => &mut r.participle_sense,
                RelationKind::SimilarSynset => &mut r.similar_synset,
                RelationKind::AlsoSynset => &mut r.also_synset,
                RelationKind::Causes => &mut r.causes,
                RelationKind::IsCausedBy => &mut r.is_caused_by,
                RelationKind::Entails => &mut r.entails,
                RelationKind::IsEntailedBy => &mut r.is_entailed_by,
                RelationKind::Attribute => &mut r.attribute,
                RelationKind::Exemplifies => &mut r.exemplifies,
                RelationKind::IsExemplifiedBy => &mut r.is_exemplified_by,
                RelationKind::HasDomainTopic => &mut r.has_domain_topic,
                RelationKind::DomainTopic => &mut r.domain_topic,
                RelationKind::HasDomainRegion => &mut r.has_domain_region,
                RelationKind::DomainRegion => &mut r.domain_region,
                RelationKind::ParticipleSynset => &mut r.participle_synset,
                RelationKind::HoloMember => &mut r.holo_member,
                RelationKind::HoloSubstance => &mut r.holo_substance,
                RelationKind::MeroMember => &mut r.mero_member,
                RelationKind::MeroSubstance => &mut r.mero_substance,
            };
            map.insert(cid(k as u64), alloc::vec![cid(100 + k as u64)]);
        }

        let store = RelationStore::build(opposition, mereology, r, n, n);
        for (k, &kind) in RelationKind::ALL.iter().enumerate() {
            assert_eq!(
                store.rel(kind, cid(k as u64)),
                &[cid(100 + k as u64)],
                "kind {kind:?} (ALL-index {k}) returned a mislabelled edge"
            );
        }
    }

    /// DIRECT [`RelationKind`] ↔ loaded-DTD conformance: every kind names the
    /// GWA-LMF `relType` value(s) it is populated from, and each value must be
    /// declared by the REGISTERED `wn_lmf_dtd` source's enumeration (queried
    /// through `wn_lmf_attlist_enum_values`, per
    /// `feedback_bottom_up_loaded_not_encoded`) — so a GWA rename/removal
    /// fails here, and the exhaustive match forces every NEW kind to state its
    /// loaded grounding. Known documented exception: OEWN emits `participle`
    /// at SYNSET level, which WN-LMF DTD 1.3 declares only on `SenseRelation`,
    /// so [`RelationKind::ParticipleSynset`]'s name is checked against the
    /// sense-level enumeration.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_relation_kind_grounds_in_the_loaded_wn_lmf_reltype_enumeration() {
        use crate::social::software::markup::xml::lmf::dtd::wn_lmf_attlist_enum_values;
        let synset = wn_lmf_attlist_enum_values("SynsetRelation", "relType")
            .expect("the loaded WN-LMF DTD declares SynsetRelation relType");
        let sense = wn_lmf_attlist_enum_values("SenseRelation", "relType")
            .expect("the loaded WN-LMF DTD declares SenseRelation relType");
        for kind in RelationKind::ALL {
            let (level, enumeration, values): (&str, &[String], &[&str]) = match kind {
                RelationKind::Opposition => ("sense", &sense, &["antonym"]),
                RelationKind::MereologyParts => ("synset", &synset, &["holo_part", "mero_part"]),
                RelationKind::Derivation => ("sense", &sense, &["derivation"]),
                RelationKind::Pertainym => ("sense", &sense, &["pertainym"]),
                RelationKind::SimilarSense => ("sense", &sense, &["similar"]),
                RelationKind::AlsoSense => ("sense", &sense, &["also"]),
                RelationKind::ExemplifiesSense => ("sense", &sense, &["exemplifies"]),
                RelationKind::IsExemplifiedBySense => ("sense", &sense, &["is_exemplified_by"]),
                RelationKind::ParticipleSense => ("sense", &sense, &["participle"]),
                RelationKind::SimilarSynset => ("synset", &synset, &["similar"]),
                RelationKind::AlsoSynset => ("synset", &synset, &["also"]),
                RelationKind::Causes => ("synset", &synset, &["causes"]),
                RelationKind::IsCausedBy => ("synset", &synset, &["is_caused_by"]),
                RelationKind::Entails => ("synset", &synset, &["entails"]),
                RelationKind::IsEntailedBy => ("synset", &synset, &["is_entailed_by"]),
                RelationKind::Attribute => ("synset", &synset, &["attribute"]),
                RelationKind::Exemplifies => ("synset", &synset, &["exemplifies"]),
                RelationKind::IsExemplifiedBy => ("synset", &synset, &["is_exemplified_by"]),
                RelationKind::HasDomainTopic => ("synset", &synset, &["has_domain_topic"]),
                RelationKind::DomainTopic => ("synset", &synset, &["domain_topic"]),
                RelationKind::HasDomainRegion => ("synset", &synset, &["has_domain_region"]),
                RelationKind::DomainRegion => ("synset", &synset, &["domain_region"]),
                // The documented OEWN extension: synset-level participle is
                // absent from the 1.3 DTD's SynsetRelation enumeration.
                RelationKind::ParticipleSynset => ("sense", &sense, &["participle"]),
                RelationKind::HoloMember => ("synset", &synset, &["holo_member"]),
                RelationKind::HoloSubstance => ("synset", &synset, &["holo_substance"]),
                RelationKind::MeroMember => ("synset", &synset, &["mero_member"]),
                RelationKind::MeroSubstance => ("synset", &synset, &["mero_substance"]),
            };
            for v in values {
                assert!(
                    enumeration.iter().any(|e| e == v),
                    "RelationKind::{kind:?}: relType {v:?} is not in the loaded WN-LMF \
                     DTD's {level}-level enumeration"
                );
            }
        }
    }
}
