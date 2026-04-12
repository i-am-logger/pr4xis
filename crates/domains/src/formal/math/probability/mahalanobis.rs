use crate::formal::math::linear_algebra::decomposition;
use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

/// Mahalanobis distance: d² = (x - μ)^T S^{-1} (x - μ).
///
/// A scale-invariant distance that accounts for correlations.
/// When S = I, reduces to Euclidean distance.
///
/// In sensor fusion, this is used for:
/// - Gating: is this measurement consistent with the predicted state?
/// - Normalized Innovation Squared (NIS): ν^T S^{-1} ν
///
/// Source: Mahalanobis, P.C. (1936). "On the generalized distance in statistics."
///         Bar-Shalom et al. (2001). Chapter 2 (gating).
pub fn mahalanobis_squared(x: &Vector, mean: &Vector, covariance: &Matrix) -> Option<f64> {
    let diff = x.sub(mean);
    let s_inv_diff = decomposition::solve_spd(covariance, &diff.data)?;
    Some(diff.data.iter().zip(&s_inv_diff).map(|(a, b)| a * b).sum())
}

/// Mahalanobis distance (square root of squared distance).
pub fn mahalanobis(x: &Vector, mean: &Vector, covariance: &Matrix) -> Option<f64> {
    mahalanobis_squared(x, mean, covariance).map(|d2| d2.sqrt())
}

/// Validation gate: is the Mahalanobis distance within the chi-squared threshold?
///
/// For n-dimensional Gaussian, d² follows chi-squared distribution with n DOF.
/// Common thresholds:
///   n=1: 3.84 (95%), 6.63 (99%)
///   n=2: 5.99 (95%), 9.21 (99%)
///   n=3: 7.81 (95%), 11.34 (99%)
///
/// Source: Bar-Shalom et al. (2001). Table 2.1.
pub fn within_gate(x: &Vector, mean: &Vector, covariance: &Matrix, threshold: f64) -> Option<bool> {
    mahalanobis_squared(x, mean, covariance).map(|d2| d2 < threshold)
}

/// Chi-squared thresholds for common confidence levels and dimensions.
/// Returns the threshold for a given dimension and confidence level.
///
/// Source: standard chi-squared distribution tables.
pub fn chi_squared_threshold(dim: usize, confidence: f64) -> f64 {
    // Approximation for common cases
    match (dim, (confidence * 100.0) as u32) {
        (1, 95) => 3.841,
        (1, 99) => 6.635,
        (2, 95) => 5.991,
        (2, 99) => 9.210,
        (3, 95) => 7.815,
        (3, 99) => 11.345,
        (4, 95) => 9.488,
        (4, 99) => 13.277,
        (6, 95) => 12.592,
        (6, 99) => 16.812,
        _ => {
            // Wilson-Hilferty approximation for chi-squared quantile
            let n = dim as f64;
            let z = if confidence > 0.99 {
                2.576
            } else if confidence > 0.95 {
                1.960
            } else {
                1.645
            };
            let term = 1.0 - 2.0 / (9.0 * n) + z * (2.0 / (9.0 * n)).sqrt();
            n * term * term * term
        }
    }
}
