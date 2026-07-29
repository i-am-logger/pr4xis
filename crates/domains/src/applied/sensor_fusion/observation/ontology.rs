//! Observation processing stages (JDL Level 0).
//!
//! Source: JDL (1999); Bar-Shalom et al. (2001).

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

use crate::applied::sensor_fusion::observation::gating::ValidationGate;
use crate::applied::sensor_fusion::observation::innovation::Innovation;
use crate::applied::sensor_fusion::observation::observation_model::LinearObservationModel;

pr4xis::ontology! {
    name: "Observation",
    source: "JDL (1999); Bar-Shalom et al. (2001)",

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
    fn verify(&self) -> Verdict {
        let h = LinearObservationModel::identity(2);
        let x = Vector::new(vec![1.0, 2.0]);
        let p = Matrix::identity(2);
        let r = Matrix::identity(2);
        let z = h.predict(&x);
        let inn = Innovation::compute(&z, &x, &p, &h, &r);
        if inn.residual.norm().value < 1e-12 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "InnovationZeroAtPrediction",
        "innovation is zero when measurement equals prediction",
        "JDL (1999); Bar-Shalom et al. (2001)."
    );
}
pr4xis::register_axiom!(
    InnovationZeroAtPrediction,
    "JDL (1999); Bar-Shalom et al. (2001)."
);

/// Axiom: gate at mean always accepts.
pub struct GateAcceptsMean;

impl Axiom for GateAcceptsMean {
    fn verify(&self) -> Verdict {
        let h = LinearObservationModel::identity(2);
        let x = Vector::new(vec![5.0, 10.0]);
        let p = Matrix::identity(2);
        let r = Matrix::identity(2);
        let z = h.predict(&x);
        let inn = Innovation::compute(&z, &x, &p, &h, &r);
        let gate = ValidationGate::new(2, 0.95);
        if gate.accept(&inn) {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "GateAcceptsMean",
        "validation gate accepts measurement at the predicted value",
        "JDL (1999); Bar-Shalom et al. (2001)."
    );
}
pr4xis::register_axiom!(GateAcceptsMean, "JDL (1999); Bar-Shalom et al. (2001).");

impl Ontology for ObservationOntology {
    type Cat = ObservationCategory;
    type Qual = StageDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(InnovationZeroAtPrediction));
        axioms.push(Box::new(GateAcceptsMean));
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
        assert_category_laws::<ObservationCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ObservationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
