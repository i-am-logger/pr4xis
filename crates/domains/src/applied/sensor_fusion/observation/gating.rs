#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::probability::mahalanobis;
use crate::formal::math::statistics::hypothesis;

use crate::applied::sensor_fusion::observation::innovation::Innovation;

/// Validation gate: decides if a measurement is consistent with the prediction.
///
/// Uses Mahalanobis distance: d² = ν^T S^{-1} ν
/// Compared against chi-squared threshold for the measurement dimension.
///
/// Source: Bar-Shalom et al. (2001), Section 2.5.
///         Mahalanobis (1936).
#[derive(Debug, Clone)]
pub struct ValidationGate {
    /// Chi-squared threshold (depends on dimension and confidence).
    pub threshold: f64,
    /// Confidence level (e.g., 0.95 for 95%).
    pub confidence: f64,
}

impl ValidationGate {
    /// Create a gate for given dimension and confidence level.
    /// Uses chi-squared threshold from the probability ontology.
    pub fn new(dimension: usize, confidence: f64) -> Self {
        let threshold = mahalanobis::chi_squared_threshold(dimension, confidence);
        Self {
            threshold,
            confidence,
        }
    }

    /// Check if the innovation passes the gate.
    pub fn accept(&self, innovation: &Innovation) -> bool {
        let d2 = mahalanobis::mahalanobis_squared(
            &innovation.residual,
            &Vector::zeros(innovation.dim()),
            &innovation.covariance,
        );
        match d2 {
            Some(d2) => d2 < self.threshold,
            None => false,
        }
    }

    /// Normalized Innovation Squared (NIS) — the test statistic.
    pub fn nis(&self, innovation: &Innovation) -> Option<f64> {
        mahalanobis::mahalanobis_squared(
            &innovation.residual,
            &Vector::zeros(innovation.dim()),
            &innovation.covariance,
        )
    }
}

/// Frame the Mahalanobis gate as a hypothesis test using the statistics ontology.
///
/// H0: measurement is from this track (innovation is chi-squared distributed).
/// H1: measurement is an outlier.
///
/// The NIS (Normalized Innovation Squared) is the test statistic.
/// Under H0, NIS ~ chi-squared(dim). We compare NIS against the chi-squared
/// threshold at the given significance level.
///
/// If NIS > threshold, we reject H0 (measurement is an outlier).
///
/// The significance level alpha is the probability of rejecting a valid
/// measurement (Type I error). Typical values: 0.05, 0.01.
///
/// Source: Bar-Shalom et al. (2001), Section 2.5 (validation gating).
///         Neyman & Pearson (1933) (hypothesis testing framework).
pub fn gate_as_hypothesis_test(
    nis: f64,
    dimension: usize,
    significance: f64,
) -> hypothesis::TestDecision {
    let threshold = mahalanobis::chi_squared_threshold(dimension, 1.0 - significance);
    // Under H0, large NIS is unlikely. If NIS > threshold, reject H0.
    // We express this via p-value: approximate p-value from chi-squared.
    // For a chi-squared test, if test_statistic > threshold then p < alpha.
    // So: p_value < significance iff nis > threshold.
    let p_value = if nis > threshold {
        // NIS exceeds threshold => p < significance => reject
        significance / 2.0
    } else {
        // NIS within threshold => p > significance => fail to reject
        1.0 - significance / 2.0
    };
    hypothesis::test_decision(p_value, significance)
}
