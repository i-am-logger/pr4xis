//! Geodesy — coordinate frames and frame conversions on the WGS84 ellipsoid.
//!
//! Models the four coordinate systems used to locate things on and near
//! the Earth (Geodetic, ECEF, NED, ENU) as a discrete categorical layer.
//! The numerical conversions live in the sibling `conversion.rs`,
//! `coordinate.rs`, and `ellipsoid.rs` modules; this ontology captures
//! the categorical structure (which frames exist, the NED↔ENU isometric
//! involution, and the round-trip / metric axioms).
//!
//! # Literature
//!
//! - **NIMA TR8350.2 (2000)** *Department of Defense World Geodetic
//!   System 1984* — the WGS84 ellipsoid parameters (a, f) and the
//!   defining identities b = a(1−f), e² = 2f − f².
//! - **Torge & Müller (2012)** *Geodesy* (4th ed.), Chapter 5 — the
//!   geodetic, ECEF, and local-tangent-plane (NED/ENU) coordinate
//!   systems.
//! - **Bowring (1976)** "Transformation from spatial to geographical
//!   coordinates", *Survey Review* 23(181):323–327 — the iterative
//!   geodetic ↔ ECEF conversion used in the round-trip axiom.
//! - **Groves (2013)** *Principles of GNSS, Inertial, and Multisensor
//!   Integrated Navigation Systems* (2nd ed.) — the canonical
//!   navigation-frame taxonomy.

use pr4xis::category::{Category, Endofunctor, FinitelyGenerated, Functor};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::natural::geodesy::conversion;
use crate::natural::geodesy::coordinate::{Geodetic, Ned};
use crate::natural::geodesy::ellipsoid;

