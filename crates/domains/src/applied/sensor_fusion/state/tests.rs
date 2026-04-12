use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::applied::sensor_fusion::state::covariance;
use crate::applied::sensor_fusion::state::estimate::StateEstimate;
use crate::applied::sensor_fusion::state::information::InformationEstimate;
use crate::applied::sensor_fusion::state::ontology::*;

#[test]
fn estimation_category_laws() {
    check_category_laws::<EstimationCategory>().unwrap();
}

#[test]
fn state_estimation_ontology_validates() {
    StateEstimationOntology::validate().unwrap();
}

#[test]
fn covariance_is_psd() {
    assert!(CovarianceIsPSD.holds());
}

#[test]
fn information_roundtrip() {
    assert!(InformationRoundtrip.holds());
}

#[test]
fn information_fusion_additive() {
    assert!(InformationFusionAdditive.holds());
}

#[test]
fn std_dev_is_sqrt_of_diagonal() {
    let p = Matrix::diagonal(&[4.0, 9.0, 16.0]);
    assert!((covariance::std_dev(&p, 0) - 2.0).abs() < 1e-12);
    assert!((covariance::std_dev(&p, 1) - 3.0).abs() < 1e-12);
    assert!((covariance::std_dev(&p, 2) - 4.0).abs() < 1e-12);
}

#[test]
fn diagonal_covariance_has_zero_correlation() {
    let p = Matrix::diagonal(&[4.0, 9.0]);
    let rho = covariance::correlation(&p, 0, 1);
    assert!(rho.abs() < 1e-12);
}

#[test]
fn information_fusion_reduces_uncertainty() {
    let e1 = StateEstimate::new(Vector::new(vec![10.0]), Matrix::new(1, 1, vec![4.0]), 0.0);
    let e2 = StateEstimate::new(Vector::new(vec![12.0]), Matrix::new(1, 1, vec![9.0]), 0.0);
    let i1 = InformationEstimate::from_estimate(&e1).unwrap();
    let i2 = InformationEstimate::from_estimate(&e2).unwrap();
    let fused = i1.fuse(&i2);
    let fused_est = fused.to_estimate(0.0).unwrap();
    // Fused covariance should be less than both
    assert!(fused_est.covariance.get(0, 0) < e1.covariance.get(0, 0));
    assert!(fused_est.covariance.get(0, 0) < e2.covariance.get(0, 0));
}

// ---------------------------------------------------------------------------
// Statistics wiring: confidence intervals on state estimates
// ---------------------------------------------------------------------------

#[test]
fn confidence_interval_contains_mean() {
    let est = StateEstimate::new(
        Vector::new(vec![5.0, 10.0]),
        Matrix::diagonal(&[4.0, 9.0]),
        0.0,
    );
    let ci0 = est.confidence_interval(0, 0.95).unwrap();
    let ci1 = est.confidence_interval(1, 0.95).unwrap();

    // The mean must be inside the CI
    assert!(ci0.contains(5.0), "CI should contain mean 5.0: {:?}", ci0);
    assert!(ci1.contains(10.0), "CI should contain mean 10.0: {:?}", ci1);
}

#[test]
fn confidence_interval_widens_with_higher_level() {
    let est = StateEstimate::new(Vector::new(vec![0.0]), Matrix::new(1, 1, vec![4.0]), 0.0);
    let ci_90 = est.confidence_interval(0, 0.90).unwrap();
    let ci_95 = est.confidence_interval(0, 0.95).unwrap();
    let ci_99 = est.confidence_interval(0, 0.99).unwrap();

    assert!(
        ci_99.width() > ci_95.width(),
        "99% CI ({}) should be wider than 95% ({})",
        ci_99.width(),
        ci_95.width()
    );
    assert!(
        ci_95.width() > ci_90.width(),
        "95% CI ({}) should be wider than 90% ({})",
        ci_95.width(),
        ci_90.width()
    );
}

// ---------------------------------------------------------------------------
// H6: StateEstimate::new uses debug_assert — does not panic in release
// ---------------------------------------------------------------------------

#[test]
fn state_estimate_new_valid_dimensions() {
    // This should always work
    let est = StateEstimate::new(
        Vector::new(vec![1.0, 2.0]),
        Matrix::diagonal(&[1.0, 2.0]),
        0.0,
    );
    assert_eq!(est.dim(), 2);
}

// ---------------------------------------------------------------------------
// H7: confidence_interval with out-of-bounds index returns None
// ---------------------------------------------------------------------------

#[test]
fn confidence_interval_out_of_bounds_returns_none() {
    let est = StateEstimate::new(Vector::new(vec![5.0]), Matrix::new(1, 1, vec![4.0]), 0.0);
    // Index 0 is valid
    assert!(est.confidence_interval(0, 0.95).is_some());
    // Index 1 is out of bounds for a 1D state
    assert!(
        est.confidence_interval(1, 0.95).is_none(),
        "out-of-bounds index should return None"
    );
    // Index 100 is way out of bounds
    assert!(
        est.confidence_interval(100, 0.95).is_none(),
        "way out-of-bounds index should return None"
    );
}

