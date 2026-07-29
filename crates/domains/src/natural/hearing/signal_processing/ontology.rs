//! Signal processing — computational analysis of sound signals.
//!
//! # Literature
//!
//! - **Oppenheim & Schafer (2010)** *Discrete-Time Signal Processing*
//!   (3rd ed.), Prentice Hall.
//! - **Shannon (1949)** "Communication in the Presence of Noise",
//!   *Proc. IRE* 37(1):10-21 — sampling theorem.
//! - **Cooley & Tukey (1965)** "An Algorithm for the Machine Calculation
//!   of Complex Fourier Series", *Math. Comput.* 19(90):297-301 — FFT.
//! - **Harris (1978)** "On the Use of Windows for Harmonic Analysis with
//!   the Discrete Fourier Transform", *Proc. IEEE* 66(1):51-83.
//! - **Welch (1967)** "The Use of Fast Fourier Transform for the
//!   Estimation of Power Spectra", *IEEE Trans. Audio Electroacoust.*
//!   15(2):70-73.

use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

use crate::formal::analytical_methods::ontology::ComplexityClass;
use crate::formal::math::quantity::level::{
    LogarithmicLevel, LogarithmicLevelReferenceConcept as Ref,
};
use crate::formal::math::quantity::unit::UNITLESS;
use crate::formal::math::quantity::value::Quantity;

