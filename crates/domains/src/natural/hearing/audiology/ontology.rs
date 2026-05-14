//! Audiology — clinical audiological assessment and rehabilitation.
//!
//! # Literature
//!
//! - **Katz et al. (2015)** *Handbook of Clinical Audiology* (7th ed.),
//!   Wolters Kluwer — comprehensive clinical reference.
//! - **Stach (2010)** *Clinical Audiology: An Introduction* (2nd ed.).
//! - **Carhart (1950)** "Clinical Application of Bone Conduction
//!   Audiometry", *Archives of Otolaryngology* 51(6):798-808.
//! - **Jerger (1970)** "Clinical Experience with Impedance Audiometry",
//!   *Archives of Otolaryngology* 92(4):311-324 — Type A/B/C tympanograms.
//! - **ASHA (2005)** *Guidelines for Manual Pure-Tone Threshold
//!   Audiometry*, American Speech-Language-Hearing Association.
//! - **Kemp (1978)** "Stimulated acoustic emissions from within the human
//!   auditory system", *J. Acoust. Soc. Am.* 64(5):1386-1391.
//!
//! # Design
//!
//! Per `feedback_one_ontology_per_module`, the dual-enum (entities +
//! parallel `AudiologyCausalEvent`) was merged into a single concept set
//! with `causes:` edges representing the clinical workflow.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Audiology",
    source: "Katz et al. (2015) Handbook of Clinical Audiology 7th ed.; Jerger (1970) Archives of Otolaryngology 92(4):311; ASHA (2005) Pure-Tone Threshold Audiometry Guidelines; Kemp (1978) JASA 64(5):1386",

    concepts: [
        // Pure-tone audiometry
        PureToneAudiometry, AirConductionTest, BoneConductionTest, MaskingProcedure,
        AirBoneGap, PureToneAverage,
        // Speech audiometry
        SpeechAudiometry, SpeechRecognitionThreshold, WordRecognitionScore,
        SpeechInNoiseTest, QuickSIN, HINT,
        // Immittance
        Tympanometry, TympanogramTypeA, TympanogramTypeB, TympanogramTypeC,
        AcousticReflex, AcousticReflexDecay, StaticCompliance,
        // OAE
        TransientOAE, DistortionProductOAE, OAEScreening,
        // ABR
        AuditoryBrainstemResponse, WaveI, WaveIII, WaveV,
        ElectroCochleography, AuditoryLateResponse,
        // Rehab
        AuralRehabilitation, HearingAidFitting, RealEarVerification,
        CochlearImplantMapping, AuditoryTraining, CommunicationStrategy,
        // Workflow
        CaseHistory, Otoscopy, Referral, Counseling,
        // Umbrellas
        DiagnosticTest, SpeechTest, ImmittanceTest, EmissionTest, EvokedPotentialTest,
        RehabilitationProcedure, ClinicalWorkflow,
        // Events
        PatientPresents, HistoryTaken, OtoscopyPerformed, PureToneCompleted,
        SpeechTestCompleted, ImmittanceCompleted, OAECompleted, DiagnosisMade,
        TreatmentPlanDeveloped, DeviceFitted, OutcomeVerified,
        AudiologyEvent,
    ],

    labels: {
        PureToneAudiometry: ("en", "Pure-tone audiometry",
            "Katz et al. (2015): air- and bone-conduction threshold testing at 250-8000 Hz."),
        AirConductionTest: ("en", "Air-conduction test",
            "ASHA (2005): threshold determination via supra-aural or insert earphones."),
        BoneConductionTest: ("en", "Bone-conduction test",
            "Carhart (1950): threshold determination via mastoid or forehead bone oscillator."),
        MaskingProcedure: ("en", "Masking procedure",
            "Katz et al. (2015) Ch. 5: introduce noise to the non-test ear to isolate the test ear."),
        AirBoneGap: ("en", "Air-bone gap",
            "Katz et al. (2015): difference between AC and BC thresholds indicating conductive loss."),
        PureToneAverage: ("en", "Pure-tone average",
            "Katz et al. (2015): average of 500/1000/2000 Hz thresholds."),
        SpeechAudiometry: ("en", "Speech audiometry",
            "Katz et al. (2015) Ch. 7: threshold + suprathreshold word/sentence recognition testing."),
        SpeechRecognitionThreshold: ("en", "Speech recognition threshold",
            "Katz et al. (2015): lowest level for 50% spondaic word recognition."),
        WordRecognitionScore: ("en", "Word recognition score",
            "Katz et al. (2015): percentage correct on a phonetically-balanced word list at suprathreshold."),
        SpeechInNoiseTest: ("en", "Speech-in-noise test",
            "Killion et al. (2004): family of tests measuring speech recognition with background noise."),
        QuickSIN: ("en", "QuickSIN",
            "Killion et al. (2004) JASA 116(4):2395 — Quick Speech-in-Noise test."),
        HINT: ("en", "HINT",
            "Nilsson et al. (1994) JASA 95(2):1085 — Hearing in Noise Test."),
        Tympanometry: ("en", "Tympanometry",
            "Jerger (1970): admittance vs ear-canal pressure measurement."),
        TympanogramTypeA: ("en", "Type A tympanogram",
            "Jerger (1970): peak admittance at 0 daPa — normal middle-ear function."),
        TympanogramTypeB: ("en", "Type B tympanogram",
            "Jerger (1970): flat trace — middle-ear effusion or perforation."),
        TympanogramTypeC: ("en", "Type C tympanogram",
            "Jerger (1970): peak shifted negative — eustachian dysfunction."),
        AcousticReflex: ("en", "Acoustic reflex",
            "Katz et al. (2015): stapedial reflex elicited by loud tone (~85 dB SL)."),
        AcousticReflexDecay: ("en", "Acoustic reflex decay",
            "Katz et al. (2015): reflex amplitude reduction over 10 s — retrocochlear marker."),
        StaticCompliance: ("en", "Static compliance",
            "Katz et al. (2015): equivalent volume of middle-ear admittance."),
        TransientOAE: ("en", "Transient-evoked OAE",
            "Kemp (1978): cochlear emission elicited by click stimulus."),
        DistortionProductOAE: ("en", "Distortion-product OAE",
            "Kemp (1978): emission at 2f1-f2 elicited by two-tone stimulus."),
        OAEScreening: ("en", "OAE screening",
            "Katz et al. (2015): newborn / adult screening via TEOAE or DPOAE."),
        AuditoryBrainstemResponse: ("en", "Auditory brainstem response",
            "Jewett & Williston (1971) Brain 94(4):681 — far-field evoked potential I-VII."),
        WaveI: ("en", "Wave I",
            "Jewett & Williston (1971): ~1.5 ms; auditory nerve action potential."),
        WaveIII: ("en", "Wave III",
            "Jewett & Williston (1971): ~3.5 ms; cochlear nucleus / superior olive."),
        WaveV: ("en", "Wave V",
            "Jewett & Williston (1971): ~5.5 ms; lateral lemniscus / inferior colliculus."),
        ElectroCochleography: ("en", "Electrocochleography",
            "Eggermont (1976) Audiology 15(1):31 — cochlear potentials via near-field electrode."),
        AuditoryLateResponse: ("en", "Auditory late response",
            "Katz et al. (2015): cortical evoked potentials, 50-300 ms latency."),
        AuralRehabilitation: ("en", "Aural rehabilitation",
            "Tye-Murray (2014): umbrella term for post-fit interventions."),
        HearingAidFitting: ("en", "Hearing aid fitting",
            "Dillon (2012) Hearing Aids 2nd ed.: prescription, programming, verification."),
        RealEarVerification: ("en", "Real-ear verification",
            "Dillon (2012): probe-microphone measurement of in-situ aid output."),
        CochlearImplantMapping: ("en", "Cochlear implant mapping",
            "Zeng et al. (2008): electrode-specific threshold/comfort programming."),
        AuditoryTraining: ("en", "Auditory training",
            "Tye-Murray (2014): structured listening practice."),
        CommunicationStrategy: ("en", "Communication strategy",
            "Tye-Murray (2014): patient-side techniques to improve conversation."),
        CaseHistory: ("en", "Case history",
            "Katz et al. (2015): patient-reported symptom and exposure intake."),
        Otoscopy: ("en", "Otoscopy",
            "Katz et al. (2015): visual inspection of canal and TM."),
        Referral: ("en", "Referral",
            "Katz et al. (2015): handoff to ENT or other specialist."),
        Counseling: ("en", "Counseling",
            "Katz et al. (2015): result interpretation and expectation management."),
        DiagnosticTest: ("en", "Diagnostic test",
            "Katz et al. (2015): umbrella concept for measurement procedures."),
        SpeechTest: ("en", "Speech test",
            "Katz et al. (2015): speech-material-based assessment."),
        ImmittanceTest: ("en", "Immittance test",
            "Jerger (1970): admittance/impedance-based middle-ear assessment."),
        EmissionTest: ("en", "Emission test",
            "Kemp (1978): OAE-based cochlear-function assessment."),
        EvokedPotentialTest: ("en", "Evoked-potential test",
            "Jewett & Williston (1971): electrophysiological auditory assessment."),
        RehabilitationProcedure: ("en", "Rehabilitation procedure",
            "Tye-Murray (2014): umbrella for post-diagnosis interventions."),
        ClinicalWorkflow: ("en", "Clinical workflow",
            "Katz et al. (2015): non-measurement clinical step."),
        PatientPresents: ("en", "Patient presents",
            "Katz et al. (2015): clinical encounter initiation."),
        HistoryTaken: ("en", "History taken",
            "Katz et al. (2015): case history completion."),
        OtoscopyPerformed: ("en", "Otoscopy performed",
            "Katz et al. (2015): canal inspection completion."),
        PureToneCompleted: ("en", "Pure-tone audiometry completed",
            "Katz et al. (2015): AC/BC thresholds obtained."),
        SpeechTestCompleted: ("en", "Speech test completed",
            "Katz et al. (2015): SRT/WRS obtained."),
        ImmittanceCompleted: ("en", "Immittance completed",
            "Jerger (1970): tympanogram and reflex obtained."),
        OAECompleted: ("en", "OAE completed",
            "Kemp (1978): OAE measurement obtained."),
        DiagnosisMade: ("en", "Diagnosis made",
            "Katz et al. (2015): clinical interpretation event."),
        TreatmentPlanDeveloped: ("en", "Treatment plan developed",
            "Katz et al. (2015): rehabilitation plan finalization."),
        DeviceFitted: ("en", "Device fitted",
            "Dillon (2012): hearing aid / CI fitting event."),
        OutcomeVerified: ("en", "Outcome verified",
            "Dillon (2012): real-ear / functional outcome measurement."),
        AudiologyEvent: ("en", "Audiology event",
            "Katz et al. (2015): umbrella for clinical-workflow perdurants."),
    },

    is_a: [
        (PureToneAudiometry, DiagnosticTest),
        (AirConductionTest, PureToneAudiometry),
        (BoneConductionTest, PureToneAudiometry),
        (SpeechAudiometry, SpeechTest),
        (SpeechRecognitionThreshold, SpeechTest),
        (WordRecognitionScore, SpeechTest),
        (SpeechInNoiseTest, SpeechTest),
        (QuickSIN, SpeechInNoiseTest),
        (HINT, SpeechInNoiseTest),
        (Tympanometry, ImmittanceTest),
        (AcousticReflex, ImmittanceTest),
        (AcousticReflexDecay, ImmittanceTest),
        (TympanogramTypeA, Tympanometry),
        (TympanogramTypeB, Tympanometry),
        (TympanogramTypeC, Tympanometry),
        (TransientOAE, EmissionTest),
        (DistortionProductOAE, EmissionTest),
        (OAEScreening, EmissionTest),
        (AuditoryBrainstemResponse, EvokedPotentialTest),
        (ElectroCochleography, EvokedPotentialTest),
        (AuditoryLateResponse, EvokedPotentialTest),
        (WaveI, AuditoryBrainstemResponse),
        (WaveIII, AuditoryBrainstemResponse),
        (WaveV, AuditoryBrainstemResponse),
        (AuralRehabilitation, RehabilitationProcedure),
        (HearingAidFitting, RehabilitationProcedure),
        (RealEarVerification, RehabilitationProcedure),
        (CochlearImplantMapping, RehabilitationProcedure),
        (AuditoryTraining, RehabilitationProcedure),
        (CommunicationStrategy, RehabilitationProcedure),
        (CaseHistory, ClinicalWorkflow),
        (Otoscopy, ClinicalWorkflow),
        (Referral, ClinicalWorkflow),
        (Counseling, ClinicalWorkflow),
        // Events under umbrella
        (PatientPresents, AudiologyEvent),
        (HistoryTaken, AudiologyEvent),
        (OtoscopyPerformed, AudiologyEvent),
        (PureToneCompleted, AudiologyEvent),
        (SpeechTestCompleted, AudiologyEvent),
        (ImmittanceCompleted, AudiologyEvent),
        (OAECompleted, AudiologyEvent),
        (DiagnosisMade, AudiologyEvent),
        (TreatmentPlanDeveloped, AudiologyEvent),
        (DeviceFitted, AudiologyEvent),
        (OutcomeVerified, AudiologyEvent),
    ],

    has_a: [
        (DiagnosticTest, PureToneAudiometry),
        (DiagnosticTest, SpeechAudiometry),
        (PureToneAudiometry, AirConductionTest),
        (PureToneAudiometry, BoneConductionTest),
        (PureToneAudiometry, MaskingProcedure),
        (AuditoryBrainstemResponse, WaveI),
        (AuditoryBrainstemResponse, WaveIII),
        (AuditoryBrainstemResponse, WaveV),
    ],

    causes: [
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
    ],

    opposes: [
        (AirConductionTest, BoneConductionTest), (BoneConductionTest, AirConductionTest),
        (TransientOAE, DistortionProductOAE), (DistortionProductOAE, TransientOAE),
        (PureToneAudiometry, SpeechAudiometry), (SpeechAudiometry, PureToneAudiometry),
    ],
}

