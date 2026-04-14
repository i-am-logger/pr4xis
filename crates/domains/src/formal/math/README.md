# Math -- Number domains as a discrete category

Parent ontology for the `formal/math/` branch. Models the classical hierarchy of number domains (natural numbers, integers, rationals, reals, complex) as a discrete category with a subset relation. The rich mathematical content lives in the child ontologies under `formal/math/*/` — geometry, linear algebra, probability, statistics, signal processing, control theory, rotation, temporal, quantity, rigid motion.

Key references:
- Landau 1930: *Grundlagen der Analysis* (rigorous construction of ℕ → ℤ → ℚ → ℝ → ℂ)
- Halmos 1960: *Naive Set Theory* (subset relation, set-theoretic foundation)

## Entities (5)

| Category | Entities |
|---|---|
| Number domains (5) | NaturalNumbers, Integers, Rationals, Reals, Complex |

## Category

`NumberHierarchy` is a discrete category over the five number-domain entities, with `Subset` as its relation. The classical containment ℕ ⊂ ℤ ⊂ ℚ ⊂ ℝ ⊂ ℂ is the structural invariant; each inclusion is an injection that preserves the operations of the smaller domain.

## Qualities

| Quality | Type | Description |
|---|---|---|
| DomainOrder | usize | Containment rank: NaturalNumbers=0, Integers=1, Rationals=2, Reals=3, Complex=4 |

## Axioms

Auto-generated structural axioms from `define_ontology!` (category laws on the discrete category). Domain axioms for each number system live in the child ontologies — `formal/math/probability` for the real-valued measure theory, `formal/math/linear_algebra` for vector space structure over ℝ or ℂ, etc.

## Child ontologies

- [`control_theory/`](control_theory/README.md) — feedback, PID, stability, transfer functions
- [`geometry/`](geometry/README.md) — Hilbert's axioms, metric and vector-space structure
- [`linear_algebra/`](linear_algebra/README.md) — matrices, determinants, eigenvalues, Cholesky
- [`probability/`](probability/README.md) — Kolmogorov axioms, Bayes, entropy, Gaussian fusion
- [`quantity/`](quantity/README.md) — SI dimensions, units, physical constants
- [`rigid_motion/`](rigid_motion/README.md) — SE(3) as rotation + translation
- [`rotation/`](rotation/README.md) — SO(3) with quaternion/DCM/Euler/axis-angle
- [`signal_processing/`](signal_processing/README.md) — filters, sampling, spectra
- [`statistics/`](statistics/README.md) — estimators, hypothesis tests, confidence intervals
- [`temporal/`](temporal/README.md) — instants, durations, Allen's interval algebra

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../docs/use/compose-via-functor.md) to add one. The parent math ontology is a discrete substrate; functors land on the child ontologies rather than on this parent.

## Files

- `ontology.rs` -- `MathDomain`, `NumberHierarchy`, `DomainOrder`, tests
- `feynman.rs`, `fibonacci.rs`, `functions.rs`, `primes.rs`, `pythagorean.rs`, `quadratic.rs`, `sets.rs` -- classical sub-topics
- `mod.rs` -- module declarations
