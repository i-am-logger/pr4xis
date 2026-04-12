use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;

use super::ontology::*;

// =============================================================================
// Category tests
// =============================================================================

#[test]
fn info_category_laws() {
    check_category_laws::<InfoCategory>().unwrap();
}

#[test]
fn info_has_8_units() {
    assert_eq!(InfoUnit::variants().len(), 8);
}

// =============================================================================
// Mereological relationships (has-a)
// =============================================================================

#[test]
fn byte_composed_of_bits() {
    let m = InfoCategory::morphisms();
    assert!(m.iter().any(|r| r.from == InfoUnit::Byte
        && r.to == InfoUnit::Bit
        && r.kind == InfoRelationKind::ComposedOf));
}

#[test]
fn word_composed_of_bytes() {
    let m = InfoCategory::morphisms();
    assert!(m.iter().any(|r| r.from == InfoUnit::Word
        && r.to == InfoUnit::Byte
        && r.kind == InfoRelationKind::ComposedOf));
}

#[test]
fn word_transitively_composed_of_bits() {
    let m = InfoCategory::morphisms();
    assert!(
        m.iter()
            .any(|r| r.from == InfoUnit::Word && r.to == InfoUnit::Bit)
    );
}

// =============================================================================
// Taxonomic relationships (is-a)
// =============================================================================

#[test]
fn reference_is_a_word() {
    let m = InfoCategory::morphisms();
    assert!(m.iter().any(|r| r.from == InfoUnit::Reference
        && r.to == InfoUnit::Word
        && r.kind == InfoRelationKind::IsA));
}

#[test]
fn text_is_a_sequence() {
    let m = InfoCategory::morphisms();
    assert!(m.iter().any(|r| r.from == InfoUnit::Text
        && r.to == InfoUnit::Sequence
        && r.kind == InfoRelationKind::IsA));
}

#[test]
fn truth_value_equivalent_to_bit() {
    let m = InfoCategory::morphisms();
    assert!(m.iter().any(|r| r.from == InfoUnit::TruthValue
        && r.to == InfoUnit::Bit
        && r.kind == InfoRelationKind::Equivalent));
}

// =============================================================================
// Reference tests
// =============================================================================

#[test]
fn ref32_size() {
    let r: Ref32 = Reference::new(42);
    assert_eq!(r.size_bytes(), 4);
    assert_eq!(r.value(), 42);
    assert_eq!(r.max_addressable(), (1u64 << 32) - 1); // ~4 billion
}

#[test]
fn ref64_size() {
    let r: Ref64 = Reference::new(999);
    assert_eq!(r.size_bytes(), 8);
    assert_eq!(r.max_addressable(), u64::MAX);
}

#[test]
fn ref32_sufficient_for_wordnet() {
    let r: Ref32 = Reference::new(0);
    // WordNet has ~107k synsets. Ref32 can address ~4 billion.
    assert!(r.max_addressable() > 107_519);
}

// =============================================================================
// Classification tests
// =============================================================================

#[test]
fn atomics() {
    assert!(InfoUnit::Bit.is_atomic());
    assert!(InfoUnit::TruthValue.is_atomic());
    assert!(!InfoUnit::Byte.is_atomic());
    assert!(!InfoUnit::Word.is_atomic());
    assert!(!InfoUnit::Reference.is_atomic());
}

#[test]
fn structured() {
    assert!(InfoUnit::Byte.is_structured());
    assert!(InfoUnit::Word.is_structured());
    assert!(InfoUnit::Reference.is_structured());
    assert!(InfoUnit::Text.is_structured());
    assert!(!InfoUnit::Bit.is_structured());
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_info_unit() -> impl Strategy<Value = InfoUnit> {
        prop_oneof![
            Just(InfoUnit::Bit),
            Just(InfoUnit::Byte),
            Just(InfoUnit::Word),
            Just(InfoUnit::Reference),
            Just(InfoUnit::Sequence),
            Just(InfoUnit::Text),
            Just(InfoUnit::TruthValue),
            Just(InfoUnit::Quantity),
        ]
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
            let r: Ref32 = Reference::new(id);
            prop_assert!(r.value() == id);
            prop_assert!(r.max_addressable() > id);
        }

        /// Composite units have at least one ComposedOf or IsA morphism.
        #[test]
        fn prop_structured_have_relations(unit in arb_info_unit()) {
            if unit.is_structured() {
                let morphisms = InfoCategory::morphisms();
                let has_outgoing = morphisms.iter().any(|m|
                    m.from == unit && m.kind != InfoRelationKind::Identity);
                prop_assert!(has_outgoing,
                    "{:?} is composite but has no non-identity outgoing morphisms", unit);
            }
        }
    }
}
