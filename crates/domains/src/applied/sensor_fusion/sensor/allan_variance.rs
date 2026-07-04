#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

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

/// Representative white-noise (N) coefficient for a consumer-grade MEMS
/// accelerometer, in g/√Hz (i.e. ~100 µg/√Hz), before conversion to m/s²/√Hz
/// via standard gravity.
///
/// White noise is the σ_y ∝ τ^{-1/2} slope identified by Allan-variance
/// analysis. Value is a representative consumer-grade MEMS figure — an
/// order-of-magnitude, not a specific product datasheet.
///
/// Noise-parameter definitions and the Allan-variance noise-identification test
/// procedure: IEEE Std 952-1997, "IEEE Standard Specification Format Guide and
/// Test Procedure for Single-Axis Interferometric Fiber Optic Gyros" (the
/// standard reference for inertial-sensor Allan-variance noise terms).
const MEMS_ACCEL_WHITE_NOISE_G_PER_SQRT_HZ: f64 = 100e-6;

/// Representative bias-instability (B) coefficient for a consumer-grade MEMS
/// accelerometer, in g (i.e. ~50 µg), before conversion to m/s² via standard
/// gravity.
///
/// Bias instability is the flat (σ_y ∝ τ^0) floor of the Allan-deviation curve.
/// Value is a representative consumer-grade MEMS figure, not a specific product
/// datasheet. Parameter definition per IEEE Std 952-1997 (see above).
const MEMS_ACCEL_BIAS_INSTABILITY_G: f64 = 50e-6;

/// Representative white-noise / angle-random-walk (N) coefficient for a
/// consumer/tactical-grade MEMS gyroscope, in °/s/√Hz (i.e. ~0.01 °/s/√Hz).
///
/// The σ_y ∝ τ^{-1/2} white-noise term. Value is a representative MEMS figure,
/// not a specific product datasheet. Parameter definition per IEEE Std 952-1997
/// (see above).
const MEMS_GYRO_WHITE_NOISE_DEG_PER_S_PER_SQRT_HZ: f64 = 0.01;

/// Representative bias-instability (B) coefficient for a consumer/tactical-grade
/// MEMS gyroscope, in °/hr (i.e. ~1 °/hr).
///
/// Bias instability is the flat (σ_y ∝ τ^0) floor of the Allan-deviation curve.
/// Value is a representative MEMS figure, not a specific product datasheet.
/// Parameter definition per IEEE Std 952-1997 (see above).
const MEMS_GYRO_BIAS_INSTABILITY_DEG_PER_HR: f64 = 1.0;

/// Seconds per hour — unit conversion expressing the gyroscope bias-instability
/// figure (°/hr) as a per-second rate (°/s).
const SECONDS_PER_HOUR: f64 = 3600.0;

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
            + b2 * 2.0 * 2.0_f64.ln() / core::f64::consts::PI
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
            white_noise: MEMS_ACCEL_WHITE_NOISE_G_PER_SQRT_HZ
                * crate::formal::math::quantity::constants::standard_gravity().value,
            random_walk: 0.0,
            bias_instability: MEMS_ACCEL_BIAS_INSTABILITY_G
                * crate::formal::math::quantity::constants::standard_gravity().value,
            rate_ramp: 0.0,
            quantization: 0.0,
        }
    }

    /// Typical MEMS gyroscope noise profile.
    pub fn mems_gyroscope() -> Self {
        Self {
            white_noise: MEMS_GYRO_WHITE_NOISE_DEG_PER_S_PER_SQRT_HZ.to_radians(),
            random_walk: 0.0,
            bias_instability: MEMS_GYRO_BIAS_INSTABILITY_DEG_PER_HR.to_radians() / SECONDS_PER_HOUR,
            rate_ramp: 0.0,
            quantization: 0.0,
        }
    }
}
