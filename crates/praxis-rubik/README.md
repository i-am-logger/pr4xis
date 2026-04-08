# praxis-rubik

[![crates.io](https://img.shields.io/crates/v/praxis-rubik.svg)](https://crates.io/crates/praxis-rubik)
[![docs.rs](https://img.shields.io/docsrs/praxis-rubik)](https://docs.rs/praxis-rubik)

Rubik's cube rules enforcement via ontology -- guarantees only valid moves, group theory.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Models a 3x3 Rubik's cube where the only way to change state is through the 18 valid face rotations (6 faces x 3 rotation types). The ontology layer enforces structural invariants -- centers never move and each color always has exactly 9 stickers -- making illegal cube states unrepresentable. The category theory formalization treats faces as objects and rotations as morphisms.

## Key Types

| Type | Description |
|---|---|
| `Cube` | A 3x3 Rubik's cube with 6 faces of 9 stickers each |
| `Face` | The 6 cube faces: U, D, F, B, L, R |
| `Color` | The 6 sticker colors: White, Yellow, Green, Blue, Red, Orange |
| `Move` | One of 18 valid moves: CW, CCW, and 180 for each face |
| `RubikCategory` | Category where faces are objects and rotations are morphisms |
| `CentersFixed` | Axiom: center stickers must match their face color |
| `NinePerColor` | Axiom: each color has exactly 9 stickers |

## Example

```rust
use praxis_rubik::{Cube, Move};

let cube = Cube::solved();
assert!(cube.is_solved());

// Apply a sequence of moves
let scrambled = cube.apply_sequence(&[Move::R, Move::U, Move::Ri, Move::Ui]);

// Every move preserves the 9-per-color invariant
assert_eq!(scrambled.color_counts(), [9; 6]);

// Inverse moves undo each other
let restored = scrambled.apply_sequence(&[Move::U, Move::R, Move::Ui, Move::Ri]);
assert!(restored.is_solved());
```

## License

CC BY-NC-SA 4.0
