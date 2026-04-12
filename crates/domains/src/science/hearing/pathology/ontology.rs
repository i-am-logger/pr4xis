//! Hearing pathology ontology.
//!
//! Models hearing disorders, their mechanisms, and perceptual consequences.
//!
//! Key references:
//! - Møller 2006: Hearing: Anatomy, Physiology, and Disorders
//! - Gates & Mills 2005: Presbycusis (Lancet)
//! - Henderson et al. 2006: noise-induced hearing loss mechanisms
//! - Merchant & Rosowski 2008: conductive hearing loss
//! - Jastreboff 1990: neurophysiological model of tinnitus
//! - Eggermont & Roberts 2004: tinnitus neural mechanisms

use praxis::category::{Category, Entity, Relationship};
use praxis::ontology::reasoning::causation::{self, CausalDef};
use praxis::ontology::reasoning::opposition::{self, OppositionDef};
use praxis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use praxis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PathologyEntity {
    // Hearing loss types
    ConductiveHearingLoss,
    SensorineuralHearingLoss,
    MixedHearingLoss,
    AuditoryNeuropathy,
    CentralAuditoryProcessingDisorder,
    // Specific conditions
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
    // Damage mechanisms
    HairCellLoss,
    StereociliaDamage,
    SynapticRibbonLoss,
    StriaDysfunction,
    OssicularFixation,
    EndolymphaticHydrops,
    DemyelinationVIII,
    Excitotoxicity,
    OxidativeStress,
    // Perceptual consequences
    ElevatedThreshold,
    ReducedFrequencySelectivity,
    LoudnessRecruitment,
    PoorSpeechInNoise,
    ReducedTemporalResolution,
    AbnormalBinauralProcessing,
    PhantomPercept,
    // Clinical measures
    Audiogram,
    PureToneAverage,
    SpeechReceptionThreshold,
    OtoacousticEmission,
    AuditoryBrainstemResponse,
    // Abstract categories
    HearingLoss,
    PeripheralPathology,
    CentralPathology,
    DamageMechanism,
    PerceptualDeficit,
    ClinicalMeasure,
}

impl Entity for PathologyEntity {
    fn variants() -> Vec<Self> {
        use PathologyEntity::*;
        vec![
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
        ]
    }
}

// ---------------------------------------------------------------------------
// Taxonomy
// ---------------------------------------------------------------------------

pub struct PathologyTaxonomy;

impl TaxonomyDef for PathologyTaxonomy {
    type Entity = PathologyEntity;

