use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::formal::math::angle::Angle;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::Quantity;
use crate::formal::math::temporal::duration::Duration;
use crate::social::military::electronic_warfare::engine::*;
use crate::social::military::electronic_warfare::ontology::*;

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn ew_category_laws() {
    assert_category_laws::<EwCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn ew_ontology_validates() {
    EwOntology::validate().unwrap();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn aoa_bounded_holds() {
    assert!(AoaBounded.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn tdoa_requires_sensor_pair_holds() {
    assert!(TdoaRequiresSensorPair.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn wrap_angle_within_range() {
    let a = wrap_angle(&Quantity::from_unit(4.0, &unit::RADIAN));
    assert!((-core::f64::consts::PI..=core::f64::consts::PI).contains(&a.value));
    assert_eq!(a.dimension, unit::RADIAN.dimension);
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn wrap_angle_identity_in_range() {
    let a = 1.5;
    assert!((wrap_angle(&Quantity::from_unit(a, &unit::RADIAN)).value - a).abs() < 1e-12);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn aoa_triangulation_perpendicular() {
    let m1 = AoaMeasurement {
        sensor_pos: Vector::new(vec![0.0, 0.0]),
        bearing: Angle::from_radians(core::f64::consts::FRAC_PI_2), // due east
        sigma: Quantity::from_unit(0.01, &unit::RADIAN),
    };
    let m2 = AoaMeasurement {
        sensor_pos: Vector::new(vec![100.0, 100.0]),
        bearing: Angle::from_radians(core::f64::consts::PI), // due south
        sigma: Quantity::from_unit(0.01, &unit::RADIAN),
    };
    let pos = aoa_triangulation(&m1, &m2).unwrap();
    assert!(
        (pos.get(0) - 100.0).abs() < 1e-6,
        "expected x~100, got {}",
        pos.get(0)
    );
    assert!(
        (pos.get(1) - 0.0).abs() < 1e-6,
        "expected y~0, got {}",
        pos.get(1)
    );
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn aoa_parallel_returns_none() {
    let m1 = AoaMeasurement {
        sensor_pos: Vector::new(vec![0.0, 0.0]),
        bearing: Angle::from_radians(0.0),
        sigma: Quantity::from_unit(0.01, &unit::RADIAN),
    };
    let m2 = AoaMeasurement {
        sensor_pos: Vector::new(vec![100.0, 0.0]),
        bearing: Angle::from_radians(0.0), // same bearing = parallel
        sigma: Quantity::from_unit(0.01, &unit::RADIAN),
    };
    assert!(aoa_triangulation(&m1, &m2).is_none());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn tdoa_residual_at_true_position() {
    let meas = TdoaMeasurement {
        sensor_a: Vector::new(vec![0.0, 0.0]),
        sensor_b: Vector::new(vec![100.0, 0.0]),
        tdoa: Duration::from_seconds(0.0), // emitter equidistant from both
        signal_speed: Quantity::from_unit(3e8, &unit::METER_PER_SECOND),
    };
    let emitter = Vector::new(vec![50.0, 50.0]); // equidistant point
    let residual = tdoa_residual(&meas, &emitter);
    assert!(
        residual.value.abs() < 1e-6,
        "residual should be ~0, got {}",
        residual.value
    );
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn wrap_angle_always_in_range(angle in -100.0..100.0_f64) {
            let wrapped = wrap_angle(&Quantity::from_unit(angle, &unit::RADIAN));
            prop_assert!((-core::f64::consts::PI..=core::f64::consts::PI).contains(&wrapped.value),
                "wrapped angle {} out of [-pi, pi] for input {}", wrapped.value, angle);
        }

        #[test]
        fn tdoa_range_difference_sign(tdoa in -0.001..0.001_f64) {
            let meas = TdoaMeasurement {
                sensor_a: Vector::new(vec![0.0, 0.0]),
                sensor_b: Vector::new(vec![100.0, 0.0]),
                tdoa: Duration::from_seconds(tdoa),
                signal_speed: Quantity::from_unit(3e8, &unit::METER_PER_SECOND),
            };
            let rd = meas.range_difference();
            // sign of range difference should match sign of TDOA
            if tdoa > 0.0 {
                prop_assert!(rd.value > 0.0);
            } else if tdoa < 0.0 {
                prop_assert!(rd.value < 0.0);
            } else {
                prop_assert!((rd.value).abs() < 1e-12);
            }
        }
    }

    pr4xis::register_praxis_value!(wrap_angle_always_in_range, Verifiable);
    pr4xis::register_praxis_value!(tdoa_range_difference_sign, Verifiable);
}
