use crate::science::math::linear_algebra::matrix::Matrix;
use crate::science::math::linear_algebra::vector_space::Vector;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::technology::sensor_fusion::observation::gating::ValidationGate;
use crate::technology::sensor_fusion::observation::innovation::Innovation;
use crate::technology::sensor_fusion::observation::likelihood;
use crate::technology::sensor_fusion::observation::observation_model::LinearObservationModel;
use crate::technology::sensor_fusion::observation::ontology::*;

#[test]
fn observation_category_laws() {
    check_category_laws::<ObservationCategory>().unwrap();
}

#[test]
fn observation_ontology_validates() {
    ObservationOntology::validate().unwrap();
}

#[test]
fn innovation_zero_at_prediction() {
    assert!(InnovationZeroAtPrediction.holds());
}

#[test]
fn gate_accepts_mean() {
    assert!(GateAcceptsMean.holds());
}

#[test]
fn outlier_rejected_by_gate() {
    let h = LinearObservationModel::identity(2);
    let x = Vector::new(vec![0.0, 0.0]);
    let p = Matrix::identity(2);
    let r = Matrix::identity(2);
    // Measurement 100 sigma away
    let z = Vector::new(vec![100.0, 100.0]);
    let inn = Innovation::compute(&z, &x, &p, &h, &r);
    let gate = ValidationGate::new(2, 0.95);
    assert!(!gate.accept(&inn));
}

#[test]
fn likelihood_positive_at_mean() {
    let h = LinearObservationModel::identity(2);
    let x = Vector::new(vec![0.0, 0.0]);
    let p = Matrix::identity(2);
    let r = Matrix::identity(2);
    let z = Vector::new(vec![0.0, 0.0]);
    let inn = Innovation::compute(&z, &x, &p, &h, &r);
    let l = likelihood::likelihood(&inn);
    assert!(l > 0.0);
}

#[test]
fn likelihood_decreases_with_distance() {
    let h = LinearObservationModel::identity(1);
    let x = Vector::new(vec![0.0]);
    let p = Matrix::new(1, 1, vec![1.0]);
    let r = Matrix::new(1, 1, vec![1.0]);

    let z_close = Vector::new(vec![0.5]);
    let z_far = Vector::new(vec![5.0]);

    let inn_close = Innovation::compute(&z_close, &x, &p, &h, &r);
    let inn_far = Innovation::compute(&z_far, &x, &p, &h, &r);

    let l_close = likelihood::likelihood(&inn_close);
    let l_far = likelihood::likelihood(&inn_far);

    assert!(l_close > l_far);
}

// ---------------------------------------------------------------------------
// Statistics wiring: hypothesis-test gating
// ---------------------------------------------------------------------------

