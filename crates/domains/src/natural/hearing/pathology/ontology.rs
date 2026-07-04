//! Hearing pathology — hearing disorders, mechanisms, and perceptual
//! consequences.
//!
//! # Literature
//!
//! - **Moller (2006)** *Hearing: Anatomy, Physiology, and Disorders of
//!   the Auditory System* (2nd ed.).
//! - **Gates & Mills (2005)** "Presbycusis", *The Lancet*
//!   366(9491):1111-1120.
//! - **Henderson et al. (2006)** "The Role of Oxidative Stress in
//!   Noise-Induced Hearing Loss", *Ear & Hearing* 27(1):1-19.
//! - **Merchant & Rosowski (2008)** "Conductive Hearing Loss Caused by
//!   Third-Window Lesions", *Otology & Neurotology* 29(3):282-289.
//! - **Jastreboff (1990)** "Phantom auditory perception (tinnitus):
//!   mechanisms of generation and perception", *Neurosci. Res.*
//!   8(4):221-254.
//! - **Eggermont & Roberts (2004)** "The neuroscience of tinnitus",
//!   *Trends Neurosci.* 27(11):676-682.

use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

use crate::formal::math::quantity::level::{
    LogarithmicLevel, LogarithmicLevelReferenceConcept as Ref,
};
use crate::formal::math::quantity::unit::UNITLESS;
use crate::formal::math::quantity::value::Quantity;

