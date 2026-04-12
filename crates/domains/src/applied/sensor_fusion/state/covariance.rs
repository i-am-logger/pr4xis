use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::positive_definite;

/// Covariance operations for state estimation.
///
/// Delegates to linear_algebra ontology for the math.
/// This module provides the sensor fusion INTERPRETATION.
///
/// Key invariant: covariance is always symmetric PSD.
///
/// Source: Maybeck (1979), Vol. 1, Chapter 5.
/// Ensure covariance is symmetric (force symmetry).
pub fn ensure_symmetric(p: &Matrix) -> Matrix {
    positive_definite::symmetrize(p)
}

/// Check if covariance is valid (symmetric PSD).
pub fn is_valid(p: &Matrix) -> bool {
    p.is_symmetric(1e-10) && positive_definite::is_positive_semidefinite(p)
}

/// Extract the uncertainty (standard deviation) for a specific state index.
pub fn std_dev(p: &Matrix, index: usize) -> f64 {
    p.get(index, index).sqrt()
}

/// Extract the correlation coefficient between two state components.
pub fn correlation(p: &Matrix, i: usize, j: usize) -> f64 {
    let sigma_i = p.get(i, i).sqrt();
    let sigma_j = p.get(j, j).sqrt();
    if sigma_i < 1e-15 || sigma_j < 1e-15 {
        return 0.0;
    }
    p.get(i, j) / (sigma_i * sigma_j)
}

/// Total uncertainty: trace(P) = sum of variances.
pub fn total_uncertainty(p: &Matrix) -> f64 {
    p.trace()
}
