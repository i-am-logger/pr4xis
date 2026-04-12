use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::science::math::probability::ontology::*;

#[test]
fn probability_category_laws() {
    check_category_laws::<ProbabilityCategory>().unwrap();
}

#[test]
fn probability_ontology_validates() {
    ProbabilityOntology::validate().unwrap();
}

#[test]
fn kolmogorov_non_negativity() {
    assert!(NonNegativity.holds());
}

#[test]
fn kolmogorov_normalization() {
    assert!(Normalization.holds());
}

#[test]
fn empty_set_zero() {
    assert!(EmptySetZero.holds());
}

#[test]
fn complement_rule() {
    assert!(ComplementRule.holds());
}

#[test]
fn probability_bounds() {
    assert!(ProbabilityBounds.holds());
}

#[test]
fn bayes_theorem() {
    assert!(BayesTheorem.holds());
}

#[test]
fn gaussian_fusion_reduces_variance() {
    assert!(GaussianFusionReducesVariance.holds());
}

#[test]
fn kl_divergence_non_negative() {
    assert!(KlDivergenceNonNegative.holds());
}

#[test]
fn kl_divergence_zero_iff_equal() {
    assert!(KlDivergenceZeroIffEqual.holds());
}

#[test]
fn entropy_non_negative() {
    assert!(EntropyNonNegative.holds());
}

#[test]
fn uniform_maximizes_entropy() {
    assert!(UniformMaximizesEntropy.holds());
}

#[test]
fn mahalanobis_non_negative() {
    assert!(MahalanobisNonNegative.holds());
}

#[test]
fn mahalanobis_reduces_to_euclidean() {
    assert!(MahalanobisReducesToEuclidean.holds());
}

#[cfg(test)]
mod proptest_proofs {
    use crate::science::math::linear_algebra::matrix::Matrix;
    use crate::science::math::linear_algebra::vector_space::Vector;
    use crate::science::math::probability::bayesian;
    use crate::science::math::probability::distribution::DiscreteDistribution;
    use crate::science::math::probability::entropy;
    use crate::science::math::probability::gaussian::Gaussian1D;
    use crate::science::math::probability::mahalanobis;
    use proptest::prelude::*;

    fn arb_distribution(n: usize) -> impl Strategy<Value = DiscreteDistribution> {
        proptest::collection::vec(0.01..10.0_f64, n).prop_map(|raw| {
            let sum: f64 = raw.iter().sum();
            let normalized: Vec<f64> = raw.iter().map(|x| x / sum).collect();
            DiscreteDistribution::new(normalized).unwrap()
        })
    }

    fn arb_gaussian_1d() -> impl Strategy<Value = Gaussian1D> {
        (-100.0..100.0_f64, 0.01..100.0_f64).prop_map(|(mean, var)| Gaussian1D::new(mean, var))
    }

    proptest! {
        #[test]
        fn distribution_sums_to_one(dist in arb_distribution(5)) {
            let sum: f64 = dist.probabilities.iter().sum();
            prop_assert!((sum - 1.0).abs() < 1e-10);
        }

        #[test]
        fn distribution_all_non_negative(dist in arb_distribution(5)) {
            prop_assert!(dist.probabilities.iter().all(|&p| p >= 0.0));
        }

        #[test]
        fn complement_sums_to_one(dist in arb_distribution(4)) {
            let event = vec![0, 1];
            let p_event = dist.event_prob(&event);
            let p_complement = dist.complement_prob(&event);
            prop_assert!((p_event + p_complement - 1.0).abs() < 1e-10);
        }

        #[test]
        fn bayesian_posteriors_sum_to_one(
            p1 in 0.01..0.99_f64,
            l1 in 0.01..1.0_f64,
            l2 in 0.01..1.0_f64,
        ) {
            let priors = [p1, 1.0 - p1];
            let likelihoods = [l1, l2];
            let posteriors = bayesian::bayesian_update(&priors, &likelihoods).unwrap();
            let sum: f64 = posteriors.iter().sum();
            prop_assert!((sum - 1.0).abs() < 1e-10);
        }

        #[test]
        fn bayesian_posteriors_are_non_negative(
            p1 in 0.01..0.99_f64,
            l1 in 0.01..1.0_f64,
            l2 in 0.01..1.0_f64,
        ) {
            let priors = [p1, 1.0 - p1];
            let likelihoods = [l1, l2];
            let posteriors = bayesian::bayesian_update(&priors, &likelihoods).unwrap();
            prop_assert!(posteriors.iter().all(|&p| p >= 0.0));
        }

        #[test]
        fn gaussian_fusion_reduces_variance(g1 in arb_gaussian_1d(), g2 in arb_gaussian_1d()) {
            let fused = g1.fuse(&g2);
            prop_assert!(fused.variance < g1.variance.min(g2.variance) + 1e-10);
        }

        #[test]
        fn gaussian_pdf_is_non_negative(g in arb_gaussian_1d(), x in -200.0..200.0_f64) {
            prop_assert!(g.pdf(x) >= 0.0);
        }

        #[test]
        fn gaussian_pdf_max_at_mean(g in arb_gaussian_1d()) {
            let pdf_mean = g.pdf(g.mean);
            let pdf_offset = g.pdf(g.mean + g.std_dev());
            prop_assert!(pdf_mean >= pdf_offset - 1e-15);
        }

        #[test]
        fn log_odds_roundtrip(p in 0.01..0.99_f64) {
            let lo = bayesian::log_odds(p);
            let p2 = bayesian::from_log_odds(lo);
            prop_assert!((p - p2).abs() < 1e-10);
        }

        #[test]
        fn entropy_non_negative(dist in arb_distribution(4)) {
            let h = entropy::shannon_entropy(&dist.probabilities);
            prop_assert!(h >= -1e-10);
        }

        #[test]
        fn kl_self_is_zero(dist in arb_distribution(4)) {
            let kl = entropy::kl_divergence_discrete(&dist.probabilities, &dist.probabilities);
            prop_assert!(kl.abs() < 1e-10);
        }

        #[test]
        fn kl_is_non_negative(p in arb_distribution(4), q in arb_distribution(4)) {
            let kl = entropy::kl_divergence_discrete(&p.probabilities, &q.probabilities);
            prop_assert!(kl >= -1e-10);
        }

        #[test]
        fn mahalanobis_at_mean_is_zero(
            mx in -10.0..10.0_f64,
            my in -10.0..10.0_f64,
        ) {
            let mean = Vector::new(vec![mx, my]);
            let cov = Matrix::new(2, 2, vec![1.0, 0.0, 0.0, 1.0]);
            let d2 = mahalanobis::mahalanobis_squared(&mean, &mean, &cov).unwrap();
            prop_assert!(d2.abs() < 1e-10);
        }

        #[test]
        fn mahalanobis_is_non_negative(
            x1 in -10.0..10.0_f64, x2 in -10.0..10.0_f64,
            m1 in -10.0..10.0_f64, m2 in -10.0..10.0_f64,
        ) {
            let x = Vector::new(vec![x1, x2]);
            let mean = Vector::new(vec![m1, m2]);
            let cov = Matrix::new(2, 2, vec![2.0, 0.5, 0.5, 3.0]); // PD
            let d2 = mahalanobis::mahalanobis_squared(&x, &mean, &cov).unwrap();
            prop_assert!(d2 >= -1e-10);
        }
    }
}
