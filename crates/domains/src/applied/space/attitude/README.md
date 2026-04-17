# Attitude -- Spacecraft attitude determination

Models the sensors used for spacecraft attitude determination and attaches a per-sensor angular accuracy (in arcseconds) to each. The category is fully connected because any pair of vector observations can be combined for attitude determination via Wahba's problem. The axioms fix the unit-norm constraint on the quaternion representation of SO(3) and verify that the star tracker is the most accurate sensor in the ontology — which, combined with the category's structure, encodes the standard practice of using star trackers as the primary reference.

Key references:
- Wertz 1978: *Spacecraft Attitude Determination and Control*
- Wahba 1965: *A Least Squares Estimate of Satellite Attitude* (SIAM Review)
- Markley & Crassidis 2014: *Fundamentals of Spacecraft Attitude Determination and Control*
- Shuster 1993: *A Survey of Attitude Representations*

## Entities (4 attitude sensors)

| Category | Entities |
|---|---|
| Stellar (1) | StarTracker |
| Solar (1) | SunSensor |
| Planetary (1) | EarthHorizon |
| Magnetic (1) | Magnetometer |

## Qualities

| Quality | Type | Description |
|---|---|---|
| SensorAccuracy | f64 | 1-sigma angular accuracy in arcseconds — StarTracker ~1, SunSensor ~60, EarthHorizon ~3600, Magnetometer ~7200 |

## Axioms (2)

| Axiom | Description | Source |
|---|---|---|
| QuaternionUnitNorm | Attitude quaternion must have unit norm (|q| = 1) — SO(3) representation constraint | Shuster 1993; Markley & Crassidis 2014 |
| StarTrackerMostAccurate | Star tracker has the smallest arcsecond accuracy among attitude sensors | Wertz 1978 |

Plus the auto-generated structural axioms from `pr4xis::ontology!` (category laws over `AttitudeCategory`).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. The celestial-navigation ontology names overlapping sensors (`StarTracker`, `SunSensor`, `HorizonSensor`); an explicit functor between the two would unify the spacecraft and terrestrial views of celestial sensing.

## Files

- `ontology.rs` -- `AttitudeConcept` entity, `AttitudeCategory`, `SensorAccuracy` quality, 2 axioms, tests
- `kinematics.rs` -- `Quaternion`, `quaternion_rate`, `propagate_attitude` (attitude kinematics on SO(3))
- `engine.rs` -- `AttitudeState` runtime type and `angle_between` helper
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
