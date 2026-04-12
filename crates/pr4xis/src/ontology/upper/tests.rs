use crate::category::entity::Entity;
use crate::category::validate::{check_category_laws, check_functor_laws};
use crate::category::{Category, Functor};

use super::being::Being;
use super::category::{DolceCategory, RelationKind};
use super::functor::{PraxisMetaCategory, PraxisToDolce, PraxisType};

// =============================================================================
// Being tests
// =============================================================================

#[test]
fn being_has_7_variants() {
    assert_eq!(Being::variants().len(), 7);
}

#[test]
fn being_endurant_classification() {
    assert!(Being::PhysicalEndurant.is_endurant());
    assert!(Being::SocialObject.is_endurant());
    assert!(Being::MentalObject.is_endurant());
    assert!(Being::AbstractObject.is_endurant());
    assert!(!Being::Event.is_endurant());
    assert!(!Being::Process.is_endurant());
    assert!(!Being::Quality.is_endurant());
}

#[test]
fn being_perdurant_classification() {
    assert!(Being::Event.is_perdurant());
    assert!(Being::Process.is_perdurant());
    assert!(!Being::PhysicalEndurant.is_perdurant());
}

#[test]
fn being_quality_classification() {
    assert!(Being::Quality.is_quality());
    assert!(!Being::Event.is_quality());
}

#[test]
fn being_categories_are_exhaustive() {
    for b in Being::variants() {
        assert!(
            b.is_endurant() || b.is_perdurant() || b.is_quality(),
            "{:?} is not classified",
            b
        );
    }
}

// =============================================================================
// DOLCE Category tests
// =============================================================================

#[test]
fn dolce_category_laws() {
    check_category_laws::<DolceCategory>().unwrap();
}

#[test]
fn dolce_has_participation_morphisms() {
    let morphisms = DolceCategory::morphisms();
    let count = morphisms
        .iter()
        .filter(|m| m.kind == RelationKind::ParticipatesIn)
        .count();
    assert_eq!(count, 8); // 4 endurants × 2 perdurants
}

#[test]
fn dolce_has_inherence_morphisms() {
    let morphisms = DolceCategory::morphisms();
    let count = morphisms
        .iter()
        .filter(|m| m.kind == RelationKind::InheresIn)
        .count();
    assert_eq!(count, 4); // Quality → 4 endurants
}

#[test]
fn dolce_has_constitution() {
    let morphisms = DolceCategory::morphisms();
    assert!(morphisms.iter().any(|m| m.from == Being::PhysicalEndurant
        && m.to == Being::SocialObject
        && m.kind == RelationKind::Constitutes));
}

#[test]
fn dolce_has_event_part_of_process() {
    let morphisms = DolceCategory::morphisms();
    assert!(morphisms.iter().any(|m| m.from == Being::Event
        && m.to == Being::Process
        && m.kind == RelationKind::PartOf));
}

// =============================================================================
// Praxis meta-category tests
// =============================================================================

#[test]
fn praxis_meta_category_laws() {
    check_category_laws::<PraxisMetaCategory>().unwrap();
}

#[test]
fn praxis_has_7_types() {
    assert_eq!(PraxisType::variants().len(), 7);
}

// =============================================================================
// Functor tests — the core proof
// =============================================================================

#[test]
fn functor_laws_hold() {
    check_functor_laws::<PraxisToDolce>().unwrap();
}

#[test]
fn functor_maps_entity_to_abstract() {
    assert_eq!(
        PraxisToDolce::map_object(&PraxisType::Entity),
        Being::AbstractObject
    );
}

#[test]
fn functor_maps_situation_to_social_object() {
    assert_eq!(
        PraxisToDolce::map_object(&PraxisType::Situation),
        Being::SocialObject
    );
}

#[test]
fn functor_maps_action_to_event() {
    assert_eq!(PraxisToDolce::map_object(&PraxisType::Action), Being::Event);
}

#[test]
fn functor_maps_quality_to_quality() {
    assert_eq!(
        PraxisToDolce::map_object(&PraxisType::Quality),
        Being::Quality
    );
}

#[test]
fn functor_preserves_identity() {
    for t in PraxisType::variants() {
        let praxis_id = PraxisMetaCategory::identity(&t);
        let mapped = PraxisToDolce::map_morphism(&praxis_id);
        let dolce_id = DolceCategory::identity(&PraxisToDolce::map_object(&t));
        assert_eq!(mapped, dolce_id, "identity not preserved for {:?}", t);
    }
}

