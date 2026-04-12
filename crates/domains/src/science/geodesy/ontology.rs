use pr4xis::category::{Category, Entity, Functor, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::science::geodesy::conversion;
use crate::science::geodesy::coordinate::{Geodetic, Ned};
use crate::science::geodesy::ellipsoid;

// ---------------------------------------------------------------------------
// Entity: coordinate systems (Hilbert's "where things are")
// ---------------------------------------------------------------------------

/// Coordinate systems used in geodesy and navigation.
///
/// Source: Groves (2013), Chapter 2; Torge & Müller (2012), Chapter 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoordinateSystem {
    /// Geodetic (latitude, longitude, altitude on ellipsoid).
    Geodetic,
    /// Earth-Centered Earth-Fixed Cartesian.
    ECEF,
    /// North-East-Down local tangent plane.
    NED,
    /// East-North-Up local tangent plane.
    ENU,
}

impl Entity for CoordinateSystem {
    fn variants() -> Vec<Self> {
        vec![Self::Geodetic, Self::ECEF, Self::NED, Self::ENU]
    }
}

/// A conversion between coordinate systems (morphism).
///
/// Each conversion is a functor: it preserves geometric structure.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CoordinateConversion {
    pub from: CoordinateSystem,
    pub to: CoordinateSystem,
}

impl Relationship for CoordinateConversion {
    type Object = CoordinateSystem;
    fn source(&self) -> CoordinateSystem {
        self.from
    }
    fn target(&self) -> CoordinateSystem {
        self.to
    }
}

/// The geodesy category: coordinate systems + conversions.
pub struct GeodesyCategory;

impl Category for GeodesyCategory {
    type Object = CoordinateSystem;
    type Morphism = CoordinateConversion;

