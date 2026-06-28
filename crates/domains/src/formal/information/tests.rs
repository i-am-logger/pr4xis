use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};

use super::ontology::*;

// =============================================================================
// Category tests
// =============================================================================

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn info_category_laws() {
    assert_category_laws::<InfoCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn info_has_8_units() {
    assert_eq!(InfoConcept::variants().len(), 8);
}

// =============================================================================
// Mereological relationships (has-a → Parthood kind)
// =============================================================================

/// CONFORMANCE: the `ontology!` macro's `has_a:` sugar emits a Parthood edge
/// PART→WHOLE (`part_of`, BFO:0000050) — the part is the source, the whole the
/// target — so `reaches(part, whole, Parthood)` holds, matching the USC corpus
/// bridge and the "is X part of Y" chat query. The OLD whole→part orientation
/// (which answered "is X part of Y" backwards) must NOT be present.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn has_a_desugars_to_a_part_to_whole_parthood_edge() {
    let m = InfoCategory::morphisms();
    // Byte has_a Bit → a Bit→Byte (part→whole) Parthood edge exists …
    assert!(
        m.iter().any(|r| r.source() == InfoConcept::Bit
            && r.target() == InfoConcept::Byte
            && r.kind() == InfoRelationKind::Parthood),
        "has_a must emit PART→WHOLE (Bit→Byte) Parthood"
    );
    // … and the inverse whole→part (Byte→Bit) Parthood edge must NOT exist.
    assert!(
        !m.iter().any(|r| r.source() == InfoConcept::Byte
            && r.target() == InfoConcept::Bit
            && r.kind() == InfoRelationKind::Parthood),
        "the OLD whole→part Parthood orientation must be gone"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn byte_composed_of_bits() {
    // part→whole (BFO:0000050): a Bit is PART of a Byte (source=part, target=whole).
    let m = InfoCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == InfoConcept::Bit
        && r.target() == InfoConcept::Byte
        && r.kind() == InfoRelationKind::Parthood));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn word_composed_of_bytes() {
    // part→whole: a Byte is PART of a Word.
    let m = InfoCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == InfoConcept::Byte
        && r.target() == InfoConcept::Word
        && r.kind() == InfoRelationKind::Parthood));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn word_transitively_composed_of_bits() {
    // Same-kind transitive closure (OBO-RO `transitive_over`): part→whole, so a
    // Bit is transitively PART of a Word (Bit→Byte→Word).
    let m = InfoCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.source() == InfoConcept::Bit && r.target() == InfoConcept::Word)
    );
}

// =============================================================================
// Taxonomic relationships (is-a → Subsumption kind)
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn reference_is_a_word() {
    let m = InfoCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == InfoConcept::Reference
        && r.target() == InfoConcept::Word
        && r.kind() == InfoRelationKind::Subsumption));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn text_is_a_sequence() {
    let m = InfoCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == InfoConcept::Text
        && r.target() == InfoConcept::Sequence
        && r.kind() == InfoRelationKind::Subsumption));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn truth_value_equivalent_to_bit() {
    // Shannon (1948).
    let m = InfoCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == InfoConcept::TruthValue
        && r.target() == InfoConcept::Bit
        && r.kind() == InfoRelationKind::Equivalence));
}

// =============================================================================
// Reference tests
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ref32_size() {
    let r: Ref32 = Ref::new(42);
    assert_eq!(r.size_bytes(), 4);
    assert_eq!(r.value(), 42);
    assert_eq!(r.max_addressable(), (1u64 << 32) - 1);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ref64_size() {
    let r: Ref64 = Ref::new(999);
    assert_eq!(r.size_bytes(), 8);
    assert_eq!(r.max_addressable(), u64::MAX);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ref32_sufficient_for_wordnet() {
    let r: Ref32 = Ref::new(0);
    // WordNet has ~107k synsets; Ref32 can address ~4 billion.
    assert!(r.max_addressable() > 107_519);
}

// =============================================================================
// Classification tests
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn atomics() {
    assert!(InfoConcept::Bit.is_atomic());
    assert!(InfoConcept::TruthValue.is_atomic());
    assert!(!InfoConcept::Byte.is_atomic());
    assert!(!InfoConcept::Word.is_atomic());
    assert!(!InfoConcept::Reference.is_atomic());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn structured() {
    assert!(InfoConcept::Byte.is_structured());
    assert!(InfoConcept::Word.is_structured());
    assert!(InfoConcept::Reference.is_structured());
    assert!(InfoConcept::Text.is_structured());
    assert!(!InfoConcept::Bit.is_structured());
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_info_unit() -> impl Strategy<Value = InfoConcept> {
        proptest::sample::select(InfoConcept::variants())
    }

    proptest! {
        /// Every unit is either primitive or composite (exhaustive partition).
        #[test]
        fn prop_atomic_or_structured(unit in arb_info_unit()) {
            prop_assert!(unit.is_atomic() != unit.is_structured());
        }

        /// Identity is idempotent.
        #[test]
        fn prop_identity_idempotent(unit in arb_info_unit()) {
            let id = InfoCategory::identity(&unit);
            let composed = InfoCategory::compose(&id, &id);
            prop_assert_eq!(composed, Some(id));
        }

        /// Reference can address more than any known lexical database.
        #[test]
        fn prop_ref32_sufficient(id in 0..1_000_000u64) {
            let r: Ref32 = Ref::new(id);
            prop_assert!(r.value() == id);
            prop_assert!(r.max_addressable() > id);
        }
    }

    pr4xis::register_praxis_value!(prop_atomic_or_structured, Verifiable);
    pr4xis::register_praxis_value!(prop_identity_idempotent, Deterministic);
    pr4xis::register_praxis_value!(prop_ref32_sufficient, Verifiable);
}
