//! The WordNet synset-id → concept index as a compact, zero-copy dictionary.
//!
//! `English` maps every synset's original WN-LMF id string (e.g.
//! `"oewn-02084071-n"`) to the dense [`ConceptId`] it was assigned at
//! [`from_wordnet`](super::ontology::English::from_wordnet) time — the reverse of
//! [`ConceptView::original_id`](super::concept_store::ConceptView::original_id).
//! Held naively as a `HashMap<String, ConceptId>` this is ~107k owned `String`
//! keys (each its own heap block + capacity slack + 24-byte control word), the
//! largest single owned map remaining on `English` after the
//! [`word_index`](super::word_index) / [`concept_store`](super::concept_store) /
//! [`taxonomy_store`](super::taxonomy_store) reclaims.
//!
//! # The representation
//!
//! This is a SIMPLER [`word_index`](super::word_index): every key maps to exactly
//! ONE [`ConceptId`] (a synset id names one synset), so the id side is a plain
//! parallel array — one id per key — with no CSR run/offset indirection. Under
//! `prx` on a little-endian target the map is transcoded ONCE, at load, into a
//! single [`AlignedVec<16>`] holding a sorted-key dictionary; the owned
//! `HashMap` is consumed by [`SynsetIndex::build`] and dropped, only the buffer
//! survives — a REPLACEMENT of the owned build, not an addition on top of it.
//!
//! Buffer layout (all offsets in bytes from the 16-aligned base):
//!
//! ```text
//! [ 0..16)                 header:  n:u32, key_blob_len:u32, _pad:u32, _pad:u32
//! [16 ..)                  id_array:    n × 8   packed little-endian ConceptId (one per key)
//! then (n+1) × 4           key_offsets: CSR offsets into key_blob (in bytes)
//! then key_blob_len        key_blob:    concatenated sorted key bytes
//! ```
//!
//! `id_array` leads the body so it inherits the buffer's 16-alignment (hence
//! 8-alignment); [`SynsetIndex::lookup`] binary-searches the sorted keys and reads
//! the parallel id at the matched slot with a plain checked little-endian 8-byte
//! read — the id is returned by value ([`ConceptId`] is `Copy`), so no borrow of
//! the buffer escapes and no `unsafe` slice cast is required (unlike the multi-id
//! [`word_index`](super::word_index), whose run is handed back as a borrowed
//! `&[ConceptId]`).
//!
//! # Endianness invariant (why this variant is `little`-only)
//!
//! The packed ids are read back as little-endian `u64`s. That read is
//! value-correct only where the machine's native integer byte order matches the
//! little-endian order the ids were written with. wasm32 and x86-64 — the two
//! targets praxis ships — are both little-endian. The whole archived variant is
//! therefore compiled only under `cfg(target_endian = "little")`; a (hypothetical)
//! big-endian target falls back to the owned `HashMap`, exactly as a non-`prx`
//! build does. `const _` below asserts the invariant at compile time.
//!
//! Reference: Hill, D. *rkyv: zero-copy deserialization framework for Rust* (v0.8)
//! — `AlignedVec` is rkyv's own little-endian aligned buffer; this module reuses
//! its aligned-buffer discipline for the synset index. See
//! <https://github.com/rkyv/rkyv>.

use hashbrown::HashMap;

use super::ontology::ConceptId;

/// The zero-copy, `prx`-gated synset index (little-endian targets).
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::SynsetIndex;

/// The owned fallback synset index (no `prx`, or a big-endian target).
#[cfg(not(all(feature = "prx", target_endian = "little")))]
pub use owned::SynsetIndex;

// ── shared surface ──────────────────────────────────────────────────────────

impl SynsetIndex {
    /// Does this index hold `synset_id`? — `true` iff [`lookup`](Self::lookup)
    /// returns `Some`.
    pub fn contains(&self, synset_id: &str) -> bool {
        self.lookup(synset_id).is_some()
    }

    /// Is the index empty (no synsets)?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl core::fmt::Debug for SynsetIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SynsetIndex")
            .field("len", &self.len())
            .finish()
    }
}

// ── owned fallback ───────────────────────────────────────────────────────────

/// The owned fallback: a plain `HashMap`. Kept as the mandatory non-`prx` (and
/// big-endian) path, mirroring the [`word_index`](super::word_index) split.
///
/// Compiled ALSO under `test` on the archived (`prx` + little-endian) path so the
/// cast unit tests can build both representations from the same map and assert the
/// archived index returns id-identical results to this owned fallback.
#[cfg(any(not(all(feature = "prx", target_endian = "little")), test))]
mod owned {
    use super::*;

