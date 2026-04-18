//! Functor: EnvironmentalAcousticsCategory -> PathologyCategory.
//!
//! Maps noise exposure and environmental conditions to hearing damage.

use crate::natural::hearing::environmental_acoustics::ontology::*;
use crate::natural::hearing::pathology::ontology::*;
use pr4xis::category::{Category, Functor, Relationship};

pub struct EnvironmentToPathology;

impl Functor for EnvironmentToPathology {
    type Source = EnvironmentalAcousticsCategory;
    type Target = PathologyCategory;

    fn map_object(obj: &EnvironmentEntity) -> PathologyEntity {
        use EnvironmentEntity as E;
        use PathologyEntity::*;
        match obj {
            // Noise measurement → threshold shift
            E::SoundPressureLevel
            | E::EquivalentContinuousLevel
            | E::PeakSoundLevel
            | E::SoundExposureLevel
            | E::NoiseMeasure => ElevatedThreshold,
            // Weightings → frequency-specific damage
            E::AWeighting | E::CWeighting => NoiseInducedHearingLoss,
            // Dose → damage mechanism
            E::NoiseDose | E::TimeWeightedAverage => HairCellLoss,
            // Standards → NIHL
            E::OSHALimit
            | E::NIOSHLimit
            | E::ExchangeRate
            | E::PermissibleExposureLimit
            | E::ActionLevel
            | E::NoiseStandard => NoiseInducedHearingLoss,
            // Room parameters → speech-in-noise difficulty
            E::ReverberationTime
            | E::RT60
            | E::EarlyDecayTime
            | E::Clarity
            | E::Definition
            | E::SpeechTransmissionIndex
            | E::CenterTime
            | E::LateralFraction
            | E::RoomParameter => PoorSpeechInNoise,
            // Acoustic properties
            E::SoundAbsorption
            | E::AbsorptionCoefficient
            | E::SoundDiffusion
            | E::AcousticProperty => PoorSpeechInNoise,
            E::SoundInsulation | E::TransmissionLoss | E::FlankingTransmission => ElevatedThreshold,
            // Soundscape → chronic exposure
            E::Soundscape
            | E::Keynote
            | E::SoundSignal
            | E::Soundmark
            | E::BackgroundNoise
            | E::SoundscapeElement => Presbycusis,
            // Room types → speech-in-noise difficulty
            E::SpeechRoom | E::MusicHall | E::WorshipSpace | E::RoomType => PoorSpeechInNoise,
            // Equipment → clinical measures
            E::SoundLevelMeter | E::Dosimeter | E::CalibrationSource | E::MeasurementDevice => {
                Audiogram
            }
        }
    }

    fn map_morphism(m: &EnvironmentRelation) -> PathologyRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind {
            EnvironmentalAcousticsCategoryRelationKind::Identity => {
                PathologyCategory::identity(&from)
            }
            _ => PathologyRelation {
                from,
                to,
                kind: PathologyCategoryRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(EnvironmentToPathology);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<EnvironmentToPathology>().unwrap();
    }
    #[test]
    fn test_analogy_validates() {
        Analogy::<EnvironmentToPathology>::validate().unwrap();
    }
    #[test]
    fn test_noise_dose_maps_to_hair_cell_loss() {
        assert_eq!(
            EnvironmentToPathology::map_object(&EnvironmentEntity::NoiseDose),
            PathologyEntity::HairCellLoss
        );
    }
    #[test]
    fn test_every_entity_maps_valid() {
        let targets = PathologyEntity::variants();
        for obj in EnvironmentEntity::variants() {
            assert!(targets.contains(&EnvironmentToPathology::map_object(&obj)));
        }
    }

    use pr4xis::category::Category;
    use proptest::prelude::*;

    fn arb_environment_entity() -> impl Strategy<Value = EnvironmentEntity> {
        (0..EnvironmentEntity::variants().len()).prop_map(|i| EnvironmentEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_environment_entity()) {
            let mapped = EnvironmentToPathology::map_object(&entity);
            prop_assert!(PathologyEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_environment_entity()) {
            let id_src = EnvironmentalAcousticsCategory::identity(&entity);
            let mapped_id = EnvironmentToPathology::map_morphism(&id_src);
            let id_tgt = PathologyCategory::identity(&EnvironmentToPathology::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}
