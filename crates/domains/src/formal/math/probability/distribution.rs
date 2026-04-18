#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// A probability distribution over a discrete finite sample space.
///
/// Kolmogorov's axioms (Grundbegriffe, 1933):
/// 1. Non-negativity: P(E) ≥ 0 for all events E
/// 2. Normalization: P(Ω) = 1
/// 3. σ-additivity: P(⋃E_i) = ΣP(E_i) for mutually exclusive events
///
/// Consequences:
/// - P(∅) = 0
/// - P(A^c) = 1 - P(A)
/// - 0 ≤ P(E) ≤ 1
/// - Monotonicity: A ⊆ B ⟹ P(A) ≤ P(B)
/// - Inclusion-exclusion: P(A ∪ B) = P(A) + P(B) - P(A ∩ B)
#[derive(Debug, Clone, PartialEq)]
pub struct DiscreteDistribution {
    /// Probability of each outcome. Must sum to 1.0, all non-negative.
    pub probabilities: Vec<f64>,
}

impl DiscreteDistribution {
    /// Create from probabilities. Validates Kolmogorov axioms.
    pub fn new(probabilities: Vec<f64>) -> Option<Self> {
        // Axiom 1: non-negativity
        if probabilities.iter().any(|&p| p < 0.0) {
            return None;
        }
        // Axiom 2: normalization
        let sum: f64 = probabilities.iter().sum();
        if (sum - 1.0).abs() > 1e-10 {
            return None;
        }
        Some(Self { probabilities })
    }

    /// Uniform distribution over n outcomes.
    pub fn uniform(n: usize) -> Self {
        let p = 1.0 / n as f64;
        Self {
            probabilities: vec![p; n],
        }
    }

    /// Number of outcomes in the sample space.
    pub fn size(&self) -> usize {
        self.probabilities.len()
    }

    /// Probability of outcome i.
    pub fn prob(&self, i: usize) -> f64 {
        self.probabilities[i]
    }

    /// Probability of an event (set of outcomes by index).
    pub fn event_prob(&self, indices: &[usize]) -> f64 {
        indices.iter().map(|&i| self.probabilities[i]).sum()
    }

    /// Complement probability: P(A^c) = 1 - P(A).
    pub fn complement_prob(&self, indices: &[usize]) -> f64 {
        1.0 - self.event_prob(indices)
    }

    /// Expected value E[X] given a value function f(i).
    pub fn expected_value(&self, f: &dyn Fn(usize) -> f64) -> f64 {
        self.probabilities
            .iter()
            .enumerate()
            .map(|(i, &p)| p * f(i))
            .sum()
    }

    /// Variance Var[X] = E[X²] - E[X]².
    pub fn variance(&self, f: &dyn Fn(usize) -> f64) -> f64 {
        let mean = self.expected_value(f);
        let mean_sq = self.expected_value(&|i| f(i) * f(i));
        mean_sq - mean * mean
    }
}
