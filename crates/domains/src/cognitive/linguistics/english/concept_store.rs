//! The English concept (synset) records as an immutable, zero-copy archive.
//!
//! `English` holds ~120k [`Concept`] records — each a synset's `original_id`,
//! part-of-speech, lemmas, definitions and examples. Held naively as an owned
//! `Vec<Concept>` this is, after the [`word_index`](super::word_index) reclaim,
//! the single largest remaining resident cost of the loaded reasoner: ~120k
//! records, each owning five heap `String`/`Vec<String>` allocations (its own
//! block + capacity slack + control word). The resident-memory gate
//! ([`examples/resident_memory`](../../../../../examples/resident_memory.rs))
//! attributes the bulk of the post-`word_index` embedded-English footprint to
//! these records.
//!
//! # The representation
//!
//! Under `prx` on a little-endian target the owned `Vec<Concept>` is transcoded
//! ONCE, at load, into a single `rkyv`-archived [`AlignedVec<16>`] buffer holding
//! the concept records in their zero-copy archived form. This is the SAME
//! "materialize the owned build into a packed archive, free the intermediate"
//! discipline the [`word_index`](super::word_index) reclaim applies, and the same
//! `rkyv` [`AlignedVec`] + `access` zero-copy primitives `pr4xis-runtime`'s
//! archive lens uses. The owned `Vec<Concept>` is consumed by
//! [`ConceptStore::build`] and dropped; only the archive buffer survives — a
//! REPLACEMENT of the owned build, not an addition on top of it (so steady-state
//! resident memory falls by the reclaimed records while load time is unchanged:
//! the transcode is one `rkyv` serialize over data already in hand, no second
//! copy retained).
//!
//! Records are addressed by index: a synset's [`ConceptId`] is exactly its
//! position in the archive (assigned as `ConceptId::new(idx)` at
//! [`from_wordnet`](super::ontology::English::from_wordnet) time), so
//! [`ConceptStore::get`] is a bounds-checked index — no id is stored per record.
//!
//! # Reads: [`ConceptView`]
//!
//! [`ConceptStore::get`] hands back a [`ConceptView`] — a borrowed accessor over
//! EITHER an owned [`Concept`] (the fallback, and every LOADED-corpus concept a
//! [`ComposedReasoner`](crate::cognitive::linguistics::composed) synthesizes) or
//! an archived record. Its `original_id` / `pos` / `lemmas` / `definitions` /
//! `examples` accessors read `&str` / [`LmfPos`] straight out of whichever
//! representation backs it, so a consumer reads glosses and lemmas identically
//! against both. The archived arm reads through `rkyv`'s own
//! [`ArchivedString`](rkyv::string::ArchivedString) /
//! [`ArchivedVec`](rkyv::vec::ArchivedVec) accessors (`.as_str()` / `.iter()`) —
//! a safe zero-copy borrow of the buffer, no owned rebuild, no transmute.
//!
//! # Endianness invariant (why the archived variant is `little`-only)
//!
//! `rkyv`'s zero-copy archived layout is little-endian by construction (its
//! relative pointers and integers are stored little-endian); [`access`] /
//! [`access_unchecked`] reinterpret the buffer in place, so they are sound only
//! where the machine's native byte order matches. wasm32 and x86-64 — the two
//! targets praxis ships — are both little-endian. The archived variant is
//! therefore compiled only under `cfg(target_endian = "little")`; a
//! (hypothetical) big-endian target falls back to the owned `Vec<Concept>`,
//! exactly as a non-`prx` build does.
//!
//! Reference: Hill, D. *rkyv: zero-copy deserialization framework for Rust* (v0.8)
//! — `AlignedVec` + `access` / `access_unchecked` are rkyv's own little-endian
//! zero-copy primitives; this module reuses them for the concept records. See
//! <https://github.com/rkyv/rkyv>.

use alloc::string::String;
use alloc::vec::Vec;

use crate::social::software::markup::xml::lmf::ontology::LmfPos;

use super::ontology::{Concept, ConceptId};

/// The zero-copy, `prx`-gated concept store (little-endian targets).
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::ConceptStore;

/// The concept-store leaf-lens mirror root — exposed so the shared lens-law
/// axioms (`formal::meta::lens::rkyv_lens_laws`) can name it as the `Mirror` of
/// the `RkyvLens<Vec<Concept>, ConceptRecords>` instance.
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::ConceptRecords;

