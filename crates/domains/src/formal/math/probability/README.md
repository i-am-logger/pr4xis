# Probability -- Kolmogorov axioms, Bayesian inference, entropy, Gaussian fusion

Models discrete probability distributions, Bayesian update, Shannon and KL information measures, 1D Gaussian fusion, and Mahalanobis distance over a discrete category of probability concepts. Each axiom is verified numerically against a fixed set of canonical distributions at test time.

Key references:
- Kolmogorov 1933: *Grundbegriffe der Wahrscheinlichkeitsrechnung* (the three axioms)
- Bayes 1763 / Laplace 1812: Bayes' theorem
- Shannon 1948: *A Mathematical Theory of Communication* (entropy, KL divergence)
- Mahalanobis 1936: *On the generalised distance in statistics*

## Entities (8)

| Category | Entities |
|---|---|
| Sample structure (3) | SampleSpace, Event, ProbabilityMeasure |
| Random variables (2) | RandomVariable, Distribution |
| Conditional / Bayes (2) | ConditionalProbability, BayesRule |
| Information (1) | Entropy |

## Category

Discrete category over the eight probability concept entities — objects only, identity morphisms only. The mathematical content lives in the axioms over canonical distributions, not in the morphism structure.

## Qualities

| Quality | Type | Description |
|---|---|---|
| ConceptDescription | &'static str | Textual description: SampleSpace="set of all possible outcomes Ω", Event="subset of sample space, element of σ-algebra F", ProbabilityMeasure="function P: F → [0,1] satisfying Kolmogorov axioms", etc. |

## Axioms (13)

| Axiom | Description | Source |
|---|---|---|
| NonNegativity | P(E) >= 0 | Kolmogorov 1933 |
| Normalization | P(Ω) = 1 | Kolmogorov 1933 |
| EmptySetZero | P(empty set) = 0 | consequence |
| ComplementRule | P(A^c) = 1 - P(A) | consequence |
| ProbabilityBounds | 0 <= P(E) <= 1 | consequence |
| BayesTheorem | P(A|B)*P(B) = P(B|A)*P(A) | Bayes 1763 |
| GaussianFusionReducesVariance | Fusing two Gaussian estimates reduces variance (information gain) | Kalman filtering |
| KlDivergenceNonNegative | D_KL(p||q) >= 0 | Gibbs' inequality |
| KlDivergenceZeroIffEqual | D_KL(p||p) = 0 | Shannon 1948 |
| EntropyNonNegative | Shannon entropy >= 0 for discrete distributions | Shannon 1948 |
| UniformMaximizesEntropy | Uniform distribution maximizes Shannon entropy | Shannon 1948 |
| MahalanobisNonNegative | Mahalanobis distance >= 0 | Mahalanobis 1936 |
| MahalanobisReducesToEuclidean | Mahalanobis with S=I equals Euclidean distance squared | Mahalanobis 1936 |

Plus the auto-generated structural axioms from `define_ontology!` (category laws on the discrete category).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. Probability is a foundational ontology that other domains (statistics, sensor fusion, estimation) compose against; the functors will land as those domains gain explicit probabilistic morphisms.

## Files

- `ontology.rs` -- Entity, discrete category, ConceptDescription quality, 13 axioms, tests
- `bayesian.rs` -- Bayesian update: posterior, likelihood, prior, evidence
- `distribution.rs` -- `DiscreteDistribution` over a finite sample space (Kolmogorov axioms)
- `entropy.rs` -- Shannon entropy and KL divergence over discrete distributions
- `gaussian.rs` -- 1D Gaussian (`Gaussian1D`) with fusion (information form)
- `mahalanobis.rs` -- Mahalanobis distance with arbitrary covariance matrix
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
