//! Music perception ontology.
//!
//! Models how the auditory system perceives musical structure.
//!
//! Key references:
//! - Helmholtz 1863: On the Sensations of Tone
//! - Krumhansl 1990: Cognitive Foundations of Musical Pitch
//! - Lerdahl & Jackendoff 1983: Generative Theory of Tonal Music
//! - Huron 2006: Sweet Anticipation
//! - Plomp & Levelt 1965: consonance and critical bandwidth
//! - Large & Palmer 2002: neural resonance theory of rhythm
//! - Patel 2008: Music, Language, and the Brain
//! - McDermott & Oxenham 2008: music perception review

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::opposition;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum MusicEntity {
    PitchHeight,
    PitchChroma,
    OctaveEquivalence,
    AbsolutePitch,
    RelativePitch,
    MelodicContour,
    IntervalPerception,
    Consonance,
    Dissonance,
    RoughnessModel,
    HarmonicSeries,
    VirtualPitchPercept,
    MissingFundamental,
    Chord,
    Tonality,
    KeySense,
    Beat,
    Meter,
    Tempo,
    Syncopation,
    Groove,
    Entrainment,
    TemporalExpectation,
    SpectralCentroid,
    AttackTime,
    SpectralFlux,
    InstrumentIdentification,
    MusicalExpectation,
    Surprise,
    Tension,
    Resolution,
    MusicalEmotion,
    EarWorm,
    MusicalMemory,
    TonalSchemaMemory,
    PitchPercept,
    HarmonicPercept,
    RhythmicPercept,
    TimbrePercept,
    AffectiveResponse,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum MusicCausalEvent {
    AuditoryInput,
    PitchExtraction,
    OnsetDetection,
    HarmonicGrouping,
    MelodicTracking,
    BeatInduction,
    MetricFraming,
    TonalInterpretation,
    MusicalExpectationFormation,
    GroovePerception,
    EmotionalResponse,
}
define_ontology! {
    /// Discrete category over music perception entities.
    pub MusicPerceptionOntology for MusicPerceptionCategory {
        entity: MusicEntity, relation: MusicRelation,
        being: MentalObject,
        source: "Helmholtz (1863); Krumhansl (1990)",
        taxonomy: MusicTaxonomy [
            (PitchHeight, PitchPercept), (PitchChroma, PitchPercept), (OctaveEquivalence, PitchPercept), (AbsolutePitch, PitchPercept), (RelativePitch, PitchPercept), (MelodicContour, PitchPercept), (IntervalPerception, PitchPercept),
            (Consonance, HarmonicPercept), (Dissonance, HarmonicPercept), (RoughnessModel, HarmonicPercept), (HarmonicSeries, HarmonicPercept), (VirtualPitchPercept, HarmonicPercept), (MissingFundamental, HarmonicPercept), (Chord, HarmonicPercept), (Tonality, HarmonicPercept), (KeySense, HarmonicPercept),
            (Beat, RhythmicPercept), (Meter, RhythmicPercept), (Tempo, RhythmicPercept), (Syncopation, RhythmicPercept), (Groove, RhythmicPercept), (Entrainment, RhythmicPercept), (TemporalExpectation, RhythmicPercept),
            (SpectralCentroid, TimbrePercept), (AttackTime, TimbrePercept), (SpectralFlux, TimbrePercept), (InstrumentIdentification, TimbrePercept),
            (MusicalExpectation, AffectiveResponse), (Surprise, AffectiveResponse), (Tension, AffectiveResponse), (Resolution, AffectiveResponse), (MusicalEmotion, AffectiveResponse),
        ],
        causation: MusicCausalGraph for MusicCausalEvent [
            (AuditoryInput, PitchExtraction), (AuditoryInput, OnsetDetection), (PitchExtraction, HarmonicGrouping), (PitchExtraction, MelodicTracking), (OnsetDetection, BeatInduction), (BeatInduction, MetricFraming), (HarmonicGrouping, TonalInterpretation), (MelodicTracking, MusicalExpectationFormation), (MetricFraming, GroovePerception), (TonalInterpretation, EmotionalResponse), (MusicalExpectationFormation, EmotionalResponse),
        ],
        opposition: MusicOpposition [ (Consonance, Dissonance), (Tension, Resolution), (AbsolutePitch, RelativePitch) ],
    }
}
#[derive(Debug, Clone)]
pub struct ConsonanceRanking;
impl Quality for ConsonanceRanking {
    type Individual = MusicEntity;
    type Value = u32;
    fn get(&self, individual: &MusicEntity) -> Option<u32> {
        use MusicEntity::*;
        match individual {
            Consonance => Some(1),
            Dissonance => Some(10),
            OctaveEquivalence => Some(1),
            _ => None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct PreferredTempoBPM;
impl Quality for PreferredTempoBPM {
    type Individual = MusicEntity;
    type Value = f64;
    fn get(&self, individual: &MusicEntity) -> Option<f64> {
        use MusicEntity::*;
        match individual {
            Tempo => Some(120.0),
            Beat => Some(120.0),
            Entrainment => Some(120.0),
            _ => None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct OctaveRatio;
impl Quality for OctaveRatio {
    type Individual = MusicEntity;
    type Value = f64;
    fn get(&self, individual: &MusicEntity) -> Option<f64> {
        match individual {
            MusicEntity::OctaveEquivalence => Some(2.0),
            _ => None,
        }
    }
}

pub struct OctaveRatioIsTwo;
impl Axiom for OctaveRatioIsTwo {
    fn description(&self) -> &str {
        "octave equivalence has a 2:1 frequency ratio"
    }
    fn holds(&self) -> bool {
        OctaveRatio.get(&MusicEntity::OctaveEquivalence) == Some(2.0)
    }
}
pr4xis::register_axiom!(OctaveRatioIsTwo);
pub struct ConsonanceOpposesDissonance;
impl Axiom for ConsonanceOpposesDissonance {
    fn description(&self) -> &str {
        "consonance and dissonance are opposed"
    }
    fn holds(&self) -> bool {
        opposition::are_opposed::<MusicOpposition>(
            &MusicEntity::Consonance,
            &MusicEntity::Dissonance,
        )
    }
}
pr4xis::register_axiom!(ConsonanceOpposesDissonance);
pub struct TensionOpposesResolution;
impl Axiom for TensionOpposesResolution {
    fn description(&self) -> &str {
        "tension and resolution are opposed"
    }
    fn holds(&self) -> bool {
        opposition::are_opposed::<MusicOpposition>(&MusicEntity::Tension, &MusicEntity::Resolution)
    }
}
pr4xis::register_axiom!(TensionOpposesResolution);
pub struct ConsonanceRankedHigher;
impl Axiom for ConsonanceRankedHigher {
    fn description(&self) -> &str {
        "consonance ranks higher (lower number) than dissonance"
    }
    fn holds(&self) -> bool {
        use MusicEntity::*;
        ConsonanceRanking.get(&Consonance).unwrap() < ConsonanceRanking.get(&Dissonance).unwrap()
    }
}
pr4xis::register_axiom!(ConsonanceRankedHigher);
pub struct FivePerceptualCategories;
impl Axiom for FivePerceptualCategories {
    fn description(&self) -> &str {
        "pitch, harmonic, rhythmic, timbre, and affective categories exist"
    }
    fn holds(&self) -> bool {
        use MusicEntity::*;
        taxonomy::is_a::<MusicTaxonomy>(&PitchHeight, &PitchPercept)
            && taxonomy::is_a::<MusicTaxonomy>(&Consonance, &HarmonicPercept)
            && taxonomy::is_a::<MusicTaxonomy>(&Beat, &RhythmicPercept)
            && taxonomy::is_a::<MusicTaxonomy>(&SpectralCentroid, &TimbrePercept)
            && taxonomy::is_a::<MusicTaxonomy>(&MusicalEmotion, &AffectiveResponse)
    }
}
pr4xis::register_axiom!(FivePerceptualCategories);
pub struct InputCausesEmotion;
impl Axiom for InputCausesEmotion {
    fn description(&self) -> &str {
        "auditory input transitively causes emotional response"
    }
    fn holds(&self) -> bool {
        use MusicCausalEvent::*;
        causation::effects_of::<MusicCausalGraph>(&AuditoryInput).contains(&EmotionalResponse)
    }
}
pr4xis::register_axiom!(InputCausesEmotion);

impl Ontology for MusicPerceptionOntology {
    type Cat = MusicPerceptionCategory;
    type Qual = PreferredTempoBPM;
    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }
    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(ConsonanceOpposesDissonance),
            Box::new(TensionOpposesResolution),
            Box::new(ConsonanceRankedHigher),
            Box::new(FivePerceptualCategories),
            Box::new(OctaveRatioIsTwo),
            Box::new(InputCausesEmotion),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;
    #[test]
    fn test_consonance_dissonance() {
        assert!(ConsonanceOpposesDissonance.holds());
    }
    #[test]
    fn test_tension_resolution() {
        assert!(TensionOpposesResolution.holds());
    }
    #[test]
    fn test_consonance_ranked() {
        assert!(ConsonanceRankedHigher.holds());
    }
    #[test]
    fn test_five_categories() {
        assert!(FivePerceptualCategories.holds());
    }
    #[test]
    fn test_input_causes_emotion() {
        assert!(InputCausesEmotion.holds());
    }
    #[test]
    fn test_category_laws() {
        check_category_laws::<MusicPerceptionCategory>().unwrap();
    }
    #[test]
    fn test_taxonomy_laws() {
        check_category_laws::<TaxonomyCategory<MusicTaxonomy>>().unwrap();
    }
    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<MusicCausalGraph>>().unwrap();
    }
    #[test]
    fn test_octave_ratio_is_two() {
        assert!(OctaveRatioIsTwo.holds());
    }
    #[test]
    fn test_octave_ratio_value() {
        assert_eq!(OctaveRatio.get(&MusicEntity::OctaveEquivalence), Some(2.0));
    }
    #[test]
    fn test_preferred_tempo() {
        assert_eq!(PreferredTempoBPM.get(&MusicEntity::Tempo), Some(120.0));
    }
    #[test]
    fn test_entity_count() {
        assert_eq!(MusicEntity::variants().len(), 40);
    }
    #[test]
    fn test_ontology_validates() {
        MusicPerceptionOntology::validate().unwrap();
    }
    fn arb_entity() -> impl Strategy<Value = MusicEntity> {
        (0..MusicEntity::variants().len()).prop_map(|i| MusicEntity::variants()[i])
    }
    proptest! { #[test] fn prop_taxonomy_reflexive(entity in arb_entity()) { prop_assert!(taxonomy::is_a::<MusicTaxonomy>(&entity, &entity)); } }
}
