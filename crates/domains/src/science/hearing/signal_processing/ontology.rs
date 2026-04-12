//! Signal processing ontology.
//!
//! Models computational analysis of sound signals.
//!
//! Key references:
//! - Oppenheim & Schafer 2010: Discrete-Time Signal Processing
//! - Shannon 1949: sampling theorem
//! - Cooley & Tukey 1965: FFT algorithm
//! - Harris 1978: window functions
//! - Welch 1967: spectral estimation

use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::reasoning::causation::{self, CausalDef};
use pr4xis::ontology::reasoning::mereology::{self, MereologyDef};
use pr4xis::ontology::reasoning::opposition::{self, OppositionDef};
use pr4xis::ontology::reasoning::taxonomy::{self, TaxonomyDef};
use pr4xis::ontology::{Axiom, Ontology, Quality};

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SignalEntity {
    // Transforms
    FourierTransform,
    FFT,
    InverseFFT,
    ShortTimeFourierTransform,
    WaveletTransform,
    HilbertTransform,
    CepstralAnalysis,
    // Representations
    Spectrogram,
    PowerSpectralDensity,
    Autocorrelation,
    Cepstrum,
    MelFrequencyCepstrum,
    // Filters
    LowPassFilter,
    HighPassFilter,
    BandPassFilter,
    BandStopFilter,
    FIRFilter,
    IIRFilter,
    GammatoneFilter,
    // Sampling
    Sampling,
    NyquistFrequency,
    Aliasing,
    Quantization,
    // Windowing
    WindowFunction,
    HannWindow,
    HammingWindow,
    BlackmanWindow,
    RectangularWindow,
    // Operations
    Convolution,
    Correlation,
    Decimation,
    Interpolation,
    // Domains
    TimeDomain,
    FrequencyDomain,
    // Abstract categories
    Transform,
    Representation,
    Filter,
    SamplingConcept,
    SignalOperation,
    AnalysisDomain,
}

impl Entity for SignalEntity {
    fn variants() -> Vec<Self> {
        use SignalEntity::*;
        vec![
            FourierTransform,
            FFT,
            InverseFFT,
            ShortTimeFourierTransform,
            WaveletTransform,
            HilbertTransform,
            CepstralAnalysis,
            Spectrogram,
            PowerSpectralDensity,
            Autocorrelation,
            Cepstrum,
            MelFrequencyCepstrum,
            LowPassFilter,
            HighPassFilter,
            BandPassFilter,
            BandStopFilter,
            FIRFilter,
            IIRFilter,
            GammatoneFilter,
            Sampling,
            NyquistFrequency,
            Aliasing,
            Quantization,
            WindowFunction,
            HannWindow,
            HammingWindow,
            BlackmanWindow,
            RectangularWindow,
            Convolution,
            Correlation,
            Decimation,
            Interpolation,
            TimeDomain,
            FrequencyDomain,
            Transform,
            Representation,
            Filter,
            SamplingConcept,
            SignalOperation,
            AnalysisDomain,
        ]
    }
}

// ---------------------------------------------------------------------------
// Taxonomy
// ---------------------------------------------------------------------------

pub struct SignalTaxonomy;

impl TaxonomyDef for SignalTaxonomy {
    type Entity = SignalEntity;

    fn relations() -> Vec<(SignalEntity, SignalEntity)> {
        use SignalEntity::*;
        vec![
            // Transforms
            (FourierTransform, Transform),
            (FFT, FourierTransform),
            (InverseFFT, Transform),
            (ShortTimeFourierTransform, Transform),
            (WaveletTransform, Transform),
            (HilbertTransform, Transform),
            (CepstralAnalysis, Transform),
            // Representations
            (Spectrogram, Representation),
            (PowerSpectralDensity, Representation),
            (Autocorrelation, Representation),
            (Cepstrum, Representation),
            (MelFrequencyCepstrum, Representation),
            // Filters
            (LowPassFilter, Filter),
            (HighPassFilter, Filter),
            (BandPassFilter, Filter),
            (BandStopFilter, Filter),
            (FIRFilter, Filter),
            (IIRFilter, Filter),
            (GammatoneFilter, Filter),
            (GammatoneFilter, BandPassFilter),
            // Sampling concepts
            (Sampling, SamplingConcept),
            (NyquistFrequency, SamplingConcept),
            (Aliasing, SamplingConcept),
            (Quantization, SamplingConcept),
            // Windows
            (HannWindow, WindowFunction),
            (HammingWindow, WindowFunction),
            (BlackmanWindow, WindowFunction),
            (RectangularWindow, WindowFunction),
            // Operations
            (Convolution, SignalOperation),
            (Correlation, SignalOperation),
            (Decimation, SignalOperation),
            (Interpolation, SignalOperation),
            // Domains
            (TimeDomain, AnalysisDomain),
            (FrequencyDomain, AnalysisDomain),
        ]
    }
}

