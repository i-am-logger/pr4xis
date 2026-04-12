//! Auditory neuroscience ontology.
//!
//! Models neural processing of sound from auditory nerve to cortex.
//!
//! Key concepts:
//! - Tonotopy: frequency maps preserved at each processing stage
//! - Rate coding: spike rate encodes intensity (Sachs & Young 1979)
//! - Temporal coding: phase locking encodes fine timing (< 4 kHz)
//! - Binaural processing: ITD in MSO, ILD in LSO (Goldberg & Brown 1969)
//! - Auditory scene analysis: stream segregation (Bregman 1990)
//!
//! References:
//! - Kandel et al. 2021: Principles of Neural Science
//! - Schnupp et al. 2011: Auditory Neuroscience
//! - Pickles 2012: Physiology of Hearing
//! - Joris, Schreiner & Rees 2004: neural processing of amplitude-modulated sounds

use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// Every entity in the auditory neuroscience domain.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum NeuralEntity {
    // Coding strategies
    RateCoding,
    TemporalCoding,
    PhaseLocking,
    PlaceCoding,
    PopulationCoding,
    SpikeTimingCode,
    // Neural response properties
    TonotopicMap,
    FrequencyTuningCurve,
    CharacteristicFrequency,
    RateLevelFunction,
    SpontaneousRate,
    DynamicRange,
    OnsetResponse,
    SustainedResponse,
    Adaptation,
    Inhibition,
    // Processing stages
    AuditoryNerveFiber,
    CochlearNucleusProcessing,
    SuperiorOliveProcessing,
    LateralLemniscus,
    InferiorColliculusProcessing,
    MedialGeniculateProcessing,
    AuditoryCortexProcessing,
    // Binaural processing
    BinauralProcessing,
    CoincidenceDetection,
    ExcitatoryInhibitory,
    MedialSuperiorOlive,
    LateralSuperiorOlive,
    // Higher processing
    AuditorySceneAnalysis,
    StreamSegregation,
    GestaltGrouping,
    EchoSuppression,
    PrecedenceEffect,
    MismatchNegativity,
    // Abstract categories
    CodingStrategy,
    ResponseProperty,
    ProcessingStage,
    BinauralMechanism,
    HigherFunction,
}

// ---------------------------------------------------------------------------
// Taxonomy (is-a)
// ---------------------------------------------------------------------------

pub struct NeuralTaxonomy;

impl TaxonomyDef for NeuralTaxonomy {
    type Entity = NeuralEntity;

