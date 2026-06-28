//! Probability-theory ontology — Kolmogorov measure-theoretic
//! framework + Bayesian update + Shannon information theory.
//!
//! The eight concepts model the standard scaffolding of mathematical
//! probability: a sample space, a sigma-algebra of events, a
//! probability measure, random variables, distributions, conditional
//! probability, Bayes' rule, and Shannon entropy.
//!
//! # Literature
//!
//! - **Kolmogorov (1933)** *Grundbegriffe der Wahrscheinlichkeits­rechnung*
//!   — the three measure-theoretic axioms of probability
//!   (non-negativity, normalisation, countable additivity).
//! - **Bayes (1763)** "An Essay towards solving a Problem in the
//!   Doctrine of Chances", *Phil. Trans. Royal Society* 53 — the
//!   inversion identity P(A|B) = P(B|A) P(A) / P(B).
//! - **Shannon (1948)** "A Mathematical Theory of Communication",
//!   *Bell System Tech. Journal* 27 — information entropy, the
//!   maximum-entropy characterisation of the uniform distribution,
//!   the Kullback-Leibler divergence.
//! - **Kullback & Leibler (1951)** "On Information and Sufficiency",
//!   *Annals of Math. Stat.* 22(1) — non-negativity of D_KL (Gibbs'
//!   inequality).
//! - **Mahalanobis (1936)** "On the Generalised Distance in Statistics",
//!   *Proc. National Inst. of Sciences of India* 2 — Mahalanobis
//!   distance and its reduction to Euclidean distance when Σ = I.

use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::formal::math::linear_algebra::matrix::Matrix;
use crate::formal::math::linear_algebra::vector_space::Vector;

use crate::formal::math::probability::bayesian;
use crate::formal::math::probability::distribution::DiscreteDistribution;
use crate::formal::math::probability::entropy;
use crate::formal::math::probability::gaussian::Gaussian1D;
use crate::formal::math::probability::mahalanobis;

pr4xis::ontology! {
    name: "Probability",
    source: "Kolmogorov (1933) Grundbegriffe der Wahrscheinlichkeitsrechnung; Bayes (1763) An Essay towards solving a Problem in the Doctrine of Chances, Phil. Trans. Royal Society 53; Shannon (1948) A Mathematical Theory of Communication, Bell System Tech. Journal 27; Mahalanobis (1936) On the Generalised Distance in Statistics, Proc. National Inst. Sci. India 2",

    concepts: [
        SampleSpace,
        Event,
        ProbabilityMeasure,
        RandomVariable,
        Distribution,
        ConditionalProbability,
        BayesRule,
        Entropy,
    ],

    labels: {
        SampleSpace: ("en", "Sample space",
            "Kolmogorov (1933) §1: the set Omega of all elementary outcomes of a random experiment."),
        Event: ("en", "Event",
            "Kolmogorov (1933) §1: a measurable subset of the sample space; an element of the sigma-algebra F."),
        ProbabilityMeasure: ("en", "Probability measure",
            "Kolmogorov (1933) §2: a function P: F -> [0,1] satisfying the three axioms (non-negativity, P(Omega) = 1, countable additivity)."),
        RandomVariable: ("en", "Random variable",
            "Kolmogorov (1933) §3: an F-measurable function X: Omega -> R whose preimages of Borel sets are events."),
        Distribution: ("en", "Distribution",
            "Kolmogorov (1933) §3: the probability law of a random variable - the pushforward measure X_*(P) on R."),
        ConditionalProbability: ("en", "Conditional probability",
            "Kolmogorov (1933) §4: P(A|B) = P(A intersection B) / P(B), defined when P(B) > 0."),
        BayesRule: ("en", "Bayes rule",
            "Bayes (1763); Kolmogorov (1933) §4: P(A|B) = P(B|A) P(A) / P(B); the inversion identity for conditional probability."),
        Entropy: ("en", "Entropy",
            "Shannon (1948) §6: H(X) = -sum p(x) log p(x); the expected information content of X measured in nats (or bits if log_2)."),
    },

    is_a: [
        // Kolmogorov hierarchy: an Event is a measurable subset of the SampleSpace.
        (Event, SampleSpace),
        // BayesRule is a specialised form of ConditionalProbability inversion.
        (BayesRule, ConditionalProbability),
    ],

    has_a: [
        // A ProbabilityMeasure is defined on the events of a SampleSpace.
        (ProbabilityMeasure, Event),
        // A RandomVariable's law is a Distribution.
        (RandomVariable, Distribution),
        // A Distribution determines Entropy (Shannon 1948).
        (Distribution, Entropy),
    ],
}

