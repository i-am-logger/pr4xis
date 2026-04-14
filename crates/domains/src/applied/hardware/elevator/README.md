# Elevator -- Floor-Travel Ontology

Models an elevator as a category whose objects are floors and whose morphisms are travel events between floors. Encodes the reachability law (every floor is reachable from every other) and the height-from-ground quality. The default building is ten floors; composition of travels is the usual `from/to` matching.

Key references:
- Mandel 1989: *Elevator Scheduling* — group-control and dispatch background
- Barney & Dos Santos 1985: *Elevator Traffic Analysis, Design and Control*

## Entities (1)

| Category | Entities |
|---|---|
| Floors (1 type, 10 variants) | Floor(0) .. Floor(9) |

## Category

Objects: `Floor`. Morphisms: `Travel { from, to }`. Identity: `Travel { from: f, to: f }`. Composition: `(a→b) ∘ (b→c) = (a→c)`. The morphism set is the full Cartesian square of floors (fully connected).

## Qualities

| Quality | Type | Description |
|---|---|---|
| HeightFromGround | usize | Distance between a floor and the ground (floor index) |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| FullyConnected | Every floor is reachable from every other floor | structural |
| (structural) | Identity and composition laws over the ElevatorCategory | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `Floor` entity, `Travel` morphism, `ElevatorCategory`, `HeightFromGround` quality, reachability check
- `elevator.rs` -- `Elevator`, `Direction`, `DoorState` runtime types
- `building.rs` -- `Building` aggregate holding elevators and requests
- `dispatch.rs` -- `Dispatch` / `DispatchStrategy` assignment logic
- `request.rs` -- `Request` — a user hall-call or cab-call
- `engine.rs` -- simulation engine used by tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
