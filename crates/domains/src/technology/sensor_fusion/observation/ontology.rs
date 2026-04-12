use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::science::math::linear_algebra::matrix::Matrix;
use crate::science::math::linear_algebra::vector_space::Vector;

use crate::technology::sensor_fusion::observation::gating::ValidationGate;
use crate::technology::sensor_fusion::observation::innovation::Innovation;
use crate::technology::sensor_fusion::observation::observation_model::LinearObservationModel;

/// Observation processing stages (JDL Level 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Entity for ObservationStage {
    fn variants() -> Vec<Self> {
        vec![
            Self::RawMeasurement,
            Self::Predicted,
            Self::InnovationComputed,
            Self::GateChecked,
            Self::Accepted,
            Self::Rejected,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObservationTransition {
    pub from: ObservationStage,
    pub to: ObservationStage,
}

impl Relationship for ObservationTransition {
    type Object = ObservationStage;
    fn source(&self) -> ObservationStage {
        self.from
    }
    fn target(&self) -> ObservationStage {
        self.to
    }
}

pub struct ObservationCategory;

impl Category for ObservationCategory {
    type Object = ObservationStage;
    type Morphism = ObservationTransition;

    fn identity(obj: &ObservationStage) -> ObservationTransition {
        ObservationTransition {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(
        f: &ObservationTransition,
        g: &ObservationTransition,
    ) -> Option<ObservationTransition> {
        if f.to != g.from {
            return None;
        }
        Some(ObservationTransition {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<ObservationTransition> {
        let all = ObservationStage::variants();
        let m: Vec<ObservationTransition> = all
            .iter()
            .flat_map(|&from| {
                all.iter()
                    .map(move |&to| ObservationTransition { from, to })
            })
            .collect();
        m
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

pub struct ObservationOntology;

impl Ontology for ObservationOntology {
    type Cat = ObservationCategory;
    type Qual = StageDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(InnovationZeroAtPrediction),
            Box::new(GateAcceptsMean),
        ]
    }
}
