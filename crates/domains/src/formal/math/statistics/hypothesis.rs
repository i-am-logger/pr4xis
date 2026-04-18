#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Hypothesis testing framework.
///
/// Neyman, J. & Pearson, E.S. (1933). "On the Problem of the Most Efficient Tests."
/// Student (Gosset) (1908). "The Probable Error of a Mean." Biometrika.
///
/// Null hypothesis H0 vs alternative H1.
/// Type I error: rejecting H0 when it is true (false positive), probability = α.
/// Type II error: failing to reject H0 when it is false (false negative), probability = β.
/// Power = 1 - β.
/// The result of a hypothesis test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestDecision {
    /// Reject the null hypothesis (evidence supports H1).
    RejectNull,
    /// Fail to reject the null hypothesis (insufficient evidence).
    FailToReject,
}

/// Decide whether to reject H0 based on p-value and significance level.
///
/// Reject H0 if p < α.
pub fn test_decision(p_value: f64, significance_level: f64) -> TestDecision {
    if p_value < significance_level {
        TestDecision::RejectNull
    } else {
        TestDecision::FailToReject
    }
}

/// Compute a z-test statistic for testing a population mean.
///
/// z = (x̄ - μ₀) / (σ / √n)
///
/// where x̄ is the sample mean, μ₀ is the hypothesized mean,
/// σ is the known population standard deviation, and n is the sample size.
pub fn z_statistic(sample_mean: f64, hypothesized_mean: f64, std_dev: f64, n: usize) -> f64 {
    if n == 0 || std_dev <= 0.0 {
        return 0.0;
    }
    (sample_mean - hypothesized_mean) / (std_dev / (n as f64).sqrt())
}

/// Approximate two-sided p-value from a z-statistic using the standard normal.
///
/// Uses the complementary error function approximation.
pub fn two_sided_p_value(z: f64) -> f64 {
    // Standard normal CDF approximation using erfc
    // P(Z > |z|) = erfc(|z|/√2) / 2
    // Two-sided p = 2 * P(Z > |z|) = erfc(|z|/√2)
    erfc(z.abs() / core::f64::consts::SQRT_2)
}

/// Complementary error function approximation (Abramowitz & Stegun 7.1.26).
fn erfc(x: f64) -> f64 {
    if x < 0.0 {
        return 2.0 - erfc(-x);
    }
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let poly = t
        * (0.254829592
            + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    poly * (-x * x).exp()
}

/// Type I error rate equals the significance level α (by definition).
pub fn type_i_error_rate(significance_level: f64) -> f64 {
    significance_level
}

/// Power of a test: 1 - β where β is the Type II error rate.
pub fn power(type_ii_error_rate: f64) -> f64 {
    1.0 - type_ii_error_rate
}
