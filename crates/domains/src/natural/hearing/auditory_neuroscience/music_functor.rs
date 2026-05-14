//! Functor: NeuroscienceCategory → MusicPerceptionCategory.
//!
//! Maps neural processing mechanisms to music cognition.
//!
//! Citation: Schnupp et al. (2011) *Auditory Neuroscience* — neural-to-
//! music correspondences; Patel (2008) *Music, Language, and the Brain*.

use crate::natural::hearing::auditory_neuroscience::ontology::*;
use crate::natural::hearing::music_perception::ontology::*;
use pr4xis::category::{Arrow, Functor};

pub struct NeuroscienceToMusic;

impl Functor for NeuroscienceToMusic {
    type Source = NeuroscienceCategory;
    type Target = MusicPerceptionCategory;

    fn map_object(obj: &NeuralEntity) -> MusicEntity {
        use MusicEntity::*;
        use NeuralEntity as N;
        match obj {
            N::RateCoding => PitchHeight,
            N::TemporalCoding | N::PhaseLocking | N::SpikeTimingCode => TemporalExpectation,
            N::PlaceCoding => PitchChroma,
            N::PopulationCoding => Chord,
            N::TonotopicMap | N::CharacteristicFrequency => KeySense,
            N::FrequencyTuningCurve => IntervalPerception,
            N::RateLevelFunction | N::DynamicRange => MusicalEmotion,
            N::SpontaneousRate => Groove,
            N::OnsetResponse => AttackTime,
            N::SustainedResponse => Tonality,
            N::Adaptation => MusicalExpectation,
            N::Inhibition => Dissonance,
            N::AuditoryNerveFiber => PitchHeight,
            N::CochlearNucleusProcessing => IntervalPerception,
            N::SuperiorOliveProcessing => Beat,
            N::LateralLemniscus => Meter,
            N::InferiorColliculusProcessing => Entrainment,
            N::MedialGeniculateProcessing => Tonality,
            N::AuditoryCortexProcessing => MusicalEmotion,
            N::BinauralProcessing
            | N::CoincidenceDetection
            | N::ExcitatoryInhibitory
            | N::MedialSuperiorOlive
            | N::LateralSuperiorOlive => Groove,
            N::AuditorySceneAnalysis => InstrumentIdentification,
            N::StreamSegregation => MelodicContour,
            N::GestaltGrouping => RhythmicPercept,
            N::EchoSuppression | N::PrecedenceEffect => Groove,
            N::MismatchNegativity => Surprise,
            N::CodingStrategy => PitchPercept,
            N::ResponseProperty => TimbrePercept,
            N::ProcessingStage => HarmonicPercept,
            N::BinauralMechanism => RhythmicPercept,
            N::HigherFunction => AffectiveResponse,
            // Neural events → music events.
            N::AuditoryNerveInput => AuditoryInput,
            N::CochlearNucleusIntegration => PitchExtraction,
            N::BinauralConvergence => HarmonicGrouping,
            N::LemniscalRelay => MelodicTracking,
            N::MultisensoryIntegration => BeatInduction,
            N::ThalamicGating => MetricFraming,
            N::CorticalAnalysis => TonalInterpretation,
            N::StreamFormation => MusicalExpectationFormation,
            N::PerceptualBinding => EmotionalResponse,
            N::NeuralEvent => MusicEvent,
        }
    }

    fn map_morphism(m: &NeuralRelation) -> MusicRelation {
        use MusicRelationKind as Tk;
        use NeuroscienceCategoryRelationKind as Sk;
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
        MusicRelation { from, to, kind }
    }
}
pr4xis::register_functor!(NeuroscienceToMusic);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;
    use pr4xis::category::laws::assert_functor_laws;

    #[test]
    fn functor_laws() {
        assert_functor_laws::<NeuroscienceToMusic>();
    }
    #[test]
    fn mismatch_negativity_maps_to_surprise() {
        assert_eq!(
            NeuroscienceToMusic::map_object(&NeuralEntity::MismatchNegativity),
            MusicEntity::Surprise
        );
    }
    #[test]
    fn every_entity_maps_valid() {
        let targets = MusicEntity::variants();
        for obj in NeuralEntity::variants() {
            assert!(targets.contains(&NeuroscienceToMusic::map_object(&obj)));
        }
    }
}
