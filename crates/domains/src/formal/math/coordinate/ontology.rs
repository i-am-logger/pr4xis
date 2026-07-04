//! CoordinateSystem — the ISO 19111 coordinate-system-type taxonomy, with the
//! conversions between the typed representations discharged as axioms.
//!
//! # Literature
//!
//! - **ISO 19111:2019** *Geographic information — Referencing by coordinates*
//!   (= OGC Abstract Specification Topic 2) — the coordinate-system types
//!   Cartesian, polar, spherical, ellipsoidal, cylindrical.

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::angle::Angle;
use crate::formal::math::coordinate::{PolarCoordinate, SphericalCoordinate};

pr4xis::ontology! {
    name: "CoordinateSystem",
    source: "ISO 19111:2019 Geographic information — Referencing by coordinates; OGC Abstract Specification Topic 2: Referencing by coordinates",

    concepts: [Cartesian, Polar, Spherical, Ellipsoidal, Cylindrical],

    labels: {
        Cartesian: ("en", "Cartesian",
            "ISO 19111: position by mutually orthogonal straight axes (2- or 3-D) — the Euclidean representation, a plain vector."),
        Polar: ("en", "Polar",
            "ISO 19111: 2-D — distance from the origin and the angle from a reference direction."),
        Spherical: ("en", "Spherical",
            "ISO 19111: 3-D — distance from the origin and two angles (azimuth and elevation/inclination)."),
        Ellipsoidal: ("en", "Ellipsoidal",
            "ISO 19111: position by geodetic latitude, geodetic longitude and (3-D) ellipsoidal height."),
        Cylindrical: ("en", "Cylindrical",
            "ISO 19111: 3-D — a polar coordinate system extended by a straight axis perpendicular to the polar plane."),
    },
}

/// Quality: how many of a coordinate system's axes are **angular** — the
/// property that separates the Euclidean system (Cartesian, 0) from the
/// non-Euclidean ones (where component-wise vector arithmetic is meaningless).
#[derive(Debug, Clone)]
pub struct AngularAxes;

impl Quality for AngularAxes {
    type Individual = CoordinateSystemConcept;
    type Value = u8;

    fn get(&self, cs: &CoordinateSystemConcept) -> Option<u8> {
        use CoordinateSystemConcept as C;
        Some(match cs {
            C::Cartesian => 0,
            C::Polar | C::Cylindrical => 1,
            C::Spherical | C::Ellipsoidal => 2,
        })
    }
}

impl Ontology for CoordinateSystemOntology {
    type Cat = CoordinateSystemCategory;
    type Qual = AngularAxes;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(CartesianIsTheEuclideanSystem));
        axioms.push(Box::new(PolarRoundTripsThroughCartesian));
        axioms.push(Box::new(SphericalRoundTripsThroughCartesian));
        axioms
    }
}

/// Axiom: Cartesian is the unique coordinate system with no angular axis — the
/// one, and only one, faithfully representable as a plain Euclidean `Vector`.
///
/// This is why the non-Euclidean systems get their own types (a `Vector` of a
/// spherical `(range, azimuth, elevation)` would let you `add`/`dot` a length
/// and two angles, which is meaningless). Verified over every concept.
pub struct CartesianIsTheEuclideanSystem;

impl Axiom for CartesianIsTheEuclideanSystem {
    fn verify(&self) -> Verdict {
        use pr4xis::category::FinitelyGenerated;
        let q = AngularAxes;
        let unique = CoordinateSystemConcept::variants()
            .iter()
            .all(|cs| (q.get(cs) == Some(0)) == (*cs == CoordinateSystemConcept::Cartesian));
        if unique {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CartesianIsTheEuclideanSystem",
        "Cartesian is the unique coordinate system with zero angular axes — the only one representable as a plain Euclidean vector",
        "ISO 19111:2019 Geographic information — Referencing by coordinates"
    );
}
pr4xis::register_axiom!(
    CartesianIsTheEuclideanSystem,
    "ISO 19111:2019 Geographic information — Referencing by coordinates"
);

/// Axiom: the polar↔Cartesian conversion is a bijection — `from_cartesian ∘
/// to_cartesian` is the identity on polar coordinates (within tolerance).
pub struct PolarRoundTripsThroughCartesian;

impl Axiom for PolarRoundTripsThroughCartesian {
    fn verify(&self) -> Verdict {
        let fixtures = [(10.0, 0.0), (500.0, 0.7), (1200.0, 2.5), (50.0, -1.1)];
        let ok = fixtures.iter().all(|&(range, azimuth)| {
            let p = PolarCoordinate::new(range, Angle::from_radians(azimuth));
            let back = PolarCoordinate::from_cartesian(&p.to_cartesian());
            (p.range - back.range).abs() < 1e-9
                && p.azimuth.difference(&back.azimuth).radians().abs() < 1e-9
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "PolarRoundTripsThroughCartesian",
        "polar → Cartesian → polar is the identity (the conversion is a bijection off the origin)",
        "ISO 19111:2019 Geographic information — Referencing by coordinates"
    );
}
pr4xis::register_axiom!(
    PolarRoundTripsThroughCartesian,
    "ISO 19111:2019 Geographic information — Referencing by coordinates"
);

/// Axiom: the spherical↔Cartesian conversion is a bijection away from the
/// origin and the poles.
pub struct SphericalRoundTripsThroughCartesian;

impl Axiom for SphericalRoundTripsThroughCartesian {
    fn verify(&self) -> Verdict {
        let fixtures = [(100.0, 0.0, 0.0), (1000.0, 1.2, 0.3), (500.0, -2.0, -0.4)];
        let ok = fixtures.iter().all(|&(range, azimuth, elevation)| {
            let s = SphericalCoordinate::new(
                range,
                Angle::from_radians(azimuth),
                Angle::from_radians(elevation),
            );
            let back = SphericalCoordinate::from_cartesian(&s.to_cartesian());
            (s.range - back.range).abs() < 1e-9
                && s.azimuth.difference(&back.azimuth).radians().abs() < 1e-9
                && s.elevation.difference(&back.elevation).radians().abs() < 1e-9
        });
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SphericalRoundTripsThroughCartesian",
        "spherical → Cartesian → spherical is the identity away from the origin/poles (the conversion is a bijection)",
        "ISO 19111:2019 Geographic information — Referencing by coordinates"
    );
}
pr4xis::register_axiom!(
    SphericalRoundTripsThroughCartesian,
    "ISO 19111:2019 Geographic information — Referencing by coordinates"
);

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<CoordinateSystemCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        CoordinateSystemOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn five_iso19111_systems() {
        assert_eq!(CoordinateSystemConcept::variants().len(), 5);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn conversion_axioms_hold() {
        assert!(CartesianIsTheEuclideanSystem.verify().is_ok());
        assert!(PolarRoundTripsThroughCartesian.verify().is_ok());
        assert!(SphericalRoundTripsThroughCartesian.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn angular_axes_total() {
        let q = AngularAxes;
        for cs in CoordinateSystemConcept::variants() {
            assert!(q.get(&cs).is_some());
        }
    }
}
