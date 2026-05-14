//! Psychoacoustics — auditory perception of physical sound.
//!
//! # Literature
//!
//! - **Fletcher & Munson (1933)** "Loudness, its definition, measurement
//!   and calculation", *JASA* 5(2):82-108 — equal-loudness contours.
//! - **Stevens (1957)** "On the Psychophysical Law", *Psychol. Rev.*
//!   64(3):153-181 — power-law of perceived intensity / sone scale.
//! - **Zwicker & Fastl (2007)** *Psychoacoustics: Facts and Models*
//!   (3rd ed.), Springer.
//! - **Moore (2012)** *An Introduction to the Psychology of Hearing*
//!   (6th ed.).
//! - **Rayleigh (1907)** "On Our Perception of Sound Direction", *Phil.
//!   Mag.* 13(74):214-232 — duplex theory.
//! - **Wegel & Lane (1924)** "The auditory masking of one pure tone by
//!   another", *Phys. Rev.* 23(2):266-285.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Psychoacoustic",
    source: "Fletcher & Munson (1933) JASA 5(2):82; Stevens (1957) Psychol. Rev. 64(3):153; Zwicker & Fastl (2007) Psychoacoustics 3rd ed.; Moore (2012) Psychology of Hearing 6th ed.; Rayleigh (1907) Phil. Mag. 13(74):214; Wegel & Lane (1924) Phys. Rev. 23(2):266",

    concepts: [
        Loudness, Pitch, Timbre, Duration,
        Phon, Sone, EqualLoudnessContour, LoudnessRecruitment,
        PlacePitch, TemporalPitch, VirtualPitch, Octave,
        SimultaneousMasking, ForwardMasking, BackwardMasking, InformationalMasking,
        CriticalBand, BarkScale, ERBScale, AuditoryFilter,
        FrequencySelectivity, TemporalResolution, GapDetection, TemporalIntegration,
        SoundLocalization, InterauralTimeDifference, InterauralLevelDifference,
        HeadRelatedTransferFunction,
        AbsoluteThreshold, DifferentialThreshold, JustNoticeableDifference,
        // Umbrellas
        PerceptualDimension, LoudnessMetric, PitchMechanism,
        MaskingType, SpatialCue, TemporalMeasure,
        // Events
        AcousticStimulus, CochlearFiltering, NeuralTransduction,
        BrainstemProcessing, CorticalAnalysis, PerceptFormation, AwareExperience,
        FrequencyAnalysis, PitchExtraction,
        PsychoacousticEvent,
    ],

    labels: {
        Loudness: ("en", "Loudness",
            "Fletcher & Munson (1933) JASA 5(2):82 — subjective magnitude of auditory sensation."),
        Pitch: ("en", "Pitch",
            "Moore (2012): perceptual attribute on a low-to-high scale."),
        Timbre: ("en", "Timbre",
            "Grey (1977) JASA 61(5):1270 — sound-quality attribute distinguishing equal-pitch-loudness-duration sounds."),
        Duration: ("en", "Duration",
            "Moore (2012): perceived temporal extent."),
        Phon: ("en", "Phon",
            "Fletcher & Munson (1933): loudness level — dB SPL of an equally loud 1 kHz tone."),
        Sone: ("en", "Sone",
            "Stevens (1957) Psychol. Rev. 64(3):153 — perceived loudness ratio scale."),
        EqualLoudnessContour: ("en", "Equal-loudness contour",
            "Fletcher & Munson (1933): SPL-vs-frequency lines of constant phon."),
        LoudnessRecruitment: ("en", "Loudness recruitment",
            "Moore (2012): abnormally rapid loudness growth."),
        PlacePitch: ("en", "Place pitch",
            "von Bekesy (1960): pitch from basilar-membrane locus of excitation."),
        TemporalPitch: ("en", "Temporal pitch",
            "Moore (2012): pitch from periodicity of neural firing."),
        VirtualPitch: ("en", "Virtual pitch",
            "Terhardt (1974) JASA 55(5):1061 — pitch from harmonic pattern."),
        Octave: ("en", "Octave",
            "Helmholtz (1863): 2:1 frequency interval."),
        SimultaneousMasking: ("en", "Simultaneous masking",
            "Wegel & Lane (1924) Phys. Rev. 23(2):266 — threshold elevation by concurrent masker."),
        ForwardMasking: ("en", "Forward masking",
            "Moore (2012): threshold elevation following masker offset."),
        BackwardMasking: ("en", "Backward masking",
            "Moore (2012): threshold elevation preceding masker onset."),
        InformationalMasking: ("en", "Informational masking",
            "Pollack (1975) JASA 57(S1):S5 — non-energetic masking from uncertainty/similarity."),
        CriticalBand: ("en", "Critical band",
            "Fletcher (1940) Rev. Mod. Phys. 12(1):47 — bandwidth of frequency-integration window."),
        BarkScale: ("en", "Bark scale",
            "Zwicker (1961) JASA 33(2):248 — 24 critical-band scale."),
        ERBScale: ("en", "ERB scale",
            "Glasberg & Moore (1990) Hearing Research 47(1-2):103 — equivalent rectangular bandwidth scale."),
        AuditoryFilter: ("en", "Auditory filter",
            "Patterson (1976) JASA 59(3):640 — psychophysical filter at each frequency."),
        FrequencySelectivity: ("en", "Frequency selectivity",
            "Moore (2012): ability to resolve frequency components."),
        TemporalResolution: ("en", "Temporal resolution",
            "Moore (2012): ability to resolve rapid temporal changes."),
        GapDetection: ("en", "Gap detection",
            "Plomp (1964) JASA 36(1):277 — shortest detectable silent gap."),
        TemporalIntegration: ("en", "Temporal integration",
            "Plomp & Bouman (1959) JASA 31(6):749 — threshold-vs-duration trading."),
        SoundLocalization: ("en", "Sound localization",
            "Rayleigh (1907) Phil. Mag. 13(74):214 — perception of source direction."),
        InterauralTimeDifference: ("en", "Interaural time difference",
            "Rayleigh (1907): inter-aural arrival-time cue; dominant <1500 Hz."),
        InterauralLevelDifference: ("en", "Interaural level difference",
            "Rayleigh (1907): inter-aural level cue; dominant >1500 Hz."),
        HeadRelatedTransferFunction: ("en", "Head-related transfer function",
            "Wightman & Kistler (1989) JASA 85(2):858 — pinna-head-torso filtering by direction."),
        AbsoluteThreshold: ("en", "Absolute threshold",
            "Moore (2012): lowest detectable stimulus level."),
        DifferentialThreshold: ("en", "Differential threshold",
            "Moore (2012): smallest detectable change in a stimulus parameter."),
        JustNoticeableDifference: ("en", "Just-noticeable difference",
            "Fechner (1860): same as differential threshold."),
        PerceptualDimension: ("en", "Perceptual dimension",
            "Moore (2012): umbrella for primary auditory attributes."),
        LoudnessMetric: ("en", "Loudness metric",
            "Fletcher & Munson (1933): umbrella for loudness scales."),
        PitchMechanism: ("en", "Pitch mechanism",
            "Moore (2012): umbrella for pitch-coding mechanisms."),
        MaskingType: ("en", "Masking type",
            "Moore (2012): umbrella for masking categories."),
        SpatialCue: ("en", "Spatial cue",
            "Rayleigh (1907): umbrella for binaural / spectral cues."),
        TemporalMeasure: ("en", "Temporal measure",
            "Moore (2012): umbrella for temporal-resolution metrics."),
        AcousticStimulus: ("en", "Acoustic stimulus",
            "Moore (2012): event — sound at the ear."),
        CochlearFiltering: ("en", "Cochlear filtering",
            "von Bekesy (1960): event — basilar-membrane frequency analysis."),
        NeuralTransduction: ("en", "Neural transduction",
            "Hudspeth (2014): event — mechanical-to-neural conversion."),
        BrainstemProcessing: ("en", "Brainstem processing",
            "Pickles (2012): event — CN/SOC/IC integration."),
        CorticalAnalysis: ("en", "Cortical analysis",
            "Pickles (2012): event — A1/belt feature analysis."),
        PerceptFormation: ("en", "Percept formation",
            "Moore (2012): event — perceptual representation emergence."),
        AwareExperience: ("en", "Aware experience",
            "Moore (2012): terminal event — conscious auditory percept."),
        FrequencyAnalysis: ("en", "Frequency analysis",
            "von Bekesy (1960): event — spectral decomposition."),
        PitchExtraction: ("en", "Pitch extraction",
            "Moore (2012): event — pitch percept formation."),
        PsychoacousticEvent: ("en", "Psychoacoustic event",
            "Moore (2012): umbrella concept for perception-pipeline perdurants."),
    },

    is_a: [
        (Loudness, PerceptualDimension), (Pitch, PerceptualDimension),
        (Timbre, PerceptualDimension), (Duration, PerceptualDimension),
        (Phon, LoudnessMetric), (Sone, LoudnessMetric),
        (EqualLoudnessContour, LoudnessMetric), (LoudnessRecruitment, LoudnessMetric),
        (PlacePitch, PitchMechanism), (TemporalPitch, PitchMechanism),
        (VirtualPitch, PitchMechanism),
        (SimultaneousMasking, MaskingType), (ForwardMasking, MaskingType),
        (BackwardMasking, MaskingType), (InformationalMasking, MaskingType),
        (InterauralTimeDifference, SpatialCue),
        (InterauralLevelDifference, SpatialCue),
        (HeadRelatedTransferFunction, SpatialCue),
        (TemporalResolution, TemporalMeasure), (GapDetection, TemporalMeasure),
        (TemporalIntegration, TemporalMeasure),
        (AcousticStimulus, PsychoacousticEvent), (CochlearFiltering, PsychoacousticEvent),
        (NeuralTransduction, PsychoacousticEvent), (BrainstemProcessing, PsychoacousticEvent),
        (CorticalAnalysis, PsychoacousticEvent), (PerceptFormation, PsychoacousticEvent),
        (AwareExperience, PsychoacousticEvent), (FrequencyAnalysis, PsychoacousticEvent),
        (PitchExtraction, PsychoacousticEvent),
    ],

    causes: [
        (AcousticStimulus, CochlearFiltering),
        (CochlearFiltering, NeuralTransduction),
        (NeuralTransduction, BrainstemProcessing),
        (BrainstemProcessing, CorticalAnalysis),
        (CorticalAnalysis, PerceptFormation),
        (PerceptFormation, AwareExperience),
        (CochlearFiltering, FrequencyAnalysis),
        (FrequencyAnalysis, PitchExtraction),
    ],

    opposes: [
        (PlacePitch, TemporalPitch), (TemporalPitch, PlacePitch),
        (SimultaneousMasking, ForwardMasking), (ForwardMasking, SimultaneousMasking),
        (InterauralTimeDifference, InterauralLevelDifference),
        (InterauralLevelDifference, InterauralTimeDifference),
    ],
}

