//! The English word→concept index as a compact, zero-copy dictionary.
//!
//! `English` maps ~160k written forms (lemmas + inflected `<Form>`s) to the
//! synset [`ConceptId`]s they can express. Held naively as a
//! `HashMap<String, Vec<ConceptId>>` this is the single largest resident cost of
//! the loaded reasoner. Under `prx` on a little-endian target the map is
//! transcoded ONCE, at load, into a single packed sorted-key dictionary — the
//! HashMap is consumed and dropped, only the buffer survives.
//!
//! This is one instance of the shared
//! [`PackedCsrDict`](crate::formal::meta::packed_csr::PackedCsrDict): a
//! [`SortedKeys`] dictionary whose value column is a
//! [`PodRun`](crate::formal::meta::packed_csr::PodRun) of [`ConceptId`] (each
//! word maps to a run of the ids it can express). The zero-copy `&[ConceptId]`
//! cast, the CSR reader, the little-endian invariant, and the owned fallback all
//! live in that one hand-audited generic.

use super::ontology::ConceptId;
use crate::formal::meta::packed_csr::{PackedCsrDict, PodRun, SortedKeys};

/// The word→concept index: a sorted-key dictionary from a written form to the
/// run of [`ConceptId`]s it can express. `lookup(word)` returns `&[ConceptId]`
/// (empty if absent); `contains` / `is_empty` / `len` / `words` round out the
/// query surface. All representation (packing, the zero-copy cast, the owned
/// fallback) is the shared [`PackedCsrDict`].
pub type WordIndex = PackedCsrDict<SortedKeys, PodRun<ConceptId>>;

impl WordIndex {
    /// Every word in the index, as `&str` (sorted key-byte order on the archived
    /// path). A query-name alias for the generic `keys` iterator.
    pub fn words(&self) -> impl Iterator<Item = &str> + '_ {
        self.keys()
    }
}

// ── dictionary fixture test ──────────────────────────────────────────────────
//
// The generic zero-copy CSR laws (`PackedCsrGetPut` / `PackedCsrPutGet` /
// `PackedCsrZeroCopyFaithful` in `formal::meta::lens::packed_csr_laws`) prove the
// build + the `&[ConceptId]` cast + archived-equals-owned once, generically. This
// fixture pins the WORD-INDEX instance's concrete reader results on a small known
// map (a multi-id surface, single-id surfaces, misses).
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod fixture_tests {
    use super::*;
    use alloc::string::String;
    use alloc::vec::Vec;
    use hashbrown::HashMap;

    fn cid(i: u64) -> ConceptId {
        ConceptId::new(i)
    }

    /// Keys sort by raw bytes to `alpha < beta < gamma`.
    ///   "alpha" → [7]        "beta" → [42]        "gamma" → [3, 9, 1]
    fn fixture() -> HashMap<String, Vec<ConceptId>> {
        let mut map: HashMap<String, Vec<ConceptId>> = HashMap::new();
        map.insert(String::from("alpha"), alloc::vec![cid(7)]);
        map.insert(String::from("beta"), alloc::vec![cid(42)]);
        map.insert(String::from("gamma"), alloc::vec![cid(3), cid(9), cid(1)]);
        map
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lookup_matches_the_known_map() {
        let index = WordIndex::build(fixture());
        let empty: &[ConceptId] = &[];
        assert_eq!(index.lookup("alpha"), &[cid(7)]);
        assert_eq!(index.lookup("beta"), &[cid(42)]);
        // Multi-id surface — the cast preserves run order.
        assert_eq!(index.lookup("gamma"), &[cid(3), cid(9), cid(1)]);
        // Miss → empty; a prefix of a present key is still a miss (exact search).
        assert_eq!(index.lookup("delta"), empty);
        assert_eq!(index.lookup("alph"), empty);
        assert_eq!(index.len(), 3);
        assert!(index.contains("gamma"));
        assert!(!index.contains("delta"));
        let words: Vec<&str> = index.words().collect();
        assert_eq!(words, alloc::vec!["alpha", "beta", "gamma"]);
    }
}
