//! Observation processing stages (JDL Level 0).
//!
//! Source: JDL (1999); Bar-Shalom et al. (2001).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

use crate::applied::sensor_fusion::observation::gating::ValidationGate;
use crate::applied::sensor_fusion::observation::innovation::Innovation;
use crate::applied::sensor_fusion::observation::observation_model::LinearObservationModel;

pr4xis::ontology! {
    name: "Observation",
    source: "JDL (1999); Bar-Shalom et al. (2001)",
    being: Event,

    concepts: [RawMeasurement, Predicted, InnovationComputed, GateChecked, Accepted, Rejected],

    labels: {
        RawMeasurement: ("en", "Raw measurement", "Raw sensor data received."),
        Predicted: ("en", "Predicted", "Observation model applied (predicted measurement)."),
        InnovationComputed: ("en", "Innovation computed", "Innovation computed (residual)."),
        GateChecked: ("en", "Gate checked", "Validation gate applied."),
        Accepted: ("en", "Accepted", "Measurement accepted for fusion."),
        Rejected: ("en", "Rejected", "Measurement rejected (outlier)."),
    },
}

#[derive(Debug, Clone)]
pub struct StageDescription;

impl Quality for StageDescription {
    type Individual = ObservationConcept;
    type Value = &'static str;

    fn get(&self, s: &ObservationConcept) -> Option<&'static str> {
        Some(match s {
            ObservationConcept::RawMeasurement => "raw sensor data z_k",
            ObservationConcept::Predicted => "predicted measurement h(x̂)",
            ObservationConcept::InnovationComputed => "innovation ν = z - h(x̂)",
            ObservationConcept::GateChecked => "Mahalanobis gate applied",
            ObservationConcept::Accepted => "measurement accepted for fusion",
            ObservationConcept::Rejected => "measurement rejected (outlier)",
        })
    }
}

/// Axiom: innovation at predicted measurement is zero.
pub struct InnovationZeroAtPrediction;

impl Axiom for InnovationZeroAtPrediction {
    fn description(&self) -> &str {
        "innovation is zero when measurement equals prediction"
    }
    fn holds(&self) -> bool {
        let h = LinearObservationModel::identity(2);
        let x = Vector::new(vec![1.0, 2.0]);
        let p = Matrix::identity(2);
        let r = Matrix::identity(2);
        let z = h.predict(&x);
        let inn = Innovation::compute(&z, &x, &p, &h, &r);
        inn.residual.norm() < 1e-12
    }
}
pr4xis::register_axiom!(
    InnovationZeroAtPrediction,
    "JDL (1999); Bar-Shalom et al. (2001)."
);

/// Axiom: gate at mean always accepts.
pub struct GateAcceptsMean;

impl Axiom for GateAcceptsMean {
    fn description(&self) -> &str {
        "validation gate accepts measurement at the predicted value"
    }
    fn holds(&self) -> bool {
        let h = LinearObservationModel::identity(2);
        let x = Vector::new(vec![5.0, 10.0]);
        let p = Matrix::identity(2);
        let r = Matrix::identity(2);
        let z = h.predict(&x);
        let inn = Innovation::compute(&z, &x, &p, &h, &r);
        let gate = ValidationGate::new(2, 0.95);
        gate.accept(&inn)
    }
}
pr4xis::register_axiom!(GateAcceptsMean, "JDL (1999); Bar-Shalom et al. (2001).");

impl Ontology for ObservationOntology {
    type Cat = ObservationCategory;
    type Qual = StageDescription;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(InnovationZeroAtPrediction),
            Box::new(GateAcceptsMean),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::ontology::Ontology;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<ObservationCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        ObservationOntology::validate().unwrap();
    }
}
