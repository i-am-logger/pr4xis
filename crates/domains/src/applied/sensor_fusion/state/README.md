# State -- State estimation concepts

Models the core concepts of Bayesian state estimation: the state vector x̂, the error covariance P, the information matrix Y = P⁻¹, and the Cramér–Rao lower bound. The ontology captures the duality between covariance and information form — fusion is additive in information form but multiplicative in covariance form — and verifies that the round-trip conversion preserves an estimate exactly, which is the formal statement of the duality.

Key references:
- Kalman 1960: *A New Approach to Linear Filtering and Prediction Problems*
- Maybeck 1979: *Stochastic Models, Estimation, and Control*
- Mutambara 1998: *Decentralized Estimation and Control for Multisensor Systems* (information filter)
- Bar-Shalom, Li, Kirubarajan 2001: *Estimation with Applications to Tracking and Navigation*
- Cramér 1946 / Rao 1945 (CRLB)

## Entities (4 estimation concepts)

| Category | Entities |
|---|---|
| Primary (2) | StateVector, Covariance |
| Dual form (1) | InformationMatrix |
| Lower bound (1) | CRLB |

## Qualities

| Quality | Type | Description |
|---|---|---|
| ConceptDescription | &'static str | One-line description (x̂, P symmetric PSD, Y = P⁻¹, J⁻¹ lower bound on variance) |

## Axioms (3)

| Axiom | Description | Source |
|---|---|---|
| CovarianceIsPSD | Covariance of a valid estimate is symmetric positive semi-definite | Bar-Shalom et al. 2001 |
| InformationRoundtrip | state → information → state roundtrip preserves the estimate | Mutambara 1998 |
| InformationFusionAdditive | Information fusion is additive: Y_fused = Y₁ + Y₂ | Mutambara 1998 |

Plus the auto-generated structural axioms from `pr4xis::ontology!` (category laws over `StateEstimationCategory`).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The state ontology is the target of functors from every estimator (Kalman filter, information filter, particle filter) and from the fusion engine which operates on `StateEstimate` directly.

## Files

- `ontology.rs` -- `StateEstimationConcept` entity, `ConceptDescription` quality, 3 axioms, canonical estimates, tests
- `estimate.rs` -- `StateEstimate { state, covariance, epoch }` — the primary runtime type
- `vector.rs` -- `StateComponent` and `StateLayout` (which dimensions mean what)
- `covariance.rs` -- `ensure_symmetric`, `is_valid`, `std_dev`, `correlation`, `total_uncertainty`
- `information.rs` -- `InformationEstimate` with `from_estimate`, `to_estimate`, `fuse`
- `process_model.rs` -- `LinearProcessModel` (transition + process noise)
- `crlb.rs` -- `crlb`, `is_efficient`, `is_consistent` (Cramér–Rao checks)
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
