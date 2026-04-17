# Observation -- Measurement processing (JDL Level 0)

Models the stages a raw sensor measurement passes through before it reaches the fusion engine: from `RawMeasurement` through `Predicted`, `InnovationComputed`, `GateChecked`, to `Accepted` or `Rejected`. This is JDL Level 0 (sub-object refinement) in the Data Fusion Lexicon. The axioms anchor two sanity checks: that an identity measurement produces zero innovation, and that a Mahalanobis validation gate accepts a measurement lying exactly at the predicted value.

Key references:
- US DoD JDL 1999: *Data Fusion Lexicon* (Level 0 = sub-object refinement)
- Bar-Shalom, Li, Kirubarajan 2001: *Estimation with Applications to Tracking and Navigation*, Chapter 2
- Mahalanobis 1936 (validation gate distance)

## Entities (6 observation stages)

| Category | Entities |
|---|---|
| Input (1) | RawMeasurement |
| Derived (3) | Predicted, InnovationComputed, GateChecked |
| Decision (2) | Accepted, Rejected |

## Qualities

| Quality | Type | Description |
|---|---|---|
| StageDescription | &'static str | One-line description of each stage (raw z_k, predicted h(x̂), innovation ν = z − h(x̂), gate applied, accepted / rejected) |

## Axioms (2)

| Axiom | Description | Source |
|---|---|---|
| InnovationZeroAtPrediction | Innovation is zero when measurement equals prediction | standard (ν = z − h(x̂)) |
| GateAcceptsMean | Validation gate accepts a measurement at the predicted value | Bar-Shalom et al. 2001 Chapter 2 |

Plus the auto-generated structural axioms from `pr4xis::ontology!` (category laws over `ObservationCategory`).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The observation ontology feeds the fusion and state ontologies directly; an explicit functor would formalize the dataflow that is currently expressed via `use` imports.

## Files

- `ontology.rs` -- `ObservationConcept` entity, `StageDescription` quality, 2 axioms, tests
- `measurement.rs` -- `Measurement` struct (raw sensor observation)
- `observation_model.rs` -- `LinearObservationModel` (H matrix and `predict`)
- `innovation.rs` -- `Innovation` (residual ν and its covariance S)
- `gating.rs` -- `ValidationGate` (Mahalanobis gate) and `gate_as_hypothesis_test`
- `likelihood.rs` -- `log_likelihood` and `likelihood` of an innovation
- `noise.rs` -- `GaussianNoise` model
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
