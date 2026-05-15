//! Smoke tests for the auto-generated SOX 1514A ontology.
//!
//! Validates the build.rs → praxis.lock → codegen pipeline end-to-end:
//! the generated `Sox1514aId` enum and `CODEGEN_DATA` static must
//! reflect the 28-term / 18-relation structure declared in
//! `praxis.lock`'s `[structural."sox_1514a@2002"]` block.

use super::{CODEGEN_DATA, Sox1514aId};
use pr4xis::category::Concept;

#[test]
fn twenty_eight_concepts() {
    assert_eq!(Sox1514aId::variants().len(), 28);
}

#[test]
fn codegen_data_entity_count_matches() {
    assert_eq!(CODEGEN_DATA.entity_count, 28);
    assert_eq!(CODEGEN_DATA.entity_labels.len(), 28);
}

#[test]
fn first_concept_is_covered_employer() {
    // Sources are sorted in praxis.lock declaration order; the first
    // term in SOX 1514A's structural block is "Covered Employer".
    assert_eq!(CODEGEN_DATA.entity_labels[0], "Covered Employer");
}

#[test]
fn relations_total_eighteen_across_kinds() {
    // 18 relations distributed across taxonomy / mereology / opposition /
    // equivalence / causation per the codegen's lossy mapping.
    let total = CODEGEN_DATA.taxonomy.len()
        + CODEGEN_DATA.mereology.len()
        + CODEGEN_DATA.opposition.len()
        + CODEGEN_DATA.equivalence.len()
        + CODEGEN_DATA.causation.len();
    assert_eq!(total, 18);
}

#[test]
fn empty_word_index_until_adjunction_codegen() {
    // SOX 1514A's structural block carries no lemmas (lemmas extraction
    // is M5 adjunction-to-English work). WORD_INDEX must therefore be
    // empty, but the symbol must still exist — the codegen emits it
    // unconditionally per the M3c WORD_INDEX-always-defined fix.
    assert_eq!(CODEGEN_DATA.word_index.len(), 0);
}

#[test]
fn lookup_returns_empty_for_any_word_pre_m5() {
    use super::lookup;
    // Before adjunction codegen, all lookups miss.
    assert_eq!(lookup("retaliation"), &[] as &[u32]);
    assert_eq!(lookup("anything"), &[] as &[u32]);
}
