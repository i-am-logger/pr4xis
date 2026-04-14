# Statistics -- Estimators, hypothesis testing, confidence intervals

Models the classical frequentist statistics framework: point estimators with bias/variance/MSE, confidence intervals, and Neyman-Pearson hypothesis testing. The category is discrete over six statistical concepts; the axioms verify the MSE decomposition, monotonicity of confidence intervals, and the Type I / Type II error tradeoff against concrete numerical examples.

Key references:
- Fisher 1925: *Theory of Statistical Estimation*
- Neyman & Pearson 1933: *On the Problem of the Most Efficient Tests*
- Student (Gosset) 1908: *The Probable Error of a Mean* (Biometrika)

## Entities (6)

| Category | Entities |
|---|---|
| Estimation (1) | Estimator |
| Hypothesis testing (4) | Hypothesis, TestStatistic, PValue, SignificanceLevel |
| Interval estimation (1) | ConfidenceInterval |

## Category

Discrete category over the six statistical concept entities — objects only, identity morphisms only.

## Qualities

| Quality | Type | Description |
|---|---|---|
| ConceptDescription | &'static str | Textual description: Estimator="function of sample data that estimates a population parameter", Hypothesis="statement about a population parameter (H0: null, H1: alternative)", PValue="probability of observing data at least as extreme as observed, given H0 is true", etc. |

## Axioms (3)

| Axiom | Description | Source |
|---|---|---|
| MSEDecomposition | MSE = bias^2 + variance | standard |
| ConfidenceMonotonicity | Wider confidence interval implies higher confidence level | Neyman 1937 |
| TypeITypeIITradeoff | Lower significance level means wider acceptance region | Neyman & Pearson 1933 |

Plus the auto-generated structural axioms from `define_ontology!` (category laws on the discrete category).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. Statistics composes downstream of probability and is itself composed against by sensor fusion and estimation domains; the functors will land as those domains gain explicit statistical morphisms.

## Files

- `ontology.rs` -- Entity, discrete category, ConceptDescription quality, 3 axioms, tests
- `confidence.rs` -- Confidence interval construction, z-score thresholds (Z_90, Z_95, Z_99)
- `estimator.rs` -- Sample mean, bias, variance, MSE from data
- `hypothesis.rs` -- Hypothesis testing primitives (Neyman-Pearson framework)
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
