//! Interpret the generic TSV record stream
//! ([`crate::applied::data_provisioning::decoders::plaintext_tsv`]'s decode
//! target) as FrameNet's field shape: a type-tagged row, either
//! `LU<TAB>lemma<TAB>pos_tag<TAB>frame` or
//! `REL<TAB>relation<TAB>sub_frame<TAB>super_frame`.
//!
//! Mirrors [`crate::cognitive::linguistics::conceptnet::reader`]'s division
//! of labor: the generic decoder turns raw bytes into a structure-preserving
//! record stream; this module says what the FIELDS mean. Fail-closed
//! per-row, not per-file.

#[allow(unused_imports)]
use alloc::{string::ToString, vec::Vec};

use super::ontology::{FrameNet, FrameNetLexicalUnit, FrameNetRelation};
use crate::applied::data_provisioning::decoders::plaintext_tsv::TsvRecords;
use crate::social::software::markup::xml::lmf::LmfPos;

/// Interpret a decoded TSV record stream as [`FrameNet`] data. A record
/// with an unrecognized type tag, wrong field count, or unparseable `pos`
/// tag is skipped rather than causing the whole load to fail — the same
/// discipline every TSV reader in this codebase applies.
#[must_use]
pub fn read_framenet(records: &TsvRecords) -> FrameNet {
    let mut lexical_units = Vec::new();
    let mut relations = Vec::new();
    for record in records {
        match record.as_slice() {
            [tag, lemma, pos_tag, frame] if tag == "LU" => {
                let pos = LmfPos::parse(pos_tag);
                if !pos.is_open_class() {
                    continue;
                }
                lexical_units.push(FrameNetLexicalUnit {
                    lemma: lemma.clone(),
                    pos,
                    frame: frame.clone(),
                });
            }
            [tag, relation, sub_frame, super_frame] if tag == "REL" => {
                relations.push(FrameNetRelation {
                    relation: relation.clone(),
                    sub_frame: sub_frame.clone(),
                    super_frame: super_frame.clone(),
                });
            }
            _ => continue,
        }
    }
    FrameNet {
        lexical_units,
        relations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn reads_well_formed_rows() {
        let records: TsvRecords = alloc::vec![
            alloc::vec![
                "LU".to_string(),
                "cause".to_string(),
                "v".to_string(),
                "Causation".to_string()
            ],
            alloc::vec![
                "REL".to_string(),
                "Causative_of".to_string(),
                "Inchoative_state".to_string(),
                "Causative".to_string()
            ],
        ];
        let fn_data = read_framenet(&records);
        assert_eq!(fn_data.lexical_units.len(), 1);
        assert_eq!(fn_data.lexical_units[0].lemma, "cause");
        assert_eq!(fn_data.lexical_units[0].pos, LmfPos::Verb);
        assert_eq!(fn_data.lexical_units[0].frame, "Causation");
        assert_eq!(fn_data.relations.len(), 1);
        assert_eq!(fn_data.relations[0].relation, "Causative_of");
    }

    #[pr4xis::praxis_value(Honest)]
    #[test]
    fn skips_malformed_rows_without_panicking() {
        let records: TsvRecords = alloc::vec![
            alloc::vec!["LU".to_string(), "cause".to_string()], // too few fields
            alloc::vec![
                "LU".to_string(),
                "of".to_string(),
                "prep".to_string(), // closed-class POS
                "Somewhere".to_string()
            ],
            alloc::vec![
                "LU".to_string(),
                "run".to_string(),
                "not-a-pos".to_string(),
                "Motion".to_string()
            ],
            alloc::vec![
                "UNKNOWN".to_string(),
                "x".to_string(),
                "y".to_string(),
                "z".to_string()
            ],
            alloc::vec![
                "LU".to_string(),
                "run".to_string(),
                "v".to_string(),
                "Motion".to_string()
            ],
        ];
        let fn_data = read_framenet(&records);
        assert_eq!(fn_data.lexical_units.len(), 1);
        assert_eq!(fn_data.lexical_units[0].lemma, "run");
        assert_eq!(fn_data.relations.len(), 0);
    }
}
