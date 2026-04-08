# praxis-puzzles

[![crates.io](https://img.shields.io/crates/v/praxis-puzzles.svg)](https://crates.io/crates/praxis-puzzles)
[![docs.rs](https://img.shields.io/docsrs/praxis-puzzles)](https://docs.rs/praxis-puzzles)

Classic logic puzzles with rule enforcement -- river crossing, Tower of Hanoi, water jugs, Monty Hall, and more.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Implements 11 classic puzzles and game-theory problems as engine-driven state machines. Each puzzle defines a `Situation`, typed `Action`s, and `Precondition`s that enforce the puzzle's rules -- illegal moves are rejected, not silently applied. Every puzzle can be played step-by-step with full trace history, undo/redo, and property-based test coverage.

## Puzzles

| Module | Puzzle | Rules Enforced |
|---|---|---|
| `hanoi` | Tower of Hanoi | No larger disk on smaller |
| `river_crossing` | Farmer, Wolf, Goat, Cabbage | No unsafe bank states |
| `water_jugs` | Measure target volume | No-op actions rejected |
| `sudoku` | 9x9 Sudoku | Row, column, and 3x3 box uniqueness |
| `n_queens` | N-Queens | No row, column, or diagonal attacks |
| `missionaries` | Missionaries and Cannibals | Cannibals never outnumber missionaries |
| `monty_hall` | Monty Hall | Host reveals goat, phase ordering |
| `knights_tour` | Knight's Tour | Valid L-moves, no revisits |
| `konigsberg` | Bridges of Konigsberg | Adjacent, untraversed bridges only |
| `byzantine` | Byzantine Generals | Loyal generals relay truthfully, phase protocol |
| `prisoner` | Prisoner's Dilemma | Turn ordering (A then B) |

## Example

```rust
use praxis_puzzles::hanoi::{self, Move};

let engine = hanoi::new_puzzle(3);
let engine = engine
    .next(Move::new(0, 2)).unwrap()
    .next(Move::new(0, 1)).unwrap()
    .next(Move::new(2, 1)).unwrap()
    .next(Move::new(0, 2)).unwrap()
    .next(Move::new(1, 0)).unwrap()
    .next(Move::new(1, 2)).unwrap()
    .next(Move::new(0, 2)).unwrap();
assert!(engine.is_terminal());  // solved in 7 moves (2^3 - 1)
```

## License

CC BY-NC-SA 4.0
