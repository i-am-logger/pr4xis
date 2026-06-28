//! Signal domain concepts — Shannon/Nyquist sampling theory.
//!
//! Source: Shannon (1949), Nyquist (1928).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::signal_processing::filter::FirstOrderLowPass;
use crate::formal::math::signal_processing::sampling;

pr4xis::ontology! {
    name: "SignalProcessing",
    source: "Shannon (1949); Nyquist (1928)",

    concepts: [TimeDomain, FrequencyDomain, SampleRate, Bandwidth, NyquistRate, AliasFrequency],

    labels: {
        TimeDomain: ("en", "Time domain", "Signal represented as amplitude vs time, x(t)."),
        FrequencyDomain: ("en", "Frequency domain", "Signal represented as amplitude/phase vs frequency, X(f) = F{x(t)}."),
        SampleRate: ("en", "Sample rate", "Number of samples per second, f_s (Hz)."),
        Bandwidth: ("en", "Bandwidth", "Range of frequencies occupied by a signal, B = f_max - f_min."),
        NyquistRate: ("en", "Nyquist rate", "Minimum sample rate to avoid aliasing: f_nyquist = 2 * f_max."),
        AliasFrequency: ("en", "Alias frequency", "Spurious frequency from under-sampling: appears when f_s < 2*f_max."),
    },
}

#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = SignalProcessingConcept;
    type Value = &'static str;

    fn get(&self, c: &SignalProcessingConcept) -> Option<&'static str> {
        Some(match c {
            SignalProcessingConcept::TimeDomain => "signal represented as amplitude vs time, x(t)",
            SignalProcessingConcept::FrequencyDomain => {
                "signal represented as amplitude/phase vs frequency, X(f) = F{x(t)}"
            }
            SignalProcessingConcept::SampleRate => "number of samples per second, f_s (Hz)",
            SignalProcessingConcept::Bandwidth => {
                "range of frequencies occupied by a signal, B = f_max - f_min"
            }
            SignalProcessingConcept::NyquistRate => {
                "minimum sample rate to avoid aliasing: f_nyquist = 2 * f_max"
            }
            SignalProcessingConcept::AliasFrequency => {
                "spurious frequency from under-sampling: appears when f_s < 2*f_max"
            }
        })
    }
}

/// Nyquist theorem: adequate sampling at f_s >= 2*bandwidth preserves all information.
pub struct NyquistTheorem;

impl Axiom for NyquistTheorem {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let bandwidths = [1.0, 100.0, 22050.0, 1e6];
        for &bw in &bandwidths {
            let nyquist = sampling::nyquist_rate(bw);
            if !sampling::is_adequately_sampled(nyquist, bw) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            if !sampling::is_adequately_sampled(nyquist + 1.0, bw) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "NyquistTheorem",
        "Nyquist theorem: sampling at f_s >= 2*bandwidth preserves signal information",
        "Shannon (1949), Nyquist (1928)."
    );
}
pr4xis::register_axiom!(NyquistTheorem, "Shannon (1949), Nyquist (1928).");

/// Aliasing occurs when the sample rate is below the Nyquist rate.
pub struct AliasingOccursBelowNyquist;

impl Axiom for AliasingOccursBelowNyquist {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let bandwidths = [100.0, 1000.0, 22050.0];
        for &bw in &bandwidths {
            let nyquist = sampling::nyquist_rate(bw);
            if sampling::is_adequately_sampled(nyquist - 1.0, bw) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "AliasingOccursBelowNyquist",
        "aliasing occurs when sample rate < 2 * bandwidth (below Nyquist rate)",
        "Shannon (1949), Nyquist (1928)."
    );
}
pr4xis::register_axiom!(
    AliasingOccursBelowNyquist,
    "Shannon (1949), Nyquist (1928)."
);

/// Bandwidth is always positive (a signal occupies non-negative frequency range).
pub struct BandwidthPositive;

impl Axiom for BandwidthPositive {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let bandwidths = [0.001, 1.0, 100.0, 1e9];
        for &bw in &bandwidths {
            if bw <= 0.0 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
            if sampling::nyquist_rate(bw) <= 0.0 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "BandwidthPositive",
        "bandwidth is positive, therefore Nyquist rate is positive",
        "Shannon (1949), Nyquist (1928)."
    );
}
pr4xis::register_axiom!(BandwidthPositive, "Shannon (1949), Nyquist (1928).");

impl Ontology for SignalProcessingOntology {
    type Cat = SignalProcessingCategory;
    type Qual = ConceptDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(NyquistTheorem));
        axioms.push(Box::new(AliasingOccursBelowNyquist));
        axioms.push(Box::new(BandwidthPositive));
        axioms
    }
}

/// Create a first-order low-pass filter for axiom testing convenience.
pub fn test_low_pass_filter(alpha: f64) -> FirstOrderLowPass {
    FirstOrderLowPass::from_alpha(alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        pr4xis::category::laws::assert_category_laws::<SignalProcessingCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        SignalProcessingOntology::validate().unwrap();
    }
}
