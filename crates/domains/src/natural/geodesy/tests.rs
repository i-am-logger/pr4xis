use pr4xis::category::validate::{check_category_laws, check_endofunctor_laws};
use pr4xis::ontology::{Axiom, Ontology};

use crate::natural::geodesy::ontology::*;

#[test]
fn geodesy_category_laws() {
    check_category_laws::<GeodesyCategory>().unwrap();
}

#[test]
fn ned_to_enu_endofunctor_laws() {
    check_endofunctor_laws::<NedToEnuFunctor>().unwrap();
}

#[test]
fn geodesy_ontology_validates() {
    GeodesyOntology::validate().unwrap();
}

#[test]
fn geodetic_ecef_roundtrip() {
    assert!(GeodeticEcefRoundtrip.holds());
}

#[test]
fn ned_enu_roundtrip() {
    assert!(NedEnuRoundtrip.holds());
}

#[test]
fn ned_enu_isometry() {
    assert!(NedEnuIsometry.holds());
}

#[test]
fn great_circle_symmetry() {
    assert!(GreatCircleSymmetry.holds());
}

#[test]
fn great_circle_self_zero() {
    assert!(GreatCircleSelfZero.holds());
}

#[test]
fn great_circle_triangle_inequality() {
    assert!(GreatCircleTriangleInequality.holds());
}

#[test]
fn wgs84_consistency() {
    assert!(Wgs84Consistency.holds());
}

#[test]
fn ned_enu_functor_identity() {
    assert!(NedEnuFunctorIdentity.holds());
}

#[cfg(test)]
mod proptest_proofs {
    use crate::natural::geodesy::conversion;
    use crate::natural::geodesy::coordinate::{Geodetic, Ned};
    use crate::natural::geodesy::ellipsoid;
    use proptest::prelude::*;

    fn arb_geodetic() -> impl Strategy<Value = Geodetic> {
        (
            -89.0..89.0_f64,     // lat degrees (avoid poles for numerical stability)
            -180.0..180.0_f64,   // lon degrees
            -500.0..50000.0_f64, // alt meters
        )
            .prop_map(|(lat, lon, alt)| Geodetic::new(lat.to_radians(), lon.to_radians(), alt))
    }

    fn arb_ned() -> impl Strategy<Value = Ned> {
        (
            -1000.0..1000.0_f64,
            -1000.0..1000.0_f64,
            -1000.0..1000.0_f64,
        )
            .prop_map(|(n, e, d)| Ned {
                north: n,
                east: e,
                down: d,
            })
    }

    proptest! {
        #[test]
        fn geodetic_ecef_roundtrip(geo in arb_geodetic()) {
            let e = ellipsoid::wgs84();
            let ecef = conversion::geodetic_to_ecef(&geo, &e);
            let geo2 = conversion::ecef_to_geodetic(&ecef, &e);
            prop_assert!((geo.lat - geo2.lat).abs() < 1e-9,
                "lat: {} vs {}", geo.lat, geo2.lat);
            prop_assert!((geo.lon - geo2.lon).abs() < 1e-9,
                "lon: {} vs {}", geo.lon, geo2.lon);
            prop_assert!((geo.alt - geo2.alt).abs() < 0.1,
                "alt: {} vs {}", geo.alt, geo2.alt);
        }

        #[test]
        fn ned_enu_roundtrip(ned in arb_ned()) {
            let enu = ned.to_enu();
            let ned2 = enu.to_ned();
            prop_assert!((ned.north - ned2.north).abs() < 1e-15);
            prop_assert!((ned.east - ned2.east).abs() < 1e-15);
            prop_assert!((ned.down - ned2.down).abs() < 1e-15);
        }

        #[test]
        fn ned_enu_preserves_distance(a in arb_ned(), b in arb_ned()) {
            let dist_ned = ((a.north - b.north).powi(2)
                + (a.east - b.east).powi(2)
                + (a.down - b.down).powi(2))
            .sqrt();
            let a_enu = a.to_enu();
            let b_enu = b.to_enu();
            let dist_enu = ((a_enu.east - b_enu.east).powi(2)
                + (a_enu.north - b_enu.north).powi(2)
                + (a_enu.up - b_enu.up).powi(2))
            .sqrt();
            prop_assert!((dist_ned - dist_enu).abs() < 1e-10);
        }

        #[test]
        fn great_circle_distance_is_symmetric(a in arb_geodetic(), b in arb_geodetic()) {
            let e = ellipsoid::wgs84();
            let d_ab = conversion::great_circle_distance(&a, &b, &e);
            let d_ba = conversion::great_circle_distance(&b, &a, &e);
            prop_assert!((d_ab - d_ba).abs() < 1.0); // 1m tolerance
        }

        #[test]
        fn great_circle_distance_non_negative(a in arb_geodetic(), b in arb_geodetic()) {
            let e = ellipsoid::wgs84();
            let d = conversion::great_circle_distance(&a, &b, &e);
            prop_assert!(d >= 0.0);
        }

        #[test]
        fn great_circle_to_self_is_zero(a in arb_geodetic()) {
            let e = ellipsoid::wgs84();
            let d = conversion::great_circle_distance(&a, &a, &e);
            prop_assert!(d < 1e-6);
        }

        #[test]
        fn ecef_is_on_ellipsoid_surface_when_alt_zero(
            lat in -89.0..89.0_f64,
            lon in -180.0..180.0_f64,
        ) {
            let e = ellipsoid::wgs84();
            let geo = Geodetic::new(lat.to_radians(), lon.to_radians(), 0.0);
            let ecef = conversion::geodetic_to_ecef(&geo, &e);
            // Point should satisfy ellipsoid equation: (x²+y²)/a² + z²/b² ≈ 1
            let a2 = e.a * e.a;
            let b2 = e.b() * e.b();
            let ellipsoid_eq = (ecef.x * ecef.x + ecef.y * ecef.y) / a2 + ecef.z * ecef.z / b2;
            prop_assert!((ellipsoid_eq - 1.0).abs() < 1e-8,
                "ellipsoid equation = {} (should be 1.0)", ellipsoid_eq);
        }
    }
}
