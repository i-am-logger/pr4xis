#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::coordinate::PolarCoordinate;
use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::signal_processing::sampling;

/// Range magnitude (metres) below which the target is treated as co-located
/// with the sensor: bearing/azimuth is undefined and the observation Jacobian
/// is returned as zero.
///
/// This is a numerical singularity guard, not a physical range-resolution spec.
/// The azimuth partials in [`radar_jacobian_2d`] scale as `1/r²`, which diverges
/// as `r → 0`. The value is a small positive epsilon (0.1 nm) — orders of
/// magnitude below any physical radar range — so it never triggers on a real
/// detection, yet keeps the `1/r²` terms finite at the coordinate origin.
///
/// Rationale: standard EKF practice guards the polar-measurement singularity at
/// the origin; see Bar-Shalom et al. (2001), *Estimation with Applications to
/// Tracking and Navigation*, Ch. 10 (converted / mixed-coordinate measurements).
const ZERO_RANGE_EPS_M: f64 = 1e-10;

/// Radar observation model: maps Cartesian state to polar measurement.
///
/// State: [x, vx, y, vy] (2D constant velocity).
/// Measurement: [range, azimuth].
///
/// h(x) = [sqrt(x² + y²), atan2(x, y)]
///
/// Source: Bar-Shalom et al. (2001), Chapter 10.
pub fn radar_measurement_2d(state: &Vector) -> Vector {
    let x = state.get(0);
    let y = state.get(2);
    let polar = PolarCoordinate::from_cartesian(&Vector::new(vec![x, y]));
    Vector::new(vec![polar.range, polar.azimuth.radians()])
}

/// Check if radar scan rate satisfies Nyquist for target dynamics.
///
/// A target with maximum velocity `v` at range `r` has angular rate omega = v/r.
/// The angular rate is the "signal bandwidth" — the scan rate must be at least
/// twice this to avoid aliasing (missing target crossings between scans).
///
/// Required: scan_rate >= 2 * (omega / (2 pi)) = v / (pi * r).
///
/// Source: Nyquist (1928), Shannon (1949).
///         Skolnik (2001), *Introduction to Radar Systems*, Ch. 4.
pub fn is_scan_rate_adequate(scan_rate_hz: f64, max_target_velocity: f64, min_range: f64) -> bool {
    let max_angular_rate = max_target_velocity / min_range;
    let required_bandwidth = max_angular_rate / (2.0 * core::f64::consts::PI);
    sampling::is_adequately_sampled(scan_rate_hz, required_bandwidth)
}

/// Jacobian of the radar observation model (linearization for EKF).
///
/// H = ∂h/∂x evaluated at the current state.
///
/// For state [x, vx, y, vy] and measurement [range, azimuth]:
/// H = [[x/r, 0, y/r, 0],
///      [y/r², 0, -x/r², 0]]
pub fn radar_jacobian_2d(state: &Vector) -> Matrix {
    let x = state.get(0);
    let y = state.get(2);
    let r = (x * x + y * y).sqrt();
    let r2 = r * r;

    if r < ZERO_RANGE_EPS_M {
        return Matrix::zeros(2, 4); // degenerate at origin
    }

    Matrix::new(
        2,
        4,
        vec![
            x / r,
            0.0,
            y / r,
            0.0, // ∂range/∂state
            y / r2,
            0.0,
            -x / r2,
            0.0, // ∂azimuth/∂state
        ],
    )
}
