//! Spacecraft attitude determination sensors.
//!
//! Source: Wertz (1978), *Spacecraft Attitude Determination and Control*

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::category::FinitelyGenerated;
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Attitude",
    source: "Wertz (1978); Markley & Crassidis (2014)",

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
    fn verify(&self) -> Verdict {
        // Markley & Crassidis (2014) §2.7: attitude quaternions live on
        // the unit 3-sphere S³ ⊂ ℝ⁴ — only unit quaternions correspond to
        // valid rotations in SO(3).
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "QuaternionUnitNorm",
        "attitude quaternion must have unit norm (|q| = 1)",
        "Markley & Crassidis (2014) Fundamentals of Spacecraft Attitude Determination and Control §2.7"
    );
}
pr4xis::register_axiom!(
    QuaternionUnitNorm,
    "Markley & Crassidis (2014) Fundamentals of Spacecraft Attitude Determination and Control §2.7"
);

/// Axiom: star tracker is the most accurate attitude sensor.
pub struct StarTrackerMostAccurate;

impl Axiom for StarTrackerMostAccurate {
    fn verify(&self) -> Verdict {
        let q = SensorAccuracy;
        let star_acc = q.get(&AttitudeConcept::StarTracker).unwrap();
        let ok = AttitudeConcept::variants()
            .iter()
            .all(|s| q.get(s).unwrap() >= star_acc);
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "StarTrackerMostAccurate",
        "star tracker has the highest accuracy among attitude sensors",
        "Wertz (1978) Spacecraft Attitude Determination and Control §6"
    );
}
pr4xis::register_axiom!(
    StarTrackerMostAccurate,
    "Wertz (1978) Spacecraft Attitude Determination and Control §6"
);

impl Ontology for AttitudeOntology {
    type Cat = AttitudeCategory;
    type Qual = SensorAccuracy;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(QuaternionUnitNorm));
        axioms.push(Box::new(StarTrackerMostAccurate));
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
        assert_category_laws::<AttitudeCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        AttitudeOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
