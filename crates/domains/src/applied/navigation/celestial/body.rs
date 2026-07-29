//! Celestial bodies — features of interest for celestial navigation.
//!
//! These are the FeatureOfInterest in SSN terms: what the celestial sensors
//! observe. Sun, Moon, catalog stars, planets. Extracted from the main
//! celestial ontology into its own module to eliminate the dual-enum smell
//! (primary ontology + manual TaxonomyDef).
//!
//! Source: Wertz (2001); Bowditch (2002).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality, QualityKind};

use crate::formal::math::quantity::value::Quantity;

pr4xis::ontology! {
    name: "CelestialBody",
    source: "Wertz (2001); Bowditch (2002)",

    concepts: [Body, Sun, Moon, Star, Planet],

    labels: {
        Body: ("en", "Celestial body", "Abstract celestial body — root of the taxonomy."),
        Sun: ("en", "Sun", "The Sun — central star of our solar system."),
        Moon: ("en", "Moon", "Earth's natural satellite."),
        Star: ("en", "Star", "A catalog star (e.g., Polaris, Sirius)."),
        Planet: ("en", "Planet", "A planet (e.g., Venus, Mars, Jupiter)."),
    },

    is_a: [
        (Sun, Body),
        (Moon, Body),
        (Star, Body),
        (Planet, Body),
    ],
}

/// Quality: apparent magnitude (brightness as seen from Earth) on the
/// dimensionless, logarithmic Pogson scale, as a [`Quantity`], NOT a prose
/// string.
///
/// `None` for the abstract `Body` ("varies", implementation-dependent) and
/// for `Star`/`Planet`: apparent magnitude is a genuinely per-instance
/// property of *which* star or planet (Sirius -1.46 vs. Polaris 1.98;
/// Venus -4.9 vs. Jupiter -2.9) — the taxon itself has no single fixed
/// value, so forcing one onto the CONCEPT would misrepresent the range.
/// `Sun` and `Moon` are singular bodies with a well-defined figure (the
/// Moon's is the full-moon value).
///
/// Source: Pogson, N. (1856). "Magnitudes of Thirty-six of the Minor
///         Planets for the First Day of each Month of the Year 1857."
///         MNRAS 17(1), 12-15 — the logarithmic magnitude scale.
#[derive(Debug, Clone)]
pub struct ApparentMagnitude;

impl Quality for ApparentMagnitude {
    type Individual = CelestialBodyConcept;
    type Value = Quantity;
    const KIND: QualityKind = QualityKind::Physical;

    fn get(&self, body: &CelestialBodyConcept) -> Option<Quantity> {
        Some(match body {
            CelestialBodyConcept::Body => return None,
            CelestialBodyConcept::Sun => Quantity::dimensionless(-26.74),
            // Full-moon value.
            CelestialBodyConcept::Moon => Quantity::dimensionless(-12.74),
            // Genuinely per-instance — no single figure for the taxon.
            CelestialBodyConcept::Star => return None,
            CelestialBodyConcept::Planet => return None,
        })
    }
}

impl Ontology for CelestialBodyOntology {
    type Cat = CelestialBodyCategory;
    type Qual = ApparentMagnitude;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
    }
}

/// A reference to the specific celestial body an observation was taken of.
///
/// The taxon (`Sun`/`Moon`/`Star`/`Planet`) alone loses per-instance
/// identity — "Polaris" and "Sirius" are both `Star`, "Venus" and "Jupiter"
/// are both `Planet` — so a real observation needs the ontology CONCEPT
/// (for taxon-level reasoning, e.g. `ApparentMagnitude` lookups on `Sun`/
/// `Moon`) plus an optional catalog identifier that disambiguates within a
/// taxon (a star or planet name; `Sun` and `Moon` are singular and need
/// none).
///
/// Source: Bowditch (2002), Nautical Almanac star/planet catalog entries.
#[derive(Debug, Clone, PartialEq)]
pub struct CelestialBodyRef {
    /// The taxon this observation belongs to.
    pub category: CelestialBodyConcept,
    /// Catalog identifier within the taxon (e.g. "Polaris", "Venus").
    /// `None` for singular bodies (`Sun`, `Moon`).
    pub catalog_name: Option<String>,
}

impl CelestialBodyRef {
    /// A named catalog star or planet.
    pub fn named(category: CelestialBodyConcept, name: &str) -> Self {
        Self {
            category,
            catalog_name: Some(name.to_string()),
        }
    }

    /// The Sun.
    pub fn sun() -> Self {
        Self {
            category: CelestialBodyConcept::Sun,
            catalog_name: None,
        }
    }

    /// The Moon.
    pub fn moon() -> Self {
        Self {
            category: CelestialBodyConcept::Moon,
            catalog_name: None,
        }
    }

    /// A catalog star.
    pub fn star(name: &str) -> Self {
        Self::named(CelestialBodyConcept::Star, name)
    }

    /// A named planet.
    pub fn planet(name: &str) -> Self {
        Self::named(CelestialBodyConcept::Planet, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn has_five_concepts() {
        assert_eq!(CelestialBodyConcept::variants().len(), 5);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<CelestialBodyCategory>();
    }
}
