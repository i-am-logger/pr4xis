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
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::mereology;
use pr4xis::ontology::reasoning::opposition;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum SpeechEntity {
    FundamentalFrequency,
    Formant,
    F1,
    F2,
    F3,
    F4,
    VoiceOnsetTime,
    SpectralTilt,
    Harmonics,
    Vowel,
    Consonant,
    Plosive,
    Fricative,
    Nasal,
    Approximant,
    Affricate,
    Voiced,
    Voiceless,
    Intonation,
    Stress,
    Rhythm,
    Syllable,
    Phoneme,
    SpeechIntelligibilityIndex,
    SignalToNoiseRatio,
    SpeechReceptionThreshold,
    ArticulationIndex,
    LowFrequencySpeech,
    MidFrequencySpeech,
    HighFrequencySpeech,
    AcousticParameter,
    SpeechSound,
    Suprasegmental,
    IntelligibilityMetric,
    SpectralRegion,
}

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

define_ontology! {
    /// Discrete category over speech entities.
    pub SpeechOntology for SpeechCategory {
        entity: SpeechEntity,
        relation: SpeechRelation,

        taxonomy: SpeechTaxonomy [
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
        ],

        mereology: SpeechMereology [
            (Phoneme, Vowel), (Phoneme, Consonant), (Syllable, Phoneme),
            (Vowel, F1), (Vowel, F2), (Vowel, F3),
            (Consonant, VoiceOnsetTime),
            (AcousticParameter, FundamentalFrequency), (AcousticParameter, SpectralTilt),
        ],

        causation: SpeechCausalGraph for SpeechCausalEvent [
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

        opposition: SpeechOpposition [
            (Voiced, Voiceless), (Vowel, Consonant),
        ],
    }
}

#[derive(Debug, Clone)]
pub struct TypicalFrequency;
impl Quality for TypicalFrequency {
    type Individual = SpeechEntity;
    type Value = f64;
    fn get(&self, individual: &SpeechEntity) -> Option<f64> {
        use SpeechEntity::*;
        match individual {
            FundamentalFrequency => Some(150.0),
            F1 => Some(500.0),
            F2 => Some(1500.0),
            F3 => Some(2500.0),
            F4 => Some(3500.0),
            LowFrequencySpeech => Some(250.0),
            MidFrequencySpeech => Some(1500.0),
            HighFrequencySpeech => Some(5000.0),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreqRange {
    pub low: f64,
    pub high: f64,
}

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

#[derive(Debug, Clone)]
pub struct TypicalVOT;
impl Quality for TypicalVOT {
    type Individual = SpeechEntity;
    type Value = f64;
    fn get(&self, individual: &SpeechEntity) -> Option<f64> {
        use SpeechEntity::*;
        match individual {
            Voiced => Some(0.0),
            Voiceless => Some(70.0),
            Plosive => Some(35.0),
            VoiceOnsetTime => Some(35.0),
            _ => None,
        }
    }
}

pub struct SpeechTaxonomyIsDAG;
impl Axiom for SpeechTaxonomyIsDAG {
    fn description(&self) -> &str {
        "speech taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        taxonomy::NoCycles::<SpeechTaxonomy>::new().holds()
    }
}
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
pub struct VoicedOpposesVoiceless;
impl Axiom for VoicedOpposesVoiceless {
    fn description(&self) -> &str {
        "voiced and voiceless are opposed"
    }
    fn holds(&self) -> bool {
        opposition::are_opposed::<SpeechOpposition>(&SpeechEntity::Voiced, &SpeechEntity::Voiceless)
    }
}
pub struct SpeechMereologyIsDAG;
impl Axiom for SpeechMereologyIsDAG {
    fn description(&self) -> &str {
        "speech mereology is a DAG"
    }
    fn holds(&self) -> bool {
        mereology::NoCycles::<SpeechMereology>::new().holds()
    }
}
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
pub struct SpeechCausalGraphIsAsymmetric;
impl Axiom for SpeechCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "speech causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<SpeechCausalGraph>::new().holds()
    }
}
pub struct SpeechCausalGraphNoSelfCause;
impl Axiom for SpeechCausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no speech production event causes itself"
    }
    fn holds(&self) -> bool {
        causation::NoSelfCausation::<SpeechCausalGraph>::new().holds()
    }
}
pub struct IntentCausesPerception;
impl Axiom for IntentCausesPerception {
    fn description(&self) -> &str {
        "communicative intent transitively causes listener perception"
    }
    fn holds(&self) -> bool {
        use SpeechCausalEvent::*;
        causation::effects_of::<SpeechCausalGraph>(&CommunicativeIntent)
            .contains(&ListenerPerception)
    }
}

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
