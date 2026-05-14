//! Auditory neuroscience — neural processing of sound from auditory
//! nerve to cortex.
//!
//! # Literature
//!
//! - **Kandel et al. (2021)** *Principles of Neural Science* (6th ed.).
//! - **Schnupp, Nelken & King (2011)** *Auditory Neuroscience*, MIT Press.
//! - **Pickles (2012)** *An Introduction to the Physiology of Hearing*.
//! - **Joris, Schreiner & Rees (2004)** "Neural Processing of
//!   Amplitude-Modulated Sounds", *Physiol. Rev.* 84(2):541-577.
//! - **Sachs & Young (1979)** "Encoding of steady-state vowels in the
//!   auditory nerve", *JASA* 66(2):470-479 — rate coding.
//! - **Goldberg & Brown (1969)** "Response of binaural neurons of dog
//!   superior olivary complex to dichotic tonal stimuli", *J. Neurophysiol.*
//!   32(4):613-636 — MSO/LSO binaural processing.
//! - **Bregman (1990)** *Auditory Scene Analysis*, MIT Press.
//!
//! # Design
//!
//! Per `feedback_one_ontology_per_module`, the dual-enum was merged.
//! Causal events become first-class `Neural*` concepts linked via
//! `causes:` edges.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Neural",
    source: "Kandel et al. (2021) Principles of Neural Science 6th ed.; Schnupp et al. (2011) Auditory Neuroscience; Joris, Schreiner & Rees (2004) Physiol. Rev. 84(2):541; Goldberg & Brown (1969) J. Neurophysiol. 32(4):613; Bregman (1990) Auditory Scene Analysis",

    concepts: [
        // Coding strategies
        RateCoding, TemporalCoding, PhaseLocking, PlaceCoding,
        PopulationCoding, SpikeTimingCode,
        // Response properties
        TonotopicMap, FrequencyTuningCurve, CharacteristicFrequency,
        RateLevelFunction, SpontaneousRate, DynamicRange,
        OnsetResponse, SustainedResponse, Adaptation, Inhibition,
        // Processing stages
        AuditoryNerveFiber, CochlearNucleusProcessing, SuperiorOliveProcessing,
        LateralLemniscus, InferiorColliculusProcessing,
        MedialGeniculateProcessing, AuditoryCortexProcessing,
        // Binaural
        BinauralProcessing, CoincidenceDetection, ExcitatoryInhibitory,
        MedialSuperiorOlive, LateralSuperiorOlive,
        // Higher
        AuditorySceneAnalysis, StreamSegregation, GestaltGrouping,
        EchoSuppression, PrecedenceEffect, MismatchNegativity,
        // Umbrellas
        CodingStrategy, ResponseProperty, ProcessingStage,
        BinauralMechanism, HigherFunction,
        // Events
        AuditoryNerveInput, CochlearNucleusIntegration, BinauralConvergence,
        LemniscalRelay, MultisensoryIntegration, ThalamicGating,
        CorticalAnalysis, StreamFormation, PerceptualBinding,
        NeuralEvent,
    ],

    labels: {
        RateCoding: ("en", "Rate coding",
            "Sachs & Young (1979) JASA 66(2):470 — neural firing rate encoding stimulus intensity."),
        TemporalCoding: ("en", "Temporal coding",
            "Joris et al. (2004) Physiol. Rev. 84(2):541 — fine-timing encoding of stimulus phase."),
        PhaseLocking: ("en", "Phase locking",
            "Joris et al. (2004): spike-time alignment to stimulus phase, up to ~4 kHz."),
        PlaceCoding: ("en", "Place coding",
            "von Bekesy (1960): frequency encoded by basilar-membrane locus and corresponding fiber."),
        PopulationCoding: ("en", "Population coding",
            "Schnupp et al. (2011) Ch. 5 — distributed activity across fiber population."),
        SpikeTimingCode: ("en", "Spike-timing code",
            "Joris et al. (2004) — precise inter-spike intervals encode stimulus features."),
        TonotopicMap: ("en", "Tonotopic map",
            "Pickles (2012): frequency-ordered representation preserved at each processing stage."),
        FrequencyTuningCurve: ("en", "Frequency tuning curve",
            "Kiang (1965): threshold as function of frequency for a single fiber."),
        CharacteristicFrequency: ("en", "Characteristic frequency",
            "Kiang (1965): frequency of lowest threshold on a tuning curve."),
        RateLevelFunction: ("en", "Rate-level function",
            "Sachs & Young (1979): firing rate vs stimulus level — typically saturating sigmoid."),
        SpontaneousRate: ("en", "Spontaneous rate",
            "Liberman (1978) JASA 63(2):442 — baseline firing in absence of stimulus."),
        DynamicRange: ("en", "Dynamic range",
            "Sachs & Young (1979): stimulus range over which rate varies."),
        OnsetResponse: ("en", "Onset response",
            "Schnupp et al. (2011): transient firing at stimulus onset."),
        SustainedResponse: ("en", "Sustained response",
            "Schnupp et al. (2011): continued firing throughout stimulus duration."),
        Adaptation: ("en", "Adaptation",
            "Schnupp et al. (2011): firing-rate decline during prolonged stimulation."),
        Inhibition: ("en", "Inhibition",
            "Schnupp et al. (2011): GABAergic suppression of postsynaptic firing."),
        AuditoryNerveFiber: ("en", "Auditory nerve fiber",
            "Pickles (2012): CN VIII cochlear afferent, ~30000 per ear in humans."),
        CochlearNucleusProcessing: ("en", "Cochlear nucleus processing",
            "Pickles (2012): first central auditory station; DCN and VCN subdivisions."),
        SuperiorOliveProcessing: ("en", "Superior olive processing",
            "Goldberg & Brown (1969): brainstem binaural-integration nuclei MSO/LSO."),
        LateralLemniscus: ("en", "Lateral lemniscus",
            "Pickles (2012): brainstem tract carrying auditory afferents to inferior colliculus."),
        InferiorColliculusProcessing: ("en", "Inferior colliculus processing",
            "Pickles (2012): midbrain integration of monaural and binaural cues."),
        MedialGeniculateProcessing: ("en", "Medial geniculate processing",
            "Pickles (2012): thalamic auditory relay to cortex."),
        AuditoryCortexProcessing: ("en", "Auditory cortex processing",
            "Pickles (2012): A1 primary auditory cortex and adjacent belt/parabelt."),
        BinauralProcessing: ("en", "Binaural processing",
            "Goldberg & Brown (1969): two-ear integration for spatial hearing."),
        CoincidenceDetection: ("en", "Coincidence detection",
            "Jeffress (1948) J. Comp. Physiol. Psychol. 41(1):35 — ITD via simultaneous bilateral input."),
        ExcitatoryInhibitory: ("en", "Excitatory-inhibitory",
            "Goldberg & Brown (1969): LSO ILD computation via ipsilateral excitation and contralateral inhibition."),
        MedialSuperiorOlive: ("en", "Medial superior olive",
            "Goldberg & Brown (1969): ITD detector — coincidence-detection model."),
        LateralSuperiorOlive: ("en", "Lateral superior olive",
            "Goldberg & Brown (1969): ILD detector — EI-cell model."),
        AuditorySceneAnalysis: ("en", "Auditory scene analysis",
            "Bregman (1990): grouping of acoustic energy into perceptual streams."),
        StreamSegregation: ("en", "Stream segregation",
            "Bregman (1990): separation of concurrent sources into distinct streams."),
        GestaltGrouping: ("en", "Gestalt grouping",
            "Bregman (1990) Ch. 1: proximity / similarity / continuity principles applied to sound."),
        EchoSuppression: ("en", "Echo suppression",
            "Wallach et al. (1949) Am. J. Psychol. 62(3):315 — precedence-effect summing of leading + lagging."),
        PrecedenceEffect: ("en", "Precedence effect",
            "Wallach et al. (1949): localisation dominated by the leading wavefront."),
        MismatchNegativity: ("en", "Mismatch negativity",
            "Naatanen et al. (1978) Acta Psychol. 42(4):313 — auditory deviance detection ERP component."),
        CodingStrategy: ("en", "Coding strategy",
            "Schnupp et al. (2011): umbrella for neural-code paradigms."),
        ResponseProperty: ("en", "Response property",
            "Schnupp et al. (2011): umbrella for measurable single-unit response features."),
        ProcessingStage: ("en", "Processing stage",
            "Pickles (2012): umbrella for an anatomical level of the central auditory pathway."),
        BinauralMechanism: ("en", "Binaural mechanism",
            "Goldberg & Brown (1969): umbrella for two-ear integration mechanisms."),
        HigherFunction: ("en", "Higher function",
            "Bregman (1990): umbrella for higher-order perceptual/cognitive auditory functions."),
        AuditoryNerveInput: ("en", "Auditory nerve input",
            "Pickles (2012): event of action-potential train arriving at cochlear nucleus."),
        CochlearNucleusIntegration: ("en", "Cochlear nucleus integration",
            "Pickles (2012): event of integration across DCN/VCN cell types."),
        BinauralConvergence: ("en", "Binaural convergence",
            "Goldberg & Brown (1969): event of bilateral input meeting at MSO/LSO."),
        LemniscalRelay: ("en", "Lemniscal relay",
            "Pickles (2012): event of transmission through lateral lemniscus."),
        MultisensoryIntegration: ("en", "Multisensory integration",
            "Stein & Meredith (1993) — event of cross-modal convergence in IC/SC."),
        ThalamicGating: ("en", "Thalamic gating",
            "Pickles (2012): event of MGB-level attention-modulated relay."),
        CorticalAnalysis: ("en", "Cortical analysis",
            "Pickles (2012): event of A1/belt feature extraction."),
        StreamFormation: ("en", "Stream formation",
            "Bregman (1990): event of perceptual stream coalescence."),
        PerceptualBinding: ("en", "Perceptual binding",
            "Bregman (1990): event of unified percept emergence."),
        NeuralEvent: ("en", "Neural event",
            "Pickles (2012): umbrella concept for an auditory neural perdurant."),
    },

    is_a: [
        (RateCoding, CodingStrategy), (TemporalCoding, CodingStrategy),
        (PhaseLocking, CodingStrategy), (PlaceCoding, CodingStrategy),
        (PopulationCoding, CodingStrategy), (SpikeTimingCode, CodingStrategy),
        (TonotopicMap, ResponseProperty), (FrequencyTuningCurve, ResponseProperty),
        (CharacteristicFrequency, ResponseProperty), (RateLevelFunction, ResponseProperty),
        (SpontaneousRate, ResponseProperty), (DynamicRange, ResponseProperty),
        (OnsetResponse, ResponseProperty), (SustainedResponse, ResponseProperty),
        (Adaptation, ResponseProperty), (Inhibition, ResponseProperty),
        (AuditoryNerveFiber, ProcessingStage), (CochlearNucleusProcessing, ProcessingStage),
        (SuperiorOliveProcessing, ProcessingStage), (LateralLemniscus, ProcessingStage),
        (InferiorColliculusProcessing, ProcessingStage),
        (MedialGeniculateProcessing, ProcessingStage),
        (AuditoryCortexProcessing, ProcessingStage),
        (CoincidenceDetection, BinauralMechanism), (ExcitatoryInhibitory, BinauralMechanism),
        (MedialSuperiorOlive, BinauralMechanism), (LateralSuperiorOlive, BinauralMechanism),
        (AuditorySceneAnalysis, HigherFunction), (StreamSegregation, HigherFunction),
        (GestaltGrouping, HigherFunction), (EchoSuppression, HigherFunction),
        (PrecedenceEffect, HigherFunction), (MismatchNegativity, HigherFunction),
        (AuditoryNerveInput, NeuralEvent), (CochlearNucleusIntegration, NeuralEvent),
        (BinauralConvergence, NeuralEvent), (LemniscalRelay, NeuralEvent),
        (MultisensoryIntegration, NeuralEvent), (ThalamicGating, NeuralEvent),
        (CorticalAnalysis, NeuralEvent), (StreamFormation, NeuralEvent),
        (PerceptualBinding, NeuralEvent),
    ],

    causes: [
        (AuditoryNerveInput, CochlearNucleusIntegration),
        (CochlearNucleusIntegration, BinauralConvergence),
        (BinauralConvergence, LemniscalRelay),
        (LemniscalRelay, MultisensoryIntegration),
        (MultisensoryIntegration, ThalamicGating),
        (ThalamicGating, CorticalAnalysis),
        (CorticalAnalysis, StreamFormation),
        (StreamFormation, PerceptualBinding),
    ],

    opposes: [
        (RateCoding, TemporalCoding), (TemporalCoding, RateCoding),
        (OnsetResponse, SustainedResponse), (SustainedResponse, OnsetResponse),
        (Inhibition, Adaptation), (Adaptation, Inhibition),
    ],
}

