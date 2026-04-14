# Traffic -- Intersection Conflict Ontology

Models a four-way intersection as a category whose objects are the cardinal traffic directions and whose relation is `Conflict`. The north-south and east-west pairs are orthogonal: NS directions do not conflict with each other, EW directions do not conflict with each other, but NS conflicts with EW. The ontology is the foundation for signal-phase assignment.

Key references:
- Highway Capacity Manual (TRB) — conflict-point analysis for signalized intersections
- Webster 1958: *Traffic Signal Settings* (classical signal timing)

## Entities (4)

| Category | Entities |
|---|---|
| Traffic directions (4) | North, South, East, West |

## Category

Dense category `TrafficCategory` generated via `define_dense_category!`. Objects: `TrafficDirection`. Relation: `Conflict`. All category laws (identity, composition) verified at test time.

## Qualities

| Quality | Type | Description |
|---|---|---|
| ConflictsWithNorth | () | Present for directions orthogonal to North (East, West); absent for North and South |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| OrthogonalConflicts | NS and EW are orthogonal conflict pairs; same-axis directions do not conflict | Highway Capacity Manual |
| (structural) | Identity and composition laws over the TrafficCategory | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `TrafficDirection` entity, `TrafficCategory`, `ConflictsWithNorth` quality, `OrthogonalConflicts` axiom
- `intersection.rs` -- `Intersection`, `IntersectionResult` runtime types
- `signal.rs` -- `Signal`, `SignalState`, `SignalAction` for phase control
- `engine.rs` -- simulation engine used by tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
