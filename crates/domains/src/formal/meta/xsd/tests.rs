//! Tests for the XSD ontology + xsd-parser AST functor.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use super::from_xsd_parser::{
    FromXsdParser, XsdAst, XsdAstCategory, XsdAstNodeKind, XsdAstRelationKind,
    classify_codegen_name, project, project_node_kind,
};
use super::ontology::{
    AnnotationBinaryPartition, DerivationChain, ElementTypeRef, EveryConceptHasPartClassification,
    EveryElementHasExactlyOneTypeReference, ModelGroupTernaryPartition, OccurrenceRange, PartSpec,
    ParticleOccurrenceRangeNonEmpty, QNameCategoryUniqueness, SchemaComponentPartitioned,
    SubstitutionGroupHierarchy, SubstitutionGroupReflexiveTransitive,
    TypeDefinitionBinaryPartition, TypeDerivationStrictPartialOrder, XsdCategory, XsdConcept,
    XsdOntology, XsdPart, instantiable_leaves, is_root,
};
use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};
use pr4xis::category::{Category, Concept};
use pr4xis::ontology::{Axiom, Ontology, Quality};
use proptest::prelude::*;

// =============================================================================
// Category + ontology validation
// =============================================================================

#[test]
fn xsd_category_laws() {
    assert_category_laws::<XsdCategory>();
}

#[test]
fn xsd_ast_category_laws() {
    assert_category_laws::<XsdAstCategory>();
}

