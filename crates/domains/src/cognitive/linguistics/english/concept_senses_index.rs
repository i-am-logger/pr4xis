//! The concept→senses inverse index — DERIVED from [`SenseConceptIndex`] at
//! construction time (never persisted, never re-parsed), mirroring
//! [`fold_index`](super::fold_index)'s "derive-from-an-already-available-store"
//! pattern: a synset's senses are recoverable by one pass over the already-
//! packed forward index, so there is no reason to carry a second bundle frame
//! for what is a pure function of the first.
//!
//! A concept (synset) can be named by several senses — its synonym set — so,
//! unlike the forward `SenseId → ConceptId` functional dependency, the inverse
//! `ConceptId → [SenseId]` is genuinely one-to-many.

use alloc::vec::Vec;

use hashbrown::HashMap;

use super::ontology::{ConceptId, SenseId};
use super::sense_concept_index::SenseConceptIndex;
use crate::formal::meta::packed_csr::{LabelKind, PackedCsrFamily, PodRun};

/// The sole column of the concept→senses family — the mirror of
/// [`SenseToConcept`](super::sense_concept_index::SenseToConcept) for the
/// inverse direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConceptToSenses;

impl LabelKind for ConceptToSenses {
    const COUNT: usize = 1;
    fn index(self) -> usize {
        0
    }
    fn all() -> &'static [Self] {
        &[ConceptToSenses]
    }
}

/// The concept→senses index: `ConceptId` (dense, `0..concept_count`) → every
/// [`SenseId`] naming that concept, in ascending order.
/// [`senses_of`](Self::senses_of) is the labelled-column read.
pub struct ConceptSensesIndex(PackedCsrFamily<ConceptToSenses, PodRun<SenseId>>);

impl ConceptSensesIndex {
    /// Every sense of `concept`, in ascending [`SenseId`] order (empty if
    /// `concept` has no senses or is out of range).
    pub fn senses_of(&self, concept: ConceptId) -> &[SenseId] {
        self.0.column(ConceptToSenses, concept)
    }
}

impl core::fmt::Debug for ConceptSensesIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ConceptSensesIndex").finish_non_exhaustive()
    }
}

/// Build the inverse from an already-packed [`SenseConceptIndex`] — one pass
/// over every sense id (`0..sense_concept.sense_count()`), grouping by its
/// concept. Each concept's sense run comes out in ascending `SenseId` order
/// because the source pass itself is ascending.
pub fn build(sense_concept: &SenseConceptIndex, concept_count: usize) -> ConceptSensesIndex {
    let mut map: HashMap<ConceptId, Vec<SenseId>> = HashMap::new();
    for i in 0..sense_concept.sense_count().value as u64 {
        let sense = SenseId::new(i);
        if let Some(concept) = sense_concept.concept_of(sense) {
            map.entry(concept).or_default().push(sense);
        }
    }
    ConceptSensesIndex(PackedCsrFamily::build(alloc::vec![(concept_count, map)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cid(i: u64) -> ConceptId {
        ConceptId::new(i)
    }
    fn sid(i: u64) -> SenseId {
        SenseId::new(i)
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn senses_of_groups_by_owning_concept_in_ascending_order() {
        let mut map: HashMap<SenseId, ConceptId> = HashMap::new();
        // Two senses (a synonym pair) both name concept 5; one names concept 6.
        map.insert(sid(0), cid(5));
        map.insert(sid(1), cid(6));
        map.insert(sid(2), cid(5));
        let forward = SenseConceptIndex::build(map, 3);

        let inverse = build(&forward, 7);
        assert_eq!(inverse.senses_of(cid(5)), &[sid(0), sid(2)]);
        assert_eq!(inverse.senses_of(cid(6)), &[sid(1)]);
        assert!(
            inverse.senses_of(cid(0)).is_empty(),
            "no sense names concept 0"
        );
        assert!(
            inverse.senses_of(cid(100)).is_empty(),
            "out-of-range concept id reads back empty, not a panic"
        );
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_sense_with_no_assigned_concept_contributes_no_entry() {
        // sid(1) is a deliberate gap (no `synset_to_concept` hit, e.g. malformed
        // source data) — `concept_of` reads back `None` for it, and the inverse
        // build must not fabricate a phantom grouping for it.
        let mut map: HashMap<SenseId, ConceptId> = HashMap::new();
        map.insert(sid(0), cid(1));
        let forward = SenseConceptIndex::build(map, 2);
        let inverse = build(&forward, 2);
        assert_eq!(inverse.senses_of(cid(1)), &[sid(0)]);
    }
}