/// Quality: short symbolic description of each probability concept,
/// matching the citation column in the ontology header.
#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = ProbabilityConcept;
    type Value = &'static str;

    fn get(&self, c: &ProbabilityConcept) -> Option<&'static str> {
        use ProbabilityConcept as P;
        Some(match c {
            P::SampleSpace => "set Omega of all possible outcomes (Kolmogorov 1933 §1)",
            P::Event => "subset of sample space, element of sigma-algebra F",
            P::ProbabilityMeasure => "P: F -> [0,1] satisfying Kolmogorov axioms",
            P::RandomVariable => "measurable function X: Omega -> R",
            P::Distribution => "probability law of a random variable",
            P::ConditionalProbability => "P(A|B) = P(A intersection B) / P(B)",
            P::BayesRule => "P(A|B) = P(B|A) P(A) / P(B)",
            P::Entropy => "H(X) = -sum p(x) ln p(x)",
        })
    }
}

impl Ontology for ProbabilityOntology {
    type Cat = ProbabilityCategory;
    type Qual = ConceptDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        let mut axioms = pr4xis::ontology::reasoning::structural_axioms_for::<Self::Cat>();
        axioms.push(Box::new(NonNegativity));
        axioms.push(Box::new(Normalization));
        axioms.push(Box::new(EmptySetZero));
        axioms.push(Box::new(ComplementRule));
        axioms.push(Box::new(ProbabilityBounds));
        axioms.push(Box::new(BayesTheorem));
        axioms.push(Box::new(GaussianFusionReducesVariance));
        axioms.push(Box::new(KlDivergenceNonNegative));
        axioms.push(Box::new(KlDivergenceZeroIffEqual));
        axioms.push(Box::new(EntropyNonNegative));
        axioms.push(Box::new(UniformMaximizesEntropy));
        axioms.push(Box::new(MahalanobisNonNegative));
        axioms.push(Box::new(MahalanobisReducesToEuclidean));
        axioms
    }
}

// ---------------------------------------------------------------------------
// Domain axioms — Kolmogorov + Bayes + Shannon + Mahalanobis.
// ---------------------------------------------------------------------------

/// Kolmogorov (1933) Axiom 1: P(E) ≥ 0 for every event E.
pub struct NonNegativity;

