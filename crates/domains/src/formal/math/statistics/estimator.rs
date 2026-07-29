use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;

/// Statistical estimators and their properties.
///
/// Fisher, R.A. (1925). "Theory of Statistical Estimation."
///
/// An estimator θ̂ of a parameter θ has:
/// - Bias: E[θ̂] - θ
/// - Variance: E[(θ̂ - E[θ̂])²]
/// - Mean Squared Error: MSE = bias² + variance
///
/// Compute the sample mean of a slice.
///
/// x̄ = (1/n) Σ x_i
///
/// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), never a bare
/// `f64` — at this generic layer the underlying data has no declared unit
/// (a caller working in a physical domain interprets the value in whatever
/// concrete unit its own context declares), the same treatment as
/// `Point3::distance_to`.
pub fn sample_mean(data: &[f64]) -> Quantity {
    if data.is_empty() {
        return Quantity::from_unit(0.0, &unit::UNITLESS);
    }
    Quantity::from_unit(
        data.iter().sum::<f64>() / data.len() as f64,
        &unit::UNITLESS,
    )
}

/// Compute the sample variance with Bessel's correction.
///
/// s² = (1/(n-1)) Σ (x_i - x̄)²
///
/// Using n-1 (Bessel's correction) gives an unbiased estimator of population variance.
///
/// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), same reasoning
/// as [`sample_mean`].
pub fn sample_variance(data: &[f64]) -> Quantity {
    if data.len() < 2 {
        return Quantity::from_unit(0.0, &unit::UNITLESS);
    }
    let mean = sample_mean(data).value;
    let sum_sq: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum();
    Quantity::from_unit(sum_sq / (data.len() - 1) as f64, &unit::UNITLESS)
}

/// Compute the sample standard deviation (square root of sample variance).
///
/// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), same reasoning
/// as [`sample_variance`].
pub fn sample_std_dev(data: &[f64]) -> Quantity {
    Quantity::from_unit(sample_variance(data).value.sqrt(), &unit::UNITLESS)
}

/// Compute the bias of an estimator given estimated value and true value.
///
/// bias = θ̂ - θ
pub fn bias(estimated: f64, true_value: f64) -> f64 {
    estimated - true_value
}

/// Compute the mean squared error from bias and variance.
///
/// MSE = bias² + variance
///
/// This is the fundamental MSE decomposition:
/// E[(θ̂ - θ)²] = (E[θ̂] - θ)² + Var(θ̂)
pub fn mean_squared_error(bias: f64, variance: f64) -> f64 {
    bias * bias + variance
}

/// Compute MSE directly from data and true value.
///
/// MSE = (1/n) Σ (x_i - θ)²
pub fn mse_from_data(estimates: &[f64], true_value: f64) -> f64 {
    if estimates.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = estimates.iter().map(|&x| (x - true_value).powi(2)).sum();
    sum_sq / estimates.len() as f64
}

/// Standard error of the mean: SE = s / √n.
///
/// Returns a dimensionless [`Quantity`] (`unit::UNITLESS`), same reasoning
/// as [`sample_std_dev`].
pub fn standard_error(data: &[f64]) -> Quantity {
    if data.len() < 2 {
        return Quantity::from_unit(0.0, &unit::UNITLESS);
    }
    Quantity::from_unit(
        sample_std_dev(data).value / (data.len() as f64).sqrt(),
        &unit::UNITLESS,
    )
}
