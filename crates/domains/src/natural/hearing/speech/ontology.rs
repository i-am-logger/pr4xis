//! Speech acoustics ontology.
//!
//! Models speech production and perception acoustics.
//!
//! Key references:
//! - Fant 1960: Acoustic Theory of Speech Production
//! - Peterson & Barney 1952: vowel formant measurements
//! - Stevens 2000: Acoustic Phonetics
//! - Lisker & Abramson 1964: voice onset time
//! - ANSI S3.5-1997: Speech Intelligibility Index

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
pub enum SpeechEntity {
    // Acoustic parameters
    FundamentalFrequency,
    Formant,
    F1,
    F2,
    F3,
    F4,
    VoiceOnsetTime,
    SpectralTilt,
    Harmonics,
    // Speech sounds
    Vowel,
    Consonant,
    Plosive,
    Fricative,
    Nasal,
    Approximant,
    Affricate,
    // Voicing
    Voiced,
    Voiceless,
    // Suprasegmentals
    Intonation,
    Stress,
    Rhythm,
    Syllable,
    Phoneme,
    // Intelligibility
    SpeechIntelligibilityIndex,
    SignalToNoiseRatio,
    SpeechReceptionThreshold,
    ArticulationIndex,
    // Spectral regions
    LowFrequencySpeech,
    MidFrequencySpeech,
    HighFrequencySpeech,
    // Abstract categories
    AcousticParameter,
    SpeechSound,
    Suprasegmental,
    IntelligibilityMetric,
    SpectralRegion,
}

// ---------------------------------------------------------------------------
// Taxonomy
// ---------------------------------------------------------------------------

pub struct SpeechTaxonomy;

impl TaxonomyDef for SpeechTaxonomy {
    type Entity = SpeechEntity;

    fn relations() -> Vec<(SpeechEntity, SpeechEntity)> {
        use SpeechEntity::*;
        vec![
            // Acoustic parameters
            (FundamentalFrequency, AcousticParameter),
            (Formant, AcousticParameter),
            (F1, Formant),
            (F2, Formant),
            (F3, Formant),
            (F4, Formant),
            (VoiceOnsetTime, AcousticParameter),
            (SpectralTilt, AcousticParameter),
            (Harmonics, AcousticParameter),
            // Speech sounds
            (Vowel, SpeechSound),
            (Consonant, SpeechSound),
            (Plosive, Consonant),
            (Fricative, Consonant),
            (Nasal, Consonant),
            (Approximant, Consonant),
            (Affricate, Consonant),
            // Suprasegmentals
            (Intonation, Suprasegmental),
            (Stress, Suprasegmental),
            (Rhythm, Suprasegmental),
            // Intelligibility metrics
            (SpeechIntelligibilityIndex, IntelligibilityMetric),
            (SignalToNoiseRatio, IntelligibilityMetric),
            (SpeechReceptionThreshold, IntelligibilityMetric),
            (ArticulationIndex, IntelligibilityMetric),
            // Spectral regions
            (LowFrequencySpeech, SpectralRegion),
            (MidFrequencySpeech, SpectralRegion),
            (HighFrequencySpeech, SpectralRegion),
        ]
    }
}

// ---------------------------------------------------------------------------
// Mereology (has-a / part-whole)
// ---------------------------------------------------------------------------

/// Part-whole relationships for speech acoustics.
///
/// Phonemes are composed of vowels and consonants; syllables contain phonemes;
/// vowels are characterised by formant frequencies; consonants by voice onset
/// time; and acoustic parameters include fundamental frequency and spectral
/// tilt.
pub struct SpeechMereology;

impl MereologyDef for SpeechMereology {
    type Entity = SpeechEntity;

