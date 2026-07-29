//! Music perception — how the auditory system perceives musical
//! structure (pitch, harmony, rhythm, timbre, expectation).
//!
//! # Literature
//!
//! - **Helmholtz (1863)** *Die Lehre von den Tonempfindungen* — On the
//!   Sensations of Tone, foundational consonance theory.
//! - **Plomp & Levelt (1965)** "Tonal consonance and critical bandwidth",
//!   *JASA* 38(4):548-560.
//! - **Lerdahl & Jackendoff (1983)** *A Generative Theory of Tonal Music*.
//! - **Krumhansl (1990)** *Cognitive Foundations of Musical Pitch*.
//! - **Huron (2006)** *Sweet Anticipation: Music and the Psychology of
//!   Expectation*.
//! - **Large & Palmer (2002)** "Perceiving temporal regularity in music",
//!   *Cog. Sci.* 26(1):1-37.
//! - **Patel (2008)** *Music, Language, and the Brain*.
//! - **McDermott & Oxenham (2008)** "Music perception, pitch, and the
//!   auditory system", *Curr. Opin. Neurobiol.* 18(4):452-463.

use crate::formal::math::quantity::unit::{BEAT_PER_MINUTE, UNITLESS};
use crate::formal::math::quantity::value::Quantity;
use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

