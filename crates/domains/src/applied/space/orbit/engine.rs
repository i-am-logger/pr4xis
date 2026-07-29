#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::applied::space::orbit::propagator::{OrbitalState, mu_earth_km3s2, propagate_rk4};
use crate::formal::math::angle::Angle;
use crate::formal::math::geometry::point::Point3;
use crate::formal::math::quantity::value::Quantity;

/// Orbit determination from radar observations.
///
/// Simplified initial orbit determination using range and range-rate.
#[derive(Debug, Clone)]
pub struct RadarObservation {
    /// Range to object, tagged `Dimension::LENGTH` (km convention, matching
    /// `OrbitalState.position`).
    pub range: Quantity,
    /// Range rate, tagged `Dimension::VELOCITY` (km/s convention, matching
    /// `OrbitalState.velocity`).
    pub range_rate: Quantity,
    /// Azimuth angle.
    pub azimuth: Angle,
    /// Elevation angle.
    pub elevation: Angle,
}

/// Convert radar observation to a position [`Point3`] in the Earth-centred
/// inertial (ECI) frame (simplified, assuming station at origin).
pub fn radar_to_eci(obs: &RadarObservation) -> Point3 {
    let cos_el = obs.elevation.cos();
    let sin_el = obs.elevation.sin();
    let cos_az = obs.azimuth.cos();
    let sin_az = obs.azimuth.sin();
    let range = obs.range.value;
    Point3::new(
        range * cos_el * cos_az,
        range * cos_el * sin_az,
        range * sin_el,
    )
}

/// Propagate orbit forward by a given number of steps.
pub fn propagate_orbit(initial: &OrbitalState, dt: f64, steps: usize) -> Vec<OrbitalState> {
    let mut trajectory = Vec::with_capacity(steps + 1);
    trajectory.push(initial.clone());
    let mut current = initial.clone();
    for _ in 0..steps {
        current = propagate_rk4(&current, dt, mu_earth_km3s2().value);
        trajectory.push(current.clone());
    }
    trajectory
}

/// Check if an orbital state represents a bound (elliptical) orbit.
pub fn is_bound_orbit(state: &OrbitalState) -> bool {
    state.specific_energy(&mu_earth_km3s2()).value < 0.0
}
