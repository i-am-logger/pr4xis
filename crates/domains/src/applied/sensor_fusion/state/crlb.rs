use crate::formal::math::linear_algebra::decomposition;
use crate::formal::math::linear_algebra::matrix::Matrix;

/// Cramér-Rao Lower Bound (CRLB).
///
/// The CRLB provides a lower bound on the variance of any unbiased
/// estimator. If P is the estimator covariance and J is the Fisher
/// information matrix, then P ≥ J^{-1} (matrix inequality).
///
/// Source: Fisher (1925), Cramér (1946), Rao (1945).
///         Van Trees (2001), *Detection, Estimation, and Modulation Theory*.
/// Compute the CRLB matrix (inverse of Fisher information).
/// Returns None if the Fisher information is singular.
pub fn crlb(fisher_information: &Matrix) -> Option<Matrix> {
    decomposition::inverse_spd(fisher_information)
}

/// Check if an estimator achieves the CRLB (efficient estimator).
///
/// An estimator is efficient if P = J^{-1} (achieves the bound).
/// The Kalman filter is the minimum variance unbiased estimator
/// for linear Gaussian systems — it achieves the CRLB.
pub fn is_efficient(covariance: &Matrix, fisher_information: &Matrix, tol: f64) -> bool {
    if let Some(bound) = crlb(fisher_information) {
        // Check P ≈ J^{-1} element-wise
        covariance
            .data
            .iter()
            .zip(&bound.data)
            .all(|(p, b)| (p - b).abs() < tol)
    } else {
        false
    }
}

/// Check if an estimator is consistent (P ≥ J^{-1}).
///
/// P - J^{-1} should be positive semi-definite.
pub fn is_consistent(covariance: &Matrix, fisher_information: &Matrix) -> bool {
    if let Some(bound) = crlb(fisher_information) {
        let diff = covariance.sub(&bound);
        crate::formal::math::linear_algebra::positive_definite::is_positive_semidefinite(&diff)
    } else {
        false
    }
}
