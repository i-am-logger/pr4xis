use crate::formal::math::linear_algebra::determinant;
use crate::formal::math::linear_algebra::matrix::Matrix;
use core::f64::consts::PI;

/// Shannon entropy of a discrete distribution.
///
/// H(X) = -Σ p_i ln(p_i)
///
/// Measures the uncertainty in a random variable.
/// Maximum for uniform distribution: H = ln(n).
///
/// Source: Shannon, C.E. (1948). "A Mathematical Theory of Communication."
pub fn shannon_entropy(probabilities: &[f64]) -> f64 {
    probabilities
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.ln())
        .sum()
}

/// Differential entropy of a univariate Gaussian.
///
/// h(X) = 0.5 * ln(2πeσ²)
///
/// Source: Cover & Thomas, *Elements of Information Theory* (2006).
pub fn gaussian_entropy_1d(variance: f64) -> f64 {
    0.5 * (2.0 * PI * core::f64::consts::E * variance).ln()
}

/// Differential entropy of a multivariate Gaussian.
///
/// h(X) = 0.5 * ln((2πe)^n |P|) = 0.5 * (n*ln(2πe) + ln|P|)
///
/// This quantifies the uncertainty in a state estimate.
/// Smaller entropy = more certain estimate.
pub fn gaussian_entropy_nd(covariance: &Matrix) -> f64 {
    let n = covariance.rows as f64;
    let log_det = determinant::det(covariance).ln();
    0.5 * (n * (2.0 * PI * core::f64::consts::E).ln() + log_det)
}

/// KL divergence from distribution q to p (discrete).
///
/// D_KL(p || q) = Σ p_i ln(p_i / q_i)
///
/// Measures how much q differs from p.
/// D_KL ≥ 0, with equality iff p = q (Gibbs' inequality).
pub fn kl_divergence_discrete(p: &[f64], q: &[f64]) -> f64 {
    assert_eq!(p.len(), q.len());
    p.iter()
        .zip(q)
        .filter(|(pi, _)| **pi > 0.0)
        .map(|(pi, qi)| {
            if *qi <= 0.0 {
                f64::INFINITY
            } else {
                pi * (pi / qi).ln()
            }
        })
        .sum()
}
