use crate::formal::math::temporal::duration::Duration;

/// Clock error model.
///
/// A real clock deviates from ideal time. The error model captures:
/// - Bias (constant offset)
/// - Drift (rate error, ppm)
/// - Jitter (random variation per tick)
///
/// Clock error at time t: ε(t) = bias + drift * t + noise(t)
///
/// These parameters are characterized by Allan variance analysis.
///
/// Sources:
///   - Riley, W.J. "Handbook of Frequency Stability Analysis" (NIST SP 1065, 2008)
///   - Allan, D.W. "Statistics of Atomic Frequency Standards" (1966)
///   - IEEE Std 1139-2008 "Standard Definitions of Physical Quantities for
///     Fundamental Frequency and Time Metrology"
#[derive(Debug, Clone, PartialEq)]
pub struct ClockModel {
    /// Constant time offset (seconds).
    pub bias: f64,
    /// Frequency drift (seconds per second, dimensionless).
    pub drift: f64,
    /// White noise spectral density (seconds / √Hz).
    pub white_noise_density: f64,
    /// Random walk coefficient (seconds / √s).
    pub random_walk: f64,
}

impl ClockModel {
    /// Ideal clock (no errors).
    pub fn ideal() -> Self {
        Self {
            bias: 0.0,
            drift: 0.0,
            white_noise_density: 0.0,
            random_walk: 0.0,
        }
    }

    /// Clock error at elapsed time t (deterministic component only).
    pub fn error_at(&self, elapsed: &Duration) -> f64 {
        let t = elapsed.seconds();
        self.bias + self.drift * t
    }

    /// Allan variance at averaging time τ (simplified two-term model).
    ///
    /// σ²_y(τ) = (3 * Q² / τ) + (N² * τ / 3)
    ///
    /// where Q = white noise density, N = random walk coefficient.
    pub fn allan_variance(&self, tau: f64) -> f64 {
        let q2 = self.white_noise_density * self.white_noise_density;
        let n2 = self.random_walk * self.random_walk;
        3.0 * q2 / tau + n2 * tau / 3.0
    }

    /// Allan deviation at averaging time τ.
    pub fn allan_deviation(&self, tau: f64) -> f64 {
        self.allan_variance(tau).sqrt()
    }
}

/// Noise types identified through Allan variance analysis.
///
/// Each noise type has a characteristic slope on a log-log plot
/// of Allan deviation vs. averaging time.
///
/// Source: Riley (2008), Table 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClockNoiseType {
    /// White phase noise: σ_y(τ) ∝ τ^{-1}. Slope = -1.
    WhitePhase,
    /// Flicker phase noise: σ_y(τ) ∝ τ^{-1}. Slope = -1.
    FlickerPhase,
    /// White frequency noise: σ_y(τ) ∝ τ^{-1/2}. Slope = -1/2.
    WhiteFrequency,
    /// Flicker frequency noise: σ_y(τ) ∝ τ^0. Slope = 0 (flat).
    FlickerFrequency,
    /// Random walk frequency: σ_y(τ) ∝ τ^{1/2}. Slope = +1/2.
    RandomWalkFrequency,
}
