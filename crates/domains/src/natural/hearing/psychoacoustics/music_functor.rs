//! Functor: PsychoacousticsCategory -> MusicPerceptionCategory.
//!
//! Maps low-level auditory percepts to their higher musical roles.
//! Pitch → melodic/harmonic percept, loudness → dynamics, etc.

use crate::natural::hearing::music_perception::ontology::*;
use crate::natural::hearing::psychoacoustics::ontology::*;
use pr4xis::category::{Functor, Relationship};

pub struct PsychoacousticsToMusic;

impl Functor for PsychoacousticsToMusic {
    type Source = PsychoacousticsCategory;
    type Target = MusicPerceptionCategory;

    fn map_object(obj: &PsychoacousticEntity) -> MusicEntity {
        use MusicEntity::*;
        use PsychoacousticEntity as P;
        match obj {
            // Pitch-related → musical pitch
            P::Pitch | P::PlacePitch | P::TemporalPitch | P::VirtualPitch | P::Octave => {
                PitchHeight
            }
            // Loudness → musical dynamics (tension/emotion)
            P::Loudness
            | P::Phon
            | P::Sone
            | P::EqualLoudnessContour
            | P::LoudnessRecruitment
            | P::LoudnessMetric => MusicalEmotion,
            // Timbre → instrument identification
            P::Timbre => InstrumentIdentification,
            // Duration → rhythm
            P::Duration => Beat,
            // Masking → stream segregation (auditory scene)
            P::SimultaneousMasking
            | P::ForwardMasking
            | P::BackwardMasking
            | P::InformationalMasking
            | P::MaskingType => Consonance,
            // Frequency analysis → harmonic series perception
            P::CriticalBand
            | P::BarkScale
            | P::ERBScale
            | P::AuditoryFilter
            | P::FrequencySelectivity => RoughnessModel,
            // Temporal → rhythm/entrainment
            P::TemporalResolution
            | P::GapDetection
            | P::TemporalIntegration
            | P::TemporalMeasure => Entrainment,
            // Spatial → not directly musical, map to groove (spatial in music)
            P::SoundLocalization
            | P::InterauralTimeDifference
            | P::InterauralLevelDifference
            | P::HeadRelatedTransferFunction
            | P::SpatialCue => Groove,
            // Thresholds → expectation
            P::AbsoluteThreshold | P::DifferentialThreshold | P::JustNoticeableDifference => {
                MusicalExpectation
            }
            // Perceptual dimension → pitch percept
            P::PerceptualDimension | P::PitchMechanism => PitchPercept,
        }
    }

    fn map_morphism(m: &PsychoacousticRelation) -> MusicRelation {
        MusicRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<PsychoacousticsToMusic>().unwrap();
    }
    #[test]
    fn test_analogy_validates() {
        Analogy::<PsychoacousticsToMusic>::validate().unwrap();
    }
    #[test]
    fn test_pitch_maps_to_pitch_height() {
        assert_eq!(
            PsychoacousticsToMusic::map_object(&PsychoacousticEntity::Pitch),
            MusicEntity::PitchHeight
        );
    }
    #[test]
    fn test_every_entity_maps_valid() {
        let targets = MusicEntity::variants();
        for obj in PsychoacousticEntity::variants() {
            assert!(targets.contains(&PsychoacousticsToMusic::map_object(&obj)));
        }
    }

    use pr4xis::category::Category;
    use proptest::prelude::*;

    fn arb_psychoacoustic_entity() -> impl Strategy<Value = PsychoacousticEntity> {
        (0..PsychoacousticEntity::variants().len())
            .prop_map(|i| PsychoacousticEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_psychoacoustic_entity()) {
            let mapped = PsychoacousticsToMusic::map_object(&entity);
            prop_assert!(MusicEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_psychoacoustic_entity()) {
            let id_src = PsychoacousticsCategory::identity(&entity);
            let mapped_id = PsychoacousticsToMusic::map_morphism(&id_src);
            let id_tgt = MusicPerceptionCategory::identity(&PsychoacousticsToMusic::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}
