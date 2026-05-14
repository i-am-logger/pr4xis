use super::ontology::*;
use pr4xis::category::{Arrow, Category, Concept};

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_repository() -> impl Strategy<Value = RepositoryConcept> {
        proptest::sample::select(RepositoryConcept::variants())
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

        /// Every concept has an Identity self-morphism (#166 removed `Composed`).
        #[test]
        fn prop_self_identity(c in arb_repository()) {
            let m = RepositoryCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == c
                && r.target() == c
                && r.kind() == RepositoryRelationKind::Identity));
        }

        /// RDF4J: every store backend subsumes Store (is-a relation).
        #[test]
        fn prop_backend_subsumes_store(backend in arb_backend()) {
            let m = RepositoryCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.source() == backend
                && r.target() == RepositoryConcept::Store
                && r.kind() == RepositoryRelationKind::Subsumption));
        }

        /// Roundtrip axiom: Materialize ∘ Realize = identity.
        #[test]
        fn prop_roundtrip_exists(_dummy in 0..1i32) {
            let m = RepositoryCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.source() == RepositoryConcept::Materialize
                && r.target() == RepositoryConcept::Realize
                && r.kind() == RepositoryRelationKind::Roundtrip));
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_repository()) {
            let m = RepositoryCategory::morphisms();
            let id = RepositoryCategory::identity(&c);
            for morph in m.iter().filter(|r| r.source() == c) {
                let composed = RepositoryCategory::compose(&id, morph);
                prop_assert_eq!(
                    composed.as_ref().map(|r| (r.source(), r.target())),
                    Some((morph.source(), morph.target()))
                );
            }
        }
    }
}
