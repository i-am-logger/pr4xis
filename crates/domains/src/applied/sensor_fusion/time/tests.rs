use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::sensor_fusion::time::clock::SensorClock;
use crate::applied::sensor_fusion::time::epoch::FusionEpoch;
use crate::applied::sensor_fusion::time::ontology::*;
use crate::applied::sensor_fusion::time::synchronization;

use crate::applied::sensor_fusion::sensor::modality::SensorType;
use crate::formal::math::temporal::duration::Duration;
use crate::formal::math::temporal::instant::Instant;
use crate::formal::math::temporal::time_system::TimeSystem;

// ---------------------------------------------------------------------------
// Category law validation
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn sensor_time_category_laws() {
    assert_category_laws::<SensorTimeCategory>();
}

// ---------------------------------------------------------------------------
// Ontology validation
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn sensor_time_ontology_validates() {
    SensorTimeOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_interpolation_bounded() {
    assert!(InterpolationBounded.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_extrapolation_unbounded() {
    assert!(ExtrapolationUnbounded.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn axiom_nearest_neighbor_bounded() {
    assert!(NearestNeighborBounded.verify().is_ok());
}

// ---------------------------------------------------------------------------
// Epoch tests
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn epoch_staleness_detection() {
    let epoch = FusionEpoch::from_gps_seconds(100.0, SensorType::GnssReceiver);
    let now = Instant::new(100.5, TimeSystem::GPS);
    assert_eq!(
        epoch.is_stale(&now, &Duration::from_seconds(1.0)),
        Some(false)
    );

    let later = Instant::new(102.0, TimeSystem::GPS);
    assert_eq!(
        epoch.is_stale(&later, &Duration::from_seconds(1.0)),
        Some(true)
    );
}

// ---------------------------------------------------------------------------
// Clock tests
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn clock_offset_conversion_round_trip() {
    let clock = SensorClock::new(
        SensorType::IMU,
        crate::formal::math::temporal::clock::ClockModel::ideal(),
        Duration::from_seconds(0.003),
    );
    let system_time = 500.0;
    let sensor_time = clock.from_system_time(system_time);
    let recovered = clock.to_system_time(sensor_time.value);
    assert!((recovered.value - system_time).abs() < 1e-12);
}

// ---------------------------------------------------------------------------
// Synchronization tests
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn interpolate_quarter_point() {
    let (value, alpha) = synchronization::interpolate(
        &Instant::new(0.0, TimeSystem::GPS),
        0.0,
        &Instant::new(4.0, TimeSystem::GPS),
        100.0,
        &Instant::new(1.0, TimeSystem::GPS),
    );
    assert!((value.value - 25.0).abs() < 1e-10);
    assert!((alpha.value - 0.25).abs() < 1e-10);
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
            let inst0 = Instant::new(t0, TimeSystem::GPS);
            let inst1 = Instant::new(t1, TimeSystem::GPS);
            let (val_at_t0, _) = synchronization::interpolate(&inst0, v0, &inst1, v1, &inst0);
            let (val_at_t1, _) = synchronization::interpolate(&inst0, v0, &inst1, v1, &inst1);
            prop_assert!((val_at_t0.value - v0).abs() < 1e-8,
                "interpolation at t_before should return v_before");
            prop_assert!((val_at_t1.value - v1).abs() < 1e-8,
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
            let (val, _) = synchronization::interpolate(
                &Instant::new(t0, TimeSystem::GPS),
                v0,
                &Instant::new(t1, TimeSystem::GPS),
                v1,
                &Instant::new(mid, TimeSystem::GPS),
            );
            let expected = (v0 + v1) / 2.0;
            prop_assert!((val.value - expected).abs() < 1e-8);
        }

        /// Extrapolation with zero rate returns original value.
        #[test]
        fn extrapolate_zero_rate(
            v in -1000.0..1000.0_f64,
            dt in -10.0..10.0_f64,
        ) {
            let result = synchronization::extrapolate(v, 0.0, &Duration::from_seconds(dt));
            prop_assert!((result.value - v).abs() < 1e-10);
        }

        /// Clock round-trip preserves time.
        #[test]
        fn clock_round_trip(
            offset in -1.0..1.0_f64,
            t in 0.0..1e6_f64,
        ) {
            let clock = SensorClock::new(
                SensorType::IMU,
                crate::formal::math::temporal::clock::ClockModel::ideal(),
                Duration::from_seconds(offset),
            );
            let sensor_t = clock.from_system_time(t);
            let recovered = clock.to_system_time(sensor_t.value);
            prop_assert!((recovered.value - t).abs() < 1e-10);
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
            prop_assert!(age.value >= -1e-10, "age should be non-negative: {}", age.value);
        }
    }

    pr4xis::register_praxis_value!(interpolate_at_endpoints, Verifiable);
    pr4xis::register_praxis_value!(interpolate_midpoint_is_average, Verifiable);
    pr4xis::register_praxis_value!(extrapolate_zero_rate, Verifiable);
    pr4xis::register_praxis_value!(clock_round_trip, Deterministic);
    pr4xis::register_praxis_value!(epoch_age_nonneg, Verifiable);
}