// =============================================================================
// Property-based tests
// =============================================================================

mod prop {
    use super::*;
    use proptest::prelude::*;

    fn arb_being() -> impl Strategy<Value = Being> {
        prop_oneof![
            Just(Being::PhysicalEndurant),
            Just(Being::SocialObject),
            Just(Being::MentalObject),
            Just(Being::AbstractObject),
            Just(Being::Event),
            Just(Being::Process),
            Just(Being::Quality),
        ]
    }

    fn arb_praxis_type() -> impl Strategy<Value = PraxisType> {
        prop_oneof![
            Just(PraxisType::Entity),
            Just(PraxisType::Situation),
            Just(PraxisType::Action),
            Just(PraxisType::Quality),
            Just(PraxisType::CategoryType),
            Just(PraxisType::Axiom),
            Just(PraxisType::Proposition),
        ]
    }

    proptest! {
        /// Every Being is exactly one of: endurant, perdurant, or quality.
        #[test]
        fn prop_being_partition(b in arb_being()) {
            let count = [b.is_endurant(), b.is_perdurant(), b.is_quality()]
                .iter()
                .filter(|&&x| x)
                .count();
            prop_assert_eq!(count, 1);
        }

        /// Functor maps every praxis type to a valid Being.
        #[test]
        fn prop_functor_maps_to_valid_being(t in arb_praxis_type()) {
            let being = PraxisToDolce::map_object(&t);
            prop_assert!(Being::variants().contains(&being));
        }

        /// Functor preserves identity for all types.
        #[test]
        fn prop_functor_preserves_identity(t in arb_praxis_type()) {
            let praxis_id = PraxisMetaCategory::identity(&t);
            let mapped = PraxisToDolce::map_morphism(&praxis_id);
            let dolce_id = DolceCategory::identity(&PraxisToDolce::map_object(&t));
            prop_assert_eq!(mapped, dolce_id);
        }

        /// Identity compose with identity equals identity.
        #[test]
        fn prop_dolce_identity_compose(b in arb_being()) {
            let id = DolceCategory::identity(&b);
            let composed = DolceCategory::compose(&id, &id);
            prop_assert_eq!(composed, Some(id));
        }

        /// Functor preserves composition for any composable pair.
        #[test]
        fn prop_functor_preserves_composition(
            a in arb_praxis_type(),
            b in arb_praxis_type(),
            c in arb_praxis_type()
        ) {
            let morphisms = PraxisMetaCategory::morphisms();
            if let Some(f) = morphisms.iter().find(|m| m.from == a && m.to == b) {
                if let Some(g) = morphisms.iter().find(|m| m.from == b && m.to == c) {
                    if let Some(gf) = PraxisMetaCategory::compose(f, g) {
                        let mapped_gf = PraxisToDolce::map_morphism(&gf);
                        let composed_mapped = DolceCategory::compose(
                            &PraxisToDolce::map_morphism(f),
                            &PraxisToDolce::map_morphism(g),
                        );
                        prop_assert_eq!(Some(mapped_gf), composed_mapped);
                    }
                }
            }
        }

        /// Endurants participate in perdurants (for all combinations).
        #[test]
        fn prop_endurant_participates_in_perdurant(a in arb_being(), b in arb_being()) {
            if a.is_endurant() && b.is_perdurant() {
                let morphisms = DolceCategory::morphisms();
                let has_participation = morphisms.iter().any(|m| m.from == a && m.to == b);
                prop_assert!(has_participation,
                    "{:?} should participate in {:?}", a, b);
            }
        }

        /// Quality inheres in all endurants.
        #[test]
        fn prop_quality_inheres_in_endurant(b in arb_being()) {
            if b.is_endurant() {
                let morphisms = DolceCategory::morphisms();
                let has_inherence = morphisms.iter().any(|m|
                    m.from == Being::Quality && m.to == b);
                prop_assert!(has_inherence,
                    "Quality should inhere in {:?}", b);
            }
        }

        /// No Being is both endurant and perdurant.
        #[test]
        fn prop_endurant_perdurant_exclusive(b in arb_being()) {
            prop_assert!(!(b.is_endurant() && b.is_perdurant()));
        }
    }
}