#[test]
fn xsd_ontology_validates() {
    XsdOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

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

#[test]
fn instantiable_leaves_count() {
    // 44 concrete kinds: 1 §2.5 SchemaDocument + 11 §2.2 schema-component
    // leaves (IdentityConstraint is now intermediate) + 4 §4.2 composition
    // directives + 6 §3.4.2 / Part 2 §4.1.2 type-construction constructs +
    // 14 Part 2 §4.3 facets + 5 §3.11 identity-constraint sub-kinds +
    // 3 XSD 1.1 content additions (Assert, OpenContent, DefaultOpenContent).
    assert_eq!(instantiable_leaves().len(), 44);
}

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

#[test]
fn part_spec_simple_is_part2() {
    assert_eq!(
        PartSpec.get(&XsdConcept::SimpleTypeDefinition),
        Some(XsdPart::Datatypes)
    );
}

#[test]
fn part_spec_complex_is_part1() {
    assert_eq!(
        PartSpec.get(&XsdConcept::ComplexTypeDefinition),
        Some(XsdPart::Structures)
    );
}

#[test]
fn part_spec_root_is_none() {
    assert_eq!(PartSpec.get(&XsdConcept::SchemaComponent), None);
}

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

#[test]
fn axiom_schema_component_partitioned() {
    assert!(SchemaComponentPartitioned.verify().is_ok());
}

#[test]
fn axiom_type_definition_binary_partition() {
    assert!(TypeDefinitionBinaryPartition.verify().is_ok());
}

#[test]
fn axiom_model_group_ternary_partition() {
    assert!(ModelGroupTernaryPartition.verify().is_ok());
}

#[test]
fn axiom_annotation_binary_partition() {
    assert!(AnnotationBinaryPartition.verify().is_ok());
}

#[test]
fn axiom_type_derivation_strict_partial_order() {
    assert!(TypeDerivationStrictPartialOrder.verify().is_ok());
}

#[test]
fn axiom_substitution_group_reflexive_transitive() {
    assert!(SubstitutionGroupReflexiveTransitive.verify().is_ok());
}

#[test]
fn axiom_every_element_has_exactly_one_type_reference() {
    assert!(EveryElementHasExactlyOneTypeReference.verify().is_ok());
}

#[test]
fn axiom_particle_occurrence_range_nonempty() {
    assert!(ParticleOccurrenceRangeNonEmpty.verify().is_ok());
}

#[test]
fn axiom_qname_category_uniqueness() {
    assert!(QNameCategoryUniqueness.verify().is_ok());
}

#[test]
fn axiom_every_concept_has_part_classification() {
    assert!(EveryConceptHasPartClassification.verify().is_ok());
}

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

#[test]
fn derivation_chain_acyclic_passes() {
    let c = DerivationChain {
        chain: vec!["xs:anyType", "A", "B", "C"],
    };
    assert!(c.is_acyclic());
}

#[test]
fn derivation_chain_with_repeat_fails() {
    let c = DerivationChain {
        chain: vec!["A", "B", "A"],
    };
    assert!(!c.is_acyclic());
}

#[test]
fn derivation_chain_empty_is_trivially_acyclic() {
    let c = DerivationChain { chain: vec![] };
    assert!(c.is_acyclic());
}

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

#[test]
fn substitution_group_reflexivity() {
    let h = SubstitutionGroupHierarchy {
        pairs: vec![("A", "B")],
    };
    assert!(h.member_of("A", "A"));
    assert!(h.member_of("B", "B"));
}

#[test]
fn substitution_group_direct_membership() {
    let h = SubstitutionGroupHierarchy {
        pairs: vec![("A", "B")],
    };
    assert!(h.member_of("A", "B"));
    // Not symmetric: B is not a member of A's group.
    assert!(!h.member_of("B", "A"));
}

#[test]
fn substitution_group_transitive_closure() {
    let h = SubstitutionGroupHierarchy {
        pairs: vec![("A", "B"), ("B", "C"), ("C", "D")],
    };
    assert!(h.member_of("A", "D"));
    assert!(h.member_of("B", "D"));
    assert!(!h.member_of("D", "A"));
}

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

#[test]
fn occurrence_range_single_is_nonempty() {
    assert!(OccurrenceRange { min: 1, max: 1 }.is_nonempty());
}

#[test]
fn occurrence_range_optional_is_nonempty() {
    assert!(OccurrenceRange { min: 0, max: 1 }.is_nonempty());
}

#[test]
fn occurrence_range_unbounded_is_nonempty() {
    let r = OccurrenceRange {
        min: 0,
        max: OccurrenceRange::UNBOUNDED,
    };
    assert!(r.is_nonempty());
}

#[test]
fn occurrence_range_zero_max_is_empty() {
    assert!(!OccurrenceRange { min: 0, max: 0 }.is_nonempty());
}

#[test]
fn occurrence_range_inverted_is_empty() {
    assert!(!OccurrenceRange { min: 5, max: 1 }.is_nonempty());
}

// =============================================================================
// Element-type-reference axiom — direct unit test for the W3C §3.3.2.3 invariant
// =============================================================================

#[test]
fn element_type_ref_explicit() {
    let e = ElementTypeRef {
        element: "title",
        type_ref: Some("TitleType"),
    };
    assert_eq!(e.type_ref, Some("TitleType"));
}

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
// Functor laws — Mac Lane §I.3
// =============================================================================

#[test]
fn from_xsd_parser_functor_laws_pass() {
    assert_functor_laws::<FromXsdParser>();
}

#[test]
fn functor_meta_carries_citation() {
    use pr4xis::category::Functor;
    let meta = FromXsdParser::meta();
    assert_eq!(meta.name.as_str(), "FromXsdParser");
    assert!(meta.citation.as_str().contains("Mac Lane"));
    assert!(meta.module_path.as_str().contains("xsd"));
}

// =============================================================================
// Functor object map — every AST node kind has an image, and the image is
// a concrete (instantiable) XSD concept (not the abstract root).
// =============================================================================

#[test]
fn project_node_kind_total_on_ast_variants() {
    for kind in XsdAstNodeKind::variants() {
        let concept = project_node_kind(kind);
        assert!(
            !is_root(concept),
            "AST node kind {kind:?} projected to the abstract root"
        );
    }
}

#[test]
fn project_element_to_element_declaration() {
    assert_eq!(
        project_node_kind(XsdAstNodeKind::Element),
        XsdConcept::ElementDeclaration
    );
}

#[test]
fn project_complex_type_to_complex_type_definition() {
    assert_eq!(
        project_node_kind(XsdAstNodeKind::ComplexType),
        XsdConcept::ComplexTypeDefinition
    );
}

#[test]
fn project_simple_type_to_simple_type_definition() {
    assert_eq!(
        project_node_kind(XsdAstNodeKind::SimpleType),
        XsdConcept::SimpleTypeDefinition
    );
}

#[test]
fn project_sequence_choice_all_to_model_group_kinds() {
    assert_eq!(
        project_node_kind(XsdAstNodeKind::Sequence),
        XsdConcept::Sequence
    );
    assert_eq!(
        project_node_kind(XsdAstNodeKind::Choice),
        XsdConcept::Choice
    );
    assert_eq!(project_node_kind(XsdAstNodeKind::All), XsdConcept::AllGroup);
}

#[test]
fn project_group_to_model_group() {
    assert_eq!(
        project_node_kind(XsdAstNodeKind::Group),
        XsdConcept::ModelGroup
    );
}

#[test]
fn project_wildcards_to_wildcard() {
    assert_eq!(
        project_node_kind(XsdAstNodeKind::AnyElement),
        XsdConcept::Wildcard
    );
    assert_eq!(
        project_node_kind(XsdAstNodeKind::AnyAttribute),
        XsdConcept::Wildcard
    );
}

#[test]
fn project_identity_constraints() {
    for k in [
        XsdAstNodeKind::Key,
        XsdAstNodeKind::Unique,
        XsdAstNodeKind::Keyref,
    ] {
        assert_eq!(project_node_kind(k), XsdConcept::IdentityConstraint);
    }
}

#[test]
fn project_annotation_children() {
    assert_eq!(
        project_node_kind(XsdAstNodeKind::AppInfo),
        XsdConcept::AppInfo
    );
    assert_eq!(
        project_node_kind(XsdAstNodeKind::Documentation),
        XsdConcept::Documentation
    );
}

// =============================================================================
// Identity / composition — Mac Lane §I.3 functor laws spelled out manually
// for the relevant constructions, in addition to the generic
// `assert_functor_laws::<FromXsdParser>()` check above.
// =============================================================================

#[test]
fn functor_preserves_identity() {
    use pr4xis::category::Functor;
    for kind in XsdAstNodeKind::variants() {
        let id_src = XsdAstCategory::identity(&kind);
        let mapped = FromXsdParser::map_morphism(&id_src);
        assert_eq!(
            mapped.from,
            project_node_kind(kind),
            "identity should map to identity on the projected object"
        );
        assert_eq!(mapped.from, mapped.to);
    }
}

#[test]
fn functor_preserves_composition_on_identities() {
    use pr4xis::category::{Category, Functor};
    // Source category is discrete, so the only composable pairs are
    // (id_x, id_x). For each such pair, F(g∘f) = F(g)∘F(f).
    for kind in XsdAstNodeKind::variants() {
        let id = XsdAstCategory::identity(&kind);
        let composed_src = XsdAstCategory::compose(&id, &id);
        assert!(composed_src.is_some());
        let mapped_composed = FromXsdParser::map_morphism(&composed_src.unwrap());
        let f_id = FromXsdParser::map_morphism(&id);
        let composed_tgt = XsdCategory::compose(&f_id, &f_id);
        assert_eq!(composed_tgt, Some(mapped_composed));
    }
}

// =============================================================================
// AST projection — `XsdAst` → `XsdOntologyInstance`
// =============================================================================

#[test]
fn empty_ast_projects_to_empty_instance() {
    let ast = XsdAst::default();
    let instance = project(&ast);
    assert_eq!(instance.schema_components().count(), 0);
}

#[test]
fn single_node_ast_projects_to_single_component() {
    let ast = XsdAst {
        nodes: vec![XsdAstNodeKind::Element],
    };
    let instance = project(&ast);
    assert_eq!(instance.schema_components().count(), 1);
    assert_eq!(
        *instance.schema_components().next().unwrap(),
        XsdConcept::ElementDeclaration
    );
}

#[test]
fn nontrivial_ast_projects_to_expected_components() {
    // A simulated USLM-shape mini-AST: a couple of elements, a couple of
    // complex types, an annotation block.
    let ast = XsdAst {
        nodes: vec![
            XsdAstNodeKind::Element,
            XsdAstNodeKind::Element,
            XsdAstNodeKind::ComplexType,
            XsdAstNodeKind::ComplexType,
            XsdAstNodeKind::SimpleType,
            XsdAstNodeKind::Annotation,
            XsdAstNodeKind::Documentation,
        ],
    };
    let instance = project(&ast);
    assert_eq!(instance.schema_components().count(), 7);
    // Five distinct kinds: Element / Complex / Simple / Annotation /
    // Documentation.
    assert_eq!(instance.distinct_concept_count(), 5);
}

// =============================================================================
// Smoke: project the loaded USLM AST through the functor.
//
// The dispatch says: demonstrate the functor by projecting the
// codegen-emitted USLM types. The codegen output lives in
// `crate::social::software::markup::xml::uslm::generated`; every
// `pub struct ... TypeItem` corresponds to one xsd-parser AST node of
// kind `ComplexType`. We classify a representative set of the
// generated type *names* (via `classify_codegen_name`) and confirm the
// functor projects them into the XSD ontology.
//
// We don't compile-time iterate over every generated type (Rust has
// no reflection over module contents); instead we exercise a curated
// list of names known to appear in the USLM-1.0.18 codegen output
// (per commit 58a6836).
// =============================================================================

#[test]
fn classify_uslm_complex_type_names() {
    // Names known to be emitted by xsd-parser for USLM-1.0.18.xsd.
    let known_complex_names = [
        "ActionTypeItem",
        "AppendixTypeItem",
        "BlockTypeItem",
        "HeadingTypeItem",
        "InlineTypeItem",
        "NoteTypeItem",
        "PTypeItem",
        "RefTypeItem",
        "TocTypeItem",
    ];
    for n in &known_complex_names {
        let k = classify_codegen_name(n).expect("classified");
        assert_eq!(k, XsdAstNodeKind::ComplexType, "{n} → ComplexType");
    }
}

#[test]
fn classify_uslm_enum_name() {
    // xsd-parser emits enumeration-typed simple types as `<X>EnumItem`.
    let k = classify_codegen_name("StatusEnumItem").expect("classified");
    assert_eq!(k, XsdAstNodeKind::SimpleType);
}

#[test]
fn classify_unknown_name_returns_none() {
    assert_eq!(classify_codegen_name("Vec"), None);
    assert_eq!(classify_codegen_name(""), None);
}

#[test]
fn uslm_projected_ast_has_schema_components() {
    // Build a minimal AST projected from a few representative USLM
    // type names; confirm the projection produces ontology instances.
    let names = ["ActionTypeItem", "StatusEnumItem", "NoteTypeItem"];
    let nodes: Vec<XsdAstNodeKind> = names
        .iter()
        .filter_map(|n| classify_codegen_name(n))
        .collect();
    assert_eq!(nodes.len(), 3);
    let ast = XsdAst { nodes };
    let instance = project(&ast);
    assert_eq!(instance.schema_components().count(), 3);
    // Two complex types + one simple type.
    let kinds: Vec<XsdConcept> = instance.schema_components().copied().collect();
    assert_eq!(
        kinds
            .iter()
            .filter(|c| **c == XsdConcept::ComplexTypeDefinition)
            .count(),
        2
    );
    assert_eq!(
        kinds
            .iter()
            .filter(|c| **c == XsdConcept::SimpleTypeDefinition)
            .count(),
        1
    );
}

// =============================================================================
// Property-based tests
// =============================================================================

fn arb_ast_node_kind() -> impl Strategy<Value = XsdAstNodeKind> {
    proptest::sample::select(XsdAstNodeKind::variants())
}

fn arb_concept() -> impl Strategy<Value = XsdConcept> {
    proptest::sample::select(XsdConcept::variants())
}

proptest! {
    /// For any AST node kind, the projected concept is never the
    /// abstract root, and is never the abstract `TypeDefinition`
    /// (always Complex or Simple). `ModelGroup` and `Annotation` are
    /// concrete enough to project to directly — xsd-parser's `Group`
    /// AST kind lands on `ModelGroup`, and `Annotation` AST kind on
    /// `Annotation` (Part 1 §3.15, an `<xs:annotation>` block without
    /// AppInfo/Documentation children still exists as the wrapper).
    #[test]
    fn prop_projection_avoids_abstract_root_and_typedef(k in arb_ast_node_kind()) {
        let c = project_node_kind(k);
        prop_assert!(!matches!(c, XsdConcept::SchemaComponent));
        prop_assert!(!matches!(c, XsdConcept::TypeDefinition));
        // ModelGroup is allowed via the `Group` AST kind; other
        // model-group AST kinds (Sequence/Choice/All) land in a
        // strict compositor leaf.
        if k != XsdAstNodeKind::Group {
            prop_assert!(!matches!(c, XsdConcept::ModelGroup));
        }
        // Annotation is allowed via the `Annotation` AST kind; the
        // two child AST kinds (AppInfo/Documentation) land in
        // strict leaves below it.
        if !matches!(k, XsdAstNodeKind::Annotation) {
            prop_assert!(!matches!(c, XsdConcept::Annotation));
        }
    }

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

    /// Functor object map composed twice yields the same concept
    /// (every kind maps to a single concept).
    #[test]
    fn prop_functor_object_map_deterministic(k in arb_ast_node_kind()) {
        prop_assert_eq!(project_node_kind(k), project_node_kind(k));
    }
}

// =============================================================================
// AST relation kind smoke — the source category's only kind is Identity.
// =============================================================================

#[test]
fn xsd_ast_relation_kind_is_identity_only() {
    use pr4xis::category::Arrow;
    let m = XsdAstCategory::identity(&XsdAstNodeKind::Element);
    assert!(matches!(m.kind(), XsdAstRelationKind::Identity));
}
