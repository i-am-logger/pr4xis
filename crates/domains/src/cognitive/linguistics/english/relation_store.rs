//! Every non-taxonomy WordNet relation English holds — opposition, mereology, and
//! the ~25 `WordnetRelations` sub-maps — as ONE immutable, zero-copy family of
//! labelled CSRs.
//!
//! `English` historically owned these as ~27 separate `HashMap`s (`opposition`,
//! `mereology_parts`, and the 25 named fields of `WordnetRelations`). Every one of
//! them is keyed by a *dense* id assigned at
//! [`from_wordnet`](super::ontology::English::from_wordnet) time — a
//! [`SenseId`](super::ontology::SenseId) in `0..sense_count` (sense-level
//! relations) or a [`ConceptId`](super::ontology::ConceptId) in `0..concept_count`
//! (synset-level relations) — and both are the same `repr(transparent)`
//! single-`u64` [`Reference<4>`](crate::formal::information::ontology::Reference).
//! So each relation is, byte-for-byte, a [`taxonomy_store`](super::taxonomy_store)-
//! style CSR: a `u64` targets array (8-aligned, cast zero-copy to `&[Ref]`) plus a
//! `u32` offsets array indexed directly by the id.
//!
//! # The representation
//!
//! Under `prx` on a little-endian target all 27 relations are transcoded ONCE, at
//! load, into a single [`AlignedVec<16>`]: every relation's targets array first
//! (concatenated in the fixed [`RelationKind::ALL`] order, each inheriting the
//! buffer's 8-alignment), then every relation's offsets array. The per-relation
//! layout map (row count, edge count, and the two byte offsets) is held in the
//! struct, not the buffer — this store is always built fresh at load (the compact
//! `.prx` path rebuilds every relation from the decoded `WordNet`; none of these
//! maps is serialized), so the metadata never needs to survive a round-trip. The
//! owned maps are consumed by [`RelationStore::build`] and dropped; only the buffer
//! survives — a REPLACEMENT of the owned build, not an addition on top of it.
//!
//! # Endianness invariant (why the archived variant is `little`-only)
//!
//! The zero-copy cast reinterprets packed id bytes as `&[Ref]`; sound only where
//! the machine's native integer byte order equals the little-endian order the ids
//! were written with. wasm32 and x86-64 — the two targets praxis ships — are both
//! little-endian, so the archived variant is compiled only under
//! `cfg(target_endian = "little")`; a big-endian target falls back to the owned
//! `HashMap`s, exactly as a non-`prx` build does.
//!
//! Reference: Hill, D. *rkyv: zero-copy deserialization framework for Rust* (v0.8)
//! — `AlignedVec` is rkyv's own little-endian aligned buffer; this module reuses
//! its aligned-buffer discipline for the labelled relation CSRs. See
//! <https://github.com/rkyv/rkyv>.

use alloc::vec::Vec;

use hashbrown::HashMap;

use super::ontology::{ConceptId, WordnetRelations};

/// The zero-copy, `prx`-gated relation store (little-endian targets).
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::RelationStore;

/// The owned fallback relation store (no `prx`, or a big-endian target).
#[cfg(not(all(feature = "prx", target_endian = "little")))]
pub use owned::RelationStore;

/// The labelled family of relations `English` holds — the tag on each CSR. The
/// discriminant order IS the layout order ([`ALL`](Self::ALL)); `kind as usize`
/// indexes the per-relation metadata, so the enum order and `ALL` MUST agree with
/// the [`normalize`] bundle below.
///
/// Literature: the relation identities are the Global WordNet Association LMF
/// relation set (Fellbaum 1998; Fellbaum-Osherson-Clark 2009 for `derivation`;
/// Bentivogli & Pianta 2004 for the domain pointers) — see [`WordnetRelations`].
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
    /// Every relation, in layout order — the SAME order the [`normalize`] bundle
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
/// the ordering coupling: `kind as usize` indexes the per-relation metadata
/// (`self.meta[kind as usize]` / `self.rels[kind as usize]`), so the enum
/// discriminant order MUST equal the `ALL` layout order that drives the build/pack
/// loops. A mid-enum variant insertion that appended to `ALL` at the end would
/// shift every following discriminant one slot past its `ALL` entry and mislabel
/// every relation after the insertion point — this loop diverges the two orders
/// and FAILS TO COMPILE before that can ship. (The [`normalize`] bundle↔kind leg
/// is pinned by the per-kind labelling test.)
const _: () = {
    let mut i = 0;
    while i < REL_COUNT {
        assert!(RelationKind::ALL[i] as usize == i);
        i += 1;
    }
};

