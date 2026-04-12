//! Environmental acoustics ontology.
//!
//! Models room acoustics, noise exposure, and soundscape ecology.
//!
//! Key references:
//! - Kuttruff 2009: Room Acoustics (5th ed.)
//! - Sabine 1922: reverberation time formula (RT60 = 0.161V/A)
//! - OSHA 29 CFR 1910.95: 90 dBA TWA / 8 hr, 5 dB exchange rate
//! - NIOSH 1998: 85 dBA TWA / 8 hr, 3 dB exchange rate (recommended)
//! - ISO 3382-1:2009: room acoustic parameters
//! - Schafer 1977: The Soundscape

use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum EnvironmentEntity {
    // Room acoustics parameters
    ReverberationTime,
    RT60,
    EarlyDecayTime,
    Clarity,
    Definition,
    SpeechTransmissionIndex,
    CenterTime,
    LateralFraction,
    // Acoustic properties
    SoundAbsorption,
    AbsorptionCoefficient,
    SoundDiffusion,
    SoundInsulation,
    TransmissionLoss,
    FlankingTransmission,
    // Noise measurement
    SoundPressureLevel,
    AWeighting,
    CWeighting,
    EquivalentContinuousLevel,
    PeakSoundLevel,
    SoundExposureLevel,
    NoiseDose,
    TimeWeightedAverage,
    // Noise standards
    OSHALimit,
    NIOSHLimit,
    ExchangeRate,
    PermissibleExposureLimit,
    ActionLevel,
    // Soundscape
    Soundscape,
    Keynote,
    SoundSignal,
    Soundmark,
    BackgroundNoise,
    // Room types (by acoustic purpose)
    SpeechRoom,
    MusicHall,
    WorshipSpace,
    // Equipment
    SoundLevelMeter,
    Dosimeter,
    CalibrationSource,
    // Abstract categories
    RoomParameter,
    AcousticProperty,
    NoiseMeasure,
    NoiseStandard,
    SoundscapeElement,
    MeasurementDevice,
    RoomType,
}

// ---------------------------------------------------------------------------
// Taxonomy
// ---------------------------------------------------------------------------

pub struct EnvironmentTaxonomy;

impl TaxonomyDef for EnvironmentTaxonomy {
    type Entity = EnvironmentEntity;

