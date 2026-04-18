//! Hearing pathology ontology.
//!
//! Models hearing disorders, their mechanisms, and perceptual consequences.
//!
//! Key references:
//! - Moller 2006: Hearing: Anatomy, Physiology, and Disorders
//! - Gates & Mills 2005: Presbycusis (Lancet)
//! - Henderson et al. 2006: noise-induced hearing loss mechanisms
//! - Merchant & Rosowski 2008: conductive hearing loss
//! - Jastreboff 1990: neurophysiological model of tinnitus
//! - Eggermont & Roberts 2004: tinnitus neural mechanisms

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Concept)]
pub enum PathologyEntity {
    ConductiveHearingLoss,
    SensorineuralHearingLoss,
    MixedHearingLoss,
    AuditoryNeuropathy,
    CentralAuditoryProcessingDisorder,
    Otosclerosis,
    Presbycusis,
    NoiseInducedHearingLoss,
    MenieresDisease,
    AcousticNeuroma,
    Tinnitus,
    Hyperacusis,
    SuddenSensorineuralLoss,
    OtitisMedia,
    TympanicPerforation,
    Cholesteatoma,
    HairCellLoss,
    StereociliaDamage,
    SynapticRibbonLoss,
    StriaDysfunction,
    OssicularFixation,
    EndolymphaticHydrops,
    DemyelinationVIII,
    Excitotoxicity,
    OxidativeStress,
    ElevatedThreshold,
    ReducedFrequencySelectivity,
    LoudnessRecruitment,
    PoorSpeechInNoise,
    ReducedTemporalResolution,
    AbnormalBinauralProcessing,
    PhantomPercept,
    Audiogram,
    PureToneAverage,
    SpeechReceptionThreshold,
    OtoacousticEmission,
    AuditoryBrainstemResponse,
    HearingLoss,
    PeripheralPathology,
    CentralPathology,
    DamageMechanism,
    PerceptualDeficit,
    ClinicalMeasure,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Concept)]