impl Axiom for NonNegativity {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for dist in &canonical_distributions() {
            if dist.probabilities.iter().any(|&p| p < 0.0) {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "NonNegativity",
        "Kolmogorov axiom 1: P(E) >= 0 (non-negativity)",
        "Kolmogorov (1933) Grundbegriffe der Wahrscheinlichkeitsrechnung §2"
    );
}

pr4xis::register_axiom!(
    NonNegativity,
    "Kolmogorov (1933) Grundbegriffe der Wahrscheinlichkeitsrechnung §2"
);

/// Kolmogorov (1933) Axiom 2: P(Ω) = 1 — normalisation.
pub struct Normalization;

impl Axiom for Normalization {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for dist in &canonical_distributions() {
            let sum: f64 = dist.probabilities.iter().sum();
            if (sum - 1.0).abs() > 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "Normalization",
        "Kolmogorov axiom 2: P(Omega) = 1",
        "Kolmogorov (1933) Grundbegriffe der Wahrscheinlichkeitsrechnung §2"
    );
}

pr4xis::register_axiom!(
    Normalization,
    "Kolmogorov (1933) Grundbegriffe der Wahrscheinlichkeitsrechnung §2"
);

/// Consequence of Kolmogorov axioms: P(∅) = 0.
pub struct EmptySetZero;

impl Axiom for EmptySetZero {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for dist in &canonical_distributions() {
            if dist.event_prob(&[]).abs() > 1e-15 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EmptySetZero",
        "P(empty set) = 0 (consequence of Kolmogorov axioms)",
        "Kolmogorov (1933) Grundbegriffe der Wahrscheinlichkeitsrechnung §2"
    );
}

pr4xis::register_axiom!(
    EmptySetZero,
    "Kolmogorov (1933) Grundbegriffe der Wahrscheinlichkeitsrechnung §2"
);

/// Complement rule: P(A^c) = 1 - P(A), derivable from the Kolmogorov
/// axioms via finite additivity over (A, A^c).
pub struct ComplementRule;

impl Axiom for ComplementRule {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for dist in &canonical_distributions() {
            if dist.size() < 2 {
                continue;
            }
            let event = vec![0];
            let pa = dist.event_prob(&event);
            let pac = dist.complement_prob(&event);
            if (pa + pac - 1.0).abs() > 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ComplementRule",
        "P(A^c) = 1 - P(A) (complement rule)",
        "Kolmogorov (1933) Grundbegriffe der Wahrscheinlichkeitsrechnung §2"
    );
}

pr4xis::register_axiom!(
    ComplementRule,
    "Kolmogorov (1933) Grundbegriffe der Wahrscheinlichkeitsrechnung §2"
);

/// Probability bounds: 0 ≤ P(E) ≤ 1 for every event E — corollary of
/// the Kolmogorov axioms.
pub struct ProbabilityBounds;

impl Axiom for ProbabilityBounds {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for dist in &canonical_distributions() {
            if dist
                .probabilities
                .iter()
                .any(|&p| !(0.0..=1.0).contains(&p))
            {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "ProbabilityBounds",
        "0 <= P(E) <= 1 for every event E",
        "Kolmogorov (1933) Grundbegriffe der Wahrscheinlichkeitsrechnung §2"
    );
}

pr4xis::register_axiom!(
    ProbabilityBounds,
    "Kolmogorov (1933) Grundbegriffe der Wahrscheinlichkeitsrechnung §2"
);

/// Bayes' theorem: P(A|B) P(B) = P(B|A) P(A).
/// Bayes (1763) Proposition 5.
pub struct BayesTheorem;

impl Axiom for BayesTheorem {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let priors = [0.3, 0.7];
        let likelihoods = [0.9, 0.2];
        let posteriors = bayesian::bayesian_update(&priors, &likelihoods).unwrap();
        let sum: f64 = posteriors.iter().sum();
        if (sum - 1.0).abs() > 1e-10 {
            return Err(Box::new(SimpleCounterexample::new(self.meta())));
        }
        let ev = bayesian::evidence(&priors, &likelihoods);
        for i in 0..2 {
            let lhs = posteriors[i] * ev;
            let rhs = likelihoods[i] * priors[i];
            if (lhs - rhs).abs() > 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "BayesTheorem",
        "Bayes' theorem: P(A|B) P(B) = P(B|A) P(A)",
        "Bayes (1763) Phil. Trans. Royal Society 53, Proposition 5"
    );
}

pr4xis::register_axiom!(
    BayesTheorem,
    "Bayes (1763) Phil. Trans. Royal Society 53, Proposition 5"
);

/// Gaussian fusion: combining two independent Gaussian estimates by
/// inverse-variance weighting strictly reduces the posterior variance
/// below either prior — the BLUE result for Gaussians (Kalman 1960
/// §III, applied to the static case).
pub struct GaussianFusionReducesVariance;

impl Axiom for GaussianFusionReducesVariance {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let cases = [
            (Gaussian1D::new(0.0, 4.0), Gaussian1D::new(1.0, 4.0)),
            (Gaussian1D::new(5.0, 1.0), Gaussian1D::new(5.0, 9.0)),
            (Gaussian1D::new(-3.0, 2.0), Gaussian1D::new(3.0, 8.0)),
        ];
        for (g1, g2) in &cases {
            let fused = g1.fuse(g2);
            if fused.variance >= g1.variance.min(g2.variance) + 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "GaussianFusionReducesVariance",
        "fusing two Gaussian estimates by inverse-variance weighting reduces variance",
        "Kalman (1960) A New Approach to Linear Filtering and Prediction Problems, ASME J. Basic Engineering 82(D) §III"
    );
}

pr4xis::register_axiom!(
    GaussianFusionReducesVariance,
    "Kalman (1960) A New Approach to Linear Filtering and Prediction Problems, ASME J. Basic Engineering 82(D) §III"
);

/// Gibbs' inequality: D_KL(p || q) ≥ 0 for every pair of distributions
/// on the same sample space. Kullback & Leibler (1951) §3.
pub struct KlDivergenceNonNegative;

impl Axiom for KlDivergenceNonNegative {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let dists = canonical_distributions();
        for p in &dists {
            for q in &dists {
                if p.size() != q.size() {
                    continue;
                }
                let kl = entropy::kl_divergence_discrete(&p.probabilities, &q.probabilities);
                if kl < -1e-10 {
                    return Err(Box::new(SimpleCounterexample::new(self.meta())));
                }
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "KlDivergenceNonNegative",
        "Gibbs' inequality: D_KL(p||q) >= 0",
        "Kullback & Leibler (1951) On Information and Sufficiency, Annals of Math. Stat. 22(1) §3"
    );
}

pr4xis::register_axiom!(
    KlDivergenceNonNegative,
    "Kullback & Leibler (1951) On Information and Sufficiency, Annals of Math. Stat. 22(1) §3"
);

/// KL divergence vanishes iff p = q: D_KL(p || p) = 0.
/// Kullback & Leibler (1951) §3.
pub struct KlDivergenceZeroIffEqual;

impl Axiom for KlDivergenceZeroIffEqual {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for p in &canonical_distributions() {
            let kl = entropy::kl_divergence_discrete(&p.probabilities, &p.probabilities);
            if kl.abs() > 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "KlDivergenceZeroIffEqual",
        "D_KL(p||p) = 0 (KL divergence vanishes for identical distributions)",
        "Kullback & Leibler (1951) On Information and Sufficiency, Annals of Math. Stat. 22(1) §3"
    );
}

pr4xis::register_axiom!(
    KlDivergenceZeroIffEqual,
    "Kullback & Leibler (1951) On Information and Sufficiency, Annals of Math. Stat. 22(1) §3"
);

/// Shannon entropy is non-negative for every discrete distribution.
/// Shannon (1948) Theorem 2.
pub struct EntropyNonNegative;

impl Axiom for EntropyNonNegative {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        for dist in &canonical_distributions() {
            let h = entropy::shannon_entropy(&dist.probabilities);
            if h < -1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "EntropyNonNegative",
        "Shannon entropy H(X) >= 0 for discrete distributions",
        "Shannon (1948) A Mathematical Theory of Communication, Bell System Tech. Journal 27, Theorem 2"
    );
}

pr4xis::register_axiom!(
    EntropyNonNegative,
    "Shannon (1948) A Mathematical Theory of Communication, Bell System Tech. Journal 27, Theorem 2"
);

/// Maximum-entropy theorem on a finite sample space: the uniform
/// distribution maximises Shannon entropy among all distributions on
/// the same support. Shannon (1948) Theorem 2.
pub struct UniformMaximizesEntropy;

impl Axiom for UniformMaximizesEntropy {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let n = 4;
        let uniform = DiscreteDistribution::uniform(n);
        let h_uniform = entropy::shannon_entropy(&uniform.probabilities);
        let non_uniform = DiscreteDistribution::new(vec![0.5, 0.25, 0.15, 0.1]).unwrap();
        let h_non_uniform = entropy::shannon_entropy(&non_uniform.probabilities);
        if h_uniform > h_non_uniform - 1e-10 {
            Ok(Box::new(SimpleProof::new(self.meta())))
        } else {
            Err(Box::new(SimpleCounterexample::new(self.meta())))
        }
    }

