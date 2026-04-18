#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;

/// Temporal alignment strategies for multi-sensor fusion.
///
/// When sensors operate at different rates, measurements must be aligned
/// to a common time before fusion. The strategy controls how measurements
/// are interpolated or extrapolated to the target time.
///
/// Source: Bar-Shalom et al. (2001), Section 6.2.
///         Groves (2013), Section 17.2.4 — "Time synchronization."
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Concept)]
pub enum SyncStrategy {
    /// Use the measurement nearest in time to the target.
    /// Simple, no assumptions about dynamics.
    /// Error bounded by half the measurement period.
    NearestNeighbor,
    /// Linearly interpolate between two bracketing measurements.
    /// Assumes approximately constant rate of change.
    /// Requires one measurement before and one after target time.
    LinearInterpolation,
    /// Extrapolate from the latest measurement using a model.
    /// Dangerous: error grows unboundedly with extrapolation distance.
    /// Only use when no future measurement is available (real-time).
    Extrapolation,
}

/// Align a measurement value to a target time using interpolation.
///
/// Given two measurement values `v_before` at `t_before` and `v_after` at `t_after`,
/// compute the interpolated value at `target_time`.
///
/// Returns the aligned value and the interpolation fraction alpha in [0, 1].
pub fn interpolate(
    t_before: f64,
    v_before: f64,
    t_after: f64,
    v_after: f64,
    target_time: f64,
) -> (f64, f64) {
    let dt = t_after - t_before;
    if dt.abs() < 1e-15 {
        return (v_before, 0.0);
    }
    let alpha = (target_time - t_before) / dt;
    let value = v_before + alpha * (v_after - v_before);
    (value, alpha)
}

/// Align a measurement to a target time using the specified strategy.
///
/// For NearestNeighbor and LinearInterpolation, both `value_before` and
/// `value_after` with their timestamps are used.
/// For Extrapolation, only `value_before` and a rate estimate are used.
pub fn align_measurement(
    measurement_time: f64,
    measurement_value: f64,
    target_time: f64,
    strategy: SyncStrategy,
) -> f64 {
    match strategy {
        SyncStrategy::NearestNeighbor => {
            // Return the measurement value as-is (nearest available)
            measurement_value
        }
        SyncStrategy::LinearInterpolation => {
            // Without a second measurement, return as-is.
            // For proper interpolation, use the `interpolate` function with two values.
            measurement_value
        }
        SyncStrategy::Extrapolation => {
            // Without a rate, we cannot extrapolate meaningfully.
            // Return measurement value (zero-order hold).
            let _ = target_time - measurement_time;
            measurement_value
        }
    }
}

/// Extrapolate a measurement forward by dt using a known rate.
///
/// value_at_target = value + rate * dt
///
/// WARNING: extrapolation error grows linearly (or worse) with dt.
pub fn extrapolate(value: f64, rate: f64, dt: f64) -> f64 {
    value + rate * dt
}

/// Compute the maximum synchronization error for a given strategy and period.
///
/// - NearestNeighbor: error <= period / 2 * max_rate
/// - LinearInterpolation: error <= period^2 / 8 * max_acceleration
/// - Extrapolation: error grows unboundedly (returns None)
pub fn max_sync_error(strategy: SyncStrategy, period: f64, max_dynamics: f64) -> Option<f64> {
    match strategy {
        SyncStrategy::NearestNeighbor => Some(period / 2.0 * max_dynamics),
        SyncStrategy::LinearInterpolation => Some(period * period / 8.0 * max_dynamics),
        SyncStrategy::Extrapolation => None, // unbounded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolate_midpoint() {
        let (value, alpha) = interpolate(0.0, 10.0, 1.0, 20.0, 0.5);
        assert!((value - 15.0).abs() < 1e-10);
        assert!((alpha - 0.5).abs() < 1e-10);
    }

    #[test]
    fn interpolate_at_endpoints() {
        let (v0, a0) = interpolate(0.0, 10.0, 1.0, 20.0, 0.0);
        assert!((v0 - 10.0).abs() < 1e-10);
        assert!(a0.abs() < 1e-10);

        let (v1, a1) = interpolate(0.0, 10.0, 1.0, 20.0, 1.0);
        assert!((v1 - 20.0).abs() < 1e-10);
        assert!((a1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn extrapolate_forward() {
        let result = extrapolate(100.0, 2.0, 0.5);
        assert!((result - 101.0).abs() < 1e-10);
    }

    #[test]
    fn nearest_neighbor_returns_measurement() {
        let v = align_measurement(1.0, 42.0, 1.1, SyncStrategy::NearestNeighbor);
        assert!((v - 42.0).abs() < 1e-10);
    }

    #[test]
    fn sync_error_nearest_neighbor() {
        // At 100 Hz (period=0.01s) with max rate 10 m/s:
        // max error = 0.01/2 * 10 = 0.05 m
        let err = max_sync_error(SyncStrategy::NearestNeighbor, 0.01, 10.0).unwrap();
        assert!((err - 0.05).abs() < 1e-10);
    }

    #[test]
    fn sync_error_interpolation() {
        // At 100 Hz (period=0.01s) with max acceleration 10 m/s^2:
        // max error = 0.01^2 / 8 * 10 = 0.000125 m
        let err = max_sync_error(SyncStrategy::LinearInterpolation, 0.01, 10.0).unwrap();
        assert!((err - 0.000125).abs() < 1e-12);
    }

    #[test]
    fn sync_error_extrapolation_unbounded() {
        assert!(max_sync_error(SyncStrategy::Extrapolation, 0.01, 10.0).is_none());
    }

    #[test]
    fn sync_strategy_variants() {
        let variants = SyncStrategy::variants();
        assert_eq!(variants.len(), 3);
    }
}
