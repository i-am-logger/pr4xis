# Time -- Sensor time synchronization

Models the temporal alignment strategies used to bring measurements from sensors operating at different rates to a common fusion epoch. The category's objects are synchronization strategies (nearest neighbor, linear interpolation, extrapolation), ordered by information content — linear interpolation uses two measurements, nearest neighbor uses one, and extrapolation uses a model without new information and is therefore the most dangerous. The axioms formalize which strategies have bounded error and which do not.

Key references:
- Bar-Shalom, Li, Kirubarajan 2001: *Estimation with Applications to Tracking and Navigation*, §6.2
- Groves 2013: *Principles of GNSS, Inertial, and Multisensor Integrated Navigation*, §17.2.4 ("Time synchronization")

## Entities (3 synchronization strategies)

| Category | Entities |
|---|---|
| Bounded (2) | NearestNeighbor, LinearInterpolation |
| Unbounded (1) | Extrapolation |

## Qualities

| Quality | Type | Description |
|---|---|---|
| ErrorBoundedness | bool | `true` for NearestNeighbor and LinearInterpolation; `false` for Extrapolation |

## Axioms (3)

| Axiom | Description | Source |
|---|---|---|
| InterpolationBounded | Linear interpolation error is bounded by O(T²) where T is the measurement period | Bar-Shalom et al. 2001 §6.2.3 |
| ExtrapolationUnbounded | Extrapolation error grows without bound (no new information) | Bar-Shalom et al. 2001 §6.2.4 |
| NearestNeighborBounded | Nearest-neighbor sync error is bounded by (T/2) × max_rate | Bar-Shalom et al. 2001 §6.2 |

Plus the auto-generated structural axioms from `define_ontology!` (category laws over `SensorTimeCategory`).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The formal temporal ontology (`crates/domains/src/formal/math/temporal/`) defines a richer category of time systems (TAI, GPS, UTC, …); an explicit functor from that category into this one would express sensor-time synchronization as a restriction of the general time-system conversion category.

## Files

- `ontology.rs` -- `SensorTimeOntology`, `ErrorBoundedness` quality, 3 axioms, tests
- `synchronization.rs` -- `SyncStrategy` enum, `interpolate`, `align_measurement`, `extrapolate`, max-error helper
- `clock.rs` -- `SensorClock` model
- `epoch.rs` -- `FusionEpoch` (common time reference)
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
