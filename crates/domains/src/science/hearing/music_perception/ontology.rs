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
use pr4xis::define_dense_category;
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum MusicEntity {
    // Pitch perception
    PitchHeight,
    PitchChroma,
    OctaveEquivalence,
    AbsolutePitch,
    RelativePitch,
    MelodicContour,
    IntervalPerception,
    // Harmony
    Consonance,
    Dissonance,
    RoughnessModel,
    HarmonicSeries,
    VirtualPitchPercept,
    MissingFundamental,
    Chord,
    Tonality,
    KeySense,
    // Rhythm & meter
    Beat,
    Meter,
    Tempo,
    Syncopation,
    Groove,
    Entrainment,
    TemporalExpectation,
    // Timbre
    SpectralCentroid,
    AttackTime,
    SpectralFlux,
    InstrumentIdentification,
    // Expectation & emotion
    MusicalExpectation,
    Surprise,
    Tension,
    Resolution,
    MusicalEmotion,
    // Memory
    EarWorm,
    MusicalMemory,
    TonalSchemaMemory,
    // Abstract categories
    PitchPercept,
    HarmonicPercept,
    RhythmicPercept,
    TimbrePercept,
    AffectiveResponse,
}

// ---------------------------------------------------------------------------
// Taxonomy
// ---------------------------------------------------------------------------

pub struct MusicTaxonomy;

impl TaxonomyDef for MusicTaxonomy {
    type Entity = MusicEntity;

