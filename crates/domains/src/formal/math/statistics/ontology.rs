//! Statistical concepts — Fisher, Neyman-Pearson estimation and testing.
//!
//! Source: Fisher (1925), Neyman & Pearson (1933).

#[allow(unused_imports)]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::statistics::confidence;
use crate::formal::math::statistics::estimator;

pr4xis::ontology! {
    name: "Statistics",
    source: "Fisher (1925); Neyman & Pearson (1933)",

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
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let test_cases = [
            (vec![1.0, 2.0, 3.0, 4.0, 5.0], 3.0),
            (vec![10.0, 10.5, 9.5, 10.0], 9.0),
            (vec![0.0, 0.0, 0.0], 0.0),
            (vec![-1.0, 1.0, -1.0, 1.0], 0.5),
        ];

        for (estimates, true_value) in &test_cases {
            let mean_est = estimator::sample_mean(estimates).value;
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
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }
    pr4xis::axiom_meta!(
        "MSEDecomposition",
        "MSE decomposition: MSE = bias^2 + variance",
        "Fisher (1925), Neyman & Pearson (1933)."
    );
}
pr4xis::register_axiom!(MSEDecomposition, "Fisher (1925), Neyman & Pearson (1933).");

/// Confidence monotonicity: wider interval implies higher confidence.
pub struct ConfidenceMonotonicity;

impl Axiom for ConfidenceMonotonicity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let estimate = 5.0;
        let se = 1.0;

        let ci_90 = confidence::confidence_interval_for_mean(estimate, se, confidence::Z_90, 0.90);
        let ci_95 = confidence::confidence_interval_for_mean(estimate, se, confidence::Z_95, 0.95);
        let ci_99 = confidence::confidence_interval_for_mean(estimate, se, confidence::Z_99, 0.99);

        if ci_90.width() < ci_95.width() && ci_95.width() < ci_99.width() {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "ConfidenceMonotonicity",
        "wider confidence interval implies higher confidence level",
        "Fisher (1925), Neyman & Pearson (1933)."
    );
}
pr4xis::register_axiom!(
    ConfidenceMonotonicity,
    "Fisher (1925), Neyman & Pearson (1933)."
);

/// Type I / Type II tradeoff: decreasing α increases β for a fixed sample size.
pub struct TypeITypeIITradeoff;

impl Axiom for TypeITypeIITradeoff {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let z_10 = confidence::Z_90;
        let z_05 = confidence::Z_95;
        let z_01 = confidence::Z_99;

        if z_10 < z_05 && z_05 < z_01 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }
    pr4xis::axiom_meta!(
        "TypeITypeIITradeoff",
        "Type I / Type II error tradeoff: lower significance level means wider acceptance region",
        "Fisher (1925), Neyman & Pearson (1933)."
    );
}
pr4xis::register_axiom!(
    TypeITypeIITradeoff,
    "Fisher (1925), Neyman & Pearson (1933)."
);

impl Ontology for StatisticsOntology {
    type Cat = StatisticsCategory;
    type Qual = ConceptDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(MSEDecomposition));
        axioms.push(Box::new(ConfidenceMonotonicity));
        axioms.push(Box::new(TypeITypeIITradeoff));
        axioms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        pr4xis::category::laws::assert_category_laws::<StatisticsCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        StatisticsOntology::validate().unwrap();
    }
}
