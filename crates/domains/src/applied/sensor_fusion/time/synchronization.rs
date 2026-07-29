/// Temporal alignment strategies for multi-sensor fusion.
///
/// Backward-compatibility alias for the proc-macro-generated
/// `SensorTimeConcept` in `super::ontology`. New code should prefer
/// `SensorTimeConcept` directly; existing call sites keep the historical
/// `SyncStrategy` spelling.
///
/// Source: Bar-Shalom et al. (2001), Section 6.2.
///         Groves (2013), Section 17.2.4 — "Time synchronization."
pub use super::ontology::SensorTimeConcept as SyncStrategy;

use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::duration::Duration;
use crate::formal::math::temporal::instant::Instant;

/// Align a measurement value to a target time using interpolation.
///
/// Given two measurement values `v_before` at `t_before` and `v_after` at `t_after`,
/// compute the interpolated value at `target_time`.
///
/// Returns the aligned value and the interpolation fraction alpha in [0, 1].
/// Both are dimensionless [`Quantity`]s (`unit::UNITLESS`) — this is a
/// generic interpolation utility over whatever physical dimension the
/// caller's measurement values already carry (position, velocity, ...),
/// the same treatment as `Vector::dot`/`Point3::distance_to`. The
/// timestamps, unlike the measurement values, are always physically time
/// regardless of caller, so they are typed [`Instant`]s.
pub fn interpolate(
    t_before: &Instant,
    v_before: f64,
    t_after: &Instant,
    v_after: f64,
    target_time: &Instant,
) -> (Quantity, Quantity) {
    let dt = t_after.seconds - t_before.seconds;
    if dt.abs() < 1e-15 {
        return (
            Quantity::from_unit(v_before, &unit::UNITLESS),
            Quantity::from_unit(0.0, &unit::UNITLESS),
        );
    }
    let alpha = (target_time.seconds - t_before.seconds) / dt;
    let value = v_before + alpha * (v_after - v_before);
    (
        Quantity::from_unit(value, &unit::UNITLESS),
        Quantity::from_unit(alpha, &unit::UNITLESS),
    )
}

/// Align a measurement to a target time using the specified strategy.
///
/// For NearestNeighbor and LinearInterpolation, both `value_before` and
/// `value_after` with their timestamps are used.
/// For Extrapolation, only `value_before` and a rate estimate are used.
/// Dimensionless [`Quantity`] (`unit::UNITLESS`), same reasoning as
/// [`interpolate`].
pub fn align_measurement(
    measurement_time: &Instant,
    measurement_value: f64,
    target_time: &Instant,
    strategy: SyncStrategy,
) -> Quantity {
    let value = match strategy {
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
            let _ = target_time.seconds - measurement_time.seconds;
            measurement_value
        }
    };
    Quantity::from_unit(value, &unit::UNITLESS)
}

/// Extrapolate a measurement forward by dt using a known rate.
///
/// value_at_target = value + rate * dt
///
/// WARNING: extrapolation error grows linearly (or worse) with dt.
///
/// Dimensionless [`Quantity`] (`unit::UNITLESS`), same reasoning as
/// [`interpolate`].
pub fn extrapolate(value: f64, rate: f64, dt: &Duration) -> Quantity {
    Quantity::from_unit(value + rate * dt.seconds(), &unit::UNITLESS)
}

/// Compute the maximum synchronization error for a given strategy and period.
///
/// - NearestNeighbor: error <= period / 2 * max_rate
/// - LinearInterpolation: error <= period^2 / 8 * max_acceleration
/// - Extrapolation: error grows unboundedly (returns None)
///
/// This is a bound on the resulting *position* error induced by imperfect
/// time alignment given the measurement's known dynamics (max rate or max
/// acceleration) — not itself a duration. `period * max_rate` (s · m/s) and
/// `period² * max_acceleration` (s² · m/s²) both reduce to meters, so the
/// result is typed [`Dimension::LENGTH`](crate::formal::math::quantity::dimension::Dimension::LENGTH),
/// matching the worked examples in this module's own tests (e.g. "max error
/// = 0.01/2 * 10 = 0.05 m").
pub fn max_sync_error(
    strategy: SyncStrategy,
    period: &Duration,
    max_dynamics: f64,
) -> Option<Quantity> {
    match strategy {
        SyncStrategy::NearestNeighbor => Some(Quantity::from_unit(
            period.seconds() / 2.0 * max_dynamics,
            &unit::METER,
        )),
        SyncStrategy::LinearInterpolation => Some(Quantity::from_unit(
            period.seconds() * period.seconds() / 8.0 * max_dynamics,
            &unit::METER,
        )),
        SyncStrategy::Extrapolation => None, // unbounded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formal::math::temporal::time_system::TimeSystem;
    use pr4xis::category::FinitelyGenerated;

    fn t(seconds: f64) -> Instant {
        Instant::new(seconds, TimeSystem::GPS)
    }

    fn dur(seconds: f64) -> Duration {
        Duration::from_seconds(seconds)
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn interpolate_midpoint() {
        let (value, alpha) = interpolate(&t(0.0), 10.0, &t(1.0), 20.0, &t(0.5));
        assert!((value.value - 15.0).abs() < 1e-10);
        assert!((alpha.value - 0.5).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn interpolate_at_endpoints() {
        let (v0, a0) = interpolate(&t(0.0), 10.0, &t(1.0), 20.0, &t(0.0));
        assert!((v0.value - 10.0).abs() < 1e-10);
        assert!(a0.value.abs() < 1e-10);

        let (v1, a1) = interpolate(&t(0.0), 10.0, &t(1.0), 20.0, &t(1.0));
        assert!((v1.value - 20.0).abs() < 1e-10);
        assert!((a1.value - 1.0).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn extrapolate_forward() {
        let result = extrapolate(100.0, 2.0, &dur(0.5));
        assert!((result.value - 101.0).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn nearest_neighbor_returns_measurement() {
        let v = align_measurement(&t(1.0), 42.0, &t(1.1), SyncStrategy::NearestNeighbor);
        assert!((v.value - 42.0).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sync_error_nearest_neighbor() {
        // At 100 Hz (period=0.01s) with max rate 10 m/s:
        // max error = 0.01/2 * 10 = 0.05 m
        let err = max_sync_error(SyncStrategy::NearestNeighbor, &dur(0.01), 10.0).unwrap();
        assert!((err.value - 0.05).abs() < 1e-10);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sync_error_interpolation() {
        // At 100 Hz (period=0.01s) with max acceleration 10 m/s^2:
        // max error = 0.01^2 / 8 * 10 = 0.000125 m
        let err = max_sync_error(SyncStrategy::LinearInterpolation, &dur(0.01), 10.0).unwrap();
        assert!((err.value - 0.000125).abs() < 1e-12);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sync_error_extrapolation_unbounded() {
        assert!(max_sync_error(SyncStrategy::Extrapolation, &dur(0.01), 10.0).is_none());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn sync_strategy_variants() {
        let variants = SyncStrategy::variants();
        assert_eq!(variants.len(), 3);
    }
}
