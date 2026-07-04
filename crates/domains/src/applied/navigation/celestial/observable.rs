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

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "CelestialObservable",
    source: "Bowditch (2002)",

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

/// The astronomical coordinate frame a celestial observable is measured in.
///
/// A closed taxonomy from spherical astronomy (Bowditch 2002 ch. 17): altitude
/// and azimuth are the observer-local *horizontal* frame; hour angle and
/// declination are the *equatorial* frame. The abstract root observable belongs
/// to no single frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CelestialFrame {
    /// Observer-local horizontal frame (altitude / azimuth).
    Horizontal,
    /// Earth-centred equatorial frame (hour angle / declination).
    Equatorial,
}

/// Quality: the [`CelestialFrame`] each observable is measured in.
#[derive(Debug, Clone)]
pub struct CoordinateSystem;

impl Quality for CoordinateSystem {
    type Individual = CelestialObservableConcept;
    type Value = CelestialFrame;

    fn get(&self, obs: &CelestialObservableConcept) -> Option<CelestialFrame> {
        Some(match obs {
            // The abstract root belongs to no single frame.
            CelestialObservableConcept::Observable => return None,
            CelestialObservableConcept::Altitude | CelestialObservableConcept::Azimuth => {
                CelestialFrame::Horizontal
            }
            CelestialObservableConcept::HourAngle | CelestialObservableConcept::Declination => {
                CelestialFrame::Equatorial
            }
        })
    }
}

impl Ontology for CelestialObservableOntology {
    type Cat = CelestialObservableCategory;
    type Qual = CoordinateSystem;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>()
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
        assert_eq!(CelestialObservableConcept::variants().len(), 5);
    }

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<CelestialObservableCategory>();
    }
}
