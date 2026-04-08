# praxis-chess

[![crates.io](https://img.shields.io/crates/v/praxis-chess.svg)](https://crates.io/crates/praxis-chess)
[![docs.rs](https://img.shields.io/docsrs/praxis-chess)](https://docs.rs/praxis-chess)

Chess rules enforcement via ontology -- guarantees no illegal moves.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

A complete chess implementation built on `praxis-engine`. The board is a `Situation`, moves are `Actions`, and all chess rules -- piece movement, check/checkmate detection, castling, en passant, promotion, the 50-move rule -- are enforced as `Preconditions`. Every attempted move is validated before application and recorded in a trace, so illegal moves are structurally impossible. Includes PGN parsing for replaying recorded games.

## Key Types

| Type | Description |
|---|---|
| `Board` | Full board state: pieces, castling rights, en passant, clocks |
| `Square` | A board position (file 0-7, rank 0-7), implements `Entity` with 64 variants |
| `Piece` | A chess piece with `PieceKind` and `Color` |
| `ChessAction` | A move from one square to another |
| `ChessEngine` | `Engine<ChessAction>` with all chess preconditions wired in |
| `ChessMove` | A move with full context: captures, castling, check, promotion |
| `new_game()` | Constructor returning a `ChessEngine` from the starting position |

## Example

```rust
use praxis_chess::{new_game, ChessAction, Square};

let game = new_game();
// 1. e4
let game = game.try_next(ChessAction::new(
    Square::new(4, 1), Square::new(4, 3)
)).expect("e4 is legal");
// 1... e5
let game = game.try_next(ChessAction::new(
    Square::new(4, 6), Square::new(4, 4)
)).expect("e5 is legal");

println!("{}", game.trace().dump());
```

## License

CC BY-NC-SA 4.0
