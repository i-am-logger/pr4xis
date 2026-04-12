use pr4xis::category::{Category, Entity, Relationship};
use pr4xis::ontology::{Axiom, Ontology, Quality};

use crate::science::math::linear_algebra::matrix::Matrix;
use crate::science::math::linear_algebra::vector_space::Vector;

use crate::science::math::probability::bayesian;
use crate::science::math::probability::distribution::DiscreteDistribution;
use crate::science::math::probability::entropy;
use crate::science::math::probability::gaussian::Gaussian1D;
use crate::science::math::probability::mahalanobis;

// ---------------------------------------------------------------------------
// Entity: probability concepts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProbabilityConcept {
    SampleSpace,
    Event,
    ProbabilityMeasure,
    RandomVariable,
    Distribution,
    ConditionalProbability,
    BayesRule,
    Entropy,
}

impl Entity for ProbabilityConcept {
    fn variants() -> Vec<Self> {
        vec![
            Self::SampleSpace,
            Self::Event,
            Self::ProbabilityMeasure,
            Self::RandomVariable,
            Self::Distribution,
            Self::ConditionalProbability,
            Self::BayesRule,
            Self::Entropy,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProbabilityRelation {
    pub from: ProbabilityConcept,
    pub to: ProbabilityConcept,
}

impl Relationship for ProbabilityRelation {
    type Object = ProbabilityConcept;
    fn source(&self) -> ProbabilityConcept {
        self.from
    }
    fn target(&self) -> ProbabilityConcept {
        self.to
    }
}

pub struct ProbabilityCategory;

impl Category for ProbabilityCategory {
    type Object = ProbabilityConcept;
    type Morphism = ProbabilityRelation;

    fn identity(obj: &ProbabilityConcept) -> ProbabilityRelation {
        ProbabilityRelation {
            from: *obj,
            to: *obj,
        }
    }

    fn compose(f: &ProbabilityRelation, g: &ProbabilityRelation) -> Option<ProbabilityRelation> {
        if f.to != g.from {
            return None;
        }
        Some(ProbabilityRelation {
            from: f.from,
            to: g.to,
        })
    }

    fn morphisms() -> Vec<ProbabilityRelation> {
        let concepts = ProbabilityConcept::variants();
        concepts
            .iter()
            .flat_map(|&from| {
                concepts
                    .iter()
                    .map(move |&to| ProbabilityRelation { from, to })
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ConceptDescription;

impl Quality for ConceptDescription {
    type Individual = ProbabilityConcept;
    type Value = &'static str;

    fn get(&self, c: &ProbabilityConcept) -> Option<&'static str> {
        Some(match c {
            ProbabilityConcept::SampleSpace => "set of all possible outcomes Ω",
            ProbabilityConcept::Event => "subset of sample space, element of σ-algebra F",
            ProbabilityConcept::ProbabilityMeasure => {
                "function P: F → [0,1] satisfying Kolmogorov axioms"
            }
            ProbabilityConcept::RandomVariable => "measurable function X: Ω → R",
            ProbabilityConcept::Distribution => "probability law of a random variable",
            ProbabilityConcept::ConditionalProbability => "P(A|B) = P(A∩B) / P(B)",
            ProbabilityConcept::BayesRule => "P(A|B) = P(B|A)P(A) / P(B)",
            ProbabilityConcept::Entropy => "H(X) = -Σ p(x) ln p(x)",
        })
    }
}

// ---------------------------------------------------------------------------
// Axioms — Kolmogorov (1933) + consequences
// ---------------------------------------------------------------------------

/// Kolmogorov Axiom 1: P(E) ≥ 0 for all events E.
pub struct NonNegativity;

impl Axiom for NonNegativity {
    fn description(&self) -> &str {
        "Kolmogorov axiom 1: P(E) >= 0 (non-negativity)"
    }

    fn holds(&self) -> bool {
        for dist in &canonical_distributions() {
            if dist.probabilities.iter().any(|&p| p < 0.0) {
                return false;
            }
        }
        true
    }
}

/// Kolmogorov Axiom 2: P(Ω) = 1.
pub struct Normalization;

impl Axiom for Normalization {
    fn description(&self) -> &str {
        "Kolmogorov axiom 2: P(Ω) = 1 (normalization)"
    }

    fn holds(&self) -> bool {
        for dist in &canonical_distributions() {
            let sum: f64 = dist.probabilities.iter().sum();
            if (sum - 1.0).abs() > 1e-10 {
                return false;
            }
        }
        true
    }
}

/// Consequence: P(∅) = 0.
pub struct EmptySetZero;

impl Axiom for EmptySetZero {
    fn description(&self) -> &str {
        "P(empty set) = 0 (consequence of axioms)"
    }

    fn holds(&self) -> bool {
        for dist in &canonical_distributions() {
            let empty_prob = dist.event_prob(&[]);
            if empty_prob.abs() > 1e-15 {
                return false;
            }
        }
        true
    }
}

/// Consequence: P(A^c) = 1 - P(A).
pub struct ComplementRule;

impl Axiom for ComplementRule {
    fn description(&self) -> &str {
        "P(A^c) = 1 - P(A) (complement rule)"
    }

    fn holds(&self) -> bool {
        for dist in &canonical_distributions() {
            let n = dist.size();
            if n < 2 {
                continue;
            }
            let event = vec![0]; // single element event
            let pa = dist.event_prob(&event);
            let pac = dist.complement_prob(&event);
            if (pa + pac - 1.0).abs() > 1e-10 {
                return false;
            }
        }
        true
    }
}

/// Consequence: 0 ≤ P(E) ≤ 1 for all events.
pub struct ProbabilityBounds;

impl Axiom for ProbabilityBounds {
    fn description(&self) -> &str {
        "0 <= P(E) <= 1 (probability bounds)"
    }

    fn holds(&self) -> bool {
        for dist in &canonical_distributions() {
            if dist
                .probabilities
                .iter()
                .any(|&p| !(0.0..=1.0).contains(&p))
            {
                return false;
            }
        }
        true
    }
}

/// Bayes' theorem: posterior * evidence = likelihood * prior.
pub struct BayesTheorem;

impl Axiom for BayesTheorem {
    fn description(&self) -> &str {
        "Bayes' theorem: P(A|B)*P(B) = P(B|A)*P(A)"
    }

    fn holds(&self) -> bool {
        let priors = [0.3, 0.7];
        let likelihoods = [0.9, 0.2];
        let posteriors = bayesian::bayesian_update(&priors, &likelihoods).unwrap();
        // Verify: posteriors sum to 1
        let sum: f64 = posteriors.iter().sum();
        if (sum - 1.0).abs() > 1e-10 {
            return false;
        }
        // Verify: posterior[i] * evidence = likelihood[i] * prior[i]
        let ev = bayesian::evidence(&priors, &likelihoods);
        for i in 0..2 {
            let lhs = posteriors[i] * ev;
            let rhs = likelihoods[i] * priors[i];
            if (lhs - rhs).abs() > 1e-10 {
                return false;
            }
        }
        true
    }
}

/// Gaussian fusion reduces variance (information gain).
pub struct GaussianFusionReducesVariance;

impl Axiom for GaussianFusionReducesVariance {
    fn description(&self) -> &str {
        "fusing two Gaussian estimates reduces variance (information gain)"
    }

    fn holds(&self) -> bool {
        let cases = [
            (Gaussian1D::new(0.0, 4.0), Gaussian1D::new(1.0, 4.0)),
            (Gaussian1D::new(5.0, 1.0), Gaussian1D::new(5.0, 9.0)),
            (Gaussian1D::new(-3.0, 2.0), Gaussian1D::new(3.0, 8.0)),
        ];
        for (g1, g2) in &cases {
            let fused = g1.fuse(g2);
            if fused.variance >= g1.variance.min(g2.variance) + 1e-10 {
                return false;
            }
        }
        true
    }
}

/// KL divergence is non-negative: D_KL(p||q) ≥ 0 (Gibbs' inequality).
pub struct KlDivergenceNonNegative;

impl Axiom for KlDivergenceNonNegative {
    fn description(&self) -> &str {
        "KL divergence >= 0 (Gibbs' inequality)"
    }

    fn holds(&self) -> bool {
        let dists = canonical_distributions();
        for p in &dists {
            for q in &dists {
                if p.size() != q.size() {
                    continue;
                }
                let kl = entropy::kl_divergence_discrete(&p.probabilities, &q.probabilities);
                if kl < -1e-10 {
                    return false;
                }
            }
        }
        true
    }
}

/// KL divergence is zero iff p = q.
pub struct KlDivergenceZeroIffEqual;

impl Axiom for KlDivergenceZeroIffEqual {
    fn description(&self) -> &str {
        "D_KL(p||p) = 0 (KL divergence zero for identical distributions)"
    }

    fn holds(&self) -> bool {
        for p in &canonical_distributions() {
            let kl = entropy::kl_divergence_discrete(&p.probabilities, &p.probabilities);
            if kl.abs() > 1e-10 {
                return false;
            }
        }
        true
    }
}

/// Mahalanobis distance is non-negative.
pub struct MahalanobisNonNegative;

impl Axiom for MahalanobisNonNegative {
    fn description(&self) -> &str {
        "Mahalanobis distance >= 0"
    }

    fn holds(&self) -> bool {
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
                return false;
            }
        }
        true
    }
}

/// Mahalanobis with identity covariance equals Euclidean distance squared.
pub struct MahalanobisReducesToEuclidean;

impl Axiom for MahalanobisReducesToEuclidean {
    fn description(&self) -> &str {
        "Mahalanobis with S=I equals Euclidean distance squared"
    }

    fn holds(&self) -> bool {
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
                return false;
            }
        }
        true
    }
}

/// Shannon entropy is non-negative for discrete distributions.
pub struct EntropyNonNegative;

impl Axiom for EntropyNonNegative {
    fn description(&self) -> &str {
        "Shannon entropy >= 0 for discrete distributions"
    }

    fn holds(&self) -> bool {
        for dist in &canonical_distributions() {
            let h = entropy::shannon_entropy(&dist.probabilities);
            if h < -1e-10 {
                return false;
            }
        }
        true
    }
}

/// Uniform distribution maximizes entropy (for fixed sample space size).
pub struct UniformMaximizesEntropy;

impl Axiom for UniformMaximizesEntropy {
    fn description(&self) -> &str {
        "uniform distribution maximizes Shannon entropy"
    }

    fn holds(&self) -> bool {
        let n = 4;
        let uniform = DiscreteDistribution::uniform(n);
        let h_uniform = entropy::shannon_entropy(&uniform.probabilities);
        let non_uniform = DiscreteDistribution::new(vec![0.5, 0.25, 0.15, 0.1]).unwrap();
        let h_non_uniform = entropy::shannon_entropy(&non_uniform.probabilities);
        h_uniform > h_non_uniform - 1e-10
    }
}

// ---------------------------------------------------------------------------
// Ontology
// ---------------------------------------------------------------------------

/// The probability ontology.
///
/// Founded on:
///   - Kolmogorov, A.N. (1933). *Grundbegriffe der Wahrscheinlichkeitsrechnung*
///   - Bayes, T. (1763). "An Essay towards solving a Problem in the Doctrine of Chances"
///   - Shannon, C.E. (1948). "A Mathematical Theory of Communication"
///   - Mahalanobis, P.C. (1936). "On the generalized distance in statistics"
///   - Fisher, R.A. (1925). "Theory of Statistical Estimation"
pub struct ProbabilityOntology;

impl Ontology for ProbabilityOntology {
    type Cat = ProbabilityCategory;
    type Qual = ConceptDescription;

    fn axioms() -> Vec<Box<dyn Axiom>> {
        vec![
            // Kolmogorov axioms
            Box::new(NonNegativity),
            Box::new(Normalization),
            Box::new(EmptySetZero),
            Box::new(ComplementRule),
            Box::new(ProbabilityBounds),
            // Bayes
            Box::new(BayesTheorem),
            // Gaussian fusion
            Box::new(GaussianFusionReducesVariance),
            // Information theory
            Box::new(KlDivergenceNonNegative),
            Box::new(KlDivergenceZeroIffEqual),
            Box::new(EntropyNonNegative),
            Box::new(UniformMaximizesEntropy),
            // Mahalanobis
            Box::new(MahalanobisNonNegative),
            Box::new(MahalanobisReducesToEuclidean),
        ]
    }
}

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
