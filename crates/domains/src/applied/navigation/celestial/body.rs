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

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "CelestialBody",
    source: "Wertz (2001); Bowditch (2002)",
    being: PhysicalEndurant,

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

/// Quality: approximate magnitude (brightness) of the body as seen from Earth.
#[derive(Debug, Clone)]
pub struct ApparentMagnitude;

impl Quality for ApparentMagnitude {
    type Individual = CelestialBodyConcept;
    type Value = &'static str;

    fn get(&self, body: &CelestialBodyConcept) -> Option<&'static str> {
        Some(match body {
            CelestialBodyConcept::Body => "varies",
            CelestialBodyConcept::Sun => "-26.74",
            CelestialBodyConcept::Moon => "-12.74 (full)",
            CelestialBodyConcept::Star => "varies (e.g., Sirius -1.46, Polaris 1.98)",
            CelestialBodyConcept::Planet => "varies (e.g., Venus -4.9, Jupiter -2.9)",
        })
    }
}

impl Ontology for CelestialBodyOntology {
    type Cat = CelestialBodyCategory;
    type Qual = ApparentMagnitude;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Concept;

    #[test]
    fn has_five_concepts() {
        assert_eq!(CelestialBodyConcept::variants().len(), 5);
    }

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<CelestialBodyCategory>().unwrap();
    }
}
