//! Statistical concepts — Fisher, Neyman-Pearson estimation and testing.
//!
//! Source: Fisher (1925), Neyman & Pearson (1933).

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::statistics::confidence;
use crate::formal::math::statistics::estimator;

pr4xis::ontology! {
    name: "Statistics",
    source: "Fisher (1925); Neyman & Pearson (1933)",
    being: AbstractObject,

    concepts: [Estimator, Hypothesis, ConfidenceInterval, TestStatistic, PValue, SignificanceLevel],

    labels: {
        Estimator: ("en", "Estimator", "Function of sample data that estimates a population parameter."),
        Hypothesis: ("en", "Hypothesis", "Statement about a population parameter (H0: null, H1: alternative)."),
        ConfidenceInterval: ("en", "Confidence interval", "Interval [θ̂ - z*σ, θ̂ + z*σ] containing true parameter with given probability."),
        TestStatistic: ("en", "Test statistic", "Summary of data used to decide between hypotheses."),
        PValue: ("en", "P-value", "Probability of observing data at least as extreme as observed, given H0 is true."),
        SignificanceLevel: ("en", "Significance level", "Threshold α for rejecting H0; P(Type I error) = α."),
    },
}

#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = StatisticsConcept;
    type Value = &'static str;

    fn get(&self, c: &StatisticsConcept) -> Option<&'static str> {
        Some(match c {
            StatisticsConcept::Estimator => {
                "function of sample data that estimates a population parameter"
            }
            StatisticsConcept::Hypothesis => {
                "statement about a population parameter (H0: null, H1: alternative)"
            }
            StatisticsConcept::ConfidenceInterval => {
                "interval [θ̂ - z*σ, θ̂ + z*σ] containing true parameter with given probability"
            }
            StatisticsConcept::TestStatistic => "summary of data used to decide between hypotheses",
            StatisticsConcept::PValue => {
                "probability of observing data at least as extreme as observed, given H0 is true"
            }
            StatisticsConcept::SignificanceLevel => {
                "threshold α for rejecting H0; P(Type I error) = α"
            }
        })
    }
}

/// MSE decomposition: MSE = bias² + variance.
pub struct MSEDecomposition;

impl Axiom for MSEDecomposition {
    fn description(&self) -> &str {
        "MSE decomposition: MSE = bias^2 + variance"
    }

    fn holds(&self) -> bool {
        let test_cases = [
            (vec![1.0, 2.0, 3.0, 4.0, 5.0], 3.0),
            (vec![10.0, 10.5, 9.5, 10.0], 9.0),
            (vec![0.0, 0.0, 0.0], 0.0),
            (vec![-1.0, 1.0, -1.0, 1.0], 0.5),
        ];

        for (estimates, true_value) in &test_cases {
            let mean_est = estimator::sample_mean(estimates);
            let b = estimator::bias(mean_est, *true_value);
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
pr4xis::register_axiom!(MSEDecomposition);

/// Confidence monotonicity: wider interval implies higher confidence.
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

        ci_90.width() < ci_95.width() && ci_95.width() < ci_99.width()
    }
}
pr4xis::register_axiom!(ConfidenceMonotonicity);

/// Type I / Type II tradeoff: decreasing α increases β for a fixed sample size.
pub struct TypeITypeIITradeoff;

impl Axiom for TypeITypeIITradeoff {
    fn description(&self) -> &str {
        "Type I / Type II error tradeoff: lower significance level means wider acceptance region"
    }

    fn holds(&self) -> bool {
        let z_10 = confidence::Z_90;
        let z_05 = confidence::Z_95;
        let z_01 = confidence::Z_99;

        z_10 < z_05 && z_05 < z_01
    }
}
pr4xis::register_axiom!(TypeITypeIITradeoff);

impl Ontology for StatisticsOntology {
    type Cat = StatisticsCategory;
    type Qual = ConceptDescription;

    fn structural_axioms() -> Vec<Box<dyn Axiom>> {
        Self::generated_structural_axioms()
    }

    fn domain_axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            Box::new(MSEDecomposition),
            Box::new(ConfidenceMonotonicity),
            Box::new(TypeITypeIITradeoff),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_laws() {
        pr4xis::category::validate::check_category_laws::<StatisticsCategory>().unwrap();
    }

    #[test]
    fn ontology_validates() {
        StatisticsOntology::validate().unwrap();
    }
}
