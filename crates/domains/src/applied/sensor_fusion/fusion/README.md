# Fusion -- Sensor fusion engine (predict / update / track)

Models the sensor-fusion lifecycle as a category of phases (Initialized, Predicted, Updated, Diverged, Reset) with transitions between them, together with auxiliary categories for track status, composition strategy, association strategy, and fusion architecture. The central property is DETERMINISM: given identical initial state and identical action sequences, the engine's output is bit-for-bit identical, which is the foundational property for safety-critical certification. Additional axioms encode the information-theoretic direction of predict and update and the positive semi-definiteness of the covariance invariant.

Key references:
- Kalman 1960: *A New Approach to Linear Filtering and Prediction Problems*
- Maybeck 1979: *Stochastic Models, Estimation, and Control*
- Bar-Shalom, Li, Kirubarajan 2001: *Estimation with Applications to Tracking and Navigation*
- US DoD JDL 1999: *Data Fusion Lexicon* (fusion levels)
- Julier & Uhlmann 1997: *A Non-divergent Estimation Algorithm in the Presence of Unknown Correlations* (covariance intersection)

## Entities (5 fusion phases)

| Category | Entities |
|---|---|
| Steady (3) | Initialized, Predicted, Updated |
| Failure / recovery (2) | Diverged, Reset |

Additional entity enums exported from this module:

- `FusionTrackStatus` (in `track.rs`): track lifecycle states
- `CompositionStrategy` (in `composition.rs`): independent, covariance-intersection, naive
- `AssociationStrategy` (in `association.rs`): nearest-neighbor, GNN, JPDA, MHT
- `FusionArchitecture` (in `architecture.rs`): centralized, distributed, hierarchical

## Qualities

| Quality | Type | Description |
|---|---|---|
| PhaseDescription | &'static str | One-line description of each fusion phase |
| TrackStatusDescription | &'static str | Lifecycle description for each track status (`track.rs`) |
| CompositionDescription | &'static str | Description of each composition strategy (`composition.rs`) |
| ArchitectureDescription | &'static str | Description of each fusion architecture (`architecture.rs`) |
| AssignmentType, ComplexityClass | enum | Assignment shape and complexity of each association strategy (`association.rs`) |

## Axioms

**Core (`ontology.rs`, 4):**

| Axiom | Description | Source |
|---|---|---|
| Determinism | Same inputs always produce bit-for-bit identical outputs | safety-critical engineering principle |
| PredictIncreasesUncertainty | Prediction step never decreases uncertainty (no free information) | Maybeck 1979 |
| UpdateReducesUncertainty | Measurement update never increases uncertainty (information gain) | Kalman 1960 |
| CovarianceInvariant | Covariance remains positive semi-definite through predict and update | Bar-Shalom et al. 2001 |

**Engine / composition / track (5):**

| Axiom | Description | Source |
|---|---|---|
| PositiveTimeStep | Prediction dt must be positive (`engine.rs`) | standard |
| DimensionConsistency | State / matrix dimensions must match under predict and update (`engine.rs`) | standard |
| CovariancePSD | Engine preserves PSD covariance (`engine.rs`) | Bar-Shalom et al. 2001 |
| ConsistentUnderUnknownCorrelation | Covariance intersection is consistent when cross-correlation is unknown (`composition.rs`) | Julier & Uhlmann 1997 |
| AcceptsUpdates | Only live track statuses accept measurement updates (`track.rs`) | Bar-Shalom et al. 2001 |
| PreservesCorrelations | Centralized architecture preserves sensor correlations (`architecture.rs`) | Bar-Shalom et al. 2001 |

Plus the auto-generated structural axioms from `define_ontology!`.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The fusion ontology is the natural composition point for frame, sensor, observation, state, and time; explicit functors would make the current `use` dependencies into formal morphisms.

## Files

- `ontology.rs` -- `FusionPhase` entity, `PhaseDescription` quality, 4 core axioms including `Determinism`, tests
- `engine.rs` -- `FusionState`, `FusionAction`, `apply_fusion`, `Engine<FusionAction>`, `PositiveTimeStep`, `DimensionConsistency`, `CovariancePSD`
- `predict.rs` -- `constant_velocity_transition`, `constant_velocity_process_noise` helpers
- `update.rs` -- `innovation`, `innovation_covariance`, `position_observation_matrix` helpers
- `track.rs` -- `FusionTrackStatus`, `AcceptsUpdates`, `TrackStatusDescription`, `is_valid_transition`
- `composition.rs` -- `CompositionStrategy`, `ConsistentUnderUnknownCorrelation` (Julier & Uhlmann 1997), `CompositionDescription`
- `association.rs` -- `AssociationStrategy`, `AssignmentType`, `Assignment`, `ComplexityClass`, `Complexity`
- `architecture.rs` -- `FusionArchitecture`, `ArchitectureDescription`, `PreservesCorrelations`
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
