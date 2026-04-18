#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::signal_processing::sampling;

use crate::applied::tracking::radar::coordinate;

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
    let (range, azimuth) = coordinate::cartesian_to_polar_2d(x, y);
    Vector::new(vec![range, azimuth])
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

    if r < 1e-10 {
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
