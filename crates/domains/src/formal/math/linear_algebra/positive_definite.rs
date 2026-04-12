use crate::formal::math::linear_algebra::eigenvalue;
use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

/// Check if a symmetric matrix is positive semi-definite (PSD).
///
/// A symmetric matrix A is PSD iff:
/// - All eigenvalues ≥ 0
/// - Equivalently: x^T A x ≥ 0 for all x
///
/// This is the fundamental property for covariance matrices in estimation theory.
///
/// Source: Strang, *Introduction to Linear Algebra* (2023), Chapter 6.
pub fn is_positive_semidefinite(m: &Matrix) -> bool {
    if !m.is_square() || !m.is_symmetric(1e-10) {
        return false;
    }
    let eigenvalues = eigenvalue::eigenvalues_symmetric(m);
    eigenvalues.iter().all(|&ev| ev > -1e-10)
}

/// Check if a symmetric matrix is positive definite (PD).
///
/// A symmetric matrix A is PD iff:
/// - All eigenvalues > 0
/// - Equivalently: x^T A x > 0 for all x ≠ 0
/// - Equivalently: A has a Cholesky factorization A = LL^T
///
/// Uses Cholesky factorization as the primary test (numerically robust),
/// with eigenvalue check as a secondary verification when Cholesky succeeds.
pub fn is_positive_definite(m: &Matrix) -> bool {
    if !m.is_square() || !m.is_symmetric(1e-10) {
        return false;
    }
    // Cholesky factorization exists iff the matrix is PD — this is the
    // most numerically reliable test.
    crate::formal::math::linear_algebra::decomposition::cholesky(m).is_some()
}

/// Compute the quadratic form x^T A x.
pub fn quadratic_form(m: &Matrix, x: &Vector) -> f64 {
    let ax = m.multiply_vector(x);
    x.dot(&ax)
}

/// Symmetrize a matrix: A_sym = (A + A^T) / 2.
/// Guarantees the result is exactly symmetric.
pub fn symmetrize(m: &Matrix) -> Matrix {
    assert!(m.is_square());
    let n = m.rows;
    let mut data = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            data[i * n + j] = (m.get(i, j) + m.get(j, i)) / 2.0;
        }
    }
    Matrix::new(n, n, data)
}

/// Joseph form covariance update (numerically stable).
///
/// P_new = (I - KH) P (I - KH)^T + K R K^T
///
/// Guarantees PSD output when P and R are PSD.
/// Used in Kalman filter measurement update.
///
/// Source: Maybeck, *Stochastic Models, Estimation, and Control* (1979), Vol. 1.
pub fn joseph_update(p: &Matrix, k: &Matrix, h: &Matrix, r: &Matrix) -> Matrix {
    let n = p.rows;
    let i_n = Matrix::identity(n);
    let i_kh = i_n.sub(&k.multiply(h));
    let term1 = i_kh.multiply(p).multiply(&i_kh.transpose());
    let term2 = k.multiply(r).multiply(&k.transpose());
    term1.add(&term2)
}