    /// Synset id text → the concept id it names, held as an owned map.
    pub struct SynsetIndex {
        map: HashMap<alloc::string::String, ConceptId>,
    }

    impl SynsetIndex {
        /// Retain the owned build as-is — the fallback keeps the `HashMap`.
        pub fn build(map: HashMap<alloc::string::String, ConceptId>) -> Self {
            Self { map }
        }

        /// Look up a synset id → the concept it names (`None` if absent).
        pub fn lookup(&self, synset_id: &str) -> Option<ConceptId> {
            self.map.get(synset_id).copied()
        }

        /// Number of distinct synset ids.
        pub fn len(&self) -> usize {
            self.map.len()
        }
    }
}

// ── archived variant ─────────────────────────────────────────────────────────

/// The zero-copy archived index: a single [`AlignedVec<16>`] holding a sorted-key
/// dictionary with a parallel packed id array, queried by binary search + a
/// checked little-endian id read.
#[cfg(all(feature = "prx", target_endian = "little"))]
mod archived {
    use alloc::vec::Vec;

    use rkyv::util::AlignedVec;

    use super::*;

    /// Byte length of the fixed header (`n`, `key_blob_len`, two pad words — four
    /// `u32`s), and the offset at which `id_array` begins. Sized to keep the
    /// buffer's 16-alignment on the `id_array`.
    const HEADER: usize = 16;

    /// The soundness precondition of the little-endian id read: native integer
    /// byte order must equal the little-endian order ids are stored in. Enforced by
    /// the `cfg(target_endian = "little")` gate on this module; asserted here so a
    /// mis-configuration is a compile error, not a silent miscast.
    const _: () = assert!(cfg!(target_endian = "little"));

    /// `SynsetIndex` is `Sync`: its only field is an [`AlignedVec`] (which rkyv
    /// declares `Send + Sync`) plus `Copy` scalars — no interior mutability. This
    /// keeps `English`'s process-wide `OnceLock<English>` static valid.
    const _: fn() = || {
        fn assert_sync<T: Sync>() {}
        assert_sync::<SynsetIndex>();
    };

    /// Synset id text → the concept it names, as one packed, sorted dictionary.
    ///
    /// See the [module docs](super) for the buffer layout and the endianness
    /// invariant.
    pub struct SynsetIndex {
        /// The whole dictionary: header + id_array + key_offsets + key blob.
        buf: AlignedVec<16>,
        /// Number of distinct synset ids (`= header.n`).
        n: usize,
        /// Byte offset of the `key_offsets` CSR array.
        key_offsets_at: usize,
        /// Byte offset of the `key_blob`.
        key_blob_at: usize,
    }

    impl SynsetIndex {
        /// Transcode the owned `HashMap` build into the packed dictionary ONCE,
        /// consuming and freeing the map. Keys are sorted by their raw UTF-8 bytes
        /// so [`lookup`](Self::lookup) can binary-search them; each key's id is
        /// written in the parallel slot (identical `lookup` results to the map).
        pub fn build(map: HashMap<alloc::string::String, ConceptId>) -> Self {
            // Sort by key bytes — the order `lookup`'s binary search assumes.
            let mut entries: Vec<(alloc::string::String, ConceptId)> = map.into_iter().collect();
            entries.sort_unstable_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

            let n = entries.len();
            let key_blob_len: usize = entries.iter().map(|(k, _)| k.len()).sum();

            let key_offsets_at = HEADER + n * 8;
            let key_blob_at = key_offsets_at + (n + 1) * 4;
            let total = key_blob_at + key_blob_len;

            let mut buf = AlignedVec::<16>::with_capacity(total);

            // Header.
            buf.extend_from_slice(&(n as u32).to_le_bytes());
            buf.extend_from_slice(&(key_blob_len as u32).to_le_bytes());
            buf.extend_from_slice(&0u32.to_le_bytes());
            buf.extend_from_slice(&0u32.to_le_bytes());

            // id_array: one little-endian id per key, in sorted-key order.
            for (_, id) in &entries {
                buf.extend_from_slice(&id.value().to_le_bytes());
            }

            // key_offsets: CSR prefix sums over key byte lengths.
            let mut acc = 0u32;
            buf.extend_from_slice(&acc.to_le_bytes());
            for (k, _) in &entries {
                acc += k.len() as u32;
                buf.extend_from_slice(&acc.to_le_bytes());
            }

            // key_blob: concatenated sorted key bytes.
            for (k, _) in &entries {
                buf.extend_from_slice(k.as_bytes());
            }

            assert_eq!(
                buf.len(),
                total,
                "synset_index buffer length must equal the computed layout size"
            );

            Self {
                buf,
                n,
                key_offsets_at,
                key_blob_at,
            }
            // `entries` (and the keys it owns) drops here — the owned build is
            // freed; only `buf` survives.
        }