pr4xis::ontology! {
    name: "Geodesy",
    source: "NIMA TR8350.2 (2000) WGS84; Torge & Muller (2012) Geodesy; Bowring (1976) Transformation from spatial to geographical coordinates; Groves (2013) Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems",

    concepts: [Geodetic, Ecef, Ned, Enu],

    labels: {
        Geodetic: ("en", "Geodetic",
            "Torge & Muller (2012) §5: geodetic frame — (lat, lon, alt) on the WGS84 ellipsoid."),
        Ecef: ("en", "ECEF",
            "Torge & Muller (2012) §5: Earth-Centered Earth-Fixed Cartesian frame."),
        Ned: ("en", "NED",
            "Groves (2013): local tangent plane with axes (North, East, Down)."),
        Enu: ("en", "ENU",
            "Groves (2013): local tangent plane with axes (East, North, Up)."),
    },

    // NED ↔ ENU are dual orientations of the local tangent plane.
    // Encoded as Opposition (orientation flip is an involutive duality).
    opposes: [
        (Ned, Enu),
        (Enu, Ned),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: number of scalar components of a coordinate in this frame.
#[derive(Debug, Clone)]
pub struct ComponentCount;

impl Quality for ComponentCount {
    type Individual = GeodesyConcept;
    type Value = usize;

    fn get(&self, c: &GeodesyConcept) -> Option<usize> {
        Some(match c {
            GeodesyConcept::Geodetic => 3, // lat, lon, alt
            GeodesyConcept::Ecef => 3,     // x, y, z
            GeodesyConcept::Ned => 3,      // north, east, down
            GeodesyConcept::Enu => 3,      // east, north, up
        })
    }
}

// ---------------------------------------------------------------------------
// Endofunctor: NED ↔ ENU involution
// ---------------------------------------------------------------------------

/// Endofunctor NED ↔ ENU on `GeodesyCategory`.
///
/// Object map: NED ↔ ENU (other frames fixed).
/// Morphism map: rewrites endpoints, preserves kind.
///
/// The underlying coordinate transformation is `(n, e, d) → (e, n, -d)`,
/// which is an isometry (distance-preserving) and an involution
/// (self-inverse). Groves (2013) §2.1.
pub struct NedToEnuFunctor;

impl Functor for NedToEnuFunctor {
    type Source = GeodesyCategory;
    type Target = GeodesyCategory;

    fn map_object(obj: &GeodesyConcept) -> GeodesyConcept {
        match obj {
            GeodesyConcept::Ned => GeodesyConcept::Enu,
            GeodesyConcept::Enu => GeodesyConcept::Ned,
            other => *other,
        }
    }

    fn map_morphism(m: &GeodesyRelation) -> GeodesyRelation {
        GeodesyRelation {
            from: Self::map_object(&m.from),
            to: Self::map_object(&m.to),
            kind: m.kind,
        }
    }
}
pr4xis::register_functor!(NedToEnuFunctor);

impl Endofunctor for NedToEnuFunctor {
    type Category = GeodesyCategory;
}

// ---------------------------------------------------------------------------
// Axioms — round-trips, isometries, and ellipsoid identities.
// ---------------------------------------------------------------------------

/// Axiom: Geodetic → ECEF → Geodetic is identity (1e-10 rad / 1 cm tolerance).
/// Bowring (1976) — the iterative algorithm converges for Earth-like positions.
pub struct GeodeticEcefRoundtrip;

impl Axiom for GeodeticEcefRoundtrip {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let e = ellipsoid::wgs84();
        for geo in &canonical_geodetic_points() {
            let ecef = conversion::geodetic_to_ecef(geo, &e);
            let geo2 = conversion::ecef_to_geodetic(&ecef, &e);
            if (geo.lat - geo2.lat).abs() > 1e-10
                || (geo.lon - geo2.lon).abs() > 1e-10
                || (geo.alt - geo2.alt).abs() > 0.01
            {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "GeodeticEcefRoundtrip",
        "geodetic -> ECEF -> geodetic round-trip is identity to 1e-10 rad / 1 cm",
        "Bowring (1976) Survey Review 23(181):323"
    );
}
pr4xis::register_axiom!(
    GeodeticEcefRoundtrip,
    "Bowring (1976) Survey Review 23(181):323"
);

/// Axiom: NED → ENU → NED is identity (the swap is an involution).
/// Groves (2013) §2.1.
pub struct NedEnuRoundtrip;

impl Axiom for NedEnuRoundtrip {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let test_neds = [
            Ned {
                north: 1.0,
                east: 2.0,
                down: 3.0,
            },
            Ned {
                north: -5.0,
                east: 10.0,
                down: -0.5,
            },
            Ned {
                north: 0.0,
                east: 0.0,
                down: 0.0,
            },
        ];
        for ned in &test_neds {
            let enu = ned.to_enu();
            let ned2 = enu.to_ned();
            if (ned.north - ned2.north).abs() > 1e-15
                || (ned.east - ned2.east).abs() > 1e-15
                || (ned.down - ned2.down).abs() > 1e-15
            {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "NedEnuRoundtrip",
        "NED -> ENU -> NED round-trip is identity (involution)",
        "Groves (2013) §2.1"
    );
}
pr4xis::register_axiom!(NedEnuRoundtrip, "Groves (2013) §2.1");

/// Axiom: NED → ENU preserves Euclidean distance (the swap is an isometry).
/// Groves (2013) §2.1.
pub struct NedEnuIsometry;

impl Axiom for NedEnuIsometry {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let a = Ned {
            north: 1.0,
            east: 2.0,
            down: 3.0,
        };
        let b = Ned {
            north: 4.0,
            east: 6.0,
            down: -1.0,
        };
        let dist_ned =
            ((a.north - b.north).powi(2) + (a.east - b.east).powi(2) + (a.down - b.down).powi(2))
                .sqrt();
        let a_enu = a.to_enu();
        let b_enu = b.to_enu();
        let dist_enu = ((a_enu.east - b_enu.east).powi(2)
            + (a_enu.north - b_enu.north).powi(2)
            + (a_enu.up - b_enu.up).powi(2))
        .sqrt();
        if (dist_ned - dist_enu).abs() < 1e-12 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "NedEnuIsometry",
        "NED -> ENU conversion preserves Euclidean distance (isometry)",
        "Groves (2013) §2.1"
    );
}
pr4xis::register_axiom!(NedEnuIsometry, "Groves (2013) §2.1");

/// Axiom: great-circle distance is symmetric: d(a,b) = d(b,a).
/// Torge & Müller (2012) §5 — metric-space axiom on the ellipsoid.
pub struct GreatCircleSymmetry;

impl Axiom for GreatCircleSymmetry {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let e = ellipsoid::wgs84();
        let pts = canonical_geodetic_points();
        for a in &pts {
            for b in &pts {
                let d_ab = conversion::great_circle_distance(a, b, &e);
                let d_ba = conversion::great_circle_distance(b, a, &e);
                if (d_ab - d_ba).abs() > 1e-6 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "GreatCircleSymmetry",
        "great circle distance is symmetric: d(a,b) = d(b,a)",
        "Torge & Muller (2012) Geodesy §5"
    );
}
pr4xis::register_axiom!(GreatCircleSymmetry, "Torge & Muller (2012) Geodesy §5");

/// Axiom: d(p, p) = 0 — great-circle distance to self is zero.
/// Torge & Müller (2012) §5.
pub struct GreatCircleSelfZero;

impl Axiom for GreatCircleSelfZero {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let e = ellipsoid::wgs84();
        for p in &canonical_geodetic_points() {
            if conversion::great_circle_distance(p, p, &e) > 1e-6 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "GreatCircleSelfZero",
        "great circle distance to self is zero",
        "Torge & Muller (2012) Geodesy §5"
    );
}
pr4xis::register_axiom!(GreatCircleSelfZero, "Torge & Muller (2012) Geodesy §5");

/// Axiom: triangle inequality holds (1 m tolerance for spherical approx).
/// Torge & Müller (2012) §5.
pub struct GreatCircleTriangleInequality;

impl Axiom for GreatCircleTriangleInequality {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let e = ellipsoid::wgs84();
        let pts = canonical_geodetic_points();
        for a in &pts {
            for b in &pts {
                for c in &pts {
                    let ac = conversion::great_circle_distance(a, c, &e);
                    let ab = conversion::great_circle_distance(a, b, &e);
                    let bc = conversion::great_circle_distance(b, c, &e);
                    if ac > ab + bc + 1.0 {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "GreatCircleTriangleInequality",
        "great circle distance satisfies the triangle inequality (1 m tolerance)",
        "Torge & Muller (2012) Geodesy §5"
    );
}
pr4xis::register_axiom!(
    GreatCircleTriangleInequality,
    "Torge & Muller (2012) Geodesy §5"
);

/// Axiom: WGS84 ellipsoid parameters are consistent: b = a(1−f), e² = 2f − f².
/// NIMA TR8350.2 (2000).
pub struct Wgs84Consistency;

impl Axiom for Wgs84Consistency {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let e = ellipsoid::wgs84();
        let b_expected = 6_356_752.314_245_179; // canonical WGS84 semi-minor axis
        let b_computed = e.b();
        if (b_computed - b_expected).abs() > 0.001 {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        let e2 = e.e_squared();
        if (e2 - 0.006_694_379_990_14).abs() > 1e-12 {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "Wgs84Consistency",
        "WGS84: b = a(1-f) and e^2 = 2f - f^2",
        "NIMA TR8350.2 (2000) WGS84"
    );
}
pr4xis::register_axiom!(Wgs84Consistency, "NIMA TR8350.2 (2000) WGS84");

/// Axiom: NedToEnu functor preserves identity morphisms.
/// Mac Lane (1971) — functoriality.
pub struct NedEnuFunctorIdentity;

impl Axiom for NedEnuFunctorIdentity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for obj in GeodesyConcept::variants() {
            let id_obj = GeodesyCategory::identity(&obj);
            let mapped = NedToEnuFunctor::map_morphism(&id_obj);
            let f_obj = NedToEnuFunctor::map_object(&obj);
            let id_f_obj = GeodesyCategory::identity(&f_obj);
            if mapped != id_f_obj {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "NedEnuFunctorIdentity",
        "NED->ENU functor preserves identity: F(id_A) = id_{F(A)}",
        "Mac Lane (1971) Categories for the Working Mathematician II.1"
    );
}
pr4xis::register_axiom!(
    NedEnuFunctorIdentity,
    "Mac Lane (1971) Categories for the Working Mathematician II.1"
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for GeodesyOntology {
    type Cat = GeodesyCategory;
    type Qual = ComponentCount;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(GeodeticEcefRoundtrip));
        axioms.push(Box::new(NedEnuRoundtrip));
        axioms.push(Box::new(NedEnuIsometry));
        axioms.push(Box::new(GreatCircleSymmetry));
        axioms.push(Box::new(GreatCircleSelfZero));
        axioms.push(Box::new(GreatCircleTriangleInequality));
        axioms.push(Box::new(Wgs84Consistency));
        axioms.push(Box::new(NedEnuFunctorIdentity));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Canonical test data
// ---------------------------------------------------------------------------

fn canonical_geodetic_points() -> Vec<Geodetic> {
    use core::f64::consts::FRAC_PI_4;
    vec![
        Geodetic::new(0.0, 0.0, 0.0),
        Geodetic::new(std::f64::consts::FRAC_PI_2, 0.0, 0.0),
        Geodetic::new(FRAC_PI_4, 0.0, 0.0),
        Geodetic::new(40.7_f64.to_radians(), -74.0_f64.to_radians(), 10.0),
        Geodetic::new(35.7_f64.to_radians(), 139.7_f64.to_radians(), 40.0),
        Geodetic::new(-33.9_f64.to_radians(), 151.2_f64.to_radians(), 58.0),
        Geodetic::new(51.5_f64.to_radians(), -0.1_f64.to_radians(), 10000.0),
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Arrow;
    use pr4xis::category::laws::{assert_category_laws, assert_functor_laws};
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<GeodesyCategory>();
    }

    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn ned_to_enu_functor_laws() {
        assert_functor_laws::<NedToEnuFunctor>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        GeodesyOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn four_coordinate_systems() {
        assert_eq!(GeodesyConcept::variants().len(), 4);
    }

    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn ned_swap_returns_enu() {
        assert_eq!(
            NedToEnuFunctor::map_object(&GeodesyConcept::Ned),
            GeodesyConcept::Enu
        );
    }
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn enu_swap_returns_ned() {
        assert_eq!(
            NedToEnuFunctor::map_object(&GeodesyConcept::Enu),
            GeodesyConcept::Ned
        );
    }
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn geodetic_is_fixed() {
        assert_eq!(
            NedToEnuFunctor::map_object(&GeodesyConcept::Geodetic),
            GeodesyConcept::Geodetic
        );
    }
    #[pr4xis::praxis_value(Verifiable, Extensible)]
    #[test]
    fn ecef_is_fixed() {
        assert_eq!(
            NedToEnuFunctor::map_object(&GeodesyConcept::Ecef),
            GeodesyConcept::Ecef
        );
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn component_count_total() {
        let q = ComponentCount;
        for c in GeodesyConcept::variants() {
            assert_eq!(q.get(&c), Some(3));
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ned_enu_are_opposed() {
        let opposed: Vec<_> = GeodesyCategory::morphisms()
            .iter()
            .filter(|m| m.kind() == GeodesyRelationKind::Opposition)
            .map(|m| (m.source(), m.target()))
            .collect();
        assert!(opposed.contains(&(GeodesyConcept::Ned, GeodesyConcept::Enu)));
        assert!(opposed.contains(&(GeodesyConcept::Enu, GeodesyConcept::Ned)));
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn geodetic_ecef_roundtrip_holds() {
        assert!(GeodeticEcefRoundtrip.verify().is_ok());
    }
    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn ned_enu_roundtrip_holds() {
        assert!(NedEnuRoundtrip.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ned_enu_isometry_holds() {
        assert!(NedEnuIsometry.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn great_circle_symmetry_holds() {
        assert!(GreatCircleSymmetry.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn great_circle_self_zero_holds() {
        assert!(GreatCircleSelfZero.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn great_circle_triangle_inequality_holds() {
        assert!(GreatCircleTriangleInequality.verify().is_ok());
    }
    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn wgs84_consistency_holds() {
        assert!(Wgs84Consistency.verify().is_ok());
    }
    #[pr4xis::praxis_value(Extensible)]
    #[test]
    fn ned_enu_functor_identity_holds() {
        assert!(NedEnuFunctorIdentity.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = GeodesyConcept> {
        proptest::sample::select(GeodesyConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_component_count_total(c in arb_concept()) {
            prop_assert!(ComponentCount.get(&c).is_some());
        }

        #[test]
        fn prop_ned_enu_involutive(c in arb_concept()) {
            let once = NedToEnuFunctor::map_object(&c);
            let twice = NedToEnuFunctor::map_object(&once);
            prop_assert_eq!(twice, c);
        }

        #[test]
        fn prop_opposition_symmetric(_seed in any::<u32>()) {
            let opposed: std::collections::HashSet<_> = GeodesyCategory::morphisms()
                .iter()
                .filter(|m| m.kind() == GeodesyRelationKind::Opposition)
                .map(|m| (m.source(), m.target()))
                .collect();
            for (a, b) in opposed.iter() {
                prop_assert!(opposed.contains(&(*b, *a)),
                    "opposition not symmetric: {:?} -> {:?}", a, b);
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in GeodesyOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_component_count_total, Verifiable);
    pr4xis::register_praxis_value!(prop_ned_enu_involutive, Deterministic);
    pr4xis::register_praxis_value!(prop_opposition_symmetric, Verifiable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
