//! Property-based tests for the DistributedFusion ontology, engine, and
//! functors.

#![cfg(test)]

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use proptest::prelude::*;

use super::engine::{
    CI_MEAN, CORRELATION_GRID, FIXTURE_EPOCH, NUMERICAL_SLACK, ci_realised_error_variance,
    covariance_intersection,
};
use super::ontology::{
    ConsistentUnderInterPeerCorrelation, DistributedFusionCategory, DistributedFusionConcept,
    DistributedFusionOntology,
};
use crate::applied::sensor_fusion::state::estimate::StateEstimate;
use crate::applied::sensor_fusion::state::information::InformationEstimate;
use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;
use pr4xis::category::{Arrow, Category, FinitelyGenerated};
use pr4xis::ontology::{Ontology, Quality};

fn arb_concept() -> impl Strategy<Value = DistributedFusionConcept> {
    proptest::sample::select(DistributedFusionConcept::variants())
}

/// Positive scalar variances bounded away from zero, so the 1x1
/// information forms stay well-conditioned.
fn arb_variance() -> impl Strategy<Value = f64> {
    0.1..10.0f64
}

/// The open interior of the omega interval — the mixing regime where CI
/// genuinely combines both estimates.
fn arb_omega() -> impl Strategy<Value = f64> {
    0.05..0.95f64
}

fn scalar_information(variance: f64) -> Option<InformationEstimate> {
    InformationEstimate::from_estimate(&StateEstimate::new(
        Vector::new(vec![CI_MEAN]),
        Matrix::diagonal(&[variance]),
        FIXTURE_EPOCH,
    ))
}

proptest! {
    /// ConsistentUnderInterPeerCorrelation is defined exactly on the two
    /// concrete architectures (Julier & Uhlmann 1997; Mutambara 1998)
    /// and nowhere else.
    #[test]
    fn prop_consistency_exactly_on_architectures(c in arb_concept()) {
        use DistributedFusionConcept as D;
        let is_architecture = matches!(c, D::CiOverNetwork | D::DistributedKalmanFilter);
        prop_assert_eq!(
            ConsistentUnderInterPeerCorrelation.get(&c).is_some(),
            is_architecture
        );
    }

    /// CI stays conservative for arbitrary variances, omegas, and every
    /// grid correlation — the Julier & Uhlmann (1997) non-divergence
    /// claim swept beyond the fixture constants.
    #[test]
    fn prop_ci_conservative(
        var_a in arb_variance(),
        var_b in arb_variance(),
        omega in arb_omega(),
    ) {
        let a = scalar_information(var_a).expect("positive variance inverts");
        let b = scalar_information(var_b).expect("positive variance inverts");
        let fused = covariance_intersection(&a, &b, omega)
            .to_estimate(FIXTURE_EPOCH)
            .expect("mixed information inverts")
            .covariance
            .get(0, 0);
        for rho in CORRELATION_GRID {
            let realised = ci_realised_error_variance(var_a, var_b, rho, omega, fused).value;
            prop_assert!(fused + NUMERICAL_SLACK >= realised);
        }
    }

    /// The CI information matrix is the convex combination of the two
    /// inputs, so the fused scalar variance lies between the harmonic
    /// extremes 1/max(Y) and 1/min(Y) — Julier & Uhlmann (1997).
    #[test]
    fn prop_ci_between_the_inputs(
        var_a in arb_variance(),
        var_b in arb_variance(),
        omega in arb_omega(),
    ) {
        let a = scalar_information(var_a).expect("positive variance inverts");
        let b = scalar_information(var_b).expect("positive variance inverts");
        let fused = covariance_intersection(&a, &b, omega)
            .to_estimate(FIXTURE_EPOCH)
            .expect("mixed information inverts")
            .covariance
            .get(0, 0);
        let lo = var_a.min(var_b);
        let hi = var_a.max(var_b);
        prop_assert!(fused >= lo - NUMERICAL_SLACK && fused <= hi + NUMERICAL_SLACK);
    }

    /// Every arrow of the category carries a non-empty name.
    #[test]
    fn prop_every_arrow_is_named(_seed in any::<u32>()) {
        for m in DistributedFusionCategory::morphisms() {
            prop_assert!(!m.meta().name.as_str().is_empty());
        }
    }

    /// Structural + domain axioms all discharge, regardless of the
    /// sampling that drives the test.
    #[test]
    fn prop_all_axioms_hold(_seed in 0..16u32) {
        for axiom in DistributedFusionOntology::axioms() {
            if let Err(c) = axiom.verify() {
                prop_assert!(false, "axiom failed: {}", c.meta().name.as_str());
            }
        }
    }
}

pr4xis::register_praxis_value!(prop_consistency_exactly_on_architectures, Verifiable);
pr4xis::register_praxis_value!(prop_ci_conservative, Verifiable);
pr4xis::register_praxis_value!(prop_ci_between_the_inputs, Verifiable);
pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
pr4xis::register_praxis_value!(prop_all_axioms_hold, Verifiable);

/// Empty peer sets are rejected, never panicking — the guard path of
/// the fold-based fusion helpers.
#[pr4xis::praxis_value(Verifiable)]
#[test]
fn empty_peer_set_is_rejected() {
    use super::engine::{centralized_information_fusion, naive_ring_refusion};
    assert!(centralized_information_fusion(&[]).is_none());
    assert!(naive_ring_refusion(&[]).is_none());
}
