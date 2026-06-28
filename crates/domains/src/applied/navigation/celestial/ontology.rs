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

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Celestial",
    source: "Bowditch (2002); Wertz (2001)",

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
    fn verify(&self) -> Verdict {
        let unknowns = 2;
        let observations_per_sight = 1;
        let min_sights = unknowns / observations_per_sight;
        if min_sights == 2 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "TwoSightsFix",
        "two celestial observations determine a position (intersection of circles of position)",
        "Bowditch (2002) Chapter 18"
    );
}
pr4xis::register_axiom!(TwoSightsFix, "Bowditch (2002) Chapter 18");

/// Star trackers provide arcsecond-level accuracy.
///
/// Source: Wertz (2001) Table 7-2, Liebe (2002).
pub struct StarTrackerMostAccurate;

impl Axiom for StarTrackerMostAccurate {
    fn verify(&self) -> Verdict {
        let star_tracker_arcsec = 5.0;
        let sun_sensor_arcsec = 180.0;
        let horizon_sensor_arcsec = 360.0;
        if star_tracker_arcsec < sun_sensor_arcsec && star_tracker_arcsec < horizon_sensor_arcsec {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StarTrackerMostAccurate",
        "star trackers provide arcsecond-level accuracy (most accurate celestial sensor)",
        "Wertz (2001) Table 7-2; Liebe (2002)"
    );
}
pr4xis::register_axiom!(
    StarTrackerMostAccurate,
    "Wertz (2001) Table 7-2; Liebe (2002)"
);

/// Atmospheric refraction corrupts near-horizon observations.
///
/// Source: Bowditch (2002) Chapter 19; Meeus (1991).
pub struct AtmosphericRefraction;

impl Axiom for AtmosphericRefraction {
    fn verify(&self) -> Verdict {
        let refraction_at_horizon = approximate_refraction_arcmin(0.5);
        let refraction_at_45deg = approximate_refraction_arcmin(45.0);
        if refraction_at_horizon > refraction_at_45deg * 10.0 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "AtmosphericRefraction",
        "near-horizon observations are corrupted by atmospheric refraction",
        "Bowditch (2002) Chapter 19; Meeus (1991)"
    );
}
pr4xis::register_axiom!(
    AtmosphericRefraction,
    "Bowditch (2002) Chapter 19; Meeus (1991)"
);

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

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(TwoSightsFix));
        axioms.push(Box::new(StarTrackerMostAccurate));
        axioms.push(Box::new(AtmosphericRefraction));
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
        assert_category_laws::<CelestialCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        CelestialOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
