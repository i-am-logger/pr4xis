# Single-Target -- Kinematic State Ontology

Models the components of a single-target kinematic state (position, velocity, acceleration, turn rate) as entities in a category whose relation is `StateDerivative`. Each component carries its dimensionality as a quality, and the classical kinematic relationship (velocity is the time derivative of position) appears as a domain axiom.

Key references:
- Bar-Shalom, Li & Kirubarajan 2001: *Estimation with Applications to Tracking and Navigation*, ch. 6
- Li & Jilkov 2003: *Survey of Maneuvering Target Tracking*

## Entities (4)

| Category | Entities |
|---|---|
| State components (4) | Position, Velocity, Acceleration, TurnRate |

## Category

`SingleTargetOntology for TargetStateCategory` via `define_ontology!`, relation `StateDerivative`.

## Qualities

| Quality | Type | Description |
|---|---|---|
| ComponentDimension | usize | Position=3, Velocity=3, Acceleration=3, TurnRate=1 |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| VelocityDerivesFromPosition | Velocity is the time derivative of position | Bar-Shalom et al. 2001 |
| (structural) | Identity and composition laws over the TargetStateCategory | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `TargetStateComponent`, `TargetStateCategory`, `ComponentDimension` quality, kinematic axiom
- `motion_model.rs` -- motion models (CV, CA, CTRV) used by the filter
- `engine.rs` -- single-target tracking engine used by tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