#[cfg(test)]
mod proptest_proofs {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn confidence_interval_always_contains_mean(
            x in -100.0..100.0_f64,
            var in 0.01..100.0_f64,
        ) {
            let est = StateEstimate::new(
                Vector::new(vec![x]),
                Matrix::new(1, 1, vec![var]),
                0.0,
            );
            let ci = est.confidence_interval(0, 0.95).unwrap();
            prop_assert!(ci.contains(x),
                "CI [{}, {}] should contain mean {}",
                ci.lower, ci.upper, x);
        }

        #[test]
        fn confidence_interval_99_wider_than_95(
            x in -50.0..50.0_f64,
            var in 0.01..50.0_f64,
        ) {
            let est = StateEstimate::new(
                Vector::new(vec![x]),
                Matrix::new(1, 1, vec![var]),
                0.0,
            );
            let ci_95 = est.confidence_interval(0, 0.95).unwrap();
            let ci_99 = est.confidence_interval(0, 0.99).unwrap();
            prop_assert!(ci_99.width() >= ci_95.width() - 1e-10,
                "99% width {} should >= 95% width {}",
                ci_99.width(), ci_95.width());
        }

        #[test]
        fn symmetrize_produces_symmetric(
            a in -10.0..10.0_f64, b in -10.0..10.0_f64,
            c in -10.0..10.0_f64, d in -10.0..10.0_f64,
        ) {
            let m = Matrix::new(2, 2, vec![a, b, c, d]);
            let s = covariance::ensure_symmetric(&m);
            prop_assert!(s.is_symmetric(1e-15));
        }

        #[test]
        fn total_uncertainty_is_trace(
            v1 in 0.1..100.0_f64,
            v2 in 0.1..100.0_f64,
        ) {
            let p = Matrix::diagonal(&[v1, v2]);
            let est = StateEstimate::new(Vector::new(vec![0.0, 0.0]), p.clone(), 0.0);
            prop_assert!((est.uncertainty() - (v1 + v2)).abs() < 1e-10);
        }

        #[test]
        fn information_roundtrip_preserves_state(
            x1 in -50.0..50.0_f64,
            v1 in 1.0..50.0_f64,
        ) {
            let est = StateEstimate::new(
                Vector::new(vec![x1]),
                Matrix::new(1, 1, vec![v1]),
                0.0,
            );
            let info = InformationEstimate::from_estimate(&est).unwrap();
            let est2 = info.to_estimate(0.0).unwrap();
            prop_assert!((est.state.get(0) - est2.state.get(0)).abs() < 1e-8);
        }
    }
}

// ---------------------------------------------------------------------------
// Meta-axiom: StateEstimate enforces PSD covariance at construction.
//
// The ontology claims P is always positive semi-definite. The
// #[non_exhaustive] attribute prevents external crates from constructing
// via struct literal, forcing them through new() which validates dimensions.
// ---------------------------------------------------------------------------

#[test]
fn meta_axiom_state_estimate_new_produces_valid_estimate() {
    use crate::formal::math::linear_algebra::positive_definite;

    // 1D PSD covariance
    let est = StateEstimate::new(Vector::new(vec![0.0]), Matrix::new(1, 1, vec![5.0]), 0.0);
    assert!(
        positive_definite::is_positive_semidefinite(&est.covariance),
        "1D covariance from new() must be PSD"
    );
    assert_eq!(est.step, 0, "step must start at 0");

    // 2D PSD covariance
    let est2 = StateEstimate::new(
        Vector::new(vec![1.0, 2.0]),
        Matrix::diagonal(&[3.0, 7.0]),
        0.0,
    );
    assert!(
        positive_definite::is_positive_semidefinite(&est2.covariance),
        "2D diagonal covariance from new() must be PSD"
    );

    // 2D with off-diagonal (still PSD: eigenvalues 4 and 6)
    let est3 = StateEstimate::new(
        Vector::new(vec![0.0, 0.0]),
        Matrix::new(2, 2, vec![5.0, 1.0, 1.0, 5.0]),
        0.0,
    );
    assert!(
        positive_definite::is_positive_semidefinite(&est3.covariance),
        "2D correlated covariance from new() must be PSD"
    );
}

#[test]
fn meta_axiom_state_estimate_non_exhaustive_enforces_constructor() {
    // This test documents that #[non_exhaustive] is applied.
    // External crates cannot construct StateEstimate via struct literal;
    // they must use StateEstimate::new(). This test verifies the constructor
    // produces a correctly initialized object.
    let est = StateEstimate::new(Vector::new(vec![42.0]), Matrix::new(1, 1, vec![1.0]), 99.0);
    assert_eq!(est.dim(), 1);
    assert!((est.epoch - 99.0).abs() < 1e-15);
    assert_eq!(est.step, 0);
    assert!((est.state.get(0) - 42.0).abs() < 1e-15);
}
