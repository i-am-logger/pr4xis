use pr4xis::category::Entity;
use pr4xis::define_dense_category;
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::science::math::statistics::confidence;
use crate::science::math::statistics::estimator;

// ---------------------------------------------------------------------------
// Entity: statistical concepts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum StatisticalConcept {
    Estimator,
    Hypothesis,
    ConfidenceInterval,
    TestStatistic,
    PValue,
    SignificanceLevel,
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

define_dense_category! {
    /// Discrete category over statistical concept entities.
    pub StatisticalCategory {
        entity: StatisticalConcept,
        relation: StatisticalRelation,
    }
}

// ---------------------------------------------------------------------------
// Quality: concept descriptions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = StatisticalConcept;
    type Value = &'static str;

    fn get(&self, c: &StatisticalConcept) -> Option<&'static str> {
        Some(match c {
            StatisticalConcept::Estimator => {
                "function of sample data that estimates a population parameter"
            }
            StatisticalConcept::Hypothesis => {
                "statement about a population parameter (H0: null, H1: alternative)"
            }
            StatisticalConcept::ConfidenceInterval => {
                "interval [θ̂ - z*σ, θ̂ + z*σ] containing true parameter with given probability"
            }
            StatisticalConcept::TestStatistic => {
                "summary of data used to decide between hypotheses"
            }
            StatisticalConcept::PValue => {
                "probability of observing data at least as extreme as observed, given H0 is true"
            }
            StatisticalConcept::SignificanceLevel => {
                "threshold α for rejecting H0; P(Type I error) = α"
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms — Fisher (1925), Neyman-Pearson (1933)
// ---------------------------------------------------------------------------

/// MSE decomposition: MSE = bias² + variance.
///
/// For any estimator θ̂ of θ:
/// E[(θ̂ - θ)²] = (E[θ̂] - θ)² + Var(θ̂)
pub struct MSEDecomposition;

impl Axiom for MSEDecomposition {
    fn description(&self) -> &str {
        "MSE decomposition: MSE = bias^2 + variance"
    }

    fn holds(&self) -> bool {
        // Verify the decomposition with concrete examples
        let test_cases = [
            // (estimates, true_value)
            (vec![1.0, 2.0, 3.0, 4.0, 5.0], 3.0),
            (vec![10.0, 10.5, 9.5, 10.0], 9.0),
            (vec![0.0, 0.0, 0.0], 0.0),
            (vec![-1.0, 1.0, -1.0, 1.0], 0.5),
        ];

        for (estimates, true_value) in &test_cases {
            let mean_est = estimator::sample_mean(estimates);
            let b = estimator::bias(mean_est, *true_value);
            // Variance of estimator values (population variance, not sample)
            let n = estimates.len() as f64;
            let var: f64 = estimates
                .iter()
                .map(|&x| (x - mean_est).powi(2))
                .sum::<f64>()
                / n;
            let mse_decomposed = b * b + var;
            let mse_direct = estimator::mse_from_data(estimates, *true_value);

            if (mse_decomposed - mse_direct).abs() > 1e-10 {
                return false;
            }
        }
        true
    }
}

/// Confidence monotonicity: wider interval implies higher confidence.
///
/// For the same data, increasing the z-score widens the interval
/// and increases the confidence level.
pub struct ConfidenceMonotonicity;

impl Axiom for ConfidenceMonotonicity {
    fn description(&self) -> &str {
        "wider confidence interval implies higher confidence level"
    }

    fn holds(&self) -> bool {
        let estimate = 5.0;
        let se = 1.0;

        let ci_90 = confidence::confidence_interval_for_mean(estimate, se, confidence::Z_90, 0.90);
        let ci_95 = confidence::confidence_interval_for_mean(estimate, se, confidence::Z_95, 0.95);
        let ci_99 = confidence::confidence_interval_for_mean(estimate, se, confidence::Z_99, 0.99);

        // Wider interval => higher confidence
        ci_90.width() < ci_95.width() && ci_95.width() < ci_99.width()
    }
}

/// Type I / Type II tradeoff: decreasing α increases β for a fixed sample size.
///
/// In the Neyman-Pearson framework, making it harder to reject H0 (lower α)
/// makes it easier to miss a true effect (higher β).
pub struct TypeITypeIITradeoff;

impl Axiom for TypeITypeIITradeoff {
    fn description(&self) -> &str {
        "Type I / Type II error tradeoff: lower significance level means wider acceptance region"
    }

    fn holds(&self) -> bool {
        // For a fixed test scenario, a stricter α requires a more extreme test statistic
        // to reject H0. We verify this via z-score thresholds.
        // At α=0.10, z_crit ≈ 1.645; at α=0.05, z_crit ≈ 1.960; at α=0.01, z_crit ≈ 2.576
        // Stricter α → higher z threshold → harder to reject → more Type II errors
        let z_10 = confidence::Z_90; // z for α=0.10 (two-sided)
        let z_05 = confidence::Z_95; // z for α=0.05
        let z_01 = confidence::Z_99; // z for α=0.01

        z_10 < z_05 && z_05 < z_01
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The statistics ontology.
///
/// Founded on:
///   - Fisher, R.A. (1925). "Theory of Statistical Estimation."
///   - Neyman, J. & Pearson, E.S. (1933). "On the Problem of the Most Efficient Tests."
///   - Student (Gosset) (1908). "The Probable Error of a Mean." Biometrika.
pub struct StatisticsOntology;

impl Ontology for StatisticsOntology {
    type Cat = StatisticalCategory;
    type Qual = ConceptDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(MSEDecomposition),
            Box::new(ConfidenceMonotonicity),
            Box::new(TypeITypeIITradeoff),
        ]
    }
}
