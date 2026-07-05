# DistributedFusion — fusing estimates across a network of peers

Distributed Kalman filtering by consensus on information contributions, network-wide covariance intersection, and the data-incest failure mode of naive additive re-fusion on cyclic topologies. The mathematics is established literature (Olfati-Saber 2005, 2007; Julier & Uhlmann 1997; Bar-Shalom 1981; Mutambara 1998) — this ontology encodes it and claims no novelty for it.

## Verification

```
cargo test -p pr4xis-domains swarm::fusion
```

Category laws, ontology validation, four domain axioms (single-point + proptest sweeps), engine guard tests, and three functor-law suites (including the two wrapper categories' own law suites).

## Concepts (6)

`NetworkFusionArchitecture` (abstract — Mutambara 1998 Ch. 1); `DistributedKalmanFilter` (Olfati-Saber 2005, 2007); `CiOverNetwork` (Julier & Uhlmann 1997); `ConsensusEstimate`, `InnovationExchange` (Olfati-Saber 2007); `DataIncest` (Bar-Shalom 1981; Mutambara 1998).

Taxonomy: `DistributedKalmanFilter` / `CiOverNetwork` is-a `NetworkFusionArchitecture`.

Custom edge kinds: `Produces` (`InnovationExchange` → `ConsensusEstimate`), `Corrupts` (`DataIncest` → `ConsensusEstimate`), `Prevents` (`CiOverNetwork` → `DataIncest`).

## Quality

- `ConsistentUnderInterPeerCorrelation` → `bool` — `CiOverNetwork` true (non-divergent for every admissible cross-correlation, Julier & Uhlmann 1997), `DistributedKalmanFilter` false (naive additive information fusion is over-confident when peer estimates are correlated, Mutambara 1998); `None` elsewhere.

## Domain axioms

| Axiom | Source | Discharged against |
|---|---|---|
| `CiFusionConsistentAcrossPeers` | Julier & Uhlmann (1997) | for every omega in the grid and every rho in [-1, 1], the CI-fused variance ≥ the realised error variance |
| `NaiveInformationFusionOverconfidentUnderCycles` | Bar-Shalom (1981); Mutambara (1998) Ch. 3 | **honest negative**: ring re-fusion yields a strictly smaller covariance; the surplus information equals the origin peer's own contribution, exactly |
| `DistributedFilterAgreesWithCentralInTheLimit` | Olfati-Saber (2007) 46th IEEE CDC | each peer's rescaled information-consensus estimate matches the centralized information-filter fuse within 1e-9 |
| `FusedCovarianceRemainsPsd` | Bar-Shalom, Li & Kirubarajan (2001) | every fixture-fused covariance passes the existing sensor-fusion symmetric-PSD validity predicate |

## Engine — code reuse, not reimplementation

[`engine.rs`](engine.rs) wraps the existing `applied/sensor_fusion/state/information.rs` `InformationEstimate` and calls its `fuse()` for **every** additive information step — CI is the fuse of the two omega-scaled information forms, the centralized filter is a fuse-fold, and the incest ring is a chain of fuses. The consensus-on-information run reuses the sibling `swarm::consensus` engine's `average_consensus_step` and stable step size per matrix/vector entry. PSD checking reuses `applied/sensor_fusion/state/covariance.rs::is_valid`. Fixtures: the omega grid over `[0, 1]` (Julier & Uhlmann 1997), the 3-peer ring (smallest cycle, Bar-Shalom 1981), and planar local estimates with distinct means and variances.

## Cross-functors

- [`architecture_functor.rs`](architecture_functor.rs) — onto the existing bare enum `FusionArchitecture` via a discrete-category wrapper (the `pipeline_step_functor` technique): every concept lands on `Distributed`, so the functor factors through that one object.
- [`composition_functor.rs`](composition_functor.rs) — onto the existing bare enum `CompositionStrategy` via an **indiscrete**-category wrapper (one arrow per ordered pair, the dual of the discrete case): a discrete wrapper cannot host this functor because the source category is connected while the object map must separate `CiOverNetwork → CovarianceIntersection` from `DistributedKalmanFilter → InformationFusion`; the indiscrete wrapper's singleton hom-sets carry no relational claims, so the separation stays lawful and honest.
- [`consensus_functor.rs`](consensus_functor.rs) — onto the sibling `Consensus` ontology: architectures → `ConsensusProtocol`, `InnovationExchange → GossipRound`, `ConsensusEstimate → Convergence`, `DataIncest → Disagreement` (documented reading).

## Files

- `ontology.rs` — `DistributedFusionOntology`, the quality, four domain axioms
- `engine.rs` — CI / incest-ring / consensus-on-information fixtures over `InformationEstimate`
- `architecture_functor.rs`, `composition_functor.rs`, `consensus_functor.rs`
- `tests.rs` — proptest sweeps + guard test
- `mod.rs`, `README.md`, `citings.md`
