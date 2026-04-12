use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::science::math::signal_processing::filter::FirstOrderLowPass;
use crate::science::math::signal_processing::sampling;

// ---------------------------------------------------------------------------
// Entity: signal domain concepts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum SignalDomainConcept {
    TimeDomain,
    FrequencyDomain,
    SampleRate,
    Bandwidth,
    NyquistRate,
    AliasFrequency,
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over signal domain concept entities.
    pub SignalCategory {
        entity: SignalDomainConcept,
        relation: SignalRelation,
    }
}

// ---------------------------------------------------------------------------
// Quality: concept descriptions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = SignalDomainConcept;
    type Value = &'static str;

    fn get(&self, c: &SignalDomainConcept) -> Option<&'static str> {
        Some(match c {
            SignalDomainConcept::TimeDomain => "signal represented as amplitude vs time, x(t)",
            SignalDomainConcept::FrequencyDomain => {
                "signal represented as amplitude/phase vs frequency, X(f) = F{x(t)}"
            }
            SignalDomainConcept::SampleRate => "number of samples per second, f_s (Hz)",
            SignalDomainConcept::Bandwidth => {
                "range of frequencies occupied by a signal, B = f_max - f_min"
            }
            SignalDomainConcept::NyquistRate => {
                "minimum sample rate to avoid aliasing: f_nyquist = 2 * f_max"
            }
            SignalDomainConcept::AliasFrequency => {
                "spurious frequency from under-sampling: appears when f_s < 2*f_max"
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms — Shannon (1949), Nyquist (1928)
// ---------------------------------------------------------------------------

/// Nyquist theorem: adequate sampling at f_s >= 2*bandwidth preserves all information.
pub struct NyquistTheorem;

impl Axiom for NyquistTheorem {
    fn description(&self) -> &str {
        "Nyquist theorem: sampling at f_s >= 2*bandwidth preserves signal information"
    }

    fn holds(&self) -> bool {
        // Test several bandwidths: nyquist rate should always work
        let bandwidths = [1.0, 100.0, 22050.0, 1e6];
        for &bw in &bandwidths {
            let nyquist = sampling::nyquist_rate(bw);
            if !sampling::is_adequately_sampled(nyquist, bw) {
                return false;
            }
            // Slightly above Nyquist must also be adequate
            if !sampling::is_adequately_sampled(nyquist + 1.0, bw) {
                return false;
            }
        }
        true
    }
}

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
            // Below Nyquist should NOT be adequately sampled
            if sampling::is_adequately_sampled(nyquist - 1.0, bw) {
                return false;
            }
        }
        true
    }
}

/// Bandwidth is always positive (a signal occupies non-negative frequency range).
pub struct BandwidthPositive;

impl Axiom for BandwidthPositive {
    fn description(&self) -> &str {
        "bandwidth is positive, therefore Nyquist rate is positive"
    }

    fn holds(&self) -> bool {
        // For any positive bandwidth, the Nyquist rate is positive
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

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The signal processing ontology.
///
/// Founded on:
///   - Shannon, C.E. (1949). "Communication in the Presence of Noise." Proc. IRE.
///   - Nyquist, H. (1928). "Certain Topics in Telegraph Transmission Theory." Trans. AIEE.
///   - Oppenheim & Willsky (1997). *Signals and Systems* (2nd ed.).
pub struct SignalProcessingOntology;

impl Ontology for SignalProcessingOntology {
    type Cat = SignalCategory;
    type Qual = ConceptDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(NyquistTheorem),
            Box::new(AliasingOccursBelowNyquist),
            Box::new(BandwidthPositive),
        ]
    }
}

// ---------------------------------------------------------------------------
// Re-export for tests
// ---------------------------------------------------------------------------

/// Create a first-order low-pass filter for axiom testing convenience.
pub fn test_low_pass_filter(alpha: f64) -> FirstOrderLowPass {
    FirstOrderLowPass::from_alpha(alpha)
}