/// The owned fallback concept store (no `prx`, or a big-endian target).
#[cfg(not(all(feature = "prx", target_endian = "little")))]
pub use owned::ConceptStore;

// ── ConceptView: the representation-agnostic read surface ────────────────────

/// A borrowed view of one concept record — the read surface [`ConceptStore::get`]
/// hands back, abstracting over an owned [`Concept`] and (under `prx`) an archived
/// record so every consumer reads `original_id` / `pos` / `lemmas` /
/// `definitions` / `examples` identically regardless of backing representation.
///
/// The `Owned` arm always exists: it backs both the non-`prx` fallback AND every
/// LOADED-corpus concept a [`ComposedReasoner`](crate::cognitive::linguistics::composed)
/// synthesizes (those are owned `Concept`s, never archived). The `Archived` arm
/// exists only where the archive does — under `prx` on a little-endian target.
#[derive(Clone, Copy)]
pub enum ConceptView<'a> {
    /// An owned concept record (the fallback store, or a loaded-corpus concept).
    Owned(&'a Concept),
    /// An archived concept record, read zero-copy out of the store's buffer.
    /// `id` is carried alongside because the archived record does not store it
    /// (a record's id IS its index — see the [module docs](self)).
    #[cfg(all(feature = "prx", target_endian = "little"))]
    Archived {
        /// The record's [`ConceptId`] — its index in the archive.
        id: ConceptId,
        /// The archived record, borrowed from the store's buffer.
        rec: &'a archived::ArchivedConceptRecord,
    },
}

impl<'a> ConceptView<'a> {
    /// The concept's [`ConceptId`] — its position in the store.
    pub fn id(&self) -> ConceptId {
        match self {
            ConceptView::Owned(c) => c.id,
            #[cfg(all(feature = "prx", target_endian = "little"))]
            ConceptView::Archived { id, .. } => *id,
        }
    }

    /// The concept's original WordNet synset id string (e.g. `"oewn-02084071-n"`).
    pub fn original_id(&self) -> &'a str {
        match self {
            ConceptView::Owned(c) => c.original_id.as_str(),
            #[cfg(all(feature = "prx", target_endian = "little"))]
            ConceptView::Archived { rec, .. } => rec.original_id.as_str(),
        }
    }

    /// The concept's part of speech.
    pub fn pos(&self) -> LmfPos {
        match self {
            ConceptView::Owned(c) => c.pos,
            #[cfg(all(feature = "prx", target_endian = "little"))]
            ConceptView::Archived { rec, .. } => archived::decode_pos(&rec.pos),
        }
    }

    /// The lemmas (written forms) that express this concept, as `&str`.
    pub fn lemmas(&self) -> ConceptStrs<'a> {
        match self {
            ConceptView::Owned(c) => ConceptStrs::owned(&c.lemmas),
            #[cfg(all(feature = "prx", target_endian = "little"))]
            ConceptView::Archived { rec, .. } => ConceptStrs::archived(&rec.lemmas),
        }
    }

    /// The concept's glosses (definitions), as `&str`.
    pub fn definitions(&self) -> ConceptStrs<'a> {
        match self {
            ConceptView::Owned(c) => ConceptStrs::owned(&c.definitions),
            #[cfg(all(feature = "prx", target_endian = "little"))]
            ConceptView::Archived { rec, .. } => ConceptStrs::archived(&rec.definitions),
        }
    }

    /// The concept's usage examples, as `&str`.
    pub fn examples(&self) -> ConceptStrs<'a> {
        match self {
            ConceptView::Owned(c) => ConceptStrs::owned(&c.examples),
            #[cfg(all(feature = "prx", target_endian = "little"))]
            ConceptView::Archived { rec, .. } => ConceptStrs::archived(&rec.examples),
        }
    }
}

impl core::fmt::Debug for ConceptView<'_> {
    /// Representation-agnostic: read through the accessors so an owned and an
    /// archived view of the same record format identically (and no archived-Debug
    /// bound is needed on the leaf types).
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ConceptView")
            .field("id", &self.id())
            .field("original_id", &self.original_id())
            .field("pos", &self.pos())
            .field("lemmas", &self.lemmas().collect::<Vec<_>>())
            .field("definitions", &self.definitions().collect::<Vec<_>>())
            .field("examples", &self.examples().collect::<Vec<_>>())
            .finish()
    }
}

