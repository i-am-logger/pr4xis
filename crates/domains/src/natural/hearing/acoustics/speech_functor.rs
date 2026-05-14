//! Functor: AcousticsCategory → SpeechCategory.
//!
//! Maps acoustic physics to speech production parameters.
//!
//! Citation: Fant (1960) *Acoustic Theory of Speech Production* —
//! source-filter model grounds these correspondences.

use crate::natural::hearing::acoustics::ontology::*;
use crate::natural::hearing::speech::ontology::*;
use pr4xis::category::{Arrow, Functor};

pub struct AcousticsToSpeech;

impl Functor for AcousticsToSpeech {
    type Source = AcousticsCategory;
    type Target = SpeechCategory;

    fn map_object(obj: &AcousticEntity) -> SpeechEntity {
        use AcousticEntity as A;
        use SpeechEntity::*;
        match obj {
            A::Frequency | A::WaveProperty => FundamentalFrequency,
            A::Amplitude | A::Intensity => SignalToNoiseRatio,
            A::Wavelength | A::Phase => Harmonics,
            A::SoundWave | A::LongitudinalWave | A::Wave => Phoneme,
            A::TransverseWave | A::ShearWave => Phoneme,
            A::Air | A::Fluid | A::Medium => Vowel,
            A::Water | A::SoftTissue | A::Cartilage => Consonant,
            A::CorticalBone | A::CancellousBone | A::Solid | A::BoneTissue => Consonant,
            A::Resonance => Formant,
            A::Reflection => SpectralTilt,
            A::Refraction | A::Diffraction => Intonation,
            A::Absorption | A::Attenuation => SignalToNoiseRatio,
            A::ImpedanceMismatch => VoiceOnsetTime,
            A::AcousticPhenomenon => AcousticParameter,
            // Acoustic events → speech-production events.
            A::SourceVibration => VocalFoldVibration,
            A::MediumCoupling => GlottalPulse,
            A::WavePropagation => AcousticRadiation,
            A::BoundaryEncounter | A::ImpedanceTransition => VocalTractFiltering,
            A::EnergyReflection => FormantTransition,
            A::EnergyTransmission => FormantProduction,
            A::EnergyAbsorption => CoarticulationEffect,
            A::WaveAttenuation => CoarticulationEffect,
            A::ResonantAmplification => FormantProduction,
            A::ReceiverExcitation => ListenerPerception,
            A::AcousticEvent => SpeechEvent,
        }
    }

    fn map_morphism(m: &AcousticRelation) -> SpeechRelation {
        use AcousticsCategoryRelationKind as Sk;
        use SpeechRelationKind as Tk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Parthood => Tk::Parthood,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
        };
        SpeechRelation { from, to, kind }
    }
}
pr4xis::register_functor!(AcousticsToSpeech);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;
    use pr4xis::category::laws::assert_functor_laws;

    #[test]
    fn functor_laws() {
        assert_functor_laws::<AcousticsToSpeech>();
    }
    #[test]
    fn resonance_maps_to_formant() {
        assert_eq!(
            AcousticsToSpeech::map_object(&AcousticEntity::Resonance),
            SpeechEntity::Formant
        );
    }
    #[test]
    fn every_entity_maps_valid() {
        let targets = SpeechEntity::variants();
        for obj in AcousticEntity::variants() {
            assert!(targets.contains(&AcousticsToSpeech::map_object(&obj)));
        }
    }
}
