//! The English word→concept index as a compact, zero-copy dictionary.
//!
//! `English` maps ~160k written forms (lemmas + inflected `<Form>`s) to the
//! synset [`ConceptId`]s they can express. Held naively as a
//! `HashMap<String, Vec<ConceptId>>` this is the single largest resident cost of
//! the loaded reasoner — ~160k owned `String` keys (each its own heap block +
//! capacity slack + 24-byte control word) plus ~160k `Vec<ConceptId>` value
//! allocations. The resident-memory gate
//! ([`examples/resident_memory`](../../../../../examples/resident_memory.rs))
//! attributes ~160 MiB of the +335 MiB embedded-English footprint to this map.
//!
//! # The representation
//!
//! Under `prx` on a little-endian target the map is transcoded ONCE, at load, into
//! a single [`AlignedVec<16>`] holding a sorted-key dictionary — the SAME
//! "materialize the owned build into a packed archive, free the intermediate"
//! discipline the compact `.prx` load path uses, applied to the word index. The
//! HashMap is consumed by [`WordIndex::build`] and dropped; only the buffer
//! survives. This is a REPLACEMENT of the owned build, not an addition on top of
//! it, so steady-state resident memory falls by the reclaimed map while load time
//! is unchanged (the transcode is a sort + memcpy over data already in hand — no
//! rkyv serialize, no bytecheck).
//!
//! Buffer layout (all offsets in bytes from the 16-aligned base):
//!
//! ```text
//! [ 0..16)                 header:  n:u32, id_count:u32, key_blob_len:u32, _pad:u32
//! [16 ..)                  id_array:    id_count × 8   packed little-endian ConceptId
//! then (n+1) × 4           id_offsets:  CSR offsets into id_array (in ConceptId units)
//! then (n+1) × 4           key_offsets: CSR offsets into key_blob (in bytes)
//! then key_blob_len        key_blob:    concatenated sorted key bytes
//! ```
//!
//! `id_array` leads the body so it inherits the buffer's 16-alignment (hence
//! 8-alignment), which is what lets [`WordIndex::lookup`] hand back a slice of the
//! packed ids AS a `&[ConceptId]` with a zero-copy cast — no per-query allocation,
//! the same `access_unchecked` discipline the runtime's archive lever uses.
//!
//! # Endianness invariant (why this variant is `little`-only)
//!
//! The zero-copy cast reinterprets the packed id bytes as `&[ConceptId]`, where
//! [`ConceptId`] = `Ref<4> { value: u64 }` is a single-`u64` POD. The cast is
//! sound only where the machine's native integer byte order equals the
//! little-endian order the ids were written with. wasm32 and x86-64 — the two
//! targets praxis ships — are both little-endian. The whole zero-copy variant is
//! therefore compiled only under `cfg(target_endian = "little")`; a (hypothetical)
//! big-endian target falls back to the owned `HashMap`, exactly as a non-`prx`
//! build does. `const _` below asserts the invariant at compile time.
//!
//! Reference: Hill, D. *rkyv: zero-copy deserialization framework for Rust* (v0.8)
//! — `AlignedVec` + `access_unchecked` are rkyv's own little-endian zero-copy
//! primitives; this module reuses their aligned-buffer discipline for the word
//! index. See <https://github.com/rkyv/rkyv>.

use alloc::vec::Vec;

use hashbrown::HashMap;

use super::ontology::ConceptId;

/// The zero-copy, `prx`-gated word index (little-endian targets).
#[cfg(all(feature = "prx", target_endian = "little"))]
pub use archived::WordIndex;

/// The owned fallback word index (no `prx`, or a big-endian target).
#[cfg(not(all(feature = "prx", target_endian = "little")))]
pub use owned::WordIndex;

// ── shared surface ──────────────────────────────────────────────────────────
//
// Methods and trait impls that are identical for both representations, written
// against the common `lookup` / `len` / `words` API so they need no `cfg`.

impl WordIndex {
    /// Does this index hold `word` at all? — `true` iff [`lookup`](Self::lookup)
    /// returns a non-empty concept slice.
    pub fn contains(&self, word: &str) -> bool {
        !self.lookup(word).is_empty()
    }