// Qualities

#[derive(Debug, Clone)]
pub struct ABRLatencyMs;
impl Quality for ABRLatencyMs {
    type Individual = AudiologyConcept;
    type Value = f64;
    fn get(&self, individual: &AudiologyConcept) -> Option<f64> {
        use AudiologyConcept::*;
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
    type Individual = AudiologyConcept;
    type Value = f64;
    fn get(&self, individual: &AudiologyConcept) -> Option<f64> {
        use AudiologyConcept::*;
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
    type Individual = AudiologyConcept;
    type Value = bool;
    fn get(&self, individual: &AudiologyConcept) -> Option<bool> {
        use AudiologyConcept::*;
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

// Helpers

fn is_a(child: AudiologyConcept, parent: AudiologyConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    if child == parent {
        return true;
    }
    AudiologyCategory::morphisms().iter().any(|m| {
        m.kind() == AudiologyRelationKind::Subsumption
            && m.source() == child
            && m.target() == parent
    })
}

fn parts_of(whole: AudiologyConcept) -> Vec<AudiologyConcept> {
    use pr4xis::category::{Arrow, Category};
    AudiologyCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == AudiologyRelationKind::Parthood && m.source() == whole)
        .map(|m| m.target())
        .collect()
}

fn effects_of(cause: AudiologyConcept) -> Vec<AudiologyConcept> {
    use pr4xis::category::{Arrow, Category};
    AudiologyCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == AudiologyRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

// Axioms

pub struct DiagnosticTestContainsConductionTests;
impl Axiom for DiagnosticTestContainsConductionTests {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AudiologyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts = parts_of(DiagnosticTest);
        if parts.contains(&AirConductionTest) && parts.contains(&BoneConductionTest) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "DiagnosticTestContainsConductionTests",
        "diagnostic test transitively contains air and bone conduction tests",
        "Katz et al. (2015) Handbook of Clinical Audiology"
    );
}
pr4xis::register_axiom!(
    DiagnosticTestContainsConductionTests,
    "Katz et al. (2015) Handbook of Clinical Audiology"
);

pub struct ABRWavesOrdered;
impl Axiom for ABRWavesOrdered {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AudiologyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let l = ABRLatencyMs;
        let i = l.get(&WaveI).unwrap_or(0.0);
        let iii = l.get(&WaveIII).unwrap_or(0.0);
        let v = l.get(&WaveV).unwrap_or(0.0);
        if i < iii && iii < v {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ABRWavesOrdered",
        "ABR wave latencies are ordered (I < III < V)",
        "Jewett & Williston (1971) Brain 94(4):681"
    );
}
pr4xis::register_axiom!(ABRWavesOrdered, "Jewett & Williston (1971) Brain 94(4):681");

pub struct ThreeTympanogramTypes;
impl Axiom for ThreeTympanogramTypes {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AudiologyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = [TympanogramTypeA, TympanogramTypeB, TympanogramTypeC]
            .iter()
            .all(|t| is_a(*t, Tympanometry));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ThreeTympanogramTypes",
        "three tympanogram types (A, B, C) are classified",
        "Jerger (1970) Archives of Otolaryngology 92(4):311"
    );
}
pr4xis::register_axiom!(
    ThreeTympanogramTypes,
    "Jerger (1970) Archives of Otolaryngology 92(4):311"
);

pub struct FullClinicalPathway;
impl Axiom for FullClinicalPathway {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use AudiologyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(PatientPresents).contains(&OutcomeVerified) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "FullClinicalPathway",
        "patient presentation transitively leads to outcome verification",
        "Katz et al. (2015) Handbook of Clinical Audiology"
    );
}
pr4xis::register_axiom!(
    FullClinicalPathway,
    "Katz et al. (2015) Handbook of Clinical Audiology"
);

impl Ontology for AudiologyOntology {
    type Cat = AudiologyCategory;
    type Qual = ABRLatencyMs;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(DiagnosticTestContainsConductionTests));
        a.push(Box::new(ABRWavesOrdered));
        a.push(Box::new(ThreeTympanogramTypes));
        a.push(Box::new(FullClinicalPathway));
        a
    }
}

// Back-compat aliases for cross-functors.
pub use AudiologyConcept as AudiologyEntity;
pub use AudiologyRelationKind as AudiologyCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<AudiologyCategory>();
    }
    #[test]
    fn ontology_validates() {
        AudiologyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[test]
    fn abr_waves_ordered() {
        assert!(ABRWavesOrdered.verify().is_ok());
    }
    #[test]
    fn three_tympanogram_types() {
        assert!(ThreeTympanogramTypes.verify().is_ok());
    }
    #[test]
    fn full_clinical_pathway() {
        assert!(FullClinicalPathway.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in AudiologyCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in AudiologyOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
}
