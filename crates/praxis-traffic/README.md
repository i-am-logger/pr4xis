# praxis-traffic

[![crates.io](https://img.shields.io/crates/v/praxis-traffic.svg)](https://crates.io/crates/praxis-traffic)
[![docs.rs](https://img.shields.io/docsrs/praxis-traffic)](https://docs.rs/praxis-traffic)

Traffic intersection signal control with conflict-safety enforcement via ontology.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Models traffic intersections where signals must obey timing constraints and conflict rules. The ontology enforces that conflicting directions (e.g., north-south vs east-west) can never both show green simultaneously. Signals follow a strict state machine with minimum durations, and the intersection rejects any advance that would create an unsafe conflict.

## Key Types

| Type | Description |
|---|---|
| `Intersection` | An intersection with multiple signals and a conflict matrix |
| `IntersectionResult` | Validation result: Safe or Conflict with the two conflicting directions |
| `Signal` | A single traffic signal with state and timing enforcement |
| `SignalState` | Signal phases: Red, Yellow, Green, LeftArrow, BlinkingYellow, BlinkingRed, Off |
| `SignalAction` | Actions on a signal: Advance, Malfunction, Recover, TurnOff, TurnOn |

## Example

```rust
use praxis_traffic::{Intersection, SignalState};

// Standard 4-way intersection: green=30, yellow=5, red=35 ticks
let mut intersection = Intersection::four_way(30, 5, 35);
assert!(intersection.is_safe());

// Advance north signal through its red duration
for _ in 0..5 {
    intersection = intersection.tick();
}
let intersection = intersection.advance_signal(0).unwrap(); // north goes green

// Attempting to also green an east signal is rejected
assert!(intersection.advance_signal(2).is_err());
```

## License

CC BY-NC-SA 4.0
