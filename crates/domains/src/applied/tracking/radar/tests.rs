use crate::applied::tracking::radar::engine::is_scan_rate_adequate;
use crate::applied::tracking::radar::ontology::*;
use crate::formal::math::angle::Angle;
use crate::formal::math::coordinate::{PolarCoordinate, SphericalCoordinate};
use crate::formal::math::linear_algebra::vector_space::Vector;
use pr4xis::ontology::Axiom;

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn range_non_negative() {
    assert!(RangeNonNegative.verify().is_ok());
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn polar_cartesian_roundtrip() {
    let cart = PolarCoordinate::new(10.0, Angle::from_radians(0.5)).to_cartesian();
    let polar = PolarCoordinate::from_cartesian(&cart);
    assert!((polar.range - 10.0).abs() < 1e-10);
    assert!(
        polar
            .azimuth
            .difference(&Angle::from_radians(0.5))
            .radians()
            .abs()
            < 1e-10
    );
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn spherical_cartesian_roundtrip() {
    let cart = SphericalCoordinate::new(100.0, Angle::from_radians(0.3), Angle::from_radians(0.2))
        .to_cartesian();
    let sph = SphericalCoordinate::from_cartesian(&cart);
    assert!((sph.range - 100.0).abs() < 1e-8);
    assert!(
        sph.azimuth
            .difference(&Angle::from_radians(0.3))
            .radians()
            .abs()
            < 1e-8
    );
    assert!(
        sph.elevation
            .difference(&Angle::from_radians(0.2))
            .radians()
            .abs()
            < 1e-8
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn zero_azimuth_points_north() {
    let cart = PolarCoordinate::new(10.0, Angle::from_radians(0.0)).to_cartesian();
    let (x, y) = (cart.get(0), cart.get(1));
    assert!(x.abs() < 1e-10); // no east component
    assert!((y - 10.0).abs() < 1e-10); // all north
}

// ---------------------------------------------------------------------------
// Signal processing: Nyquist scan rate check
// ---------------------------------------------------------------------------

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn fast_scan_rate_is_adequate() {
    // Target: 300 m/s at 10 km range
    // Angular rate = 300/10000 = 0.03 rad/s
    // Bandwidth = 0.03 / (2pi) ~ 0.00477 Hz
    // Nyquist rate = 2 * 0.00477 ~ 0.00955 Hz
    // 1 Hz scan rate >> 0.00955 Hz
    assert!(is_scan_rate_adequate(1.0, 300.0, 10_000.0));
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn slow_scan_rate_is_inadequate() {
    // Target: 1000 m/s at 100 m range (very close, very fast)
    // Angular rate = 1000/100 = 10 rad/s
    // Bandwidth = 10 / (2pi) ~ 1.59 Hz
    // Nyquist rate = 2 * 1.59 ~ 3.18 Hz
    // 1 Hz scan rate < 3.18 Hz
    assert!(!is_scan_rate_adequate(1.0, 1000.0, 100.0));
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn scan_rate_check_is_deterministic(
            rate in 0.01..100.0_f64,
            vel in 1.0..1000.0_f64,
            range in 10.0..100_000.0_f64,
        ) {
            let r1 = is_scan_rate_adequate(rate, vel, range);
            let r2 = is_scan_rate_adequate(rate, vel, range);
            prop_assert_eq!(r1, r2);
        }

        #[test]
        fn doubling_scan_rate_passes_if_original_does(
            rate in 0.01..100.0_f64,
            vel in 1.0..500.0_f64,
            range in 100.0..100_000.0_f64,
        ) {
            if is_scan_rate_adequate(rate, vel, range) {
                prop_assert!(is_scan_rate_adequate(2.0 * rate, vel, range),
                    "doubling scan rate should still be adequate");
            }
        }

        #[test]
        fn polar_cartesian_roundtrip_random(
            r in 0.1..1000.0_f64,
            az in -core::f64::consts::PI..core::f64::consts::PI,
        ) {
            let cart = PolarCoordinate::new(r, Angle::from_radians(az)).to_cartesian();
            let polar = PolarCoordinate::from_cartesian(&cart);
            let (r2, az2) = (polar.range, polar.azimuth.radians());
            prop_assert!((r - r2).abs() < 1e-8);
            // Azimuth wraps, so compare via sin/cos
            prop_assert!((az.sin() - az2.sin()).abs() < 1e-8);
            prop_assert!((az.cos() - az2.cos()).abs() < 1e-8);
        }

        #[test]
        fn spherical_cartesian_roundtrip_random(
            r in 0.1..1000.0_f64,
            az in -core::f64::consts::PI..core::f64::consts::PI,
            el in -1.0..1.0_f64, // avoid poles
        ) {
            let cart = SphericalCoordinate::new(
                r,
                Angle::from_radians(az),
                Angle::from_radians(el),
            )
            .to_cartesian();
            let sph = SphericalCoordinate::from_cartesian(&cart);
            let (r2, el2) = (sph.range, sph.elevation.radians());
            prop_assert!((r - r2).abs() < 1e-6);
            prop_assert!((el - el2).abs() < 1e-6);
        }

        #[test]
        fn range_is_non_negative(x in -100.0..100.0_f64, y in -100.0..100.0_f64) {
            let polar = PolarCoordinate::from_cartesian(&Vector::new(vec![x, y]));
            prop_assert!(polar.range >= 0.0);
        }

        #[test]
        fn coordinate_conversion_is_deterministic(
            r in 0.1..100.0_f64,
            az in -3.0..3.0_f64,
        ) {
            let c1 = PolarCoordinate::new(r, Angle::from_radians(az)).to_cartesian();
            let c2 = PolarCoordinate::new(r, Angle::from_radians(az)).to_cartesian();
            prop_assert_eq!(c1.get(0).to_bits(), c2.get(0).to_bits());
            prop_assert_eq!(c1.get(1).to_bits(), c2.get(1).to_bits());
        }
    }

    pr4xis::register_praxis_value!(scan_rate_check_is_deterministic, Deterministic);
    pr4xis::register_praxis_value!(doubling_scan_rate_passes_if_original_does, Verifiable);
    pr4xis::register_praxis_value!(polar_cartesian_roundtrip_random, Deterministic);
    pr4xis::register_praxis_value!(spherical_cartesian_roundtrip_random, Deterministic);
    pr4xis::register_praxis_value!(range_is_non_negative, Verifiable);
    pr4xis::register_praxis_value!(coordinate_conversion_is_deterministic, Deterministic);
}