#[derive(Debug, Clone)]
pub struct HearingThresholdDB;
impl Quality for HearingThresholdDB {
    type Individual = PsychoacousticConcept;
    type Value = f64;
    fn get(&self, individual: &PsychoacousticConcept) -> Option<f64> {
        use PsychoacousticConcept::*;
        match individual {
            AbsoluteThreshold => Some(0.0),
            JustNoticeableDifference => Some(1.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CriticalBandwidth;
impl Quality for CriticalBandwidth {
    type Individual = PsychoacousticConcept;
    type Value = f64;
    fn get(&self, individual: &PsychoacousticConcept) -> Option<f64> {
        use PsychoacousticConcept::*;
        match individual {
            CriticalBand => Some(160.0),
            AuditoryFilter => Some(130.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GapDetectionThreshold;
impl Quality for GapDetectionThreshold {
    type Individual = PsychoacousticConcept;
    type Value = f64;
    fn get(&self, individual: &PsychoacousticConcept) -> Option<f64> {
        use PsychoacousticConcept::*;
        match individual {
            GapDetection => Some(2.5),
            TemporalResolution => Some(2.5),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ITDThreshold;
impl Quality for ITDThreshold {
    type Individual = PsychoacousticConcept;
    type Value = f64;
    fn get(&self, individual: &PsychoacousticConcept) -> Option<f64> {
        use PsychoacousticConcept::*;
        match individual {
            InterauralTimeDifference => Some(15.0),
            InterauralLevelDifference => Some(1.0),
            _ => None,
        }
    }
}

fn is_a(child: PsychoacousticConcept, parent: PsychoacousticConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    if child == parent {
        return true;
    }
    PsychoacousticCategory::morphisms().iter().any(|m| {
        m.kind() == PsychoacousticRelationKind::Subsumption
            && m.source() == child
            && m.target() == parent
    })
}

fn effects_of(cause: PsychoacousticConcept) -> Vec<PsychoacousticConcept> {
    use pr4xis::category::{Arrow, Category};
    PsychoacousticCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == PsychoacousticRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

pub struct FourPerceptualDimensions;
impl Axiom for FourPerceptualDimensions {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use PsychoacousticConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = [Loudness, Pitch, Timbre, Duration]
            .iter()
            .all(|d| is_a(*d, PerceptualDimension));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "FourPerceptualDimensions",
        "loudness, pitch, timbre, and duration are all perceptual dimensions",
        "Moore (2012) Psychology of Hearing 6th ed."
    );
}
pr4xis::register_axiom!(
    FourPerceptualDimensions,
    "Moore (2012) Psychology of Hearing 6th ed."
);

pub struct FourMaskingTypes;
impl Axiom for FourMaskingTypes {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use PsychoacousticConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = [
            SimultaneousMasking,
            ForwardMasking,
            BackwardMasking,
            InformationalMasking,
        ]
        .iter()
        .all(|m| is_a(*m, MaskingType));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "FourMaskingTypes",
        "simultaneous, forward, backward, and informational masking are all classified",
        "Wegel & Lane (1924) Phys. Rev. 23(2):266; Pollack (1975) JASA 57(S1):S5"
    );
}
pr4xis::register_axiom!(
    FourMaskingTypes,
    "Wegel & Lane (1924) Phys. Rev. 23(2):266; Pollack (1975) JASA 57(S1):S5"
);

pub struct ThreeSpatialCues;
impl Axiom for ThreeSpatialCues {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use PsychoacousticConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = [
            InterauralTimeDifference,
            InterauralLevelDifference,
            HeadRelatedTransferFunction,
        ]
        .iter()
        .all(|c| is_a(*c, SpatialCue));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ThreeSpatialCues",
        "ITD, ILD, and HRTF are all spatial cues",
        "Rayleigh (1907) Phil. Mag. 13(74):214; Wightman & Kistler (1989) JASA 85(2):858"
    );
}
pr4xis::register_axiom!(
    ThreeSpatialCues,
    "Rayleigh (1907) Phil. Mag. 13(74):214; Wightman & Kistler (1989) JASA 85(2):858"
);

pub struct StimulusCausesExperience;
impl Axiom for StimulusCausesExperience {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use PsychoacousticConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(AcousticStimulus).contains(&AwareExperience) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "StimulusCausesExperience",
        "acoustic stimulus transitively causes aware experience",
        "Moore (2012) Psychology of Hearing 6th ed."
    );
}
pr4xis::register_axiom!(
    StimulusCausesExperience,
    "Moore (2012) Psychology of Hearing 6th ed."
);

impl Ontology for PsychoacousticOntology {
    type Cat = PsychoacousticCategory;
    type Qual = HearingThresholdDB;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(FourPerceptualDimensions));
        a.push(Box::new(FourMaskingTypes));
        a.push(Box::new(ThreeSpatialCues));
        a.push(Box::new(StimulusCausesExperience));
        a
    }
}

// Back-compat aliases.
pub use PsychoacousticCategory as PsychoacousticsCategory;
pub use PsychoacousticConcept as PsychoacousticEntity;
pub use PsychoacousticOntology as PsychoacousticsOntology;
pub use PsychoacousticRelationKind as PsychoacousticsCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[test]
    fn category_laws() {
        assert_category_laws::<PsychoacousticCategory>();
    }
    #[test]
    fn ontology_validates() {
        PsychoacousticOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[test]
    fn four_perceptual_dimensions() {
        assert!(FourPerceptualDimensions.verify().is_ok());
    }
    #[test]
    fn four_masking_types() {
        assert!(FourMaskingTypes.verify().is_ok());
    }
    #[test]
    fn three_spatial_cues() {
        assert!(ThreeSpatialCues.verify().is_ok());
    }
    #[test]
    fn stimulus_causes_experience() {
        assert!(StimulusCausesExperience.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in PsychoacousticCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in PsychoacousticOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
}
