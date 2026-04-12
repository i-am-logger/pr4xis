use crate::formal::math::linear_algebra::vector_space::Vector;

/// Dense matrix over R stored in row-major order.
///
/// Matrix algebra axioms (Horn & Johnson, *Matrix Analysis*):
/// - Associativity: (AB)C = A(BC)
/// - Distributivity: A(B+C) = AB + AC
/// - Identity: AI = IA = A
/// - Transpose of transpose: (A^T)^T = A
/// - Transpose of product: (AB)^T = B^T A^T
/// - Trace is linear: tr(A+B) = tr(A) + tr(B)
/// - Trace of product is commutative: tr(AB) = tr(BA)
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl Matrix {
    /// Create from row-major data.
    pub fn new(rows: usize, cols: usize, data: Vec<f64>) -> Self {
        assert_eq!(data.len(), rows * cols);
        Self { rows, cols, data }
    }

    /// Zero matrix.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    /// Identity matrix.
    pub fn identity(n: usize) -> Self {
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            data[i * n + i] = 1.0;
        }
        Self {
            rows: n,
            cols: n,
            data,
        }
    }

    /// Create from diagonal elements.
    pub fn diagonal(diag: &[f64]) -> Self {
        let n = diag.len();
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            data[i * n + i] = diag[i];
        }
        Self {
            rows: n,
            cols: n,
            data,
        }
    }

    /// Element access (row, col).
    pub fn get(&self, i: usize, j: usize) -> f64 {
        self.data[i * self.cols + j]
    }

    /// Mutable element access.
    pub fn set(&mut self, i: usize, j: usize, val: f64) {
        self.data[i * self.cols + j] = val;
    }

    /// Is square?
    pub fn is_square(&self) -> bool {
        self.rows == self.cols
    }

    /// Matrix addition.
    pub fn add(&self, other: &Self) -> Self {
        assert_eq!(self.rows, other.rows);
        assert_eq!(self.cols, other.cols);
        Self {
            rows: self.rows,
            cols: self.cols,
            data: self
                .data
                .iter()
                .zip(&other.data)
                .map(|(a, b)| a + b)
                .collect(),
        }
    }

    /// Matrix subtraction.
    pub fn sub(&self, other: &Self) -> Self {
        assert_eq!(self.rows, other.rows);
        assert_eq!(self.cols, other.cols);
        Self {
            rows: self.rows,
            cols: self.cols,
            data: self
                .data
                .iter()
                .zip(&other.data)
                .map(|(a, b)| a - b)
                .collect(),
        }
    }

    /// Scalar multiplication.
    pub fn scale(&self, s: f64) -> Self {
        Self {
            rows: self.rows,
            cols: self.cols,
            data: self.data.iter().map(|x| x * s).collect(),
        }
    }

    /// Matrix multiplication: C = A * B where A is m×n, B is n×p.
    pub fn multiply(&self, other: &Self) -> Self {
        assert_eq!(self.cols, other.rows);
        let m = self.rows;
        let n = self.cols;
        let p = other.cols;
        let mut data = vec![0.0; m * p];
        for i in 0..m {
            for j in 0..p {
                let mut sum = 0.0;
                for k in 0..n {
                    sum += self.get(i, k) * other.get(k, j);
                }
                data[i * p + j] = sum;
            }
        }
        Self {
            rows: m,
            cols: p,
            data,
        }
    }

    /// Matrix-vector multiply: y = A * x.
    pub fn multiply_vector(&self, x: &Vector) -> Vector {
        assert_eq!(self.cols, x.dim());
        let mut result = vec![0.0; self.rows];
        for (i, ri) in result.iter_mut().enumerate() {
            for j in 0..self.cols {
                *ri += self.get(i, j) * x.get(j);
            }
        }
        Vector::new(result)
    }

    /// Transpose: (A^T)_{ij} = A_{ji}.
    pub fn transpose(&self) -> Self {
        let mut data = vec![0.0; self.rows * self.cols];
        for i in 0..self.rows {
            for j in 0..self.cols {
                data[j * self.rows + i] = self.get(i, j);
            }
        }
        Self {
            rows: self.cols,
            cols: self.rows,
            data,
        }
    }

    /// Trace: tr(A) = Σ a_{ii}. Only for square matrices.
    pub fn trace(&self) -> f64 {
        assert!(self.is_square());
        let n = self.rows;
        (0..n).map(|i| self.get(i, i)).sum()
    }

    /// Is symmetric? A = A^T within tolerance.
    pub fn is_symmetric(&self, tol: f64) -> bool {
        if !self.is_square() {
            return false;
        }
        for i in 0..self.rows {
            for j in (i + 1)..self.cols {
                if (self.get(i, j) - self.get(j, i)).abs() > tol {
                    return false;
                }
            }
        }
        true
    }

    /// Frobenius norm: ||A||_F = sqrt(Σ a_{ij}²).
    pub fn frobenius_norm(&self) -> f64 {
        self.data.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    /// Extract column as Vector.
    pub fn column(&self, j: usize) -> Vector {
        Vector::new((0..self.rows).map(|i| self.get(i, j)).collect())
    }

    /// Extract row as Vector.
    pub fn row(&self, i: usize) -> Vector {
        Vector::new((0..self.cols).map(|j| self.get(i, j)).collect())
    }
}

/// Approximate equality for floating-point matrices.
pub fn approx_eq(a: &Matrix, b: &Matrix, tol: f64) -> bool {
    if a.rows != b.rows || a.cols != b.cols {
        return false;
    }
    a.data.iter().zip(&b.data).all(|(x, y)| (x - y).abs() < tol)
}