    fn relations() -> Vec<(SpeechEntity, SpeechEntity)> {
        use SpeechEntity::*;
        vec![
            // Phoneme composition
            (Phoneme, Vowel),
            (Phoneme, Consonant),
            // Syllable contains phonemes
            (Syllable, Phoneme),
            // Vowel characterised by formant frequencies
            (Vowel, F1),
            (Vowel, F2),
            (Vowel, F3),
            // Consonant characterised by voice onset time
            (Consonant, VoiceOnsetTime),
            // Acoustic parameter composition
            (AcousticParameter, FundamentalFrequency),
            (AcousticParameter, SpectralTilt),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Causal events in the speech production chain.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum SpeechCausalEvent {
    CommunicativeIntent,
    ArticulatoryPlanning,
    VocalFoldVibration,
    GlottalPulse,
    VocalTractFiltering,
    FormantProduction,
    AcousticRadiation,
    ListenerPerception,
    CoarticulationEffect,
    FormantTransition,
}

/// Causal graph for speech production pipeline.
pub struct SpeechCausalGraph;

impl CausalDef for SpeechCausalGraph {
    type Entity = SpeechCausalEvent;

    fn relations() -> Vec<(SpeechCausalEvent, SpeechCausalEvent)> {
        use SpeechCausalEvent::*;
        vec![
            // Intent initiates articulatory planning
            (CommunicativeIntent, ArticulatoryPlanning),
            // Articulatory plan activates vocal fold vibration
            (ArticulatoryPlanning, VocalFoldVibration),
            // Vocal fold vibration generates glottal pulse
            (VocalFoldVibration, GlottalPulse),
            // Glottal pulse filtered by vocal tract
            (GlottalPulse, VocalTractFiltering),
            // Vocal tract filtering produces formants
            (VocalTractFiltering, FormantProduction),
            // Formants radiate acoustically
            (FormantProduction, AcousticRadiation),
            // Acoustic radiation reaches listener
            (AcousticRadiation, ListenerPerception),
            // Articulatory planning also causes coarticulation
            (ArticulatoryPlanning, CoarticulationEffect),
            // Coarticulation produces formant transitions
            (CoarticulationEffect, FormantTransition),
        ]
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

pub struct SpeechOpposition;

impl OppositionDef for SpeechOpposition {
    type Entity = SpeechEntity;

    fn pairs() -> Vec<(SpeechEntity, SpeechEntity)> {
        use SpeechEntity::*;
        vec![(Voiced, Voiceless), (Vowel, Consonant)]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over speech entities.
    pub SpeechCategory {
        entity: SpeechEntity,
        relation: SpeechRelation,
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Typical frequency (Hz) for speech parameters.
///
/// - F0 male: ~120 Hz, F0 female: ~220 Hz (Peterson & Barney 1952)
/// - F1: 250-900 Hz (vowel height)
/// - F2: 800-2500 Hz (vowel frontness)
/// - F3: 1800-3500 Hz (rhoticity)
#[derive(Debug, Clone)]
pub struct TypicalFrequency;

impl Quality for TypicalFrequency {
    type Individual = SpeechEntity;
    type Value = f64;

    fn get(&self, individual: &SpeechEntity) -> Option<f64> {
        use SpeechEntity::*;
        match individual {
            FundamentalFrequency => Some(150.0), // average adult
            F1 => Some(500.0),                   // average across vowels
            F2 => Some(1500.0),                  // average across vowels
            F3 => Some(2500.0),
            F4 => Some(3500.0),
            LowFrequencySpeech => Some(250.0),   // 125-500 Hz
            MidFrequencySpeech => Some(1500.0),  // 500-3000 Hz
            HighFrequencySpeech => Some(5000.0), // 3000-8000 Hz
            _ => None,
        }
    }
}

/// Frequency range for spectral regions (Hz).
#[derive(Debug, Clone, PartialEq)]
pub struct FreqRange {
    pub low: f64,
    pub high: f64,
}

/// Quality: frequency range for spectral regions.
#[derive(Debug, Clone)]
pub struct SpectralRange;

impl Quality for SpectralRange {
    type Individual = SpeechEntity;
    type Value = FreqRange;

    fn get(&self, individual: &SpeechEntity) -> Option<FreqRange> {
        use SpeechEntity::*;
        match individual {
            LowFrequencySpeech => Some(FreqRange {
                low: 125.0,
                high: 500.0,
            }),
            MidFrequencySpeech => Some(FreqRange {
                low: 500.0,
                high: 3000.0,
            }),
            HighFrequencySpeech => Some(FreqRange {
                low: 3000.0,
                high: 8000.0,
            }),
            _ => None,
        }
    }
}

/// Voice onset time (ms) for consonant categories.
///
/// Voiced plosives: ~0 ms (prevoiced in some languages)
/// Voiceless unaspirated: ~20 ms
/// Voiceless aspirated: ~70 ms
///
/// Lisker & Abramson, *Word*, 1964.
#[derive(Debug, Clone)]
pub struct TypicalVOT;

impl Quality for TypicalVOT {
    type Individual = SpeechEntity;
    type Value = f64;

    fn get(&self, individual: &SpeechEntity) -> Option<f64> {
        use SpeechEntity::*;
        match individual {
            Voiced => Some(0.0),          // ms, prevoiced
            Voiceless => Some(70.0),      // ms, aspirated
            Plosive => Some(35.0),        // ms, average across voicing
            VoiceOnsetTime => Some(35.0), // ms, reference value
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

pub struct SpeechTaxonomyIsDAG;
impl Axiom for SpeechTaxonomyIsDAG {
    fn description(&self) -> &str {
        "speech taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        taxonomy::NoCycles::<SpeechTaxonomy>::new().holds()
    }
}

/// F1 < F2 < F3 < F4 (formant ordering).
///
/// Fant 1960.
pub struct FormantsAreOrdered;
impl Axiom for FormantsAreOrdered {
    fn description(&self) -> &str {
        "formants are frequency-ordered (F1 < F2 < F3 < F4)"
    }
    fn holds(&self) -> bool {
        use SpeechEntity::*;
        let f = TypicalFrequency;
        f.get(&F1).unwrap() < f.get(&F2).unwrap()
            && f.get(&F2).unwrap() < f.get(&F3).unwrap()
            && f.get(&F3).unwrap() < f.get(&F4).unwrap()
    }
}

/// All formants (F1-F4) are classified as Formant is-a AcousticParameter.
pub struct FormantsClassified;
impl Axiom for FormantsClassified {
    fn description(&self) -> &str {
        "F1-F4 are all formants, which are acoustic parameters"
    }
    fn holds(&self) -> bool {
        use SpeechEntity::*;
        [F1, F2, F3, F4].iter().all(|f| {
            taxonomy::is_a::<SpeechTaxonomy>(f, &Formant)
                && taxonomy::is_a::<SpeechTaxonomy>(f, &AcousticParameter)
        })
    }
}

/// Five consonant manner classes are classified.
pub struct FiveConsonantManners;
impl Axiom for FiveConsonantManners {
    fn description(&self) -> &str {
        "plosive, fricative, nasal, approximant, affricate are consonants"
    }
    fn holds(&self) -> bool {
        use SpeechEntity::*;
        [Plosive, Fricative, Nasal, Approximant, Affricate]
            .iter()
            .all(|c| taxonomy::is_a::<SpeechTaxonomy>(c, &Consonant))
    }
}

/// Voiced opposes voiceless.
pub struct VoicedOpposesVoiceless;
impl Axiom for VoicedOpposesVoiceless {
    fn description(&self) -> &str {
        "voiced and voiceless are opposed"
    }
    fn holds(&self) -> bool {
        opposition::are_opposed::<SpeechOpposition>(&SpeechEntity::Voiced, &SpeechEntity::Voiceless)
    }
}

/// Mereology is a DAG.
pub struct SpeechMereologyIsDAG;
impl Axiom for SpeechMereologyIsDAG {
    fn description(&self) -> &str {
        "speech mereology is a DAG"
    }
    fn holds(&self) -> bool {
        mereology::NoCycles::<SpeechMereology>::new().holds()
    }
}

/// Syllable transitively contains vowels and consonants (via Phoneme).
///
/// A syllable has phonemes, and phonemes have vowels and consonants.
pub struct SyllableContainsVowelsAndConsonants;
impl Axiom for SyllableContainsVowelsAndConsonants {
    fn description(&self) -> &str {
        "syllable transitively contains vowels and consonants"
    }
    fn holds(&self) -> bool {
        use SpeechEntity::*;
        let parts = mereology::parts_of::<SpeechMereology>(&Syllable);
        parts.contains(&Vowel) && parts.contains(&Consonant)
    }
}

pub struct SpeechOppositionSymmetric;
impl Axiom for SpeechOppositionSymmetric {
    fn description(&self) -> &str {
        "speech opposition is symmetric"
    }
    fn holds(&self) -> bool {
        opposition::Symmetric::<SpeechOpposition>::new().holds()
    }
}

/// Causal graph is asymmetric.
pub struct SpeechCausalGraphIsAsymmetric;
impl Axiom for SpeechCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "speech causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<SpeechCausalGraph>::new().holds()
    }
}

/// No event causes itself.
pub struct SpeechCausalGraphNoSelfCause;
impl Axiom for SpeechCausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no speech production event causes itself"
    }
    fn holds(&self) -> bool {
        causation::NoSelfCausation::<SpeechCausalGraph>::new().holds()
    }
}

/// Communicative intent transitively causes listener perception.
pub struct IntentCausesPerception;
impl Axiom for IntentCausesPerception {
    fn description(&self) -> &str {
        "communicative intent transitively causes listener perception"
    }
    fn holds(&self) -> bool {
        use SpeechCausalEvent::*;
        let effects = causation::effects_of::<SpeechCausalGraph>(&CommunicativeIntent);
        effects.contains(&ListenerPerception)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

pub struct SpeechOntology;

impl Ontology for SpeechOntology {
    type Cat = SpeechCategory;
    type Qual = TypicalFrequency;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(SpeechTaxonomyIsDAG),
            Box::new(SpeechMereologyIsDAG),
            Box::new(SyllableContainsVowelsAndConsonants),
            Box::new(FormantsAreOrdered),
            Box::new(FormantsClassified),
            Box::new(FiveConsonantManners),
            Box::new(VoicedOpposesVoiceless),
            Box::new(SpeechOppositionSymmetric),
            Box::new(SpeechCausalGraphIsAsymmetric),
            Box::new(SpeechCausalGraphNoSelfCause),
            Box::new(IntentCausesPerception),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::mereology::MereologyCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_taxonomy_is_dag() {
        assert!(SpeechTaxonomyIsDAG.holds());
    }
    #[test]
    fn test_mereology_is_dag() {
        assert!(SpeechMereologyIsDAG.holds());
    }
    #[test]
    fn test_syllable_contains_vowels_and_consonants() {
        assert!(SyllableContainsVowelsAndConsonants.holds());
    }
    #[test]
    fn test_formants_ordered() {
        assert!(FormantsAreOrdered.holds());
    }
    #[test]
    fn test_formants_classified() {
        assert!(FormantsClassified.holds());
    }
    #[test]
    fn test_five_manners() {
        assert!(FiveConsonantManners.holds());
    }
    #[test]
    fn test_voiced_voiceless() {
        assert!(VoicedOpposesVoiceless.holds());
    }
    #[test]
    fn test_opposition_symmetric() {
        assert!(SpeechOppositionSymmetric.holds());
    }
    #[test]
    fn test_causal_graph_asymmetric() {
        assert!(SpeechCausalGraphIsAsymmetric.holds());
    }
    #[test]
    fn test_causal_graph_no_self_cause() {
        assert!(SpeechCausalGraphNoSelfCause.holds());
    }
    #[test]
    fn test_intent_causes_perception() {
        assert!(IntentCausesPerception.holds());
    }

    #[test]
    fn test_category_laws() {
        check_category_laws::<SpeechCategory>().unwrap();
    }
    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<SpeechTaxonomy>>().unwrap();
    }
    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<SpeechCausalGraph>>().unwrap();
    }
    #[test]
    fn test_mereology_category_laws() {
        check_category_laws::<MereologyCategory<SpeechMereology>>().unwrap();
    }

    #[test]
    fn test_f0_value() {
        assert_eq!(
            TypicalFrequency.get(&SpeechEntity::FundamentalFrequency),
            Some(150.0)
        );
    }

    #[test]
    fn test_entity_count() {
        assert_eq!(SpeechEntity::variants().len(), 35);
    }
    #[test]
    fn test_ontology_validates() {
        SpeechOntology::validate().unwrap();
    }

    fn arb_entity() -> impl Strategy<Value = SpeechEntity> {
        (0..SpeechEntity::variants().len()).prop_map(|i| SpeechEntity::variants()[i])
    }
    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<SpeechTaxonomy>(&entity, &entity));
        }
    }
}
