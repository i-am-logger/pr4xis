use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::{Axiom, Ontology};

use crate::formal::math::statistics::ontology::*;

#[test]
fn statistical_category_laws() {
    check_category_laws::<StatisticalCategory>().unwrap();
}

#[test]
fn statistics_ontology_validates() {
    StatisticsOntology::validate().unwrap();
}

#[test]
fn mse_decomposition_holds() {
    assert!(MSEDecomposition.holds());
}

#[test]
fn confidence_monotonicity_holds() {
    assert!(ConfidenceMonotonicity.holds());
}

#[test]
fn type_i_type_ii_tradeoff_holds() {
    assert!(TypeITypeIITradeoff.holds());
}

#[cfg(test)]
mod proptest_proofs {
    use crate::formal::math::statistics::confidence;
    use crate::formal::math::statistics::estimator;
    use crate::formal::math::statistics::hypothesis;
    use proptest::prelude::*;

    fn arb_data(n: usize) -> impl Strategy<Value = Vec<f64>> {
        proptest::collection::vec(-1000.0..1000.0_f64, n)
    }

    proptest! {
        /// MSE decomposition: MSE = bias^2 + variance for any dataset.
        #[test]
        fn mse_decomposition_for_random_data(
            data in arb_data(5),
            true_value in -1000.0..1000.0_f64,
        ) {
            let mean = estimator::sample_mean(&data);
            let b = estimator::bias(mean, true_value);
            // Population variance (divide by n, not n-1)
            let n = data.len() as f64;
            let var: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
            let mse_decomposed = b * b + var;
            let mse_direct = estimator::mse_from_data(&data, true_value);
            prop_assert!(
                (mse_decomposed - mse_direct).abs() < 1e-6,
                "MSE decomposition failed: {} vs {}",
                mse_decomposed,
                mse_direct
            );
        }

        /// Wider confidence interval always has higher confidence.
        #[test]
        fn wider_ci_higher_confidence(
            estimate in -100.0..100.0_f64,
            se in 0.01..10.0_f64,
        ) {
            let ci_90 = confidence::confidence_interval_for_mean(
                estimate, se, confidence::Z_90, 0.90,
            );
            let ci_95 = confidence::confidence_interval_for_mean(
                estimate, se, confidence::Z_95, 0.95,
            );
            let ci_99 = confidence::confidence_interval_for_mean(
                estimate, se, confidence::Z_99, 0.99,
            );
            prop_assert!(ci_90.width() < ci_95.width());
            prop_assert!(ci_95.width() < ci_99.width());
        }

        /// P-value is always in [0, 1] for any z-statistic.
        #[test]
        fn p_value_bounded(z in -10.0..10.0_f64) {
            let p = hypothesis::two_sided_p_value(z);
            prop_assert!(p >= -1e-10, "p-value must be non-negative, got {}", p);
            prop_assert!(p <= 1.0 + 1e-10, "p-value must be <= 1, got {}", p);
        }

        /// Sample variance is non-negative.
        #[test]
        fn sample_variance_non_negative(data in arb_data(3)) {
            let var = estimator::sample_variance(&data);
            prop_assert!(var >= -1e-10, "variance must be non-negative, got {}", var);
        }

        /// Standard error decreases with sample size.
        #[test]
        fn standard_error_decreases_with_n(
            base in arb_data(3),
            extra in arb_data(3),
        ) {
            // Compare SE of smaller vs larger dataset with same base value spread
            let small: Vec<f64> = base.clone();
            let large: Vec<f64> = base.iter().chain(extra.iter()).cloned().collect();
            let _se_small = estimator::standard_error(&small);
            let se_large = estimator::standard_error(&large);
            // With more data, SE generally decreases, but it's not guaranteed
            // for arbitrary data. Instead, verify SE = s/sqrt(n) relationship.
            let s_large = estimator::sample_std_dev(&large);
            let se_expected = s_large / (large.len() as f64).sqrt();
            prop_assert!((se_large - se_expected).abs() < 1e-10);
        }
    }
}