    fn identity(obj: &CoordinateSystem) -> CoordinateConversion {
        CoordinateConversion {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &CoordinateConversion, g: &CoordinateConversion) -> Option<CoordinateConversion> {
        if f.to != g.from {
            return None;
        }
        Some(CoordinateConversion {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<CoordinateConversion> {
        let systems = CoordinateSystem::variants();
        systems
            .iter()
            .flat_map(|&from| {
                systems
                    .iter()
                    .map(move |&to| CoordinateConversion { from, to })
            })
            .collect()
    }
}

/// Quality: number of components.
#[derive(Debug, Clone)]
pub struct ComponentCount;

impl Quality for ComponentCount {
    type Individual = CoordinateSystem;
    type Value = usize;

    fn get(&self, cs: &CoordinateSystem) -> Option<usize> {
        Some(match cs {
            CoordinateSystem::Geodetic => 3, // lat, lon, alt
            CoordinateSystem::ECEF => 3,     // x, y, z
            CoordinateSystem::NED => 3,      // north, east, down
            CoordinateSystem::ENU => 3,      // east, north, up
        })
    }
}

// ---------------------------------------------------------------------------
// Functor: NED ↔ ENU
// ---------------------------------------------------------------------------

/// Functor: NED → ENU. Maps North-East-Down to East-North-Up.
///
/// Object map: NED → ENU
/// Morphism map: preserves all geometric relationships (distances, angles)
///
/// The transformation: (n, e, d) → (e, n, -d)
/// This is an isometry (distance-preserving) and an involution (self-inverse after two applications).
pub struct NedToEnuFunctor;

impl Functor for NedToEnuFunctor {
    type Source = GeodesyCategory;
    type Target = GeodesyCategory;

    fn map_object(obj: &CoordinateSystem) -> CoordinateSystem {
        match obj {
            CoordinateSystem::NED => CoordinateSystem::ENU,
            CoordinateSystem::ENU => CoordinateSystem::NED,
            other => *other,
        }
    }

    fn map_morphism(m: &CoordinateConversion) -> CoordinateConversion {
        CoordinateConversion {
            from: Self::map_object(&m.from),
            to: Self::map_object(&m.to),
        }
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Geodetic ↔ ECEF roundtrip: the conversion is invertible.
///
/// Source: Bowring (1976). The iterative algorithm converges for all Earth-like positions.
pub struct GeodeticEcefRoundtrip;

impl Axiom for GeodeticEcefRoundtrip {
    fn description(&self) -> &str {
        "geodetic -> ECEF -> geodetic roundtrip is identity (Bowring 1976)"
    }

    fn holds(&self) -> bool {
        let e = ellipsoid::wgs84();
        for geo in &canonical_geodetic_points() {
            let ecef = conversion::geodetic_to_ecef(geo, &e);
            let geo2 = conversion::ecef_to_geodetic(&ecef, &e);
            if (geo.lat - geo2.lat).abs() > 1e-10
                || (geo.lon - geo2.lon).abs() > 1e-10
                || (geo.alt - geo2.alt).abs() > 0.01
            // 1cm tolerance
            {
                return false;
            }
        }
        true
    }
}

/// NED ↔ ENU roundtrip: the functor is an involution.
pub struct NedEnuRoundtrip;

impl Axiom for NedEnuRoundtrip {
    fn description(&self) -> &str {
        "NED -> ENU -> NED roundtrip is identity (involution)"
    }

    fn holds(&self) -> bool {
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
                return false;
            }
        }
        true
    }
}

/// NED → ENU preserves distances (isometry).
pub struct NedEnuIsometry;

impl Axiom for NedEnuIsometry {
    fn description(&self) -> &str {
        "NED -> ENU conversion preserves Euclidean distance (isometry)"
    }

    fn holds(&self) -> bool {
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

        (dist_ned - dist_enu).abs() < 1e-12
    }
}

/// Great circle distance is symmetric: d(a,b) = d(b,a).
pub struct GreatCircleSymmetry;

impl Axiom for GreatCircleSymmetry {
    fn description(&self) -> &str {
        "great circle distance is symmetric: d(a,b) = d(b,a)"
    }

    fn holds(&self) -> bool {
        let e = ellipsoid::wgs84();
        let pts = canonical_geodetic_points();
        for a in &pts {
            for b in &pts {
                let d_ab = conversion::great_circle_distance(a, b, &e);
                let d_ba = conversion::great_circle_distance(b, a, &e);
                if (d_ab - d_ba).abs() > 1e-6 {
                    return false;
                }
            }
        }
        true
    }
}

/// Great circle distance to self is zero.
pub struct GreatCircleSelfZero;

impl Axiom for GreatCircleSelfZero {
    fn description(&self) -> &str {
        "great circle distance to self is zero"
    }

    fn holds(&self) -> bool {
        let e = ellipsoid::wgs84();
        for p in &canonical_geodetic_points() {
            if conversion::great_circle_distance(p, p, &e) > 1e-6 {
                return false;
            }
        }
        true
    }
}

/// Great circle distance satisfies triangle inequality.
pub struct GreatCircleTriangleInequality;

impl Axiom for GreatCircleTriangleInequality {
    fn description(&self) -> &str {
        "great circle distance satisfies triangle inequality"
    }

    fn holds(&self) -> bool {
        let e = ellipsoid::wgs84();
        let pts = canonical_geodetic_points();
        for a in &pts {
            for b in &pts {
                for c in &pts {
                    let ac = conversion::great_circle_distance(a, c, &e);
                    let ab = conversion::great_circle_distance(a, b, &e);
                    let bc = conversion::great_circle_distance(b, c, &e);
                    if ac > ab + bc + 1.0 {
                        // 1m tolerance for spherical approx
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// WGS84 ellipsoid parameters are consistent: b = a(1-f), e² = 2f - f².
pub struct Wgs84Consistency;

impl Axiom for Wgs84Consistency {
    fn description(&self) -> &str {
        "WGS84: b = a(1-f) and e² = 2f - f² (NIMA TR8350.2)"
    }

    fn holds(&self) -> bool {
        let e = ellipsoid::wgs84();
        let b_expected = 6_356_752.314_245_179; // known WGS84 semi-minor axis
        let b_computed = e.b();
        if (b_computed - b_expected).abs() > 0.001 {
            return false;
        }
        // e² should be approximately 0.00669437999014
        let e2 = e.e_squared();
        if (e2 - 0.006_694_379_990_14).abs() > 1e-12 {
            return false;
        }
        true
    }
}

/// Functor law: NedToEnu preserves identity morphisms.
pub struct NedEnuFunctorIdentity;

impl Axiom for NedEnuFunctorIdentity {
    fn description(&self) -> &str {
        "NED→ENU functor preserves identity: F(id_A) = id_{F(A)}"
    }

    fn holds(&self) -> bool {
        for obj in CoordinateSystem::variants() {
            let id_obj = GeodesyCategory::identity(&obj);
            let mapped = NedToEnuFunctor::map_morphism(&id_obj);
            let f_obj = NedToEnuFunctor::map_object(&obj);
            let id_f_obj = GeodesyCategory::identity(&f_obj);
            if mapped != id_f_obj {
                return false;
            }
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The geodesy ontology.
///
/// Founded on:
///   - NIMA TR8350.2 (2000). WGS84 specification.
///   - Torge & Müller (2012). *Geodesy* (4th ed.).
///   - Bowring, B.R. (1976). Geodetic ↔ ECEF conversion.
///   - Groves (2013). Navigation coordinate frames.
pub struct GeodesyOntology;

impl Ontology for GeodesyOntology {
    type Cat = GeodesyCategory;
    type Qual = ComponentCount;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(GeodeticEcefRoundtrip),
            Box::new(NedEnuRoundtrip),
            Box::new(NedEnuIsometry),
            Box::new(GreatCircleSymmetry),
            Box::new(GreatCircleSelfZero),
            Box::new(GreatCircleTriangleInequality),
            Box::new(Wgs84Consistency),
            Box::new(NedEnuFunctorIdentity),
        ]
    }
}

// ---------------------------------------------------------------------------
// Canonical test data
// ---------------------------------------------------------------------------

fn canonical_geodetic_points() -> Vec<Geodetic> {
    use std::f64::consts::FRAC_PI_4;
    vec![
        // Origin: equator / prime meridian
        Geodetic::new(0.0, 0.0, 0.0),
        // North pole
        Geodetic::new(std::f64::consts::FRAC_PI_2, 0.0, 0.0),
        // 45°N, 0°E
        Geodetic::new(FRAC_PI_4, 0.0, 0.0),
        // New York (approx)
        Geodetic::new(40.7_f64.to_radians(), -74.0_f64.to_radians(), 10.0),
        // Tokyo (approx)
        Geodetic::new(35.7_f64.to_radians(), 139.7_f64.to_radians(), 40.0),
        // Sydney (approx)
        Geodetic::new(-33.9_f64.to_radians(), 151.2_f64.to_radians(), 58.0),
        // High altitude (aircraft)
        Geodetic::new(51.5_f64.to_radians(), -0.1_f64.to_radians(), 10000.0),
    ]
}
