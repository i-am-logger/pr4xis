# Structural -- Structural Health Monitoring Ontology

Models the three canonical structural health monitoring sensor types (strain gauge, accelerometer, crack sensor) and their fusion into damage assessment. The category is fully connected because all sensors contribute to a common damage picture. Material-mechanics bounds appear as axioms: elastic strain limits (Hooke's law) and crack monotonicity (Paris' law).

Key references:
- Farrar & Worden 2007: *An Introduction to Structural Health Monitoring*
- Paris & Erdogan 1963: *A Critical Analysis of Crack Propagation Laws* (Paris' law)

## Entities (3)

| Category | Entities |
|---|---|
| Structural sensors (3) | StrainGauge, Accelerometer, CrackSensor |

## Category

`StructuralOntology`/`StructuralCategory`/`StructuralConcept` via `pr4xis::ontology!`, relation `StructuralRelation`, fully connected (all sensors contribute to damage assessment).

## Qualities

| Quality | Type | Description |
|---|---|---|
| SensorMeasurand | &'static str | StrainGauge=microstrain, Accelerometer=m/s^2, CrackSensor=crack length mm |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| StrainBoundedElastic | Strain is bounded within the elastic deformation region (Hooke's law) | Farrar & Worden 2007 |
| CrackMonotonicity | Crack length is non-negative and non-decreasing (fatigue cracks only grow) | Paris & Erdogan 1963 |
| (structural) | Identity and composition laws over the StructuralCategory | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `StructuralConcept`, `StructuralCategory`, `SensorMeasurand` quality, mechanics axioms
- `engine.rs` -- monitoring engine used by tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
