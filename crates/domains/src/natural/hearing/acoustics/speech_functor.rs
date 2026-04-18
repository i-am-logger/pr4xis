//! Functor: AcousticsCategory -> SpeechCategory.
//!
//! Maps acoustic physics to speech production parameters.

use crate::natural::hearing::acoustics::ontology::*;
use crate::natural::hearing::speech::ontology::*;
use pr4xis::category::{Category, Functor, Relationship};

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
        }
    }

    fn map_morphism(m: &AcousticRelation) -> SpeechRelation {
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        match m.kind {
            AcousticsCategoryRelationKind::Identity => SpeechCategory::identity(&from),
            _ => SpeechRelation {
                from,
                to,
                kind: SpeechCategoryRelationKind::Composed,
            },
        }
    }
}
pr4xis::register_functor!(AcousticsToSpeech);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<AcousticsToSpeech>().unwrap();
    }
    #[test]
    fn test_analogy_validates() {
        Analogy::<AcousticsToSpeech>::validate().unwrap();
    }
    #[test]
    fn test_resonance_maps_to_formant() {
        assert_eq!(
            AcousticsToSpeech::map_object(&AcousticEntity::Resonance),
            SpeechEntity::Formant
        );
    }
    #[test]
    fn test_every_entity_maps_valid() {
        let targets = SpeechEntity::variants();
        for obj in AcousticEntity::variants() {
            assert!(targets.contains(&AcousticsToSpeech::map_object(&obj)));
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
            let mapped = AcousticsToSpeech::map_object(&entity);
            prop_assert!(SpeechEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_acoustic_entity()) {
            let id_src = AcousticsCategory::identity(&entity);
            let mapped_id = AcousticsToSpeech::map_morphism(&id_src);
            let id_tgt = SpeechCategory::identity(&AcousticsToSpeech::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}
