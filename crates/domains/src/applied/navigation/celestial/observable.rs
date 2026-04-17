//! Celestial observables — the angular measurements taken during celestial
//! navigation.
//!
//! These specialize concepts from the shared `ObservableProperty` ontology:
//! - Altitude is an Elevation (angle above horizon)
//! - Azimuth is a Bearing (angle from north)
//! - HourAngle and Declination are celestial-specific equatorial coordinates
//!
//! The `CelestialObservableToProperty` functor expresses the specialization.
//!
//! Source: Bowditch (2002) Chapter 17.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "CelestialObservable",
    source: "Bowditch (2002)",
    being: Quality,

    concepts: [Observable, Altitude, Azimuth, HourAngle, Declination],

    labels: {
        Observable: ("en", "Celestial observable", "Abstract celestial observable — root of the taxonomy."),
        Altitude: ("en", "Altitude", "Elevation angle above the horizon."),
        Azimuth: ("en", "Azimuth", "Bearing from north."),
        HourAngle: ("en", "Hour angle", "Angular distance from the meridian (equatorial coordinate)."),
        Declination: ("en", "Declination", "Angular distance from the celestial equator (equatorial coordinate)."),
    },

    is_a: [
        (Altitude, Observable),
        (Azimuth, Observable),
        (HourAngle, Observable),
        (Declination, Observable),
    ],
}

/// Quality: coordinate system the observable lives in.
#[derive(Debug, Clone)]
pub struct CoordinateSystem;

impl Quality for CoordinateSystem {
    type Individual = CelestialObservableConcept;
    type Value = &'static str;

    fn get(&self, obs: &CelestialObservableConcept) -> Option<&'static str> {
        Some(match obs {
            CelestialObservableConcept::Observable => "varies",
            CelestialObservableConcept::Altitude | CelestialObservableConcept::Azimuth => {
                "horizontal (alt/az)"
            }
            CelestialObservableConcept::HourAngle | CelestialObservableConcept::Declination => {
                "equatorial"
            }
        })
    }
}

impl Ontology for CelestialObservableOntology {
    type Cat = CelestialObservableCategory;
    type Qual = CoordinateSystem;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::Entity;

    #[test]
    fn has_five_concepts() {
        assert_eq!(CelestialObservableConcept::variants().len(), 5);
    }

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<CelestialObservableCategory>().unwrap();
    }
}
