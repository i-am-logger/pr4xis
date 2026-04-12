use super::ontology::*;
use pr4xis::category::Category;

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_schema() -> impl Strategy<Value = SchemaConcept> {
        prop_oneof![
            Just(SchemaConcept::Schema),
            Just(SchemaConcept::EntityType),
            Just(SchemaConcept::MorphismType),
            Just(SchemaConcept::PathEquation),
            Just(SchemaConcept::Axiom),
            Just(SchemaConcept::Instance),
            Just(SchemaConcept::Population),
            Just(SchemaConcept::SchemaMapping),
            Just(SchemaConcept::Transform),
            Just(SchemaConcept::Presentation),
            Just(SchemaConcept::Algebra),
        ]
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_schema()) {
            let id = SchemaCategory::identity(&c);
            prop_assert_eq!(SchemaCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has both Identity and Composed self-morphisms.
        #[test]
        fn prop_self_morphisms(c in arb_schema()) {
            let m = SchemaCategory::morphisms();
            let has_identity = m.iter().any(|r| r.from == c && r.to == c && r.kind == SchemaRelationKind::Identity);
            let has_composed = m.iter().any(|r| r.from == c && r.to == c && r.kind == SchemaRelationKind::Composed);
            prop_assert!(has_identity);
            prop_assert!(has_composed);
        }

        /// Spivak (2012): Schema contains its structural components.
        #[test]
        fn prop_schema_contains_components(_dummy in 0..1i32) {
            let m = SchemaCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.from == SchemaConcept::Schema && r.to == SchemaConcept::EntityType));
            prop_assert!(m.iter().any(|r| r.from == SchemaConcept::Schema && r.to == SchemaConcept::MorphismType));
            prop_assert!(m.iter().any(|r| r.from == SchemaConcept::Schema && r.to == SchemaConcept::PathEquation));
            prop_assert!(m.iter().any(|r| r.from == SchemaConcept::Schema && r.to == SchemaConcept::Axiom));
        }

        /// Spivak: Instance is a functor from Schema.
        #[test]
        fn prop_instance_from_schema(_dummy in 0..1i32) {
            let m = SchemaCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.from == SchemaConcept::Instance
                && r.to == SchemaConcept::Schema
                && r.kind == SchemaRelationKind::InstantiatedFrom));
        }

        /// CQL: Presentation ↔ Algebra (evaluation/presentation adjunction).
        #[test]
        fn prop_presentation_algebra_adjunction(_dummy in 0..1i32) {
            let m = SchemaCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.from == SchemaConcept::Presentation && r.to == SchemaConcept::Algebra));
            prop_assert!(m.iter().any(|r| r.from == SchemaConcept::Algebra && r.to == SchemaConcept::Presentation));
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_schema()) {
            let m = SchemaCategory::morphisms();
            let id = SchemaCategory::identity(&c);
            for morph in m.iter().filter(|r| r.from == c) {
                let composed = SchemaCategory::compose(&id, morph);
                prop_assert_eq!(composed.as_ref().map(|r| (r.from, r.to)), Some((morph.from, morph.to)));
            }
        }
    }
}
