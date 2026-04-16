use super::ontology::*;
use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::category::validate::check_category_laws;

#[test]
fn category_laws() {
    check_category_laws::<KnowledgeBaseCategory>().unwrap();
}

#[test]
fn six_concepts() {
    assert_eq!(KnowledgeConcept::variants().len(), 6);
}

#[test]
fn knowledge_base_catalogs_vocabulary() {
    let m = KnowledgeBaseCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::KnowledgeBase
        && r.to == KnowledgeConcept::Vocabulary
        && r.kind == KnowledgeRelationKind::Catalogs));
}

#[test]
fn vocabulary_conforms_to_schema() {
    let m = KnowledgeBaseCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::Vocabulary
        && r.to == KnowledgeConcept::Schema
        && r.kind == KnowledgeRelationKind::ConformsTo));
}

#[test]
fn vocabulary_contains_entries() {
    let m = KnowledgeBaseCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::Vocabulary
        && r.to == KnowledgeConcept::Entry
        && r.kind == KnowledgeRelationKind::Contains));
}

#[test]
fn vocabulary_derived_from_datasource() {
    let m = KnowledgeBaseCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::Vocabulary
        && r.to == KnowledgeConcept::DataSource
        && r.kind == KnowledgeRelationKind::DerivedFrom));
}

#[test]
fn schema_defines_entry() {
    let m = KnowledgeBaseCategory::morphisms();
    assert!(m.iter().any(|r| r.from == KnowledgeConcept::Schema
        && r.to == KnowledgeConcept::Entry
        && r.kind == KnowledgeRelationKind::Defines));
}

// =============================================================================
// Knowledge-base descriptor registry tests
// =============================================================================

#[test]
fn describe_knowledge_base_has_108_ontologies() {
    let descriptors = super::descriptor::describe_knowledge_base();
    assert_eq!(
        descriptors.len(),
        108,
        "describe_knowledge_base() should return 108 ontologies; got {}. \
         If you added a new ontology, add a descriptor::<C, E>() entry.",
        descriptors.len()
    );
}

#[test]
fn describe_knowledge_base_names_are_unique() {
    let descriptors = super::descriptor::describe_knowledge_base();
    let mut seen = std::collections::HashSet::new();
    for d in &descriptors {
        assert!(
            seen.insert((d.name, d.domain)),
            "duplicate (name, domain): ({}, {})",
            d.name,
            d.domain
        );
    }
}

#[test]
fn describe_knowledge_base_no_stale_science_prefix() {
    let descriptors = super::descriptor::describe_knowledge_base();
    for d in &descriptors {
        assert!(
            !d.domain.starts_with("science."),
            "stale domain prefix: {} has domain '{}' — should use cognitive/formal/natural/social/applied",
            d.name,
            d.domain
        );
    }
}

#[test]
fn every_descriptor_has_nonzero_concepts() {
    let descriptors = super::descriptor::describe_knowledge_base();
    for d in &descriptors {
        assert!(d.concepts > 0, "{} ({}) has 0 concepts", d.name, d.domain);
    }
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_knowledge() -> impl Strategy<Value = KnowledgeConcept> {
        prop_oneof![
            Just(KnowledgeConcept::KnowledgeBase),
            Just(KnowledgeConcept::Vocabulary),
            Just(KnowledgeConcept::Schema),
            Just(KnowledgeConcept::Entry),
            Just(KnowledgeConcept::Descriptor),
            Just(KnowledgeConcept::DataSource),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_knowledge()) {
            let id = KnowledgeBaseCategory::identity(&c);
            prop_assert_eq!(KnowledgeBaseCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has both Identity and Composed self-morphisms.
        #[test]
        fn prop_self_morphisms(c in arb_knowledge()) {
            let m = KnowledgeBaseCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c && r.kind == KnowledgeRelationKind::Identity);
            let has_composed = m.iter().any(|r| r.from == c && r.to == c && r.kind == KnowledgeRelationKind::Composed);
            prop_assert!(has_identity);
            prop_assert!(has_composed);
        }

        /// VoID: KnowledgeBase reaches every concept transitively.
        #[test]
        fn prop_knowledge_base_reaches_all(c in arb_knowledge()) {
            let m = KnowledgeBaseCategory::morphisms();
            let reachable = m.iter().any(|r| r.from == KnowledgeConcept::KnowledgeBase && r.to == c);
            prop_assert!(reachable, "KnowledgeBase should reach {:?}", c);
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_knowledge()) {
            let m = KnowledgeBaseCategory::morphisms();
            let id = KnowledgeBaseCategory::identity(&c);
            for morph in m.iter().filter(|r| r.from == c) {
                let composed = KnowledgeBaseCategory::compose(&id, morph);
                prop_assert_eq!(composed.as_ref().map(|r| (r.from, r.to)), Some((morph.from, morph.to)));
            }
        }
    }
}
