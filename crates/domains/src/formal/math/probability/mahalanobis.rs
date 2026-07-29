use crate::formal::math::linear_algebra::decomposition;
use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

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
///
/// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), never a bare
/// `f64` — Mahalanobis distance is normalized by the covariance and is
/// conventionally dimensionless by construction (Mahalanobis 1936), the
/// same treatment as `Vector::dot`/`Vector::norm`.
pub fn mahalanobis_squared(x: &Vector, mean: &Vector, covariance: &Matrix) -> Option<Quantity> {
    let diff = x.sub(mean);
    let s_inv_diff = decomposition::solve_spd(covariance, &diff.data)?;
    let sum: f64 = diff.data.iter().zip(&s_inv_diff).map(|(a, b)| a * b).sum();
    Some(Quantity::from_unit(sum, &unit::UNITLESS))
}

/// Mahalanobis distance (square root of squared distance).
///
/// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), same reasoning
/// as [`mahalanobis_squared`].
pub fn mahalanobis(x: &Vector, mean: &Vector, covariance: &Matrix) -> Option<Quantity> {
    mahalanobis_squared(x, mean, covariance)
        .map(|d2| Quantity::from_unit(d2.value.sqrt(), &unit::UNITLESS))
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
    mahalanobis_squared(x, mean, covariance).map(|d2| d2.value < threshold)
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