        /// Read the `i`-th little-endian `u32` of the CSR offset array at byte
        /// offset `base`. A checked 4-byte read.
        #[inline]
        fn csr(&self, base: usize, i: usize) -> usize {
            let at = base + i * 4;
            let b = &self.buf.as_slice()[at..at + 4];
            u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize
        }

        /// The `i`-th key's raw bytes.
        #[inline]
        fn key_bytes(&self, i: usize) -> &[u8] {
            let s = self.key_blob_at + self.csr(self.key_offsets_at, i);
            let e = self.key_blob_at + self.csr(self.key_offsets_at, i + 1);
            &self.buf.as_slice()[s..e]
        }

        /// The `i`-th key's packed [`ConceptId`], read as a checked little-endian
        /// `u64`. Returned by value (`ConceptId` is `Copy`) — no buffer borrow
        /// escapes, so this needs no `unsafe` slice cast.
        #[inline]
        fn id_at(&self, i: usize) -> ConceptId {
            let at = HEADER + i * 8;
            let b = &self.buf.as_slice()[at..at + 8];
            ConceptId::new(u64::from_le_bytes([
                b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
            ]))
        }

        /// Look up a synset id → the concept it names, by binary search over the
        /// sorted keys. `None` if absent.
        pub fn lookup(&self, synset_id: &str) -> Option<ConceptId> {
            let target = synset_id.as_bytes();
            let (mut lo, mut hi) = (0usize, self.n);
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                match self.key_bytes(mid).cmp(target) {
                    core::cmp::Ordering::Less => lo = mid + 1,
                    core::cmp::Ordering::Greater => hi = mid,
                    core::cmp::Ordering::Equal => return Some(self.id_at(mid)),
                }
            }
            None
        }

        /// Number of distinct synset ids.
        pub fn len(&self) -> usize {
            self.n
        }
    }
}

// ── dictionary lookup unit tests (archived path) ─────────────────────────────
//
// Direct coverage of the zero-copy dictionary build + the little-endian id read: a
// small KNOWN synset-id → ConceptId map (keys chosen so their byte order differs
// from insertion order), asserting the archived `lookup` returns exactly the
// expected id, a miss returns `None`, AND the archived index returns ids identical
// to the owned fallback built from the SAME map.
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod cast_tests {
    use super::SynsetIndex; // the archived, zero-copy index (the crate-level re-export)
    use super::owned::SynsetIndex as OwnedIndex; // the owned fallback (compiled under `test`)
    use super::{ConceptId, HashMap};
    use alloc::string::String;

    fn cid(i: u64) -> ConceptId {
        ConceptId::new(i)
    }

    /// A small KNOWN synset index. Keys sort by raw bytes to
    /// `oewn-00001740-n < oewn-02084071-n < oewn-99999999-v`, which is NOT the
    /// insertion order below — so the build's sort is exercised.
    fn fixture() -> HashMap<String, ConceptId> {
        let mut map: HashMap<String, ConceptId> = HashMap::new();
        map.insert(String::from("oewn-99999999-v"), cid(2));
        map.insert(String::from("oewn-00001740-n"), cid(0));
        map.insert(String::from("oewn-02084071-n"), cid(42));
        map
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn archived_lookup_matches_the_known_map() {
        let index = SynsetIndex::build(fixture());
        assert_eq!(index.lookup("oewn-00001740-n"), Some(cid(0)));
        assert_eq!(index.lookup("oewn-02084071-n"), Some(cid(42)));
        assert_eq!(index.lookup("oewn-99999999-v"), Some(cid(2)));
        // Miss → None; a prefix of a present key is still a miss (the binary
        // search is exact, not a prefix match).
        assert_eq!(index.lookup("oewn-00000000-n"), None);
        assert_eq!(index.lookup("oewn-00001740"), None);
        assert_eq!(index.len(), 3);
        assert!(index.contains("oewn-02084071-n"));
        assert!(!index.contains("nope"));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn archived_lookup_is_identical_to_the_owned_fallback() {
        let archived = SynsetIndex::build(fixture());
        let owned = OwnedIndex::build(fixture());
        assert_eq!(archived.len(), owned.len());
        for key in ["oewn-00001740-n", "oewn-02084071-n", "oewn-99999999-v"] {
            assert_eq!(archived.lookup(key), owned.lookup(key), "lookup {key}");
        }
        // A synset id neither holds → None on both.
        assert_eq!(archived.lookup("absent"), None);
        assert_eq!(owned.lookup("absent"), None);
    }
}
