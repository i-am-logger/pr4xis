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

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Concept)]
pub enum NeuralEntity {
    RateCoding,
    TemporalCoding,
    PhaseLocking,
    PlaceCoding,
    PopulationCoding,
    SpikeTimingCode,
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
    AuditoryNerveFiber,
    CochlearNucleusProcessing,
    SuperiorOliveProcessing,
    LateralLemniscus,
    InferiorColliculusProcessing,
    MedialGeniculateProcessing,
    AuditoryCortexProcessing,
    BinauralProcessing,
    CoincidenceDetection,
    ExcitatoryInhibitory,
    MedialSuperiorOlive,
    LateralSuperiorOlive,
    AuditorySceneAnalysis,
    StreamSegregation,
    GestaltGrouping,
    EchoSuppression,
    PrecedenceEffect,
    MismatchNegativity,
    CodingStrategy,
    ResponseProperty,
    ProcessingStage,
    BinauralMechanism,
    HigherFunction,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Concept)]
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

define_ontology! {
    /// Discrete category over neural entities.
    pub NeuroscienceOntology for NeuroscienceCategory {
        entity: NeuralEntity, relation: NeuralRelation,
        being: AbstractObject,
        source: "Kandel et al. (2021); Schnupp et al. (2011)",
        taxonomy: NeuralTaxonomy [
            (RateCoding, CodingStrategy), (TemporalCoding, CodingStrategy), (PhaseLocking, CodingStrategy), (PlaceCoding, CodingStrategy), (PopulationCoding, CodingStrategy), (SpikeTimingCode, CodingStrategy),
            (TonotopicMap, ResponseProperty), (FrequencyTuningCurve, ResponseProperty), (CharacteristicFrequency, ResponseProperty), (RateLevelFunction, ResponseProperty), (SpontaneousRate, ResponseProperty), (DynamicRange, ResponseProperty), (OnsetResponse, ResponseProperty), (SustainedResponse, ResponseProperty), (Adaptation, ResponseProperty), (Inhibition, ResponseProperty),
            (AuditoryNerveFiber, ProcessingStage), (CochlearNucleusProcessing, ProcessingStage), (SuperiorOliveProcessing, ProcessingStage), (LateralLemniscus, ProcessingStage), (InferiorColliculusProcessing, ProcessingStage), (MedialGeniculateProcessing, ProcessingStage), (AuditoryCortexProcessing, ProcessingStage),
            (CoincidenceDetection, BinauralMechanism), (ExcitatoryInhibitory, BinauralMechanism), (MedialSuperiorOlive, BinauralMechanism), (LateralSuperiorOlive, BinauralMechanism),
            (AuditorySceneAnalysis, HigherFunction), (StreamSegregation, HigherFunction), (GestaltGrouping, HigherFunction), (EchoSuppression, HigherFunction), (PrecedenceEffect, HigherFunction), (MismatchNegativity, HigherFunction),
        ],
        causation: NeuralCausalGraph for NeuralCausalEvent [
            (AuditoryNerveInput, CochlearNucleusIntegration), (CochlearNucleusIntegration, BinauralConvergence), (BinauralConvergence, LemniscalRelay), (LemniscalRelay, MultisensoryIntegration), (MultisensoryIntegration, ThalamicGating), (ThalamicGating, CorticalAnalysis), (CorticalAnalysis, StreamFormation), (StreamFormation, PerceptualBinding),
        ],
        opposition: NeuralOpposition [
            (RateCoding, TemporalCoding), (OnsetResponse, SustainedResponse), (Inhibition, Adaptation),
        ],
    }
}

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

#[derive(Debug, Clone)]
pub struct SynapticDelay;
impl Quality for SynapticDelay {
    type Individual = NeuralEntity;
    type Value = f64;
    fn get(&self, individual: &NeuralEntity) -> Option<f64> {
        use NeuralEntity::*;
        match individual {
            CochlearNucleusProcessing => Some(0.8),
            SuperiorOliveProcessing => Some(1.2),
            _ => None,
        }
    }
}

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
            | AuditoryCortexProcessing
            | SuperiorOliveProcessing => Some(true),
            _ => None,
        }
    }
}

// Axioms

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
pr4xis::register_axiom!(InputCausesBinding);

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
pr4xis::register_axiom!(SixCodingStrategies);

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
pr4xis::register_axiom!(SOCDelayLongerThanCN);

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
pr4xis::register_axiom!(AllStagesAreTonotopic);

impl Ontology for NeuroscienceOntology {
    type Cat = NeuroscienceCategory;
    type Qual = PhaseLockingLimit;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(InputCausesBinding),
            Box::new(SixCodingStrategies),
            Box::new(SOCDelayLongerThanCN),
            Box::new(AllStagesAreTonotopic),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::opposition;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

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
        #[test] fn prop_taxonomy_reflexive(entity in arb_entity()) { prop_assert!(taxonomy::is_a::<NeuralTaxonomy>(&entity, &entity)); }
    }
}
