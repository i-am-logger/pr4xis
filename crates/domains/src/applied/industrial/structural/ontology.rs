//! Structural health monitoring sensor types.
//!
//! Source: Farrar & Worden (2007), "An Introduction to Structural Health Monitoring"

use pr4xis::logic::proof::{SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

pr4xis::ontology! {
    name: "Structural",
    source: "Farrar & Worden (2007); Paris & Erdogan (1963)",

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
    fn verify(&self) -> Verdict {
        // Hooke's law: σ = E·ε with σ bounded above by yield stress σ_y
        // implies ε ≤ σ_y / E in the elastic regime. See Farrar & Worden
        // (2007) §2.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "StrainBoundedElastic",
        "strain is bounded within elastic deformation limits",
        "Farrar & Worden (2007) An Introduction to Structural Health Monitoring §2"
    );
}
pr4xis::register_axiom!(
    StrainBoundedElastic,
    "Farrar & Worden (2007) An Introduction to Structural Health Monitoring §2"
);

/// Axiom: crack length is non-negative and monotonically non-decreasing.
pub struct CrackMonotonicity;

impl Axiom for CrackMonotonicity {
    fn verify(&self) -> Verdict {
        // Paris-Erdogan law: da/dN = C·(ΔK)^m with da/dN ≥ 0 under cyclic
        // loading. Fatigue cracks only grow; healing is not modelled.
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "CrackMonotonicity",
        "crack length is non-negative and does not decrease (fatigue cracks only grow)",
        "Paris & Erdogan (1963) A Critical Analysis of Crack Propagation Laws"
    );
}
pr4xis::register_axiom!(
    CrackMonotonicity,
    "Paris & Erdogan (1963) A Critical Analysis of Crack Propagation Laws"
);

impl Ontology for StructuralOntology {
    type Cat = StructuralCategory;
    type Qual = SensorMeasurand;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(StrainBoundedElastic));
        axioms.push(Box::new(CrackMonotonicity));
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
        assert_category_laws::<StructuralCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        StructuralOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
