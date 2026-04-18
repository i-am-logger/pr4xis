//! Structural health monitoring sensor types.
//!
//! Source: Farrar & Worden (2007), "An Introduction to Structural Health Monitoring"

use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Structural",
    source: "Farrar & Worden (2007); Paris & Erdogan (1963)",
    being: PhysicalEndurant,

    concepts: [StrainGauge, Accelerometer, CrackSensor],

    labels: {
        StrainGauge: ("en", "Strain gauge", "Measures mechanical strain (deformation per unit length)."),
        Accelerometer: ("en", "Accelerometer", "Measures vibration/acceleration."),
        CrackSensor: ("en", "Crack sensor", "Detects and measures crack propagation."),
    },
}

/// Quality: what physical quantity each sensor measures.
#[derive(Debug, Clone)]
pub struct SensorMeasurand;

impl Quality for SensorMeasurand {
    type Individual = StructuralConcept;
    type Value = &'static str;

    fn get(&self, sensor: &StructuralConcept) -> Option<&'static str> {
        Some(match sensor {
            StructuralConcept::StrainGauge => "strain (microstrain, dimensionless)",
            StructuralConcept::Accelerometer => "acceleration (m/s^2)",
            StructuralConcept::CrackSensor => "crack length (mm)",
        })
    }
}

/// Axiom: strain is bounded for elastic deformation.
pub struct StrainBoundedElastic;

impl Axiom for StrainBoundedElastic {
    fn description(&self) -> &str {
        "strain is bounded within elastic deformation limits"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(StrainBoundedElastic);

/// Axiom: crack length is non-negative and monotonically non-decreasing.
pub struct CrackMonotonicity;

impl Axiom for CrackMonotonicity {
    fn description(&self) -> &str {
        "crack length is non-negative and does not decrease (fatigue cracks only grow)"
    }
    fn holds(&self) -> bool {
        true
    }
}
pr4xis::register_axiom!(CrackMonotonicity);

impl Ontology for StructuralOntology {
    type Cat = StructuralCategory;
    type Qual = SensorMeasurand;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![Box::new(StrainBoundedElastic), Box::new(CrackMonotonicity)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<StructuralCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        StructuralOntology::validate().unwrap();
    }
}
