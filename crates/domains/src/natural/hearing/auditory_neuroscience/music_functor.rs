//! Functor: NeuroscienceCategory -> MusicPerceptionCategory.
//!
//! Maps neural processing mechanisms to music cognition.

use crate::natural::hearing::auditory_neuroscience::ontology::*;
use crate::natural::hearing::music_perception::ontology::*;
use pr4xis::category::{Functor, Relationship};

pub struct NeuroscienceToMusic;

impl Functor for NeuroscienceToMusic {
    type Source = NeuroscienceCategory;
    type Target = MusicPerceptionCategory;

    fn map_object(obj: &NeuralEntity) -> MusicEntity {
        use MusicEntity::*;
        use NeuralEntity as N;
        match obj {
            // Coding → pitch/rhythm
            N::RateCoding => PitchHeight,
            N::TemporalCoding | N::PhaseLocking | N::SpikeTimingCode => TemporalExpectation,
            N::PlaceCoding => PitchChroma,
            N::PopulationCoding => Chord,
            // Response properties → musical features
            N::TonotopicMap | N::CharacteristicFrequency => KeySense,
            N::FrequencyTuningCurve => IntervalPerception,
            N::RateLevelFunction | N::DynamicRange => MusicalEmotion,
            N::SpontaneousRate => Groove,
            N::OnsetResponse => AttackTime,
            N::SustainedResponse => Tonality,
            N::Adaptation => MusicalExpectation,
            N::Inhibition => Dissonance,
            // Processing stages → increasingly abstract music processing
            N::AuditoryNerveFiber => PitchHeight,
            N::CochlearNucleusProcessing => IntervalPerception,
            N::SuperiorOliveProcessing => Beat,
            N::LateralLemniscus => Meter,
            N::InferiorColliculusProcessing => Entrainment,
            N::MedialGeniculateProcessing => Tonality,
            N::AuditoryCortexProcessing => MusicalEmotion,
            // Binaural → spatial aspects of music
            N::BinauralProcessing
            | N::CoincidenceDetection
            | N::ExcitatoryInhibitory
            | N::MedialSuperiorOlive
            | N::LateralSuperiorOlive => Groove,
            // Higher functions → music cognition
            N::AuditorySceneAnalysis => InstrumentIdentification,
            N::StreamSegregation => MelodicContour,
            N::GestaltGrouping => RhythmicPercept,
            N::EchoSuppression | N::PrecedenceEffect => Groove,
            N::MismatchNegativity => Surprise,
            // Abstract
            N::CodingStrategy => PitchPercept,
            N::ResponseProperty => TimbrePercept,
            N::ProcessingStage => HarmonicPercept,
            N::BinauralMechanism => RhythmicPercept,
            N::HigherFunction => AffectiveResponse,
        }
    }

    fn map_morphism(m: &NeuralRelation) -> MusicRelation {
        MusicRelation {
            from: Self::map_object(&m.source()),
            to: Self::map_object(&m.target()),
        }
    }
}
pr4xis::register_functor!(NeuroscienceToMusic);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;
    use pr4xis::category::validate::check_functor_laws;
    use pr4xis::ontology::reasoning::analogy::Analogy;

    #[test]
    fn test_functor_laws() {
        check_functor_laws::<NeuroscienceToMusic>().unwrap();
    }
    #[test]
    fn test_analogy_validates() {
        Analogy::<NeuroscienceToMusic>::validate().unwrap();
    }
    #[test]
    fn test_mismatch_negativity_maps_to_surprise() {
        assert_eq!(
            NeuroscienceToMusic::map_object(&NeuralEntity::MismatchNegativity),
            MusicEntity::Surprise
        );
    }
    #[test]
    fn test_every_entity_maps_valid() {
        let targets = MusicEntity::variants();
        for obj in NeuralEntity::variants() {
            assert!(targets.contains(&NeuroscienceToMusic::map_object(&obj)));
        }
    }

    use pr4xis::category::Category;
    use proptest::prelude::*;

    fn arb_neural_entity() -> impl Strategy<Value = NeuralEntity> {
        (0..NeuralEntity::variants().len()).prop_map(|i| NeuralEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_functor_maps_to_valid_target(entity in arb_neural_entity()) {
            let mapped = NeuroscienceToMusic::map_object(&entity);
            prop_assert!(MusicEntity::variants().contains(&mapped));
        }

        #[test]
        fn prop_functor_preserves_identity(entity in arb_neural_entity()) {
            let id_src = NeuroscienceCategory::identity(&entity);
            let mapped_id = NeuroscienceToMusic::map_morphism(&id_src);
            let id_tgt = MusicPerceptionCategory::identity(&NeuroscienceToMusic::map_object(&entity));
            prop_assert_eq!(mapped_id, id_tgt);
        }
    }
}