// ---------------------------------------------------------------------------
// Mereology (has-a / part-whole)
// ---------------------------------------------------------------------------

/// Part-whole relationships for signal processing.
///
/// Computational representations are composed of their constituent components:
/// - Spectrogram is composed of window function, frequency domain, and time domain
/// - Mel-frequency cepstrum contains cepstral analysis and bandpass filter
///
/// Oppenheim & Schafer 2010; Harris 1978.
pub struct SignalMereology;

impl MereologyDef for SignalMereology {
    type Entity = SignalEntity;

    fn relations() -> Vec<(SignalEntity, SignalEntity)> {
        use SignalEntity::*;
        vec![
            // Spectrogram composition
            (Spectrogram, WindowFunction),
            (Spectrogram, FrequencyDomain),
            (Spectrogram, TimeDomain),
            // Mel-frequency cepstrum components
            (MelFrequencyCepstrum, CepstralAnalysis),
            (MelFrequencyCepstrum, BandPassFilter),
        ]
    }
}

// ---------------------------------------------------------------------------
// Causal graph
// ---------------------------------------------------------------------------

/// Causal events in the signal processing pipeline.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SignalCausalEvent {
    RawSignal,
    AntiAliasFiltering,
    Digitization,
    WindowApplication,
    SpectralTransform,
    SpectralSmoothing,
    FeatureExtraction,
    PatternClassification,
}

impl Entity for SignalCausalEvent {
    fn variants() -> Vec<Self> {
        use SignalCausalEvent::*;
        vec![
            RawSignal,
            AntiAliasFiltering,
            Digitization,
            WindowApplication,
            SpectralTransform,
            SpectralSmoothing,
            FeatureExtraction,
            PatternClassification,
        ]
    }
}

/// Causal graph for signal processing pipeline.
pub struct SignalCausalGraph;

impl CausalDef for SignalCausalGraph {
    type Entity = SignalCausalEvent;

    fn relations() -> Vec<(SignalCausalEvent, SignalCausalEvent)> {
        use SignalCausalEvent::*;
        vec![
            // Raw signal passes through anti-alias filter
            (RawSignal, AntiAliasFiltering),
            // Anti-alias filtered signal is digitized
            (AntiAliasFiltering, Digitization),
            // Digitized signal gets windowed
            (Digitization, WindowApplication),
            // Windowed signal undergoes spectral transform
            (WindowApplication, SpectralTransform),
            // Spectral data feeds feature extraction
            (SpectralTransform, FeatureExtraction),
            // Features drive pattern classification
            (FeatureExtraction, PatternClassification),
            // Spectral data can also be smoothed first
            (SpectralTransform, SpectralSmoothing),
            // Smoothed spectrum also feeds feature extraction
            (SpectralSmoothing, FeatureExtraction),
        ]
    }
}

// ---------------------------------------------------------------------------
// Opposition
// ---------------------------------------------------------------------------

pub struct SignalOpposition;

impl OppositionDef for SignalOpposition {
    type Entity = SignalEntity;

    fn pairs() -> Vec<(SignalEntity, SignalEntity)> {
        use SignalEntity::*;
        vec![
            (TimeDomain, FrequencyDomain),
            (LowPassFilter, HighPassFilter),
            (Decimation, Interpolation),
            (FFT, InverseFFT),
        ]
    }
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalRelation {
    pub from: SignalEntity,
    pub to: SignalEntity,
}

impl Relationship for SignalRelation {
    type Object = SignalEntity;
    fn source(&self) -> SignalEntity {
        self.from
    }
    fn target(&self) -> SignalEntity {
        self.to
    }
}

pub struct SignalProcessingCategory;

impl Category for SignalProcessingCategory {
    type Object = SignalEntity;
    type Morphism = SignalRelation;