/// A zero-allocation iterator over one concept field's strings (`lemmas` /
/// `definitions` / `examples`), yielding `&str` regardless of whether the record
/// is owned (`&[String]`) or archived (`&[ArchivedString]`).
pub enum ConceptStrs<'a> {
    /// Iterating an owned `Vec<String>`.
    Owned(core::slice::Iter<'a, String>),
    /// Iterating an archived `ArchivedVec<ArchivedString>`.
    #[cfg(all(feature = "prx", target_endian = "little"))]
    Archived(core::slice::Iter<'a, rkyv::string::ArchivedString>),
}

impl<'a> ConceptStrs<'a> {
    fn owned(v: &'a [String]) -> Self {
        ConceptStrs::Owned(v.iter())
    }

    #[cfg(all(feature = "prx", target_endian = "little"))]
    fn archived(v: &'a rkyv::vec::ArchivedVec<rkyv::string::ArchivedString>) -> Self {
        ConceptStrs::Archived(v.iter())
    }
}

impl<'a> Iterator for ConceptStrs<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        match self {
            ConceptStrs::Owned(it) => it.next().map(String::as_str),
            #[cfg(all(feature = "prx", target_endian = "little"))]
            ConceptStrs::Archived(it) => it.next().map(rkyv::string::ArchivedString::as_str),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            ConceptStrs::Owned(it) => it.size_hint(),
            #[cfg(all(feature = "prx", target_endian = "little"))]
            ConceptStrs::Archived(it) => it.size_hint(),
        }
    }
}

impl ExactSizeIterator for ConceptStrs<'_> {}

// ── owned fallback ───────────────────────────────────────────────────────────

/// The owned fallback: a plain `Vec<Concept>`. Kept as the mandatory non-`prx`
/// (and big-endian) path, mirroring the [`word_index`](super::word_index) split.
///
/// Compiled ALSO under `test` on the archived (`prx` + little-endian) path so the
/// cast unit tests can build both representations from the same records and assert
/// the archived store reads each field identically to this owned fallback (it is
/// only `pub use`d as `ConceptStore` on the non-archived path, so there is no
/// re-export conflict).
#[cfg(any(not(all(feature = "prx", target_endian = "little")), test))]
mod owned {
    use super::*;

    /// The concept records, held as an owned vector.
    #[derive(Clone)]
    pub struct ConceptStore {
        concepts: Vec<Concept>,
    }

    impl ConceptStore {
        /// Retain the owned build as-is — the fallback keeps the `Vec`.
        pub fn build(concepts: Vec<Concept>) -> Self {
            Self { concepts }
        }

        /// The record at `id` (its index), or `None` if out of range.
        pub fn get(&self, id: ConceptId) -> Option<ConceptView<'_>> {
            self.concepts
                .get(id.value() as usize)
                .map(ConceptView::Owned)
        }

        /// Number of records.
        pub fn len(&self) -> usize {
            self.concepts.len()
        }

        /// Whether the store holds no records.
        pub fn is_empty(&self) -> bool {
            self.concepts.is_empty()
        }

        /// Every record, as a [`ConceptView`], in id order.
        pub fn iter(&self) -> impl Iterator<Item = ConceptView<'_>> {
            self.concepts.iter().map(ConceptView::Owned)
        }
    }

    impl core::fmt::Debug for ConceptStore {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("ConceptStore")
                .field("len", &self.len())
                .finish()
        }
    }
}

// ── archived variant ─────────────────────────────────────────────────────────

/// The zero-copy archived store: the concept records `rkyv`-serialized into a
/// single [`AlignedVec<16>`], read back through `rkyv`'s safe archived accessors.
#[cfg(all(feature = "prx", target_endian = "little"))]
mod archived {
    use rkyv::util::AlignedVec;

    use pr4xis_runtime::lens::rkyv_lens::{RkyvLens, RkyvMirror, RkyvOwned};

    use crate::social::software::markup::xml::lmf::ontology::ArchivedLmfPos;

    use super::*;

    /// The concrete lens for the concept store instance.
    type ConceptLens = RkyvLens<Vec<Concept>, ConceptRecords>;

