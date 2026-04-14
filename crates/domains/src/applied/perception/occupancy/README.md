# Occupancy -- Bayesian Occupancy Grid Ontology

Models the cell states of a Bayesian occupancy grid (Free, Occupied, Unknown) and their transitions under sensor evidence. The category is fully connected because a cell can transition from any state to any other on a new observation (log-odds Bayesian update). Each state carries its posterior probability range as a quality.

Key references:
- Elfes 1989: *Using Occupancy Grids for Mobile Robot Perception and Navigation*
- Thrun, Burgard & Fox 2005: *Probabilistic Robotics* (log-odds form of occupancy mapping)

## Entities (3)

| Category | Entities |
|---|---|
| Cell states (3) | Free, Occupied, Unknown |

## Category

`OccupancyOntology for OccupancyCategory` via `define_ontology!`, relation `CellTransition`, fully connected. Bayesian evidence can drive any cell from any state to any other.

## Qualities

| Quality | Type | Description |
|---|---|---|
| OccupancyProbability | (f64, f64) | Posterior probability range: Free=(0.0,0.5), Occupied=(0.5,1.0), Unknown=(0.5,0.5) |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| ProbabilityBounded | Occupancy probabilities lie in [0, 1] | Elfes 1989 |
| LogOddsUpdateDeterministic | Log-odds Bayesian update is a deterministic function of prior and observation | Thrun et al. 2005 |
| (structural) | Identity and composition laws over the OccupancyCategory | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `CellState`, `OccupancyCategory`, `OccupancyProbability` quality, Bayesian axioms
- `engine.rs` -- occupancy-grid update engine used by tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
