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

use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::reasoning::causation;
use pr4xis::ontology::reasoning::mereology;
use pr4xis::ontology::reasoning::opposition;
use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
pub enum SignalEntity {
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Entity)]
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

define_ontology! {
    /// Discrete category over signal processing entities.
    pub SignalProcessingOntology for SignalProcessingCategory {
        entity: SignalEntity,
        relation: SignalRelation,

        taxonomy: SignalTaxonomy [
            (FourierTransform, Transform), (FFT, FourierTransform), (InverseFFT, Transform),
            (ShortTimeFourierTransform, Transform), (WaveletTransform, Transform),
            (HilbertTransform, Transform), (CepstralAnalysis, Transform),
            (Spectrogram, Representation), (PowerSpectralDensity, Representation),
            (Autocorrelation, Representation), (Cepstrum, Representation),
            (MelFrequencyCepstrum, Representation),
            (LowPassFilter, Filter), (HighPassFilter, Filter), (BandPassFilter, Filter),
            (BandStopFilter, Filter), (FIRFilter, Filter), (IIRFilter, Filter),
            (GammatoneFilter, Filter), (GammatoneFilter, BandPassFilter),
            (Sampling, SamplingConcept), (NyquistFrequency, SamplingConcept),
            (Aliasing, SamplingConcept), (Quantization, SamplingConcept),
            (HannWindow, WindowFunction), (HammingWindow, WindowFunction),
            (BlackmanWindow, WindowFunction), (RectangularWindow, WindowFunction),
            (Convolution, SignalOperation), (Correlation, SignalOperation),
            (Decimation, SignalOperation), (Interpolation, SignalOperation),
            (TimeDomain, AnalysisDomain), (FrequencyDomain, AnalysisDomain),
        ],

        mereology: SignalMereology [
            (Spectrogram, WindowFunction), (Spectrogram, FrequencyDomain),
            (Spectrogram, TimeDomain),
            (MelFrequencyCepstrum, CepstralAnalysis), (MelFrequencyCepstrum, BandPassFilter),
        ],

        causation: SignalCausalGraph for SignalCausalEvent [
            (RawSignal, AntiAliasFiltering), (AntiAliasFiltering, Digitization),
            (Digitization, WindowApplication), (WindowApplication, SpectralTransform),
            (SpectralTransform, FeatureExtraction), (FeatureExtraction, PatternClassification),
            (SpectralTransform, SpectralSmoothing), (SpectralSmoothing, FeatureExtraction),
        ],

        opposition: SignalOpposition [
            (TimeDomain, FrequencyDomain), (LowPassFilter, HighPassFilter),
            (Decimation, Interpolation), (FFT, InverseFFT),
        ],
    }
}

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

#[derive(Debug, Clone)]
pub struct SidelobeLevel;
impl Quality for SidelobeLevel {
    type Individual = SignalEntity;
    type Value = f64;
    fn get(&self, individual: &SignalEntity) -> Option<f64> {
        use SignalEntity::*;
        match individual {
            RectangularWindow => Some(-13.0),
            HannWindow => Some(-31.5),
            HammingWindow => Some(-42.0),
            BlackmanWindow => Some(-58.0),
            _ => None,
        }
    }
}

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

pub struct RectangularNarrowestMainlobe;
impl Axiom for RectangularNarrowestMainlobe {
    fn description(&self) -> &str {
        "rectangular window has narrowest mainlobe bandwidth"
    }
    fn holds(&self) -> bool {
        use SignalEntity::*;
        let r = MainlobeBandwidth.get(&RectangularWindow).unwrap();
        let h = MainlobeBandwidth.get(&HannWindow).unwrap();
        let b = MainlobeBandwidth.get(&BlackmanWindow).unwrap();
        r < h && h < b
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
pub struct SignalMereologyIsDAG;
impl Axiom for SignalMereologyIsDAG {
    fn description(&self) -> &str {
        "signal processing mereology is a DAG"
    }
    fn holds(&self) -> bool {
        mereology::NoCycles::<SignalMereology>::new().holds()
    }
}
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
pub struct BlackmanBestSidelobes;
impl Axiom for BlackmanBestSidelobes {
    fn description(&self) -> &str {
        "Blackman window has lowest sidelobes"
    }
    fn holds(&self) -> bool {
        use SignalEntity::*;
        let s = SidelobeLevel;
        s.get(&BlackmanWindow).unwrap() < s.get(&HannWindow).unwrap()
            && s.get(&HannWindow).unwrap() < s.get(&RectangularWindow).unwrap()
    }
}
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
pub struct SignalCausalGraphIsAsymmetric;
impl Axiom for SignalCausalGraphIsAsymmetric {
    fn description(&self) -> &str {
        "signal processing causal graph is asymmetric"
    }
    fn holds(&self) -> bool {
        causation::Asymmetric::<SignalCausalGraph>::new().holds()
    }
}
pub struct SignalCausalGraphNoSelfCause;
impl Axiom for SignalCausalGraphNoSelfCause {
    fn description(&self) -> &str {
        "no signal processing event causes itself"
    }
    fn holds(&self) -> bool {
        causation::NoSelfCausation::<SignalCausalGraph>::new().holds()
    }
}
pub struct RawSignalCausesClassification;
impl Axiom for RawSignalCausesClassification {
    fn description(&self) -> &str {
        "raw signal transitively causes pattern classification"
    }
    fn holds(&self) -> bool {
        use SignalCausalEvent::*;
        causation::effects_of::<SignalCausalGraph>(&RawSignal).contains(&PatternClassification)
    }
}

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