pub enum PathologyCausalEvent {
    NoiseExposure,
    AgingDegeneration,
    Infection,
    Autoimmune,
    GeneticMutation,
    OHCDamage,
    IHCDamage,
    SynapseLoss,
    StriDegeneration,
    MiddleEarDysfunction,
    NeuralDegeneration,
    ThresholdShift,
    FrequencyResolutionLoss,
    TemporalSmearing,
    TinnitusGeneration,
    CommunicationDifficulty,
}
define_ontology! {
    /// Discrete category over pathology entities.
    pub PathologyOntology for PathologyCategory {
        entity: PathologyEntity, relation: PathologyRelation,
        being: AbstractObject,
        source: "Moller (2006); Gates & Mills (2005)",
        taxonomy: PathologyTaxonomy [
            (ConductiveHearingLoss, HearingLoss), (SensorineuralHearingLoss, HearingLoss), (MixedHearingLoss, HearingLoss), (AuditoryNeuropathy, HearingLoss), (CentralAuditoryProcessingDisorder, HearingLoss),
            (Otosclerosis, PeripheralPathology), (Presbycusis, PeripheralPathology), (NoiseInducedHearingLoss, PeripheralPathology), (MenieresDisease, PeripheralPathology), (Tinnitus, PeripheralPathology), (Hyperacusis, PeripheralPathology), (SuddenSensorineuralLoss, PeripheralPathology), (OtitisMedia, PeripheralPathology), (TympanicPerforation, PeripheralPathology), (Cholesteatoma, PeripheralPathology),
            (AcousticNeuroma, CentralPathology), (CentralAuditoryProcessingDisorder, CentralPathology),
            (HairCellLoss, DamageMechanism), (StereociliaDamage, DamageMechanism), (SynapticRibbonLoss, DamageMechanism), (StriaDysfunction, DamageMechanism), (OssicularFixation, DamageMechanism), (EndolymphaticHydrops, DamageMechanism), (DemyelinationVIII, DamageMechanism), (Excitotoxicity, DamageMechanism), (OxidativeStress, DamageMechanism),
            (ElevatedThreshold, PerceptualDeficit), (ReducedFrequencySelectivity, PerceptualDeficit), (LoudnessRecruitment, PerceptualDeficit), (PoorSpeechInNoise, PerceptualDeficit), (ReducedTemporalResolution, PerceptualDeficit), (AbnormalBinauralProcessing, PerceptualDeficit), (PhantomPercept, PerceptualDeficit),
            (Audiogram, ClinicalMeasure), (PureToneAverage, ClinicalMeasure), (SpeechReceptionThreshold, ClinicalMeasure), (OtoacousticEmission, ClinicalMeasure), (AuditoryBrainstemResponse, ClinicalMeasure),
        ],
        causation: PathologyCausalGraph for PathologyCausalEvent [
            (NoiseExposure, OHCDamage), (NoiseExposure, IHCDamage), (NoiseExposure, SynapseLoss),
            (AgingDegeneration, OHCDamage), (AgingDegeneration, StriDegeneration), (AgingDegeneration, NeuralDegeneration),
            (Infection, MiddleEarDysfunction), (GeneticMutation, OHCDamage), (GeneticMutation, IHCDamage),
            (OHCDamage, ThresholdShift), (OHCDamage, FrequencyResolutionLoss), (OHCDamage, TinnitusGeneration),
            (IHCDamage, ThresholdShift), (SynapseLoss, TemporalSmearing), (StriDegeneration, ThresholdShift), (MiddleEarDysfunction, ThresholdShift),
            (ThresholdShift, CommunicationDifficulty), (FrequencyResolutionLoss, CommunicationDifficulty), (TemporalSmearing, CommunicationDifficulty),
        ],
        opposition: PathologyOpposition [ (ConductiveHearingLoss, SensorineuralHearingLoss), (Tinnitus, Hyperacusis), (HairCellLoss, SynapticRibbonLoss) ],
    }
}
#[derive(Debug, Clone)]
pub struct TypicalSeverityDB;
impl Quality for TypicalSeverityDB {
    type Individual = PathologyEntity;
    type Value = f64;
    fn get(&self, individual: &PathologyEntity) -> Option<f64> {
        use PathologyEntity::*;
        match individual {
            Otosclerosis => Some(40.0),
            Presbycusis => Some(45.0),
            NoiseInducedHearingLoss => Some(50.0),
            MenieresDisease => Some(40.0),
            OtitisMedia => Some(25.0),
            TympanicPerforation => Some(30.0),
            AcousticNeuroma => Some(55.0),
            SuddenSensorineuralLoss => Some(60.0),
            _ => None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct PrevalencePercent;
impl Quality for PrevalencePercent {
    type Individual = PathologyEntity;
    type Value = f64;
    fn get(&self, individual: &PathologyEntity) -> Option<f64> {
        use PathologyEntity::*;
        match individual {
            Presbycusis => Some(33.0),
            NoiseInducedHearingLoss => Some(12.0),
            Tinnitus => Some(15.0),
            _ => None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct OAEsPresent;
impl Quality for OAEsPresent {
    type Individual = PathologyEntity;
    type Value = bool;
    fn get(&self, individual: &PathologyEntity) -> Option<bool> {
        use PathologyEntity::*;
        match individual {
            ConductiveHearingLoss => Some(true),
            SensorineuralHearingLoss => Some(false),
            AuditoryNeuropathy => Some(true),
            NoiseInducedHearingLoss => Some(false),
            Presbycusis => Some(false),
            _ => None,
        }
    }
}

pub struct NoiseCausesDifficulty;
impl Axiom for NoiseCausesDifficulty {
    fn description(&self) -> &str {
        "noise exposure transitively causes communication difficulty"
    }
    fn holds(&self) -> bool {
        use PathologyCausalEvent::*;
        causation::effects_of::<PathologyCausalGraph>(&NoiseExposure)
            .contains(&CommunicationDifficulty)
    }
}
pr4xis::register_axiom!(NoiseCausesDifficulty);
pub struct FiveHearingLossTypes;
impl Axiom for FiveHearingLossTypes {
    fn description(&self) -> &str {
        "five hearing loss types are classified"
    }
    fn holds(&self) -> bool {
        use PathologyEntity::*;
        [
            ConductiveHearingLoss,
            SensorineuralHearingLoss,
            MixedHearingLoss,
            AuditoryNeuropathy,
            CentralAuditoryProcessingDisorder,
        ]
        .iter()
        .all(|t| taxonomy::is_a::<PathologyTaxonomy>(t, &HearingLoss))
    }
}
pr4xis::register_axiom!(FiveHearingLossTypes);
pub struct PresbycusisMostPrevalent;
impl Axiom for PresbycusisMostPrevalent {
    fn description(&self) -> &str {
        "presbycusis has highest prevalence among modeled conditions"
    }
    fn holds(&self) -> bool {
        use PathologyEntity::*;
        let p = PrevalencePercent.get(&Presbycusis).unwrap();
        let n = PrevalencePercent.get(&NoiseInducedHearingLoss).unwrap();
        let t = PrevalencePercent.get(&Tinnitus).unwrap();
        p > n && p > t
    }
}
pr4xis::register_axiom!(PresbycusisMostPrevalent);
pub struct NeuropathyHasOAEs;
impl Axiom for NeuropathyHasOAEs {
    fn description(&self) -> &str {
        "auditory neuropathy has present OAEs (OHCs intact)"
    }
    fn holds(&self) -> bool {
        OAEsPresent.get(&PathologyEntity::AuditoryNeuropathy) == Some(true)
            && OAEsPresent.get(&PathologyEntity::SensorineuralHearingLoss) == Some(false)
    }
}
pr4xis::register_axiom!(NeuropathyHasOAEs);

impl Ontology for PathologyOntology {
    type Cat = PathologyCategory;
    type Qual = TypicalSeverityDB;
    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }
    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(NoiseCausesDifficulty),
            Box::new(FiveHearingLossTypes),
            Box::new(PresbycusisMostPrevalent),
            Box::new(NeuropathyHasOAEs),
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
    fn test_noise_causes_difficulty() {
        assert!(NoiseCausesDifficulty.holds());
    }
    #[test]
    fn test_five_hl_types() {
        assert!(FiveHearingLossTypes.holds());
    }
    #[test]
    fn test_neuropathy_has_oaes() {
        assert!(NeuropathyHasOAEs.holds());
    }
    #[test]
    fn test_conductive_opposes_sensorineural() {
        assert!(opposition::are_opposed::<PathologyOpposition>(
            &PathologyEntity::ConductiveHearingLoss,
            &PathologyEntity::SensorineuralHearingLoss
        ));
    }
    #[test]
    fn test_category_laws() {
        check_category_laws::<PathologyCategory>().unwrap();
    }
    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<PathologyTaxonomy>>().unwrap();
    }
    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<PathologyCausalGraph>>().unwrap();
    }
    #[test]
    fn test_presbycusis_most_prevalent() {
        assert!(PresbycusisMostPrevalent.holds());
    }
    #[test]
    fn test_presbycusis_prevalence() {
        assert_eq!(
            PrevalencePercent.get(&PathologyEntity::Presbycusis),
            Some(33.0)
        );
    }
    #[test]
    fn test_tinnitus_prevalence() {
        assert_eq!(
            PrevalencePercent.get(&PathologyEntity::Tinnitus),
            Some(15.0)
        );
    }
    #[test]
    fn test_otosclerosis_severity() {
        assert_eq!(
            TypicalSeverityDB.get(&PathologyEntity::Otosclerosis),
            Some(40.0)
        );
    }
    #[test]
    fn test_entity_count() {
        assert_eq!(PathologyEntity::variants().len(), 43);
    }
    #[test]
    fn test_ontology_validates() {
        PathologyOntology::validate().unwrap();
    }
    fn arb_entity() -> impl Strategy<Value = PathologyEntity> {
        (0..PathologyEntity::variants().len()).prop_map(|i| PathologyEntity::variants()[i])
    }
    proptest! { #[test] fn prop_taxonomy_reflexive(entity in arb_entity()) { prop_assert!(taxonomy::is_a::<PathologyTaxonomy>(&entity, &entity)); } }
}
