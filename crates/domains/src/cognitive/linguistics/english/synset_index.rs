//! The WordNet synset-id → concept index as a compact, zero-copy dictionary.
//!
//! `English` maps every synset's original WN-LMF id string (e.g.
//! `"oewn-02084071-n"`) to the dense [`ConceptId`] it was assigned at
//! [`from_wordnet`](super::ontology::English::from_wordnet) time. Held naively as
//! a `HashMap<String, ConceptId>` this is ~107k owned `String` keys; under `prx`
//! on a little-endian target the map is transcoded ONCE, at load, into a single
//! packed sorted-key dictionary and the HashMap is dropped.
//!
//! This is one instance of the shared
//! [`PackedCsrDict`](crate::formal::meta::packed_csr::PackedCsrDict): a
//! [`SortedKeys`] dictionary whose value column is a
//! [`PodScalar`](crate::formal::meta::packed_csr::PodScalar) of [`ConceptId`]
//! (each synset id names exactly ONE concept), so `lookup` returns
//! `Option<ConceptId>` — no run/offset indirection, the id read back by value.

use super::ontology::ConceptId;
use crate::formal::meta::packed_csr::{PackedCsrDict, PodScalar, SortedKeys};

/// The synset-id→concept index: a sorted-key dictionary from a synset id string
/// to the single [`ConceptId`] it names. `lookup(synset_id)` returns
/// `Option<ConceptId>`; `contains` / `is_empty` / `len` round out the query
/// surface. All representation is the shared [`PackedCsrDict`].
pub type SynsetIndex = PackedCsrDict<SortedKeys, PodScalar<ConceptId>>;

// ── dictionary fixture test ──────────────────────────────────────────────────
//
// The generic zero-copy CSR laws prove the build + archived-equals-owned once,
// generically. This fixture pins the SYNSET-INDEX instance's concrete reader
// results on a small known map (keys whose byte order differs from insertion
// order, plus misses).
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod fixture_tests {
    use super::*;
    use alloc::string::String;
    use hashbrown::HashMap;

    fn cid(i: u64) -> ConceptId {
        ConceptId::new(i)
    }

    /// Keys sort by raw bytes to
    /// `oewn-00001740-n < oewn-02084071-n < oewn-99999999-v` — NOT insertion order.
    fn fixture() -> HashMap<String, ConceptId> {
        let mut map: HashMap<String, ConceptId> = HashMap::new();
        map.insert(String::from("oewn-99999999-v"), cid(2));
        map.insert(String::from("oewn-00001740-n"), cid(0));
        map.insert(String::from("oewn-02084071-n"), cid(42));
        map
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lookup_matches_the_known_map() {
        let index = SynsetIndex::build(fixture());
        assert_eq!(index.lookup("oewn-00001740-n"), Some(cid(0)));
        assert_eq!(index.lookup("oewn-02084071-n"), Some(cid(42)));
        assert_eq!(index.lookup("oewn-99999999-v"), Some(cid(2)));
        // Miss → None; a prefix of a present key is still a miss (exact search).
        assert_eq!(index.lookup("oewn-00000000-n"), None);
        assert_eq!(index.lookup("oewn-00001740"), None);
        assert_eq!(index.len(), 3);
        assert!(index.contains("oewn-02084071-n"));
        assert!(!index.contains("nope"));
    }
}
