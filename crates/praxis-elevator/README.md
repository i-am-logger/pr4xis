# praxis-elevator

[![crates.io](https://img.shields.io/crates/v/praxis-elevator.svg)](https://crates.io/crates/praxis-elevator)
[![docs.rs](https://img.shields.io/docsrs/praxis-elevator)](https://docs.rs/praxis-elevator)

Elevator dispatch rules enforcement via ontology -- optimal multi-elevator scheduling with safety guarantees.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

Models a building with multiple elevators, enforcing physical and safety constraints at the type level. Doors must be closed before moving, capacity limits are checked before boarding, and direction commitments prevent inefficient backtracking. Dispatch strategies (nearest-car and zone-based) assign requests to elevators while respecting all invariants.

## Key Types

| Type | Description |
|---|---|
| `Building` | A building with N floors and M elevators, manages requests and simulation |
| `Elevator` | A single elevator car with floor, direction, door state, and load tracking |
| `Request` | A hall call: origin floor, destination, and passenger weight |
| `Direction` | Elevator travel direction: Up, Down, or Idle |
| `DoorState` | Door position: Open or Closed |
| `Dispatch` | Assigns pending requests to elevators |
| `DispatchStrategy` | Algorithm selection: NearestCar or Zone |

## Example

```rust
use praxis_elevator::{Building, Request};

let mut building = Building::new(10, 2, 1000); // 10 floors, 2 elevators, 1000kg capacity

// Submit requests -- validated against floor bounds
building.request(Request::new(0, 5, 80)).unwrap();
building.request(Request::new(3, 0, 75)).unwrap();

// Run simulation to completion
let events = building.run_to_completion(100);

// All elevators return to idle with doors closed
assert!(building.elevators.iter().all(|e| e.is_idle()));
```

## License

CC BY-NC-SA 4.0