/// Bundle the owned input maps into one `(row_count, map)` vector in the fixed
/// [`RelationKind::ALL`] order — the SINGLE place that couples a relation's tag to
/// its owned source map and its key space (sense- vs concept-keyed → row count).
/// Consumes every owned map; shared by both representations so the two are
/// guaranteed to fold the identical inputs.
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
    // One entry per relation, in `ALL` order — a plain `assert!` (holds in release
    // too), O(1), fires only on a construction bug.
    assert_eq!(
        bundle.len(),
        REL_COUNT,
        "relation bundle must have exactly one entry per RelationKind"
    );
    bundle
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

impl RelationStore {
    /// The total number of edges across every relation — for the [`Debug`] summary
    /// and coarse gap checks.
    pub fn total_edge_count(&self) -> usize {
        RelationKind::ALL.iter().map(|&k| self.edge_count(k)).sum()
    }
}

// ── owned fallback ───────────────────────────────────────────────────────────

/// The owned fallback: the labelled maps English used to hold, kept as plain
/// `HashMap`s indexed by [`RelationKind`]. The mandatory non-`prx` (and big-endian)
/// path, mirroring the [`taxonomy_store`](super::taxonomy_store) split.
///
/// Compiled ALSO under `test` on the archived path so the CSR unit tests can build
/// both representations from the same inputs and assert the archived store returns
/// byte-identical slices to this owned fallback.
#[cfg(any(not(all(feature = "prx", target_endian = "little")), test))]
mod owned {
    use super::*;

    /// One owned relation's map plus its dense row count (key space).
    struct OwnedRel {
        row_count: usize,
        map: HashMap<ConceptId, Vec<ConceptId>>,
    }

    /// The labelled relations, held as owned maps in [`RelationKind::ALL`] order.
    pub struct RelationStore {
        rels: Vec<OwnedRel>,
    }

    impl RelationStore {
        /// Retain the owned maps as-is — the fallback keeps the `HashMap`s.
        pub fn build(
            opposition: HashMap<ConceptId, Vec<ConceptId>>,
            mereology_parts: HashMap<ConceptId, Vec<ConceptId>>,
            relations: WordnetRelations,
            sense_count: usize,
            concept_count: usize,
        ) -> Self {
            let rels = super::normalize(
                opposition,
                mereology_parts,
                relations,
                sense_count,
                concept_count,
            )
            .into_iter()
            .map(|(row_count, map)| OwnedRel { row_count, map })
            .collect();
            Self { rels }
        }

        /// The targets of `id` under `kind` (empty slice if none or out of range).
        pub fn rel(&self, kind: RelationKind, id: ConceptId) -> &[ConceptId] {
            self.rels[kind as usize]
                .map
                .get(&id)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
        }

        /// Total number of edges of `kind`.
        pub fn edge_count(&self, kind: RelationKind) -> usize {
            self.rels[kind as usize].map.values().map(Vec::len).sum()
        }

        /// The dense row count (key space) of `kind`.
        pub fn row_count(&self, kind: RelationKind) -> usize {
            self.rels[kind as usize].row_count
        }
    }
}

// ── archived variant ─────────────────────────────────────────────────────────

/// The zero-copy archived store: every relation's CSR packed into one
/// [`AlignedVec<16>`], read back through the same zero-copy id-slice cast as
/// [`taxonomy_store`](super::taxonomy_store).
#[cfg(all(feature = "prx", target_endian = "little"))]
mod archived {
    use rkyv::util::AlignedVec;

    use super::*;

    /// The soundness precondition of the zero-copy id cast: native integer byte
    /// order must equal the little-endian order ids are stored in. Enforced by the
    /// `cfg(target_endian = "little")` gate; asserted here so a mis-configuration
    /// is a compile error, not a silent miscast.
    const _: () = assert!(cfg!(target_endian = "little"));

    /// `RelationStore` is `Sync`: an [`AlignedVec`] (rkyv declares it `Send +
    /// Sync`) plus `Copy` per-relation metadata — no interior mutability. Keeps
    /// `English`'s process-wide `OnceLock<English>` static valid.
    const _: fn() = || {
        fn assert_sync<T: Sync>() {}
        assert_sync::<RelationStore>();
    };

