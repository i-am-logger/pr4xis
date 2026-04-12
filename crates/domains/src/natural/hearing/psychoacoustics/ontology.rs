//! Psychoacoustics ontology.
//!
//! Models auditory perception: how physical sound becomes subjective experience.
//!
//! Key concepts:
//! - Loudness: perceived intensity (sones), equal-loudness contours (phons)
//! - Pitch: perceived frequency, related to place on basilar membrane
//! - Timbre: spectral shape distinguishing instruments at same pitch/loudness
//! - Masking: one sound rendering another inaudible
//! - Critical bands: frequency resolution of cochlear filters (Bark scale)
//! - Sound localization: ITD and ILD cues
//!
//! References:
//! - Fletcher & Munson 1933: equal-loudness contours
//! - Zwicker & Fastl 2007: psychoacoustic models
//! - Moore 2012: psychology of hearing
//! - Stevens 1957: sone scale
//! - Rayleigh 1907: duplex theory

use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every entity in the psychoacoustics domain.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum PsychoacousticEntity {
    // Perceptual dimensions
    Loudness,
    Pitch,
    Timbre,
    Duration,
    // Loudness-related
    Phon,
    Sone,
    EqualLoudnessContour,
    LoudnessRecruitment,
    // Pitch-related
    PlacePitch,
    TemporalPitch,
    VirtualPitch,
    Octave,
    // Masking
    SimultaneousMasking,
    ForwardMasking,
    BackwardMasking,
    InformationalMasking,
    // Frequency analysis
    CriticalBand,
    BarkScale,
    ERBScale,
    AuditoryFilter,
    FrequencySelectivity,
    // Temporal processing
    TemporalResolution,
    GapDetection,
    TemporalIntegration,
    // Spatial hearing
    SoundLocalization,
    InterauralTimeDifference,
    InterauralLevelDifference,
    HeadRelatedTransferFunction,
    // Thresholds
    AbsoluteThreshold,
    DifferentialThreshold,
    JustNoticeableDifference,
    // Abstract categories
    PerceptualDimension,
    LoudnessMetric,
    PitchMechanism,
    MaskingType,
    SpatialCue,
    TemporalMeasure,
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

/// Subsumption hierarchy for psychoacoustic entities.
pub struct PsychoacousticTaxonomy;

impl TaxonomyDef for PsychoacousticTaxonomy {
    type Entity = PsychoacousticEntity;

    fn relations() -> Vec<(PsychoacousticEntity, PsychoacousticEntity)> {
        use PsychoacousticEntity::*;
        vec![
            // Perceptual dimensions
            (Loudness, PerceptualDimension),
            (Pitch, PerceptualDimension),
            (Timbre, PerceptualDimension),
            (Duration, PerceptualDimension),
            // Loudness metrics
            (Phon, LoudnessMetric),
            (Sone, LoudnessMetric),
            (EqualLoudnessContour, LoudnessMetric),
            (LoudnessRecruitment, LoudnessMetric),
            // Pitch mechanisms
            (PlacePitch, PitchMechanism),
            (TemporalPitch, PitchMechanism),
            (VirtualPitch, PitchMechanism),
            // Masking types
            (SimultaneousMasking, MaskingType),
            (ForwardMasking, MaskingType),
            (BackwardMasking, MaskingType),
            (InformationalMasking, MaskingType),
            // Spatial cues
            (InterauralTimeDifference, SpatialCue),
            (InterauralLevelDifference, SpatialCue),
            (HeadRelatedTransferFunction, SpatialCue),
            // Temporal measures
            (TemporalResolution, TemporalMeasure),
            (GapDetection, TemporalMeasure),
            (TemporalIntegration, TemporalMeasure),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over psychoacoustic entities.
    pub PsychoacousticsCategory {
        entity: PsychoacousticEntity,
        relation: PsychoacousticRelation,
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Causal events in psychoacoustic processing.
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

/// Causal graph for psychoacoustic processing from stimulus to experience.
pub struct PsychoacousticCausalGraph;

impl CausalDef for PsychoacousticCausalGraph {
    type Entity = PsychoacousticCausalEvent;

    fn relations() -> Vec<(PsychoacousticCausalEvent, PsychoacousticCausalEvent)> {
        use PsychoacousticCausalEvent::*;
        vec![
            // Acoustic stimulus enters cochlea for filtering
            (AcousticStimulus, CochlearFiltering),
            // Cochlear filtering drives neural transduction
            (CochlearFiltering, NeuralTransduction),
            // Neural signals processed in brainstem
            (NeuralTransduction, BrainstemProcessing),
            // Brainstem output analyzed in cortex
            (BrainstemProcessing, CorticalAnalysis),
            // Cortical analysis forms percept
            (CorticalAnalysis, PerceptFormation),
            // Percept formation leads to aware experience
            (PerceptFormation, AwareExperience),
            // Cochlear filtering also performs frequency analysis
            (CochlearFiltering, FrequencyAnalysis),
            // Frequency analysis enables pitch extraction
            (FrequencyAnalysis, PitchExtraction),
        ]
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Absolute hearing threshold (dB SPL) at characteristic frequency.
///
/// - 1 kHz: 0 dB SPL (reference, by definition)
/// - 4 kHz: -3.6 dB SPL (ear canal resonance, most sensitive)
/// - 125 Hz: 22 dB SPL (low-frequency roll-off)
/// - 8 kHz: 13 dB SPL
///
/// ISO 226:2003 (equal-loudness contours).
#[derive(Debug, Clone)]
pub struct HearingThresholdDB;

impl Quality for HearingThresholdDB {
    type Individual = PsychoacousticEntity;
    type Value = f64;

    fn get(&self, individual: &PsychoacousticEntity) -> Option<f64> {
        use PsychoacousticEntity::*;
        match individual {
            // Reference threshold at 1 kHz
            AbsoluteThreshold => Some(0.0),
            // JND for intensity: ~1 dB at moderate levels (Weber's law)
            JustNoticeableDifference => Some(1.0),
            _ => None,
        }
    }
}

/// Critical bandwidth (Hz) at different center frequencies.
///
/// At 1 kHz: ~160 Hz (Zwicker 1961)
/// ERB at 1 kHz: ~130 Hz (Glasberg & Moore 1990)
#[derive(Debug, Clone)]
pub struct CriticalBandwidth;

impl Quality for CriticalBandwidth {
    type Individual = PsychoacousticEntity;
    type Value = f64;

    fn get(&self, individual: &PsychoacousticEntity) -> Option<f64> {
        use PsychoacousticEntity::*;
        match individual {
            CriticalBand => Some(160.0),   // Hz at 1 kHz (Zwicker 1961)
            AuditoryFilter => Some(130.0), // ERB at 1 kHz (Glasberg & Moore 1990)
            _ => None,
        }
    }
}

/// Gap detection threshold (ms).
///
/// Minimum detectable silent gap in a broadband noise.
/// ~2-3 ms for broadband noise (Plomp 1964; Moore 2012).
#[derive(Debug, Clone)]
pub struct GapDetectionThreshold;

impl Quality for GapDetectionThreshold {
    type Individual = PsychoacousticEntity;
    type Value = f64;

    fn get(&self, individual: &PsychoacousticEntity) -> Option<f64> {
        use PsychoacousticEntity::*;
        match individual {
            GapDetection => Some(2.5),       // ms, broadband noise
            TemporalResolution => Some(2.5), // ms, same measure
            _ => None,
        }
    }
}

/// ITD threshold (microseconds).
///
/// Just-noticeable ITD for low-frequency tones.
/// ~10-20 us for experienced listeners (Klumpp & Eady 1956).
#[derive(Debug, Clone)]
pub struct ITDThreshold;

impl Quality for ITDThreshold {
    type Individual = PsychoacousticEntity;
    type Value = f64;

    fn get(&self, individual: &PsychoacousticEntity) -> Option<f64> {
        use PsychoacousticEntity::*;
        match individual {
            InterauralTimeDifference => Some(15.0), // microseconds
            InterauralLevelDifference => Some(1.0), // dB (JND for ILD)
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in psychoacoustics.
///
/// - PlacePitch ↔ TemporalPitch: two competing mechanisms
/// - SimultaneousMasking ↔ ForwardMasking: concurrent vs sequential
/// - ITD ↔ ILD: time vs level cues (duplex theory, Rayleigh 1907)
pub struct PsychoacousticOpposition;

impl OppositionDef for PsychoacousticOpposition {
    type Entity = PsychoacousticEntity;

    fn pairs() -> Vec<(PsychoacousticEntity, PsychoacousticEntity)> {
        use PsychoacousticEntity::*;
        vec![
            (PlacePitch, TemporalPitch),
            (SimultaneousMasking, ForwardMasking),
            (InterauralTimeDifference, InterauralLevelDifference),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Taxonomy is a DAG.
pub struct PsychoacousticTaxonomyIsDAG;

impl Axiom for PsychoacousticTaxonomyIsDAG {
    fn description(&self) -> &str {
        "psychoacoustic taxonomy is a directed acyclic graph"
    }

    fn holds(&self) -> bool {
        taxonomy::NoCycles::<PsychoacousticTaxonomy>::new().holds()
    }
}

/// Taxonomy is antisymmetric.
pub struct PsychoacousticTaxonomyIsAntisymmetric;

impl Axiom for PsychoacousticTaxonomyIsAntisymmetric {
    fn description(&self) -> &str {
        "psychoacoustic taxonomy is antisymmetric"
    }

    fn holds(&self) -> bool {
        taxonomy::Antisymmetric::<PsychoacousticTaxonomy>::new().holds()
    }
}

/// Four perceptual dimensions are classified.
///
/// Loudness, pitch, timbre, and duration are the primary auditory percepts.
/// Moore 2012, Ch. 1.
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

/// Four masking types are classified.
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

/// Three spatial cues are classified (duplex theory + HRTF).
///
/// Rayleigh 1907; Blauert 1997.
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

/// Opposition is symmetric.
pub struct PsychoacousticOppositionSymmetric;

impl Axiom for PsychoacousticOppositionSymmetric {
    fn description(&self) -> &str {
        "psychoacoustic opposition is symmetric"
    }

    fn holds(&self) -> bool {
        opposition::Symmetric::<PsychoacousticOpposition>::new().holds()
    }
}

/// Opposition is irreflexive.
pub struct PsychoacousticOppositionIrreflexive;

impl Axiom for PsychoacousticOppositionIrreflexive {
    fn description(&self) -> &str {
        "psychoacoustic opposition is irreflexive"
    }

    fn holds(&self) -> bool {
        opposition::Irreflexive::<PsychoacousticOpposition>::new().holds()
    }
}

/// ITD and ILD are opposed (duplex theory).
///
/// Rayleigh 1907: ITD dominates at low frequencies, ILD at high.
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

/// Causal graph is asymmetric.
pub struct PsychoacousticCausalGraphIsAsymmetric;
impl Axiom for PsychoacousticCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "psychoacoustic causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<PsychoacousticCausalGraph>::new().holds()
    }
}

/// No event causes itself.
pub struct PsychoacousticCausalGraphNoSelfCause;
impl Axiom for PsychoacousticCausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no psychoacoustic event causes itself"
    }
    fn holds(&self) -> bool {
        causation::NoSelfCausation::<PsychoacousticCausalGraph>::new().holds()
    }
}

/// Acoustic stimulus transitively causes aware experience.
pub struct StimulusCausesExperience;
impl Axiom for StimulusCausesExperience {
    fn description(&self) -> &str {
        "acoustic stimulus transitively causes aware experience"
    }
    fn holds(&self) -> bool {
        use PsychoacousticCausalEvent::*;
        let effects = causation::effects_of::<PsychoacousticCausalGraph>(&AcousticStimulus);
        effects.contains(&AwareExperience)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// Top-level psychoacoustics ontology.
pub struct PsychoacousticsOntology;

impl Ontology for PsychoacousticsOntology {
    type Cat = PsychoacousticsCategory;
    type Qual = HearingThresholdDB;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(PsychoacousticTaxonomyIsDAG),
            Box::new(PsychoacousticTaxonomyIsAntisymmetric),
            Box::new(FourPerceptualDimensions),
            Box::new(FourMaskingTypes),
            Box::new(ThreeSpatialCues),
            Box::new(PsychoacousticOppositionSymmetric),
            Box::new(PsychoacousticOppositionIrreflexive),
            Box::new(ITDOpposesILD),
            Box::new(PsychoacousticCausalGraphIsAsymmetric),
            Box::new(PsychoacousticCausalGraphNoSelfCause),
            Box::new(StimulusCausesExperience),
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    // -- Axiom tests --

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(
            PsychoacousticTaxonomyIsDAG.holds(),
            "{}",
            PsychoacousticTaxonomyIsDAG.description()
        );
    }

    #[test]
    fn test_taxonomy_is_antisymmetric() {
        assert!(
            PsychoacousticTaxonomyIsAntisymmetric.holds(),
            "{}",
            PsychoacousticTaxonomyIsAntisymmetric.description()
        );
    }

    #[test]
    fn test_four_perceptual_dimensions() {
        assert!(
            FourPerceptualDimensions.holds(),
            "{}",
            FourPerceptualDimensions.description()
        );
    }

    #[test]
    fn test_four_masking_types() {
        assert!(
            FourMaskingTypes.holds(),
            "{}",
            FourMaskingTypes.description()
        );
    }

    #[test]
    fn test_three_spatial_cues() {
        assert!(
            ThreeSpatialCues.holds(),
            "{}",
            ThreeSpatialCues.description()
        );
    }

    #[test]
    fn test_opposition_symmetric() {
        assert!(
            PsychoacousticOppositionSymmetric.holds(),
            "{}",
            PsychoacousticOppositionSymmetric.description()
        );
    }

    #[test]
    fn test_opposition_irreflexive() {
        assert!(
            PsychoacousticOppositionIrreflexive.holds(),
            "{}",
            PsychoacousticOppositionIrreflexive.description()
        );
    }

    #[test]
    fn test_itd_opposes_ild() {
        assert!(ITDOpposesILD.holds(), "{}", ITDOpposesILD.description());
    }

    #[test]
    fn test_causal_graph_asymmetric() {
        assert!(
            PsychoacousticCausalGraphIsAsymmetric.holds(),
            "{}",
            PsychoacousticCausalGraphIsAsymmetric.description()
        );
    }

    #[test]
    fn test_causal_graph_no_self_cause() {
        assert!(
            PsychoacousticCausalGraphNoSelfCause.holds(),
            "{}",
            PsychoacousticCausalGraphNoSelfCause.description()
        );
    }

    #[test]
    fn test_stimulus_causes_experience() {
        assert!(
            StimulusCausesExperience.holds(),
            "{}",
            StimulusCausesExperience.description()
        );
    }

    // -- Category law tests --

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

    // -- Taxonomy tests --

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

    // -- Opposition tests --

    #[test]
    fn test_place_opposes_temporal_pitch() {
        assert!(opposition::are_opposed::<PsychoacousticOpposition>(
            &PsychoacousticEntity::PlacePitch,
            &PsychoacousticEntity::TemporalPitch
        ));
    }

    // -- Quality tests --

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

    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_psychoacoustic_entity()) {
            prop_assert!(taxonomy::is_a::<PsychoacousticTaxonomy>(&entity, &entity));
        }
    }
}
