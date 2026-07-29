//! Interpret the generic TSV record stream
//! ([`crate::applied::data_provisioning::decoders::plaintext_tsv`]'s decode
//! target) as the ConceptNet field shape: `relation<TAB>start_lemma<TAB>
//! end_lemma<TAB>weight`.
//!
//! Mirrors [`crate::cognitive::linguistics::verbnet::reader`]'s division of
//! labor: the generic decoder turns raw bytes into a structure-preserving
//! record stream; this module says what the FIELDS mean. Fail-closed
//! per-row, not per-file — one malformed row must not blank out the rest.

#[allow(unused_imports)]
use alloc::{string::ToString, vec::Vec};

use super::ontology::{ConceptNet, ConceptNetEdge};
use crate::applied::data_provisioning::decoders::plaintext_tsv::TsvRecords;

/// Interpret a decoded TSV record stream as [`ConceptNet`] edges. A record
/// that doesn't have exactly 4 fields, or whose `weight` field doesn't parse
/// as a finite `f32`, is skipped rather than causing the whole load to fail —
/// the same discipline
/// `crate::cognitive::linguistics::verbnet::store::parse_crosswalk_tsv`
/// applies to its own TSV.
#[must_use]
pub fn read_conceptnet(records: &TsvRecords) -> ConceptNet {
    let mut edges = Vec::new();
    for record in records {
        let [relation, start_lemma, end_lemma, weight] = record.as_slice() else {
            continue;
        };
        let Ok(weight) = weight.parse::<f32>() else {
            continue;
        };
        if !weight.is_finite() {
            continue;
        }
        edges.push(ConceptNetEdge {
            relation: relation.clone(),
            start_lemma: start_lemma.clone(),
            end_lemma: end_lemma.clone(),
            weight,
        });
    }
    ConceptNet { edges }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reads_well_formed_rows() {
        let records: TsvRecords = alloc::vec![
            alloc::vec![
                "IsA".to_string(),
                "cut".to_string(),
                "action".to_string(),
                "2.0".to_string()
            ],
            alloc::vec![
                "RelatedTo".to_string(),
                "cut".to_string(),
                "end".to_string(),
                "1.0".to_string()
            ],
        ];
        let cn = read_conceptnet(&records);
        assert_eq!(cn.edges.len(), 2);
        assert_eq!(cn.edges[0].relation, "IsA");
        assert_eq!(cn.edges[0].start_lemma, "cut");
        assert_eq!(cn.edges[0].end_lemma, "action");
        assert_eq!(cn.edges[0].weight, 2.0);
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn skips_malformed_rows_without_panicking() {
        let records: TsvRecords = alloc::vec![
            alloc::vec!["IsA".to_string(), "cut".to_string()], // too few fields
            alloc::vec![
                "IsA".to_string(),
                "cut".to_string(),
                "action".to_string(),
                "not-a-number".to_string()
            ],
            alloc::vec![
                "RelatedTo".to_string(),
                "cut".to_string(),
                "end".to_string(),
                "1.0".to_string()
            ],
        ];
        let cn = read_conceptnet(&records);
        assert_eq!(cn.edges.len(), 1);
        assert_eq!(cn.edges[0].relation, "RelatedTo");
    }
}