pr4xis::ontology! {
    name: "Pathology",
    source: "Moller (2006) Hearing: Anatomy, Physiology, and Disorders 2nd ed.; Gates & Mills (2005) Lancet 366(9491):1111; Henderson et al. (2006) Ear & Hearing 27(1):1; Merchant & Rosowski (2008) Otology & Neurotology 29(3):282; Jastreboff (1990) Neurosci. Res. 8(4):221; Eggermont & Roberts (2004) Trends Neurosci. 27(11):676",

    concepts: [
        ConductiveHearingLoss, SensorineuralHearingLoss, MixedHearingLoss,
        AuditoryNeuropathy, CentralAuditoryProcessingDisorder,
        Otosclerosis, Presbycusis, NoiseInducedHearingLoss, MenieresDisease,
        AcousticNeuroma, Tinnitus, Hyperacusis, SuddenSensorineuralLoss,
        OtitisMedia, TympanicPerforation, Cholesteatoma,
        HairCellLoss, StereociliaDamage, SynapticRibbonLoss, StriaDysfunction,
        OssicularFixation, EndolymphaticHydrops, DemyelinationVIII,
        Excitotoxicity, OxidativeStress,
        ElevatedThreshold, ReducedFrequencySelectivity, LoudnessRecruitment,
        PoorSpeechInNoise, ReducedTemporalResolution, AbnormalBinauralProcessing,
        PhantomPercept,
        Audiogram, PureToneAverage, SpeechReceptionThreshold,
        OtoacousticEmission, AuditoryBrainstemResponse,
        // Umbrellas
        HearingLoss, PeripheralPathology, CentralPathology,
        DamageMechanism, PerceptualDeficit, ClinicalMeasure,
        // Events
        NoiseExposure, AgingDegeneration, Infection, Autoimmune, GeneticMutation,
        OHCDamage, IHCDamage, SynapseLoss, StriDegeneration, MiddleEarDysfunction,
        NeuralDegeneration, ThresholdShift, FrequencyResolutionLoss,
        TemporalSmearing, TinnitusGeneration, CommunicationDifficulty,
        PathologyEvent,
    ],

    labels: {
        ConductiveHearingLoss: ("en", "Conductive hearing loss",
            "Moller (2006) §13: outer/middle-ear-origin loss with air-bone gap."),
        SensorineuralHearingLoss: ("en", "Sensorineural hearing loss",
            "Moller (2006) §14: cochlear or auditory-nerve-origin loss."),
        MixedHearingLoss: ("en", "Mixed hearing loss",
            "Moller (2006): combined conductive + sensorineural components."),
        AuditoryNeuropathy: ("en", "Auditory neuropathy",
            "Starr et al. (1996) Brain 119(3):741 — present OAEs, absent ABR, abnormal speech perception."),
        CentralAuditoryProcessingDisorder: ("en", "Central auditory processing disorder",
            "ASHA (2005): central-pathway-origin deficits with normal periphery."),
        Otosclerosis: ("en", "Otosclerosis",
            "Merchant & Rosowski (2008) Otology & Neurotology 29(3):282 — stapes-fixation conductive loss."),
        Presbycusis: ("en", "Presbycusis",
            "Gates & Mills (2005) Lancet 366(9491):1111 — age-related sensorineural loss."),
        NoiseInducedHearingLoss: ("en", "Noise-induced hearing loss",
            "Henderson et al. (2006) Ear & Hearing 27(1):1 — noise-exposure-origin loss."),
        MenieresDisease: ("en", "Meniere's disease",
            "Moller (2006) §15: endolymphatic hydrops with vertigo + fluctuating SNHL."),
        AcousticNeuroma: ("en", "Acoustic neuroma",
            "Moller (2006) §16: vestibular schwannoma — CN VIII tumour."),
        Tinnitus: ("en", "Tinnitus",
            "Jastreboff (1990) Neurosci. Res. 8(4):221 — phantom auditory percept."),
        Hyperacusis: ("en", "Hyperacusis",
            "Jastreboff (1990): abnormal loudness intolerance."),
        SuddenSensorineuralLoss: ("en", "Sudden sensorineural loss",
            "Moller (2006) §14: rapid-onset (<72 hr) idiopathic SNHL."),
        OtitisMedia: ("en", "Otitis media",
            "Moller (2006) §13: middle-ear infection / effusion."),
        TympanicPerforation: ("en", "Tympanic perforation",
            "Moller (2006) §13: eardrum hole, traumatic or infectious."),
        Cholesteatoma: ("en", "Cholesteatoma",
            "Moller (2006) §13: keratinizing-epithelium middle-ear cyst."),
        HairCellLoss: ("en", "Hair-cell loss",
            "Henderson et al. (2006): mechanism — OHC/IHC death."),
        StereociliaDamage: ("en", "Stereocilia damage",
            "Henderson et al. (2006): mechanism — bundle disruption."),
        SynapticRibbonLoss: ("en", "Synaptic ribbon loss",
            "Kujawa & Liberman (2009) J. Neurosci. 29(45):14077 — IHC ribbon synapse degeneration."),
        StriaDysfunction: ("en", "Stria dysfunction",
            "Schuknecht (1993): mechanism — endocochlear-potential loss."),
        OssicularFixation: ("en", "Ossicular fixation",
            "Merchant & Rosowski (2008): mechanism — ossicle immobilisation."),
        EndolymphaticHydrops: ("en", "Endolymphatic hydrops",
            "Moller (2006) §15: mechanism — endolymph overproduction."),
        DemyelinationVIII: ("en", "Demyelination of CN VIII",
            "Starr et al. (1996): mechanism — auditory-nerve myelin loss."),
        Excitotoxicity: ("en", "Excitotoxicity",
            "Pujol & Puel (1999) Ann. NY Acad. Sci. 884:249 — glutamate-mediated IHC-afferent damage."),
        OxidativeStress: ("en", "Oxidative stress",
            "Henderson et al. (2006): mechanism — reactive-oxygen-species cochlear damage."),
        ElevatedThreshold: ("en", "Elevated threshold",
            "Moller (2006): deficit — raised audiometric threshold."),
        ReducedFrequencySelectivity: ("en", "Reduced frequency selectivity",
            "Moore (2012): deficit — broader auditory filters."),
        LoudnessRecruitment: ("en", "Loudness recruitment",
            "Moore (2012): deficit — abnormally rapid loudness growth."),
        PoorSpeechInNoise: ("en", "Poor speech in noise",
            "Moore (2012): deficit — disproportionate SIN-test difficulty."),
        ReducedTemporalResolution: ("en", "Reduced temporal resolution",
            "Moore (2012): deficit — worse gap-detection thresholds."),
        AbnormalBinauralProcessing: ("en", "Abnormal binaural processing",
            "Moore (2012): deficit — impaired ITD/ILD use."),
        PhantomPercept: ("en", "Phantom percept",
            "Jastreboff (1990): deficit — auditory percept without external stimulus."),
        Audiogram: ("en", "Audiogram",
            "Katz et al. (2015): clinical measure — threshold-vs-frequency plot."),
        PureToneAverage: ("en", "Pure-tone average",
            "Katz et al. (2015): mean of 500/1000/2000 Hz thresholds."),
        SpeechReceptionThreshold: ("en", "Speech reception threshold",
            "Katz et al. (2015): clinical measure — 50% spondee recognition level."),
        OtoacousticEmission: ("en", "Otoacoustic emission",
            "Kemp (1978) JASA 64(5):1386 — OHC-origin cochlear emission."),
        AuditoryBrainstemResponse: ("en", "Auditory brainstem response",
            "Jewett & Williston (1971) Brain 94(4):681 — brainstem evoked potential."),
        HearingLoss: ("en", "Hearing loss",
            "Moller (2006): umbrella concept for audiometric deficit."),
        PeripheralPathology: ("en", "Peripheral pathology",
            "Moller (2006): umbrella for outer/middle/inner-ear disorders."),
        CentralPathology: ("en", "Central pathology",
            "Moller (2006): umbrella for brainstem/cortical disorders."),
        DamageMechanism: ("en", "Damage mechanism",
            "Henderson et al. (2006): umbrella for cochlear-damage mechanisms."),
        PerceptualDeficit: ("en", "Perceptual deficit",
            "Moore (2012): umbrella for suprathreshold perceptual impairments."),
        ClinicalMeasure: ("en", "Clinical measure",
            "Katz et al. (2015): umbrella for diagnostic measurements."),
        NoiseExposure: ("en", "Noise exposure",
            "Henderson et al. (2006): event — sustained or impulse noise above safe levels."),
        AgingDegeneration: ("en", "Aging degeneration",
            "Gates & Mills (2005): event — cumulative age-related cochlear changes."),
        Infection: ("en", "Infection",
            "Moller (2006): event — pathogen-driven middle-ear or labyrinth involvement."),
        Autoimmune: ("en", "Autoimmune",
            "Moller (2006): event — immune-mediated inner-ear injury."),
        GeneticMutation: ("en", "Genetic mutation",
            "Moller (2006): event — congenital channel/connexin/synaptic gene defect."),
        OHCDamage: ("en", "Outer hair cell damage",
            "Henderson et al. (2006): event — OHC injury."),
        IHCDamage: ("en", "Inner hair cell damage",
            "Henderson et al. (2006): event — IHC injury."),
        SynapseLoss: ("en", "Synapse loss",
            "Kujawa & Liberman (2009) J. Neurosci. 29(45):14077 — event — ribbon synapse loss."),
        StriDegeneration: ("en", "Stria degeneration",
            "Schuknecht (1993): event — stria-vascularis cell loss."),
        MiddleEarDysfunction: ("en", "Middle-ear dysfunction",
            "Moller (2006): event — TM/ossicle/fluid impairment."),
        NeuralDegeneration: ("en", "Neural degeneration",
            "Starr et al. (1996): event — auditory-nerve fibre loss."),
        ThresholdShift: ("en", "Threshold shift",
            "NIOSH (1998): event — temporary or permanent threshold elevation."),
        FrequencyResolutionLoss: ("en", "Frequency resolution loss",
            "Moore (2012): event — auditory-filter broadening."),
        TemporalSmearing: ("en", "Temporal smearing",
            "Moore (2012): event — degraded temporal resolution."),
        TinnitusGeneration: ("en", "Tinnitus generation",
            "Jastreboff (1990): event — central-gain-driven phantom percept onset."),
        CommunicationDifficulty: ("en", "Communication difficulty",
            "Moore (2012): terminal event — measurable speech-understanding decrement."),
        PathologyEvent: ("en", "Pathology event",
            "Moller (2006): umbrella concept for pathology perdurants."),
    },

    is_a: [
        (ConductiveHearingLoss, HearingLoss),
        (SensorineuralHearingLoss, HearingLoss),
        (MixedHearingLoss, HearingLoss),
        (AuditoryNeuropathy, HearingLoss),
        (CentralAuditoryProcessingDisorder, HearingLoss),
        (Otosclerosis, PeripheralPathology),
        (Presbycusis, PeripheralPathology),
        (NoiseInducedHearingLoss, PeripheralPathology),
        (MenieresDisease, PeripheralPathology),
        (Tinnitus, PeripheralPathology), (Hyperacusis, PeripheralPathology),
        (SuddenSensorineuralLoss, PeripheralPathology),
        (OtitisMedia, PeripheralPathology),
        (TympanicPerforation, PeripheralPathology),
        (Cholesteatoma, PeripheralPathology),
        (AcousticNeuroma, CentralPathology),
        (CentralAuditoryProcessingDisorder, CentralPathology),
        (HairCellLoss, DamageMechanism), (StereociliaDamage, DamageMechanism),
        (SynapticRibbonLoss, DamageMechanism), (StriaDysfunction, DamageMechanism),
        (OssicularFixation, DamageMechanism), (EndolymphaticHydrops, DamageMechanism),
        (DemyelinationVIII, DamageMechanism), (Excitotoxicity, DamageMechanism),
        (OxidativeStress, DamageMechanism),
        (ElevatedThreshold, PerceptualDeficit),
        (ReducedFrequencySelectivity, PerceptualDeficit),
        (LoudnessRecruitment, PerceptualDeficit),
        (PoorSpeechInNoise, PerceptualDeficit),
        (ReducedTemporalResolution, PerceptualDeficit),
        (AbnormalBinauralProcessing, PerceptualDeficit),
        (PhantomPercept, PerceptualDeficit),
        (Audiogram, ClinicalMeasure), (PureToneAverage, ClinicalMeasure),
        (SpeechReceptionThreshold, ClinicalMeasure),
        (OtoacousticEmission, ClinicalMeasure),
        (AuditoryBrainstemResponse, ClinicalMeasure),
        (NoiseExposure, PathologyEvent), (AgingDegeneration, PathologyEvent),
        (Infection, PathologyEvent), (Autoimmune, PathologyEvent),
        (GeneticMutation, PathologyEvent), (OHCDamage, PathologyEvent),
        (IHCDamage, PathologyEvent), (SynapseLoss, PathologyEvent),
        (StriDegeneration, PathologyEvent), (MiddleEarDysfunction, PathologyEvent),
        (NeuralDegeneration, PathologyEvent), (ThresholdShift, PathologyEvent),
        (FrequencyResolutionLoss, PathologyEvent), (TemporalSmearing, PathologyEvent),
        (TinnitusGeneration, PathologyEvent), (CommunicationDifficulty, PathologyEvent),
    ],

    causes: [
        (NoiseExposure, OHCDamage),
        (NoiseExposure, IHCDamage),
        (NoiseExposure, SynapseLoss),
        (AgingDegeneration, OHCDamage),
        (AgingDegeneration, StriDegeneration),
        (AgingDegeneration, NeuralDegeneration),
        (Infection, MiddleEarDysfunction),
        (GeneticMutation, OHCDamage),
        (GeneticMutation, IHCDamage),
        (OHCDamage, ThresholdShift),
        (OHCDamage, FrequencyResolutionLoss),
        (OHCDamage, TinnitusGeneration),
        (IHCDamage, ThresholdShift),
        (SynapseLoss, TemporalSmearing),
        (StriDegeneration, ThresholdShift),
        (MiddleEarDysfunction, ThresholdShift),
        (ThresholdShift, CommunicationDifficulty),
        (FrequencyResolutionLoss, CommunicationDifficulty),
        (TemporalSmearing, CommunicationDifficulty),
    ],

    opposes: [
        (ConductiveHearingLoss, SensorineuralHearingLoss),
        (SensorineuralHearingLoss, ConductiveHearingLoss),
        (Tinnitus, Hyperacusis), (Hyperacusis, Tinnitus),
        (HairCellLoss, SynapticRibbonLoss), (SynapticRibbonLoss, HairCellLoss),
    ],
}

