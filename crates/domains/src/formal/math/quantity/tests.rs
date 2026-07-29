use pr4xis::category::laws::assert_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::formal::math::quantity::dimension::Dimension;
use crate::formal::math::quantity::ontology::*;
use crate::formal::math::quantity::unit;
use crate::formal::math::quantity::value::{Quantity, QuantityRange};

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn dimension_category_laws() {
    assert_category_laws::<QuantityCategory>();
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn quantity_ontology_validates() {
    QuantityOntology::validate().unwrap();
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn quantity_range_guards_dimension_and_order() {
    // A well-formed, same-dimension, ordered interval is accepted.
    let r = QuantityRange::new(
        Quantity::from_unit(0.01, &unit::METER),
        Quantity::from_unit(0.1, &unit::METER),
    )
    .expect("0.01 m ≤ 0.1 m is a valid length interval");
    assert_eq!(r.dimension(), Dimension::LENGTH);
    assert!(r.contains(&Quantity::from_unit(0.05, &unit::METER)));
    assert!(!r.contains(&Quantity::from_unit(0.5, &unit::METER)));
    // A quantity of a different dimension is never contained (the guard).
    assert!(!r.contains(&Quantity::from_unit(0.05, &unit::SECOND)));
    // A range spanning two dimensions (metres..seconds) is refused.
    assert!(
        QuantityRange::new(
            Quantity::from_unit(1.0, &unit::METER),
            Quantity::from_unit(5.0, &unit::SECOND),
        )
        .is_none(),
        "a cross-dimension range must be rejected"
    );
    // An inverted interval (max < min) is refused.
    assert!(
        QuantityRange::new(Quantity::dimensionless(1.0), Quantity::dimensionless(0.0)).is_none(),
        "an inverted range must be rejected"
    );
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dimension_commutativity() {
    assert!(DimensionCommutativity.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dimension_associativity() {
    assert!(DimensionAssociativity.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dimension_identity() {
    assert!(DimensionIdentity.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn dimension_inverse() {
    assert!(DimensionInverse.verify().is_ok());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn addition_requires_same_dimension() {
    assert!(AdditionRequiresSameDimension.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn derived_dimension_consistency() {
    assert!(DerivedDimensionConsistency.verify().is_ok());
}

#[pr4xis::praxis_value(Deterministic)]
#[test]
fn unit_conversion_roundtrip() {
    assert!(UnitConversionRoundtrip.verify().is_ok());
}

#[pr4xis::praxis_value(Honest)]
#[test]
fn incompatible_unit_conversion_fails() {
    assert!(IncompatibleUnitConversionFails.verify().is_ok());
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn f_equals_ma_dimension() {
    // F = m*a → [F] = [M]·[L·T⁻²] = M·L·T⁻²
    let m = Quantity::new(10.0, Dimension::MASS);
    let a = Quantity::new(9.8, Dimension::ACCELERATION);
    let f = m.mul(&a);
    assert_eq!(f.dimension, Dimension::FORCE);
    assert!((f.value - 98.0).abs() < 1e-10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn kinetic_energy_dimension() {
    // E = 0.5 * m * v² → [E] = [M]·[V]² = M·L²·T⁻²
    let m = Quantity::new(2.0, Dimension::MASS);
    let v = Quantity::new(3.0, Dimension::VELOCITY);
    let v_sq = v.power(2);
    let ke = m.mul(&v_sq).scale(0.5);
    assert_eq!(ke.dimension, Dimension::ENERGY);
    assert!((ke.value - 9.0).abs() < 1e-10);
}

#[pr4xis::praxis_value(Verifiable)]
#[test]
fn degree_radian_conversion() {
    let deg_val = 180.0;
    let rad_val = unit::DEGREE.convert(deg_val, &unit::RADIAN).unwrap();
    assert!((rad_val - core::f64::consts::PI).abs() < 1e-10);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    fn arb_dimension() -> impl Strategy<Value = Dimension> {
        (-3..4_i8, -3..4_i8, -3..4_i8).prop_map(|(l, m, t)| Dimension {
            length: l,
            mass: m,
            time: t,
            ..Dimension::DIMENSIONLESS
        })
    }

    fn _arb_quantity() -> impl Strategy<Value = (f64, Dimension)> {
        (-100.0..100.0_f64, arb_dimension())
    }

    proptest! {
        #[test]
        fn dimension_multiply_is_commutative(a in arb_dimension(), b in arb_dimension()) {
            prop_assert_eq!(a.multiply(&b), b.multiply(&a));
        }

        #[test]
        fn dimension_multiply_is_associative(
            a in arb_dimension(), b in arb_dimension(), c in arb_dimension(),
        ) {
            prop_assert_eq!(a.multiply(&b).multiply(&c), a.multiply(&b.multiply(&c)));
        }

        #[test]
        fn dimension_inverse_yields_dimensionless(d in arb_dimension()) {
            prop_assert!(d.multiply(&d.inverse()).is_dimensionless());
        }

        #[test]
        fn dimensionless_is_identity(d in arb_dimension()) {
            prop_assert_eq!(d.multiply(&Dimension::DIMENSIONLESS), d);
        }

        #[test]
        fn addition_same_dimension_succeeds(v1 in -100.0..100.0_f64, v2 in -100.0..100.0_f64) {
            let q1 = Quantity::new(v1, Dimension::LENGTH);
            let q2 = Quantity::new(v2, Dimension::LENGTH);
            let sum = q1.add(&q2);
            prop_assert!(sum.is_some());
            prop_assert!((sum.unwrap().value - (v1 + v2)).abs() < 1e-10);
        }

        #[test]
        fn addition_different_dimension_fails(v1 in -100.0..100.0_f64, v2 in -100.0..100.0_f64) {
            let q1 = Quantity::new(v1, Dimension::LENGTH);
            let q2 = Quantity::new(v2, Dimension::TIME);
            prop_assert!(q1.add(&q2).is_none());
        }

        #[test]
        fn multiplication_dimension_is_sum_of_exponents(
            a in arb_dimension(), b in arb_dimension(),
        ) {
            let product = a.multiply(&b);
            prop_assert_eq!(product.length, a.length + b.length);
            prop_assert_eq!(product.mass, a.mass + b.mass);
            prop_assert_eq!(product.time, a.time + b.time);
        }

        #[test]
        fn unit_conversion_roundtrip(val in 0.1..1000.0_f64) {
            let m = unit::KILOMETER.to_si(val);
            let back = unit::KILOMETER.from_si(m);
            prop_assert!((val - back).abs() < 1e-10);
        }

        /// Every unit is an invertible conversion: `from_si(to_si(v)) == v`,
        /// across a representative spread of scales/offsets.
        #[test]
        fn all_units_roundtrip(val in -1e6..1e6_f64) {
            let units = [
                unit::METER, unit::KILOMETER, unit::METER_PER_SECOND, unit::KNOT,
                unit::DEGREE, unit::RADIAN, unit::ARCSECOND, unit::HERTZ,
                unit::NAT, unit::SHANNON,
                unit::MILLIVOLT, unit::PICOSIEMENS, unit::RAYL, unit::NANOMETER,
                unit::MILLISECOND, unit::MINUTE, unit::PASCAL, unit::KELVIN, unit::CELSIUS,
                unit::RECIPROCAL_METER,
            ];
            for u in units {
                let back = u.from_si(u.to_si(val));
                prop_assert!((val - back).abs() < 1e-6 * val.abs().max(1.0),
                    "unit {} failed round-trip at {}", u.symbol, val);
            }
        }

        /// The Celsius↔Kelvin affine relation holds exactly: `K = °C + 273.15`.
        /// (A ∀-guard for the offset-sign class of unit bug.)
        #[test]
        fn celsius_kelvin_affine(celsius in -273.15..1000.0_f64) {
            let kelvin = unit::CELSIUS.to_si(celsius);
            prop_assert!((kelvin - (celsius + 273.15)).abs() < 1e-9);
            prop_assert!(kelvin >= 0.0, "a temperature at or above absolute zero must be >= 0 K");
        }

        #[test]
        fn quantity_is_deterministic(v in -100.0..100.0_f64) {
            let q1 = Quantity::new(v, Dimension::VELOCITY);
            let q2 = Quantity::new(v, Dimension::VELOCITY);
            let s1 = q1.mul(&Quantity::new(2.0, Dimension::TIME));
            let s2 = q2.mul(&Quantity::new(2.0, Dimension::TIME));
            prop_assert_eq!(s1.value.to_bits(), s2.value.to_bits());
            prop_assert_eq!(s1.dimension, s2.dimension);
        }
    }

    pr4xis::register_praxis_value!(dimension_multiply_is_commutative, Verifiable);
    pr4xis::register_praxis_value!(dimension_multiply_is_associative, Verifiable);
    pr4xis::register_praxis_value!(dimension_inverse_yields_dimensionless, Verifiable);
    pr4xis::register_praxis_value!(dimensionless_is_identity, Verifiable);
    pr4xis::register_praxis_value!(addition_same_dimension_succeeds, Verifiable);
    pr4xis::register_praxis_value!(addition_different_dimension_fails, Honest);
    pr4xis::register_praxis_value!(multiplication_dimension_is_sum_of_exponents, Verifiable);
    pr4xis::register_praxis_value!(unit_conversion_roundtrip, Deterministic);
    pr4xis::register_praxis_value!(all_units_roundtrip, Verifiable);
    pr4xis::register_praxis_value!(celsius_kelvin_affine, Verifiable);
    pr4xis::register_praxis_value!(quantity_is_deterministic, Deterministic);
}