    fn identity(obj: &SignalEntity) -> SignalRelation {
        SignalRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &SignalRelation, g: &SignalRelation) -> Option<SignalRelation> {
        if f.to != g.from {
            return None;
        }
        Some(SignalRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<SignalRelation> {
        let v = SignalEntity::variants();
        v.iter()
            .flat_map(|&a| v.iter().map(move |&b| SignalRelation { from: a, to: b }))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Computational complexity (Big-O notation exponent for N-point operation).
///
/// - FFT: O(N log N), Cooley & Tukey 1965
/// - DFT: O(N^2)
/// - Convolution (direct): O(N^2)
/// - Convolution (FFT): O(N log N)
#[derive(Debug, Clone, PartialEq)]
pub struct Complexity {
    pub order: &'static str,
}

#[derive(Debug, Clone)]
pub struct ComputationalComplexity;

impl Quality for ComputationalComplexity {
    type Individual = SignalEntity;
    type Value = Complexity;

    fn get(&self, individual: &SignalEntity) -> Option<Complexity> {
        use SignalEntity::*;
        match individual {
            FFT | InverseFFT => Some(Complexity { order: "N log N" }),
            FourierTransform => Some(Complexity { order: "N^2" }),
            Convolution => Some(Complexity { order: "N^2" }),
            Correlation => Some(Complexity { order: "N^2" }),
            _ => None,
        }
    }
}

/// Sidelobe level (dB) for window functions.
///
/// Harris 1978.
#[derive(Debug, Clone)]
pub struct SidelobeLevel;

impl Quality for SidelobeLevel {
    type Individual = SignalEntity;
    type Value = f64;

    fn get(&self, individual: &SignalEntity) -> Option<f64> {
        use SignalEntity::*;
        match individual {
            RectangularWindow => Some(-13.0), // worst sidelobes
            HannWindow => Some(-31.5),
            HammingWindow => Some(-42.0),
            BlackmanWindow => Some(-58.0), // best sidelobes
            _ => None,
        }
    }
}

/// Normalized mainlobe bandwidth (Hz*N product) for window functions.
///
/// Measures the width of the main spectral lobe; higher values mean
/// wider mainlobe (poorer frequency resolution) but better sidelobe
/// suppression.
///
/// - Rectangular: 1.0
/// - Hann: 2.0
/// - Blackman: 3.0
///
/// Harris 1978, Table I.
#[derive(Debug, Clone)]
pub struct MainlobeBandwidth;

impl Quality for MainlobeBandwidth {
    type Individual = SignalEntity;
    type Value = f64;

    fn get(&self, individual: &SignalEntity) -> Option<f64> {
        use SignalEntity::*;
        match individual {
            RectangularWindow => Some(1.0),
            HannWindow => Some(2.0),
            BlackmanWindow => Some(3.0),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Rectangular window has narrowest mainlobe (best frequency resolution).
///
/// Harris 1978.
pub struct RectangularNarrowestMainlobe;
impl Axiom for RectangularNarrowestMainlobe {
    fn description(&self) -> &str {
        "rectangular window has narrowest mainlobe bandwidth"
    }
    fn holds(&self) -> bool {
        use SignalEntity::*;
        let rect = MainlobeBandwidth.get(&RectangularWindow).unwrap();
        let hann = MainlobeBandwidth.get(&HannWindow).unwrap();
        let blackman = MainlobeBandwidth.get(&BlackmanWindow).unwrap();
        rect < hann && hann < blackman
    }
}

pub struct SignalTaxonomyIsDAG;
impl Axiom for SignalTaxonomyIsDAG {
    fn description(&self) -> &str {
        "signal processing taxonomy is a DAG"
    }
    fn holds(&self) -> bool {
        taxonomy::NoCycles::<SignalTaxonomy>::new().holds()
    }
}

/// Mereology is a DAG.
pub struct SignalMereologyIsDAG;
impl Axiom for SignalMereologyIsDAG {
    fn description(&self) -> &str {
        "signal processing mereology is a DAG"
    }
    fn holds(&self) -> bool {
        mereology::NoCycles::<SignalMereology>::new().holds()
    }
}

/// Spectrogram contains both time and frequency domain representations.
///
/// A spectrogram is a time-frequency representation that combines windowed
/// segments (time domain) with spectral analysis (frequency domain).
pub struct SpectrogramContainsDomains;
impl Axiom for SpectrogramContainsDomains {
    fn description(&self) -> &str {
        "spectrogram contains time and frequency domain components"
    }
    fn holds(&self) -> bool {
        use SignalEntity::*;
        let parts = mereology::parts_of::<SignalMereology>(&Spectrogram);
        parts.contains(&TimeDomain)
            && parts.contains(&FrequencyDomain)
            && parts.contains(&WindowFunction)
    }
}

/// FFT is-a FourierTransform is-a Transform (two-level subsumption).
pub struct FFTSubsumption;
impl Axiom for FFTSubsumption {
    fn description(&self) -> &str {
        "FFT is-a FourierTransform is-a Transform"
    }
    fn holds(&self) -> bool {
        use SignalEntity::*;
        taxonomy::is_a::<SignalTaxonomy>(&FFT, &FourierTransform)
            && taxonomy::is_a::<SignalTaxonomy>(&FourierTransform, &Transform)
            && taxonomy::is_a::<SignalTaxonomy>(&FFT, &Transform)
    }
}

/// Time and frequency domains are opposed.
pub struct DomainsOpposed;
impl Axiom for DomainsOpposed {
    fn description(&self) -> &str {
        "time and frequency domains are opposed"
    }
    fn holds(&self) -> bool {
        opposition::are_opposed::<SignalOpposition>(
            &SignalEntity::TimeDomain,
            &SignalEntity::FrequencyDomain,
        )
    }
}

/// Blackman window has best (most negative) sidelobes.
///
/// Harris 1978.
pub struct BlackmanBestSidelobes;
impl Axiom for BlackmanBestSidelobes {
    fn description(&self) -> &str {
        "Blackman window has lowest sidelobes"
    }
    fn holds(&self) -> bool {
        use SignalEntity::*;
        let s = SidelobeLevel;
        let blackman = s.get(&BlackmanWindow).unwrap();
        let hann = s.get(&HannWindow).unwrap();
        let rect = s.get(&RectangularWindow).unwrap();
        blackman < hann && hann < rect
    }
}

/// Gammatone filter is both a Filter and a BandPassFilter.
///
/// Patterson et al. 1992: gammatone as auditory filter model.
pub struct GammatoneIsBandpass;
impl Axiom for GammatoneIsBandpass {
    fn description(&self) -> &str {
        "gammatone filter is-a bandpass filter"
    }
    fn holds(&self) -> bool {
        taxonomy::is_a::<SignalTaxonomy>(
            &SignalEntity::GammatoneFilter,
            &SignalEntity::BandPassFilter,
        )
    }
}

pub struct SignalOppositionSymmetric;
impl Axiom for SignalOppositionSymmetric {
    fn description(&self) -> &str {
        "signal processing opposition is symmetric"
    }
    fn holds(&self) -> bool {
        opposition::Symmetric::<SignalOpposition>::new().holds()
    }
}

/// Causal graph is asymmetric.
pub struct SignalCausalGraphIsAsymmetric;
impl Axiom for SignalCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "signal processing causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<SignalCausalGraph>::new().holds()
    }
}

/// No event causes itself.
pub struct SignalCausalGraphNoSelfCause;
impl Axiom for SignalCausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no signal processing event causes itself"
    }
    fn holds(&self) -> bool {
        causation::NoSelfCausation::<SignalCausalGraph>::new().holds()
    }
}

/// Raw signal transitively causes pattern classification.
pub struct RawSignalCausesClassification;
impl Axiom for RawSignalCausesClassification {
    fn description(&self) -> &str {
        "raw signal transitively causes pattern classification"
    }
    fn holds(&self) -> bool {
        use SignalCausalEvent::*;
        let effects = causation::effects_of::<SignalCausalGraph>(&RawSignal);
        effects.contains(&PatternClassification)
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

pub struct SignalProcessingOntology;

impl Ontology for SignalProcessingOntology {
    type Cat = SignalProcessingCategory;
    type Qual = SidelobeLevel;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(SignalTaxonomyIsDAG),
            Box::new(SignalMereologyIsDAG),
            Box::new(SpectrogramContainsDomains),
            Box::new(FFTSubsumption),
            Box::new(DomainsOpposed),
            Box::new(BlackmanBestSidelobes),
            Box::new(GammatoneIsBandpass),
            Box::new(SignalOppositionSymmetric),
            Box::new(SignalCausalGraphIsAsymmetric),
            Box::new(SignalCausalGraphNoSelfCause),
            Box::new(RectangularNarrowestMainlobe),
            Box::new(RawSignalCausesClassification),
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
        assert!(SignalTaxonomyIsDAG.holds());
    }
    #[test]
    fn test_mereology_is_dag() {
        assert!(SignalMereologyIsDAG.holds());
    }
    #[test]
    fn test_spectrogram_contains_domains() {
        assert!(SpectrogramContainsDomains.holds());
    }
    #[test]
    fn test_fft_subsumption() {
        assert!(FFTSubsumption.holds());
    }
    #[test]
    fn test_domains_opposed() {
        assert!(DomainsOpposed.holds());
    }
    #[test]
    fn test_blackman_best() {
        assert!(BlackmanBestSidelobes.holds());
    }
    #[test]
    fn test_gammatone_bandpass() {
        assert!(GammatoneIsBandpass.holds());
    }
    #[test]
    fn test_opposition_symmetric() {
        assert!(SignalOppositionSymmetric.holds());
    }
    #[test]
    fn test_causal_graph_asymmetric() {
        assert!(SignalCausalGraphIsAsymmetric.holds());
    }
    #[test]
    fn test_causal_graph_no_self_cause() {
        assert!(SignalCausalGraphNoSelfCause.holds());
    }
    #[test]
    fn test_raw_signal_causes_classification() {
        assert!(RawSignalCausesClassification.holds());
    }

    #[test]
    fn test_category_laws() {
        check_category_laws::<SignalProcessingCategory>().unwrap();
    }
    #[test]
    fn test_taxonomy_category_laws() {
        check_category_laws::<TaxonomyCategory<SignalTaxonomy>>().unwrap();
    }
    #[test]
    fn test_causal_category_laws() {
        check_category_laws::<CausalCategory<SignalCausalGraph>>().unwrap();
    }
    #[test]
    fn test_mereology_category_laws() {
        check_category_laws::<MereologyCategory<SignalMereology>>().unwrap();
    }

    #[test]
    fn test_rectangular_narrowest_mainlobe() {
        assert!(RectangularNarrowestMainlobe.holds());
    }
    #[test]
    fn test_rectangular_mainlobe_bandwidth() {
        assert_eq!(
            MainlobeBandwidth.get(&SignalEntity::RectangularWindow),
            Some(1.0)
        );
    }
    #[test]
    fn test_blackman_mainlobe_bandwidth() {
        assert_eq!(
            MainlobeBandwidth.get(&SignalEntity::BlackmanWindow),
            Some(3.0)
        );
    }

    #[test]
    fn test_fft_complexity() {
        assert_eq!(
            ComputationalComplexity
                .get(&SignalEntity::FFT)
                .unwrap()
                .order,
            "N log N"
        );
    }

    #[test]
    fn test_entity_count() {
        assert_eq!(SignalEntity::variants().len(), 40);
    }
    #[test]
    fn test_ontology_validates() {
        SignalProcessingOntology::validate().unwrap();
    }

    fn arb_entity() -> impl Strategy<Value = SignalEntity> {
        (0..SignalEntity::variants().len()).prop_map(|i| SignalEntity::variants()[i])
    }
    proptest! {
        #[test]
        fn prop_taxonomy_reflexive(entity in arb_entity()) {
            prop_assert!(taxonomy::is_a::<SignalTaxonomy>(&entity, &entity));
        }
    }
}
