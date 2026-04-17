# AUV -- Autonomous Underwater Vehicle Sensor Ontology

Models the four canonical navigation sensors of an autonomous underwater vehicle (DVL, depth sensor, compass, ADCP) and their fusion into a navigation filter. The category is fully connected because all sensors can be correlated in the filter. Physical bounds on depth (non-negative) and DVL operation (bottom lock required) appear as axioms.

Key references:
- Kinsey, Eustice & Whitcomb 2006: *A Survey of Underwater Vehicle Navigation*
- Paull, Saeedi, Seto & Li 2014: *AUV Navigation and Localization — A Review*

## Entities (4)

| Category | Entities |
|---|---|
| AUV sensors (4) | DVL, DepthSensor, Compass, ADCP |

## Category

`AuvOntology`/`AuvCategory`/`AuvConcept` via `pr4xis::ontology!`, relation `AuvRelation`, fully connected.

## Qualities

| Quality | Type | Description |
|---|---|---|
| MeasuredQuantity | &'static str | DVL=m/s vs seabed, DepthSensor=meters, Compass=rad, ADCP=m/s current profile |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| DepthNonNegative | Depth measurements are non-negative (at or below surface) | Kinsey et al. 2006 |
| DvlRequiresBottomLock | DVL velocity measurement requires bottom lock (finite altitude above seabed) | Kinsey et al. 2006 |
| (structural) | Identity and composition laws over the AuvCategory | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `AuvConcept`, `AuvCategory`, `MeasuredQuantity` quality, navigation axioms
- `engine.rs` -- navigation/fusion engine used by tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
