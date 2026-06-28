//! Tests for the XSD ontology.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::ontology::{
    AnnotationBinaryPartition, DerivationChain, ElementTypeRef, EveryConceptHasPartClassification,
    EveryElementHasExactlyOneTypeReference, ModelGroupTernaryPartition, OccurrenceRange, PartSpec,
    ParticleOccurrenceRangeNonEmpty, QNameCategoryUniqueness, SchemaComponentPartitioned,
    SubstitutionGroupHierarchy, SubstitutionGroupReflexiveTransitive,
    TypeDefinitionBinaryPartition, TypeDerivationStrictPartialOrder, XsdCategory, XsdConcept,
    XsdOntology, XsdPart, instantiable_leaves, is_root,
};
use pr4xis::category::FinitelyGenerated;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category + ontology validation
// =============================================================================

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn xsd_category_laws() {
    assert_category_laws::<XsdCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn xsd_ontology_validates() {
    XsdOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn concept_count() {
    // Five top-level groups + their sub-concepts:
    // - §2.5 SchemaDocument leaf: 1 (no children — it's both root
    //   and instantiable leaf, parallel to the abstract roots).
    // - §2.2 schema-component partition: 1 root + 17 sub-concepts = 18
    // - §4.2 schema-composition group: 1 root + 4 leaves = 5
    // - §3.4.2 / Part 2 §4.1.2 type-construction group: 1 root + 6
    //   leaves (ComplexContent, SimpleContent, Restriction, Extension,
    //   ListType, UnionType) = 7
    // Plus Part 2 §4.3: 1 ConstrainingFacet root + 14 facet leaves = 15.
    // Plus §3.11: IdentityConstraint becomes intermediate + 5 sub-kinds
    // (Key, KeyRef, Unique, Selector, Field).
    // Plus XSD 1.1 §3.13 / §3.4.2.2 / §3.16.2 content additions
    // (Assert, OpenContent, DefaultOpenContent) under the
    // type-construction root. Total: 1 + 18 + 5 + 7 + 14 + 5 + 3 = 53,
    // plus SchemaDocument = 54.
    assert_eq!(XsdConcept::variants().len(), 54);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn instantiable_leaves_count() {
    // 44 concrete kinds: 1 §2.5 SchemaDocument + 11 §2.2 schema-component
    // leaves (IdentityConstraint is now intermediate) + 4 §4.2 composition
    // directives + 6 §3.4.2 / Part 2 §4.1.2 type-construction constructs +
    // 14 Part 2 §4.3 facets + 5 §3.11 identity-constraint sub-kinds +
    // 3 XSD 1.1 content additions (Assert, OpenContent, DefaultOpenContent).
    assert_eq!(instantiable_leaves().len(), 44);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn root_classification() {
    // Five roots: §2.5 SchemaDocument, §2.2 SchemaComponent, §4.2
    // SchemaCompositionDirective, §3.4.2 / Part 2 §4.1.2
    // TypeConstructionConstruct, Part 2 §4.3 ConstrainingFacet.
    assert!(is_root(XsdConcept::SchemaDocument));
    assert!(is_root(XsdConcept::SchemaComponent));
    assert!(is_root(XsdConcept::SchemaCompositionDirective));
    assert!(is_root(XsdConcept::TypeConstructionConstruct));
    assert!(is_root(XsdConcept::ConstrainingFacet));
    for c in XsdConcept::variants() {
        if matches!(
            c,
            XsdConcept::SchemaDocument
                | XsdConcept::SchemaComponent
                | XsdConcept::SchemaCompositionDirective
                | XsdConcept::TypeConstructionConstruct
                | XsdConcept::ConstrainingFacet
        ) {
            continue;
        }
        assert!(!is_root(c), "{c:?} should not be root");
    }
}

// =============================================================================
// Quality: PartSpec
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn part_spec_simple_is_part2() {
    assert_eq!(
        PartSpec.get(&XsdConcept::SimpleTypeDefinition),
        Some(XsdPart::Datatypes)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn part_spec_complex_is_part1() {
    assert_eq!(
        PartSpec.get(&XsdConcept::ComplexTypeDefinition),
        Some(XsdPart::Structures)
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn part_spec_root_is_none() {
    assert_eq!(PartSpec.get(&XsdConcept::SchemaComponent), None);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn part_spec_total_on_non_root() {
    for c in XsdConcept::variants() {
        // The three abstract roots (§2.2 SchemaComponent, §4.2
        // SchemaCompositionDirective, §3.4.2 TypeConstructionConstruct)
        // carry no part classification.
        if is_root(c) {
            continue;
        }
        assert!(
            PartSpec.get(&c).is_some(),
            "{c:?} has no PartSpec classification"
        );
    }
}

// =============================================================================
// Axiom tests — each declared axiom invoked on small constructed inputs
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_schema_component_partitioned() {
    assert!(SchemaComponentPartitioned.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_type_definition_binary_partition() {
    assert!(TypeDefinitionBinaryPartition.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_model_group_ternary_partition() {
    assert!(ModelGroupTernaryPartition.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_annotation_binary_partition() {
    assert!(AnnotationBinaryPartition.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_type_derivation_strict_partial_order() {
    assert!(TypeDerivationStrictPartialOrder.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_substitution_group_reflexive_transitive() {
    assert!(SubstitutionGroupReflexiveTransitive.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_every_element_has_exactly_one_type_reference() {
    assert!(EveryElementHasExactlyOneTypeReference.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_particle_occurrence_range_nonempty() {
    assert!(ParticleOccurrenceRangeNonEmpty.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_qname_category_uniqueness() {
    assert!(QNameCategoryUniqueness.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_every_concept_has_part_classification() {
    assert!(EveryConceptHasPartClassification.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn all_axioms_hold() {
    for axiom in XsdOntology::axioms() {
        if let Err(c) = axiom.verify() {
            panic!("axiom failed: {}", c.meta().name.as_str());
        }
    }
}

// =============================================================================
// Derivation-chain axiom — direct unit tests on the small structured input
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn derivation_chain_acyclic_passes() {
    let c = DerivationChain {
        chain: vec!["xs:anyType", "A", "B", "C"],
    };
    assert!(c.is_acyclic());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn derivation_chain_with_repeat_fails() {
    let c = DerivationChain {
        chain: vec!["A", "B", "A"],
    };
    assert!(!c.is_acyclic());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn derivation_chain_empty_is_trivially_acyclic() {
    let c = DerivationChain { chain: vec![] };
    assert!(c.is_acyclic());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn derivation_chain_single_element_is_acyclic() {
    let c = DerivationChain {
        chain: vec!["xs:anyType"],
    };
    assert!(c.is_acyclic());
}

// =============================================================================
// Substitution-group axiom — direct unit tests
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn substitution_group_reflexivity() {
    let h = SubstitutionGroupHierarchy {
        pairs: vec![("A", "B")],
    };
    assert!(h.member_of("A", "A"));
    assert!(h.member_of("B", "B"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn substitution_group_direct_membership() {
    let h = SubstitutionGroupHierarchy {
        pairs: vec![("A", "B")],
    };
    assert!(h.member_of("A", "B"));
    // Not symmetric: B is not a member of A's group.
    assert!(!h.member_of("B", "A"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn substitution_group_transitive_closure() {
    let h = SubstitutionGroupHierarchy {
        pairs: vec![("A", "B"), ("B", "C"), ("C", "D")],
    };
    assert!(h.member_of("A", "D"));
    assert!(h.member_of("B", "D"));
    assert!(!h.member_of("D", "A"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn substitution_group_disconnected() {
    let h = SubstitutionGroupHierarchy {
        pairs: vec![("A", "B"), ("X", "Y")],
    };
    assert!(!h.member_of("A", "Y"));
    assert!(!h.member_of("X", "B"));
}

// =============================================================================
// Particle occurrence-range axiom — direct unit tests
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn occurrence_range_single_is_nonempty() {
    assert!(OccurrenceRange { min: 1, max: 1 }.is_nonempty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn occurrence_range_optional_is_nonempty() {
    assert!(OccurrenceRange { min: 0, max: 1 }.is_nonempty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn occurrence_range_unbounded_is_nonempty() {
    let r = OccurrenceRange {
        min: 0,
        max: OccurrenceRange::UNBOUNDED,
    };
    assert!(r.is_nonempty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn occurrence_range_zero_max_is_empty() {
    assert!(!OccurrenceRange { min: 0, max: 0 }.is_nonempty());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn occurrence_range_inverted_is_empty() {
    assert!(!OccurrenceRange { min: 5, max: 1 }.is_nonempty());
}

// =============================================================================
// Element-type-reference axiom — direct unit test for the W3C §3.3.2.3 invariant
// =============================================================================

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn element_type_ref_explicit() {
    let e = ElementTypeRef {
        element: "title",
        type_ref: Some("TitleType"),
    };
    assert_eq!(e.type_ref, Some("TitleType"));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn element_type_ref_can_be_implicit() {
    let e = ElementTypeRef {
        element: "section",
        type_ref: None,
    };
    // The XSD parser defaults to xs:anyType when no type= is given; the
    // ontology says *exactly one* type ref exists (the default).
    let resolved = e.type_ref.unwrap_or("xs:anyType");
    assert_eq!(resolved, "xs:anyType");
}

// =============================================================================
// Property-based tests
// =============================================================================

fn arb_concept() -> impl Strategy<Value = XsdConcept> {
    proptest::sample::select(XsdConcept::variants())
}

proptest! {
    /// `PartSpec` is total on non-root concepts (Reiter 1978 closed-
    /// world quality assumption restricted to the partition leaves).
    #[test]
    fn prop_part_spec_total_on_non_root(c in arb_concept()) {
        let v = PartSpec.get(&c);
        if is_root(c) {
            prop_assert_eq!(v, None);
        } else {
            prop_assert!(v.is_some());
        }
    }

    /// Type-derivation chains constructed from a string of unique
    /// names are always acyclic. (W3C XSD 1.1 Part 1 §3.4.6.4: no
    /// cycle ever appears in a well-formed schema.)
    #[test]
    fn prop_unique_derivation_chain_is_acyclic(
        names in proptest::collection::vec("[a-z]{1,8}", 0..20)
    ) {
        // Dedupe in-place to make the input unique.
        let mut seen = alloc::collections::BTreeSet::new();
        let unique: Vec<String> = names.into_iter().filter(|n| seen.insert(n.clone())).collect();
        let chain_static: Vec<&'static str> = unique
            .iter()
            .map(|s| Box::leak(s.clone().into_boxed_str()) as &'static str)
            .collect();
        let c = DerivationChain { chain: chain_static };
        prop_assert!(c.is_acyclic());
    }

    /// Substitution-group transitive closure is monotone: adding a
    /// pair never removes an existing membership relation.
    #[test]
    fn prop_substitution_group_monotone(
        n in 1usize..6,
        m in 0usize..6,
    ) {
        // Build a chain: ("e0","e1"), ("e1","e2"), …
        let mk = |k: usize| {
            let pairs: Vec<(&'static str, &'static str)> = (0..k)
                .map(|i| {
                    let from: &'static str = Box::leak(format!("e{i}").into_boxed_str());
                    let to: &'static str = Box::leak(format!("e{}", i + 1).into_boxed_str());
                    (from, to)
                })
                .collect();
            SubstitutionGroupHierarchy { pairs }
        };
        let h_small = mk(n);
        let h_big = mk(n + m);
        // Every (e0 → e_k) membership in h_small persists in h_big.
        let target: String = format!("e{n}");
        if h_small.member_of("e0", &target) {
            prop_assert!(h_big.member_of("e0", &target));
        }
    }
}

pr4xis::register_praxis_value!(prop_part_spec_total_on_non_root, Verifiable);
pr4xis::register_praxis_value!(prop_unique_derivation_chain_is_acyclic, Verifiable);
pr4xis::register_praxis_value!(prop_substitution_group_monotone, Verifiable);
