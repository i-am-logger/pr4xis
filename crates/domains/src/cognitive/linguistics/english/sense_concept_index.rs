//! The WordNet sense→concept index — dense `SenseId → ConceptId`.
//!
//! Every WordNet sense names exactly one synset (Fellbaum 1998): the LMF
//! `Sense` record's `synset` attribute. `English` assigns each sense a dense
//! [`SenseId`] at [`from_wordnet`](super::ontology::English::from_wordnet)
//! time (Phase 2) but, until this store, never retained which [`ConceptId`]
//! that sense belongs to — the mapping was a build-time-only local
//! (`sense_to_id`, keyed the other direction) dropped once the sense-level
//! relation folds (opposition, derivation, …) were built.
//!
//! # Why this store exists
//!
//! WordNet's antonymy (`Opposition`, see
//! [`RelationKind::Opposition`](super::relation_store::RelationKind::Opposition))
//! is SENSE-keyed — `big#1` opposes `small#1`, not the *concept* "big" wholesale
//! — but every other reasoning surface in `English` (`is_a`, `parts`, the
//! [`ComposedReasoner`](crate::cognitive::linguistics::composed::ComposedReasoner)'s
//! relation-parametric `reaches`) is CONCEPT-keyed. Answering "does the concept
//! *big* oppose the concept *small*?" therefore requires bridging concept → its
//! senses → each sense's direct opposition edges → back to the target's
//! concept. This store is the forward leg of that bridge (`SenseId →
//! ConceptId`); [`concept_senses_index`](super::concept_senses_index) derives
//! the inverse (`ConceptId → [SenseId]`) from it at construction.
//!
//! # The representation
//!
//! One instance of the shared
//! [`PackedCsrFamily`]: a
//! [`DenseId`](crate::formal::meta::packed_csr::DenseId)-indexed family whose
//! sole column is a [`PodScalar`] of
//! [`ConceptId`] (a sense names exactly ONE concept — `SenseId → ConceptId` is
//! a total function, not a run), labelled by the single-variant
//! [`SenseToConcept`] marker. A `PackedCsrFamily` with one column is the same
//! shape [`TaxonomyStore`](super::taxonomy_store::TaxonomyStore) (two columns)
//! and [`RelationStore`](super::relation_store::RelationStore) (27 columns)
//! already instantiate — reusing the one hand-audited generic rather than
//! adding a new persistence primitive for a dense-keyed, no-label-axis store
//! (`PackedCsrDict` binary-searches STRING keys only; no dense-id-keyed,
//! label-free counterpart exists, and inventing one is a bigger, riskier
//! change to the shared `packed_csr` module than reusing `PackedCsrFamily`
//! with `COUNT = 1`).

use hashbrown::HashMap;

use super::ontology::{ConceptId, SenseId};
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::meta::packed_csr::{LabelKind, PackedCsrFamily, PodScalar};

/// The sole column of the sense→concept family — a marker so the dense
/// sense-id space can be addressed through the shared [`PackedCsrFamily`]
/// engine. `SenseId → ConceptId` is a single functional dependency, not a
/// multi-relation family, so `COUNT` is 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SenseToConcept;

impl LabelKind for SenseToConcept {
    const COUNT: usize = 1;
    fn index(self) -> usize {
        0
    }
    fn all() -> &'static [Self] {
        &[SenseToConcept]
    }
}

/// The sense→concept index: `SenseId` (dense, `0..sense_count`) → the one
/// [`ConceptId`] (synset) that sense belongs to. [`concept_of`](Self::concept_of)
/// is the labelled-column read.
pub struct SenseConceptIndex(PackedCsrFamily<SenseToConcept, PodScalar<ConceptId>>);

impl SenseConceptIndex {
    /// Transcode the owned `SenseId → ConceptId` map into the packed family
    /// ONCE, consuming and freeing it.
    pub fn build(sense_concept: HashMap<SenseId, ConceptId>, sense_count: usize) -> Self {
        Self(PackedCsrFamily::build(alloc::vec![(
            sense_count,
            sense_concept
        )]))
    }

    /// The concept (synset) `sense` belongs to (`None` if `sense` is absent
    /// from the source WordNet dump or out of range).
    pub fn concept_of(&self, sense: SenseId) -> Option<ConceptId> {
        self.0.column(SenseToConcept, sense)
    }

    /// The dense sense count (`0..sense_count` are valid sense ids).
    pub fn sense_count(&self) -> Quantity {
        Quantity::from_unit(self.0.row_count(SenseToConcept) as f64, &unit::UNITLESS)
    }
}

/// The store-bundle serialization surface — the packed family bytes, the
/// per-column layout table the buffer does not carry, and the VALIDATING
/// re-entry for bytes this process did not pack. Archived (`prx` +
/// little-endian) only: the bundle serializes the packed representation.
#[cfg(all(feature = "prx", target_endian = "little"))]
impl SenseConceptIndex {
    /// The packed family bytes (see
    /// [`PackedCsrFamily::as_bytes`](crate::formal::meta::packed_csr::ArchivedCsrFamily::as_bytes)).
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// The per-column `(row_count, edge_count)` table (always one entry, the
    /// [`SenseToConcept`] column) — carried in the bundle frame beside the
    /// bytes.
    pub fn col_layout(&self) -> alloc::vec::Vec<(usize, usize)> {
        self.0.col_layout()
    }

    /// Recover a sense-concept index from UNTRUSTED packed bytes + the
    /// declared column table, through the fail-closed
    /// [`ArchivedCsrFamily::from_untrusted_buf`](crate::formal::meta::packed_csr::ArchivedCsrFamily::from_untrusted_buf)
    /// validation.
    pub fn from_untrusted_buf(
        buf: rkyv::util::AlignedVec<16>,
        cols: &[(usize, usize)],
    ) -> Result<Self, crate::formal::meta::packed_csr::PackedCsrError> {
        Ok(Self(PackedCsrFamily::from_untrusted_buf(buf, cols)?))
    }
}

impl core::fmt::Debug for SenseConceptIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SenseConceptIndex")
            .field("sense_count", &self.sense_count())
            .finish()
    }
}

// ── CSR family fixture test ──────────────────────────────────────────────────
//
// The generic zero-copy CSR laws prove the build + the `&[ConceptId]` cast +
// archived-equals-owned once, generically. This fixture pins the SENSE-CONCEPT
// instance's concrete `concept_of` results on a small known map (a sense with
// no assigned concept — a genuine gap — plus an out-of-range id).
#[cfg(all(test, feature = "prx", target_endian = "little"))]
mod fixture_tests {
    use super::*;

    fn cid(i: u64) -> ConceptId {
        ConceptId::new(i)
    }
    fn sid(i: u64) -> SenseId {
        SenseId::new(i)
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn concept_of_matches_the_known_map() {
        let mut map: HashMap<SenseId, ConceptId> = HashMap::new();
        map.insert(sid(0), cid(7));
        map.insert(sid(2), cid(9));
        // sid(1) is deliberately absent — a real WordNet dump has none, but the
        // representation must not panic on the gap; it reads back `None`.
        let index = SenseConceptIndex::build(map, 3);

        assert_eq!(index.concept_of(sid(0)), Some(cid(7)));
        assert_eq!(index.concept_of(sid(1)), None);
        assert_eq!(index.concept_of(sid(2)), Some(cid(9)));
        // Out of the declared `0..3` range → also `None`, not a panic.
        assert_eq!(index.concept_of(sid(3)), None);
        assert_eq!(index.sense_count().value, 3.0);
    }
}
