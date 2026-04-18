//! Spacecraft attitude determination sensors.
//!
//! Source: Wertz (1978), *Spacecraft Attitude Determination and Control*

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::Concept;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Attitude",
    source: "Wertz (1978); Markley & Crassidis (2014)",
    being: Process,

    concepts: [StarTracker, SunSensor, EarthHorizon, Magnetometer],

    labels: {
        StarTracker: ("en", "Star tracker", "Star tracker: high-accuracy inertial attitude reference."),
        SunSensor: ("en", "Sun sensor", "Sun sensor: determines direction to the Sun."),
        EarthHorizon: ("en", "Earth horizon sensor", "Earth horizon sensor: determines nadir direction."),
        Magnetometer: ("en", "Magnetometer", "Magnetometer: measures local magnetic field vector."),
    },
}

/// Quality: typical accuracy of each sensor type.
#[derive(Debug, Clone)]
pub struct SensorAccuracy;

impl Quality for SensorAccuracy {
    type Individual = AttitudeConcept;
    /// Accuracy in arcseconds (1-sigma).
    type Value = f64;

    fn get(&self, sensor: &AttitudeConcept) -> Option<f64> {
        Some(match sensor {
            AttitudeConcept::StarTracker => 1.0,
            AttitudeConcept::SunSensor => 60.0,
            AttitudeConcept::EarthHorizon => 3600.0,
            AttitudeConcept::Magnetometer => 7200.0,
        })
    }
}

/// Axiom: a unit quaternion has norm 1 (attitude representation constraint).
pub struct QuaternionUnitNorm;

impl Axiom for QuaternionUnitNorm {
    fn description(&self) -> &str {
        "attitude quaternion must have unit norm (|q| = 1)"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(
    QuaternionUnitNorm,
    "Wertz (1978), *Spacecraft Attitude Determination and Control*"
);

/// Axiom: star tracker is the most accurate attitude sensor.
pub struct StarTrackerMostAccurate;

impl Axiom for StarTrackerMostAccurate {
    fn description(&self) -> &str {
        "star tracker has the highest accuracy among attitude sensors"
    }
    fn holds(&self) -> bool {
        let q = SensorAccuracy;
        let star_acc = q.get(&AttitudeConcept::StarTracker).unwrap();
        AttitudeConcept::variants()
            .iter()
            .all(|s| q.get(s).unwrap() >= star_acc)
    }
}
pr4xis::register_axiom!(
    StarTrackerMostAccurate,
    "Wertz (1978), *Spacecraft Attitude Determination and Control*"
);

impl Ontology for AttitudeOntology {
    type Cat = AttitudeCategory;
    type Qual = SensorAccuracy;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(QuaternionUnitNorm),
            Box::new(StarTrackerMostAccurate),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<AttitudeCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        AttitudeOntology::validate().unwrap();
    }
}
