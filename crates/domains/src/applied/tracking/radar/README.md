# Radar -- Radar Measurement Ontology

Models the four components of a radar measurement (range, azimuth, elevation, Doppler) as entities, each with its SI unit as a quality. Range is structurally non-negative because it is the Euclidean norm of a 3D offset.

Key references:
- Bar-Shalom, Li & Kirubarajan 2001: *Estimation with Applications to Tracking and Navigation*, ch. 10
- Skolnik 2008: *Introduction to Radar Systems*, 3rd ed.
- Richards 2014: *Fundamentals of Radar Signal Processing*

## Entities (4)

| Category | Entities |
|---|---|
| Radar measurements (4) | Range, Azimuth, Elevation, Doppler |

## Qualities

| Quality | Type | Description |
|---|---|---|
| RadarMeasurementUnit | &'static str | Range=meters, Azimuth=radians, Elevation=radians, Doppler=m/s |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| RangeNonNegative | Radar range is non-negative (range = ‖(x,y,z)‖ ≥ 0) | Skolnik 2008 |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `RadarMeasurement` entity, `RadarMeasurementUnit` quality, `RangeNonNegative` axiom
- `coordinate.rs` -- radar coordinate-frame conversions
- `engine.rs` -- radar processing engine used by tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
