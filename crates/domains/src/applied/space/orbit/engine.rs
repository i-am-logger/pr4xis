#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::space::orbit::propagator::{OrbitalState, mu_earth_km3s2, propagate_rk4};

/// Orbit determination from radar observations.
///
/// Simplified initial orbit determination using range and range-rate.
#[derive(Debug, Clone)]
pub struct RadarObservation {
    /// Range to object (km).
    pub range: f64,
    /// Range rate (km/s).
    pub range_rate: f64,
    /// Azimuth angle (rad).
    pub azimuth: f64,
    /// Elevation angle (rad).
    pub elevation: f64,
}

/// Convert radar observation to position in ECI (simplified, assuming station at origin).
pub fn radar_to_eci(obs: &RadarObservation) -> [f64; 3] {
    let cos_el = obs.elevation.cos();
    let sin_el = obs.elevation.sin();
    let cos_az = obs.azimuth.cos();
    let sin_az = obs.azimuth.sin();
    [
        obs.range * cos_el * cos_az,
        obs.range * cos_el * sin_az,
        obs.range * sin_el,
    ]
}

/// Propagate orbit forward by a given number of steps.
pub fn propagate_orbit(initial: &OrbitalState, dt: f64, steps: usize) -> Vec<OrbitalState> {
    let mut trajectory = Vec::with_capacity(steps + 1);
    trajectory.push(initial.clone());
    let mut current = initial.clone();
    for _ in 0..steps {
        current = propagate_rk4(&current, dt, mu_earth_km3s2());
        trajectory.push(current.clone());
    }
    trajectory
}

/// Check if an orbital state represents a bound (elliptical) orbit.
pub fn is_bound_orbit(state: &OrbitalState) -> bool {
    state.specific_energy(mu_earth_km3s2()) < 0.0
}