pr4xis::ontology! {
    name: "Music",
    source: "Helmholtz (1863) On the Sensations of Tone; Plomp & Levelt (1965) JASA 38(4):548; Krumhansl (1990) Cognitive Foundations of Musical Pitch; Huron (2006) Sweet Anticipation; Lerdahl & Jackendoff (1983) Generative Theory of Tonal Music; Large & Palmer (2002) Cog. Sci. 26(1):1; Patel (2008) Music Language and the Brain; McDermott & Oxenham (2008) Curr. Opin. Neurobiol. 18(4):452",

    concepts: [
        // Pitch percepts
        PitchHeight, PitchChroma, OctaveEquivalence, AbsolutePitch,
        RelativePitch, MelodicContour, IntervalPerception,
        // Harmonic percepts
        Consonance, Dissonance, RoughnessModel, HarmonicSeries,
        VirtualPitchPercept, MissingFundamental, Chord, Tonality, KeySense,
        // Rhythmic
        Beat, Meter, Tempo, Syncopation, Groove, Entrainment, TemporalExpectation,
        // Timbre
        SpectralCentroid, AttackTime, SpectralFlux, InstrumentIdentification,
        // Affective / expectation
        MusicalExpectation, Surprise, Tension, Resolution, MusicalEmotion,
        // Memory
        EarWorm, MusicalMemory, TonalSchemaMemory,
        // Umbrellas
        PitchPercept, HarmonicPercept, RhythmicPercept, TimbrePercept, AffectiveResponse,
        // Events
        AuditoryInput, PitchExtraction, OnsetDetection, HarmonicGrouping,
        MelodicTracking, BeatInduction, MetricFraming, TonalInterpretation,
        MusicalExpectationFormation, GroovePerception, EmotionalResponse,
        MusicEvent,
    ],

    labels: {
        PitchHeight: ("en", "Pitch height",
            "Shepard (1982) Psychol. Rev. 89(4):305 — log-frequency dimension of pitch."),
        PitchChroma: ("en", "Pitch chroma",
            "Shepard (1982): circular pitch-class dimension."),
        OctaveEquivalence: ("en", "Octave equivalence",
            "Helmholtz (1863): perceptual identity of 2:1 frequency ratio."),
        AbsolutePitch: ("en", "Absolute pitch",
            "Krumhansl (1990) Ch. 7: identification of pitch class without reference."),
        RelativePitch: ("en", "Relative pitch",
            "Krumhansl (1990) Ch. 7: identification of interval relations."),
        MelodicContour: ("en", "Melodic contour",
            "Dowling (1978) Psychol. Rev. 85(4):341 — up/down pattern of pitch changes."),
        IntervalPerception: ("en", "Interval perception",
            "Krumhansl (1990): perception of frequency ratios between successive pitches."),
        Consonance: ("en", "Consonance",
            "Plomp & Levelt (1965) JASA 38(4):548 — perceived smoothness from non-overlapping critical bands."),
        Dissonance: ("en", "Dissonance",
            "Plomp & Levelt (1965): perceived roughness from overlapping critical bands."),
        RoughnessModel: ("en", "Roughness model",
            "Plomp & Levelt (1965): quantitative dissonance model from beating partials."),
        HarmonicSeries: ("en", "Harmonic series",
            "Helmholtz (1863): integer multiples of a fundamental frequency."),
        VirtualPitchPercept: ("en", "Virtual pitch",
            "Terhardt (1974) JASA 55(5):1061 — perceived pitch from harmonic pattern, even without f0."),
        MissingFundamental: ("en", "Missing fundamental",
            "Terhardt (1974): virtual pitch heard when fundamental is absent."),
        Chord: ("en", "Chord",
            "Lerdahl & Jackendoff (1983): simultaneous combination of three or more pitches."),
        Tonality: ("en", "Tonality",
            "Krumhansl (1990): hierarchical organization around a tonic."),
        KeySense: ("en", "Key sense",
            "Krumhansl (1990): perception of the prevailing tonal centre."),
        Beat: ("en", "Beat",
            "Large & Palmer (2002): perceived periodic pulse in music."),
        Meter: ("en", "Meter",
            "Lerdahl & Jackendoff (1983): hierarchical alternation of strong and weak beats."),
        Tempo: ("en", "Tempo",
            "Lerdahl & Jackendoff (1983): beat rate (e.g. BPM)."),
        Syncopation: ("en", "Syncopation",
            "Lerdahl & Jackendoff (1983): displacement of accent from metric strong positions."),
        Groove: ("en", "Groove",
            "Madison (2006) J. Exp. Psychol. Hum. Percept. 32(1):201 — subjective sensation of wanting to move."),
        Entrainment: ("en", "Entrainment",
            "Large & Palmer (2002): synchronization of motor / neural oscillators to a beat."),
        TemporalExpectation: ("en", "Temporal expectation",
            "Huron (2006): predicted onset of upcoming musical events."),
        SpectralCentroid: ("en", "Spectral centroid",
            "Grey (1977) JASA 61(5):1270 — perceived brightness via spectral first moment."),
        AttackTime: ("en", "Attack time",
            "Grey (1977): temporal envelope onset characteristic."),
        SpectralFlux: ("en", "Spectral flux",
            "Grey (1977): rate of spectral change over time."),
        InstrumentIdentification: ("en", "Instrument identification",
            "Grey (1977): categorical recognition of instrumental sound source."),
        MusicalExpectation: ("en", "Musical expectation",
            "Huron (2006): listener-generated prediction of upcoming events."),
        Surprise: ("en", "Surprise",
            "Huron (2006): expectation-violation response."),
        Tension: ("en", "Tension",
            "Lerdahl & Jackendoff (1983): perceived instability requiring resolution."),
        Resolution: ("en", "Resolution",
            "Lerdahl & Jackendoff (1983): perceived release of tension."),
        MusicalEmotion: ("en", "Musical emotion",
            "Juslin & Vastfjall (2008) BBS 31(5):559 — affect arising from music."),
        EarWorm: ("en", "Earworm",
            "Beaman & Williams (2010) Br. J. Psychol. 101(4):637 — involuntarily recurring musical imagery."),
        MusicalMemory: ("en", "Musical memory",
            "Krumhansl (1990): long-term store of musical material."),
        TonalSchemaMemory: ("en", "Tonal schema memory",
            "Krumhansl (1990) Ch. 4: internalised tonal hierarchy."),
        PitchPercept: ("en", "Pitch percept",
            "Krumhansl (1990): umbrella for pitch-related percepts."),
        HarmonicPercept: ("en", "Harmonic percept",
            "Helmholtz (1863): umbrella for harmony-related percepts."),
        RhythmicPercept: ("en", "Rhythmic percept",
            "Large & Palmer (2002): umbrella for rhythm-related percepts."),
        TimbrePercept: ("en", "Timbre percept",
            "Grey (1977): umbrella for timbre-related percepts."),
        AffectiveResponse: ("en", "Affective response",
            "Juslin & Vastfjall (2008): umbrella for emotional / expectation responses."),
        AuditoryInput: ("en", "Auditory input",
            "McDermott & Oxenham (2008): event of acoustic signal arriving at the ear."),
        PitchExtraction: ("en", "Pitch extraction",
            "McDermott & Oxenham (2008): event of pitch percept formation."),
        OnsetDetection: ("en", "Onset detection",
            "Large & Palmer (2002): event of acoustic onset detection."),
        HarmonicGrouping: ("en", "Harmonic grouping",
            "Helmholtz (1863): event of partial-grouping into a pitch percept."),
        MelodicTracking: ("en", "Melodic tracking",
            "Dowling (1978): event of melody-contour following."),
        BeatInduction: ("en", "Beat induction",
            "Large & Palmer (2002): event of beat-percept formation."),
        MetricFraming: ("en", "Metric framing",
            "Lerdahl & Jackendoff (1983): event of meter-percept formation."),
        TonalInterpretation: ("en", "Tonal interpretation",
            "Krumhansl (1990): event of key-and-chord interpretation."),
        MusicalExpectationFormation: ("en", "Musical expectation formation",
            "Huron (2006): event of expectation generation."),
        GroovePerception: ("en", "Groove perception",
            "Madison (2006): event of groove-sensation onset."),
        EmotionalResponse: ("en", "Emotional response",
            "Juslin & Vastfjall (2008): terminal event — emotion arising from music."),
        MusicEvent: ("en", "Music event",
            "Huron (2006): umbrella concept for music-perception perdurants."),
    },

    is_a: [
        (PitchHeight, PitchPercept), (PitchChroma, PitchPercept),
        (OctaveEquivalence, PitchPercept), (AbsolutePitch, PitchPercept),
        (RelativePitch, PitchPercept), (MelodicContour, PitchPercept),
        (IntervalPerception, PitchPercept),
        (Consonance, HarmonicPercept), (Dissonance, HarmonicPercept),
        (RoughnessModel, HarmonicPercept), (HarmonicSeries, HarmonicPercept),
        (VirtualPitchPercept, HarmonicPercept), (MissingFundamental, HarmonicPercept),
        (Chord, HarmonicPercept), (Tonality, HarmonicPercept), (KeySense, HarmonicPercept),
        (Beat, RhythmicPercept), (Meter, RhythmicPercept), (Tempo, RhythmicPercept),
        (Syncopation, RhythmicPercept), (Groove, RhythmicPercept),
        (Entrainment, RhythmicPercept), (TemporalExpectation, RhythmicPercept),
        (SpectralCentroid, TimbrePercept), (AttackTime, TimbrePercept),
        (SpectralFlux, TimbrePercept), (InstrumentIdentification, TimbrePercept),
        (MusicalExpectation, AffectiveResponse), (Surprise, AffectiveResponse),
        (Tension, AffectiveResponse), (Resolution, AffectiveResponse),
        (MusicalEmotion, AffectiveResponse),
        (AuditoryInput, MusicEvent), (PitchExtraction, MusicEvent),
        (OnsetDetection, MusicEvent), (HarmonicGrouping, MusicEvent),
        (MelodicTracking, MusicEvent), (BeatInduction, MusicEvent),
        (MetricFraming, MusicEvent), (TonalInterpretation, MusicEvent),
        (MusicalExpectationFormation, MusicEvent), (GroovePerception, MusicEvent),
        (EmotionalResponse, MusicEvent),
    ],

    causes: [
        (AuditoryInput, PitchExtraction),
        (AuditoryInput, OnsetDetection),
        (PitchExtraction, HarmonicGrouping),
        (PitchExtraction, MelodicTracking),
        (OnsetDetection, BeatInduction),
        (BeatInduction, MetricFraming),
        (HarmonicGrouping, TonalInterpretation),
        (MelodicTracking, MusicalExpectationFormation),
        (MetricFraming, GroovePerception),
        (TonalInterpretation, EmotionalResponse),
        (MusicalExpectationFormation, EmotionalResponse),
    ],

    opposes: [
        (Consonance, Dissonance), (Dissonance, Consonance),
        (Tension, Resolution), (Resolution, Tension),
        (AbsolutePitch, RelativePitch), (RelativePitch, AbsolutePitch),
    ],
}

