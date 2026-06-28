//! Functor: PsychoacousticsCategory → MusicPerceptionCategory.
//!
//! Maps low-level auditory percepts to their higher musical roles.
//!
//! Citation: McDermott & Oxenham (2008) *Curr. Opin. Neurobiol.* 18(4):452
//! — psychoacoustics-to-music perception correspondences.

use crate::natural::hearing::music_perception::ontology::*;
use crate::natural::hearing::psychoacoustics::ontology::*;
use pr4xis::category::{Arrow, Functor};

pub struct PsychoacousticsToMusic;

impl Functor for PsychoacousticsToMusic {
    type Source = PsychoacousticsCategory;
    type Target = MusicPerceptionCategory;

    fn map_object(obj: &PsychoacousticEntity) -> MusicEntity {
        use MusicEntity::*;
        use PsychoacousticEntity as P;
        match obj {
            P::Pitch | P::PlacePitch | P::TemporalPitch | P::VirtualPitch | P::Octave => {
                PitchHeight
            }
            P::Loudness
            | P::Phon
            | P::Sone
            | P::EqualLoudnessContour
            | P::LoudnessRecruitment
            | P::LoudnessMetric => MusicalEmotion,
            P::Timbre => InstrumentIdentification,
            P::Duration => Beat,
            P::SimultaneousMasking
            | P::ForwardMasking
            | P::BackwardMasking
            | P::InformationalMasking
            | P::MaskingType => Consonance,
            P::CriticalBand
            | P::BarkScale
            | P::ERBScale
            | P::AuditoryFilter
            | P::FrequencySelectivity => RoughnessModel,
            P::TemporalResolution
            | P::GapDetection
            | P::TemporalIntegration
            | P::TemporalMeasure => Entrainment,
            P::SoundLocalization
            | P::InterauralTimeDifference
            | P::InterauralLevelDifference
            | P::HeadRelatedTransferFunction
            | P::SpatialCue => Groove,
            P::AbsoluteThreshold | P::DifferentialThreshold | P::JustNoticeableDifference => {
                MusicalExpectation
            }
            P::PerceptualDimension | P::PitchMechanism => PitchPercept,
            // Psychoacoustic events → music events.
            P::AcousticStimulus => AuditoryInput,
            P::CochlearFiltering => PitchExtraction,
            P::NeuralTransduction => OnsetDetection,
            P::BrainstemProcessing => HarmonicGrouping,
            P::CorticalAnalysis => TonalInterpretation,
            P::PerceptFormation => MusicalExpectationFormation,
            P::AwareExperience => EmotionalResponse,
            P::FrequencyAnalysis => HarmonicGrouping,
            P::PitchExtraction => PitchExtraction,
            P::PsychoacousticEvent => MusicEvent,
        }
    }

    fn map_morphism(m: &PsychoacousticRelation) -> MusicRelation {
        use MusicRelationKind as Tk;
        use PsychoacousticsCategoryRelationKind as Sk;
        let from = Self::map_object(&m.source());
        let to = Self::map_object(&m.target());
        let kind = match m.kind {
            Sk::Identity => Tk::Identity,
            Sk::Subsumption => Tk::Subsumption,
            Sk::Causation => Tk::Causation,
            Sk::Opposition => Tk::Opposition,
            Sk::Parthood => Tk::Parthood,
        };
        MusicRelation { from, to, kind }
    }
}
pr4xis::register_functor!(PsychoacousticsToMusic);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_functor_laws;

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn functor_laws() {
        assert_functor_laws::<PsychoacousticsToMusic>();
    }
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn pitch_maps_to_pitch_height() {
        assert_eq!(
            PsychoacousticsToMusic::map_object(&PsychoacousticEntity::Pitch),
            MusicEntity::PitchHeight
        );
    }
    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn every_entity_maps_valid() {
        let targets = MusicEntity::variants();
        for obj in PsychoacousticEntity::variants() {
            assert!(targets.contains(&PsychoacousticsToMusic::map_object(&obj)));
        }
    }
}
