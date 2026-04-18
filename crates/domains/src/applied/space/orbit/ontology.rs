//! Classical orbital elements (Keplerian elements).
//!
//! Source: Vallado (2013), *Fundamentals of Astrodynamics and Applications*, 4th ed.

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Orbit",
    source: "Vallado (2013); Battin (1999)",
    being: Process,

    concepts: [SemiMajorAxis, Eccentricity, Inclination, RAAN, ArgPeriapsis, TrueAnomaly],

    labels: {
        SemiMajorAxis: ("en", "Semi-major axis", "Semi-major axis (km)."),
        Eccentricity: ("en", "Eccentricity", "Eccentricity (dimensionless)."),
        Inclination: ("en", "Inclination", "Inclination (rad)."),
        RAAN: ("en", "RAAN", "Right Ascension of Ascending Node (rad)."),
        ArgPeriapsis: ("en", "Argument of periapsis", "Argument of periapsis (rad)."),
        TrueAnomaly: ("en", "True anomaly", "True anomaly (rad)."),
    },
}

/// Quality: physical units for each orbital element.
#[derive(Debug, Clone)]
pub struct ElementUnit;

impl Quality for ElementUnit {
    type Individual = OrbitConcept;
    type Value = &'static str;

    fn get(&self, element: &OrbitConcept) -> Option<&'static str> {
        Some(match element {
            OrbitConcept::SemiMajorAxis => "km",
            OrbitConcept::Eccentricity => "dimensionless",
            OrbitConcept::Inclination => "rad",
            OrbitConcept::RAAN => "rad",
            OrbitConcept::ArgPeriapsis => "rad",
            OrbitConcept::TrueAnomaly => "rad",
        })
    }
}

/// Axiom: eccentricity must be in [0, 1) for elliptical orbits.
pub struct EccentricityBounded;

impl Axiom for EccentricityBounded {
    fn description(&self) -> &str {
        "eccentricity is in [0, 1) for elliptical (bound) orbits"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(
    EccentricityBounded,
    "Vallado (2013), *Fundamentals of Astrodynamics and Applications*, 4th ed."
);

/// Axiom: semi-major axis must be positive for bound orbits.
pub struct SemiMajorAxisPositive;

impl Axiom for SemiMajorAxisPositive {
    fn description(&self) -> &str {
        "semi-major axis is positive for bound orbits"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(
    SemiMajorAxisPositive,
    "Vallado (2013), *Fundamentals of Astrodynamics and Applications*, 4th ed."
);

impl Ontology for OrbitOntology {
    type Cat = OrbitCategory;
    type Qual = ElementUnit;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(EccentricityBounded),
            Box::new(SemiMajorAxisPositive),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<OrbitCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        OrbitOntology::validate().unwrap();
    }
}