pr4xis::ontology! {
    name: "Signal",
    source: "Oppenheim & Schafer (2010) Discrete-Time Signal Processing 3rd ed.; Shannon (1949) Proc. IRE 37(1):10; Cooley & Tukey (1965) Math. Comput. 19(90):297; Harris (1978) Proc. IEEE 66(1):51; Welch (1967) IEEE Trans. Audio Electroacoust. 15(2):70",

    concepts: [
        FourierTransform, FFT, InverseFFT,
        ShortTimeFourierTransform, WaveletTransform, HilbertTransform, CepstralAnalysis,
        Spectrogram, PowerSpectralDensity, Autocorrelation, Cepstrum, MelFrequencyCepstrum,
        LowPassFilter, HighPassFilter, BandPassFilter, BandStopFilter,
        FIRFilter, IIRFilter, GammatoneFilter,
        Sampling, NyquistFrequency, Aliasing, Quantization,
        WindowFunction, HannWindow, HammingWindow, BlackmanWindow, RectangularWindow,
        Convolution, Correlation, Decimation, Interpolation,
        TimeDomain, FrequencyDomain,
        // Umbrellas
        Transform, Representation, Filter, SamplingConcept, SignalOperation, AnalysisDomain,
        // Events
        RawSignal, AntiAliasFiltering, Digitization, WindowApplication,
        SpectralTransform, SpectralSmoothing, FeatureExtraction, PatternClassification,
        SignalEvent,
    ],

    labels: {
        FourierTransform: ("en", "Fourier transform",
            "Oppenheim & Schafer (2010) §8: time→frequency integral transform."),
        FFT: ("en", "FFT",
            "Cooley & Tukey (1965) Math. Comput. 19(90):297 — O(N log N) DFT algorithm."),
        InverseFFT: ("en", "Inverse FFT",
            "Cooley & Tukey (1965): frequency→time inverse DFT."),
        ShortTimeFourierTransform: ("en", "STFT",
            "Oppenheim & Schafer (2010) §10: windowed Fourier transform."),
        WaveletTransform: ("en", "Wavelet transform",
            "Daubechies (1992): multi-resolution time-frequency transform."),
        HilbertTransform: ("en", "Hilbert transform",
            "Oppenheim & Schafer (2010): analytic-signal transform."),
        CepstralAnalysis: ("en", "Cepstral analysis",
            "Bogert et al. (1963): IFT(log|FT|) — separating excitation from filter."),
        Spectrogram: ("en", "Spectrogram",
            "Oppenheim & Schafer (2010) §10: time-frequency magnitude representation."),
        PowerSpectralDensity: ("en", "Power spectral density",
            "Welch (1967) IEEE TAE 15(2):70 — power per unit frequency."),
        Autocorrelation: ("en", "Autocorrelation",
            "Oppenheim & Schafer (2010): signal-with-shifted-self correlation."),
        Cepstrum: ("en", "Cepstrum",
            "Bogert et al. (1963): output of cepstral analysis."),
        MelFrequencyCepstrum: ("en", "MFCC",
            "Davis & Mermelstein (1980) IEEE TASSP 28(4):357 — mel-scale cepstrum."),
        LowPassFilter: ("en", "Low-pass filter",
            "Oppenheim & Schafer (2010) §7: filter passing low frequencies."),
        HighPassFilter: ("en", "High-pass filter",
            "Oppenheim & Schafer (2010) §7: filter passing high frequencies."),
        BandPassFilter: ("en", "Band-pass filter",
            "Oppenheim & Schafer (2010) §7: filter passing a frequency band."),
        BandStopFilter: ("en", "Band-stop filter",
            "Oppenheim & Schafer (2010) §7: filter rejecting a frequency band."),
        FIRFilter: ("en", "FIR filter",
            "Oppenheim & Schafer (2010) §7: finite-impulse-response filter."),
        IIRFilter: ("en", "IIR filter",
            "Oppenheim & Schafer (2010) §7: infinite-impulse-response filter."),
        GammatoneFilter: ("en", "Gammatone filter",
            "Patterson et al. (1992) JASA 91(4):2310 — cochlear-modeling bandpass filter."),
        Sampling: ("en", "Sampling",
            "Shannon (1949) Proc. IRE 37(1):10 — discrete-time signal acquisition."),
        NyquistFrequency: ("en", "Nyquist frequency",
            "Shannon (1949): fs/2 — upper bound for aliasing-free sampling."),
        Aliasing: ("en", "Aliasing",
            "Shannon (1949): spectral folding from undersampling."),
        Quantization: ("en", "Quantization",
            "Oppenheim & Schafer (2010) §4: amplitude discretization."),
        WindowFunction: ("en", "Window function",
            "Harris (1978) Proc. IEEE 66(1):51 — finite-support analysis weighting."),
        HannWindow: ("en", "Hann window",
            "Harris (1978): cosine-squared window."),
        HammingWindow: ("en", "Hamming window",
            "Harris (1978): cosine-on-pedestal window."),
        BlackmanWindow: ("en", "Blackman window",
            "Harris (1978): three-term cosine window — low sidelobes."),
        RectangularWindow: ("en", "Rectangular window",
            "Harris (1978): unweighted window — narrowest mainlobe, highest sidelobes."),
        Convolution: ("en", "Convolution",
            "Oppenheim & Schafer (2010) §2: integral/sum of one signal weighted by reversed other."),
        Correlation: ("en", "Correlation",
            "Oppenheim & Schafer (2010) §2: convolution with non-reversed signal."),
        Decimation: ("en", "Decimation",
            "Oppenheim & Schafer (2010) §4: sample-rate reduction."),
        Interpolation: ("en", "Interpolation",
            "Oppenheim & Schafer (2010) §4: sample-rate increase."),
        TimeDomain: ("en", "Time domain",
            "Oppenheim & Schafer (2010): signal as function of time."),
        FrequencyDomain: ("en", "Frequency domain",
            "Oppenheim & Schafer (2010): signal as function of frequency."),
        Transform: ("en", "Transform",
            "Oppenheim & Schafer (2010): umbrella for invertible signal mappings."),
        Representation: ("en", "Representation",
            "Oppenheim & Schafer (2010): umbrella for spectral representations."),
        Filter: ("en", "Filter",
            "Oppenheim & Schafer (2010): umbrella for frequency-selective operators."),
        SamplingConcept: ("en", "Sampling concept",
            "Shannon (1949): umbrella for sampling-related concepts."),
        SignalOperation: ("en", "Signal operation",
            "Oppenheim & Schafer (2010): umbrella for signal operators."),
        AnalysisDomain: ("en", "Analysis domain",
            "Oppenheim & Schafer (2010): umbrella for time/frequency views."),
        RawSignal: ("en", "Raw signal",
            "Oppenheim & Schafer (2010): event — pre-processing analog signal."),
        AntiAliasFiltering: ("en", "Anti-alias filtering",
            "Shannon (1949): event — pre-sampling low-pass."),
        Digitization: ("en", "Digitization",
            "Oppenheim & Schafer (2010) §4: event — sample + quantize."),
        WindowApplication: ("en", "Window application",
            "Harris (1978): event — windowing prior to spectral analysis."),
        SpectralTransform: ("en", "Spectral transform",
            "Cooley & Tukey (1965): event — FFT/STFT computation."),
        SpectralSmoothing: ("en", "Spectral smoothing",
            "Welch (1967): event — periodogram averaging."),
        FeatureExtraction: ("en", "Feature extraction",
            "Davis & Mermelstein (1980): event — descriptor computation."),
        PatternClassification: ("en", "Pattern classification",
            "Davis & Mermelstein (1980): terminal event — class assignment."),
        SignalEvent: ("en", "Signal event",
            "Oppenheim & Schafer (2010): umbrella for signal-processing perdurants."),
    },

    is_a: [
        (FourierTransform, Transform), (FFT, FourierTransform), (InverseFFT, Transform),
        (ShortTimeFourierTransform, Transform), (WaveletTransform, Transform),
        (HilbertTransform, Transform), (CepstralAnalysis, Transform),
        (Spectrogram, Representation), (PowerSpectralDensity, Representation),
        (Autocorrelation, Representation), (Cepstrum, Representation),
        (MelFrequencyCepstrum, Representation),
        (LowPassFilter, Filter), (HighPassFilter, Filter),
        (BandPassFilter, Filter), (BandStopFilter, Filter),
        (FIRFilter, Filter), (IIRFilter, Filter),
        (GammatoneFilter, Filter), (GammatoneFilter, BandPassFilter),
        (Sampling, SamplingConcept), (NyquistFrequency, SamplingConcept),
        (Aliasing, SamplingConcept), (Quantization, SamplingConcept),
        (HannWindow, WindowFunction), (HammingWindow, WindowFunction),
        (BlackmanWindow, WindowFunction), (RectangularWindow, WindowFunction),
        (Convolution, SignalOperation), (Correlation, SignalOperation),
        (Decimation, SignalOperation), (Interpolation, SignalOperation),
        (TimeDomain, AnalysisDomain), (FrequencyDomain, AnalysisDomain),
        (RawSignal, SignalEvent), (AntiAliasFiltering, SignalEvent),
        (Digitization, SignalEvent), (WindowApplication, SignalEvent),
        (SpectralTransform, SignalEvent), (SpectralSmoothing, SignalEvent),
        (FeatureExtraction, SignalEvent), (PatternClassification, SignalEvent),
    ],

    has_a: [
        (Spectrogram, WindowFunction), (Spectrogram, FrequencyDomain), (Spectrogram, TimeDomain),
        (MelFrequencyCepstrum, CepstralAnalysis), (MelFrequencyCepstrum, BandPassFilter),
    ],

    causes: [
        (RawSignal, AntiAliasFiltering),
        (AntiAliasFiltering, Digitization),
        (Digitization, WindowApplication),
        (WindowApplication, SpectralTransform),
        (SpectralTransform, FeatureExtraction),
        (FeatureExtraction, PatternClassification),
        (SpectralTransform, SpectralSmoothing),
        (SpectralSmoothing, FeatureExtraction),
    ],

    opposes: [
        (TimeDomain, FrequencyDomain), (FrequencyDomain, TimeDomain),
        (LowPassFilter, HighPassFilter), (HighPassFilter, LowPassFilter),
        (Decimation, Interpolation), (Interpolation, Decimation),
        (FFT, InverseFFT), (InverseFFT, FFT),
    ],
}

