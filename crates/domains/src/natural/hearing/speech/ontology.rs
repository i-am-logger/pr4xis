//! Speech acoustics — production and perception of speech sounds.
//!
//! # Literature
//!
//! - **Fant (1960)** *Acoustic Theory of Speech Production*, Mouton —
//!   source-filter model.
//! - **Peterson & Barney (1952)** "Control Methods Used in a Study of the
//!   Vowels", *JASA* 24(2):175-184 — formant measurements.
//! - **Stevens (2000)** *Acoustic Phonetics*, MIT Press.
//! - **Lisker & Abramson (1964)** "A Cross-Language Study of Voicing in
//!   Initial Stops: Acoustical Measurements", *Word* 20(3):384-422 — VOT.
//! - **ANSI S3.5-1997** *Speech Intelligibility Index*.

use crate::formal::math::quantity::unit::{HERTZ, MILLISECOND};
use crate::formal::math::quantity::value::{Quantity, QuantityRange};
use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

pr4xis::ontology! {
    name: "Speech",
    source: "Fant (1960) Acoustic Theory of Speech Production; Peterson & Barney (1952) JASA 24(2):175; Stevens (2000) Acoustic Phonetics; Lisker & Abramson (1964) Word 20(3):384; ANSI S3.5-1997 Speech Intelligibility Index",

    concepts: [
        FundamentalFrequency, Formant, F1, F2, F3, F4,
        VoiceOnsetTime, SpectralTilt, Harmonics,
        Vowel, Consonant, Plosive, Fricative, Nasal, Approximant, Affricate,
        Voiced, Voiceless,
        Intonation, Stress, Rhythm, Syllable, Phoneme,
        SpeechIntelligibilityIndex, SignalToNoiseRatio, SpeechReceptionThreshold,
        ArticulationIndex,
        LowFrequencySpeech, MidFrequencySpeech, HighFrequencySpeech,
        // Umbrellas
        AcousticParameter, SpeechSound, Suprasegmental,
        IntelligibilityMetric, SpectralRegion,
        // Events
        CommunicativeIntent, ArticulatoryPlanning, VocalFoldVibration,
        GlottalPulse, VocalTractFiltering, FormantProduction,
        AcousticRadiation, ListenerPerception,
        CoarticulationEffect, FormantTransition,
        SpeechEvent,
    ],

    labels: {
        FundamentalFrequency: ("en", "Fundamental frequency",
            "Fant (1960): F0 — vocal-fold-vibration frequency."),
        Formant: ("en", "Formant",
            "Fant (1960): vocal-tract resonance peak."),
        F1: ("en", "F1",
            "Peterson & Barney (1952) JASA 24(2):175 — first formant; ~500 Hz; vowel height."),
        F2: ("en", "F2",
            "Peterson & Barney (1952): second formant; ~1500 Hz; vowel frontness."),
        F3: ("en", "F3",
            "Peterson & Barney (1952): third formant; ~2500 Hz."),
        F4: ("en", "F4",
            "Peterson & Barney (1952): fourth formant; ~3500 Hz."),
        VoiceOnsetTime: ("en", "Voice onset time",
            "Lisker & Abramson (1964) Word 20(3):384 — delay between release and voicing onset."),
        SpectralTilt: ("en", "Spectral tilt",
            "Stevens (2000): spectral envelope slope."),
        Harmonics: ("en", "Harmonics",
            "Fant (1960): integer multiples of F0."),
        Vowel: ("en", "Vowel",
            "Stevens (2000): open-tract speech sound, voiced and continuant."),
        Consonant: ("en", "Consonant",
            "Stevens (2000): speech sound with constricted or closed vocal-tract configuration."),
        Plosive: ("en", "Plosive",
            "Stevens (2000): stop consonant with complete closure + release burst."),
        Fricative: ("en", "Fricative",
            "Stevens (2000): turbulent-noise-production constriction."),
        Nasal: ("en", "Nasal",
            "Stevens (2000): velum-lowered consonant with nasal-tract radiation."),
        Approximant: ("en", "Approximant",
            "Stevens (2000): near-vowel constriction without turbulence."),
        Affricate: ("en", "Affricate",
            "Stevens (2000): plosive + fricative sequence (e.g., /tʃ/)."),
        Voiced: ("en", "Voiced",
            "Stevens (2000): laryngeal vibration during the sound."),
        Voiceless: ("en", "Voiceless",
            "Stevens (2000): no laryngeal vibration."),
        Intonation: ("en", "Intonation",
            "Stevens (2000): F0 contour over an utterance."),
        Stress: ("en", "Stress",
            "Stevens (2000): syllable prominence."),
        Rhythm: ("en", "Rhythm",
            "Stevens (2000): temporal patterning of syllables."),
        Syllable: ("en", "Syllable",
            "Stevens (2000): phonological unit centred on a vowel."),
        Phoneme: ("en", "Phoneme",
            "Jakobson et al. (1952): contrastive segmental phonological unit."),
        SpeechIntelligibilityIndex: ("en", "Speech intelligibility index",
            "ANSI S3.5-1997: 0..1 metric for speech intelligibility."),
        SignalToNoiseRatio: ("en", "Signal-to-noise ratio",
            "ANSI S3.5-1997: dB difference between signal and noise levels."),
        SpeechReceptionThreshold: ("en", "Speech reception threshold",
            "Katz et al. (2015): 50% spondaic-word recognition level."),
        ArticulationIndex: ("en", "Articulation index",
            "ANSI S3.5-1997: AI — predecessor of SII."),
        LowFrequencySpeech: ("en", "Low-frequency speech",
            "ANSI S3.5-1997: ~125-500 Hz speech band."),
        MidFrequencySpeech: ("en", "Mid-frequency speech",
            "ANSI S3.5-1997: ~500-3000 Hz speech band."),
        HighFrequencySpeech: ("en", "High-frequency speech",
            "ANSI S3.5-1997: ~3000-8000 Hz speech band."),
        AcousticParameter: ("en", "Acoustic parameter",
            "Fant (1960): umbrella for acoustic speech features."),
        SpeechSound: ("en", "Speech sound",
            "Stevens (2000): umbrella for vowels, consonants, and segmental units."),
        Suprasegmental: ("en", "Suprasegmental",
            "Stevens (2000): umbrella for prosodic features."),
        IntelligibilityMetric: ("en", "Intelligibility metric",
            "ANSI S3.5-1997: umbrella for speech-intelligibility measures."),
        SpectralRegion: ("en", "Spectral region",
            "ANSI S3.5-1997: umbrella for speech-band partitions."),
        CommunicativeIntent: ("en", "Communicative intent",
            "Stevens (2000): event — speaker's pragmatic goal."),
        ArticulatoryPlanning: ("en", "Articulatory planning",
            "Stevens (2000): event — motor planning of vocal-tract gestures."),
        VocalFoldVibration: ("en", "Vocal-fold vibration",
            "Fant (1960): event — periodic glottal closure cycle."),
        GlottalPulse: ("en", "Glottal pulse",
            "Fant (1960): event — pulse train from glottal closure."),
        VocalTractFiltering: ("en", "Vocal-tract filtering",
            "Fant (1960): event — vocal-tract resonant filtering of glottal source."),
        FormantProduction: ("en", "Formant production",
            "Fant (1960): event — formant peaks emerging at vocal-tract resonances."),
        AcousticRadiation: ("en", "Acoustic radiation",
            "Fant (1960): event — radiation of sound from lips/nostrils."),
        ListenerPerception: ("en", "Listener perception",
            "Stevens (2000): terminal event — listener auditory percept."),
        CoarticulationEffect: ("en", "Coarticulation effect",
            "Ohman (1966) JASA 39(1):151 — articulatory overlap across segments."),
        FormantTransition: ("en", "Formant transition",
            "Liberman et al. (1954) Psychol. Rev. 61(6):379 — formant motion at consonant-vowel boundary."),
        SpeechEvent: ("en", "Speech event",
            "Stevens (2000): umbrella concept for speech-production-perception perdurants."),
    },

    is_a: [
        (FundamentalFrequency, AcousticParameter), (Formant, AcousticParameter),
        (F1, Formant), (F2, Formant), (F3, Formant), (F4, Formant),
        (VoiceOnsetTime, AcousticParameter), (SpectralTilt, AcousticParameter),
        (Harmonics, AcousticParameter),
        (Vowel, SpeechSound), (Consonant, SpeechSound),
        (Plosive, Consonant), (Fricative, Consonant), (Nasal, Consonant),
        (Approximant, Consonant), (Affricate, Consonant),
        (Intonation, Suprasegmental), (Stress, Suprasegmental), (Rhythm, Suprasegmental),
        (SpeechIntelligibilityIndex, IntelligibilityMetric),
        (SignalToNoiseRatio, IntelligibilityMetric),
        (SpeechReceptionThreshold, IntelligibilityMetric),
        (ArticulationIndex, IntelligibilityMetric),
        (LowFrequencySpeech, SpectralRegion), (MidFrequencySpeech, SpectralRegion),
        (HighFrequencySpeech, SpectralRegion),
        (CommunicativeIntent, SpeechEvent), (ArticulatoryPlanning, SpeechEvent),
        (VocalFoldVibration, SpeechEvent), (GlottalPulse, SpeechEvent),
        (VocalTractFiltering, SpeechEvent), (FormantProduction, SpeechEvent),
        (AcousticRadiation, SpeechEvent), (ListenerPerception, SpeechEvent),
        (CoarticulationEffect, SpeechEvent), (FormantTransition, SpeechEvent),
    ],

    has_a: [
        (Phoneme, Vowel), (Phoneme, Consonant), (Syllable, Phoneme),
        (Vowel, F1), (Vowel, F2), (Vowel, F3),
        (Consonant, VoiceOnsetTime),
        (AcousticParameter, FundamentalFrequency), (AcousticParameter, SpectralTilt),
    ],

    causes: [
        (CommunicativeIntent, ArticulatoryPlanning),
        (ArticulatoryPlanning, VocalFoldVibration),
        (VocalFoldVibration, GlottalPulse),
        (GlottalPulse, VocalTractFiltering),
        (VocalTractFiltering, FormantProduction),
        (FormantProduction, AcousticRadiation),
        (AcousticRadiation, ListenerPerception),
        (ArticulatoryPlanning, CoarticulationEffect),
        (CoarticulationEffect, FormantTransition),
    ],

    opposes: [
        (Voiced, Voiceless), (Voiceless, Voiced),
        (Vowel, Consonant), (Consonant, Vowel),
    ],
}

