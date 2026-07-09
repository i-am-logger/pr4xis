//! The English verb → transitivity index as a compact, zero-copy dictionary.
//!
//! `English` maps every verb surface (from the WordNet subcategorization frames,
//! [`build_verb_transitivity`](crate::cognitive::linguistics::language::build_verb_transitivity))
//! to the [`Transitivity`] options it can take. Held naively as a
//! `HashMap<String, Vec<Transitivity>>` this is one more owned `String`-keyed map
//! on `English`; under `prx` on a little-endian target it is transcoded ONCE, at
//! load, into a packed sorted-key dictionary and the HashMap is dropped.
//!
//! This is one instance of the shared
//! [`PackedCsrDict`]: a
//! [`SortedKeys`] dictionary whose value column is a
//! [`CheckedEnumRun`] of
//! [`Transitivity`]. Unlike the `ConceptId` runs, the payload here is a run of
//! one-byte `#[repr(u8)]` discriminants; the [`EnumBound`] impl below tells the
//! generic the valid discriminant range so every packed byte is VALIDATED at
//! build, which is what makes the zero-copy `&[Transitivity]` cast sound.

use crate::cognitive::linguistics::lexicon::pos::Transitivity;
use crate::formal::meta::packed_csr::{
    CheckedEnumRun, EnumBound, PackedCsrDict, PodElem, SortedKeys,
};

/// [`Transitivity`] is a `#[repr(u8)]` fieldless enum: its in-memory byte IS its
/// discriminant, so its little-endian serialization is that one byte.
impl PodElem for Transitivity {
    const SIZE: usize = 1;
    #[inline]
    fn le_bytes(&self) -> [u8; 8] {
        [*self as u8, 0, 0, 0, 0, 0, 0, 0]
    }
}

/// `Ditransitive` (= 2) is the largest valid discriminant; a packed byte above it
/// is not a valid variant and fails the build.
impl EnumBound for Transitivity {
    const MAX_DISCRIMINANT: u8 = Transitivity::Ditransitive as u8;
}

/// The verb→transitivity index: a sorted-key dictionary from a verb surface to
/// the run of [`Transitivity`] options it can take. `lookup(word)` returns
/// `&[Transitivity]` (empty if absent); `is_empty` / `len` round out the query
/// surface. All representation is the shared [`PackedCsrDict`].
pub type VerbTransitivityIndex = PackedCsrDict<SortedKeys, CheckedEnumRun<Transitivity>>;

// ── dictionary fixture test ──────────────────────────────────────────────────
//
// The generic zero-copy CSR laws prove the build (incl. the discriminant
// validation) + the `&[Transitivity]` cast + archived-equals-owned once,
// generically. This fixture pins the VERB-TRANSITIVITY instance's concrete reader
// results on a small known map (a multi-value surface, single-value surfaces,
// misses).
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod fixture_tests {
    use super::*;
    use alloc::string::String;
    use alloc::vec::Vec;
    use hashbrown::HashMap;

    /// Keys sort by raw bytes to `run < see < walk`, NOT insertion order.
    fn fixture() -> HashMap<String, Vec<Transitivity>> {
        let mut map: HashMap<String, Vec<Transitivity>> = HashMap::new();
        map.insert(
            String::from("walk"),
            alloc::vec![Transitivity::Intransitive],
        );
        map.insert(
            String::from("run"),
            alloc::vec![Transitivity::Intransitive, Transitivity::Transitive],
        );
        map.insert(String::from("see"), alloc::vec![Transitivity::Transitive]);
        map
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn lookup_matches_the_known_map() {
        let index = VerbTransitivityIndex::build(fixture());
        let empty: &[Transitivity] = &[];
        assert_eq!(index.lookup("walk"), &[Transitivity::Intransitive]);
        assert_eq!(
            index.lookup("run"),
            &[Transitivity::Intransitive, Transitivity::Transitive]
        );
        assert_eq!(index.lookup("see"), &[Transitivity::Transitive]);
        // Miss → empty slice; a prefix of a present key is still a miss.
        assert_eq!(index.lookup("swim"), empty);
        assert_eq!(index.lookup("ru"), empty);
        assert_eq!(index.len(), 3);
    }
}
