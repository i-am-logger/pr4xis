use crate::technology::tracking::radar::coordinate::*;
use crate::technology::tracking::radar::engine::is_scan_rate_adequate;
use crate::technology::tracking::radar::ontology::*;
use pr4xis::ontology::Axiom;

#[test]
fn range_non_negative() {
    assert!(RangeNonNegative.holds());
}

#[test]
fn polar_cartesian_roundtrip() {
    let (x, y) = polar_to_cartesian_2d(10.0, 0.5);
    let (r, az) = cartesian_to_polar_2d(x, y);
    assert!((r - 10.0).abs() < 1e-10);
    assert!((az - 0.5).abs() < 1e-10);
}

#[test]
fn spherical_cartesian_roundtrip() {
    let (x, y, z) = spherical_to_cartesian(100.0, 0.3, 0.2);
    let (r, az, el) = cartesian_to_spherical(x, y, z);
    assert!((r - 100.0).abs() < 1e-8);
    assert!((az - 0.3).abs() < 1e-8);
    assert!((el - 0.2).abs() < 1e-8);
}

#[test]
fn zero_azimuth_points_north() {
    let (x, y) = polar_to_cartesian_2d(10.0, 0.0);
    assert!(x.abs() < 1e-10); // no east component
    assert!((y - 10.0).abs() < 1e-10); // all north
}

// ---------------------------------------------------------------------------
// Signal processing: Nyquist scan rate check
// ---------------------------------------------------------------------------

#[test]
fn fast_scan_rate_is_adequate() {
    // Target: 300 m/s at 10 km range
    // Angular rate = 300/10000 = 0.03 rad/s
    // Bandwidth = 0.03 / (2pi) ~ 0.00477 Hz
    // Nyquist rate = 2 * 0.00477 ~ 0.00955 Hz
    // 1 Hz scan rate >> 0.00955 Hz
    assert!(is_scan_rate_adequate(1.0, 300.0, 10_000.0));
}

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
            az in -std::f64::consts::PI..std::f64::consts::PI,
        ) {
            let (x, y) = polar_to_cartesian_2d(r, az);
            let (r2, az2) = cartesian_to_polar_2d(x, y);
            prop_assert!((r - r2).abs() < 1e-8);
            // Azimuth wraps, so compare via sin/cos
            prop_assert!((az.sin() - az2.sin()).abs() < 1e-8);
            prop_assert!((az.cos() - az2.cos()).abs() < 1e-8);
        }

        #[test]
        fn spherical_cartesian_roundtrip_random(
            r in 0.1..1000.0_f64,
            az in -std::f64::consts::PI..std::f64::consts::PI,
            el in -1.0..1.0_f64, // avoid poles
        ) {
            let (x, y, z) = spherical_to_cartesian(r, az, el);
            let (r2, _az2, el2) = cartesian_to_spherical(x, y, z);
            prop_assert!((r - r2).abs() < 1e-6);
            prop_assert!((el - el2).abs() < 1e-6);
        }

        #[test]
        fn range_is_non_negative(x in -100.0..100.0_f64, y in -100.0..100.0_f64) {
            let (r, _) = cartesian_to_polar_2d(x, y);
            prop_assert!(r >= 0.0);
        }

        #[test]
        fn coordinate_conversion_is_deterministic(
            r in 0.1..100.0_f64,
            az in -3.0..3.0_f64,
        ) {
            let (x1, y1) = polar_to_cartesian_2d(r, az);
            let (x2, y2) = polar_to_cartesian_2d(r, az);
            prop_assert_eq!(x1.to_bits(), x2.to_bits());
            prop_assert_eq!(y1.to_bits(), y2.to_bits());
        }
    }
}