    /// Is the index empty (no words)?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl PartialEq for WordIndex {
    /// Logical equality: the SAME set of words, each mapping to the SAME
    /// `ConceptId` slice in the SAME order. Representation-agnostic (compares
    /// through `words`/`lookup`), so an owned index and an archived one built from
    /// the same source compare equal. Used by the corpus round-trip gates.
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.words().all(|w| self.lookup(w) == other.lookup(w))
    }
}

impl core::fmt::Debug for WordIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_map()
            .entries(self.words().map(|w| (w, self.lookup(w))))
            .finish()
    }
}

/// The owned fallback: a plain `HashMap`. Kept as the mandatory non-`prx` (and
/// big-endian) path, mirroring the `english_load_owned` `prx` split.
///
/// Compiled ALSO under `test` on the archived (`prx` + little-endian) path so the
/// cast unit tests can build both representations from the same map and assert the
/// zero-copy archived index returns id slices value-identical to this owned
/// fallback (it is only `pub use`d as `WordIndex` on the non-archived path, so
/// there is no re-export conflict).
#[cfg(any(not(all(feature = "prx", target_endian = "little")), test))]
mod owned {
    use super::*;

    /// Word text → the concept ids it can express, held as an owned map.
    pub struct WordIndex {
        map: HashMap<alloc::string::String, Vec<ConceptId>>,
    }

    impl WordIndex {
        /// Retain the owned build as-is — the fallback keeps the `HashMap`.
        pub fn build(map: HashMap<alloc::string::String, Vec<ConceptId>>) -> Self {
            Self { map }
        }

        /// Look up a word → all concepts it can express (empty slice if absent).
        pub fn lookup(&self, word: &str) -> &[ConceptId] {
            self.map.get(word).map(|v| v.as_slice()).unwrap_or(&[])
        }

        /// Number of distinct words.
        pub fn len(&self) -> usize {
            self.map.len()
        }

        /// Every word in the index, as `&str` (unordered — this is the owned
        /// fallback; the archived variant yields them sorted).
        pub fn words(&self) -> impl Iterator<Item = &str> {
            self.map.keys().map(|s| s.as_str())
        }
    }
}

/// The zero-copy archived index: a single [`AlignedVec<16>`] holding a sorted-key
/// dictionary, queried by binary search + a zero-copy id-slice cast.
#[cfg(all(feature = "prx", target_endian = "little"))]
mod archived {
    use rkyv::util::AlignedVec;

    use super::*;

    /// Byte length of the fixed header (`n`, `id_count`, `key_blob_len`, pad —
    /// four `u32`s), and the offset at which `id_array` begins. Sized to keep the
    /// buffer's 16-alignment on the `id_array` (a multiple of 8 ≥ the `u64` ids'
    /// alignment).
    const HEADER: usize = 16;

    /// The soundness precondition of the zero-copy id cast: native integer byte
    /// order must equal the little-endian order ids are stored in. Enforced by the
    /// `cfg(target_endian = "little")` gate on this module; asserted here so a
    /// mis-configuration is a compile error, not a silent miscast.
    const _: () = assert!(cfg!(target_endian = "little"));

    /// `WordIndex` is `Sync`: its only field is an [`AlignedVec`] (which rkyv
    /// declares `Send + Sync`) plus `Copy` scalars — no interior mutability. This
    /// keeps `English`'s process-wide `OnceLock<English>` static valid.
    const _: fn() = || {
        fn assert_sync<T: Sync>() {}
        assert_sync::<WordIndex>();
    };

    /// Word text → concept ids, as one packed, sorted, zero-copy dictionary.
    ///
    /// See the [module docs](super) for the buffer layout and the endianness
    /// invariant.
    pub struct WordIndex {
        /// The whole dictionary: header + id_array + CSR offset arrays + key blob.
        buf: AlignedVec<16>,
        /// Number of distinct words (`= header.n`).
        n: usize,
        /// Byte offset of the `id_offsets` CSR array.
        id_offsets_at: usize,
        /// Byte offset of the `key_offsets` CSR array.
        key_offsets_at: usize,
        /// Byte offset of the `key_blob`.
        key_blob_at: usize,
    }