    fn relations() -> Vec<(MusicEntity, MusicEntity)> {
        use MusicEntity::*;
        vec![
            // Pitch percepts
            (PitchHeight, PitchPercept),
            (PitchChroma, PitchPercept),
            (OctaveEquivalence, PitchPercept),
            (AbsolutePitch, PitchPercept),
            (RelativePitch, PitchPercept),
            (MelodicContour, PitchPercept),
            (IntervalPerception, PitchPercept),
            // Harmonic percepts
            (Consonance, HarmonicPercept),
            (Dissonance, HarmonicPercept),
            (RoughnessModel, HarmonicPercept),
            (HarmonicSeries, HarmonicPercept),
            (VirtualPitchPercept, HarmonicPercept),
            (MissingFundamental, HarmonicPercept),
            (Chord, HarmonicPercept),
            (Tonality, HarmonicPercept),
            (KeySense, HarmonicPercept),
            // Rhythmic percepts
            (Beat, RhythmicPercept),
            (Meter, RhythmicPercept),
            (Tempo, RhythmicPercept),
            (Syncopation, RhythmicPercept),
            (Groove, RhythmicPercept),
            (Entrainment, RhythmicPercept),
            (TemporalExpectation, RhythmicPercept),
            // Timbre percepts
            (SpectralCentroid, TimbrePercept),
            (AttackTime, TimbrePercept),
            (SpectralFlux, TimbrePercept),
            (InstrumentIdentification, TimbrePercept),
            // Affective responses
            (MusicalExpectation, AffectiveResponse),
            (Surprise, AffectiveResponse),
            (Tension, AffectiveResponse),
            (Resolution, AffectiveResponse),
            (MusicalEmotion, AffectiveResponse),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Causal events in music perception processing.
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

/// Causal graph for music perception processing pipeline.
pub struct MusicCausalGraph;

impl CausalDef for MusicCausalGraph {
    type Entity = MusicCausalEvent;

    fn relations() -> Vec<(MusicCausalEvent, MusicCausalEvent)> {
        use MusicCausalEvent::*;
        vec![
            // Auditory input drives pitch extraction and onset detection
            (AuditoryInput, PitchExtraction),
            (AuditoryInput, OnsetDetection),
            // Pitch extraction feeds harmonic grouping and melodic tracking
            (PitchExtraction, HarmonicGrouping),
            (PitchExtraction, MelodicTracking),
            // Onset detection drives beat induction
            (OnsetDetection, BeatInduction),
            // Beat induction establishes metric framing
            (BeatInduction, MetricFraming),
            // Harmonic grouping yields tonal interpretation
            (HarmonicGrouping, TonalInterpretation),
            // Melodic tracking forms musical expectations
            (MelodicTracking, MusicalExpectationFormation),
            // Metric framing enables groove perception
            (MetricFraming, GroovePerception),
            // Tonal interpretation and expectation both produce emotional response
            (TonalInterpretation, EmotionalResponse),
            (MusicalExpectationFormation, EmotionalResponse),
        ]
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

/// Opposition pairs in music perception.
///
/// - Consonance ↔ Dissonance (Helmholtz 1863; Plomp & Levelt 1965)
/// - Tension ↔ Resolution (Lerdahl & Jackendoff 1983)
/// - AbsolutePitch ↔ RelativePitch (Levitin & Rogers 2005)
pub struct MusicOpposition;

impl OppositionDef for MusicOpposition {
    type Entity = MusicEntity;

    fn pairs() -> Vec<(MusicEntity, MusicEntity)> {
        use MusicEntity::*;
        vec![
            (Consonance, Dissonance),
            (Tension, Resolution),
            (AbsolutePitch, RelativePitch),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over music perception entities.
    pub MusicPerceptionCategory {
        entity: MusicEntity,
        relation: MusicRelation,
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Frequency ratio for consonant intervals.
///
/// Simple integer ratios = more consonant (Helmholtz 1863).
/// - Octave: 2:1
/// - Perfect fifth: 3:2
/// - Perfect fourth: 4:3
/// - Major third: 5:4
#[derive(Debug, Clone)]
pub struct ConsonanceRanking;

impl Quality for ConsonanceRanking {
    type Individual = MusicEntity;
    type Value = u32;

    fn get(&self, individual: &MusicEntity) -> Option<u32> {
        use MusicEntity::*;
        match individual {
            Consonance => Some(1),        // most consonant
            Dissonance => Some(10),       // most dissonant
            OctaveEquivalence => Some(1), // octave = most consonant interval
            _ => None,
        }
    }
}

/// Preferred tempo range (BPM).
///
/// Spontaneous motor tempo ~120 BPM (Fraisse 1982).
/// Preferred range: 80-160 BPM (van Noorden & Moelants 1999).
#[derive(Debug, Clone)]
pub struct PreferredTempoBPM;

impl Quality for PreferredTempoBPM {
    type Individual = MusicEntity;
    type Value = f64;

    fn get(&self, individual: &MusicEntity) -> Option<f64> {
        use MusicEntity::*;
        match individual {
            Tempo => Some(120.0),       // spontaneous motor tempo
            Beat => Some(120.0),        // preferred beat rate
            Entrainment => Some(120.0), // natural entrainment rate
            _ => None,
        }
    }
}

/// Octave frequency ratio.
///
/// The octave is defined by a 2:1 frequency ratio, the most fundamental
/// interval in music perception. Octave equivalence means notes separated
/// by this ratio are perceived as having the same pitch chroma.
///
/// Helmholtz 1863, Ch. 10.
#[derive(Debug, Clone)]
pub struct OctaveRatio;

impl Quality for OctaveRatio {
    type Individual = MusicEntity;
    type Value = f64;

    fn get(&self, individual: &MusicEntity) -> Option<f64> {
        use MusicEntity::*;
        match individual {
            OctaveEquivalence => Some(2.0), // the 2:1 frequency ratio
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Octave ratio is exactly 2.0 (Helmholtz 1863).
pub struct OctaveRatioIsTwo;
impl Axiom for OctaveRatioIsTwo {
    fn description(&self) -> &str {
        "octave equivalence has a 2:1 frequency ratio"
    }
    fn holds(&self) -> bool {
        OctaveRatio.get(&MusicEntity::OctaveEquivalence) == Some(2.0)
    }
}

pub struct MusicTaxonomyIsDAG;
impl Axiom for MusicTaxonomyIsDAG {
    fn description(&self) -> &str {
        "music perception taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        taxonomy::NoCycles::<MusicTaxonomy>::new().holds()
    }
}

/// Consonance and dissonance are opposed.
///
/// Helmholtz 1863; Plomp & Levelt 1965.
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

/// Tension and resolution are opposed.
///
/// Lerdahl & Jackendoff 1983.
pub struct TensionOpposesResolution;
impl Axiom for TensionOpposesResolution {
    fn description(&self) -> &str {
        "tension and resolution are opposed"
    }
    fn holds(&self) -> bool {
        opposition::are_opposed::<MusicOpposition>(&MusicEntity::Tension, &MusicEntity::Resolution)
    }
}

pub struct MusicOppositionSymmetric;
impl Axiom for MusicOppositionSymmetric {
    fn description(&self) -> &str {
        "music opposition is symmetric"
    }
    fn holds(&self) -> bool {
        opposition::Symmetric::<MusicOpposition>::new().holds()
    }
}

pub struct MusicOppositionIrreflexive;
impl Axiom for MusicOppositionIrreflexive {
    fn description(&self) -> &str {
        "music opposition is irreflexive"
    }
    fn holds(&self) -> bool {
        opposition::Irreflexive::<MusicOpposition>::new().holds()
    }
}

/// Consonance is more consonant than dissonance (ranking order).
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

/// Five perceptual categories are represented.
pub struct FivePerceptualCategories;
impl Axiom for FivePerceptualCategories {
    fn description(&self) -> &str {
        "pitch, harmonic, rhythmic, timbre, and affective categories exist"
    }
    fn holds(&self) -> bool {
        use MusicEntity::*;
        // At least one entity in each category
        taxonomy::is_a::<MusicTaxonomy>(&PitchHeight, &PitchPercept)
            && taxonomy::is_a::<MusicTaxonomy>(&Consonance, &HarmonicPercept)
            && taxonomy::is_a::<MusicTaxonomy>(&Beat, &RhythmicPercept)
            && taxonomy::is_a::<MusicTaxonomy>(&SpectralCentroid, &TimbrePercept)
            && taxonomy::is_a::<MusicTaxonomy>(&MusicalEmotion, &AffectiveResponse)
    }
}

/// Causal graph is asymmetric.
pub struct MusicCausalGraphIsAsymmetric;
impl Axiom for MusicCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "music perception causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<MusicCausalGraph>::new().holds()
    }
}

/// No event causes itself.
pub struct MusicCausalGraphNoSelfCause;
impl Axiom for MusicCausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no music perception event causes itself"
    }
    fn holds(&self) -> bool {
        causation::NoSelfCausation::<MusicCausalGraph>::new().holds()
    }
}

/// Auditory input transitively causes emotional response.
pub struct InputCausesEmotion;
impl Axiom for InputCausesEmotion {
    fn description(&self) -> &str {
        "auditory input transitively causes emotional response"
    }
    fn holds(&self) -> bool {
        use MusicCausalEvent::*;
        let effects = causation::effects_of::<MusicCausalGraph>(&AuditoryInput);
        effects.contains(&EmotionalResponse)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

pub struct MusicPerceptionOntology;

impl Ontology for MusicPerceptionOntology {
    type Cat = MusicPerceptionCategory;
    type Qual = PreferredTempoBPM;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(MusicTaxonomyIsDAG),
            Box::new(ConsonanceOpposesDissonance),
            Box::new(TensionOpposesResolution),
            Box::new(MusicOppositionSymmetric),
            Box::new(MusicOppositionIrreflexive),
            Box::new(ConsonanceRankedHigher),
            Box::new(FivePerceptualCategories),
            Box::new(MusicCausalGraphIsAsymmetric),
            Box::new(MusicCausalGraphNoSelfCause),
            Box::new(OctaveRatioIsTwo),
            Box::new(InputCausesEmotion),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Category;
    use pr4xis::category::validate::check_category_laws;
    use pr4xis::ontology::reasoning::causation::CausalCategory;
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyCategory;
    use proptest::prelude::*;

    #[test]
    fn test_taxonomy_dag() {
        assert!(MusicTaxonomyIsDAG.holds());
    }
    #[test]
    fn test_consonance_dissonance() {
        assert!(ConsonanceOpposesDissonance.holds());
    }
    #[test]
    fn test_tension_resolution() {
        assert!(TensionOpposesResolution.holds());
    }
    #[test]
    fn test_opposition_symmetric() {
        assert!(MusicOppositionSymmetric.holds());
    }
    #[test]
    fn test_opposition_irreflexive() {
        assert!(MusicOppositionIrreflexive.holds());
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
    fn test_causal_graph_asymmetric() {
        assert!(MusicCausalGraphIsAsymmetric.holds());
    }
    #[test]
    fn test_causal_graph_no_self_cause() {
        assert!(MusicCausalGraphNoSelfCause.holds());
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
    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<MusicTaxonomy>(&entity, &entity));
        }
    }
}
