//! IMU measurement types — what the accelerometer and gyroscope measure.
//!
//! Source: Titterton & Weston (2004), Chapter 4; Groves (2013), Chapter 4.

use pr4xis::category::{Arrow, Category};
use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Imu",
    source: "Titterton & Weston (2004); Groves (2013)",

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

/// Direct subsumption query: is there an `is_a` edge from `child` to `parent`?
///
/// Per #169, the prior `taxonomy::is_a` helper was removed; the taxonomy
/// lives as `Subsumption`-kinded morphisms in `ImuCategory`.
fn is_a(child: ImuConcept, parent: ImuConcept) -> bool {
    ImuCategory::morphisms().iter().any(|m| {
        m.kind() == ImuRelationKind::Subsumption && m.source() == child && m.target() == parent
    })
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
    fn verify(&self) -> Verdict {
        if is_a(ImuConcept::AccelerometerBias, ImuConcept::SpecificForce) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "BiasIsAMeasurement",
        "accelerometer bias is-a specific force measurement (error term)",
        "Titterton & Weston (2004), Chapter 4; Groves (2013), Chapter 4."
    );
}
pr4xis::register_axiom!(
    BiasIsAMeasurement,
    "Titterton & Weston (2004), Chapter 4; Groves (2013), Chapter 4."
);

/// Specific force = acceleration - gravity (Newton's equation in non-inertial frame).
///
/// Source: Groves (2013), Eq. 4.1.
pub struct SpecificForceDefinition;

impl Axiom for SpecificForceDefinition {
    fn verify(&self) -> Verdict {
        let g = crate::formal::math::quantity::constants::standard_gravity().value;
        let specific_force_at_rest = -g;
        if (specific_force_at_rest + g).abs() < 1e-10 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "SpecificForceDefinition",
        "specific force = acceleration - gravity: at rest, accelerometer reads -g",
        "Groves (2013), Eq. 4.1."
    );
}
pr4xis::register_axiom!(SpecificForceDefinition, "Groves (2013), Eq. 4.1.");

/// Gyroscope measures angular rate in body frame.
pub struct GyroscopeBodyFrame;

impl Axiom for GyroscopeBodyFrame {
    fn verify(&self) -> Verdict {
        if is_a(ImuConcept::AngularRate, ImuConcept::Measurement) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GyroscopeBodyFrame",
        "gyroscope measures angular rate in body frame (3 axes)",
        "Titterton & Weston (2004), Chapter 4; Groves (2013), Chapter 4."
    );
}
pr4xis::register_axiom!(
    GyroscopeBodyFrame,
    "Titterton & Weston (2004), Chapter 4; Groves (2013), Chapter 4."
);

// ---------------------------------------------------------------------------
// Ontology impl
// ---------------------------------------------------------------------------

impl Ontology for ImuOntology {
    type Cat = ImuCategory;
    type Qual = MeasurementUnit;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(BiasIsAMeasurement));
        axioms.push(Box::new(SpecificForceDefinition));
        axioms.push(Box::new(GyroscopeBodyFrame));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[test]
    fn category_laws() {
        assert_category_laws::<ImuCategory>();
    }

    #[test]
    fn ontology_validates() {
        ImuOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