#[test]
fn gate_as_hypothesis_test_agrees_with_validation_gate() {
    use crate::science::math::statistics::hypothesis::TestDecision;
    use crate::technology::sensor_fusion::observation::gating::gate_as_hypothesis_test;

    let h = LinearObservationModel::identity(2);
    let x = Vector::new(vec![0.0, 0.0]);
    let p = Matrix::identity(2);
    let r = Matrix::identity(2);

    // Measurement close to prediction — should pass both
    let z_close = Vector::new(vec![0.1, 0.1]);
    let inn_close = Innovation::compute(&z_close, &x, &p, &h, &r);
    let gate = ValidationGate::new(2, 0.95);
    let nis_close = gate.nis(&inn_close).unwrap();
    let accept_gate = gate.accept(&inn_close);
    let decision_close = gate_as_hypothesis_test(nis_close, 2, 0.05);

    // Gate accepts => hypothesis test fails to reject H0
    assert!(accept_gate);
    assert_eq!(decision_close, TestDecision::FailToReject);

    // Measurement far from prediction — should fail both
    let z_far = Vector::new(vec![100.0, 100.0]);
    let inn_far = Innovation::compute(&z_far, &x, &p, &h, &r);
    let nis_far = gate.nis(&inn_far).unwrap();
    let accept_gate_far = gate.accept(&inn_far);
    let decision_far = gate_as_hypothesis_test(nis_far, 2, 0.05);

    assert!(!accept_gate_far);
    assert_eq!(decision_far, TestDecision::RejectNull);
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn gate_hypothesis_test_agrees_with_accept(
            z1 in -50.0..50.0_f64,
            z2 in -50.0..50.0_f64,
        ) {
            use crate::technology::sensor_fusion::observation::gating::gate_as_hypothesis_test;
            use crate::science::math::statistics::hypothesis::TestDecision;

            let h = LinearObservationModel::identity(2);
            let x = Vector::new(vec![0.0, 0.0]);
            let p = Matrix::identity(2);
            let r = Matrix::identity(2);
            let z = Vector::new(vec![z1, z2]);
            let inn = Innovation::compute(&z, &x, &p, &h, &r);
            let gate = ValidationGate::new(2, 0.95);
            let accepted = gate.accept(&inn);
            if let Some(nis) = gate.nis(&inn) {
                let decision = gate_as_hypothesis_test(nis, 2, 0.05);
                // accept => FailToReject, reject => RejectNull
                if accepted {
                    prop_assert_eq!(decision, TestDecision::FailToReject);
                } else {
                    prop_assert_eq!(decision, TestDecision::RejectNull);
                }
            }
        }

        #[test]
        fn nis_zero_always_passes_hypothesis_test(
            x1 in -100.0..100.0_f64,
            x2 in -100.0..100.0_f64,
        ) {
            use crate::technology::sensor_fusion::observation::gating::gate_as_hypothesis_test;
            use crate::science::math::statistics::hypothesis::TestDecision;

            // NIS = 0 means innovation is zero — always passes
            let decision = gate_as_hypothesis_test(0.0, 2, 0.05);
            let _ = (x1, x2); // consume strategy variables
            prop_assert_eq!(decision, TestDecision::FailToReject);
        }

        #[test]
        fn innovation_at_prediction_is_zero(
            x1 in -100.0..100.0_f64,
            x2 in -100.0..100.0_f64,
        ) {
            let h = LinearObservationModel::identity(2);
            let x = Vector::new(vec![x1, x2]);
            let p = Matrix::identity(2);
            let r = Matrix::identity(2);
            let z = h.predict(&x);
            let inn = Innovation::compute(&z, &x, &p, &h, &r);
            prop_assert!(inn.residual.norm() < 1e-10);
        }

        #[test]
        fn gate_always_accepts_at_mean(
            x1 in -50.0..50.0_f64,
            x2 in -50.0..50.0_f64,
        ) {
            let h = LinearObservationModel::identity(2);
            let x = Vector::new(vec![x1, x2]);
            let p = Matrix::identity(2);
            let r = Matrix::identity(2);
            let z = h.predict(&x);
            let inn = Innovation::compute(&z, &x, &p, &h, &r);
            let gate = ValidationGate::new(2, 0.95);
            prop_assert!(gate.accept(&inn));
        }

        #[test]
        fn likelihood_is_non_negative(
            z1 in -50.0..50.0_f64,
            z2 in -50.0..50.0_f64,
        ) {
            let h = LinearObservationModel::identity(2);
            let x = Vector::new(vec![0.0, 0.0]);
            let p = Matrix::identity(2);
            let r = Matrix::identity(2);
            let z = Vector::new(vec![z1, z2]);
            let inn = Innovation::compute(&z, &x, &p, &h, &r);
            let l = likelihood::likelihood(&inn);
            prop_assert!(l >= 0.0);
        }

        #[test]
        fn nis_is_non_negative(
            z1 in -50.0..50.0_f64,
            z2 in -50.0..50.0_f64,
        ) {
            let h = LinearObservationModel::identity(2);
            let x = Vector::new(vec![0.0, 0.0]);
            let p = Matrix::identity(2);
            let r = Matrix::identity(2);
            let z = Vector::new(vec![z1, z2]);
            let inn = Innovation::compute(&z, &x, &p, &h, &r);
            let gate = ValidationGate::new(2, 0.95);
            if let Some(nis) = gate.nis(&inn) {
                prop_assert!(nis >= -1e-10);
            }
        }
    }
}
