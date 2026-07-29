//! The loaded WordNet↔SUMO mapping table, indexed for the corroboration query:
//! do two WordNet concepts map to the same SUMO term?
//!
//! Mirrors [`crate::cognitive::linguistics::english::ontology::english_loaded`]'s
//! shape: a process-wide cached, indexed view over the typed
//! [`super::ontology::Sumo`] data the reader produces — built once, queried
//! many times.
//!
//! ## `ConceptId`-indexed, resolved OFFLINE (no WordNet at load)
//!
//! Every committed row is already `(ConceptId, term, relation)` — the
//! synset→concept resolution was done offline by `super::regenerate` via the
//! version-stable WordNet SENSE KEY path, because SUMO's raw PWN-3.0 synset
//! offsets do NOT match Open English WordNet 2025's synset ids, and the compact
//! runtime `english_loaded()` cannot resolve synset/sense strings at all (its
//! fast-path stores drop the synset index). The baked value is the numeric
//! `ConceptId`, which VerbNet's store doc establishes is IDENTICAL across the
//! raw-XML and compact/store-bundle load paths — so it is valid at runtime
//! without re-resolution. This store therefore takes NO `LexicalReasoner` to
//! build (unlike FrameNet/ConceptNet, which resolve lemmas live).
//!
//! ## No SUMO taxonomy walk — flat same-term matching only
//!
//! Only the direct synset↔term crosswalk is loaded; SUMO's own class hierarchy
//! (`Merge.kif`) is NOT. So [`SumoStore::shares_sumo_class`] is flat same-term
//! intersection (like ConceptNet's "any shared assertion"), never an
//! ancestor-walk (VerbNet's nested-class model). Two concepts that map to
//! related-but-distinct SUMO terms are correctly reported as no-match — the
//! honest limit of loading the crosswalk without the upper ontology.

#[allow(unused_imports)]
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use super::ontology::{Sumo, SumoRelationKind};
use crate::cognitive::linguistics::english::ConceptId;

/// The loaded, indexed WordNet↔SUMO data — the corroboration mechanism's query
/// surface. Keyed by `ConceptId::value()`: each concept maps to the SUMO
/// `(term, relation)` pairs its synset was annotated with.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SumoStore {
    /// `ConceptId::value()` → the SUMO `(term, relation)` pairs the concept's
    /// synset carries (usually one, occasionally two — some synsets carry two
    /// annotations; kept as a `Vec`, not assumed unique).
    concept_to_terms: BTreeMap<u64, Vec<(String, SumoRelationKind)>>,
}

impl SumoStore {
    /// Build the indexed store from the typed, reader-produced [`Sumo`] data.
    /// The data is already resolved to concepts (see the module doc), so this
    /// is a pure re-index — no WordNet needed.
    #[must_use]
    pub fn from_sumo(sumo: &Sumo) -> Self {
        let mut concept_to_terms: BTreeMap<u64, Vec<(String, SumoRelationKind)>> = BTreeMap::new();
        for m in &sumo.mappings {
            concept_to_terms
                .entry(m.concept.value())
                .or_default()
                .push((m.term.clone(), m.relation));
        }
        Self { concept_to_terms }
    }

    /// Does `concept` have ANY SUMO mapping at all (positive OR complement)?
    /// The epistemic distinction [`SumoStore::shares_sumo_class`]'s `false`
    /// alone can't make — a concept SUMO never mapped ("no coverage") is a
    /// different fact than one it mapped but that shares no term with the other
    /// ("queried, no connection"). Mirrors
    /// [`crate::cognitive::linguistics::verbnet::store::VerbNetStore::has_coverage`]'s
    /// same rationale.
    #[must_use]
    pub fn has_coverage(&self, concept: ConceptId) -> bool {
        self.concept_to_terms.contains_key(&concept.value())
    }

    /// Does `a` share a SUMO class with `b` — i.e. do they map to at least one
    /// SUMO term in common where NEITHER occurrence is a `Complement*` relation?
    ///
    /// The complement filter is load-bearing, not incidental: a complement
    /// mapping (`:`/`[`/`]`) asserts the synset is explicitly NOT that SUMO
    /// class (the source's own legend — see [`super::ontology`]). Two synsets
    /// both being NOT-the-same-class is not a positive similarity signal, and
    /// counting it as one would be actively wrong — the same citation-grounded
    /// restraint the FrameNet store applies to frame Inheritance (a real
    /// relation that still earns no special trust). Flat same-term matching
    /// only; no SUMO taxonomy walk (the upper ontology is not loaded — see the
    /// module doc).
    #[must_use]
    pub fn shares_sumo_class(&self, a: ConceptId, b: ConceptId) -> bool {
        let (Some(terms_a), Some(terms_b)) = (
            self.concept_to_terms.get(&a.value()),
            self.concept_to_terms.get(&b.value()),
        ) else {
            return false;
        };
        terms_a.iter().any(|(term_a, rel_a)| {
            !rel_a.is_complement()
                && terms_b
                    .iter()
                    .any(|(term_b, rel_b)| !rel_b.is_complement() && term_a == term_b)
        })
    }
}

