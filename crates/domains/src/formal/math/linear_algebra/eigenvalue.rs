#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::matrix::Matrix;

/// Compute eigenvalues of a symmetric matrix using QR iteration.
///
/// For symmetric matrices, all eigenvalues are real (Spectral Theorem).
/// The determinant equals the product of eigenvalues.
/// The trace equals the sum of eigenvalues.
///
/// Source: Golub & Van Loan, *Matrix Computations* (2013), Chapter 8.
pub fn eigenvalues_symmetric(m: &Matrix) -> Vec<f64> {
    assert!(m.is_square());
    let n = m.rows;

    if n == 1 {
        return vec![m.get(0, 0)];
    }

    if n == 2 {
        return eigenvalues_2x2(m);
    }

    // QR iteration with shifts (simplified Wilkinson shift)
    let mut a = m.data.clone();
    let max_iter = 100 * n;

    for _ in 0..max_iter {
        // Check convergence: off-diagonal elements small
        let mut max_off = 0.0_f64;
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    max_off = max_off.max(a[i * n + j].abs());
                }
            }
        }
        if max_off < 1e-12 {
            break;
        }

        // Wilkinson shift: eigenvalue of bottom-right 2x2 closest to a[n-1][n-1]
        let shift = wilkinson_shift(
            a[(n - 2) * n + (n - 2)],
            a[(n - 2) * n + (n - 1)],
            a[(n - 1) * n + (n - 1)],
        );

        // Shift
        for i in 0..n {
            a[i * n + i] -= shift;
        }

        // QR decomposition via Givens rotations
        let (q, r) = qr_givens(n, &a);

        // A = RQ + shift*I
        a = mat_multiply(n, &r, &q);
        for i in 0..n {
            a[i * n + i] += shift;
        }
    }

    // Check convergence: off-diagonal elements should be near zero
    // relative to the matrix scale (Frobenius norm of diagonal).
    let diag_scale: f64 = (0..n)
        .map(|i| a[i * n + i] * a[i * n + i])
        .sum::<f64>()
        .sqrt()
        .max(1.0);
    let mut max_off = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            if i != j {
                max_off = max_off.max(a[i * n + j].abs());
            }
        }
    }
    if max_off > 1e-6 * diag_scale {
        return vec![f64::NAN; n];
    }

    (0..n).map(|i| a[i * n + i]).collect()
}

fn eigenvalues_2x2(m: &Matrix) -> Vec<f64> {
    let a = m.get(0, 0);
    let b = m.get(0, 1);
    let c = m.get(1, 0);
    let d = m.get(1, 1);
    let trace = a + d;
    let det = a * d - b * c;
    let disc = (trace * trace - 4.0 * det).max(0.0).sqrt();
    vec![(trace + disc) / 2.0, (trace - disc) / 2.0]
}

fn wilkinson_shift(a: f64, b: f64, d: f64) -> f64 {
    let delta = (a - d) / 2.0;
    if delta.abs() < 1e-15 {
        d - b.abs()
    } else {
        d - b * b / (delta + delta.signum() * (delta * delta + b * b).sqrt())
    }
}

fn qr_givens(n: usize, a: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut r = a.to_vec();
    let mut q = {
        let mut id = vec![0.0; n * n];
        for i in 0..n {
            id[i * n + i] = 1.0;
        }
        id
    };

    for j in 0..n.saturating_sub(1) {
        for i in (j + 1..n).rev() {
            let a_val = r[(i - 1) * n + j];
            let b_val = r[i * n + j];
            let rr = (a_val * a_val + b_val * b_val).sqrt();
            if rr < 1e-15 {
                continue;
            }
            let cos = a_val / rr;
            let sin = -b_val / rr;

            // Apply Givens to R (rows i-1, i)
            for k in 0..n {
                let ri1 = r[(i - 1) * n + k];
                let ri = r[i * n + k];
                r[(i - 1) * n + k] = cos * ri1 - sin * ri;
                r[i * n + k] = sin * ri1 + cos * ri;
            }

            // Accumulate Q^T
            for k in 0..n {
                let qi1 = q[k * n + (i - 1)];
                let qi = q[k * n + i];
                q[k * n + (i - 1)] = cos * qi1 - sin * qi;
                q[k * n + i] = sin * qi1 + cos * qi;
            }
        }
    }

    (q, r)
}

fn mat_multiply(n: usize, a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut c = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                c[i * n + j] += a[i * n + k] * b[k * n + j];
            }
        }
    }
    c
}
