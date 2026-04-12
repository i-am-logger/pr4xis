use crate::formal::math::linear_algebra::matrix::Matrix;

/// Compute the determinant of a square matrix.
///
/// Axiomatic characterization (unique function satisfying all three):
/// 1. Multilinear: linear in each column separately
/// 2. Alternating: swapping two columns negates the determinant
/// 3. Normalized: det(I) = 1
///
/// Properties that follow:
/// - det(AB) = det(A) * det(B)        (multiplicativity)
/// - det(A^T) = det(A)                (transpose invariance)
/// - det(A^{-1}) = 1/det(A)           (inverse)
/// - det(cA) = c^n * det(A)           (homogeneity)
/// - A invertible iff det(A) ≠ 0
///
/// Source: Horn & Johnson, *Matrix Analysis* (2013), Chapter 0.
pub fn det(m: &Matrix) -> f64 {
    assert!(m.is_square());
    let n = m.rows;
    match n {
        0 => 1.0,
        1 => m.get(0, 0),
        2 => m.get(0, 0) * m.get(1, 1) - m.get(0, 1) * m.get(1, 0),
        3 => {
            m.get(0, 0) * (m.get(1, 1) * m.get(2, 2) - m.get(1, 2) * m.get(2, 1))
                - m.get(0, 1) * (m.get(1, 0) * m.get(2, 2) - m.get(1, 2) * m.get(2, 0))
                + m.get(0, 2) * (m.get(1, 0) * m.get(2, 1) - m.get(1, 1) * m.get(2, 0))
        }
        _ => det_lu(m),
    }
}

/// Determinant via LU decomposition for n > 3.
/// Uses partial pivoting for numerical stability.
fn det_lu(m: &Matrix) -> f64 {
    let n = m.rows;
    let mut a = m.data.clone();
    let mut sign = 1.0_f64;

    for k in 0..n {
        // Partial pivoting: find max in column k below diagonal
        let mut max_val = a[k * n + k].abs();
        let mut max_row = k;
        for i in (k + 1)..n {
            let val = a[i * n + k].abs();
            if val > max_val {
                max_val = val;
                max_row = i;
            }
        }

        if max_val < 1e-15 {
            return 0.0; // singular
        }

        // Swap rows
        if max_row != k {
            for j in 0..n {
                a.swap(k * n + j, max_row * n + j);
            }
            sign = -sign;
        }

        // Eliminate below
        let pivot = a[k * n + k];
        for i in (k + 1)..n {
            let factor = a[i * n + k] / pivot;
            for j in (k + 1)..n {
                a[i * n + j] -= factor * a[k * n + j];
            }
            a[i * n + k] = 0.0;
        }
    }

    // Product of diagonal
    let mut result = sign;
    for i in 0..n {
        result *= a[i * n + i];
    }
    result
}
