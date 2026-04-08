# praxis-engine

[![crates.io](https://img.shields.io/crates/v/praxis-engine.svg)](https://crates.io/crates/praxis-engine)
[![docs.rs](https://img.shields.io/docsrs/praxis-engine)](https://docs.rs/praxis-engine)

Runtime enforcement engine -- situations, actions, preconditions, and traces.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

The engine applies actions to immutable situations with precondition checking. Every action must pass all preconditions before it is applied; violations are captured with full diagnostics. The engine maintains back/forward history for undo/redo and records a complete trace of every attempted action, making it suitable for any domain where illegal state transitions must be prevented and audited.

## Key Types

| Type | Description |
|---|---|
| `Situation` | An immutable snapshot of the world (implement for your domain state) |
| `Action` | A proposed state transition that transforms one situation into another |
| `Precondition` | A rule checked before an action can be applied |
| `PreconditionResult` | `Satisfied` or `Violated` with rule name, reason, and context |
| `Engine` | The core runner: validates actions, applies them, maintains history |
| `Trace` | Full log of all attempted actions with precondition results |
| `TraceEntry` | A single step: situation before/after, action, results, success flag |

## Example

```rust
use praxis_engine::{Engine, Situation, Action, Precondition, PreconditionResult};

// 1. Define your Situation (domain state)
// 2. Define your Action (state transitions)
// 3. Define Preconditions (enforcement rules)
// 4. Wire them together:
let engine = Engine::new(initial_state, preconditions, apply_fn);
let engine = engine.try_next(some_action)?;   // validates + applies
let engine = engine.back().unwrap();           // undo
let engine = engine.forward().unwrap();        // redo
println!("{}", engine.trace().dump());         // full audit log
```

## License

CC BY-NC-SA 4.0
