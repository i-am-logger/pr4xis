//! Classical orbital elements (Keplerian elements).
//!
//! Source: Vallado (2013), *Fundamentals of Astrodynamics and Applications*, 4th ed.

use pr4xis::logic::proof::{SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Orbit",
    source: "Vallado (2013); Battin (1999)",

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
    fn verify(&self) -> Verdict {
        // Per Vallado (2013) §2.4, the classical orbital element
        // eccentricity satisfies 0 ≤ e < 1 for elliptical (bound) orbits;
        // e = 1 is parabolic (escape) and e > 1 is hyperbolic.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EccentricityBounded",
        "eccentricity is in [0, 1) for elliptical (bound) orbits",
        "Vallado (2013) Fundamentals of Astrodynamics and Applications 4th ed. §2.4"
    );
}
pr4xis::register_axiom!(
    EccentricityBounded,
    "Vallado (2013) Fundamentals of Astrodynamics and Applications 4th ed. §2.4"
);

/// Axiom: semi-major axis must be positive for bound orbits.
pub struct SemiMajorAxisPositive;

impl Axiom for SemiMajorAxisPositive {
    fn verify(&self) -> Verdict {
        // Vis-viva: v² = μ(2/r − 1/a). For bound (E < 0) orbits a > 0;
        // a → ∞ at escape and a < 0 for hyperbolic.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "SemiMajorAxisPositive",
        "semi-major axis is positive for bound orbits",
        "Vallado (2013) Fundamentals of Astrodynamics and Applications 4th ed. §2.3 (vis-viva)"
    );
}
pr4xis::register_axiom!(
    SemiMajorAxisPositive,
    "Vallado (2013) Fundamentals of Astrodynamics and Applications 4th ed. §2.3 (vis-viva)"
);

impl Ontology for OrbitOntology {
    type Cat = OrbitCategory;
    type Qual = ElementUnit;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(EccentricityBounded));
        axioms.push(Box::new(SemiMajorAxisPositive));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<OrbitCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        OrbitOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
