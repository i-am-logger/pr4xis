//! Tests for the XSD built-in datatype hierarchy ontology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    BaseTypeAgreesWithCategory, DatatypeLatticeSingleRoot, ListDatatypesHaveBuiltInItemType,
    PrimitivesDeriveFromAnyAtomicType, Variety, VarietyOf, Xsd11DatatypeAdditionsPresent,
    XsdDatatypeCategory, XsdDatatypeConcept, XsdDatatypeOntology, base_type, derived_datatypes,
    is_derived, is_primitive, is_special, item_type, list_datatypes, primitive_datatypes,
    special_datatypes,
};
use pr4xis::category::Concept;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category + ontology validation
// =============================================================================

#[test]
fn datatype_category_laws() {
    assert_category_laws::<XsdDatatypeCategory>();
}

#[test]
fn datatype_ontology_validates() {
    XsdDatatypeOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

// =============================================================================
// Concept counts — the closed Part 2 inventory.
// =============================================================================

#[test]
fn concept_count() {
    // 3 special (§3.2) + 19 primitive (§3.3) + 28 derived (§3.4) = 50.
    assert_eq!(XsdDatatypeConcept::variants().len(), 50);
}

#[test]
fn group_counts() {
    assert_eq!(special_datatypes().len(), 3);
    assert_eq!(primitive_datatypes().len(), 19);
    assert_eq!(derived_datatypes().len(), 28);
    assert_eq!(list_datatypes().len(), 3);
}

#[test]
fn groups_partition_the_inventory() {
    // Every concept is in exactly one of special / primitive / derived.
    for d in XsdDatatypeConcept::variants() {
        let count = [is_special(d), is_primitive(d), is_derived(d)]
            .into_iter()
            .filter(|b| *b)
            .count();
        assert_eq!(count, 1, "{d:?} should be in exactly one group");
    }
    // And the three groups together cover all 50.
    let total = special_datatypes().len() + primitive_datatypes().len() + derived_datatypes().len();
    assert_eq!(total, XsdDatatypeConcept::variants().len());
}

// =============================================================================
// base_type — the {base type definition} relation.
// =============================================================================

#[test]
fn any_type_is_the_unique_root() {
    let roots: Vec<_> = XsdDatatypeConcept::variants()
        .into_iter()
        .filter(|d| base_type(*d).is_none())
        .collect();
    assert_eq!(roots, vec![XsdDatatypeConcept::AnyType]);
}

#[test]
fn base_type_is_total_off_root() {
    for d in XsdDatatypeConcept::variants() {
        if d == XsdDatatypeConcept::AnyType {
            continue;
        }
        assert!(base_type(d).is_some(), "{d:?} should have a base type");
    }
}

#[test]
fn known_derivation_chains() {
    use XsdDatatypeConcept as D;
    // §3.4.17 int ⊏ long ⊏ integer ⊏ decimal ⊏ anyAtomicType.
    assert_eq!(base_type(D::Int), Some(D::Long));
    assert_eq!(base_type(D::Long), Some(D::Integer));
    assert_eq!(base_type(D::Integer), Some(D::Decimal));
    assert_eq!(base_type(D::Decimal), Some(D::AnyAtomicType));
    // §3.4.8 ID ⊏ NCName ⊏ Name ⊏ token ⊏ normalizedString ⊏ string.
    assert_eq!(base_type(D::Id), Some(D::NcName));
    assert_eq!(base_type(D::NcName), Some(D::Name));
    assert_eq!(base_type(D::Name), Some(D::Token));
    assert_eq!(base_type(D::Token), Some(D::NormalizedString));
    assert_eq!(base_type(D::NormalizedString), Some(D::StringType));
    // §3.2 special chain.
    assert_eq!(base_type(D::AnyAtomicType), Some(D::AnySimpleType));
    assert_eq!(base_type(D::AnySimpleType), Some(D::AnyType));
}

#[test]
fn list_datatypes_base_on_any_simple_type() {
    use XsdDatatypeConcept as D;
    // §3.4.5 / §3.4.10 / §3.4.12 — list types base on anySimpleType.
    assert_eq!(base_type(D::NmTokens), Some(D::AnySimpleType));
    assert_eq!(base_type(D::IdRefs), Some(D::AnySimpleType));
    assert_eq!(base_type(D::Entities), Some(D::AnySimpleType));
}

// =============================================================================
// item_type — the list {item type definition} relation.
// =============================================================================

#[test]
fn list_item_types() {
    use XsdDatatypeConcept as D;
    assert_eq!(item_type(D::NmTokens), Some(D::NmToken));
    assert_eq!(item_type(D::IdRefs), Some(D::IdRef));
    assert_eq!(item_type(D::Entities), Some(D::Entity));
}

#[test]
fn only_list_datatypes_have_item_type() {
    let lists = list_datatypes();
    for d in XsdDatatypeConcept::variants() {
        assert_eq!(
            item_type(d).is_some(),
            lists.contains(&d),
            "{d:?}: item_type presence should match list membership"
        );
    }
}

// =============================================================================
// Variety quality.
// =============================================================================

#[test]
fn variety_of_lists_is_list() {
    for d in list_datatypes() {
        assert_eq!(VarietyOf.get(&d), Some(Variety::List));
    }
}

#[test]
fn variety_of_primitives_is_atomic() {
    for d in primitive_datatypes() {
        assert_eq!(VarietyOf.get(&d), Some(Variety::Atomic));
    }
}

#[test]
fn variety_absent_for_any_type_and_any_simple_type() {
    assert_eq!(VarietyOf.get(&XsdDatatypeConcept::AnyType), None);
    assert_eq!(VarietyOf.get(&XsdDatatypeConcept::AnySimpleType), None);
}

#[test]
fn any_atomic_type_is_atomic() {
    assert_eq!(
        VarietyOf.get(&XsdDatatypeConcept::AnyAtomicType),
        Some(Variety::Atomic)
    );
}

// =============================================================================
// Axiom tests.
// =============================================================================

#[test]
fn axiom_single_root_holds() {
    assert!(DatatypeLatticeSingleRoot.verify().is_ok());
}

#[test]
fn axiom_base_type_agrees_with_category() {
    assert!(BaseTypeAgreesWithCategory.verify().is_ok());
}

#[test]
fn axiom_primitives_derive_from_any_atomic_type() {
    assert!(PrimitivesDeriveFromAnyAtomicType.verify().is_ok());
}

#[test]
fn axiom_list_datatypes_have_built_in_item_type() {
    assert!(ListDatatypesHaveBuiltInItemType.verify().is_ok());
}

#[test]
fn axiom_xsd11_additions_present() {
    assert!(Xsd11DatatypeAdditionsPresent.verify().is_ok());
}

// =============================================================================
// Property-based tests.
// =============================================================================

fn arb_datatype() -> impl Strategy<Value = XsdDatatypeConcept> {
    proptest::sample::select(XsdDatatypeConcept::variants())
}

proptest! {
    /// Following `base_type` from any datatype terminates at `anyType`
    /// in at most |variants| steps — the lattice is finite and
    /// acyclic (Peterson et al. 2012 §3.4: the hierarchy is a tree
    /// rooted at anyType).
    #[test]
    fn prop_base_type_reaches_root(d in arb_datatype()) {
        let mut cur = d;
        let mut steps = 0usize;
        while let Some(parent) = base_type(cur) {
            cur = parent;
            steps += 1;
            prop_assert!(steps <= XsdDatatypeConcept::variants().len());
        }
        prop_assert_eq!(cur, XsdDatatypeConcept::AnyType);
    }

    /// `base_type` is deterministic.
    #[test]
    fn prop_base_type_deterministic(d in arb_datatype()) {
        prop_assert_eq!(base_type(d), base_type(d));
    }

    /// Every datatype is classified into exactly one of the three
    /// groups (special / primitive / derived).
    #[test]
    fn prop_exactly_one_group(d in arb_datatype()) {
        let count = [is_special(d), is_primitive(d), is_derived(d)]
            .into_iter()
            .filter(|b| *b)
            .count();
        prop_assert_eq!(count, 1);
    }

    /// A datatype has an item type iff it is a list datatype, and the
    /// item type is atomic (§2.4.1.2).
    #[test]
    fn prop_item_type_iff_list(d in arb_datatype()) {
        match item_type(d) {
            Some(item) => {
                prop_assert!(list_datatypes().contains(&d));
                prop_assert_eq!(VarietyOf.get(&item), Some(Variety::Atomic));
            }
            None => prop_assert!(!list_datatypes().contains(&d)),
        }
    }
}
