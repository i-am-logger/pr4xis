//! Functor: AcousticsCategory -> EnvironmentalAcousticsCategory.
//!
//! Maps acoustic physics to applied environmental/room acoustics.

use crate::natural::hearing::acoustics::ontology::*;
use crate::natural::hearing::environmental_acoustics::ontology::*;
use pr4xis::category::{Category, Functor, Relationship};

pub struct AcousticsToEnvironment;

impl Functor for AcousticsToEnvironment {
    type Source = AcousticsCategory;
    type Target = EnvironmentalAcousticsCategory;

    fn map_object(obj: &AcousticEntity) -> EnvironmentEntity {
        use AcousticEntity as A;
        use EnvironmentEntity::*;
        match obj {
            // Wave properties → noise measurement
            A::Frequency | A::Amplitude | A::Intensity | A::WaveProperty => SoundPressureLevel,
            A::Wavelength | A::Phase => AWeighting,
            // Wave types → SPL measurement
            A::SoundWave | A::LongitudinalWave | A::TransverseWave | A::ShearWave | A::Wave => {
                SoundPressureLevel
            }
            // Media → acoustic properties
            A::Air | A::Water | A::SoftTissue | A::Cartilage | A::Fluid | A::Medium => {
                SoundAbsorption
            }
            A::CorticalBone | A::CancellousBone | A::Solid | A::BoneTissue => SoundInsulation,
            // Phenomena → room parameters
            A::Resonance => ReverberationTime,
            A::Reflection => EarlyDecayTime,
            A::Refraction => LateralFraction,
            A::Diffraction => SoundDiffusion,
            A::Absorption => AbsorptionCoefficient,
            A::Attenuation => TransmissionLoss,
            A::ImpedanceMismatch => FlankingTransmission,
            A::AcousticPhenomenon => RoomParameter,
        }
    }

    fn map_morphism(m: &AcousticRelation) -> EnvironmentRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind {
            AcousticsCategoryRelationKind::Identity => {
                EnvironmentalAcousticsCategory::identity(&from)
            }
            _ => EnvironmentRelation {
                from,
                to,
                kind: EnvironmentalAcousticsCategoryRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(AcousticsToEnvironment);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<AcousticsToEnvironment>().unwrap();
    }
    #[test]
    fn test_analogy_validates() {
        Analogy::<AcousticsToEnvironment>::validate().unwrap();
    }
    #[test]
    fn test_resonance_maps_to_rt() {
        assert_eq!(
            AcousticsToEnvironment::map_object(&AcousticEntity::Resonance),
            EnvironmentEntity::ReverberationTime
        );
    }
    #[test]
    fn test_every_entity_maps_valid() {
        let targets = EnvironmentEntity::variants();
        for obj in AcousticEntity::variants() {
            assert!(targets.contains(&AcousticsToEnvironment::map_object(&obj)));
        }
    }

    use pr4xis::category::Category;
    use proptest::prelude::*;

    fn arb_acoustic_entity() -> impl Strategy<Value = AcousticEntity> {
        (0..AcousticEntity::variants().len()).prop_map(|i| AcousticEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_acoustic_entity()) {
            let mapped = AcousticsToEnvironment::map_object(&entity);
            prop_assert!(EnvironmentEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_acoustic_entity()) {
            let id_src = AcousticsCategory::identity(&entity);
            let mapped_id = AcousticsToEnvironment::map_morphism(&id_src);
            let id_tgt = EnvironmentalAcousticsCategory::identity(&AcousticsToEnvironment::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}