    /// The `rkyv` mirror of one [`Concept`] — its serializable shadow, minus the
    /// `id` (a record's id IS its index, so storing it would be redundant). Only
    /// the payload the read surface exposes is carried. Authored as a mirror (not
    /// `#[derive]`d on [`Concept`]) so the live domain type stays free of `rkyv`'s
    /// layout coupling, exactly as `pr4xis-runtime`'s archive lens does.
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct ConceptRecord {
        /// Mirrors [`Concept::original_id`].
        pub original_id: String,
        /// Mirrors [`Concept::pos`]. `LmfPos` derives `rkyv::Archive` under `prx`.
        pub pos: LmfPos,
        /// Mirrors [`Concept::lemmas`].
        pub lemmas: Vec<String>,
        /// Mirrors [`Concept::definitions`].
        pub definitions: Vec<String>,
        /// Mirrors [`Concept::examples`].
        pub examples: Vec<String>,
    }

    // The zero-copy archived form of one `ConceptRecord` — what a
    // `ConceptView::Archived` borrows out of the store's buffer — is the
    // derive-generated `ArchivedConceptRecord` (fields `ArchivedString`,
    // `ArchivedVec<ArchivedString>`, `ArchivedLmfPos`). It is referenced by that
    // name directly; no alias is declared (the `rkyv::Archive` derive already
    // defines the name).

    /// The archive root: the concept records. A local newtype (not a bare
    /// `Vec<ConceptRecord>`) so the [`RkyvMirror`] / [`RkyvOwned`] leaf-lens
    /// conversions land on a domains-local type (the orphan rule forbids
    /// impl'ing the foreign `RkyvMirror`/`RkyvOwned` traits for a bare foreign
    /// `Vec<_>`).
    #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
    pub struct ConceptRecords {
        /// The concept records, in id (index) order.
        pub records: Vec<ConceptRecord>,
    }

    /// The archived form of the record root — the buffer's root type.
    type ArchivedRecords = rkyv::Archived<ConceptRecords>;

    // ── leaf lens: Vec<Concept> ⇄ ConceptRecords ──────────────────────────────

    /// PUT leg: project the owned concepts into their record mirror. The `id`
    /// field is dropped (a record's id IS its index — see the [module docs](super));
    /// only the payload the read surface exposes is carried.
    impl RkyvMirror<Vec<Concept>> for ConceptRecords {
        fn from_owned(concepts: &Vec<Concept>) -> Self {
            ConceptRecords {
                records: concepts
                    .iter()
                    .map(|c| ConceptRecord {
                        original_id: c.original_id.clone(),
                        pos: c.pos,
                        lemmas: c.lemmas.clone(),
                        definitions: c.definitions.clone(),
                        examples: c.examples.clone(),
                    })
                    .collect(),
            }
        }
    }

    /// GET leg: rebuild the owned concepts, re-deriving each record's
    /// [`ConceptId`] from its position (the mirror stores no id). Total — the
    /// index → id reconstruction cannot fail.
    impl RkyvOwned<ConceptRecords> for Vec<Concept> {
        type Error = core::convert::Infallible;
        fn from_mirror(mirror: ConceptRecords) -> Result<Self, core::convert::Infallible> {
            Ok(mirror
                .records
                .into_iter()
                .enumerate()
                .map(|(i, r)| Concept {
                    id: ConceptId::new(i as u64),
                    original_id: r.original_id,
                    pos: r.pos,
                    lemmas: r.lemmas,
                    definitions: r.definitions,
                    examples: r.examples,
                })
                .collect())
        }
    }

    /// Decode an archived [`LmfPos`] back to the owned (Copy) enum. `LmfPos` is a
    /// unit-only enum, so this is a total, allocation-free discriminant mapping;
    /// the exhaustive match makes a new `LmfPos` variant a compile error here
    /// until it is handled, so the bijection can never silently drift.
    pub fn decode_pos(pos: &ArchivedLmfPos) -> LmfPos {
        match pos {
            ArchivedLmfPos::Noun => LmfPos::Noun,
            ArchivedLmfPos::Verb => LmfPos::Verb,
            ArchivedLmfPos::Adjective => LmfPos::Adjective,
            ArchivedLmfPos::SatelliteAdjective => LmfPos::SatelliteAdjective,
            ArchivedLmfPos::Adverb => LmfPos::Adverb,
            ArchivedLmfPos::Determiner => LmfPos::Determiner,
            ArchivedLmfPos::Pronoun => LmfPos::Pronoun,
            ArchivedLmfPos::Preposition => LmfPos::Preposition,
            ArchivedLmfPos::Conjunction => LmfPos::Conjunction,
            ArchivedLmfPos::Particle => LmfPos::Particle,
            ArchivedLmfPos::Copula => LmfPos::Copula,
            ArchivedLmfPos::Auxiliary => LmfPos::Auxiliary,
            ArchivedLmfPos::Interjection => LmfPos::Interjection,
            ArchivedLmfPos::Numeral => LmfPos::Numeral,
            ArchivedLmfPos::Other => LmfPos::Other,
        }
    }