    fn relations() -> Vec<(EnvironmentEntity, EnvironmentEntity)> {
        use EnvironmentEntity::*;
        vec![
            // Room parameters
            (ReverberationTime, RoomParameter),
            (RT60, ReverberationTime),
            (EarlyDecayTime, RoomParameter),
            (Clarity, RoomParameter),
            (Definition, RoomParameter),
            (SpeechTransmissionIndex, RoomParameter),
            (CenterTime, RoomParameter),
            (LateralFraction, RoomParameter),
            // Acoustic properties
            (SoundAbsorption, AcousticProperty),
            (AbsorptionCoefficient, AcousticProperty),
            (SoundDiffusion, AcousticProperty),
            (SoundInsulation, AcousticProperty),
            (TransmissionLoss, AcousticProperty),
            (FlankingTransmission, AcousticProperty),
            // Noise measures
            (SoundPressureLevel, NoiseMeasure),
            (AWeighting, NoiseMeasure),
            (CWeighting, NoiseMeasure),
            (EquivalentContinuousLevel, NoiseMeasure),
            (PeakSoundLevel, NoiseMeasure),
            (SoundExposureLevel, NoiseMeasure),
            (NoiseDose, NoiseMeasure),
            (TimeWeightedAverage, NoiseMeasure),
            // Standards
            (OSHALimit, NoiseStandard),
            (NIOSHLimit, NoiseStandard),
            (ExchangeRate, NoiseStandard),
            (PermissibleExposureLimit, NoiseStandard),
            (ActionLevel, NoiseStandard),
            // Soundscape elements
            (Keynote, SoundscapeElement),
            (SoundSignal, SoundscapeElement),
            (Soundmark, SoundscapeElement),
            (BackgroundNoise, SoundscapeElement),
            // Room types
            (SpeechRoom, RoomType),
            (MusicHall, RoomType),
            (WorshipSpace, RoomType),
            // Measurement devices
            (SoundLevelMeter, MeasurementDevice),
            (Dosimeter, MeasurementDevice),
            (CalibrationSource, MeasurementDevice),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Causal events in environmental noise exposure and room acoustics.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum EnvironmentCausalEvent {
    NoiseSource,
    SoundPropagation,
    WorkerExposure,
    DoseAccumulation,
    ThresholdShift,
    HearingDamageRisk,
    RoomReverberation,
    SpeechIntelligibilityReduction,
}

/// Causal graph for noise exposure and room acoustic effects.
pub struct EnvironmentCausalGraph;

impl CausalDef for EnvironmentCausalGraph {
    type Entity = EnvironmentCausalEvent;

    fn relations() -> Vec<(EnvironmentCausalEvent, EnvironmentCausalEvent)> {
        use EnvironmentCausalEvent::*;
        vec![
            // Noise source emits sound that propagates
            (NoiseSource, SoundPropagation),
            // Propagating sound reaches workers
            (SoundPropagation, WorkerExposure),
            // Exposure accumulates into dose
            (WorkerExposure, DoseAccumulation),
            // Accumulated dose causes threshold shift
            (DoseAccumulation, ThresholdShift),
            // Threshold shift creates hearing damage risk
            (ThresholdShift, HearingDamageRisk),
            // Propagation also creates room reverberation
            (SoundPropagation, RoomReverberation),
            // Reverberation reduces speech intelligibility
            (RoomReverberation, SpeechIntelligibilityReduction),
        ]
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

pub struct EnvironmentOpposition;

impl OppositionDef for EnvironmentOpposition {
    type Entity = EnvironmentEntity;

    fn pairs() -> Vec<(EnvironmentEntity, EnvironmentEntity)> {
        use EnvironmentEntity::*;
        vec![(SoundAbsorption, SoundDiffusion), (AWeighting, CWeighting)]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over environmental acoustics entities.
    pub EnvironmentalAcousticsCategory {
        entity: EnvironmentEntity,
        relation: EnvironmentRelation,
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Regulatory limit (dB TWA over 8 hours).
///
/// - OSHA PEL: 90 dBA (5 dB exchange rate) — 29 CFR 1910.95
/// - NIOSH REL: 85 dBA (3 dB exchange rate) — NIOSH 1998
/// - OSHA Action Level: 85 dBA
#[derive(Debug, Clone)]
pub struct RegulatoryLimitDB;

impl Quality for RegulatoryLimitDB {
    type Individual = EnvironmentEntity;
    type Value = f64;

    fn get(&self, individual: &EnvironmentEntity) -> Option<f64> {
        use EnvironmentEntity::*;
        match individual {
            OSHALimit => Some(90.0),
            NIOSHLimit => Some(85.0),
            PermissibleExposureLimit => Some(90.0),
            ActionLevel => Some(85.0),
            _ => None,
        }
    }
}

/// Exchange rate (dB) for noise dose calculation.
///
/// OSHA: 5 dB (halving rate)
/// NIOSH: 3 dB (equal-energy principle)
#[derive(Debug, Clone)]
pub struct ExchangeRateDB;

impl Quality for ExchangeRateDB {
    type Individual = EnvironmentEntity;
    type Value = f64;

    fn get(&self, individual: &EnvironmentEntity) -> Option<f64> {
        use EnvironmentEntity::*;
        match individual {
            OSHALimit => Some(5.0),
            NIOSHLimit => Some(3.0),
            _ => None,
        }
    }
}

/// Ideal RT60 reverberation time (seconds) by room purpose.
///
/// - SpeechRoom: 0.5 s (optimized for speech intelligibility)
/// - MusicHall: 1.5 s (warmth and envelopment for orchestral music)
/// - WorshipSpace: 2.0 s (long reverb for organ/choral music)
///
/// Kuttruff 2009, Ch. 9.
#[derive(Debug, Clone)]
pub struct IdealRT60Seconds;

impl Quality for IdealRT60Seconds {
    type Individual = EnvironmentEntity;
    type Value = f64;

    fn get(&self, individual: &EnvironmentEntity) -> Option<f64> {
        use EnvironmentEntity::*;
        match individual {
            SpeechRoom => Some(0.5),
            MusicHall => Some(1.5),
            WorshipSpace => Some(2.0),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Speech rooms have shortest ideal RT60 (Kuttruff 2009).
pub struct SpeechRoomShortestRT60;
impl Axiom for SpeechRoomShortestRT60 {
    fn description(&self) -> &str {
        "speech rooms have shortest ideal RT60"
    }
    fn holds(&self) -> bool {
        use EnvironmentEntity::*;
        let speech = IdealRT60Seconds.get(&SpeechRoom).unwrap();
        let music = IdealRT60Seconds.get(&MusicHall).unwrap();
        let worship = IdealRT60Seconds.get(&WorshipSpace).unwrap();
        speech < music && music < worship
    }
}

pub struct EnvironmentTaxonomyIsDAG;
impl Axiom for EnvironmentTaxonomyIsDAG {
    fn description(&self) -> &str {
        "environmental acoustics taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        taxonomy::NoCycles::<EnvironmentTaxonomy>::new().holds()
    }
}

/// NIOSH limit is stricter (lower) than OSHA limit.
///
/// NIOSH 1998 recommends 85 dBA; OSHA requires 90 dBA.
pub struct NIOSHStricterThanOSHA;
impl Axiom for NIOSHStricterThanOSHA {
    fn description(&self) -> &str {
        "NIOSH limit (85 dBA) is stricter than OSHA (90 dBA)"
    }
    fn holds(&self) -> bool {
        use EnvironmentEntity::*;
        RegulatoryLimitDB.get(&NIOSHLimit).unwrap() < RegulatoryLimitDB.get(&OSHALimit).unwrap()
    }
}

/// NIOSH uses 3 dB exchange rate (equal-energy), OSHA uses 5 dB.
pub struct NIOSHUsesEqualEnergy;
impl Axiom for NIOSHUsesEqualEnergy {
    fn description(&self) -> &str {
        "NIOSH uses 3 dB exchange rate (stricter than OSHA 5 dB)"
    }
    fn holds(&self) -> bool {
        use EnvironmentEntity::*;
        ExchangeRateDB.get(&NIOSHLimit).unwrap() < ExchangeRateDB.get(&OSHALimit).unwrap()
    }
}

/// RT60 is-a ReverberationTime is-a RoomParameter.
pub struct RT60Subsumption;
impl Axiom for RT60Subsumption {
    fn description(&self) -> &str {
        "RT60 is-a ReverberationTime is-a RoomParameter"
    }
    fn holds(&self) -> bool {
        use EnvironmentEntity::*;
        taxonomy::is_a::<EnvironmentTaxonomy>(&RT60, &ReverberationTime)
            && taxonomy::is_a::<EnvironmentTaxonomy>(&ReverberationTime, &RoomParameter)
            && taxonomy::is_a::<EnvironmentTaxonomy>(&RT60, &RoomParameter)
    }
}

pub struct EnvironmentOppositionSymmetric;
impl Axiom for EnvironmentOppositionSymmetric {
    fn description(&self) -> &str {
        "environmental opposition is symmetric"
    }
    fn holds(&self) -> bool {
        opposition::Symmetric::<EnvironmentOpposition>::new().holds()
    }
}

/// Causal graph is asymmetric.
pub struct EnvironmentCausalGraphIsAsymmetric;
impl Axiom for EnvironmentCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "environmental causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<EnvironmentCausalGraph>::new().holds()
    }
}

/// No event causes itself.
pub struct EnvironmentCausalGraphNoSelfCause;
impl Axiom for EnvironmentCausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no environmental causal event causes itself"
    }
    fn holds(&self) -> bool {
        causation::NoSelfCausation::<EnvironmentCausalGraph>::new().holds()
    }
}

/// Noise source transitively causes hearing damage risk.
pub struct NoiseCausesHearingDamage;
impl Axiom for NoiseCausesHearingDamage {
    fn description(&self) -> &str {
        "noise source transitively causes hearing damage risk"
    }
    fn holds(&self) -> bool {
        use EnvironmentCausalEvent::*;
        let effects = causation::effects_of::<EnvironmentCausalGraph>(&NoiseSource);
        effects.contains(&HearingDamageRisk)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

pub struct EnvironmentalAcousticsOntology;

impl Ontology for EnvironmentalAcousticsOntology {
    type Cat = EnvironmentalAcousticsCategory;
    type Qual = RegulatoryLimitDB;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(EnvironmentTaxonomyIsDAG),
            Box::new(NIOSHStricterThanOSHA),
            Box::new(NIOSHUsesEqualEnergy),
            Box::new(RT60Subsumption),
            Box::new(EnvironmentOppositionSymmetric),
            Box::new(EnvironmentCausalGraphIsAsymmetric),
            Box::new(EnvironmentCausalGraphNoSelfCause),
            Box::new(SpeechRoomShortestRT60),
            Box::new(NoiseCausesHearingDamage),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_taxonomy_dag() {
        assert!(EnvironmentTaxonomyIsDAG.holds());
    }
    #[test]
    fn test_niosh_stricter() {
        assert!(NIOSHStricterThanOSHA.holds());
    }
    #[test]
    fn test_niosh_equal_energy() {
        assert!(NIOSHUsesEqualEnergy.holds());
    }
    #[test]
    fn test_rt60_subsumption() {
        assert!(RT60Subsumption.holds());
    }
    #[test]
    fn test_opposition_symmetric() {
        assert!(EnvironmentOppositionSymmetric.holds());
    }
    #[test]
    fn test_causal_graph_asymmetric() {
        assert!(EnvironmentCausalGraphIsAsymmetric.holds());
    }
    #[test]
    fn test_causal_graph_no_self_cause() {
        assert!(EnvironmentCausalGraphNoSelfCause.holds());
    }
    #[test]
    fn test_noise_causes_hearing_damage() {
        assert!(NoiseCausesHearingDamage.holds());
    }

    #[test]
    fn test_category_laws() {
        check_category_laws::<EnvironmentalAcousticsCategory>().unwrap();
    }
    #[test]
    fn test_taxonomy_laws() {
        check_category_laws::<TaxonomyCategory<EnvironmentTaxonomy>>().unwrap();
    }
    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<EnvironmentCausalGraph>>().unwrap();
    }

    #[test]
    fn test_speech_room_shortest_rt60() {
        assert!(SpeechRoomShortestRT60.holds());
    }
    #[test]
    fn test_speech_room_rt60() {
        assert_eq!(
            IdealRT60Seconds.get(&EnvironmentEntity::SpeechRoom),
            Some(0.5)
        );
    }
    #[test]
    fn test_worship_space_rt60() {
        assert_eq!(
            IdealRT60Seconds.get(&EnvironmentEntity::WorshipSpace),
            Some(2.0)
        );
    }

    #[test]
    fn test_osha_limit() {
        assert_eq!(
            RegulatoryLimitDB.get(&EnvironmentEntity::OSHALimit),
            Some(90.0)
        );
    }
    #[test]
    fn test_niosh_limit() {
        assert_eq!(
            RegulatoryLimitDB.get(&EnvironmentEntity::NIOSHLimit),
            Some(85.0)
        );
    }
    #[test]
    fn test_entity_count() {
        assert_eq!(EnvironmentEntity::variants().len(), 45);
    }
    #[test]
    fn test_ontology_validates() {
        EnvironmentalAcousticsOntology::validate().unwrap();
    }

    fn arb_entity() -> impl Strategy<Value = EnvironmentEntity> {
        (0..EnvironmentEntity::variants().len()).prop_map(|i| EnvironmentEntity::variants()[i])
    }
    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<EnvironmentTaxonomy>(&entity, &entity));
        }
    }
}
