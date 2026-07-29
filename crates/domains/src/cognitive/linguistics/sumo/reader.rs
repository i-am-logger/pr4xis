//! Interpret the generic TSV record stream
//! ([`crate::applied::data_provisioning::decoders::plaintext_tsv`]'s decode
//! target) as SUMO's RESOLVED field shape:
//! `concept_value<TAB>term<TAB>relation_code<TAB>oewn_synset_id`.
//!
//! Mirrors [`crate::cognitive::linguistics::framenet::reader`]'s division of
//! labor: the generic decoder turns raw bytes into a structure-preserving
//! record stream; this module says what the FIELDS mean. Fail-closed per-row,
//! not per-file. The `concept_value` column is the numeric [`ConceptId`] the
//! synset→concept resolution (`super::regenerate`) already resolved offline —
//! no WordNet lookup happens here (see [`super::ontology`]). The
//! `oewn_synset_id` column is the real, external OEWN synset id that concept
//! resolves to — carried through unparsed (a plain string), consumed by
//! [`super::sssom`] for the SSSOM `subject_id`.

#[allow(unused_imports)]
use alloc::{string::ToString, vec::Vec};

use super::ontology::{Sumo, SumoMapping, SumoRelationKind};
use crate::applied::data_provisioning::decoders::plaintext_tsv::TsvRecords;
use crate::cognitive::linguistics::english::ConceptId;

/// Interpret a decoded TSV record stream as [`Sumo`] mapping data. A record
/// with the wrong field count, a non-numeric concept value, or an unrecognized
/// relation code is skipped rather than causing the whole load to fail — the
/// same discipline every TSV reader in this codebase applies.
#[must_use]
pub fn read_sumo(records: &TsvRecords) -> Sumo {
    let mut mappings = Vec::new();
    for record in records {
        let [concept_value, term, relation_code, oewn_synset_id] = record.as_slice() else {
            continue;
        };
        let Ok(value) = concept_value.parse::<u64>() else {
            continue;
        };
        let Some(relation) = SumoRelationKind::from_code(relation_code) else {
            continue;
        };
        mappings.push(SumoMapping {
            concept: ConceptId::new(value),
            term: term.clone(),
            relation,
            oewn_synset_id: oewn_synset_id.clone(),
        });
    }
    Sumo { mappings }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reads_well_formed_rows() {
        let records: TsvRecords = alloc::vec![
            alloc::vec![
                "1740".to_string(),
                "Entity".to_string(),
                "EQ".to_string(),
                "oewn-00001740-n".to_string(),
            ],
            alloc::vec![
                "219738".to_string(),
                "Shooting".to_string(),
                "SUB".to_string(),
                "oewn-00219738-n".to_string(),
            ],
            alloc::vec![
                "219738".to_string(),
                "Murder".to_string(),
                "SUB".to_string(),
                "oewn-00219738-n".to_string(),
            ],
        ];
        let sumo = read_sumo(&records);
        assert_eq!(sumo.mappings.len(), 3);
        assert_eq!(sumo.mappings[0].concept, ConceptId::new(1740));
        assert_eq!(sumo.mappings[0].term, "Entity");
        assert_eq!(sumo.mappings[0].relation, SumoRelationKind::Equivalence);
        assert_eq!(sumo.mappings[0].oewn_synset_id, "oewn-00001740-n");
        assert_eq!(sumo.mappings[2].concept, ConceptId::new(219738));
        assert_eq!(sumo.mappings[2].term, "Murder");
        assert_eq!(sumo.mappings[2].relation, SumoRelationKind::Subsumption);
        assert_eq!(sumo.mappings[2].oewn_synset_id, "oewn-00219738-n");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn skips_malformed_rows_without_panicking() {
        let records: TsvRecords = alloc::vec![
            alloc::vec!["1740".to_string(), "Entity".to_string()], // too few fields
            alloc::vec![
                "not-a-number".to_string(),
                "Entity".to_string(),
                "EQ".to_string(),
                "oewn-00001740-n".to_string(),
            ],
            alloc::vec![
                "1740".to_string(),
                "Entity".to_string(),
                "BOGUS".to_string(), // unrecognized relation code
                "oewn-00001740-n".to_string(),
            ],
            alloc::vec![
                "1930".to_string(),
                "Physical".to_string(),
                "EQ".to_string(),
                "oewn-00001930-n".to_string(),
            ],
        ];
        let sumo = read_sumo(&records);
        assert_eq!(sumo.mappings.len(), 1);
        assert_eq!(sumo.mappings[0].term, "Physical");
    }

    /// A pre-migration 3-column row (`concept_value<TAB>term<TAB>relation_code`,
    /// no `oewn_synset_id`) must be rejected fail-closed, not silently accepted —
    /// the migration to the 4-column shape (`regenerate.rs`) must not leave a
    /// stale 3-column row readable as if its 4th field were simply absent.
    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn a_pre_migration_three_column_row_is_rejected_fail_closed() {
        let records: TsvRecords = alloc::vec![alloc::vec![
            "1740".to_string(),
            "Entity".to_string(),
            "EQ".to_string()
        ],];
        let sumo = read_sumo(&records);
        assert!(
            sumo.mappings.is_empty(),
            "a 3-column (pre-migration) row must be skipped, not read"
        );
    }
}
