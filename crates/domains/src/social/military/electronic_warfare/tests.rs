use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::social::military::electronic_warfare::engine::*;
use crate::social::military::electronic_warfare::ontology::*;

#[test]
fn ew_category_laws() {
    check_category_laws::<EwCategory>().unwrap();
}

#[test]
fn ew_ontology_validates() {
    EwOntology::validate().unwrap();
}

#[test]
fn aoa_bounded_holds() {
    assert!(AoaBounded.holds());
}

#[test]
fn tdoa_requires_sensor_pair_holds() {
    assert!(TdoaRequiresSensorPair.holds());
}

#[test]
fn wrap_angle_within_range() {
    let a = wrap_angle(4.0);
    assert!(a >= -core::f64::consts::PI && a <= core::f64::consts::PI);
}

#[test]
fn wrap_angle_identity_in_range() {
    let a = 1.5;
    assert!((wrap_angle(a) - a).abs() < 1e-12);
}

#[test]
fn aoa_triangulation_perpendicular() {
    let m1 = AoaMeasurement {
        sensor_pos: [0.0, 0.0],
        bearing: core::f64::consts::FRAC_PI_2, // due east
        sigma: 0.01,
    };
    let m2 = AoaMeasurement {
        sensor_pos: [100.0, 100.0],
        bearing: core::f64::consts::PI, // due south
        sigma: 0.01,
    };
    let pos = aoa_triangulation(&m1, &m2).unwrap();
    assert!(
        (pos[0] - 100.0).abs() < 1e-6,
        "expected x~100, got {}",
        pos[0]
    );
    assert!((pos[1] - 0.0).abs() < 1e-6, "expected y~0, got {}", pos[1]);
}

#[test]
fn aoa_parallel_returns_none() {
    let m1 = AoaMeasurement {
        sensor_pos: [0.0, 0.0],
        bearing: 0.0,
        sigma: 0.01,
    };
    let m2 = AoaMeasurement {
        sensor_pos: [100.0, 0.0],
        bearing: 0.0, // same bearing = parallel
        sigma: 0.01,
    };
    assert!(aoa_triangulation(&m1, &m2).is_none());
}

#[test]
fn tdoa_residual_at_true_position() {
    let meas = TdoaMeasurement {
        sensor_a: [0.0, 0.0],
        sensor_b: [100.0, 0.0],
        tdoa: 0.0, // emitter equidistant from both
        signal_speed: 3e8,
    };
    let emitter = [50.0, 50.0]; // equidistant point
    let residual = tdoa_residual(&meas, &emitter);
    assert!(
        residual.abs() < 1e-6,
        "residual should be ~0, got {}",
        residual
    );
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn wrap_angle_always_in_range(angle in -100.0..100.0_f64) {
            let wrapped = wrap_angle(angle);
            prop_assert!(wrapped >= -core::f64::consts::PI && wrapped <= core::f64::consts::PI,
                "wrapped angle {} out of [-pi, pi] for input {}", wrapped, angle);
        }

        #[test]
        fn tdoa_range_difference_sign(tdoa in -0.001..0.001_f64) {
            let meas = TdoaMeasurement {
                sensor_a: [0.0, 0.0],
                sensor_b: [100.0, 0.0],
                tdoa,
                signal_speed: 3e8,
            };
            let rd = meas.range_difference();
            // sign of range difference should match sign of TDOA
            if tdoa > 0.0 {
                prop_assert!(rd > 0.0);
            } else if tdoa < 0.0 {
                prop_assert!(rd < 0.0);
            } else {
                prop_assert!((rd).abs() < 1e-12);
            }
        }
    }
}