    pr4xis::axiom_meta!(
        "UniformMaximizesEntropy",
        "the uniform distribution maximises Shannon entropy on a finite sample space",
        "Shannon (1948) A Mathematical Theory of Communication, Bell System Tech. Journal 27, Theorem 2"
    );
}

pr4xis::register_axiom!(
    UniformMaximizesEntropy,
    "Shannon (1948) A Mathematical Theory of Communication, Bell System Tech. Journal 27, Theorem 2"
);

/// Mahalanobis distance is non-negative — direct from the quadratic
/// form (x − μ)^T Σ^{-1} (x − μ) on a positive-definite covariance.
/// Mahalanobis (1936) §3.
pub struct MahalanobisNonNegative;

impl Axiom for MahalanobisNonNegative {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let mean = Vector::new(vec![0.0, 0.0]);
        let cov = Matrix::new(2, 2, vec![1.0, 0.0, 0.0, 1.0]);
        let test_points = [
            Vector::new(vec![0.0, 0.0]),
            Vector::new(vec![1.0, 0.0]),
            Vector::new(vec![3.0, 4.0]),
            Vector::new(vec![-2.0, 1.0]),
        ];
        for x in &test_points {
            let d2 = mahalanobis::mahalanobis_squared(x, &mean, &cov).unwrap();
            if d2 < -1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MahalanobisNonNegative",
        "Mahalanobis distance is non-negative",
        "Mahalanobis (1936) On the Generalised Distance in Statistics, Proc. National Inst. Sci. India 2 §3"
    );
}

pr4xis::register_axiom!(
    MahalanobisNonNegative,
    "Mahalanobis (1936) On the Generalised Distance in Statistics, Proc. National Inst. Sci. India 2 §3"
);

/// Σ = I reduction: Mahalanobis distance with identity covariance
/// equals squared Euclidean distance — Mahalanobis (1936) §3.
pub struct MahalanobisReducesToEuclidean;

impl Axiom for MahalanobisReducesToEuclidean {
    fn verify(&self) -> pr4xis::logic::proof::Verdict {
        use pr4xis::logic::proof::{SimpleCounterexample, SimpleProof};
        let mean = Vector::new(vec![1.0, 2.0, 3.0]);
        let identity = Matrix::identity(3);
        let test_points = [
            Vector::new(vec![4.0, 6.0, 3.0]),
            Vector::new(vec![1.0, 2.0, 3.0]),
            Vector::new(vec![0.0, 0.0, 0.0]),
        ];
        for x in &test_points {
            let d2_mahal = mahalanobis::mahalanobis_squared(x, &mean, &identity).unwrap();
            let diff = x.sub(&mean);
            let d2_euclid = diff.norm_squared();
            if (d2_mahal - d2_euclid).abs() > 1e-10 {
                return Err(Box::new(SimpleCounterexample::new(self.meta())));
            }
        }
        Ok(Box::new(SimpleProof::new(self.meta())))
    }