    impl WordIndex {
        /// Transcode the owned `HashMap` build into the packed dictionary ONCE,
        /// consuming and freeing the map. Keys are sorted by their raw UTF-8 bytes
        /// so [`lookup`](Self::lookup) can binary-search them; each word's id
        /// order is preserved exactly (identical `lookup` results to the map).
        pub fn build(map: HashMap<alloc::string::String, Vec<ConceptId>>) -> Self {
            // Sort by key bytes — the order `lookup`'s binary search assumes.
            let mut entries: Vec<(alloc::string::String, Vec<ConceptId>)> =
                map.into_iter().collect();
            entries.sort_unstable_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

            let n = entries.len();
            let id_count: usize = entries.iter().map(|(_, ids)| ids.len()).sum();
            let key_blob_len: usize = entries.iter().map(|(k, _)| k.len()).sum();

            let id_offsets_at = HEADER + id_count * 8;
            let key_offsets_at = id_offsets_at + (n + 1) * 4;
            let key_blob_at = key_offsets_at + (n + 1) * 4;
            let total = key_blob_at + key_blob_len;

            let mut buf = AlignedVec::<16>::with_capacity(total);

            // Header.
            buf.extend_from_slice(&(n as u32).to_le_bytes());
            buf.extend_from_slice(&(id_count as u32).to_le_bytes());
            buf.extend_from_slice(&(key_blob_len as u32).to_le_bytes());
            buf.extend_from_slice(&0u32.to_le_bytes());

            // id_array: every word's ids, concatenated, little-endian.
            for (_, ids) in &entries {
                for id in ids {
                    buf.extend_from_slice(&id.value().to_le_bytes());
                }
            }

            // id_offsets: CSR prefix sums over id counts (in ConceptId units).
            let mut acc = 0u32;
            buf.extend_from_slice(&acc.to_le_bytes());
            for (_, ids) in &entries {
                acc += ids.len() as u32;
                buf.extend_from_slice(&acc.to_le_bytes());
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
                "word_index buffer length must equal the computed layout size"
            );

            Self {
                buf,
                n,
                id_offsets_at,
                key_offsets_at,
                key_blob_at,
            }
            // `entries` (and the ids/keys it owns) drops here — the owned build is
            // freed; only `buf` survives.
        }

        /// Read the `i`-th little-endian `u32` of the CSR array at byte offset
        /// `base`. Uses a checked 4-byte read (no alignment requirement on the
        /// offset arrays; only `id_array` is alignment-load-bearing).
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

        /// The `i`-th key as `&str` (keys are UTF-8 by construction).
        #[inline]
        fn key_str(&self, i: usize) -> &str {
            core::str::from_utf8(self.key_bytes(i))
                .expect("word_index keys are UTF-8 by construction")
        }

        /// The `i`-th word's packed ids, cast zero-copy to `&[ConceptId]`.
        #[inline]
        fn ids_at(&self, i: usize) -> &[ConceptId] {
            let start = self.csr(self.id_offsets_at, i);
            let end = self.csr(self.id_offsets_at, i + 1);
            let byte_start = HEADER + start * 8;
            let len = end - start;
            // SAFETY: `id_array` begins at `HEADER` (16), which inherits the
            // buffer's 16-alignment (⇒ 8-aligned), so `byte_start = 16 + start*8`
            // is 8-aligned — the alignment `ConceptId` (a single `u64`) requires.
            // The `len` ConceptIds at `byte_start` are in bounds by construction
            // (the CSR offsets partition exactly `id_count` ids). Each was written
            // as a little-endian `u64`; on this little-endian-gated build that IS
            // `ConceptId`'s in-memory representation, so the reinterpretation is a
            // value-preserving zero-copy view. The returned slice borrows `self`.
            unsafe {
                let ptr = self.buf.as_ptr().add(byte_start) as *const ConceptId;
                core::slice::from_raw_parts(ptr, len)
            }
        }