    fn relations() -> Vec<(NeuralEntity, NeuralEntity)> {
        use NeuralEntity::*;
        vec![
            // Coding strategies
            (RateCoding, CodingStrategy),
            (TemporalCoding, CodingStrategy),
            (PhaseLocking, CodingStrategy),
            (PlaceCoding, CodingStrategy),
            (PopulationCoding, CodingStrategy),
            (SpikeTimingCode, CodingStrategy),
            // Response properties
            (TonotopicMap, ResponseProperty),
            (FrequencyTuningCurve, ResponseProperty),
            (CharacteristicFrequency, ResponseProperty),
            (RateLevelFunction, ResponseProperty),
            (SpontaneousRate, ResponseProperty),
            (DynamicRange, ResponseProperty),
            (OnsetResponse, ResponseProperty),
            (SustainedResponse, ResponseProperty),
            (Adaptation, ResponseProperty),
            (Inhibition, ResponseProperty),
            // Processing stages
            (AuditoryNerveFiber, ProcessingStage),
            (CochlearNucleusProcessing, ProcessingStage),
            (SuperiorOliveProcessing, ProcessingStage),
            (LateralLemniscus, ProcessingStage),
            (InferiorColliculusProcessing, ProcessingStage),
            (MedialGeniculateProcessing, ProcessingStage),
            (AuditoryCortexProcessing, ProcessingStage),
            // Binaural mechanisms
            (CoincidenceDetection, BinauralMechanism),
            (ExcitatoryInhibitory, BinauralMechanism),
            (MedialSuperiorOlive, BinauralMechanism),
            (LateralSuperiorOlive, BinauralMechanism),
            // Higher functions
            (AuditorySceneAnalysis, HigherFunction),
            (StreamSegregation, HigherFunction),
            (GestaltGrouping, HigherFunction),
            (EchoSuppression, HigherFunction),
            (PrecedenceEffect, HigherFunction),
            (MismatchNegativity, HigherFunction),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Causal events in the ascending auditory pathway.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum NeuralCausalEvent {
    AuditoryNerveInput,
    CochlearNucleusIntegration,
    BinauralConvergence,
    LemniscalRelay,
    MultisensoryIntegration,
    ThalamicGating,
    CorticalAnalysis,
    StreamFormation,
    PerceptualBinding,
}

/// Ascending auditory pathway causal chain.
pub struct NeuralCausalGraph;

impl CausalDef for NeuralCausalGraph {
    type Entity = NeuralCausalEvent;

    fn relations() -> Vec<(NeuralCausalEvent, NeuralCausalEvent)> {
        use NeuralCausalEvent::*;
        vec![
            (AuditoryNerveInput, CochlearNucleusIntegration),
            (CochlearNucleusIntegration, BinauralConvergence),
            (BinauralConvergence, LemniscalRelay),
            (LemniscalRelay, MultisensoryIntegration),
            (MultisensoryIntegration, ThalamicGating),
            (ThalamicGating, CorticalAnalysis),
            (CorticalAnalysis, StreamFormation),
            (StreamFormation, PerceptualBinding),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over neural entities.
    pub NeuroscienceCategory {
        entity: NeuralEntity,
        relation: NeuralRelation,
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Phase locking limit (Hz) — frequency above which phase locking degrades.
///
/// Auditory nerve: ~4-5 kHz (Palmer & Russell 1986)
/// MSO neurons: ~1.5 kHz (Goldberg & Brown 1969)
#[derive(Debug, Clone)]
pub struct PhaseLockingLimit;

impl Quality for PhaseLockingLimit {
    type Individual = NeuralEntity;
    type Value = f64;

    fn get(&self, individual: &NeuralEntity) -> Option<f64> {
        use NeuralEntity::*;
        match individual {
            AuditoryNerveFiber => Some(4000.0),
            MedialSuperiorOlive => Some(1500.0),
            CochlearNucleusProcessing => Some(4000.0),
            _ => None,
        }
    }
}

/// Synaptic delay (ms) from one processing stage to the next.
///
/// - AuditoryNerveFiber -> CochlearNucleus: ~0.8 ms
/// - SuperiorOliveProcessing (SOC): ~1.2 ms
///
/// Kandel et al. 2021, Ch. 30.
#[derive(Debug, Clone)]
pub struct SynapticDelay;

impl Quality for SynapticDelay {
    type Individual = NeuralEntity;
    type Value = f64;

    fn get(&self, individual: &NeuralEntity) -> Option<f64> {
        use NeuralEntity::*;
        match individual {
            CochlearNucleusProcessing => Some(0.8), // AN→CN delay
            SuperiorOliveProcessing => Some(1.2),   // CN→SOC delay
            _ => None,
        }
    }
}

/// Whether this stage preserves tonotopic organization.
#[derive(Debug, Clone)]
pub struct IsTonotopic;

impl Quality for IsTonotopic {
    type Individual = NeuralEntity;
    type Value = bool;

    fn get(&self, individual: &NeuralEntity) -> Option<bool> {
        use NeuralEntity::*;
        match individual {
            AuditoryNerveFiber
            | CochlearNucleusProcessing
            | InferiorColliculusProcessing
            | MedialGeniculateProcessing
            | AuditoryCortexProcessing => Some(true),
            SuperiorOliveProcessing => Some(true),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in auditory neuroscience.
///
/// - RateCoding vs TemporalCoding: two competing coding strategies
/// - OnsetResponse vs SustainedResponse: transient vs sustained firing
/// - Inhibition vs Adaptation: two forms of response reduction (different mechanisms)
pub struct NeuralOpposition;

impl OppositionDef for NeuralOpposition {
    type Entity = NeuralEntity;

    fn pairs() -> Vec<(NeuralEntity, NeuralEntity)> {
        use NeuralEntity::*;
        vec![
            (RateCoding, TemporalCoding),
            (OnsetResponse, SustainedResponse),
            (Inhibition, Adaptation),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

pub struct NeuralTaxonomyIsDAG;
impl Axiom for NeuralTaxonomyIsDAG {
    fn description(&self) -> &str {
        "neural taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        taxonomy::NoCycles::<NeuralTaxonomy>::new().holds()
    }
}

pub struct NeuralCausalGraphIsAsymmetric;
impl Axiom for NeuralCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "neural causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<NeuralCausalGraph>::new().holds()
    }
}

pub struct NeuralCausalGraphNoSelfCause;
impl Axiom for NeuralCausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no neural event causes itself"
    }
    fn holds(&self) -> bool {
        causation::NoSelfCausation::<NeuralCausalGraph>::new().holds()
    }
}

/// Auditory nerve input transitively causes perceptual binding.
pub struct InputCausesBinding;
impl Axiom for InputCausesBinding {
    fn description(&self) -> &str {
        "auditory nerve input transitively causes perceptual binding"
    }
    fn holds(&self) -> bool {
        use NeuralCausalEvent::*;
        causation::effects_of::<NeuralCausalGraph>(&AuditoryNerveInput).contains(&PerceptualBinding)
    }
}

/// Six coding strategies are classified.
pub struct SixCodingStrategies;
impl Axiom for SixCodingStrategies {
    fn description(&self) -> &str {
        "six coding strategies are classified"
    }
    fn holds(&self) -> bool {
        use NeuralEntity::*;
        [
            RateCoding,
            TemporalCoding,
            PhaseLocking,
            PlaceCoding,
            PopulationCoding,
            SpikeTimingCode,
        ]
        .iter()
        .all(|c| taxonomy::is_a::<NeuralTaxonomy>(c, &CodingStrategy))
    }
}

/// Opposition is symmetric.
pub struct NeuralOppositionSymmetric;
impl Axiom for NeuralOppositionSymmetric {
    fn description(&self) -> &str {
        "neural opposition is symmetric"
    }
    fn holds(&self) -> bool {
        opposition::Symmetric::<NeuralOpposition>::new().holds()
    }
}

/// Opposition is irreflexive.
pub struct NeuralOppositionIrreflexive;
impl Axiom for NeuralOppositionIrreflexive {
    fn description(&self) -> &str {
        "neural opposition is irreflexive"
    }
    fn holds(&self) -> bool {
        opposition::Irreflexive::<NeuralOpposition>::new().holds()
    }
}

/// SOC synaptic delay is longer than CN delay (further from periphery).
///
/// Kandel et al. 2021.
pub struct SOCDelayLongerThanCN;
impl Axiom for SOCDelayLongerThanCN {
    fn description(&self) -> &str {
        "SOC synaptic delay > CN synaptic delay"
    }
    fn holds(&self) -> bool {
        use NeuralEntity::*;
        SynapticDelay.get(&SuperiorOliveProcessing).unwrap()
            > SynapticDelay.get(&CochlearNucleusProcessing).unwrap()
    }
}

/// All processing stages from AN to cortex are tonotopic.
///
/// Kandel et al. 2021, Ch. 30.
pub struct AllStagesAreTonotopic;
impl Axiom for AllStagesAreTonotopic {
    fn description(&self) -> &str {
        "all major processing stages are tonotopic"
    }
    fn holds(&self) -> bool {
        use NeuralEntity::*;
        [
            AuditoryNerveFiber,
            CochlearNucleusProcessing,
            SuperiorOliveProcessing,
            InferiorColliculusProcessing,
            MedialGeniculateProcessing,
            AuditoryCortexProcessing,
        ]
        .iter()
        .all(|s| IsTonotopic.get(s) == Some(true))
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

pub struct NeuroscienceOntology;

impl Ontology for NeuroscienceOntology {
    type Cat = NeuroscienceCategory;
    type Qual = PhaseLockingLimit;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(NeuralTaxonomyIsDAG),
            Box::new(NeuralCausalGraphIsAsymmetric),
            Box::new(NeuralCausalGraphNoSelfCause),
            Box::new(InputCausesBinding),
            Box::new(SixCodingStrategies),
            Box::new(SOCDelayLongerThanCN),
            Box::new(AllStagesAreTonotopic),
            Box::new(NeuralOppositionSymmetric),
            Box::new(NeuralOppositionIrreflexive),
        ]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(NeuralTaxonomyIsDAG.holds());
    }
    #[test]
    fn test_causal_asymmetric() {
        assert!(NeuralCausalGraphIsAsymmetric.holds());
    }
    #[test]
    fn test_causal_no_self() {
        assert!(NeuralCausalGraphNoSelfCause.holds());
    }
    #[test]
    fn test_input_causes_binding() {
        assert!(InputCausesBinding.holds());
    }
    #[test]
    fn test_six_coding_strategies() {
        assert!(SixCodingStrategies.holds());
    }
    #[test]
    fn test_all_stages_tonotopic() {
        assert!(AllStagesAreTonotopic.holds());
    }
    #[test]
    fn test_opposition_symmetric() {
        assert!(NeuralOppositionSymmetric.holds());
    }
    #[test]
    fn test_opposition_irreflexive() {
        assert!(NeuralOppositionIrreflexive.holds());
    }
    #[test]
    fn test_rate_opposes_temporal_coding() {
        assert!(opposition::are_opposed::<NeuralOpposition>(
            &NeuralEntity::RateCoding,
            &NeuralEntity::TemporalCoding
        ));
    }

    #[test]
    fn test_category_laws() {
        check_category_laws::<NeuroscienceCategory>().unwrap();
    }
    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<NeuralTaxonomy>>().unwrap();
    }
    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<NeuralCausalGraph>>().unwrap();
    }

    #[test]
    fn test_soc_delay_longer_than_cn() {
        assert!(SOCDelayLongerThanCN.holds());
    }
    #[test]
    fn test_cn_synaptic_delay() {
        assert_eq!(
            SynapticDelay.get(&NeuralEntity::CochlearNucleusProcessing),
            Some(0.8)
        );
    }
    #[test]
    fn test_soc_synaptic_delay() {
        assert_eq!(
            SynapticDelay.get(&NeuralEntity::SuperiorOliveProcessing),
            Some(1.2)
        );
    }

    #[test]
    fn test_phase_locking_limit_an() {
        assert_eq!(
            PhaseLockingLimit.get(&NeuralEntity::AuditoryNerveFiber),
            Some(4000.0)
        );
    }

    #[test]
    fn test_entity_count() {
        assert_eq!(NeuralEntity::variants().len(), 39);
    }

    #[test]
    fn test_ontology_validates() {
        NeuroscienceOntology::validate().unwrap();
    }

    fn arb_entity() -> impl Strategy<Value = NeuralEntity> {
        (0..NeuralEntity::variants().len()).prop_map(|i| NeuralEntity::variants()[i])
    }

    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<NeuralTaxonomy>(&entity, &entity));
        }
    }
}
