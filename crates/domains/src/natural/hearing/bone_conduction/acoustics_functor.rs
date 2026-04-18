//! Functor: AcousticsCategory -> BoneConductionCategory.
//!
//! Proves that the acoustics domain has a structure-preserving map into
//! the bone conduction domain. Each acoustic entity maps to its BC role:
//! sound waves become skull vibrations, media become application contexts,
//! phenomena become BC-specific effects.
//!
//! Functor laws (identity + composition preservation) guarantee the mapping
//! is mathematically valid.

use pr4xis::category::{Category, Functor, Relationship};

use crate::natural::hearing::acoustics::ontology::{
    AcousticEntity, AcousticRelation, AcousticsCategory, AcousticsCategoryRelationKind,
};
use crate::natural::hearing::bone_conduction::ontology::{
    BoneCondEntity, BoneCondRelation, BoneConductionCategory, BoneConductionCategoryRelationKind,
};

/// Structure-preserving map from acoustic entities to their BC role.
pub struct AcousticsToBoneConduction;

impl Functor for AcousticsToBoneConduction {
    type Source = AcousticsCategory;
    type Target = BoneConductionCategory;

    fn map_object(obj: &AcousticEntity) -> BoneCondEntity {
        use AcousticEntity as A;
        use BoneCondEntity::*;
        match obj {
            // Wave types → skull vibration
            A::SoundWave | A::LongitudinalWave | A::TransverseWave | A::ShearWave | A::Wave => {
                SkullVibration
            }
            // Wave properties → force level (the measurable quantity in BC)
            A::Frequency
            | A::Amplitude
            | A::Wavelength
            | A::Phase
            | A::Intensity
            | A::WaveProperty => ForceLevel,
            // Bone media → application sites
            A::CorticalBone => Mastoid,
            A::CancellousBone => TemporalBone,
            // Other media → skin drive (external coupling)
            A::Air
            | A::Water
            | A::SoftTissue
            | A::Cartilage
            | A::Fluid
            | A::Medium
            | A::Solid
            | A::BoneTissue => SkinDriveTransducer,
            // Acoustic phenomena → BC phenomena
            A::Resonance => SkullResonance,
            A::ImpedanceMismatch => TranscranialAttenuation,
            A::Reflection | A::Refraction | A::Diffraction => SkullResonance,
            A::Absorption | A::Attenuation => TranscranialAttenuation,
            A::AcousticPhenomenon => BCPhenomenon,
        }
    }

    fn map_morphism(m: &AcousticRelation) -> BoneCondRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind {
            AcousticsCategoryRelationKind::Identity => BoneConductionCategory::identity(&from),
            _ => BoneCondRelation {
                from,
                to,
                kind: BoneConductionCategoryRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(AcousticsToBoneConduction);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::category::{Category, Entity};
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<AcousticsToBoneConduction>().unwrap();
    }

    #[test]
    fn test_analogy_validates() {
        Analogy::<AcousticsToBoneConduction>::validate().unwrap();
    }

    #[test]
    fn test_identity_preservation() {
        for obj in AcousticEntity::variants() {
            let id_src = AcousticsCategory::identity(&obj);
            let mapped_id = AcousticsToBoneConduction::map_morphism(&id_src);
            let id_tgt =
                BoneConductionCategory::identity(&AcousticsToBoneConduction::map_object(&obj));
            assert_eq!(mapped_id, id_tgt, "identity law failed for {:?}", obj);
        }
    }

    #[test]
    fn test_composition_preservation() {
        let objs = AcousticEntity::variants();
        for &a in &objs[..5] {
            for &b in &objs[5..10] {
                for &c in &objs[10..15] {
                    let f = AcousticRelation {
                        from: a,
                        to: b,
                        kind: AcousticsCategoryRelationKind::Composed,
                    };
                    let g = AcousticRelation {
                        from: b,
                        to: c,
                        kind: AcousticsCategoryRelationKind::Composed,
                    };
                    let composed = AcousticsCategory::compose(&f, &g).unwrap();
                    let mapped_composed = AcousticsToBoneConduction::map_morphism(&composed);
                    let composed_mapped = BoneConductionCategory::compose(
                        &AcousticsToBoneConduction::map_morphism(&f),
                        &AcousticsToBoneConduction::map_morphism(&g),
                    )
                    .unwrap();
                    assert_eq!(
                        mapped_composed, composed_mapped,
                        "composition law failed for {:?} -> {:?} -> {:?}",
                        a, b, c
                    );
                }
            }
        }
    }

    #[test]
    fn test_sound_wave_maps_to_skull_vibration() {
        assert_eq!(
            AcousticsToBoneConduction::map_object(&AcousticEntity::SoundWave),
            BoneCondEntity::SkullVibration,
        );
    }

    #[test]
    fn test_cortical_bone_maps_to_mastoid() {
        assert_eq!(
            AcousticsToBoneConduction::map_object(&AcousticEntity::CorticalBone),
            BoneCondEntity::Mastoid,
        );
    }

    #[test]
    fn test_resonance_maps_to_skull_resonance() {
        assert_eq!(
            AcousticsToBoneConduction::map_object(&AcousticEntity::Resonance),
            BoneCondEntity::SkullResonance,
        );
    }

    #[test]
    fn test_impedance_mismatch_maps_to_transcranial_attenuation() {
        assert_eq!(
            AcousticsToBoneConduction::map_object(&AcousticEntity::ImpedanceMismatch),
            BoneCondEntity::TranscranialAttenuation,
        );
    }

    #[test]
    fn test_every_entity_maps_to_valid_target() {
        let target_variants = BoneCondEntity::variants();
        for obj in AcousticEntity::variants() {
            let mapped = AcousticsToBoneConduction::map_object(&obj);
            assert!(
                target_variants.contains(&mapped),
                "{:?} mapped to {:?} which is not a valid BoneCondEntity",
                obj,
                mapped
            );
        }
    }

    use proptest::prelude::*;

    fn arb_acoustic_entity() -> impl Strategy<Value = AcousticEntity> {
        (0..AcousticEntity::variants().len()).prop_map(|i| AcousticEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_acoustic_entity()) {
            let mapped = AcousticsToBoneConduction::map_object(&entity);
            prop_assert!(BoneCondEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_acoustic_entity()) {
            let id_src = AcousticsCategory::identity(&entity);
            let mapped_id = AcousticsToBoneConduction::map_morphism(&id_src);
            let id_tgt = BoneConductionCategory::identity(&AcousticsToBoneConduction::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}
