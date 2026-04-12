use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::science::math::quantity::dimension::Dimension;
use crate::science::math::quantity::ontology::*;
use crate::science::math::quantity::unit;
use crate::science::math::quantity::value::Quantity;

#[test]
fn dimension_category_laws() {
    check_category_laws::<DimensionCategory>().unwrap();
}

#[test]
fn quantity_ontology_validates() {
    QuantityOntology::validate().unwrap();
}

#[test]
fn dimension_commutativity() {
    assert!(DimensionCommutativity.holds());
}

#[test]
fn dimension_associativity() {
    assert!(DimensionAssociativity.holds());
}

#[test]
fn dimension_identity() {
    assert!(DimensionIdentity.holds());
}

#[test]
fn dimension_inverse() {
    assert!(DimensionInverse.holds());
}

#[test]
fn addition_requires_same_dimension() {
    assert!(AdditionRequiresSameDimension.holds());
}

#[test]
fn derived_dimension_consistency() {
    assert!(DerivedDimensionConsistency.holds());
}

#[test]
fn unit_conversion_roundtrip() {
    assert!(UnitConversionRoundtrip.holds());
}

#[test]
fn incompatible_unit_conversion_fails() {
    assert!(IncompatibleUnitConversionFails.holds());
}

#[test]
fn f_equals_ma_dimension() {
    // F = m*a → [F] = [M]·[L·T⁻²] = M·L·T⁻²
    let m = Quantity::new(10.0, Dimension::MASS);
    let a = Quantity::new(9.8, Dimension::ACCELERATION);
    let f = m.mul(&a);
    assert_eq!(f.dimension, Dimension::FORCE);
    assert!((f.value - 98.0).abs() < 1e-10);
}

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

#[test]
fn degree_radian_conversion() {
    let deg_val = 180.0;
    let rad_val = unit::DEGREE.convert(deg_val, &unit::RADIAN).unwrap();
    assert!((rad_val - std::f64::consts::PI).abs() < 1e-10);
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
}