#[derive(Debug, Clone)]
pub struct TypicalFrequency;
impl Quality for TypicalFrequency {
    type Individual = SpeechConcept;
    type Value = Quantity;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &SpeechConcept) -> Option<Quantity> {
        use SpeechConcept::*;
        match individual {
            FundamentalFrequency => Some(Quantity::from_unit(150.0, &HERTZ)),
            F1 => Some(Quantity::from_unit(500.0, &HERTZ)),
            F2 => Some(Quantity::from_unit(1500.0, &HERTZ)),
            F3 => Some(Quantity::from_unit(2500.0, &HERTZ)),
            F4 => Some(Quantity::from_unit(3500.0, &HERTZ)),
            LowFrequencySpeech => Some(Quantity::from_unit(250.0, &HERTZ)),
            MidFrequencySpeech => Some(Quantity::from_unit(1500.0, &HERTZ)),
            HighFrequencySpeech => Some(Quantity::from_unit(5000.0, &HERTZ)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpectralRange;
impl Quality for SpectralRange {
    type Individual = SpeechConcept;
    type Value = QuantityRange;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &SpeechConcept) -> Option<QuantityRange> {
        use SpeechConcept::*;
        match individual {
            LowFrequencySpeech => Some(QuantityRange {
                min: Quantity::from_unit(125.0, &HERTZ),
                max: Quantity::from_unit(500.0, &HERTZ),
            }),
            MidFrequencySpeech => Some(QuantityRange {
                min: Quantity::from_unit(500.0, &HERTZ),
                max: Quantity::from_unit(3000.0, &HERTZ),
            }),
            HighFrequencySpeech => Some(QuantityRange {
                min: Quantity::from_unit(3000.0, &HERTZ),
                max: Quantity::from_unit(8000.0, &HERTZ),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypicalVOT;
impl Quality for TypicalVOT {
    type Individual = SpeechConcept;
    type Value = Quantity;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &SpeechConcept) -> Option<Quantity> {
        use SpeechConcept::*;
        match individual {
            Voiced => Some(Quantity::from_unit(0.0, &MILLISECOND)),
            Voiceless => Some(Quantity::from_unit(70.0, &MILLISECOND)),
            Plosive => Some(Quantity::from_unit(35.0, &MILLISECOND)),
            VoiceOnsetTime => Some(Quantity::from_unit(35.0, &MILLISECOND)),
            _ => None,
        }
    }
}

fn is_a(child: SpeechConcept, parent: SpeechConcept) -> bool {
    use pr4xis::category::{Arrow, Category};
    if child == parent {
        return true;
    }
    SpeechCategory::morphisms().iter().any(|m| {
        m.kind() == SpeechRelationKind::Subsumption && m.source() == child && m.target() == parent
    })
}

fn parts_of(whole: SpeechConcept) -> Vec<SpeechConcept> {
    use pr4xis::category::{Arrow, Category};
    SpeechCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == SpeechRelationKind::Parthood && m.target() == whole)
        .map(|m| m.source())
        .collect()
}

fn effects_of(cause: SpeechConcept) -> Vec<SpeechConcept> {
    use pr4xis::category::{Arrow, Category};
    SpeechCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == SpeechRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

pub struct FormantsAreOrdered;
impl Axiom for FormantsAreOrdered {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use SpeechConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let f = TypicalFrequency;
        // All formant frequencies share the HERTZ unit, so comparing the SI
        // `.value` fields preserves the frequency ordering exactly.
        let f1 = f.get(&F1).map(|q| q.value).unwrap_or(0.0);
        let f2 = f.get(&F2).map(|q| q.value).unwrap_or(0.0);
        let f3 = f.get(&F3).map(|q| q.value).unwrap_or(0.0);
        let f4 = f.get(&F4).map(|q| q.value).unwrap_or(0.0);
        if f1 < f2 && f2 < f3 && f3 < f4 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "FormantsAreOrdered",
        "formants are frequency-ordered (F1 < F2 < F3 < F4)",
        "Peterson & Barney (1952) JASA 24(2):175"
    );
}
pr4xis::register_axiom!(
    FormantsAreOrdered,
    "Peterson & Barney (1952) JASA 24(2):175"
);

pub struct FormantsClassified;
impl Axiom for FormantsClassified {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use SpeechConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = [F1, F2, F3, F4]
            .iter()
            .all(|f| is_a(*f, Formant) && is_a(*f, AcousticParameter));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "FormantsClassified",
        "F1-F4 are all formants and acoustic parameters",
        "Fant (1960) Acoustic Theory of Speech Production"
    );
}
pr4xis::register_axiom!(
    FormantsClassified,
    "Fant (1960) Acoustic Theory of Speech Production"
);

pub struct FiveConsonantManners;
impl Axiom for FiveConsonantManners {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use SpeechConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let ok = [Plosive, Fricative, Nasal, Approximant, Affricate]
            .iter()
            .all(|c| is_a(*c, Consonant));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "FiveConsonantManners",
        "plosive, fricative, nasal, approximant, affricate are consonants",
        "Stevens (2000) Acoustic Phonetics"
    );
}
pr4xis::register_axiom!(FiveConsonantManners, "Stevens (2000) Acoustic Phonetics");

pub struct SyllableContainsVowelsAndConsonants;
impl Axiom for SyllableContainsVowelsAndConsonants {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use SpeechConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts = parts_of(Syllable);
        if parts.contains(&Vowel) && parts.contains(&Consonant) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "SyllableContainsVowelsAndConsonants",
        "syllable transitively contains vowels and consonants",
        "Stevens (2000) Acoustic Phonetics"
    );
}
pr4xis::register_axiom!(
    SyllableContainsVowelsAndConsonants,
    "Stevens (2000) Acoustic Phonetics"
);

pub struct IntentCausesPerception;
impl Axiom for IntentCausesPerception {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use SpeechConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(CommunicativeIntent).contains(&ListenerPerception) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "IntentCausesPerception",
        "communicative intent transitively causes listener perception",
        "Stevens (2000) Acoustic Phonetics"
    );
}
pr4xis::register_axiom!(IntentCausesPerception, "Stevens (2000) Acoustic Phonetics");

impl Ontology for SpeechOntology {
    type Cat = SpeechCategory;
    type Qual = TypicalFrequency;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(FormantsAreOrdered));
        a.push(Box::new(FormantsClassified));
        a.push(Box::new(FiveConsonantManners));
        a.push(Box::new(SyllableContainsVowelsAndConsonants));
        a.push(Box::new(IntentCausesPerception));
        a
    }
}

// Back-compat aliases.
pub use SpeechConcept as SpeechEntity;
pub use SpeechRelationKind as SpeechCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<SpeechCategory>();
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        SpeechOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn formants_are_ordered() {
        assert!(FormantsAreOrdered.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn formants_classified() {
        assert!(FormantsClassified.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn five_consonant_manners() {
        assert!(FiveConsonantManners.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn syllable_contains() {
        assert!(SyllableContainsVowelsAndConsonants.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn intent_causes_perception() {
        assert!(IntentCausesPerception.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in SpeechCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in SpeechOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
