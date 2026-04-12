use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::formal::math::temporal::ontology::*;

#[test]
fn time_system_category_laws() {
    check_category_laws::<TimeSystemCategory>().unwrap();
}

#[test]
fn time_ontology_validates() {
    TimeOntology::validate().unwrap();
}

#[test]
fn total_order() {
    assert!(TotalOrder.holds());
}

#[test]
fn duration_non_negativity() {
    assert!(DurationNonNegativity.holds());
}

#[test]
fn duration_identity() {
    assert!(DurationIdentity.holds());
}

#[test]
fn duration_antisymmetry() {
    assert!(DurationAntisymmetry.holds());
}

#[test]
fn duration_additivity() {
    assert!(DurationAdditivity.holds());
}

#[test]
fn allen_exhaustive() {
    assert!(AllenExhaustive.holds());
}

#[test]
fn allen_inverse_law() {
    assert!(AllenInverseLaw.holds());
}

#[test]
fn gps_tai_conversion() {
    assert!(GpsTaiConversion.holds());
}

#[test]
fn tt_tai_conversion() {
    assert!(TtTaiConversion.holds());
}

#[cfg(test)]
mod proptest_proofs {
    use crate::formal::math::temporal::allen;
    use crate::formal::math::temporal::duration::Duration;
    use crate::formal::math::temporal::instant::Instant;
    use crate::formal::math::temporal::interval::Interval;
    use crate::formal::math::temporal::time_system::TimeSystem;
    use proptest::prelude::*;

    fn arb_instant_tai() -> impl Strategy<Value = Instant> {
        (0.0..1e9_f64).prop_map(|s| Instant::new(s, TimeSystem::TAI))
    }

    fn arb_duration() -> impl Strategy<Value = Duration> {
        (-1e6..1e6_f64).prop_map(Duration::from_seconds)
    }

    fn arb_positive_duration() -> impl Strategy<Value = Duration> {
        (1e-6..1e6_f64).prop_map(Duration::from_seconds)
    }

    proptest! {
        #[test]
        fn duration_to_self_is_zero(a in arb_instant_tai()) {
            let d = a.duration_to(&a).unwrap();
            prop_assert!(d.seconds().abs() < 1e-15);
        }

        #[test]
        fn duration_antisymmetry(a in arb_instant_tai(), b in arb_instant_tai()) {
            let d_ab = a.duration_to(&b).unwrap().seconds();
            let d_ba = b.duration_to(&a).unwrap().seconds();
            prop_assert!((d_ab + d_ba).abs() < 1e-10);
        }

        #[test]
        fn duration_additivity(
            a in arb_instant_tai(),
            b in arb_instant_tai(),
            c in arb_instant_tai(),
        ) {
            let ab = a.duration_to(&b).unwrap().seconds();
            let bc = b.duration_to(&c).unwrap().seconds();
            let ac = a.duration_to(&c).unwrap().seconds();
            prop_assert!((ab + bc - ac).abs() < 1e-6,
                "ab={}, bc={}, ac={}", ab, bc, ac);
        }

        #[test]
        fn advance_then_duration_roundtrip(
            a in arb_instant_tai(),
            dt in arb_positive_duration(),
        ) {
            let b = a.advance(&dt);
            let d = a.duration_to(&b).unwrap();
            // Relative tolerance for large timestamps + small durations (f64 ULP)
            let tol = 1e-10 + a.seconds.abs() * f64::EPSILON * 4.0;
            prop_assert!((d.seconds() - dt.seconds()).abs() < tol,
                "d={}, dt={}, tol={}", d.seconds(), dt.seconds(), tol);
        }

        #[test]
        fn advance_retreat_identity(
            a in arb_instant_tai(),
            dt in arb_duration(),
        ) {
            let b = a.advance(&dt);
            let c = b.retreat(&dt);
            let tol = 1e-10 + a.seconds.abs() * f64::EPSILON * 4.0;
            prop_assert!((c.seconds - a.seconds).abs() < tol);
        }

        #[test]
        fn duration_vector_space_additive_identity(dt in arb_duration()) {
            let sum = dt.add(&Duration::zero());
            prop_assert!((sum.seconds() - dt.seconds()).abs() < 1e-15);
        }

        #[test]
        fn duration_vector_space_additive_inverse(dt in arb_duration()) {
            let sum = dt.add(&dt.negate());
            prop_assert!(sum.seconds().abs() < 1e-10);
        }

        #[test]
        fn duration_vector_space_commutativity(
            a in arb_duration(),
            b in arb_duration(),
        ) {
            let ab = a.add(&b);
            let ba = b.add(&a);
            prop_assert!((ab.seconds() - ba.seconds()).abs() < 1e-10);
        }

        #[test]
        fn allen_inverse_holds(
            xb in 0.0..100.0_f64,
            xlen in 0.1..50.0_f64,
            yb in 0.0..100.0_f64,
            ylen in 0.1..50.0_f64,
        ) {
            let s = TimeSystem::TAI;
            let x = Interval::new(
                Instant::new(xb, s),
                Instant::new(xb + xlen, s),
            ).unwrap();
            let y = Interval::new(
                Instant::new(yb, s),
                Instant::new(yb + ylen, s),
            ).unwrap();
            let r_xy = allen::relate(&x, &y, 1e-10);
            let r_yx = allen::relate(&y, &x, 1e-10);
            prop_assert!(r_xy.inverse() == r_yx,
                "R(X,Y)={:?}, R(Y,X)={:?}", r_xy, r_yx);
        }

        #[test]
        fn gps_tai_roundtrip(t in 0.0..1e9_f64) {
            use crate::formal::math::temporal::time_system;
            let t_tai = time_system::convert(t, TimeSystem::GPS, TimeSystem::TAI).unwrap();
            let t_gps = time_system::convert(t_tai, TimeSystem::TAI, TimeSystem::GPS).unwrap();
            prop_assert!((t_gps - t).abs() < 1e-10);
        }

        #[test]
        fn gps_tt_via_tai(t in 0.0..1e9_f64) {
            use crate::formal::math::temporal::time_system;
            // GPS → TAI → TT should equal GPS → TT
            let tai = time_system::convert(t, TimeSystem::GPS, TimeSystem::TAI).unwrap();
            let tt_via_tai = time_system::convert(tai, TimeSystem::TAI, TimeSystem::TT).unwrap();
            let tt_direct = time_system::convert(t, TimeSystem::GPS, TimeSystem::TT).unwrap();
            prop_assert!((tt_via_tai - tt_direct).abs() < 1e-10);
        }
    }
}
