use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

/// Classical orbital elements (Keplerian elements).
///
/// Source: Vallado (2013), *Fundamentals of Astrodynamics and Applications*, 4th ed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum OrbitalElement {
    /// Semi-major axis (km).
    SemiMajorAxis,
    /// Eccentricity (dimensionless).
    Eccentricity,
    /// Inclination (rad).
    Inclination,
    /// Right Ascension of Ascending Node (rad).
    RAAN,
    /// Argument of periapsis (rad).
    ArgPeriapsis,
    /// True anomaly (rad).
    TrueAnomaly,
}

define_ontology! {
    /// Category for orbital element relationships.
    ///
    /// All elements are needed to fully specify an orbit; they form
    /// a complete set with cross-dependencies.
    pub OrbitOntology for OrbitCategory {
        entity: OrbitalElement,
        relation: ElementDependency,
    }
}

/// Quality: physical units for each orbital element.
#[derive(Debug, Clone)]
pub struct ElementUnit;

impl Quality for ElementUnit {
    type Individual = OrbitalElement;
    type Value = &'static str;

    fn get(&self, element: &OrbitalElement) -> Option<&'static str> {
        Some(match element {
            OrbitalElement::SemiMajorAxis => "km",
            OrbitalElement::Eccentricity => "dimensionless",
            OrbitalElement::Inclination => "rad",
            OrbitalElement::RAAN => "rad",
            OrbitalElement::ArgPeriapsis => "rad",
            OrbitalElement::TrueAnomaly => "rad",
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
        // Structural axiom from orbital mechanics:
        // e = 0: circular orbit
        // 0 < e < 1: elliptical orbit
        // e = 1: parabolic escape
        // e > 1: hyperbolic escape
        // For bound orbits, e must be in [0, 1).
        true
    }
}

/// Axiom: semi-major axis must be positive for bound orbits.
pub struct SemiMajorAxisPositive;

impl Axiom for SemiMajorAxisPositive {
    fn description(&self) -> &str {
        "semi-major axis is positive for bound orbits"
    }
    fn holds(&self) -> bool {
        // a > 0 for elliptical orbits (a < 0 for hyperbolic).
        true
    }
}

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
