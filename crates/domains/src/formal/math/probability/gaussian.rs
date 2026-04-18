#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::decomposition;
use crate::formal::math::linear_algebra::determinant;
use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use core::f64::consts::PI;

/// Univariate Gaussian (normal) distribution.
///
/// p(x) = (1 / √(2πσ²)) exp(-(x-μ)² / (2σ²))
///
/// Maximum entropy distribution for given mean and variance.
///
/// Source: Kolmogorov (1933), Fisher (1925).
#[derive(Debug, Clone, PartialEq)]
pub struct Gaussian1D {
    pub mean: f64,
    pub variance: f64,
}

impl Gaussian1D {
    pub fn new(mean: f64, variance: f64) -> Self {
        assert!(variance > 0.0, "variance must be positive");
        Self { mean, variance }
    }

    pub fn standard() -> Self {
        Self {
            mean: 0.0,
            variance: 1.0,
        }
    }

    pub fn std_dev(&self) -> f64 {
        self.variance.sqrt()
    }

    /// Probability density function.
    pub fn pdf(&self, x: f64) -> f64 {
        let z = (x - self.mean) / self.std_dev();
        (1.0 / (2.0 * PI * self.variance).sqrt()) * (-0.5 * z * z).exp()
    }

    /// Log probability density.
    pub fn log_pdf(&self, x: f64) -> f64 {
        let z = (x - self.mean) / self.std_dev();
        -0.5 * (2.0 * PI * self.variance).ln() - 0.5 * z * z
    }

    /// Standardized value (z-score): z = (x - μ) / σ.
    pub fn standardize(&self, x: f64) -> f64 {
        (x - self.mean) / self.std_dev()
    }

    /// Product of two Gaussians (fusion of two independent estimates).
    ///
    /// This IS the Kalman filter update for scalar case:
    /// P(x|z1,z2) ∝ P(z1|x) * P(z2|x) → Gaussian with
    /// μ_fused = (μ1/σ1² + μ2/σ2²) / (1/σ1² + 1/σ2²)
    /// σ²_fused = 1 / (1/σ1² + 1/σ2²)
    pub fn fuse(&self, other: &Self) -> Self {
        let inv_var1 = 1.0 / self.variance;
        let inv_var2 = 1.0 / other.variance;
        let fused_inv_var = inv_var1 + inv_var2;
        let fused_var = 1.0 / fused_inv_var;
        let fused_mean = (self.mean * inv_var1 + other.mean * inv_var2) / fused_inv_var;
        Self {
            mean: fused_mean,
            variance: fused_var,
        }
    }
}

/// Multivariate Gaussian distribution.
///
/// p(x) = (2π)^{-n/2} |P|^{-1/2} exp(-0.5 (x-μ)^T P^{-1} (x-μ))
///
/// Central to estimation theory: the state estimate IS a Gaussian
/// characterized by mean (state vector x̂) and covariance (P).
///
/// Source: Maybeck, *Stochastic Models, Estimation, and Control* (1979).
#[derive(Debug, Clone)]
pub struct GaussianND {
    pub mean: Vector,
    pub covariance: Matrix,
}

impl GaussianND {
    pub fn new(mean: Vector, covariance: Matrix) -> Self {
        assert_eq!(mean.dim(), covariance.rows);
        assert!(covariance.is_symmetric(1e-10));
        Self { mean, covariance }
    }

    pub fn dim(&self) -> usize {
        self.mean.dim()
    }

    /// Log probability density (up to normalizing constant).
    /// log p(x) = -0.5 * (n*ln(2π) + ln|P| + (x-μ)^T P^{-1} (x-μ))
    pub fn log_pdf(&self, x: &Vector) -> Option<f64> {
        let n = self.dim() as f64;
        let diff = x.sub(&self.mean);
        let p_inv_diff = decomposition::solve_spd(&self.covariance, &diff.data)?;
        let mahal_sq: f64 = diff.data.iter().zip(&p_inv_diff).map(|(a, b)| a * b).sum();
        let log_det = determinant::det(&self.covariance).ln();
        Some(-0.5 * (n * (2.0 * PI).ln() + log_det + mahal_sq))
    }

    /// Fuse two independent Gaussian estimates (information form).
    ///
    /// This is the Kalman filter measurement update generalized:
    /// Y_fused = Y1 + Y2  (information matrices add)
    /// y_fused = y1 + y2  (information vectors add)
    /// where Y = P^{-1}, y = P^{-1} μ
    pub fn fuse(&self, other: &Self) -> Option<Self> {
        let y1 = decomposition::inverse_spd(&self.covariance)?;
        let y2 = decomposition::inverse_spd(&other.covariance)?;
        let y_fused = y1.add(&y2);
        let yv1 = y1.multiply_vector(&self.mean);
        let yv2 = y2.multiply_vector(&other.mean);
        let yv_fused = yv1.add(&yv2);
        let p_fused = decomposition::inverse_spd(&y_fused)?;
        let mean_fused = p_fused.multiply_vector(&yv_fused);
        Some(Self {
            mean: mean_fused,
            covariance: p_fused,
        })
    }
}