        /// Look up a word → all concepts it can express, by binary search over the
        /// sorted keys. Empty slice if absent. Zero allocation; the returned slice
        /// borrows the packed buffer.
        pub fn lookup(&self, word: &str) -> &[ConceptId] {
            let target = word.as_bytes();
            let (mut lo, mut hi) = (0usize, self.n);
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                match self.key_bytes(mid).cmp(target) {
                    core::cmp::Ordering::Less => lo = mid + 1,
                    core::cmp::Ordering::Greater => hi = mid,
                    core::cmp::Ordering::Equal => return self.ids_at(mid),
                }
            }
            &[]
        }

        /// Number of distinct words.
        pub fn len(&self) -> usize {
            self.n
        }

        /// Every word in the index, as `&str`, in sorted (key-byte) order.
        pub fn words(&self) -> impl Iterator<Item = &str> {
            (0..self.n).map(move |i| self.key_str(i))
        }
    }
}

// ── dictionary cast unit tests (archived path) ───────────────────────────────
//
// Direct coverage of the zero-copy dictionary build + the unsafe id-slice cast in
// `ids_at` (the `from_raw_parts` reinterpreting packed little-endian `u64`s as
// `&[ConceptId]`): a small KNOWN word→ids map (a multi-id surface, two single-id
// surfaces, plus misses), asserting the archived `lookup` returns exactly the
// expected `&[ConceptId]`, a miss returns `&[]`, AND the archived index returns id
// slices value-identical to the owned fallback built from the SAME map.
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod cast_tests {
    use super::WordIndex; // the archived, zero-copy index (the crate-level re-export)
    use super::owned::WordIndex as OwnedIndex; // the owned fallback (compiled under `test`)
    use super::{ConceptId, HashMap};
    use alloc::string::String;
    use alloc::vec::Vec;

    fn cid(i: u64) -> ConceptId {
        ConceptId::new(i)
    }

    /// A small KNOWN word index (returned as the owned build the two representations
    /// share). Keys sort by raw bytes to `alpha < beta < gamma`.
    ///   "alpha" → [7]         (single-id surface)
    ///   "beta"  → [42]        (single-id surface — sorts between the others)
    ///   "gamma" → [3, 9, 1]   (multi-id surface — the cast must preserve run order)
    fn fixture() -> HashMap<String, Vec<ConceptId>> {
        let mut map: HashMap<String, Vec<ConceptId>> = HashMap::new();
        map.insert(String::from("alpha"), alloc::vec![cid(7)]);
        map.insert(String::from("beta"), alloc::vec![cid(42)]);
        map.insert(String::from("gamma"), alloc::vec![cid(3), cid(9), cid(1)]);
        map
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn archived_lookup_matches_the_known_map() {
        let index = WordIndex::build(fixture());
        let empty: &[ConceptId] = &[];
        // Single-id surfaces — one packed id, cast back to a length-1 slice.
        assert_eq!(index.lookup("alpha"), &[cid(7)]);
        assert_eq!(index.lookup("beta"), &[cid(42)]);
        // Multi-id surface — the cast reinterprets the whole packed run, in order.
        assert_eq!(index.lookup("gamma"), &[cid(3), cid(9), cid(1)]);
        // Miss → empty slice; a prefix of a present key is still a miss (the binary
        // search is exact, not a prefix match).
        assert_eq!(index.lookup("delta"), empty);
        assert_eq!(index.lookup("alph"), empty);
        assert_eq!(index.len(), 3);
        // Keys come back in sorted (key-byte) order.
        let words: Vec<&str> = index.words().collect();
        let expected: Vec<&str> = alloc::vec!["alpha", "beta", "gamma"];
        assert_eq!(words, expected);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn archived_lookup_is_identical_to_the_owned_fallback() {
        let archived = WordIndex::build(fixture());
        let owned = OwnedIndex::build(fixture());
        assert_eq!(archived.len(), owned.len());
        // Every word the owned map holds resolves, through the archived zero-copy
        // cast, to the byte/value-identical id slice …
        for word in owned.words() {
            assert_eq!(archived.lookup(word), owned.lookup(word), "lookup {word}");
        }
        // … and a surface neither holds is `&[]` on both.
        let empty: &[ConceptId] = &[];
        assert_eq!(archived.lookup("delta"), empty);
        assert_eq!(owned.lookup("delta"), empty);
    }
}
