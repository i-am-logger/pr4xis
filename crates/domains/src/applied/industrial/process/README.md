# Process -- Process Control Variables Ontology

Models the four canonical process variables in industrial control (temperature, pressure, flow, level) and their couplings. The category is fully connected because process variables are typically coupled: pressure affects flow, temperature affects pressure, level depends on flow. Each variable carries its SI unit as a quality, and thermodynamic bounds (absolute zero, non-negative absolute pressure) appear as axioms.

Key references:
- Ogunnaike & Ray 1994: *Process Dynamics, Modeling, and Control*
- Seborg, Edgar, Mellichamp & Doyle 2011: *Process Dynamics and Control*

## Entities (4)

| Category | Entities |
|---|---|
| Process variables (4) | Temperature, Pressure, Flow, Level |

## Category

`ProcessOntology`/`ProcessCategory`/`ProcessConcept` via `pr4xis::ontology!`, relation `ProcessRelation`, fully connected. All category laws verified structurally.

## Qualities

| Quality | Type | Description |
|---|---|---|
| PhysicalUnit | &'static str | SI unit: Temperature=K, Pressure=Pa, Flow=m^3/s, Level=m |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| TemperatureAboveAbsoluteZero | Temperature must be ≥ 0 K (third law of thermodynamics) | Ogunnaike & Ray 1994 |
| PressureNonNegative | Absolute pressure is non-negative (bounded below by vacuum) | Ogunnaike & Ray 1994 |
| (structural) | Identity and composition laws over the ProcessCategory | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `ProcessConcept`, `ProcessCategory`, `PhysicalUnit` quality, thermodynamic axioms
- `engine.rs` -- simulation/control engine used by tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
