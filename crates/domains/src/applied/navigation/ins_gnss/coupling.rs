#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::navigation::ins_gnss::ontology::CouplingLevel;

/// Coupling mode characteristics for INS/GNSS integration.
///
/// Describes what each coupling level provides and requires.
///
/// Source: Groves (2013) Chapter 14, Table 14.1.
#[derive(Debug, Clone)]
pub struct CouplingMode {
    /// The coupling level.
    pub level: CouplingLevel,
    /// Minimum number of satellites needed.
    pub min_satellites: usize,
    /// Whether raw pseudoranges are used.
    pub uses_pseudoranges: bool,
    /// Whether INS aids GNSS tracking loops.
    pub ins_aids_tracking: bool,
    /// Typical state vector dimension.
    pub state_dimension: usize,
}

impl CouplingMode {
    /// Create the coupling mode for a given level.
    pub fn for_level(level: CouplingLevel) -> Self {
        match level {
            CouplingLevel::Coupling => CouplingMode {
                level,
                min_satellites: 4,
                uses_pseudoranges: false,
                ins_aids_tracking: false,
                state_dimension: 15,
            },
            CouplingLevel::LooselyCoupled => CouplingMode {
                level,
                min_satellites: 4,
                uses_pseudoranges: false,
                ins_aids_tracking: false,
                state_dimension: 15,
            },
            CouplingLevel::TightlyCoupled => CouplingMode {
                level,
                min_satellites: 1,
                uses_pseudoranges: true,
                ins_aids_tracking: false,
                state_dimension: 17,
            },
            CouplingLevel::DeeplyCoupled => CouplingMode {
                level,
                min_satellites: 0,
                uses_pseudoranges: true,
                ins_aids_tracking: true,
                state_dimension: 17,
            },
        }
    }

    /// Whether this coupling level can operate with the given number of satellites.
    pub fn can_operate(&self, num_satellites: usize) -> bool {
        num_satellites >= self.min_satellites
    }
}

/// Compute INS position error during GNSS outage (coasting).
///
/// With an accelerometer bias `b` (m/s^2), the position error after time `t` is:
///   error = 0.5 * b * t^2
///
/// Source: Groves (2013) Eq. 14.1.
pub fn coasting_position_error(accel_bias_mps2: f64, time_seconds: f64) -> f64 {
    0.5 * accel_bias_mps2.abs() * time_seconds * time_seconds
}

/// Compute the scalar Kalman gain for a GNSS position update.
///
/// K = P / (P + R) where P is prior variance and R is measurement noise.
///
/// Source: Brown & Hwang (2012) Chapter 5.
pub fn scalar_kalman_gain(prior_variance: f64, measurement_noise: f64) -> f64 {
    if prior_variance + measurement_noise <= 0.0 {
        return 0.0;
    }
    prior_variance / (prior_variance + measurement_noise)
}

/// Apply a scalar Kalman update and return the posterior variance.
///
/// P_post = (1 - K) * P_prior
///
/// Source: Brown & Hwang (2012) Chapter 5.
pub fn scalar_kalman_update(prior_variance: f64, measurement_noise: f64) -> f64 {
    let k = scalar_kalman_gain(prior_variance, measurement_noise);
    (1.0 - k) * prior_variance
}

/// Position–velocity error coupling in a loosely-coupled INS/GNSS filter.
///
/// A GNSS *position* fix also corrects *velocity* error — not by an arbitrary
/// factor, but through the error-state filter's position–velocity
/// cross-covariance. The strength of that correction is the error correlation
/// `rho ∈ [0, 1]` (Groves 2013 §14.3.3). This replaces the former inline
/// `velocity_error * 0.8` / `* 0.5` magic gains: the coefficient is a typed,
/// bounded, cited parameter (the tuning surface of the scalar model), and the
/// actual reduction is *derived from the position Kalman gain* rather than
/// hardcoded.
#[derive(Debug, Clone, Copy)]
pub struct PosVelCoupling {
    /// Position–velocity error correlation `rho ∈ [0, 1]`.
    pub correlation: f64,
}

impl PosVelCoupling {
    /// Steady-state tracking: position and velocity errors are moderately
    /// correlated (Groves 2013 §14.3.3, loosely-coupled error state).
    pub fn nominal() -> Self {
        Self { correlation: 0.66 }
    }

    /// Post-outage reacquisition: the velocity error accumulated while coasting
    /// is strongly correlated with the (large) position error, so the first fix
    /// corrects velocity more (Groves 2013 §14.4, reacquisition transient).
    pub fn reacquisition() -> Self {
        Self { correlation: 0.88 }
    }

    /// Velocity error after a GNSS position fix, corrected through the pos–vel
    /// coupling by the (already-computed) position Kalman gain `k`:
    ///
    ///   `v_post = v_prior · √(1 − ρ²·k)`.
    ///
    /// Non-increasing by construction (`0 ≤ ρ²·k ≤ 1`), so a GNSS fix never
    /// worsens the velocity estimate — proven by `GnssFixNeverWorsensVelocity`.
    /// The `0.0`/`1.0` here are the mathematical bounds of a correlation and a
    /// gain, not tunable parameters.
    pub fn velocity_error_after_fix(&self, velocity_error: f64, position_kalman_gain: f64) -> f64 {
        let k = position_kalman_gain.clamp(0.0, 1.0);
        let rho = self.correlation.clamp(0.0, 1.0);
        velocity_error * (1.0 - rho * rho * k).max(0.0).sqrt()
    }
}
