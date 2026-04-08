# praxis-tetris

[![crates.io](https://img.shields.io/crates/v/praxis-tetris.svg)](https://crates.io/crates/praxis-tetris)
[![docs.rs](https://img.shields.io/docsrs/praxis-tetris)](https://docs.rs/praxis-tetris)

Tetris rules enforcement via ontology -- spatial constraints, gravity, rotation, line clears.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Models a complete Tetris game on a 10x20 board where every player action is validated against spatial constraints. Pieces cannot overlap, move out of bounds, or defy gravity. The ontology enforces collision detection, wall kicks on rotation, line clear mechanics, and game-over conditions when a new piece cannot spawn. Scoring follows standard Tetris rules with level multipliers.

## Key Types

| Type | Description |
|---|---|
| `Game` | Full game state: board, current piece, score, level, game-over flag |
| `GameAction` | Player actions: MoveLeft, MoveRight, MoveDown, HardDrop, RotateCW, RotateCCW |
| `ActionResult` | Enforcement response: Moved, Locked (with lines cleared), Blocked, or GameOver |
| `Board` | A 10x20 grid of filled/empty cells with collision detection and line clearing |
| `Piece` | A tetromino with position and rotation on the board |
| `PieceKind` | The 7 standard tetrominoes: I, O, T, S, Z, J, L |
| `Rotation` | Rotation state: R0, R90, R180, R270 |

## Example

```rust
use praxis_tetris::{Game, GameAction, ActionResult};

let mut game = Game::new(42); // seeded for reproducibility

// Move piece right, then hard drop
assert_eq!(game.act(GameAction::MoveRight), ActionResult::Moved);
if let ActionResult::Locked { lines_cleared } = game.act(GameAction::HardDrop) {
    println!("Piece locked, cleared {} lines", lines_cleared);
}

// Gravity tick -- equivalent to MoveDown
let result = game.tick();
assert!(!game.game_over);
```

## License

CC BY-NC-SA 4.0