    /// One relation's layout within the packed buffer.
    #[derive(Clone, Copy)]
    struct RelMeta {
        /// Dense row count (key space): `0..row_count` are valid ids.
        row_count: usize,
        /// Number of edges in this relation.
        edge_count: usize,
        /// Byte offset of this relation's targets array (8-aligned).
        targets_at: usize,
        /// Byte offset of this relation's `(row_count + 1)`-entry offsets array.
        offsets_at: usize,
    }

    /// Every labelled relation, packed into one dense-indexed, zero-copy CSR
    /// family. See the [module docs](super) for the layout and endianness
    /// invariant.
    pub struct RelationStore {
        /// All relations' targets arrays, then all their offsets arrays.
        buf: AlignedVec<16>,
        /// Per-relation layout, in [`RelationKind::ALL`] order.
        meta: [RelMeta; REL_COUNT],
    }

    impl RelationStore {
        /// Transcode the owned maps into the packed CSR family ONCE, consuming and
        /// freeing them. Each relation's per-id run is written in dense-id order,
        /// each id copied in its owned-map order, so [`rel`](Self::rel) returns
        /// byte-identical slices to the owned fallback.
        pub fn build(
            opposition: HashMap<ConceptId, Vec<ConceptId>>,
            mereology_parts: HashMap<ConceptId, Vec<ConceptId>>,
            relations: WordnetRelations,
            sense_count: usize,
            concept_count: usize,
        ) -> Self {
            let bundle = super::normalize(
                opposition,
                mereology_parts,
                relations,
                sense_count,
                concept_count,
            );

            // Per-relation edge counts (sum of per-id run lengths over the dense
            // key space, so a stray out-of-range key contributes nothing — matching
            // the CSR that only visits `0..row_count`).
            let run = |map: &HashMap<ConceptId, Vec<ConceptId>>, i: usize| -> usize {
                map.get(&ConceptId::new(i as u64)).map_or(0, Vec::len)
            };
            let edge_counts: Vec<usize> = bundle
                .iter()
                .map(|(row_count, map)| (0..*row_count).map(|i| run(map, i)).sum())
                .collect();

            // Layout: all targets arrays (each `edge_count * 8` bytes, 8-aligned),
            // then all offsets arrays (each `(row_count + 1) * 4` bytes).
            let mut targets_at = [0usize; REL_COUNT];
            let mut offsets_at = [0usize; REL_COUNT];
            let mut cursor = 0usize;
            for k in 0..REL_COUNT {
                targets_at[k] = cursor;
                cursor += edge_counts[k] * 8;
            }
            for k in 0..REL_COUNT {
                offsets_at[k] = cursor;
                cursor += (bundle[k].0 + 1) * 4;
            }
            let total = cursor;

            let mut buf = AlignedVec::<16>::with_capacity(total);

            // Targets: each relation's runs, in dense-id order, each id a
            // little-endian u64 (`Reference<4>`'s in-memory representation).
            for (row_count, map) in &bundle {
                for i in 0..*row_count {
                    if let Some(v) = map.get(&ConceptId::new(i as u64)) {
                        for id in v {
                            buf.extend_from_slice(&id.value().to_le_bytes());
                        }
                    }
                }
            }

            // Offsets: CSR prefix sums over the per-id run lengths.
            for (row_count, map) in &bundle {
                let mut acc = 0u32;
                buf.extend_from_slice(&acc.to_le_bytes());
                for i in 0..*row_count {
                    acc += run(map, i) as u32;
                    buf.extend_from_slice(&acc.to_le_bytes());
                }
            }

            assert_eq!(
                buf.len(),
                total,
                "relation CSR buffer length must equal the computed layout size"
            );

            let mut meta = [RelMeta {
                row_count: 0,
                edge_count: 0,
                targets_at: 0,
                offsets_at: 0,
            }; REL_COUNT];
            for k in 0..REL_COUNT {
                meta[k] = RelMeta {
                    row_count: bundle[k].0,
                    edge_count: edge_counts[k],
                    targets_at: targets_at[k],
                    offsets_at: offsets_at[k],
                };
            }

            Self { buf, meta }
            // `bundle` (and every owned map + id it holds) drops here — the owned
            // build is freed; only `buf` survives.
        }

