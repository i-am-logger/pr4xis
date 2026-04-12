//! Audiology ontology.
//!
//! Models clinical audiological assessment and rehabilitation.
//!
//! Key references:
//! - Katz et al. 2015: Handbook of Clinical Audiology (7th ed.)
//! - Stach 2010: Clinical Audiology
//! - Carhart 1950: clinical bone conduction testing
//! - Jerger 1970: tympanometry Type A/B/C classification
//! - ASHA 2005: Guidelines for Manual Pure-Tone Threshold Audiometry
//! - Kemp 1978: otoacoustic emissions discovery

use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::mereology::{self, MereologyDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum AudiologyEntity {
    // Pure tone audiometry
    PureToneAudiometry,
    AirConductionTest,
    BoneConductionTest,
    MaskingProcedure,
    AirBoneGap,
    PureToneAverage,
    // Speech audiometry
    SpeechAudiometry,
    SpeechRecognitionThreshold,
    WordRecognitionScore,
    SpeechInNoiseTest,
    QuickSIN,
    HINT,
    // Immittance
    Tympanometry,
    TympanogramTypeA,
    TympanogramTypeB,
    TympanogramTypeC,
    AcousticReflex,
    AcousticReflexDecay,
    StaticCompliance,
    // Otoacoustic emissions
    TransientOAE,
    DistortionProductOAE,
    OAEScreening,
    // Evoked potentials
    AuditoryBrainstemResponse,
    WaveI,
    WaveIII,
    WaveV,
    ElectroCochleography,
    AuditoryLateResponse,
    // Rehabilitation
    AuralRehabilitation,
    HearingAidFitting,
    RealEarVerification,
    CochlearImplantMapping,
    AuditoryTraining,
    CommunicationStrategy,
    // Workflow
    CaseHistory,
    Otoscopy,
    Referral,
    Counseling,
    // Abstract categories
    DiagnosticTest,
    SpeechTest,
    ImmittanceTest,
    EmissionTest,
    EvokedPotentialTest,
    RehabilitationProcedure,
    ClinicalWorkflow,
}

// ---------------------------------------------------------------------------
// Taxonomy
// ---------------------------------------------------------------------------

pub struct AudiologyTaxonomy;

impl TaxonomyDef for AudiologyTaxonomy {
    type Entity = AudiologyEntity;

    fn relations() -> Vec<(AudiologyEntity, AudiologyEntity)> {
        use AudiologyEntity::*;
        vec![
            // Diagnostic tests
            (PureToneAudiometry, DiagnosticTest),
            (AirConductionTest, PureToneAudiometry),
            (BoneConductionTest, PureToneAudiometry),
            // Speech tests
            (SpeechAudiometry, SpeechTest),
            (SpeechRecognitionThreshold, SpeechTest),
            (WordRecognitionScore, SpeechTest),
            (SpeechInNoiseTest, SpeechTest),
            (QuickSIN, SpeechInNoiseTest),
            (HINT, SpeechInNoiseTest),
            // Immittance tests
            (Tympanometry, ImmittanceTest),
            (AcousticReflex, ImmittanceTest),
            (AcousticReflexDecay, ImmittanceTest),
            // Tympanogram types (Jerger 1970)
            (TympanogramTypeA, Tympanometry),
            (TympanogramTypeB, Tympanometry),
            (TympanogramTypeC, Tympanometry),
            // Emission tests
            (TransientOAE, EmissionTest),
            (DistortionProductOAE, EmissionTest),
            (OAEScreening, EmissionTest),
            // Evoked potentials
            (AuditoryBrainstemResponse, EvokedPotentialTest),
            (ElectroCochleography, EvokedPotentialTest),
            (AuditoryLateResponse, EvokedPotentialTest),
            // ABR waves
            (WaveI, AuditoryBrainstemResponse),
            (WaveIII, AuditoryBrainstemResponse),
            (WaveV, AuditoryBrainstemResponse),
            // Rehabilitation
            (AuralRehabilitation, RehabilitationProcedure),
            (HearingAidFitting, RehabilitationProcedure),
            (RealEarVerification, RehabilitationProcedure),
            (CochlearImplantMapping, RehabilitationProcedure),
            (AuditoryTraining, RehabilitationProcedure),
            (CommunicationStrategy, RehabilitationProcedure),
            // Clinical workflow
            (CaseHistory, ClinicalWorkflow),
            (Otoscopy, ClinicalWorkflow),
            (Referral, ClinicalWorkflow),
            (Counseling, ClinicalWorkflow),
        ]
    }
}

// ---------------------------------------------------------------------------
// Mereology (has-a / part-whole)
// ---------------------------------------------------------------------------

/// Part-whole relationships for audiological assessment.
///
/// Diagnostic tests are composed of their constituent procedures:
/// - Diagnostic test battery includes pure tone and speech audiometry
/// - Pure tone audiometry includes air conduction, bone conduction, and masking
/// - ABR contains waves I, III, and V
///
/// Katz et al. 2015; Stach 2010.
pub struct AudiologyMereology;

impl MereologyDef for AudiologyMereology {
    type Entity = AudiologyEntity;

    fn relations() -> Vec<(AudiologyEntity, AudiologyEntity)> {
        use AudiologyEntity::*;
        vec![
            // Diagnostic test battery
            (DiagnosticTest, PureToneAudiometry),
            (DiagnosticTest, SpeechAudiometry),
            // Pure tone audiometry components
            (PureToneAudiometry, AirConductionTest),
            (PureToneAudiometry, BoneConductionTest),
            (PureToneAudiometry, MaskingProcedure),
            // ABR wave components
            (AuditoryBrainstemResponse, WaveI),
            (AuditoryBrainstemResponse, WaveIII),
            (AuditoryBrainstemResponse, WaveV),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Clinical assessment workflow.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum AudiologyCausalEvent {
    PatientPresents,
    HistoryTaken,
    OtoscopyPerformed,
    PureToneCompleted,
    SpeechTestCompleted,
    ImmittanceCompleted,
    OAECompleted,
    DiagnosisMade,
    TreatmentPlanDeveloped,
    DeviceFitted,
    OutcomeVerified,
}

pub struct AudiologyCausalGraph;

impl CausalDef for AudiologyCausalGraph {
    type Entity = AudiologyCausalEvent;

    fn relations() -> Vec<(AudiologyCausalEvent, AudiologyCausalEvent)> {
        use AudiologyCausalEvent::*;
        vec![
            (PatientPresents, HistoryTaken),
            (HistoryTaken, OtoscopyPerformed),
            (OtoscopyPerformed, PureToneCompleted),
            (OtoscopyPerformed, ImmittanceCompleted),
            (PureToneCompleted, SpeechTestCompleted),
            (PureToneCompleted, OAECompleted),
            (SpeechTestCompleted, DiagnosisMade),
            (ImmittanceCompleted, DiagnosisMade),
            (OAECompleted, DiagnosisMade),
            (DiagnosisMade, TreatmentPlanDeveloped),
            (TreatmentPlanDeveloped, DeviceFitted),
            (DeviceFitted, OutcomeVerified),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over audiology entities.
    pub AudiologyCategory {
        entity: AudiologyEntity,
        relation: AudiologyRelation,
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// ABR wave latency (milliseconds).
///
/// - Wave I: ~1.5 ms (distal auditory nerve)
/// - Wave III: ~3.5 ms (cochlear nucleus)
/// - Wave V: ~5.5 ms (lateral lemniscus/inferior colliculus)
///
/// Jewett & Williston 1971; Stach 2010.
#[derive(Debug, Clone)]
pub struct ABRLatencyMs;

impl Quality for ABRLatencyMs {
    type Individual = AudiologyEntity;
    type Value = f64;

    fn get(&self, individual: &AudiologyEntity) -> Option<f64> {
        use AudiologyEntity::*;
        match individual {
            WaveI => Some(1.5),
            WaveIII => Some(3.5),
            WaveV => Some(5.5),
            _ => None,
        }
    }
}

/// Typical test duration in minutes.
///
/// - PureToneAudiometry: ~20 min (Stach 2010)
/// - ABR: ~30 min (Stach 2010)
/// - Tympanometry: ~2 min (Stach 2010)
#[derive(Debug, Clone)]
pub struct TestDurationMinutes;

impl Quality for TestDurationMinutes {
    type Individual = AudiologyEntity;
    type Value = f64;

    fn get(&self, individual: &AudiologyEntity) -> Option<f64> {
        use AudiologyEntity::*;
        match individual {
            PureToneAudiometry => Some(20.0),
            AuditoryBrainstemResponse => Some(30.0),
            Tympanometry => Some(2.0),
            _ => None,
        }
    }
}

/// Whether the test requires patient cooperation (behavioral vs objective).
#[derive(Debug, Clone)]
pub struct RequiresCooperation;

impl Quality for RequiresCooperation {
    type Individual = AudiologyEntity;
    type Value = bool;

    fn get(&self, individual: &AudiologyEntity) -> Option<bool> {
        use AudiologyEntity::*;
        match individual {
            PureToneAudiometry | AirConductionTest | BoneConductionTest => Some(true),
            SpeechAudiometry | SpeechRecognitionThreshold | WordRecognitionScore => Some(true),
            AuditoryBrainstemResponse => Some(false), // objective
            TransientOAE | DistortionProductOAE | OAEScreening => Some(false), // objective
            Tympanometry => Some(false),              // objective
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in audiology.
///
/// - AirConductionTest vs BoneConductionTest: two audiometric methods
/// - TransientOAE vs DistortionProductOAE: two emission types
/// - PureToneAudiometry vs SpeechAudiometry: tone vs speech testing
pub struct AudiologyOpposition;

impl OppositionDef for AudiologyOpposition {
    type Entity = AudiologyEntity;

    fn pairs() -> Vec<(AudiologyEntity, AudiologyEntity)> {
        use AudiologyEntity::*;
        vec![
            (AirConductionTest, BoneConductionTest),
            (TransientOAE, DistortionProductOAE),
            (PureToneAudiometry, SpeechAudiometry),
        ]
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

pub struct AudiologyTaxonomyIsDAG;
impl Axiom for AudiologyTaxonomyIsDAG {
    fn description(&self) -> &str {
        "audiology taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        taxonomy::NoCycles::<AudiologyTaxonomy>::new().holds()
    }
}

/// Mereology is a DAG.
pub struct AudiologyMereologyIsDAG;
impl Axiom for AudiologyMereologyIsDAG {
    fn description(&self) -> &str {
        "audiology mereology is a DAG"
    }
    fn holds(&self) -> bool {
        mereology::NoCycles::<AudiologyMereology>::new().holds()
    }
}

/// Diagnostic test transitively contains air and bone conduction tests.
///
/// DiagnosticTest has-a PureToneAudiometry, which has-a AirConductionTest
/// and BoneConductionTest.
pub struct DiagnosticTestContainsConductionTests;
impl Axiom for DiagnosticTestContainsConductionTests {
    fn description(&self) -> &str {
        "diagnostic test transitively contains air and bone conduction tests"
    }
    fn holds(&self) -> bool {
        use AudiologyEntity::*;
        let parts = mereology::parts_of::<AudiologyMereology>(&DiagnosticTest);
        parts.contains(&AirConductionTest) && parts.contains(&BoneConductionTest)
    }
}

pub struct AudiologyCausalIsAsymmetric;
impl Axiom for AudiologyCausalIsAsymmetric {
    fn description(&self) -> &str {
        "audiology causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<AudiologyCausalGraph>::new().holds()
    }
}

/// ABR wave latencies are ordered: I < III < V.
///
/// Jewett & Williston 1971.
pub struct ABRWavesOrdered;
impl Axiom for ABRWavesOrdered {
    fn description(&self) -> &str {
        "ABR wave latencies are ordered (I < III < V)"
    }
    fn holds(&self) -> bool {
        use AudiologyEntity::*;
        let l = ABRLatencyMs;
        l.get(&WaveI).unwrap() < l.get(&WaveIII).unwrap()
            && l.get(&WaveIII).unwrap() < l.get(&WaveV).unwrap()
    }
}

/// Three tympanogram types are classified (Jerger 1970).
pub struct ThreeTympanogramTypes;
impl Axiom for ThreeTympanogramTypes {
    fn description(&self) -> &str {
        "three tympanogram types (A, B, C) are classified"
    }
    fn holds(&self) -> bool {
        use AudiologyEntity::*;
        [TympanogramTypeA, TympanogramTypeB, TympanogramTypeC]
            .iter()
            .all(|t| taxonomy::is_a::<AudiologyTaxonomy>(t, &Tympanometry))
    }
}

/// ABR takes longer than tympanometry (Stach 2010).
pub struct ABRLongerThanTympanometry;
impl Axiom for ABRLongerThanTympanometry {
    fn description(&self) -> &str {
        "ABR takes longer than tympanometry"
    }
    fn holds(&self) -> bool {
        use AudiologyEntity::*;
        TestDurationMinutes.get(&AuditoryBrainstemResponse).unwrap()
            > TestDurationMinutes.get(&Tympanometry).unwrap()
    }
}

/// ABR is an objective test (no cooperation needed).
pub struct ABRIsObjective;
impl Axiom for ABRIsObjective {
    fn description(&self) -> &str {
        "ABR does not require patient cooperation"
    }
    fn holds(&self) -> bool {
        RequiresCooperation.get(&AudiologyEntity::AuditoryBrainstemResponse) == Some(false)
    }
}

/// Patient presentation transitively causes outcome verification.
pub struct FullClinicalPathway;
impl Axiom for FullClinicalPathway {
    fn description(&self) -> &str {
        "patient presentation transitively leads to outcome verification"
    }
    fn holds(&self) -> bool {
        use AudiologyCausalEvent::*;
        causation::effects_of::<AudiologyCausalGraph>(&PatientPresents).contains(&OutcomeVerified)
    }
}

/// Opposition is symmetric.
pub struct AudiologyOppositionSymmetric;
impl Axiom for AudiologyOppositionSymmetric {
    fn description(&self) -> &str {
        "audiology opposition is symmetric"
    }
    fn holds(&self) -> bool {
        opposition::Symmetric::<AudiologyOpposition>::new().holds()
    }
}

/// Opposition is irreflexive.
pub struct AudiologyOppositionIrreflexive;
impl Axiom for AudiologyOppositionIrreflexive {
    fn description(&self) -> &str {
        "audiology opposition is irreflexive"
    }
    fn holds(&self) -> bool {
        opposition::Irreflexive::<AudiologyOpposition>::new().holds()
    }
}

/// QuickSIN is-a speech-in-noise test is-a speech test.
pub struct QuickSINSubsumption;
impl Axiom for QuickSINSubsumption {
    fn description(&self) -> &str {
        "QuickSIN is-a speech-in-noise test is-a speech test"
    }
    fn holds(&self) -> bool {
        use AudiologyEntity::*;
        taxonomy::is_a::<AudiologyTaxonomy>(&QuickSIN, &SpeechInNoiseTest)
            && taxonomy::is_a::<AudiologyTaxonomy>(&SpeechInNoiseTest, &SpeechTest)
            && taxonomy::is_a::<AudiologyTaxonomy>(&QuickSIN, &SpeechTest)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

pub struct AudiologyOntology;

impl Ontology for AudiologyOntology {
    type Cat = AudiologyCategory;
    type Qual = ABRLatencyMs;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(AudiologyTaxonomyIsDAG),
            Box::new(AudiologyMereologyIsDAG),
            Box::new(DiagnosticTestContainsConductionTests),
            Box::new(AudiologyCausalIsAsymmetric),
            Box::new(ABRWavesOrdered),
            Box::new(ThreeTympanogramTypes),
            Box::new(ABRLongerThanTympanometry),
            Box::new(ABRIsObjective),
            Box::new(FullClinicalPathway),
            Box::new(QuickSINSubsumption),
            Box::new(AudiologyOppositionSymmetric),
            Box::new(AudiologyOppositionIrreflexive),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::mereology::MereologyCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_taxonomy_dag() {
        assert!(AudiologyTaxonomyIsDAG.holds());
    }
    #[test]
    fn test_mereology_is_dag() {
        assert!(AudiologyMereologyIsDAG.holds());
    }
    #[test]
    fn test_diagnostic_test_contains_conduction_tests() {
        assert!(DiagnosticTestContainsConductionTests.holds());
    }
    #[test]
    fn test_causal_asymmetric() {
        assert!(AudiologyCausalIsAsymmetric.holds());
    }
    #[test]
    fn test_abr_waves_ordered() {
        assert!(ABRWavesOrdered.holds());
    }
    #[test]
    fn test_three_tympanograms() {
        assert!(ThreeTympanogramTypes.holds());
    }
    #[test]
    fn test_abr_objective() {
        assert!(ABRIsObjective.holds());
    }
    #[test]
    fn test_full_pathway() {
        assert!(FullClinicalPathway.holds());
    }
    #[test]
    fn test_quicksin_subsumption() {
        assert!(QuickSINSubsumption.holds());
    }
    #[test]
    fn test_opposition_symmetric() {
        assert!(AudiologyOppositionSymmetric.holds());
    }
    #[test]
    fn test_opposition_irreflexive() {
        assert!(AudiologyOppositionIrreflexive.holds());
    }
    #[test]
    fn test_air_opposes_bone_conduction() {
        assert!(opposition::are_opposed::<AudiologyOpposition>(
            &AudiologyEntity::AirConductionTest,
            &AudiologyEntity::BoneConductionTest
        ));
    }

    #[test]
    fn test_category_laws() {
        check_category_laws::<AudiologyCategory>().unwrap();
    }
    #[test]
    fn test_taxonomy_laws() {
        check_category_laws::<TaxonomyCategory<AudiologyTaxonomy>>().unwrap();
    }
    #[test]
    fn test_causal_laws() {
        check_category_laws::<CausalCategory<AudiologyCausalGraph>>().unwrap();
    }
    #[test]
    fn test_mereology_category_laws() {
        check_category_laws::<MereologyCategory<AudiologyMereology>>().unwrap();
    }

    #[test]
    fn test_abr_longer_than_tympanometry() {
        assert!(ABRLongerThanTympanometry.holds());
    }
    #[test]
    fn test_pta_duration() {
        assert_eq!(
            TestDurationMinutes.get(&AudiologyEntity::PureToneAudiometry),
            Some(20.0)
        );
    }
    #[test]
    fn test_tympanometry_duration() {
        assert_eq!(
            TestDurationMinutes.get(&AudiologyEntity::Tympanometry),
            Some(2.0)
        );
    }
    #[test]
    fn test_wave_v_latency() {
        assert_eq!(ABRLatencyMs.get(&AudiologyEntity::WaveV), Some(5.5));
    }
    #[test]
    fn test_entity_count() {
        assert_eq!(AudiologyEntity::variants().len(), 45);
    }
    #[test]
    fn test_ontology_validates() {
        AudiologyOntology::validate().unwrap();
    }

    fn arb_entity() -> impl Strategy<Value = AudiologyEntity> {
        (0..AudiologyEntity::variants().len()).prop_map(|i| AudiologyEntity::variants()[i])
    }
    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<AudiologyTaxonomy>(&entity, &entity));
        }
    }
}
