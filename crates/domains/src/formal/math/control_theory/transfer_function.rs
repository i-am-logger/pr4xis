#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

/// Transfer functions: G(s) = Y(s) / U(s).
///
/// Ogata (2010). *Modern Control Engineering* (5th ed.).
///
/// A transfer function is the Laplace-domain representation of a linear
/// time-invariant system. It is a ratio of polynomials in s:
/// G(s) = (b_m*s^m + ... + b_1*s + b_0) / (a_n*s^n + ... + a_1*s + a_0)
///
/// - Zeros: roots of the numerator polynomial
/// - Poles: roots of the denominator polynomial
/// - System order: degree of the denominator
///
/// A transfer function G(s) = numerator(s) / denominator(s).
///
/// Coefficients are stored in ascending order: [a_0, a_1, ..., a_n]
/// represents a_0 + a_1*s + a_2*s^2 + ... + a_n*s^n.
#[derive(Debug, Clone, PartialEq)]
pub struct TransferFunction {
    /// Numerator polynomial coefficients (ascending order).
    pub numerator: Vec<f64>,
    /// Denominator polynomial coefficients (ascending order).
    pub denominator: Vec<f64>,
}

impl TransferFunction {
    /// Create a new transfer function.
    pub fn new(numerator: Vec<f64>, denominator: Vec<f64>) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// System order: degree of the denominator polynomial.
    pub fn order(&self) -> usize {
        if self.denominator.is_empty() {
            return 0;
        }
        self.denominator.len() - 1
    }

    /// Number of zeros: degree of the numerator polynomial.
    pub fn num_zeros(&self) -> usize {
        if self.numerator.is_empty() {
            return 0;
        }
        self.numerator.len() - 1
    }

    /// Whether the system is proper (order of numerator <= order of denominator).
    pub fn is_proper(&self) -> bool {
        self.num_zeros() <= self.order()
    }

    /// Whether the system is strictly proper (order of numerator < order of denominator).
    pub fn is_strictly_proper(&self) -> bool {
        self.num_zeros() < self.order()
    }

    /// DC gain: G(0) = numerator(0) / denominator(0) = b_0 / a_0.
    pub fn dc_gain(&self) -> Option<f64> {
        let num_0 = self.numerator.first().copied().unwrap_or(0.0);
        let den_0 = self.denominator.first().copied().unwrap_or(0.0);
        if den_0.abs() < 1e-15 {
            return None;
        }
        Some(num_0 / den_0)
    }
}

/// Find the real root of a first-order polynomial a_0 + a_1*s = 0.
///
/// Returns s = -a_0 / a_1.
pub fn first_order_pole(a0: f64, a1: f64) -> Option<f64> {
    if a1.abs() < 1e-15 {
        return None;
    }
    Some(-a0 / a1)
}

/// Create a first-order system G(s) = K / (τs + 1).
pub fn first_order_system(gain: f64, time_constant: f64) -> TransferFunction {
    TransferFunction::new(vec![gain], vec![1.0, time_constant])
}

/// Create a second-order system G(s) = ωn² / (s² + 2ζωn*s + ωn²).
pub fn second_order_system(natural_freq: f64, damping_ratio: f64) -> TransferFunction {
    let wn2 = natural_freq * natural_freq;
    TransferFunction::new(
        vec![wn2],
        vec![wn2, 2.0 * damping_ratio * natural_freq, 1.0],
    )
}
