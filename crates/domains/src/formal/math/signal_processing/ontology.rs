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
    being: AbstractObject,

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
    fn description(&self) -> &str {
        "Nyquist theorem: sampling at f_s >= 2*bandwidth preserves signal information"
    }

    fn holds(&self) -> bool {
        let bandwidths = [1.0, 100.0, 22050.0, 1e6];
        for &bw in &bandwidths {
            let nyquist = sampling::nyquist_rate(bw);
            if !sampling::is_adequately_sampled(nyquist, bw) {
                return false;
            }
            if !sampling::is_adequately_sampled(nyquist + 1.0, bw) {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(NyquistTheorem, "Shannon (1949), Nyquist (1928).");

/// Aliasing occurs when the sample rate is below the Nyquist rate.
pub struct AliasingOccursBelowNyquist;

impl Axiom for AliasingOccursBelowNyquist {
    fn description(&self) -> &str {
        "aliasing occurs when sample rate < 2 * bandwidth (below Nyquist rate)"
    }

    fn holds(&self) -> bool {
        let bandwidths = [100.0, 1000.0, 22050.0];
        for &bw in &bandwidths {
            let nyquist = sampling::nyquist_rate(bw);
            if sampling::is_adequately_sampled(nyquist - 1.0, bw) {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(
    AliasingOccursBelowNyquist,
    "Shannon (1949), Nyquist (1928)."
);

/// Bandwidth is always positive (a signal occupies non-negative frequency range).
pub struct BandwidthPositive;

impl Axiom for BandwidthPositive {
    fn description(&self) -> &str {
        "bandwidth is positive, therefore Nyquist rate is positive"
    }

    fn holds(&self) -> bool {
        let bandwidths = [0.001, 1.0, 100.0, 1e9];
        for &bw in &bandwidths {
            if bw <= 0.0 {
                return false;
            }
            if sampling::nyquist_rate(bw) <= 0.0 {
                return false;
            }
        }
        true
    }
}
pr4xis::register_axiom!(BandwidthPositive, "Shannon (1949), Nyquist (1928).");

impl Ontology for SignalProcessingOntology {
    type Cat = SignalProcessingCategory;
    type Qual = ConceptDescription;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(NyquistTheorem),
            Box::new(AliasingOccursBelowNyquist),
            Box::new(BandwidthPositive),
        ]
    }
}

/// Create a first-order low-pass filter for axiom testing convenience.
pub fn test_low_pass_filter(alpha: f64) -> FirstOrderLowPass {
    FirstOrderLowPass::from_alpha(alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<SignalProcessingCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        SignalProcessingOntology::validate().unwrap();
    }
}