#[derive(Debug, Clone)]
pub struct PhaseLockingLimit;
impl Quality for PhaseLockingLimit {
    type Individual = NeuralConcept;
    type Value = f64;
    fn get(&self, individual: &NeuralConcept) -> Option<f64> {
        use NeuralConcept::*;
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
    type Individual = NeuralConcept;
    type Value = f64;
    fn get(&self, individual: &NeuralConcept) -> Option<f64> {
        use NeuralConcept::*;
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
    type Individual = NeuralConcept;
    type Value = bool;
    fn get(&self, individual: &NeuralConcept) -> Option<bool> {
        use NeuralConcept::*;
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

fn effects_of(cause: NeuralConcept) -> Vec<NeuralConcept> {
    use pr4xis::category::{Arrow, Category};
    NeuralCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == NeuralRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

pub struct InputCausesBinding;
impl Axiom for InputCausesBinding {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use NeuralConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(AuditoryNerveInput).contains(&PerceptualBinding) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "InputCausesBinding",
        "auditory nerve input transitively causes perceptual binding",
        "Bregman (1990) Auditory Scene Analysis"
    );
}
pr4xis::register_axiom!(InputCausesBinding, "Bregman (1990) Auditory Scene Analysis");

pub struct SOCDelayLongerThanCN;
impl Axiom for SOCDelayLongerThanCN {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use NeuralConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let soc = SynapticDelay.get(&SuperiorOliveProcessing).unwrap_or(0.0);
        let cn = SynapticDelay.get(&CochlearNucleusProcessing).unwrap_or(0.0);
        if soc > cn {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "SOCDelayLongerThanCN",
        "SOC synaptic delay > CN synaptic delay",
        "Pickles (2012) Physiology of Hearing"
    );
}
pr4xis::register_axiom!(SOCDelayLongerThanCN, "Pickles (2012) Physiology of Hearing");

pub struct AllStagesAreTonotopic;
impl Axiom for AllStagesAreTonotopic {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use NeuralConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = [
            AuditoryNerveFiber,
            CochlearNucleusProcessing,
            SuperiorOliveProcessing,
            InferiorColliculusProcessing,
            MedialGeniculateProcessing,
            AuditoryCortexProcessing,
        ]
        .iter()
        .all(|s| IsTonotopic.get(s) == Some(true));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "AllStagesAreTonotopic",
        "all major processing stages preserve tonotopic organization",
        "Pickles (2012) Physiology of Hearing"
    );
}
pr4xis::register_axiom!(
    AllStagesAreTonotopic,
    "Pickles (2012) Physiology of Hearing"
);

impl Ontology for NeuralOntology {
    type Cat = NeuralCategory;
    type Qual = PhaseLockingLimit;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(InputCausesBinding));
        a.push(Box::new(SOCDelayLongerThanCN));
        a.push(Box::new(AllStagesAreTonotopic));
        a
    }
}

// Back-compat aliases used by sibling functors.
pub use NeuralCategory as NeuroscienceCategory;
pub use NeuralConcept as NeuralEntity;
pub use NeuralOntology as NeuroscienceOntology;
pub use NeuralRelationKind as NeuroscienceCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<NeuralCategory>();
    }
    #[test]
    fn ontology_validates() {
        NeuralOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[test]
    fn input_causes_binding() {
        assert!(InputCausesBinding.verify().is_ok());
    }
    #[test]
    fn soc_delay_longer_than_cn() {
        assert!(SOCDelayLongerThanCN.verify().is_ok());
    }
    #[test]
    fn all_stages_tonotopic() {
        assert!(AllStagesAreTonotopic.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in NeuralCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in NeuralOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
}