    /// The concept records, held as one `rkyv`-archived, zero-copy buffer.
    ///
    /// See the [module docs](super) for the transcode discipline and the
    /// endianness invariant.
    pub struct ConceptStore {
        /// The archived record vector's bytes — a 16-aligned `rkyv` buffer,
        /// validated ONCE in [`build`](Self::build) and immutable since.
        buf: AlignedVec<16>,
        /// Number of records (`= archived_records().len()`), cached to answer
        /// [`len`](Self::len) and bound [`get`](Self::get) without re-reading the
        /// archive root.
        len: usize,
    }

    /// `ConceptStore` is `Sync`: its only fields are an [`AlignedVec`] (which rkyv
    /// declares `Send + Sync`) and a `usize` — no interior mutability. This keeps
    /// `English`'s process-wide `OnceLock<English>` static valid.
    const _: fn() = || {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ConceptStore>();
    };

    impl ConceptStore {
        /// Transcode the owned `Vec<Concept>` build into the archived buffer ONCE,
        /// consuming and freeing the vector. The records' order (hence each
        /// record's index-as-id) is preserved exactly, so [`get`](Self::get)
        /// returns identical data to the owned fallback.
        pub fn build(concepts: Vec<Concept>) -> Self {
            let len = concepts.len();

            // PUT the owned concepts through the shared `RkyvLens` (project the
            // `ConceptRecords` mirror, `rkyv`-serialize to a 16-aligned buffer),
            // then validate ONCE here at materialize so every hot query can read
            // the immutable buffer through `access_unchecked` (the validate-once /
            // access_unchecked-many discipline). The owned `concepts` (and the
            // transient mirror) drop at the end of this function; only `buf`
            // survives.
            let buf = ConceptLens::put_aligned(&concepts);
            ConceptLens::access(buf.as_slice())
                .expect("freshly-serialized concept records must bytecheck-validate");

            Self { buf, len }
        }

        /// The archived record root, borrowed zero-copy from the buffer.
        #[inline]
        fn root(&self) -> &ArchivedRecords {
            // SAFETY: `buf` was produced by `RkyvLens::put_aligned` in `build` (so
            // it is 16-aligned and structurally sound), `bytecheck`-validated ONCE
            // there via `access`, and is never mutated after (no interior
            // mutability; the store is immutable). This is the deliberate
            // `access_unchecked` the runtime uses to pay bytecheck exactly once.
            unsafe { ConceptLens::access_unchecked(self.buf.as_slice()) }
        }

        /// The record at `id` (its index), or `None` if out of range.
        pub fn get(&self, id: ConceptId) -> Option<ConceptView<'_>> {
            let idx = id.value() as usize;
            if idx >= self.len {
                return None;
            }
            Some(ConceptView::Archived {
                id,
                rec: &self.root().records[idx],
            })
        }

        /// Number of records.
        pub fn len(&self) -> usize {
            self.len
        }

        /// Whether the store holds no records.
        pub fn is_empty(&self) -> bool {
            self.len == 0
        }

        /// Every record, as a [`ConceptView`], in id order.
        pub fn iter(&self) -> impl Iterator<Item = ConceptView<'_>> {
            self.root()
                .records
                .iter()
                .enumerate()
                .map(|(i, rec)| ConceptView::Archived {
                    id: ConceptId::new(i as u64),
                    rec,
                })
        }
    }

    impl core::fmt::Debug for ConceptStore {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("ConceptStore")
                .field("len", &self.len())
                .finish()
        }
    }
}

