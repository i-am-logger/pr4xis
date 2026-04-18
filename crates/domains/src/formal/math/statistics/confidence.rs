#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Confidence intervals.
///
/// Fisher, R.A. (1925). "Theory of Statistical Estimation."
/// Neyman, J. (1937). "Outline of a Theory of Statistical Estimation."
///
/// A confidence interval [θ̂ - z*σ, θ̂ + z*σ] contains the true parameter
/// with probability equal to the confidence level.
/// Z-scores for common confidence levels.
pub const Z_90: f64 = 1.645;
pub const Z_95: f64 = 1.960;
pub const Z_99: f64 = 2.576;

/// A confidence interval with lower and upper bounds.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfidenceInterval {
    /// Lower bound of the interval.
    pub lower: f64,
    /// Upper bound of the interval.
    pub upper: f64,
    /// Confidence level (e.g. 0.95 for 95%).
    pub confidence_level: f64,
}

impl ConfidenceInterval {
    /// Construct a confidence interval.
    pub fn new(lower: f64, upper: f64, confidence_level: f64) -> Self {
        Self {
            lower,
            upper,
            confidence_level,
        }
    }

    /// Width of the confidence interval.
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }

    /// Whether a value falls within the confidence interval.
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }

    /// Center of the interval (point estimate).
    pub fn center(&self) -> f64 {
        (self.lower + self.upper) / 2.0
    }
}

/// Compute a confidence interval for a mean given:
/// - `estimate`: sample mean θ̂
/// - `std_error`: standard error of the mean (σ/√n)
/// - `z`: z-score for the desired confidence level
/// - `confidence_level`: the confidence level (e.g. 0.95)
///
/// Interval: [θ̂ - z*SE, θ̂ + z*SE]
pub fn confidence_interval_for_mean(
    estimate: f64,
    std_error: f64,
    z: f64,
    confidence_level: f64,
) -> ConfidenceInterval {
    let margin = z * std_error;
    ConfidenceInterval::new(estimate - margin, estimate + margin, confidence_level)
}

/// Get the z-score for a standard confidence level.
///
/// Returns None for unsupported levels.
pub fn z_score_for_level(confidence_level: f64) -> Option<f64> {
    if (confidence_level - 0.90).abs() < 0.005 {
        Some(Z_90)
    } else if (confidence_level - 0.95).abs() < 0.005 {
        Some(Z_95)
    } else if (confidence_level - 0.99).abs() < 0.005 {
        Some(Z_99)
    } else {
        None
    }
}
