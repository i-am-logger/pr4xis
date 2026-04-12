use super::ontology::*;
use pr4xis::category::Category;

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_repository() -> impl Strategy<Value = RepositoryConcept> {
        prop_oneof![
            Just(RepositoryConcept::Repository),
            Just(RepositoryConcept::Store),
            Just(RepositoryConcept::StoredOntology),
            Just(RepositoryConcept::Materialize),
            Just(RepositoryConcept::Realize),
            Just(RepositoryConcept::Equivalence),
            Just(RepositoryConcept::StaticStore),
            Just(RepositoryConcept::MappedStore),
            Just(RepositoryConcept::HeapStore),
            Just(RepositoryConcept::DatabaseStore),
            Just(RepositoryConcept::EndpointStore),
        ]
    }

    fn arb_backend() -> impl Strategy<Value = RepositoryConcept> {
        prop_oneof![
            Just(RepositoryConcept::StaticStore),
            Just(RepositoryConcept::MappedStore),
            Just(RepositoryConcept::HeapStore),
            Just(RepositoryConcept::DatabaseStore),
            Just(RepositoryConcept::EndpointStore),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_repository()) {
            let id = RepositoryCategory::identity(&c);
            prop_assert_eq!(RepositoryCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has both Identity and Composed self-morphisms.
        #[test]
        fn prop_self_morphisms(c in arb_repository()) {
            let m = RepositoryCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c && r.kind == RepositoryRelationKind::Identity);
            let has_composed = m.iter().any(|r| r.from == c && r.to == c && r.kind == RepositoryRelationKind::Composed);
            prop_assert!(has_identity);
            prop_assert!(has_composed);
        }

        /// RDF4J: every store backend specializes Store.
        #[test]
        fn prop_backend_specializes_store(backend in arb_backend()) {
            let m = RepositoryCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.from == backend
                && r.to == RepositoryConcept::Store
                && r.kind == RepositoryRelationKind::SpecializesTo));
        }

        /// Roundtrip axiom: Materialize∘Realize = identity.
        #[test]
        fn prop_roundtrip_exists(_dummy in 0..1i32) {
            let m = RepositoryCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.from == RepositoryConcept::Materialize
                && r.to == RepositoryConcept::Realize
                && r.kind == RepositoryRelationKind::Roundtrip));
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_repository()) {
            let m = RepositoryCategory::morphisms();
            let id = RepositoryCategory::identity(&c);
            for morph in m.iter().filter(|r| r.from == c) {
                let composed = RepositoryCategory::compose(&id, morph);
                prop_assert_eq!(composed.as_ref().map(|r| (r.from, r.to)), Some((morph.from, morph.to)));
            }
        }
    }
}
