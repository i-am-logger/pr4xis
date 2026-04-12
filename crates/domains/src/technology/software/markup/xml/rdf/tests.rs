use super::ontology::*;
use pr4xis::category::Category;
use pr4xis::category::entity::Entity;
use pr4xis::logic::Axiom;

// =============================================================================
// Category law tests
// =============================================================================

#[test]
fn identity_law() {
    for obj in RdfNodeKind::variants() {
        let id = RdfCategory::identity(&obj);
        assert_eq!(id.source, obj);
        assert_eq!(id.target, obj);
    }
}

#[test]
fn composition_with_identity() {
    let morphisms = RdfCategory::morphisms();
    for m in &morphisms {
        // id_source ∘ f = f
        let id_src = RdfCategory::identity(&m.source);
        let composed = RdfCategory::compose(&id_src, m);
        assert_eq!(composed.as_ref(), Some(m), "id ∘ f should equal f");

        // f ∘ id_target = f
        let id_tgt = RdfCategory::identity(&m.target);
        let composed = RdfCategory::compose(m, &id_tgt);
        assert_eq!(composed.as_ref(), Some(m), "f ∘ id should equal f");
    }
}

#[test]
fn associativity() {
    let morphisms = RdfCategory::morphisms();
    for f in &morphisms {
        for g in morphisms.iter().filter(|g| g.source == f.target) {
            for h in morphisms.iter().filter(|h| h.source == g.target) {
                let fg = RdfCategory::compose(f, g);
                let gh = RdfCategory::compose(g, h);
                if let (Some(fg), Some(gh)) = (&fg, &gh) {
                    let f_gh = RdfCategory::compose(f, &gh);
                    let fg_h = RdfCategory::compose(&fg, h);
                    assert_eq!(f_gh, fg_h, "associativity: (f∘g)∘h = f∘(g∘h)");
                }
            }
        }
    }
}

// =============================================================================
// Axiom tests
// =============================================================================

#[test]
fn literals_cannot_be_subjects() {
    assert!(LiteralsCannotBeSubjects.holds());
}

#[test]
fn predicates_must_be_properties() {
    assert!(PredicatesMustBeProperties.holds());
}

// =============================================================================
// Node kind property tests
// =============================================================================

#[test]
fn resources_can_be_subjects() {
    let resources = [
        RdfNodeKind::IriResource,
        RdfNodeKind::BlankNode,
        RdfNodeKind::Class,
        RdfNodeKind::Property,
    ];
    for r in resources {
        assert!(r.can_be_subject(), "{:?} should be able to be a subject", r);
    }
}

#[test]
fn literals_are_literals() {
    assert!(RdfNodeKind::PlainLiteral.is_literal());
    assert!(RdfNodeKind::TypedLiteral.is_literal());
    assert!(!RdfNodeKind::IriResource.is_literal());
    assert!(!RdfNodeKind::Class.is_literal());
}

// =============================================================================
// RDFS taxonomy tests
// =============================================================================

#[test]
fn rdfs_taxonomy_has_expected_relations() {
    let tax = rdfs_taxonomy();
    assert!(tax.contains(&(RdfNodeKind::Class, RdfNodeKind::IriResource)));
    assert!(tax.contains(&(RdfNodeKind::Property, RdfNodeKind::IriResource)));
    assert!(tax.contains(&(RdfNodeKind::Datatype, RdfNodeKind::Class)));
    assert!(tax.contains(&(RdfNodeKind::Nil, RdfNodeKind::List)));
}

#[test]
fn rdfs_taxonomy_datatype_is_a_class_is_a_resource() {
    let tax = rdfs_taxonomy();
    assert!(tax.contains(&(RdfNodeKind::Datatype, RdfNodeKind::Class)));
    assert!(tax.contains(&(RdfNodeKind::Class, RdfNodeKind::IriResource)));
}

// =============================================================================
// Vocabulary tests
// =============================================================================

#[test]
fn rdf_vocabulary_iris_are_valid() {
    assert!(RdfVocabulary::RDF_TYPE.starts_with("http://"));
    assert!(RdfVocabulary::RDFS_CLASS.starts_with("http://"));
    assert!(RdfVocabulary::RDFS_SUB_CLASS_OF.contains("subClassOf"));
    assert!(RdfVocabulary::RDFS_DOMAIN.contains("domain"));
    assert!(RdfVocabulary::RDFS_RANGE.contains("range"));
}

#[test]
fn vocabulary_resolve_known_prefixes() {
    let resolved = RdfVocabulary::resolve("rdf", "type");
    assert_eq!(resolved.as_deref(), Some(RdfVocabulary::RDF_TYPE));

    let resolved = RdfVocabulary::resolve("rdfs", "subClassOf");
    assert_eq!(resolved.as_deref(), Some(RdfVocabulary::RDFS_SUB_CLASS_OF));
}

#[test]
fn vocabulary_resolve_unknown_prefix() {
    assert!(RdfVocabulary::resolve("foo", "bar").is_none());
}

#[test]
fn morphism_set_is_nonempty() {
    let morphisms = RdfCategory::morphisms();
    assert!(morphisms.len() > 10);
}

#[test]
fn category_laws() {
    use pr4xis::category::validate::check_category_laws;
    check_category_laws::<RdfCategory>().unwrap();
}

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_rdf() -> impl Strategy<Value = RdfNodeKind> {
        prop_oneof![
            Just(RdfNodeKind::IriResource),
            Just(RdfNodeKind::BlankNode),
            Just(RdfNodeKind::PlainLiteral),
            Just(RdfNodeKind::TypedLiteral),
            Just(RdfNodeKind::Statement),
            Just(RdfNodeKind::Class),
            Just(RdfNodeKind::Property),
            Just(RdfNodeKind::Datatype),
            Just(RdfNodeKind::Nil),
            Just(RdfNodeKind::List),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_rdf()) {
            let id = RdfCategory::identity(&c);
            prop_assert_eq!(RdfCategory::compose(&id, &id), Some(id));
        }

        /// RDF 1.1 §3.3: literals can never be subjects.
        #[test]
        fn prop_literals_cannot_be_subjects(c in arb_rdf()) {
            if c.is_literal() {
                prop_assert!(!c.can_be_subject());
            }
        }

        /// RDFS: resources can be subjects.
        #[test]
        fn prop_resources_can_be_subjects(_dummy in 0..1i32) {
            prop_assert!(RdfNodeKind::IriResource.can_be_subject());
            prop_assert!(RdfNodeKind::BlankNode.can_be_subject());
            prop_assert!(RdfNodeKind::Class.can_be_subject());
            prop_assert!(RdfNodeKind::Property.can_be_subject());
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_rdf()) {
            let m = RdfCategory::morphisms();
            let id = RdfCategory::identity(&c);
            for morph in m.iter().filter(|r| r.source == c) {
                let composed = RdfCategory::compose(&id, morph);
                prop_assert_eq!(composed.as_ref().map(|r| (r.source, r.target)), Some((morph.source, morph.target)));
            }
        }
    }
}
