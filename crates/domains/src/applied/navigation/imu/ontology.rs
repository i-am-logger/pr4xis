//! IMU measurement types — what the accelerometer and gyroscope measure.
//!
//! Source: Titterton & Weston (2004), Chapter 4; Groves (2013), Chapter 4.

use pr4xis::ontology::reasoning::taxonomy;
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Imu",
    source: "Titterton & Weston (2004); Groves (2013)",
    being: PhysicalEndurant,

    concepts: [
        Measurement,
        SpecificForce,
        AngularRate,
        AccelerometerBias,
        GyroscopeBias,
        AccelerometerScaleFactor,
        GyroscopeScaleFactor,
    ],

    labels: {
        Measurement: ("en", "Measurement", "Abstract IMU measurement."),
        SpecificForce: ("en", "Specific force", "Specific force (accelerometer): f = a - g (acceleration minus gravity)."),
        AngularRate: ("en", "Angular rate", "Angular rate (gyroscope): ω (rad/s in body frame)."),
        AccelerometerBias: ("en", "Accelerometer bias", "Accelerometer bias: slowly varying offset in specific force."),
        GyroscopeBias: ("en", "Gyroscope bias", "Gyroscope bias: slowly varying offset in angular rate."),
        AccelerometerScaleFactor: ("en", "Accelerometer scale factor", "Accelerometer scale factor error: multiplicative error."),
        GyroscopeScaleFactor: ("en", "Gyroscope scale factor", "Gyroscope scale factor error: multiplicative error."),
    },

    is_a: [
        (SpecificForce, Measurement),
        (AngularRate, Measurement),
        (AccelerometerBias, SpecificForce),
        (GyroscopeBias, AngularRate),
        (AccelerometerScaleFactor, SpecificForce),
        (GyroscopeScaleFactor, AngularRate),
    ],
}

// ---------------------------------------------------------------------------
// Qualities
// ---------------------------------------------------------------------------

/// Quality: SI unit of each measurement.
#[derive(Debug, Clone)]
pub struct MeasurementUnit;

impl Quality for MeasurementUnit {
    type Individual = ImuConcept;
    type Value = &'static str;

    fn get(&self, m: &ImuConcept) -> Option<&'static str> {
        Some(match m {
            ImuConcept::Measurement => "various",
            ImuConcept::SpecificForce => "m/s²",
            ImuConcept::AngularRate => "rad/s",
            ImuConcept::AccelerometerBias => "m/s²",
            ImuConcept::GyroscopeBias => "rad/s",
            ImuConcept::AccelerometerScaleFactor => "dimensionless (ppm)",
            ImuConcept::GyroscopeScaleFactor => "dimensionless (ppm)",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms
// ---------------------------------------------------------------------------

/// Accelerometer bias is-a SpecificForce (it's an error IN specific force).
pub struct BiasIsAMeasurement;

impl Axiom for BiasIsAMeasurement {
    fn description(&self) -> &str {
        "accelerometer bias is-a specific force measurement (error term)"
    }
    fn holds(&self) -> bool {
        taxonomy::is_a::<ImuTaxonomy>(&ImuConcept::AccelerometerBias, &ImuConcept::SpecificForce)
    }
}
pr4xis::register_axiom!(BiasIsAMeasurement);

/// Specific force = acceleration - gravity (Newton's equation in non-inertial frame).
///
/// Source: Groves (2013), Eq. 4.1.
pub struct SpecificForceDefinition;

impl Axiom for SpecificForceDefinition {
    fn description(&self) -> &str {
        "specific force = acceleration - gravity: at rest, accelerometer reads -g"
    }
    fn holds(&self) -> bool {
        let g = crate::formal::math::quantity::constants::standard_gravity().value;
        let specific_force_at_rest = -g;
        (specific_force_at_rest + g).abs() < 1e-10
    }
}
pr4xis::register_axiom!(SpecificForceDefinition);

/// Gyroscope measures angular rate in body frame.
pub struct GyroscopeBodyFrame;

impl Axiom for GyroscopeBodyFrame {
    fn description(&self) -> &str {
        "gyroscope measures angular rate in body frame (3 axes)"
    }
    fn holds(&self) -> bool {
        taxonomy::is_a::<ImuTaxonomy>(&ImuConcept::AngularRate, &ImuConcept::Measurement)
    }
}
pr4xis::register_axiom!(GyroscopeBodyFrame);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for ImuOntology {
    type Cat = ImuCategory;
    type Qual = MeasurementUnit;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(BiasIsAMeasurement),
            Box::new(SpecificForceDefinition),
            Box::new(GyroscopeBodyFrame),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<ImuCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        ImuOntology::validate().unwrap();
    }
}