#[derive(Debug, Clone)]
pub struct ComputationalComplexity;
impl Quality for ComputationalComplexity {
    type Individual = SignalConcept;
    type Value = ComplexityClass;
    fn get(&self, individual: &SignalConcept) -> Option<ComplexityClass> {
        use SignalConcept::*;
        match individual {
            // Cooley & Tukey (1965) Math. Comput. 19(90):297 — O(N log N).
            FFT | InverseFFT => Some(ComplexityClass::Linearithmic),
            // Direct DFT / naive convolution and correlation — O(N^2).
            FourierTransform | Convolution | Correlation => Some(ComplexityClass::Quadratic),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SidelobeLevel;
impl Quality for SidelobeLevel {
    type Individual = SignalConcept;
    type Value = LogarithmicLevel;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &SignalConcept) -> Option<LogarithmicLevel> {
        use SignalConcept::*;
        // Peak sidelobe level relative to the mainlobe peak — a relative dB
        // field ratio (20·log₁₀ of a root-power ratio), IEC 80000-15.
        // Harris (1978) Proc. IEEE 66(1):51, Table I.
        let decibels = match individual {
            RectangularWindow => -13.0,
            HannWindow => -31.5,
            HammingWindow => -42.0,
            BlackmanWindow => -58.0,
            _ => return None,
        };
        Some(LogarithmicLevel::new(decibels, Ref::FieldRatio))
    }
}

#[derive(Debug, Clone)]
pub struct MainlobeBandwidth;
impl Quality for MainlobeBandwidth {
    type Individual = SignalConcept;
    type Value = Quantity;
    const KIND: QualityKind = QualityKind::Physical;
    fn get(&self, individual: &SignalConcept) -> Option<Quantity> {
        use SignalConcept::*;
        // Mainlobe width in DFT bins, normalised to the rectangular window's —
        // a dimensionless (unitless) figure. Harris (1978) Proc. IEEE
        // 66(1):51, Table I.
        let bins = match individual {
            RectangularWindow => 1.0,
            HannWindow => 2.0,
            BlackmanWindow => 3.0,
            _ => return None,
        };
        Some(Quantity::from_unit(bins, &UNITLESS))
    }
}

fn parts_of(whole: SignalConcept) -> Vec<SignalConcept> {
    use pr4xis::category::{Arrow, Category};
    SignalCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == SignalRelationKind::Parthood && m.target() == whole)
        .map(|m| m.source())
        .collect()
}

fn effects_of(cause: SignalConcept) -> Vec<SignalConcept> {
    use pr4xis::category::{Arrow, Category};
    SignalCategory::morphisms()
        .iter()
        .filter(|m| m.kind() == SignalRelationKind::Causation && m.source() == cause)
        .map(|m| m.target())
        .collect()
}

pub struct SpectrogramContainsDomains;
impl Axiom for SpectrogramContainsDomains {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use SignalConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let parts = parts_of(Spectrogram);
        if parts.contains(&TimeDomain)
            && parts.contains(&FrequencyDomain)
            && parts.contains(&WindowFunction)
        {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "SpectrogramContainsDomains",
        "spectrogram contains time-domain, frequency-domain, and window components",
        "Oppenheim & Schafer (2010) Discrete-Time Signal Processing §10"
    );
}
pr4xis::register_axiom!(
    SpectrogramContainsDomains,
    "Oppenheim & Schafer (2010) Discrete-Time Signal Processing §10"
);

pub struct RectangularNarrowestMainlobe;
impl Axiom for RectangularNarrowestMainlobe {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use SignalConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let r = MainlobeBandwidth
            .get(&RectangularWindow)
            .map(|q| q.value)
            .unwrap_or(f64::MAX);
        let h = MainlobeBandwidth
            .get(&HannWindow)
            .map(|q| q.value)
            .unwrap_or(0.0);
        let b = MainlobeBandwidth
            .get(&BlackmanWindow)
            .map(|q| q.value)
            .unwrap_or(0.0);
        if r < h && h < b {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "RectangularNarrowestMainlobe",
        "rectangular window has narrowest mainlobe bandwidth",
        "Harris (1978) Proc. IEEE 66(1):51"
    );
}
pr4xis::register_axiom!(
    RectangularNarrowestMainlobe,
    "Harris (1978) Proc. IEEE 66(1):51"
);

pub struct BlackmanBestSidelobes;
impl Axiom for BlackmanBestSidelobes {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use SignalConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let s = SidelobeLevel;
        let bk = s.get(&BlackmanWindow).map(|l| l.decibels).unwrap_or(0.0);
        let hn = s.get(&HannWindow).map(|l| l.decibels).unwrap_or(0.0);
        let re = s.get(&RectangularWindow).map(|l| l.decibels).unwrap_or(0.0);
        if bk < hn && hn < re {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "BlackmanBestSidelobes",
        "Blackman window has lowest sidelobes",
        "Harris (1978) Proc. IEEE 66(1):51"
    );
}
pr4xis::register_axiom!(BlackmanBestSidelobes, "Harris (1978) Proc. IEEE 66(1):51");

pub struct RawSignalCausesClassification;
impl Axiom for RawSignalCausesClassification {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use SignalConcept::*;
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        if effects_of(RawSignal).contains(&PatternClassification) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "RawSignalCausesClassification",
        "raw signal transitively causes pattern classification",
        "Oppenheim & Schafer (2010) Discrete-Time Signal Processing"
    );
}
pr4xis::register_axiom!(
    RawSignalCausesClassification,
    "Oppenheim & Schafer (2010) Discrete-Time Signal Processing"
);

impl Ontology for SignalOntology {
    type Cat = SignalCategory;
    type Qual = SidelobeLevel;
    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut a = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        a.push(Box::new(SpectrogramContainsDomains));
        a.push(Box::new(RectangularNarrowestMainlobe));
        a.push(Box::new(BlackmanBestSidelobes));
        a.push(Box::new(RawSignalCausesClassification));
        a
    }
}

// Back-compat aliases.
pub use SignalCategory as SignalProcessingCategory;
pub use SignalConcept as SignalEntity;
pub use SignalOntology as SignalProcessingOntology;
pub use SignalRelationKind as SignalProcessingCategoryRelationKind;

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;
    use pr4xis::category::{Arrow, Category};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<SignalCategory>();
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        SignalOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn spectrogram_contains_domains() {
        assert!(SpectrogramContainsDomains.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn rectangular_narrowest_mainlobe() {
        assert!(RectangularNarrowestMainlobe.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn blackman_best_sidelobes() {
        assert!(BlackmanBestSidelobes.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn raw_signal_causes_classification() {
        assert!(RawSignalCausesClassification.verify().is_ok());
    }

    proptest! {
        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            for m in SignalCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }
        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in SignalOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