    fn relations() -> Vec<(PathologyEntity, PathologyEntity)> {
        use PathologyEntity::*;
        vec![
            // Hearing loss types
            (ConductiveHearingLoss, HearingLoss),
            (SensorineuralHearingLoss, HearingLoss),
            (MixedHearingLoss, HearingLoss),
            (AuditoryNeuropathy, HearingLoss),
            (CentralAuditoryProcessingDisorder, HearingLoss),
            // Peripheral conditions
            (Otosclerosis, PeripheralPathology),
            (Presbycusis, PeripheralPathology),
            (NoiseInducedHearingLoss, PeripheralPathology),
            (MenieresDisease, PeripheralPathology),
            (Tinnitus, PeripheralPathology),
            (Hyperacusis, PeripheralPathology),
            (SuddenSensorineuralLoss, PeripheralPathology),
            (OtitisMedia, PeripheralPathology),
            (TympanicPerforation, PeripheralPathology),
            (Cholesteatoma, PeripheralPathology),
            // Central conditions
            (AcousticNeuroma, CentralPathology),
            (CentralAuditoryProcessingDisorder, CentralPathology),
            // Damage mechanisms
            (HairCellLoss, DamageMechanism),
            (StereociliaDamage, DamageMechanism),
            (SynapticRibbonLoss, DamageMechanism),
            (StriaDysfunction, DamageMechanism),
            (OssicularFixation, DamageMechanism),
            (EndolymphaticHydrops, DamageMechanism),
            (DemyelinationVIII, DamageMechanism),
            (Excitotoxicity, DamageMechanism),
            (OxidativeStress, DamageMechanism),
            // Perceptual deficits
            (ElevatedThreshold, PerceptualDeficit),
            (ReducedFrequencySelectivity, PerceptualDeficit),
            (LoudnessRecruitment, PerceptualDeficit),
            (PoorSpeechInNoise, PerceptualDeficit),
            (ReducedTemporalResolution, PerceptualDeficit),
            (AbnormalBinauralProcessing, PerceptualDeficit),
            (PhantomPercept, PerceptualDeficit),
            // Clinical measures
            (Audiogram, ClinicalMeasure),
            (PureToneAverage, ClinicalMeasure),
            (SpeechReceptionThreshold, ClinicalMeasure),
            (OtoacousticEmission, ClinicalMeasure),
            (AuditoryBrainstemResponse, ClinicalMeasure),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

impl Entity for PathologyCausalEvent {
    fn variants() -> Vec<Self> {
        use PathologyCausalEvent::*;
        vec![
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
        ]
    }
}

pub struct PathologyCausalGraph;

impl CausalDef for PathologyCausalGraph {
    type Entity = PathologyCausalEvent;

    fn relations() -> Vec<(PathologyCausalEvent, PathologyCausalEvent)> {
        use PathologyCausalEvent::*;
        vec![
            // Noise → OHC and IHC damage
            (NoiseExposure, OHCDamage),
            (NoiseExposure, IHCDamage),
            (NoiseExposure, SynapseLoss),
            // Aging → multiple degeneration pathways
            (AgingDegeneration, OHCDamage),
            (AgingDegeneration, StriDegeneration),
            (AgingDegeneration, NeuralDegeneration),
            // Infection → middle ear
            (Infection, MiddleEarDysfunction),
            // Genetic → various
            (GeneticMutation, OHCDamage),
            (GeneticMutation, IHCDamage),
            // OHC damage → loss of amplification
            (OHCDamage, ThresholdShift),
            (OHCDamage, FrequencyResolutionLoss),
            (OHCDamage, TinnitusGeneration),
            // IHC damage → severe threshold shift
            (IHCDamage, ThresholdShift),
            // Synapse loss → temporal smearing
            (SynapseLoss, TemporalSmearing),
            // Stria → reduced EP → threshold shift
            (StriDegeneration, ThresholdShift),
            // Middle ear → conductive loss
            (MiddleEarDysfunction, ThresholdShift),
            // Perceptual consequences
            (ThresholdShift, CommunicationDifficulty),
            (FrequencyResolutionLoss, CommunicationDifficulty),
            (TemporalSmearing, CommunicationDifficulty),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathologyRelation {
    pub from: PathologyEntity,
    pub to: PathologyEntity,
}

impl Relationship for PathologyRelation {
    type Object = PathologyEntity;
    fn source(&self) -> PathologyEntity {
        self.from
    }
    fn target(&self) -> PathologyEntity {
        self.to
    }
}

pub struct PathologyCategory;

impl Category for PathologyCategory {
    type Object = PathologyEntity;
    type Morphism = PathologyRelation;

    fn identity(obj: &PathologyEntity) -> PathologyRelation {
        PathologyRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &PathologyRelation, g: &PathologyRelation) -> Option<PathologyRelation> {
        if f.to != g.from {
            return None;
        }
        Some(PathologyRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<PathologyRelation> {
        let v = PathologyEntity::variants();
        v.iter()
            .flat_map(|&a| v.iter().map(move |&b| PathologyRelation { from: a, to: b }))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Typical hearing loss severity (dB HL) for each condition.
///
/// WHO grading: mild 26-40, moderate 41-60, severe 61-80, profound >80.
#[derive(Debug, Clone)]
pub struct TypicalSeverityDB;

impl Quality for TypicalSeverityDB {
    type Individual = PathologyEntity;
    type Value = f64;

    fn get(&self, individual: &PathologyEntity) -> Option<f64> {
        use PathologyEntity::*;
        match individual {
            Otosclerosis => Some(40.0),            // moderate conductive
            Presbycusis => Some(45.0),             // moderate SNHL
            NoiseInducedHearingLoss => Some(50.0), // moderate SNHL (4 kHz notch)
            MenieresDisease => Some(40.0),         // fluctuating, low-frequency
            OtitisMedia => Some(25.0),             // mild conductive
            TympanicPerforation => Some(30.0),     // mild-moderate conductive
            AcousticNeuroma => Some(55.0),         // unilateral SNHL
            SuddenSensorineuralLoss => Some(60.0), // variable, often severe
            _ => None,
        }
    }
}

/// Prevalence (percent) of hearing conditions in their target populations.
///
/// - Presbycusis: ~33% of adults over 65 (Gates & Mills 2005)
/// - NoiseInducedHearingLoss: ~12% of exposed workers (WHO 2021)
/// - Tinnitus: ~15% of adults (WHO 2021)
#[derive(Debug, Clone)]
pub struct PrevalencePercent;

impl Quality for PrevalencePercent {
    type Individual = PathologyEntity;
    type Value = f64;

    fn get(&self, individual: &PathologyEntity) -> Option<f64> {
        use PathologyEntity::*;
        match individual {
            Presbycusis => Some(33.0),             // age > 65
            NoiseInducedHearingLoss => Some(12.0), // exposed workers
            Tinnitus => Some(15.0),                // general adult population
            _ => None,
        }
    }
}

/// Whether OAEs are present (indicates OHC function).
///
/// Kemp 1978: OAEs as indicator of OHC function.
#[derive(Debug, Clone)]
pub struct OAEsPresent;

impl Quality for OAEsPresent {
    type Individual = PathologyEntity;
    type Value = bool;

    fn get(&self, individual: &PathologyEntity) -> Option<bool> {
        use PathologyEntity::*;
        match individual {
            ConductiveHearingLoss => Some(true), // OHCs intact, but OAEs may not pass middle ear
            SensorineuralHearingLoss => Some(false), // OHC damage → no OAEs
            AuditoryNeuropathy => Some(true),    // OHCs intact, neural problem
            NoiseInducedHearingLoss => Some(false), // OHC damage
            Presbycusis => Some(false),          // OHC loss
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in hearing pathology.
///
/// - ConductiveHearingLoss vs SensorineuralHearingLoss: middle ear vs inner ear
/// - Tinnitus vs Hyperacusis: phantom sound vs sound sensitivity
/// - HairCellLoss vs SynapticRibbonLoss: sensory vs neural damage
pub struct PathologyOpposition;

impl OppositionDef for PathologyOpposition {
    type Entity = PathologyEntity;

    fn pairs() -> Vec<(PathologyEntity, PathologyEntity)> {
        use PathologyEntity::*;
        vec![
            (ConductiveHearingLoss, SensorineuralHearingLoss),
            (Tinnitus, Hyperacusis),
            (HairCellLoss, SynapticRibbonLoss),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

pub struct PathologyTaxonomyIsDAG;
impl Axiom for PathologyTaxonomyIsDAG {
    fn description(&self) -> &str {
        "pathology taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        taxonomy::NoCycles::<PathologyTaxonomy>::new().holds()
    }
}

pub struct PathologyCausalGraphIsAsymmetric;
impl Axiom for PathologyCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "pathology causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<PathologyCausalGraph>::new().holds()
    }
}

pub struct PathologyCausalNoSelfCause;
impl Axiom for PathologyCausalNoSelfCause {
    fn description(&self) -> &str {
        "no pathology event causes itself"
    }
    fn holds(&self) -> bool {
        causation::NoSelfCausation::<PathologyCausalGraph>::new().holds()
    }
}

/// Noise exposure transitively causes communication difficulty.
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

/// Five hearing loss types are classified.
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

/// Opposition is symmetric.
pub struct PathologyOppositionSymmetric;
impl Axiom for PathologyOppositionSymmetric {
    fn description(&self) -> &str {
        "pathology opposition is symmetric"
    }
    fn holds(&self) -> bool {
        opposition::Symmetric::<PathologyOpposition>::new().holds()
    }
}

/// Opposition is irreflexive.
pub struct PathologyOppositionIrreflexive;
impl Axiom for PathologyOppositionIrreflexive {
    fn description(&self) -> &str {
        "pathology opposition is irreflexive"
    }
    fn holds(&self) -> bool {
        opposition::Irreflexive::<PathologyOpposition>::new().holds()
    }
}

/// Presbycusis is the most prevalent condition (Gates & Mills 2005).
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

/// Auditory neuropathy has present OAEs (distinguishes it from SNHL).
///
/// Starr et al. 1996.
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

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

pub struct PathologyOntology;

impl Ontology for PathologyOntology {
    type Cat = PathologyCategory;
    type Qual = TypicalSeverityDB;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(PathologyTaxonomyIsDAG),
            Box::new(PathologyCausalGraphIsAsymmetric),
            Box::new(PathologyCausalNoSelfCause),
            Box::new(NoiseCausesDifficulty),
            Box::new(FiveHearingLossTypes),
            Box::new(PresbycusisMostPrevalent),
            Box::new(NeuropathyHasOAEs),
            Box::new(PathologyOppositionSymmetric),
            Box::new(PathologyOppositionIrreflexive),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis::category::validate::check_category_laws;
    use praxis::ontology::reasoning::causation::CausalCategory;
    use praxis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(PathologyTaxonomyIsDAG.holds());
    }
    #[test]
    fn test_causal_asymmetric() {
        assert!(PathologyCausalGraphIsAsymmetric.holds());
    }
    #[test]
    fn test_causal_no_self() {
        assert!(PathologyCausalNoSelfCause.holds());
    }
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
    fn test_opposition_symmetric() {
        assert!(PathologyOppositionSymmetric.holds());
    }
    #[test]
    fn test_opposition_irreflexive() {
        assert!(PathologyOppositionIrreflexive.holds());
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
    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<PathologyTaxonomy>(&entity, &entity));
        }
    }
}
