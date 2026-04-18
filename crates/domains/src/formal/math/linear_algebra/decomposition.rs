#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::matrix::Matrix;

/// Cholesky decomposition: A = L L^T.
///
/// Exists iff A is symmetric positive definite.
/// L is lower triangular with positive diagonal.
///
/// This is the fundamental factorization for covariance matrices.
///
/// Source: Golub & Van Loan, *Matrix Computations* (2013), Chapter 4.
pub fn cholesky(a: &Matrix) -> Option<Matrix> {
    if !a.is_square() {
        return None;
    }
    let n = a.rows;
    let mut l = vec![0.0; n * n];

    for j in 0..n {
        let mut sum = 0.0;
        for k in 0..j {
            sum += l[j * n + k] * l[j * n + k];
        }
        let diag = a.get(j, j) - sum;
        if diag <= 0.0 {
            return None; // not positive definite
        }
        l[j * n + j] = diag.sqrt();

        for i in (j + 1)..n {
            let mut sum = 0.0;
            for k in 0..j {
                sum += l[i * n + k] * l[j * n + k];
            }
            l[i * n + j] = (a.get(i, j) - sum) / l[j * n + j];
        }
    }

    Some(Matrix::new(n, n, l))
}

/// Solve L x = b where L is lower triangular (forward substitution).
/// Returns None if a diagonal element is zero (singular).
#[allow(clippy::needless_range_loop)]
pub fn solve_lower_triangular(l: &Matrix, b: &[f64]) -> Option<Vec<f64>> {
    let n = l.rows;
    let mut x = vec![0.0; n];
    for i in 0..n {
        let mut sum = 0.0;
        for j in 0..i {
            sum += l.get(i, j) * x[j];
        }
        let diag = l.get(i, i);
        if diag.abs() < 1e-15 {
            return None;
        }
        x[i] = (b[i] - sum) / diag;
    }
    Some(x)
}

/// Solve L^T x = b where L is lower triangular (back substitution).
/// Returns None if a diagonal element is zero (singular).
#[allow(clippy::needless_range_loop)]
pub fn solve_upper_triangular(l: &Matrix, b: &[f64]) -> Option<Vec<f64>> {
    let n = l.rows;
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut sum = 0.0;
        for j in (i + 1)..n {
            sum += l.get(j, i) * x[j]; // L^T[i][j] = L[j][i]
        }
        let diag = l.get(i, i);
        if diag.abs() < 1e-15 {
            return None;
        }
        x[i] = (b[i] - sum) / diag;
    }
    Some(x)
}

/// Solve A x = b where A is symmetric positive definite (via Cholesky).
/// Returns None if Cholesky fails (A not PD) or triangular solve fails.
pub fn solve_spd(a: &Matrix, b: &[f64]) -> Option<Vec<f64>> {
    let l = cholesky(a)?;
    let y = solve_lower_triangular(&l, b)?;
    solve_upper_triangular(&l, &y)
}

/// Compute A^{-1} for symmetric positive definite A (via Cholesky).
pub fn inverse_spd(a: &Matrix) -> Option<Matrix> {
    let n = a.rows;
    let l = cholesky(a)?;
    let mut inv_data = vec![0.0; n * n];

    for j in 0..n {
        let mut e = vec![0.0; n];
        e[j] = 1.0;
        let y = solve_lower_triangular(&l, &e)?;
        let col = solve_upper_triangular(&l, &y)?;
        for i in 0..n {
            inv_data[i * n + j] = col[i];
        }
    }

    Some(Matrix::new(n, n, inv_data))
}
