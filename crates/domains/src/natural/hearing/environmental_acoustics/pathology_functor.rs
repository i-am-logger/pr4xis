//! Functor: EnvironmentalAcousticsCategory → PathologyCategory.
//!
//! Maps noise exposure and environmental conditions to hearing damage.
//!
//! Citation: NIOSH (1998) *Occupational Noise Exposure Recommended
//! Criteria*; Henderson et al. (2006) *Ear & Hearing* 27(1):1 — noise
//! exposure to pathology pathway.

use crate::natural::hearing::environmental_acoustics::ontology::*;
use crate::natural::hearing::pathology::ontology::*;
use pr4xis::category::{Arrow, Functor};

pub struct EnvironmentToPathology;

impl Functor for EnvironmentToPathology {
    type Source = EnvironmentalAcousticsCategory;
    type Target = PathologyCategory;

    fn map_object(obj: &EnvironmentEntity) -> PathologyEntity {
        use EnvironmentEntity as E;
        use PathologyEntity::*;
        match obj {
            E::SoundPressureLevel
            | E::EquivalentContinuousLevel
            | E::PeakSoundLevel
            | E::SoundExposureLevel
            | E::NoiseMeasure => ElevatedThreshold,
            E::AWeighting | E::CWeighting => NoiseInducedHearingLoss,
            E::NoiseDose | E::TimeWeightedAverage => HairCellLoss,
            E::OSHALimit
            | E::NIOSHLimit
            | E::ExchangeRate
            | E::PermissibleExposureLimit
            | E::ActionLevel
            | E::NoiseStandard => NoiseInducedHearingLoss,
            E::ReverberationTime
            | E::RT60
            | E::EarlyDecayTime
            | E::Clarity
            | E::Definition
            | E::SpeechTransmissionIndex
            | E::CenterTime
            | E::LateralFraction
            | E::RoomParameter => PoorSpeechInNoise,
            E::SoundAbsorption
            | E::AbsorptionCoefficient
            | E::SoundDiffusion
            | E::AcousticProperty => PoorSpeechInNoise,
            E::SoundInsulation | E::TransmissionLoss | E::FlankingTransmission => ElevatedThreshold,
            E::Soundscape
            | E::Keynote
            | E::SoundSignal
            | E::Soundmark
            | E::BackgroundNoise
            | E::SoundscapeElement => Presbycusis,
            E::SpeechRoom | E::MusicHall | E::WorshipSpace | E::RoomType => PoorSpeechInNoise,
            E::SoundLevelMeter | E::Dosimeter | E::CalibrationSource | E::MeasurementDevice => {
                Audiogram
            }
            // Environmental events → pathology events.
            E::NoiseSourceEvent => NoiseExposure,
            E::SoundPropagation => NoiseExposure,
            E::WorkerExposure => NoiseExposure,
            E::DoseAccumulation => OHCDamage,
            E::ThresholdShift => ThresholdShift,
            E::HearingDamageRisk => CommunicationDifficulty,
            E::RoomReverberation => TemporalSmearing,
            E::SpeechIntelligibilityReduction => CommunicationDifficulty,
            E::EnvironmentEvent => PathologyEvent,
        }
    }

    fn map_morphism(m: &EnvironmentRelation) -> PathologyRelation {
        use EnvironmentalAcousticsCategoryRelationKind as Sk;
        use PathologyRelationKind as Tk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
            // Canonical kinds always emitted by ontology! macro; unreachable
            // when source has no edges of these kinds.
            Sk::Parthood => Tk::Parthood,
        };
        PathologyRelation { from, to, kind }
    }
}
pr4xis::register_functor!(EnvironmentToPathology);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;
    use pr4xis::category::laws::assert_functor_laws;

    #[test]
    fn functor_laws() {
        assert_functor_laws::<EnvironmentToPathology>();
    }
    #[test]
    fn noise_dose_maps_to_hair_cell_loss() {
        assert_eq!(
            EnvironmentToPathology::map_object(&EnvironmentEntity::NoiseDose),
            PathologyEntity::HairCellLoss
        );
    }
    #[test]
    fn every_entity_maps_valid() {
        let targets = PathologyEntity::variants();
        for obj in EnvironmentEntity::variants() {
            assert!(targets.contains(&EnvironmentToPathology::map_object(&obj)));
        }
    }
}
