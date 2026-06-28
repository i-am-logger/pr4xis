//! Functor: AcousticsCategory → EnvironmentalAcousticsCategory.
//!
//! Maps acoustic physics to applied environmental / room acoustics.
//!
//! Citation: Kuttruff (2009) *Room Acoustics* and Kinsler et al. (2000)
//! *Fundamentals of Acoustics* — the acoustic-to-room-parameter
//! correspondences are standard textbook content.

use crate::natural::hearing::acoustics::ontology::*;
use crate::natural::hearing::environmental_acoustics::ontology::*;
use pr4xis::category::{Arrow, Functor};

pub struct AcousticsToEnvironment;

impl Functor for AcousticsToEnvironment {
    type Source = AcousticsCategory;
    type Target = EnvironmentalAcousticsCategory;

    fn map_object(obj: &AcousticEntity) -> EnvironmentEntity {
        use AcousticEntity as A;
        use EnvironmentEntity::*;
        match obj {
            A::Frequency | A::Amplitude | A::Intensity | A::WaveProperty => SoundPressureLevel,
            A::Wavelength | A::Phase => AWeighting,
            A::SoundWave | A::LongitudinalWave | A::TransverseWave | A::ShearWave | A::Wave => {
                SoundPressureLevel
            }
            A::Air | A::Water | A::SoftTissue | A::Cartilage | A::Fluid | A::Medium => {
                SoundAbsorption
            }
            A::CorticalBone | A::CancellousBone | A::Solid | A::BoneTissue => SoundInsulation,
            A::Resonance => ReverberationTime,
            A::Reflection => EarlyDecayTime,
            A::Refraction => LateralFraction,
            A::Diffraction => SoundDiffusion,
            A::Absorption => AbsorptionCoefficient,
            A::Attenuation => TransmissionLoss,
            A::ImpedanceMismatch => FlankingTransmission,
            A::AcousticPhenomenon => RoomParameter,
            // Acoustic events → environmental events.
            A::SourceVibration => NoiseSourceEvent,
            A::MediumCoupling | A::WavePropagation => SoundPropagation,
            A::BoundaryEncounter
            | A::ImpedanceTransition
            | A::EnergyReflection
            | A::EnergyTransmission
            | A::EnergyAbsorption => RoomReverberation,
            A::WaveAttenuation => DoseAccumulation,
            A::ResonantAmplification => WorkerExposure,
            A::ReceiverExcitation => HearingDamageRisk,
            A::AcousticEvent => EnvironmentEvent,
        }
    }

    fn map_morphism(m: &AcousticRelation) -> EnvironmentRelation {
        use AcousticsCategoryRelationKind as Sk;
        use EnvironmentRelationKind as Tk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Subsumption, // env has no Parthood — collapse
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        EnvironmentRelation { from, to, kind }
    }
}
pr4xis::register_functor!(AcousticsToEnvironment);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws() {
        assert_functor_laws::<AcousticsToEnvironment>();
    }
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn resonance_maps_to_rt() {
        assert_eq!(
            AcousticsToEnvironment::map_object(&AcousticEntity::Resonance),
            EnvironmentEntity::ReverberationTime
        );
    }
    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn every_entity_maps_valid() {
        let targets = EnvironmentEntity::variants();
        for obj in AcousticEntity::variants() {
            assert!(targets.contains(&AcousticsToEnvironment::map_object(&obj)));
        }
    }
}
