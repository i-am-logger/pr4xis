//! State estimation concepts.
//!
//! Source: Kalman (1960); Maybeck (1979).

use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof, Verdict};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use crate::formal::math::temporal::instant::Instant;
use crate::formal::math::temporal::time_system::TimeSystem;

use crate::applied::sensor_fusion::state::covariance;
use crate::applied::sensor_fusion::state::estimate::StateEstimate;
use crate::applied::sensor_fusion::state::information::InformationEstimate;

pr4xis::ontology! {
    name: "StateEstimation",
    source: "Kalman (1960); Maybeck (1979)",

    concepts: [StateVector, Covariance, InformationMatrix, CRLB],

    labels: {
        StateVector: ("en", "State vector", "The state vector x̂."),
        Covariance: ("en", "Covariance", "The error covariance P."),
        InformationMatrix: ("en", "Information matrix", "The information matrix Y = P^{-1}."),
        CRLB: ("en", "Cramér-Rao lower bound", "The Cramér-Rao lower bound."),
    },
}

#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = StateEstimationConcept;
    type Value = &'static str;

    fn get(&self, c: &StateEstimationConcept) -> Option<&'static str> {
        Some(match c {
            StateEstimationConcept::StateVector => "x̂: best estimate of hidden state",
            StateEstimationConcept::Covariance => "P: uncertainty of the estimate (symmetric PSD)",
            StateEstimationConcept::InformationMatrix => "Y = P^{-1}: precision/information",
            StateEstimationConcept::CRLB => "J^{-1}: lower bound on estimator variance",
        })
    }
}

/// Axiom: covariance of a valid estimate is always PSD.
pub struct CovarianceIsPSD;

impl Axiom for CovarianceIsPSD {
    fn verify(&self) -> Verdict {
        let estimates = canonical_estimates();
        let ok = estimates
            .iter()
            .all(|e| covariance::is_valid(&e.covariance));
        if ok {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "CovarianceIsPSD",
        "covariance of a valid estimate is symmetric positive semi-definite",
        "Kalman (1960); Maybeck (1979)."
    );
}
pr4xis::register_axiom!(CovarianceIsPSD, "Kalman (1960); Maybeck (1979).");

/// Axiom: information form roundtrip preserves the estimate.
pub struct InformationRoundtrip;

impl Axiom for InformationRoundtrip {
    fn verify(&self) -> Verdict {
        for est in &canonical_estimates() {
            if let Some(info) = InformationEstimate::from_estimate(est) {
                if let Some(est2) = info.to_estimate(est.epoch.clone()) {
                    let state_diff: f64 = est
                        .state
                        .data
                        .iter()
                        .zip(&est2.state.data)
                        .map(|(a, b)| (a - b).abs())
                        .sum();
                    if state_diff > 1e-6 {
                        return Err(Box::new(SimpleCounterexample::new(self.meta())));
                    }
                } else {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            } else {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "InformationRoundtrip",
        "state -> information -> state roundtrip preserves estimate",
        "Kalman (1960); Maybeck (1979)."
    );
}
pr4xis::register_axiom!(InformationRoundtrip, "Kalman (1960); Maybeck (1979).");

/// Axiom: information fusion is additive.
pub struct InformationFusionAdditive;

impl Axiom for InformationFusionAdditive {
    fn verify(&self) -> Verdict {
        let e1 = StateEstimate::new(
            Vector::new(vec![1.0, 0.0]),
            Matrix::diagonal(&[2.0, 2.0]),
            Instant::new(0.0, TimeSystem::GPS),
        );
        let e2 = StateEstimate::new(
            Vector::new(vec![0.0, 1.0]),
            Matrix::diagonal(&[3.0, 3.0]),
            Instant::new(0.0, TimeSystem::GPS),
        );
        let i1 = InformationEstimate::from_estimate(&e1).unwrap();
        let i2 = InformationEstimate::from_estimate(&e2).unwrap();
        let fused = i1.fuse(&i2);

        let expected_y = i1.information_matrix.add(&i2.information_matrix);
        let diff: f64 = fused
            .information_matrix
            .data
            .iter()
            .zip(&expected_y.data)
            .map(|(a, b)| (a - b).abs())
            .sum();
        if diff < 1e-10 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "InformationFusionAdditive",
        "information fusion: Y_fused = Y1 + Y2 (additive)",
        "Kalman (1960); Maybeck (1979)."
    );
}
pr4xis::register_axiom!(InformationFusionAdditive, "Kalman (1960); Maybeck (1979).");

impl Ontology for StateEstimationOntology {
    type Cat = StateEstimationCategory;
    type Qual = ConceptDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(CovarianceIsPSD));
        axioms.push(Box::new(InformationRoundtrip));
        axioms.push(Box::new(InformationFusionAdditive));
        axioms
    }
}

fn canonical_estimates() -> Vec<StateEstimate> {
    vec![
        StateEstimate::new(
            Vector::new(vec![0.0]),
            Matrix::new(1, 1, vec![1.0]),
            Instant::new(0.0, TimeSystem::GPS),
        ),
        StateEstimate::new(
            Vector::new(vec![1.0, 2.0]),
            Matrix::diagonal(&[2.0, 3.0]),
            Instant::new(0.0, TimeSystem::GPS),
        ),
        StateEstimate::new(
            Vector::new(vec![0.0, 0.0, 0.0]),
            Matrix::new(3, 3, vec![4.0, 1.0, 0.0, 1.0, 5.0, 1.0, 0.0, 1.0, 6.0]),
            Instant::new(0.0, TimeSystem::GPS),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::laws::assert_category_laws;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<StateEstimationCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        StateEstimationOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }
}
