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
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::mereology;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum AudiologyEntity {
    PureToneAudiometry,
    AirConductionTest,
    BoneConductionTest,
    MaskingProcedure,
    AirBoneGap,
    PureToneAverage,
    SpeechAudiometry,
    SpeechRecognitionThreshold,
    WordRecognitionScore,
    SpeechInNoiseTest,
    QuickSIN,
    HINT,
    Tympanometry,
    TympanogramTypeA,
    TympanogramTypeB,
    TympanogramTypeC,
    AcousticReflex,
    AcousticReflexDecay,
    StaticCompliance,
    TransientOAE,
    DistortionProductOAE,
    OAEScreening,
    AuditoryBrainstemResponse,
    WaveI,
    WaveIII,
    WaveV,
    ElectroCochleography,
    AuditoryLateResponse,
    AuralRehabilitation,
    HearingAidFitting,
    RealEarVerification,
    CochlearImplantMapping,
    AuditoryTraining,
    CommunicationStrategy,
    CaseHistory,
    Otoscopy,
    Referral,
    Counseling,
    DiagnosticTest,
    SpeechTest,
    ImmittanceTest,
    EmissionTest,
    EvokedPotentialTest,
    RehabilitationProcedure,
    ClinicalWorkflow,
}

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

define_ontology! {
    /// Discrete category over audiology entities.
    pub AudiologyOntology for AudiologyCategory {
        entity: AudiologyEntity,
        relation: AudiologyRelation,

        taxonomy: AudiologyTaxonomy [
            (PureToneAudiometry, DiagnosticTest), (AirConductionTest, PureToneAudiometry), (BoneConductionTest, PureToneAudiometry),
            (SpeechAudiometry, SpeechTest), (SpeechRecognitionThreshold, SpeechTest), (WordRecognitionScore, SpeechTest),
            (SpeechInNoiseTest, SpeechTest), (QuickSIN, SpeechInNoiseTest), (HINT, SpeechInNoiseTest),
            (Tympanometry, ImmittanceTest), (AcousticReflex, ImmittanceTest), (AcousticReflexDecay, ImmittanceTest),
            (TympanogramTypeA, Tympanometry), (TympanogramTypeB, Tympanometry), (TympanogramTypeC, Tympanometry),
            (TransientOAE, EmissionTest), (DistortionProductOAE, EmissionTest), (OAEScreening, EmissionTest),
            (AuditoryBrainstemResponse, EvokedPotentialTest), (ElectroCochleography, EvokedPotentialTest), (AuditoryLateResponse, EvokedPotentialTest),
            (WaveI, AuditoryBrainstemResponse), (WaveIII, AuditoryBrainstemResponse), (WaveV, AuditoryBrainstemResponse),
            (AuralRehabilitation, RehabilitationProcedure), (HearingAidFitting, RehabilitationProcedure),
            (RealEarVerification, RehabilitationProcedure), (CochlearImplantMapping, RehabilitationProcedure),
            (AuditoryTraining, RehabilitationProcedure), (CommunicationStrategy, RehabilitationProcedure),
            (CaseHistory, ClinicalWorkflow), (Otoscopy, ClinicalWorkflow), (Referral, ClinicalWorkflow), (Counseling, ClinicalWorkflow),
        ],

        mereology: AudiologyMereology [
            (DiagnosticTest, PureToneAudiometry), (DiagnosticTest, SpeechAudiometry),
            (PureToneAudiometry, AirConductionTest), (PureToneAudiometry, BoneConductionTest), (PureToneAudiometry, MaskingProcedure),
            (AuditoryBrainstemResponse, WaveI), (AuditoryBrainstemResponse, WaveIII), (AuditoryBrainstemResponse, WaveV),
        ],

        causation: AudiologyCausalGraph for AudiologyCausalEvent [
            (PatientPresents, HistoryTaken), (HistoryTaken, OtoscopyPerformed),
            (OtoscopyPerformed, PureToneCompleted), (OtoscopyPerformed, ImmittanceCompleted),
            (PureToneCompleted, SpeechTestCompleted), (PureToneCompleted, OAECompleted),
            (SpeechTestCompleted, DiagnosisMade), (ImmittanceCompleted, DiagnosisMade), (OAECompleted, DiagnosisMade),
            (DiagnosisMade, TreatmentPlanDeveloped), (TreatmentPlanDeveloped, DeviceFitted), (DeviceFitted, OutcomeVerified),
        ],

        opposition: AudiologyOpposition [
            (AirConductionTest, BoneConductionTest), (TransientOAE, DistortionProductOAE), (PureToneAudiometry, SpeechAudiometry),
        ],
    }
}

// Qualities

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
            AuditoryBrainstemResponse => Some(false),
            TransientOAE | DistortionProductOAE | OAEScreening => Some(false),
            Tympanometry => Some(false),
            _ => None,
        }
    }
}

// Axioms

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

pub struct ABRIsObjective;
impl Axiom for ABRIsObjective {
    fn description(&self) -> &str {
        "ABR does not require patient cooperation"
    }
    fn holds(&self) -> bool {
        RequiresCooperation.get(&AudiologyEntity::AuditoryBrainstemResponse) == Some(false)
    }
}

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

// Ontology impl

impl Ontology for AudiologyOntology {
    type Cat = AudiologyCategory;
    type Qual = ABRLatencyMs;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(DiagnosticTestContainsConductionTests),
            Box::new(ABRWavesOrdered),
            Box::new(ThreeTympanogramTypes),
            Box::new(ABRLongerThanTympanometry),
            Box::new(ABRIsObjective),
            Box::new(FullClinicalPathway),
            Box::new(QuickSINSubsumption),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::mereology::MereologyCategory;
    use pr4xis::ontology::reasoning::opposition;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_diagnostic_test_contains_conduction_tests() {
        assert!(DiagnosticTestContainsConductionTests.holds());
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
        #[test] fn prop_taxonomy_reflexive(entity in arb_entity()) { prop_assert!(taxonomy::is_a::<AudiologyTaxonomy>(&entity, &entity)); }
    }
}