// ── record cast unit tests (archived path) ───────────────────────────────────
//
// Direct coverage of the zero-copy record access — `get` casting an index into the
// buffer to a `ConceptView::Archived` borrow of an `ArchivedConceptRecord`, whose
// `original_id`/`pos`/`lemmas`/`definitions`/`examples` accessors read through
// rkyv's own archived accessors. A small KNOWN two-record store (distinct pos,
// single- and multi-valued string fields, one empty field), asserting the archived
// view reads each field exactly, an out-of-range id → `None`, AND the archived
// view reads field-identically to the owned fallback built from the SAME records.
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod cast_tests {
    use super::ConceptStore; // the archived, zero-copy store (the crate-level re-export)
    use super::owned::ConceptStore as OwnedStore; // the owned fallback (compiled under `test`)
    use super::{Concept, ConceptId, ConceptView, LmfPos};
    use alloc::string::String;
    use alloc::vec::Vec;

    fn cid(i: u64) -> ConceptId {
        ConceptId::new(i)
    }

    /// Two KNOWN concept records (the owned build the two representations share). A
    /// record's id IS its index (§module docs): the archived build drops `id` and
    /// re-derives it from the slot, the owned build reads it back — so each fixture
    /// record's `id` is set to its position and the two must agree.
    fn fixture() -> Vec<Concept> {
        alloc::vec![
            Concept {
                id: cid(0),
                original_id: String::from("oewn-00001740-n"),
                pos: LmfPos::Noun,
                lemmas: alloc::vec![String::from("entity")],
                definitions: alloc::vec![String::from("that which is perceived or known")],
                examples: Vec::new(),
            },
            Concept {
                id: cid(1),
                original_id: String::from("oewn-02604760-v"),
                pos: LmfPos::Verb,
                lemmas: alloc::vec![String::from("be"), String::from("exist")],
                definitions: alloc::vec![
                    String::from("have the quality of being"),
                    String::from("occupy a certain position"),
                ],
                examples: alloc::vec![String::from("there is a God")],
            },
        ]
    }

    /// Assert two views expose byte/value-identical `id`/`original_id`/`pos`/
    /// `lemmas`/`definitions`/`examples` — the fields read through the archived
    /// cast against the same fields read from the owned record.
    fn assert_views_identical(a: ConceptView<'_>, b: ConceptView<'_>) {
        assert_eq!(a.id(), b.id());
        assert_eq!(a.original_id(), b.original_id());
        assert_eq!(a.pos(), b.pos());
        assert_eq!(
            a.lemmas().collect::<Vec<_>>(),
            b.lemmas().collect::<Vec<_>>()
        );
        assert_eq!(
            a.definitions().collect::<Vec<_>>(),
            b.definitions().collect::<Vec<_>>()
        );
        assert_eq!(
            a.examples().collect::<Vec<_>>(),
            b.examples().collect::<Vec<_>>()
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn archived_get_reads_the_known_records() {
        let store = ConceptStore::build(fixture());
        assert_eq!(store.len(), 2);
        assert!(!store.is_empty());

        let c0 = store.get(cid(0)).expect("id 0 is in range");
        assert_eq!(c0.id(), cid(0));
        assert_eq!(c0.original_id(), "oewn-00001740-n");
        assert_eq!(c0.pos(), LmfPos::Noun);
        assert_eq!(c0.lemmas().collect::<Vec<_>>(), ["entity"]);
        assert_eq!(
            c0.definitions().collect::<Vec<_>>(),
            ["that which is perceived or known"]
        );
        assert_eq!(c0.examples().collect::<Vec<_>>(), Vec::<&str>::new());

        let c1 = store.get(cid(1)).expect("id 1 is in range");
        assert_eq!(c1.id(), cid(1));
        assert_eq!(c1.pos(), LmfPos::Verb);
        assert_eq!(c1.lemmas().collect::<Vec<_>>(), ["be", "exist"]);
        assert_eq!(
            c1.definitions().collect::<Vec<_>>(),
            ["have the quality of being", "occupy a certain position"]
        );
        assert_eq!(c1.examples().collect::<Vec<_>>(), ["there is a God"]);

        // Out-of-range id → None (the `idx >= len` guard).
        assert!(store.get(cid(2)).is_none());
        assert!(store.get(cid(99)).is_none());
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn archived_get_is_identical_to_the_owned_fallback() {
        let archived = ConceptStore::build(fixture());
        let owned = OwnedStore::build(fixture());
        assert_eq!(archived.len(), owned.len());
        assert_eq!(archived.is_empty(), owned.is_empty());
        // Every record reads field-identically through the archived cast and the
        // owned record …
        for i in 0..archived.len() as u64 {
            let a = archived.get(cid(i)).expect("in range");
            let o = owned.get(cid(i)).expect("in range");
            assert_views_identical(a, o);
        }
        // … `iter()` walks the same records in id order …
        for (a, o) in archived.iter().zip(owned.iter()) {
            assert_views_identical(a, o);
        }
        // … and an out-of-range id → None on both.
        assert!(archived.get(cid(2)).is_none());
        assert!(owned.get(cid(2)).is_none());
    }
}