#[derive(Debug, Clone)]
pub struct TypicalSeverityDB;
impl Quality for TypicalSeverityDB {
    type Individual = PathologyConcept;
    type Value = LogarithmicLevel;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &PathologyConcept) -> Option<LogarithmicLevel> {
        use PathologyConcept::*;
        // Typical audiometric severity — a hearing-loss magnitude in dB HL
        // (hearing level, ISO 389 audiometric zero). A dB figure is a
        // logarithmic level (IEC 80000-15), not a linear Quantity.
        let db = match individual {
            Otosclerosis => 40.0,
            Presbycusis => 45.0,
            NoiseInducedHearingLoss => 50.0,
            MenieresDisease => 40.0,
            OtitisMedia => 25.0,
            TympanicPerforation => 30.0,
            AcousticNeuroma => 55.0,
            SuddenSensorineuralLoss => 60.0,
            _ => return None,
        };
        Some(LogarithmicLevel::new(db, Ref::HearingLevel))
    }
}

#[derive(Debug, Clone)]
pub struct PrevalencePercent;
impl Quality for PrevalencePercent {
    type Individual = PathologyConcept;
    type Value = Quantity;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &PathologyConcept) -> Option<Quantity> {
        use PathologyConcept::*;
        // Prevalence is a dimensionless proportion; a percentage figure is the
        // fraction (33% → 0.33), UNITLESS.
        let fraction = match individual {
            Presbycusis => 0.33,
            NoiseInducedHearingLoss => 0.12,
            Tinnitus => 0.15,
            _ => return None,
        };
        Some(Quantity::from_unit(fraction, &UNITLESS))
    }
}

