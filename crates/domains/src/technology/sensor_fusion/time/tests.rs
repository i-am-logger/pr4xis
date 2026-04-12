use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::technology::sensor_fusion::time::clock::SensorClock;
use crate::technology::sensor_fusion::time::epoch::FusionEpoch;
use crate::technology::sensor_fusion::time::ontology::*;
use crate::technology::sensor_fusion::time::synchronization;

use crate::science::math::temporal::instant::Instant;
use crate::science::math::temporal::time_system::TimeSystem;
use crate::technology::sensor_fusion::sensor::modality::SensorType;

// ---------------------------------------------------------------------------
// Category law validation
// ---------------------------------------------------------------------------

#[test]
fn sensor_time_category_laws() {
    check_category_laws::<SensorTimeCategory>().unwrap();
}

// ---------------------------------------------------------------------------
// Ontology validation
// ---------------------------------------------------------------------------

#[test]
fn sensor_time_ontology_validates() {
    SensorTimeOntology::validate().unwrap();
}

#[test]
fn axiom_interpolation_bounded() {
    assert!(InterpolationBounded.holds());
}

#[test]
fn axiom_extrapolation_unbounded() {
    assert!(ExtrapolationUnbounded.holds());
}

#[test]
fn axiom_nearest_neighbor_bounded() {
    assert!(NearestNeighborBounded.holds());
}

// ---------------------------------------------------------------------------
// Epoch tests
// ---------------------------------------------------------------------------

#[test]
fn epoch_staleness_detection() {
    let epoch = FusionEpoch::from_gps_seconds(100.0, SensorType::GnssReceiver);
    let now = Instant::new(100.5, TimeSystem::GPS);
    assert_eq!(epoch.is_stale(&now, 1.0), Some(false));

    let later = Instant::new(102.0, TimeSystem::GPS);
    assert_eq!(epoch.is_stale(&later, 1.0), Some(true));
}

// ---------------------------------------------------------------------------
// Clock tests
// ---------------------------------------------------------------------------

#[test]
fn clock_offset_conversion_round_trip() {
    let clock = SensorClock::new(
        SensorType::IMU,
        crate::science::math::temporal::clock::ClockModel::ideal(),
        0.003,
    );
    let system_time = 500.0;
    let sensor_time = clock.from_system_time(system_time);
    let recovered = clock.to_system_time(sensor_time);
    assert!((recovered - system_time).abs() < 1e-12);
}

// ---------------------------------------------------------------------------
// Synchronization tests
// ---------------------------------------------------------------------------

#[test]
fn interpolate_quarter_point() {
    let (value, alpha) = synchronization::interpolate(0.0, 0.0, 4.0, 100.0, 1.0);
    assert!((value - 25.0).abs() < 1e-10);
    assert!((alpha - 0.25).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// Property-based proofs
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Interpolation at t_before returns v_before, at t_after returns v_after.
        #[test]
        fn interpolate_at_endpoints(
            v0 in -1000.0..1000.0_f64,
            v1 in -1000.0..1000.0_f64,
            t0 in 0.0..100.0_f64,
            dt in 0.001..10.0_f64,
        ) {
            let t1 = t0 + dt;
            let (val_at_t0, _) = synchronization::interpolate(t0, v0, t1, v1, t0);
            let (val_at_t1, _) = synchronization::interpolate(t0, v0, t1, v1, t1);
            prop_assert!((val_at_t0 - v0).abs() < 1e-8,
                "interpolation at t_before should return v_before");
            prop_assert!((val_at_t1 - v1).abs() < 1e-8,
                "interpolation at t_after should return v_after");
        }

        /// Interpolation at midpoint is average of endpoints (linear).
        #[test]
        fn interpolate_midpoint_is_average(
            v0 in -1000.0..1000.0_f64,
            v1 in -1000.0..1000.0_f64,
            t0 in 0.0..100.0_f64,
            dt in 0.001..10.0_f64,
        ) {
            let t1 = t0 + dt;
            let mid = (t0 + t1) / 2.0;
            let (val, _) = synchronization::interpolate(t0, v0, t1, v1, mid);
            let expected = (v0 + v1) / 2.0;
            prop_assert!((val - expected).abs() < 1e-8);
        }

        /// Extrapolation with zero rate returns original value.
        #[test]
        fn extrapolate_zero_rate(
            v in -1000.0..1000.0_f64,
            dt in -10.0..10.0_f64,
        ) {
            let result = synchronization::extrapolate(v, 0.0, dt);
            prop_assert!((result - v).abs() < 1e-10);
        }

        /// Clock round-trip preserves time.
        #[test]
        fn clock_round_trip(
            offset in -1.0..1.0_f64,
            t in 0.0..1e6_f64,
        ) {
            let clock = SensorClock::new(
                SensorType::IMU,
                crate::science::math::temporal::clock::ClockModel::ideal(),
                offset,
            );
            let sensor_t = clock.from_system_time(t);
            let recovered = clock.to_system_time(sensor_t);
            prop_assert!((recovered - t).abs() < 1e-10);
        }

        /// Epoch age is non-negative when reference is after measurement.
        #[test]
        fn epoch_age_nonneg(
            t_meas in 0.0..1000.0_f64,
            dt in 0.0..100.0_f64,
        ) {
            let epoch = FusionEpoch::from_gps_seconds(t_meas, SensorType::IMU);
            let reference = Instant::new(t_meas + dt, TimeSystem::GPS);
            let age = epoch.age(&reference).unwrap();
            prop_assert!(age >= -1e-10, "age should be non-negative: {}", age);
        }
    }
}
