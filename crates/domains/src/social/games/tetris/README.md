# Tetris -- Tetromino Piece Ontology

Models the seven standard Tetris tetrominoes with qualities that expose their cell count and distinct rotation count. The board, piece, and engine modules realize a runnable Tetris game over this entity set.

Key references:
- Pajitnov 1984 (original *Tetris*)
- Tetris Guideline: standard tetromino shapes (I, O, T, S, Z, J, L)

## Entities

| Category | Entities |
|---|---|
| Tetrominoes (7) | I, O, T, S, Z, J, L |

## Qualities

| Quality | Type | Description |
|---|---|---|
| CellCount | usize | Number of cells in the piece (always 4 for tetrominoes) |
| RotationCount | usize | Number of distinct rotations modulo translation: O=1, I/S/Z=2, T/J/L=4 |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Entity-set coverage and quality totality | auto-generated |

Domain invariants verified by tests: all seven tetrominoes have exactly four cells (Tetris Guideline); O has one rotation, I has two, T has four.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `CellCount` and `RotationCount` qualities, tests
- `piece.rs` -- `PieceKind` entity, `Piece`, `Rotation`
- `board.rs` -- `Board` grid with collision and line-clear logic
- `game.rs` -- `Game`, `GameAction`, `ActionResult` — session state
- `engine.rs` -- runtime Tetris engine / action loop
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
