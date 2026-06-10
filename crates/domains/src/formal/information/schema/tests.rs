use super::ontology::*;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_schema() -> impl Strategy<Value = SchemaConcept> {
        proptest::sample::select(SchemaConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_identity_idempotent(c in arb_schema()) {
            let id = SchemaCategory::identity(&c);
            prop_assert_eq!(SchemaCategory::compose(&id, &id), Some(id));
        }

        /// Every concept has an Identity self-morphism. The dense
        /// `Composed` kind was removed (#166).
        #[test]
        fn prop_self_identity(c in arb_schema()) {
            let m = SchemaCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == c
                && r.target() == c
                && r.kind() == SchemaRelationKind::Identity));
        }

        /// Spivak (2012): Schema contains its structural components.
        #[test]
        fn prop_schema_contains_components(_dummy in 0..1i32) {
            let m = SchemaCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == SchemaConcept::Schema
                && r.target() == SchemaConcept::EntityType));
            prop_assert!(m.iter().any(|r| r.source() == SchemaConcept::Schema
                && r.target() == SchemaConcept::MorphismType));
            prop_assert!(m.iter().any(|r| r.source() == SchemaConcept::Schema
                && r.target() == SchemaConcept::PathEquation));
            prop_assert!(m.iter().any(|r| r.source() == SchemaConcept::Schema
                && r.target() == SchemaConcept::Axiom));
        }

        /// Spivak (2012): Instance is a functor from Schema.
        #[test]
        fn prop_instance_from_schema(_dummy in 0..1i32) {
            let m = SchemaCategory::morphisms();
            prop_assert!(m.iter().any(|r|
                r.source() == SchemaConcept::Instance
                && r.target() == SchemaConcept::Schema
                && r.kind() == SchemaRelationKind::InstantiatedFrom));
        }

        /// CQL: Presentation ↔ Algebra (evaluation/presentation adjunction).
        #[test]
        fn prop_presentation_algebra_adjunction(_dummy in 0..1i32) {
            let m = SchemaCategory::morphisms();
            prop_assert!(m.iter().any(|r| r.source() == SchemaConcept::Presentation
                && r.target() == SchemaConcept::Algebra));
            prop_assert!(m.iter().any(|r| r.source() == SchemaConcept::Algebra
                && r.target() == SchemaConcept::Presentation));
        }

        /// Composition with identity preserves any morphism.
        #[test]
        fn prop_left_identity(c in arb_schema()) {
            let m = SchemaCategory::morphisms();
            let id = SchemaCategory::identity(&c);
            for morph in m.iter().filter(|r| r.source() == c) {
                let composed = SchemaCategory::compose(&id, morph);
                prop_assert_eq!(
                    composed.as_ref().map(|r| (r.source(), r.target())),
                    Some((morph.source(), morph.target()))
                );
            }
        }
    }
}
