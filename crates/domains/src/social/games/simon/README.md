# Simon -- Simon Says Button Ontology

Models the four-button memory game Simon as an ontology over the four button colors with a canonical ordering. The game, input, and engine modules realize the sequence-memory loop on top of the entity set.

Key references:
- Baer & Morrison 1978 (Milton Bradley *Simon*, original design)

## Entities

| Category | Entities |
|---|---|
| Buttons (4) | Red, Blue, Green, Yellow |

## Qualities

| Quality | Type | Description |
|---|---|---|
| ButtonIndex | usize | Canonical ordinal for each button: Red=0, Blue=1, Green=2, Yellow=3 |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Entity-set coverage and quality totality | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `ButtonIndex` quality and tests
- `color.rs` -- `SimonColor` entity (Red, Blue, Green, Yellow) and seed-based selection
- `input.rs` -- `Input` rich type for user button presses
- `game.rs` -- `Game`, `GameState`, `RoundResult` — round-by-round state
- `engine.rs` -- runtime Simon engine / action loop
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