        /// Read the `i`-th little-endian `u32` of the offsets array at byte offset
        /// `base`. A checked 4-byte read (no alignment requirement on the offset
        /// arrays; only the targets arrays are alignment-load-bearing).
        #[inline]
        fn csr(&self, base: usize, i: usize) -> usize {
            let at = base + i * 4;
            let b = &self.buf.as_slice()[at..at + 4];
            u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize
        }

        /// The targets of `id` under `kind`, cast zero-copy from the CSR (empty
        /// slice if `id` has no targets or is out of range).
        pub fn rel(&self, kind: RelationKind, id: ConceptId) -> &[ConceptId] {
            let m = &self.meta[kind as usize];
            let i = id.value() as usize;
            if i >= m.row_count {
                return &[];
            }
            let start = self.csr(m.offsets_at, i);
            let end = self.csr(m.offsets_at, i + 1);
            let byte_start = m.targets_at + start * 8;
            let len = end - start;
            // SAFETY: each relation's targets array begins at a multiple of 8
            // (they are laid out first, each `edge_count * 8` bytes, from the
            // buffer's 16-aligned base), so `byte_start = targets_at + start*8` is
            // 8-aligned — the alignment `ConceptId` (a single `u64`) requires. The
            // `len` ConceptIds at `byte_start` are in bounds by construction (the
            // CSR offsets partition exactly this relation's edge count). Each was
            // written as a little-endian `u64`; on this little-endian-gated build
            // that IS `ConceptId`'s representation, so the reinterpretation is a
            // value-preserving zero-copy view. The returned slice borrows `self`.
            unsafe {
                let ptr = self.buf.as_ptr().add(byte_start) as *const ConceptId;
                core::slice::from_raw_parts(ptr, len)
            }
        }

        /// Total number of edges of `kind`.
        pub fn edge_count(&self, kind: RelationKind) -> usize {
            self.meta[kind as usize].edge_count
        }

        /// The dense row count (key space) of `kind`.
        pub fn row_count(&self, kind: RelationKind) -> usize {
            self.meta[kind as usize].row_count
        }
    }
}

