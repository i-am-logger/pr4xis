use pr4xis::category::Entity;
use pr4xis::define_ontology;
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

use crate::applied::sensor_fusion::observation::gating::ValidationGate;
use crate::applied::sensor_fusion::observation::innovation::Innovation;
use crate::applied::sensor_fusion::observation::observation_model::LinearObservationModel;

/// Observation processing stages (JDL Level 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum ObservationStage {
    /// Raw sensor data received.
    RawMeasurement,
    /// Observation model applied (predicted measurement).
    Predicted,
    /// Innovation computed (residual).
    InnovationComputed,
    /// Validation gate applied.
    GateChecked,
    /// Measurement accepted for fusion.
    Accepted,
    /// Measurement rejected (outlier).
    Rejected,
}

define_ontology! {
    pub ObservationOntology for ObservationCategory {
        entity: ObservationStage,
        relation: ObservationTransition,
    }
}

#[derive(Debug, Clone)]
pub struct StageDescription;

impl Quality for StageDescription {
    type Individual = ObservationStage;
    type Value = &'static str;

    fn get(&self, s: &ObservationStage) -> Option<&'static str> {
        Some(match s {
            ObservationStage::RawMeasurement => "raw sensor data z_k",
            ObservationStage::Predicted => "predicted measurement h(x̂)",
            ObservationStage::InnovationComputed => "innovation ν = z - h(x̂)",
            ObservationStage::GateChecked => "Mahalanobis gate applied",
            ObservationStage::Accepted => "measurement accepted for fusion",
            ObservationStage::Rejected => "measurement rejected (outlier)",
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
        let z = h.predict(&x); // z = Hx = x (identity)
        let inn = Innovation::compute(&z, &x, &p, &h, &r);
        inn.residual.norm() < 1e-12
    }
}

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
