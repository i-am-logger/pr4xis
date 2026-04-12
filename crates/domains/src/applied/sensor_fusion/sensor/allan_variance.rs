/// Allan variance noise characterization — rich type carrying the full noise profile.
///
/// Each noise type has a characteristic slope on a log-log plot
/// of Allan deviation vs. averaging time τ.
///
/// Source: Riley, W.J. (2008). *Handbook of Frequency Stability Analysis*. NIST SP 1065.
///         Allan, D.W. (1966). "Statistics of Atomic Frequency Standards."
///         IEEE Std 1139-2008.
#[derive(Debug, Clone, PartialEq)]
pub struct AllanVarianceProfile {
    /// White noise coefficient (σ_y ∝ τ^{-1/2}). Units: sensor-specific / √Hz.
    pub white_noise: f64,
    /// Random walk coefficient (σ_y ∝ τ^{1/2}). Units: sensor-specific · √s.
    pub random_walk: f64,
    /// Bias instability (σ_y ∝ τ^0, flat region). Units: sensor-specific.
    pub bias_instability: f64,
    /// Rate ramp (σ_y ∝ τ). Units: sensor-specific / s.
    pub rate_ramp: f64,
    /// Quantization noise (σ_y ∝ τ^{-1}). Units: sensor-specific · s.
    pub quantization: f64,
}

impl AllanVarianceProfile {
    /// Ideal sensor (no noise).
    pub fn ideal() -> Self {
        Self {
            white_noise: 0.0,
            random_walk: 0.0,
            bias_instability: 0.0,
            rate_ramp: 0.0,
            quantization: 0.0,
        }
    }

    /// Allan variance at averaging time τ (full 5-term model).
    ///
    /// σ²_y(τ) = 3Q²/τ² + N²/τ + B²·(2ln2/π) + K²τ/3 + R²τ²/2
    ///
    /// where Q=quantization, N=white_noise, B=bias_instability,
    /// K=random_walk, R=rate_ramp.
    pub fn variance_at(&self, tau: f64) -> f64 {
        let q2 = self.quantization * self.quantization;
        let n2 = self.white_noise * self.white_noise;
        let b2 = self.bias_instability * self.bias_instability;
        let k2 = self.random_walk * self.random_walk;
        let r2 = self.rate_ramp * self.rate_ramp;

        3.0 * q2 / (tau * tau)
            + n2 / tau
            + b2 * 2.0 * 2.0_f64.ln() / std::f64::consts::PI
            + k2 * tau / 3.0
            + r2 * tau * tau / 2.0
    }

    /// Allan deviation at averaging time τ.
    pub fn deviation_at(&self, tau: f64) -> f64 {
        self.variance_at(tau).sqrt()
    }

    /// Typical MEMS accelerometer noise profile.
    pub fn mems_accelerometer() -> Self {
        Self {
            white_noise: 100e-6
                * crate::formal::math::quantity::constants::standard_gravity().value, // 100 µg/√Hz
            random_walk: 0.0,
            bias_instability: 50e-6
                * crate::formal::math::quantity::constants::standard_gravity().value, // 50 µg
            rate_ramp: 0.0,
            quantization: 0.0,
        }
    }

    /// Typical MEMS gyroscope noise profile.
    pub fn mems_gyroscope() -> Self {
        Self {
            white_noise: 0.01_f64.to_radians(), // 0.01 °/s/√Hz
            random_walk: 0.0,
            bias_instability: 1.0_f64.to_radians() / 3600.0, // 1 °/hr
            rate_ramp: 0.0,
            quantization: 0.0,
        }
    }
}