#[derive(Debug, Clone)]
pub struct OAEsPresent;
impl Quality for OAEsPresent {
    type Individual = PathologyConcept;
    type Value = bool;
    fn get(&self, individual: &PathologyConcept) -> Option<bool> {
        use PathologyConcept::*;
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

fn effects_of(cause: PathologyConcept) -> Vec<PathologyConcept> {
    use pr4xis::category::{Arrow, Category};
    PathologyCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == PathologyRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

pub struct NoiseCausesDifficulty;
impl Axiom for NoiseCausesDifficulty {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use PathologyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(NoiseExposure).contains(&CommunicationDifficulty) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "NoiseCausesDifficulty",
        "noise exposure transitively causes communication difficulty",
        "Henderson et al. (2006) Ear & Hearing 27(1):1"
    );
}
pr4xis::register_axiom!(
    NoiseCausesDifficulty,
    "Henderson et al. (2006) Ear & Hearing 27(1):1"
);

pub struct PresbycusisMostPrevalent;
impl Axiom for PresbycusisMostPrevalent {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use PathologyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        // All three are UNITLESS prevalence fractions (same dimension), so a
        // direct `.value` compare preserves the "highest prevalence" meaning.
        let p = PrevalencePercent
            .get(&Presbycusis)
            .map(|q| q.value)
            .unwrap_or(0.0);
        let n = PrevalencePercent
            .get(&NoiseInducedHearingLoss)
            .map(|q| q.value)
            .unwrap_or(0.0);
        let t = PrevalencePercent
            .get(&Tinnitus)
            .map(|q| q.value)
            .unwrap_or(0.0);
        if p > n && p > t {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "PresbycusisMostPrevalent",
        "presbycusis has highest prevalence among modelled conditions",
        "Gates & Mills (2005) Lancet 366(9491):1111"
    );
}
pr4xis::register_axiom!(
    PresbycusisMostPrevalent,
    "Gates & Mills (2005) Lancet 366(9491):1111"
);

pub struct NeuropathyHasOAEs;
impl Axiom for NeuropathyHasOAEs {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use PathologyConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = OAEsPresent.get(&AuditoryNeuropathy) == Some(true)
            && OAEsPresent.get(&SensorineuralHearingLoss) == Some(false);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "NeuropathyHasOAEs",
        "auditory neuropathy preserves OAEs (OHCs intact) but absent ABR",
        "Starr et al. (1996) Brain 119(3):741"
    );
}
pr4xis::register_axiom!(NeuropathyHasOAEs, "Starr et al. (1996) Brain 119(3):741");

impl Ontology for PathologyOntology {
    type Cat = PathologyCategory;
    type Qual = TypicalSeverityDB;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(NoiseCausesDifficulty));
        a.push(Box::new(PresbycusisMostPrevalent));
        a.push(Box::new(NeuropathyHasOAEs));
        a
    }
}

// Back-compat aliases.
pub use PathologyConcept as PathologyEntity;
pub use PathologyRelationKind as PathologyCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<PathologyCategory>();
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        PathologyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn noise_causes_difficulty() {
        assert!(NoiseCausesDifficulty.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn presbycusis_most_prevalent() {
        assert!(PresbycusisMostPrevalent.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn neuropathy_has_oaes() {
        assert!(NeuropathyHasOAEs.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in PathologyCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in PathologyOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