/// Quality: consonance ranking (lower = more consonant). A unitless
/// perceptual ordinal, not a physical quantity.
#[derive(Debug, Clone)]
pub struct ConsonanceRanking;
impl Quality for ConsonanceRanking {
    type Individual = MusicConcept;
    type Value = Quantity;
    fn get(&self, individual: &MusicConcept) -> Option<Quantity> {
        use MusicConcept::*;
        match individual {
            Consonance => Some(Quantity::from_unit(1.0, &UNITLESS)),
            Dissonance => Some(Quantity::from_unit(10.0, &UNITLESS)),
            OctaveEquivalence => Some(Quantity::from_unit(1.0, &UNITLESS)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreferredTempoBPM;
impl Quality for PreferredTempoBPM {
    type Individual = MusicConcept;
    type Value = Quantity;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &MusicConcept) -> Option<Quantity> {
        use MusicConcept::*;
        match individual {
            Tempo => Some(Quantity::from_unit(120.0, &BEAT_PER_MINUTE)),
            Beat => Some(Quantity::from_unit(120.0, &BEAT_PER_MINUTE)),
            Entrainment => Some(Quantity::from_unit(120.0, &BEAT_PER_MINUTE)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OctaveRatio;
impl Quality for OctaveRatio {
    type Individual = MusicConcept;
    type Value = Quantity;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &MusicConcept) -> Option<Quantity> {
        match individual {
            // Dimensionless 2:1 frequency ratio (a pure number).
            MusicConcept::OctaveEquivalence => Some(Quantity::from_unit(2.0, &UNITLESS)),
            _ => None,
        }
    }
}

fn effects_of(cause: MusicConcept) -> Vec<MusicConcept> {
    use pr4xis::category::{Arrow, Category};
    MusicCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == MusicRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

pub struct OctaveRatioIsTwo;
impl Axiom for OctaveRatioIsTwo {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if OctaveRatio
            .get(&MusicConcept::OctaveEquivalence)
            .map(|q| q.value)
            == Some(2.0)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "OctaveRatioIsTwo",
        "octave equivalence has a 2:1 frequency ratio",
        "Helmholtz (1863) On the Sensations of Tone"
    );
}
pr4xis::register_axiom!(
    OctaveRatioIsTwo,
    "Helmholtz (1863) On the Sensations of Tone"
);

pub struct ConsonanceRankedHigher;
impl Axiom for ConsonanceRankedHigher {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use MusicConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let c = ConsonanceRanking
            .get(&Consonance)
            .unwrap_or(Quantity::from_unit(f64::MAX, &UNITLESS));
        let d = ConsonanceRanking
            .get(&Dissonance)
            .unwrap_or(Quantity::from_unit(0.0, &UNITLESS));
        if c < d {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ConsonanceRankedHigher",
        "consonance ranks higher (lower number) than dissonance",
        "Plomp & Levelt (1965) JASA 38(4):548"
    );
}
pr4xis::register_axiom!(
    ConsonanceRankedHigher,
    "Plomp & Levelt (1965) JASA 38(4):548"
);

pub struct InputCausesEmotion;
impl Axiom for InputCausesEmotion {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use MusicConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(AuditoryInput).contains(&EmotionalResponse) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "InputCausesEmotion",
        "auditory input transitively causes emotional response",
        "Juslin & Vastfjall (2008) BBS 31(5):559"
    );
}
pr4xis::register_axiom!(
    InputCausesEmotion,
    "Juslin & Vastfjall (2008) BBS 31(5):559"
);

impl Ontology for MusicOntology {
    type Cat = MusicCategory;
    type Qual = PreferredTempoBPM;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(OctaveRatioIsTwo));
        a.push(Box::new(ConsonanceRankedHigher));
        a.push(Box::new(InputCausesEmotion));
        a
    }
}

// Back-compat aliases.
pub use MusicCategory as MusicPerceptionCategory;
pub use MusicConcept as MusicEntity;
pub use MusicOntology as MusicPerceptionOntology;
pub use MusicRelationKind as MusicPerceptionCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<MusicCategory>();
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        MusicOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn octave_ratio_is_two() {
        assert!(OctaveRatioIsTwo.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn consonance_ranked_higher() {
        assert!(ConsonanceRankedHigher.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn input_causes_emotion() {
        assert!(InputCausesEmotion.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in MusicCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in MusicOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