    pr4xis::axiom_meta!(
        "MahalanobisReducesToEuclidean",
        "Mahalanobis distance with Sigma = I equals squared Euclidean distance",
        "Mahalanobis (1936) On the Generalised Distance in Statistics, Proc. National Inst. Sci. India 2 §3"
    );
}

pr4xis::register_axiom!(
    MahalanobisReducesToEuclidean,
    "Mahalanobis (1936) On the Generalised Distance in Statistics, Proc. National Inst. Sci. India 2 §3"
);

// ---------------------------------------------------------------------------
// Canonical test data
// ---------------------------------------------------------------------------

fn canonical_distributions() -> Vec<DiscreteDistribution> {
    vec![
        DiscreteDistribution::uniform(2),
        DiscreteDistribution::uniform(4),
        DiscreteDistribution::uniform(6),
        DiscreteDistribution::new(vec![0.5, 0.5]).unwrap(),
        DiscreteDistribution::new(vec![0.7, 0.2, 0.1]).unwrap(),
        DiscreteDistribution::new(vec![0.25, 0.25, 0.25, 0.25]).unwrap(),
        DiscreteDistribution::new(vec![0.1, 0.2, 0.3, 0.4]).unwrap(),
        DiscreteDistribution::new(vec![1.0]).unwrap(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use pr4xis::category::FinitelyGenerated;
    use pr4xis::category::laws::assert_category_laws;
    use proptest::prelude::*;

    #[pr4xis::praxis_value(Deterministic)]
    #[test]
    fn category_laws() {
        assert_category_laws::<ProbabilityCategory>();
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn ontology_validates() {
        ProbabilityOntology::validate()
            .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn eight_probability_concepts() {
        assert_eq!(ProbabilityConcept::variants().len(), 8);
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn concept_description_total() {
        let q = ConceptDescription;
        for c in ProbabilityConcept::variants() {
            assert!(q.get(&c).is_some(), "{:?} missing description", c);
        }
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn kolmogorov_axioms_hold() {
        assert!(NonNegativity.verify().is_ok());
        assert!(Normalization.verify().is_ok());
        assert!(EmptySetZero.verify().is_ok());
        assert!(ComplementRule.verify().is_ok());
        assert!(ProbabilityBounds.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn bayes_and_gaussian_hold() {
        assert!(BayesTheorem.verify().is_ok());
        assert!(GaussianFusionReducesVariance.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn information_theory_axioms_hold() {
        assert!(KlDivergenceNonNegative.verify().is_ok());
        assert!(KlDivergenceZeroIffEqual.verify().is_ok());
        assert!(EntropyNonNegative.verify().is_ok());
        assert!(UniformMaximizesEntropy.verify().is_ok());
    }

    #[pr4xis::praxis_value(Verifiable)]
    #[test]
    fn mahalanobis_axioms_hold() {
        assert!(MahalanobisNonNegative.verify().is_ok());
        assert!(MahalanobisReducesToEuclidean.verify().is_ok());
    }

    fn arb_concept() -> impl Strategy<Value = ProbabilityConcept> {
        proptest::sample::select(ProbabilityConcept::variants())
    }

    proptest! {
        #[test]
        fn prop_concept_description_total(c in arb_concept()) {
            prop_assert!(ConceptDescription.get(&c).is_some());
        }

        #[test]
        fn prop_every_arrow_is_named(_seed in any::<u32>()) {
            use pr4xis::category::{Arrow, Category};
            for m in ProbabilityCategory::morphisms() {
                prop_assert!(!m.meta().name.as_str().is_empty());
            }
        }

        #[test]
        fn prop_structural_axioms_hold(_seed in any::<u32>()) {
            for axiom in ProbabilityOntology::axioms() {
                if let Err(c) = axiom.verify() {
                    prop_assert!(
                        false,
                        "axiom failed: {}",
                        c.meta().name.as_str()
                    );
                }
            }
        }
    }

    pr4xis::register_praxis_value!(prop_concept_description_total, Verifiable);
    pr4xis::register_praxis_value!(prop_every_arrow_is_named, Explainable);
    pr4xis::register_praxis_value!(prop_structural_axioms_hold, Verifiable);
}