/// Decode + parse the committed `sumo` `.prx` into the typed, UNINDEXED
/// [`Sumo`] mapping list — the one place the embedded bytes and pinned
/// version live, shared by [`sumo_loaded`] (which indexes it into a
/// [`SumoStore`]) and [`sumo_mappings`] (which exposes it unindexed for a
/// consumer that needs to iterate every row directly, e.g.
/// [`super::sssom`]'s SSSOM mapping-set producer).
#[cfg(feature = "std")]
fn load_sumo() -> Sumo {
    use crate::applied::data_provisioning::decoders::plaintext_tsv;
    use crate::applied::data_provisioning::raw_source_prx::raw_source_bytes_embedded;

    const SUMO_PRX: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/sumo/sumo-wordnetmappings-30.prx"
    ));

    let bytes = raw_source_bytes_embedded("sumo", "2026.07.14", SUMO_PRX);
    let records = plaintext_tsv::decode(&bytes)
        .unwrap_or_else(|e| panic!("sumo committed .prx archive failed to decode: {e}"));
    super::reader::read_sumo(&records)
}

/// The process-wide loaded SUMO store — the committed `sumo` `.prx` decoded,
/// parsed, and indexed once. Mirrors
/// [`crate::cognitive::linguistics::framenet::store::framenet_loaded`]'s
/// caching shape: built lazily on first use, reused for the process lifetime.
#[cfg(feature = "std")]
pub fn sumo_loaded() -> &'static SumoStore {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<SumoStore> = OnceLock::new();
    INSTANCE.get_or_init(|| SumoStore::from_sumo(&load_sumo()))
}

/// The process-wide loaded, UNINDEXED SUMO mapping table — the same committed
/// data [`sumo_loaded`] indexes, exposed as the flat [`Sumo`] list for a
/// consumer that needs every `(concept, term, relation, oewn_synset_id)` row
/// directly rather than the flat same-term intersection [`SumoStore`] offers
/// (e.g. [`super::sssom`]'s SSSOM mapping-set producer, which needs to group
/// rows by concept to detect ambiguous EQ mappings).
#[cfg(feature = "std")]
pub fn sumo_mappings() -> &'static Sumo {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<Sumo> = OnceLock::new();
    INSTANCE.get_or_init(load_sumo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cognitive::linguistics::sumo::ontology::{Sumo, SumoMapping};

    fn mapping(concept: u64, term: &str, relation: SumoRelationKind) -> SumoMapping {
        SumoMapping {
            concept: ConceptId::new(concept),
            term: term.to_string(),
            relation,
            // Fixture synset id — not exercised by these store-indexing tests
            // (which key on `concept`/`term`/`relation` only), but a real-shaped
            // OEWN id string keeps the fixture honest rather than empty.
            oewn_synset_id: alloc::format!("oewn-{concept:08}-n"),
        }
    }

    fn fixture_sumo() -> Sumo {
        Sumo {
            mappings: alloc::vec![
                // concepts 1740 and 1930 both map to "Physical" (positive) —
                // a real shared SUMO class.
                mapping(1740, "Entity", SumoRelationKind::Equivalence),
                mapping(1740, "Physical", SumoRelationKind::Subsumption),
                mapping(1930, "Physical", SumoRelationKind::Subsumption),
                // concept 2137 is explicitly NOT Physical (complement) — must
                // not count as sharing "Physical" with 1740.
                mapping(2137, "Attribute", SumoRelationKind::Subsumption),
                mapping(2137, "Physical", SumoRelationKind::ComplementSubsumption),
                // concepts 3000 and 4000 both map to "Motion" but BOTH as
                // complements — shared term, neither positive: not a match.
                mapping(3000, "Motion", SumoRelationKind::ComplementSubsumption),
                mapping(4000, "Motion", SumoRelationKind::ComplementSubsumption),
            ],
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn shares_sumo_class_finds_a_positive_shared_term() {
        let store = SumoStore::from_sumo(&fixture_sumo());
        assert!(store.shares_sumo_class(ConceptId::new(1740), ConceptId::new(1930)));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_complement_mapping_on_one_side_is_not_a_shared_class() {
        // 1740 maps to "Physical" (positive); 2137 maps to "Physical" as a
        // COMPLEMENT (explicitly NOT Physical). They must not be reported as
        // sharing that class.
        let store = SumoStore::from_sumo(&fixture_sumo());
        assert!(!store.shares_sumo_class(ConceptId::new(1740), ConceptId::new(2137)));
        // Both DO have coverage — a real "queried, no connection", not "no data".
        assert!(store.has_coverage(ConceptId::new(1740)));
        assert!(store.has_coverage(ConceptId::new(2137)));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_term_shared_only_via_complements_on_both_sides_is_not_a_match() {
        let store = SumoStore::from_sumo(&fixture_sumo());
        assert!(!store.shares_sumo_class(ConceptId::new(3000), ConceptId::new(4000)));
        // A complement mapping still IS coverage — SUMO did map the synset.
        assert!(store.has_coverage(ConceptId::new(3000)));
        assert!(store.has_coverage(ConceptId::new(4000)));
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_concept_sumo_never_mapped_has_no_coverage() {
        let store = SumoStore::from_sumo(&fixture_sumo());
        assert!(store.has_coverage(ConceptId::new(1740)));
        assert!(!store.has_coverage(ConceptId::new(9999)));
    }
}
