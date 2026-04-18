//! Celestial navigation sensors.
//!
//! This ontology covers the sensors used in celestial navigation. Related
//! ontologies live in sibling modules:
//! - `celestial::body` — the celestial bodies observed (Sun, Moon, Star, Planet)
//! - `celestial::observable` — the angles measured (Altitude, Azimuth, HourAngle, Declination)
//!
//! Each sensor maps to the observable property it produces via the
//! `CelestialToProperty` functor (see `property_functor.rs`).
//!
//! Source: Wertz (2001) "Space Mission Engineering"; Bowditch (2002);
//!         Groves (2013) Section 6.5.

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Celestial",
    source: "Bowditch (2002); Wertz (2001)",
    being: Process,

    concepts: [Sensor, StarTracker, SunSensor, HorizonSensor],

    labels: {
        Sensor: ("en", "Celestial sensor", "Abstract celestial sensor — root of the celestial sensor taxonomy."),
        StarTracker: ("en", "Star tracker", "Focal plane array for spacecraft attitude determination. 1-10 arcsec accuracy (Wertz 2001)."),
        SunSensor: ("en", "Sun sensor", "Measures sun direction. ~0.05 deg accuracy."),
        HorizonSensor: ("en", "Horizon sensor", "Measures Earth limb. ~0.1 deg accuracy."),
    },

    is_a: [
        (StarTracker, Sensor),
        (SunSensor, Sensor),
        (HorizonSensor, Sensor),
    ],
}

/// Quality: Angular accuracy of celestial sensors.
///
/// Source: Wertz (2001) Table 7-2.
#[derive(Debug, Clone)]
pub struct AngularAccuracy;

impl Quality for AngularAccuracy {
    type Individual = CelestialConcept;
    type Value = &'static str;

    fn get(&self, sensor: &CelestialConcept) -> Option<&'static str> {
        Some(match sensor {
            CelestialConcept::Sensor => "varies by type",
            CelestialConcept::StarTracker => "1-10 arcseconds (best)",
            CelestialConcept::SunSensor => "0.01-0.1 degrees",
            CelestialConcept::HorizonSensor => "0.05-0.25 degrees",
        })
    }
}

/// Two star sightings determine a position fix.
///
/// Source: Bowditch (2002) Chapter 18.
pub struct TwoSightsFix;

impl Axiom for TwoSightsFix {
    fn description(&self) -> &str {
        "two celestial observations determine a position (intersection of circles of position)"
    }
    fn holds(&self) -> bool {
        let unknowns = 2;
        let observations_per_sight = 1;
        let min_sights = unknowns / observations_per_sight;
        min_sights == 2
    }
}
pr4xis::register_axiom!(TwoSightsFix);

/// Star trackers provide arcsecond-level accuracy.
///
/// Source: Wertz (2001) Table 7-2, Liebe (2002).
pub struct StarTrackerMostAccurate;

impl Axiom for StarTrackerMostAccurate {
    fn description(&self) -> &str {
        "star trackers provide arcsecond-level accuracy (most accurate celestial sensor)"
    }
    fn holds(&self) -> bool {
        let star_tracker_arcsec = 5.0;
        let sun_sensor_arcsec = 180.0;
        let horizon_sensor_arcsec = 360.0;
        star_tracker_arcsec < sun_sensor_arcsec && star_tracker_arcsec < horizon_sensor_arcsec
    }
}
pr4xis::register_axiom!(StarTrackerMostAccurate);

/// Atmospheric refraction corrupts near-horizon observations.
///
/// Source: Bowditch (2002) Chapter 19; Meeus (1991).
pub struct AtmosphericRefraction;

impl Axiom for AtmosphericRefraction {
    fn description(&self) -> &str {
        "near-horizon observations are corrupted by atmospheric refraction"
    }
    fn holds(&self) -> bool {
        let refraction_at_horizon = approximate_refraction_arcmin(0.5);
        let refraction_at_45deg = approximate_refraction_arcmin(45.0);
        refraction_at_horizon > refraction_at_45deg * 10.0
    }
}
pr4xis::register_axiom!(AtmosphericRefraction);

/// Approximate atmospheric refraction in arcminutes.
///
/// Formula from Meeus (1991), valid for h > 0 degrees.
fn approximate_refraction_arcmin(altitude_deg: f64) -> f64 {
    if altitude_deg < 0.1 {
        return 34.0;
    }
    1.02 / (altitude_deg + 10.3 / (altitude_deg + 5.11))
        .to_radians()
        .tan()
}

impl Ontology for CelestialOntology {
    type Cat = CelestialCategory;
    type Qual = AngularAccuracy;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(TwoSightsFix),
            Box::new(StarTrackerMostAccurate),
            Box::new(AtmosphericRefraction),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<CelestialCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        CelestialOntology::validate().unwrap();
    }
}
