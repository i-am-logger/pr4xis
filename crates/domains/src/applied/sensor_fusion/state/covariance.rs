use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::positive_definite;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

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
///
/// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`) — `Matrix` is a
/// generic covariance accessor with no fixed physical meaning at this layer
/// (same treatment as `Matrix::trace`/`Vector::norm`); a caller that knows
/// the state component is, say, a position re-grounds it in the physical
/// unit that component actually carries.
pub fn std_dev(p: &Matrix, index: usize) -> Quantity {
    Quantity::from_unit(p.get(index, index).sqrt(), &unit::UNITLESS)
}

/// Extract the correlation coefficient between two state components.
///
/// A correlation coefficient is always dimensionless by definition
/// (Pearson 1895), regardless of what physical quantity the covariance
/// matrix represents.
pub fn correlation(p: &Matrix, i: usize, j: usize) -> Quantity {
    let sigma_i = p.get(i, i).sqrt();
    let sigma_j = p.get(j, j).sqrt();
    if sigma_i < 1e-15 || sigma_j < 1e-15 {
        return Quantity::from_unit(0.0, &unit::UNITLESS);
    }
    Quantity::from_unit(p.get(i, j) / (sigma_i * sigma_j), &unit::UNITLESS)
}

/// Total uncertainty: trace(P) = sum of variances.
pub fn total_uncertainty(p: &Matrix) -> Quantity {
    p.trace()
}