// ── CSR unit tests (archived path) ───────────────────────────────────────────
//
// Direct coverage of the labelled multi-CSR build + the zero-copy id-slice cast: a
// small KNOWN set of relations — a sense-keyed one (Opposition), a concept-keyed
// one (MereologyParts), and a WordnetRelations sub-map (Derivation, sense-keyed) —
// asserting the archived `rel` slices are exactly right, out-of-range ids and an
// empty relation give `&[]`, AND the archived store returns slices byte-identical
// to the owned fallback built from the SAME inputs.
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod csr_tests {
    use super::RelationStore; // the archived, zero-copy store (the crate-level re-export)
    use super::owned::RelationStore as OwnedStore; // the owned fallback (compiled under `test`)
    use super::{ConceptId, HashMap, RelationKind, WordnetRelations};
    use alloc::vec::Vec;

    /// One `RelationStore::build` input tuple: `(opposition, mereology_parts,
    /// relations, sense_count, concept_count)`.
    type Inputs = (
        HashMap<ConceptId, Vec<ConceptId>>,
        HashMap<ConceptId, Vec<ConceptId>>,
        WordnetRelations,
        usize,
        usize,
    );

    fn cid(i: u64) -> ConceptId {
        ConceptId::new(i)
    }

    /// A small KNOWN relation set.
    /// opposition (sense 0..4):   0:[3]  2:[3, 1] (multi-target)  (1,3 have none)
    /// mereology  (concept 0..3): 0:[1, 2]  1:[2]  (2 has none — leaf)
    /// derivation (sense 0..4):   3:[0]                            (rest none)
    fn fixture() -> Inputs {
        let mut opposition: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        opposition.insert(cid(0), alloc::vec![cid(3)]);
        opposition.insert(cid(2), alloc::vec![cid(3), cid(1)]);
        let mut mereology: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        mereology.insert(cid(0), alloc::vec![cid(1), cid(2)]);
        mereology.insert(cid(1), alloc::vec![cid(2)]);
        let mut relations = WordnetRelations::default();
        relations.derivation.insert(cid(3), alloc::vec![cid(0)]);
        (opposition, mereology, relations, 4, 3)
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn archived_csr_slices_match_the_known_relations() {
        let (o, m, r, s, c) = fixture();
        let store = RelationStore::build(o, m, r, s, c);
        let empty: &[ConceptId] = &[];

        // Opposition (sense-keyed, row space 4).
        assert_eq!(store.rel(RelationKind::Opposition, cid(0)), &[cid(3)]);
        assert_eq!(store.rel(RelationKind::Opposition, cid(1)), empty);
        // Multi-target — order preserved through the cast.
        assert_eq!(
            store.rel(RelationKind::Opposition, cid(2)),
            &[cid(3), cid(1)]
        );
        assert_eq!(store.rel(RelationKind::Opposition, cid(3)), empty);
        assert_eq!(store.edge_count(RelationKind::Opposition), 3);
        assert_eq!(store.row_count(RelationKind::Opposition), 4);

        // Mereology (concept-keyed, row space 3).
        assert_eq!(
            store.rel(RelationKind::MereologyParts, cid(0)),
            &[cid(1), cid(2)]
        );
        assert_eq!(store.rel(RelationKind::MereologyParts, cid(1)), &[cid(2)]);
        assert_eq!(store.rel(RelationKind::MereologyParts, cid(2)), empty);
        assert_eq!(store.edge_count(RelationKind::MereologyParts), 3);

        // Derivation (a WordnetRelations sub-map, sense-keyed).
        assert_eq!(store.rel(RelationKind::Derivation, cid(3)), &[cid(0)]);
        assert_eq!(store.rel(RelationKind::Derivation, cid(0)), empty);
        assert_eq!(store.edge_count(RelationKind::Derivation), 1);

        // An unpopulated relation is entirely empty.
        assert_eq!(store.rel(RelationKind::Pertainym, cid(0)), empty);
        assert_eq!(store.edge_count(RelationKind::Pertainym), 0);

        // Out-of-range id → empty slice (the `i >= row_count` guard).
        assert_eq!(store.rel(RelationKind::Opposition, cid(99)), empty);
        assert_eq!(store.rel(RelationKind::MereologyParts, cid(99)), empty);
    }

    /// Pins the [`normalize`](super::normalize) bundle↔kind leg of the ordering
    /// coupling that the compile-time discriminant↔`ALL` guard cannot see: give
    /// EVERY [`RelationKind`] its own distinct known edge — relation at `ALL`-index
    /// `k` gets the single edge `k → 100 + k`, keyed at id `k` and inserted into
    /// THAT relation's own source map — then assert `rel(kind, k)` returns exactly
    /// it, for all 27 kinds (not just the first three the [`fixture`] covers).
    ///
    /// Each key id `k` is unique to one kind, so if any relation were folded into
    /// the wrong bundle slot (a mid-enum insertion appended to the bundle at the
    /// end, say), `rel(kind, k)` would look `k` up in some OTHER relation's map —
    /// whose only edge sits at a different id — and return `&[]`, failing here.
    /// This is the test a mislabel escapes the other tests through: the
    /// byte-identical archived-vs-owned check shares `normalize` (so cannot see a
    /// bundle mis-order), and the `fixture` populates only three kinds.
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn every_kind_returns_its_own_distinct_edge() {
        let n = RelationKind::ALL.len();
        let mut opposition: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        let mut mereology: HashMap<ConceptId, Vec<ConceptId>> = HashMap::new();
        let mut r = WordnetRelations::default();

        // One distinct edge per relation, placed in that relation's own map.
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

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn archived_csr_slices_are_identical_to_the_owned_fallback() {
        let (o, m, r, s, c) = fixture();
        let (o2, m2, r2, _, _) = fixture();
        let archived = RelationStore::build(o, m, r, s, c);
        let owned = OwnedStore::build(o2, m2, r2, s, c);
        for &kind in &RelationKind::ALL {
            for i in 0..6u64 {
                assert_eq!(
                    archived.rel(kind, cid(i)),
                    owned.rel(kind, cid(i)),
                    "rel {kind:?} id {i}"
                );
            }
            assert_eq!(
                archived.edge_count(kind),
                owned.edge_count(kind),
                "edge_count {kind:?}"
            );
            assert_eq!(
                archived.row_count(kind),
                owned.row_count(kind),
                "row_count {kind:?}"
            );
        }
    }
}
