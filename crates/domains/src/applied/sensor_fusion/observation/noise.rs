use crate::formal::math::linear_algebra::matrix::Matrix;

/// Noise model: characterizes the statistical properties of measurement error.
///
/// v_k ~ N(0, R) for Gaussian noise.
///
/// Source: Maybeck (1979), Vol. 1, Chapter 5.
#[derive(Debug, Clone)]
pub struct GaussianNoise {
    /// Noise covariance matrix R (must be symmetric PSD).
    pub covariance: Matrix,
}

impl GaussianNoise {
    pub fn new(covariance: Matrix) -> Self {
        assert!(covariance.is_symmetric(1e-10));
        Self { covariance }
    }

    /// Isotropic noise with the same variance on all axes.
    pub fn isotropic(dim: usize, variance: f64) -> Self {
        Self {
            covariance: Matrix::identity(dim).scale(variance),
        }
    }

    /// Dimension.
    pub fn dim(&self) -> usize {
        self.covariance.rows
    }
}
