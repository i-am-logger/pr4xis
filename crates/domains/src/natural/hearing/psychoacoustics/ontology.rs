//! Psychoacoustics ontology.
//!
//! Models auditory perception: how physical sound becomes subjective experience.
//!
//! References:
//! - Fletcher & Munson 1933: equal-loudness contours
//! - Zwicker & Fastl 2007: psychoacoustic models
//! - Moore 2012: psychology of hearing
//! - Stevens 1957: sone scale
//! - Rayleigh 1907: duplex theory

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::opposition;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum PsychoacousticEntity {
    Loudness,
    Pitch,
    Timbre,
    Duration,
    Phon,
    Sone,
    EqualLoudnessContour,
    LoudnessRecruitment,
    PlacePitch,
    TemporalPitch,
    VirtualPitch,
    Octave,
    SimultaneousMasking,
    ForwardMasking,
    BackwardMasking,
    InformationalMasking,
    CriticalBand,
    BarkScale,
    ERBScale,
    AuditoryFilter,
    FrequencySelectivity,
    TemporalResolution,
    GapDetection,
    TemporalIntegration,
    SoundLocalization,
    InterauralTimeDifference,
    InterauralLevelDifference,
    HeadRelatedTransferFunction,
    AbsoluteThreshold,
    DifferentialThreshold,
    JustNoticeableDifference,
    PerceptualDimension,
    LoudnessMetric,
    PitchMechanism,
    MaskingType,
    SpatialCue,
    TemporalMeasure,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum PsychoacousticCausalEvent {
    AcousticStimulus,
    CochlearFiltering,
    NeuralTransduction,
    BrainstemProcessing,
    CorticalAnalysis,
    PerceptFormation,
    AwareExperience,
    FrequencyAnalysis,
    PitchExtraction,
}
define_ontology! {
    /// Discrete category over psychoacoustic entities.
    pub PsychoacousticsOntology for PsychoacousticsCategory {
        entity: PsychoacousticEntity, relation: PsychoacousticRelation,
        taxonomy: PsychoacousticTaxonomy [
            (Loudness, PerceptualDimension), (Pitch, PerceptualDimension), (Timbre, PerceptualDimension), (Duration, PerceptualDimension),
            (Phon, LoudnessMetric), (Sone, LoudnessMetric), (EqualLoudnessContour, LoudnessMetric), (LoudnessRecruitment, LoudnessMetric),
            (PlacePitch, PitchMechanism), (TemporalPitch, PitchMechanism), (VirtualPitch, PitchMechanism),
            (SimultaneousMasking, MaskingType), (ForwardMasking, MaskingType), (BackwardMasking, MaskingType), (InformationalMasking, MaskingType),
            (InterauralTimeDifference, SpatialCue), (InterauralLevelDifference, SpatialCue), (HeadRelatedTransferFunction, SpatialCue),
            (TemporalResolution, TemporalMeasure), (GapDetection, TemporalMeasure), (TemporalIntegration, TemporalMeasure),
        ],
        causation: PsychoacousticCausalGraph for PsychoacousticCausalEvent [
            (AcousticStimulus, CochlearFiltering), (CochlearFiltering, NeuralTransduction), (NeuralTransduction, BrainstemProcessing), (BrainstemProcessing, CorticalAnalysis), (CorticalAnalysis, PerceptFormation), (PerceptFormation, AwareExperience), (CochlearFiltering, FrequencyAnalysis), (FrequencyAnalysis, PitchExtraction),
        ],
        opposition: PsychoacousticOpposition [ (PlacePitch, TemporalPitch), (SimultaneousMasking, ForwardMasking), (InterauralTimeDifference, InterauralLevelDifference) ],
    }
}
#[derive(Debug, Clone)]
pub struct HearingThresholdDB;
impl Quality for HearingThresholdDB {
    type Individual = PsychoacousticEntity;
    type Value = f64;
    fn get(&self, individual: &PsychoacousticEntity) -> Option<f64> {
        use PsychoacousticEntity::*;
        match individual {
            AbsoluteThreshold => Some(0.0),
            JustNoticeableDifference => Some(1.0),
            _ => None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct CriticalBandwidth;
impl Quality for CriticalBandwidth {
    type Individual = PsychoacousticEntity;
    type Value = f64;
    fn get(&self, individual: &PsychoacousticEntity) -> Option<f64> {
        use PsychoacousticEntity::*;
        match individual {
            CriticalBand => Some(160.0),
            AuditoryFilter => Some(130.0),
            _ => None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct GapDetectionThreshold;
impl Quality for GapDetectionThreshold {
    type Individual = PsychoacousticEntity;
    type Value = f64;
    fn get(&self, individual: &PsychoacousticEntity) -> Option<f64> {
        use PsychoacousticEntity::*;
        match individual {
            GapDetection => Some(2.5),
            TemporalResolution => Some(2.5),
            _ => None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct ITDThreshold;
impl Quality for ITDThreshold {
    type Individual = PsychoacousticEntity;
    type Value = f64;
    fn get(&self, individual: &PsychoacousticEntity) -> Option<f64> {
        use PsychoacousticEntity::*;
        match individual {
            InterauralTimeDifference => Some(15.0),
            InterauralLevelDifference => Some(1.0),
            _ => None,
        }
    }
}

pub struct FourPerceptualDimensions;
impl Axiom for FourPerceptualDimensions {
    fn description(&self) -> &str {
        "loudness, pitch, timbre, and duration are all perceptual dimensions"
    }
    fn holds(&self) -> bool {
        use PsychoacousticEntity::*;
        [Loudness, Pitch, Timbre, Duration]
            .iter()
            .all(|d| taxonomy::is_a::<PsychoacousticTaxonomy>(d, &PerceptualDimension))
    }
}
pub struct FourMaskingTypes;
impl Axiom for FourMaskingTypes {
    fn description(&self) -> &str {
        "simultaneous, forward, backward, and informational masking are all classified"
    }
    fn holds(&self) -> bool {
        use PsychoacousticEntity::*;
        [
            SimultaneousMasking,
            ForwardMasking,
            BackwardMasking,
            InformationalMasking,
        ]
        .iter()
        .all(|m| taxonomy::is_a::<PsychoacousticTaxonomy>(m, &MaskingType))
    }
}
pub struct ThreeSpatialCues;
impl Axiom for ThreeSpatialCues {
    fn description(&self) -> &str {
        "ITD, ILD, and HRTF are all spatial cues"
    }
    fn holds(&self) -> bool {
        use PsychoacousticEntity::*;
        [
            InterauralTimeDifference,
            InterauralLevelDifference,
            HeadRelatedTransferFunction,
        ]
        .iter()
        .all(|c| taxonomy::is_a::<PsychoacousticTaxonomy>(c, &SpatialCue))
    }
}
pub struct ITDOpposesILD;
impl Axiom for ITDOpposesILD {
    fn description(&self) -> &str {
        "ITD and ILD are opposed spatial cues (duplex theory)"
    }
    fn holds(&self) -> bool {
        use PsychoacousticEntity::*;
        opposition::are_opposed::<PsychoacousticOpposition>(
            &InterauralTimeDifference,
            &InterauralLevelDifference,
        )
    }
}
pub struct StimulusCausesExperience;
impl Axiom for StimulusCausesExperience {
    fn description(&self) -> &str {
        "acoustic stimulus transitively causes aware experience"
    }
    fn holds(&self) -> bool {
        use PsychoacousticCausalEvent::*;
        causation::effects_of::<PsychoacousticCausalGraph>(&AcousticStimulus)
            .contains(&AwareExperience)
    }
}

impl Ontology for PsychoacousticsOntology {
    type Cat = PsychoacousticsCategory;
    type Qual = HearingThresholdDB;
    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }
    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(FourPerceptualDimensions),
            Box::new(FourMaskingTypes),
            Box::new(ThreeSpatialCues),
            Box::new(ITDOpposesILD),
            Box::new(StimulusCausesExperience),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;
    #[test]
    fn test_four_perceptual_dimensions() {
        assert!(FourPerceptualDimensions.holds());
    }
    #[test]
    fn test_four_masking_types() {
        assert!(FourMaskingTypes.holds());
    }
    #[test]
    fn test_three_spatial_cues() {
        assert!(ThreeSpatialCues.holds());
    }
    #[test]
    fn test_itd_opposes_ild() {
        assert!(ITDOpposesILD.holds());
    }
    #[test]
    fn test_stimulus_causes_experience() {
        assert!(StimulusCausesExperience.holds());
    }
    #[test]
    fn test_psychoacoustics_category_laws() {
        check_category_laws::<PsychoacousticsCategory>().unwrap();
    }
    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<PsychoacousticTaxonomy>>().unwrap();
    }
    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<PsychoacousticCausalGraph>>().unwrap();
    }
    #[test]
    fn test_loudness_is_perceptual_dimension() {
        assert!(taxonomy::is_a::<PsychoacousticTaxonomy>(
            &PsychoacousticEntity::Loudness,
            &PsychoacousticEntity::PerceptualDimension
        ));
    }
    #[test]
    fn test_place_pitch_is_pitch_mechanism() {
        assert!(taxonomy::is_a::<PsychoacousticTaxonomy>(
            &PsychoacousticEntity::PlacePitch,
            &PsychoacousticEntity::PitchMechanism
        ));
    }
    #[test]
    fn test_itd_is_spatial_cue() {
        assert!(taxonomy::is_a::<PsychoacousticTaxonomy>(
            &PsychoacousticEntity::InterauralTimeDifference,
            &PsychoacousticEntity::SpatialCue
        ));
    }
    #[test]
    fn test_forward_masking_is_masking_type() {
        assert!(taxonomy::is_a::<PsychoacousticTaxonomy>(
            &PsychoacousticEntity::ForwardMasking,
            &PsychoacousticEntity::MaskingType
        ));
    }
    #[test]
    fn test_place_opposes_temporal_pitch() {
        assert!(opposition::are_opposed::<PsychoacousticOpposition>(
            &PsychoacousticEntity::PlacePitch,
            &PsychoacousticEntity::TemporalPitch
        ));
    }
    #[test]
    fn test_absolute_threshold_at_1khz() {
        assert_eq!(
            HearingThresholdDB.get(&PsychoacousticEntity::AbsoluteThreshold),
            Some(0.0)
        );
    }
    #[test]
    fn test_critical_bandwidth() {
        assert_eq!(
            CriticalBandwidth.get(&PsychoacousticEntity::CriticalBand),
            Some(160.0)
        );
    }
    #[test]
    fn test_gap_detection_threshold() {
        assert_eq!(
            GapDetectionThreshold.get(&PsychoacousticEntity::GapDetection),
            Some(2.5)
        );
    }
    #[test]
    fn test_itd_threshold() {
        assert_eq!(
            ITDThreshold.get(&PsychoacousticEntity::InterauralTimeDifference),
            Some(15.0)
        );
    }
    #[test]
    fn test_entity_count() {
        assert_eq!(PsychoacousticEntity::variants().len(), 37);
    }
    #[test]
    fn test_ontology_validates() {
        PsychoacousticsOntology::validate().unwrap();
    }
    fn arb_psychoacoustic_entity() -> impl Strategy<Value = PsychoacousticEntity> {
        (0..PsychoacousticEntity::variants().len())
            .prop_map(|i| PsychoacousticEntity::variants()[i])
    }
    proptest! { #[test] fn prop_taxonomy_reflexive(entity in arb_psychoacoustic_entity()) { prop_assert!(taxonomy::is_a::<PsychoacousticTaxonomy>(&entity, &entity)); } }
}
