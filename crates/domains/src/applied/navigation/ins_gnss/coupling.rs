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
