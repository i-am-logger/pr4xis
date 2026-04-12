use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

use crate::applied::sensor_fusion::state::covariance;
use crate::applied::sensor_fusion::state::estimate::StateEstimate;
use crate::applied::sensor_fusion::state::information::InformationEstimate;

/// State estimation concepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum EstimationConcept {
    /// The state vector x̂.
    StateVector,
    /// The error covariance P.
    Covariance,
    /// The information matrix Y = P^{-1}.
    InformationMatrix,
    /// The Cramér-Rao lower bound.
    CRLB,
}

define_dense_category! {
    pub EstimationCategory {
        entity: EstimationConcept,
        relation: EstimationRelation,
    }
}

#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = EstimationConcept;
    type Value = &'static str;

    fn get(&self, c: &EstimationConcept) -> Option<&'static str> {
        Some(match c {
            EstimationConcept::StateVector => "x̂: best estimate of hidden state",
            EstimationConcept::Covariance => "P: uncertainty of the estimate (symmetric PSD)",
            EstimationConcept::InformationMatrix => "Y = P^{-1}: precision/information",
            EstimationConcept::CRLB => "J^{-1}: lower bound on estimator variance",
        })
    }
}

/// Axiom: covariance of a valid estimate is always PSD.
pub struct CovarianceIsPSD;

impl Axiom for CovarianceIsPSD {
    fn description(&self) -> &str {
        "covariance of a valid estimate is symmetric positive semi-definite"
    }
    fn holds(&self) -> bool {
        let estimates = canonical_estimates();
        estimates
            .iter()
            .all(|e| covariance::is_valid(&e.covariance))
    }
}

/// Axiom: information form roundtrip preserves the estimate.
pub struct InformationRoundtrip;

impl Axiom for InformationRoundtrip {
    fn description(&self) -> &str {
        "state -> information -> state roundtrip preserves estimate"
    }
    fn holds(&self) -> bool {
        for est in &canonical_estimates() {
            if let Some(info) = InformationEstimate::from_estimate(est) {
                if let Some(est2) = info.to_estimate(est.epoch) {
                    let state_diff: f64 = est
                        .state
                        .data
                        .iter()
                        .zip(&est2.state.data)
                        .map(|(a, b)| (a - b).abs())
                        .sum();
                    if state_diff > 1e-6 {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

/// Axiom: information fusion is additive.
pub struct InformationFusionAdditive;

impl Axiom for InformationFusionAdditive {
    fn description(&self) -> &str {
        "information fusion: Y_fused = Y1 + Y2 (additive)"
    }
    fn holds(&self) -> bool {
        let e1 = StateEstimate::new(
            Vector::new(vec![1.0, 0.0]),
            Matrix::diagonal(&[2.0, 2.0]),
            0.0,
        );
        let e2 = StateEstimate::new(
            Vector::new(vec![0.0, 1.0]),
            Matrix::diagonal(&[3.0, 3.0]),
            0.0,
        );
        let i1 = InformationEstimate::from_estimate(&e1).unwrap();
        let i2 = InformationEstimate::from_estimate(&e2).unwrap();
        let fused = i1.fuse(&i2);

        // Fused information should be sum
        let expected_y = i1.information_matrix.add(&i2.information_matrix);
        let diff: f64 = fused
            .information_matrix
            .data
            .iter()
            .zip(&expected_y.data)
            .map(|(a, b)| (a - b).abs())
            .sum();
        diff < 1e-10
    }
}

pub struct StateEstimationOntology;

impl Ontology for StateEstimationOntology {
    type Cat = EstimationCategory;
    type Qual = ConceptDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(CovarianceIsPSD),
            Box::new(InformationRoundtrip),
            Box::new(InformationFusionAdditive),
        ]
    }
}

fn canonical_estimates() -> Vec<StateEstimate> {
    vec![
        StateEstimate::new(Vector::new(vec![0.0]), Matrix::new(1, 1, vec![1.0]), 0.0),
        StateEstimate::new(
            Vector::new(vec![1.0, 2.0]),
            Matrix::diagonal(&[2.0, 3.0]),
            0.0,
        ),
        StateEstimate::new(
            Vector::new(vec![0.0, 0.0, 0.0]),
            Matrix::new(3, 3, vec![4.0, 1.0, 0.0, 1.0, 5.0, 1.0, 0.0, 1.0, 6.0]),
            0.0,
        ),
    ]
}
