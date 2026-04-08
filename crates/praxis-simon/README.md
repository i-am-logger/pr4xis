# praxis-simon

[![crates.io](https://img.shields.io/crates/v/praxis-simon.svg)](https://crates.io/crates/praxis-simon)
[![docs.rs](https://img.shields.io/docsrs/praxis-simon)](https://docs.rs/praxis-simon)

Simon Says sequence enforcement -- memory game rules guaranteed by ontology.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Models the Simon Says memory game where a growing color sequence must be reproduced exactly. The ontology enforces a strict state machine: the game shows a sequence, the player inputs colors one at a time, and any wrong input immediately ends the game. State transitions (Showing, Inputting, RoundComplete, GameOver) are enforced -- you cannot input during the showing phase or advance rounds before completing the current one.

## Key Types

| Type | Description |
|---|---|
| `Game` | The game state: sequence, current round, and state machine |
| `GameState` | Phase of the game: Showing, Inputting, RoundComplete, or GameOver |
| `RoundResult` | Result of a single input: Correct, RoundComplete, Wrong, or InvalidState |
| `SimonColor` | The four buttons: Red, Blue, Green, Yellow |
| `Input` | A player input: a color at a specific sequence position |

## Example

```rust
use praxis_simon::{Game, GameState, RoundResult, SimonColor};

let mut game = Game::new(42); // seeded for reproducibility

// Game starts in Showing state -- transition to Inputting
game.start_input().unwrap();

// Play back the correct sequence
let correct = game.sequence().to_vec();
for &color in &correct {
    match game.input(color) {
        RoundResult::Correct { remaining } => println!("{} inputs left", remaining),
        RoundResult::RoundComplete { round } => println!("Round {} complete!", round),
        _ => {}
    }
}

// Advance to next round (sequence grows by one)
game.next_round().unwrap();
```

## License

CC BY-NC-SA 4.0
