# praxis-calculator

[![crates.io](https://img.shields.io/crates/v/praxis-calculator.svg)](https://crates.io/crates/praxis-calculator)
[![docs.rs](https://img.shields.io/docsrs/praxis-calculator)](https://docs.rs/praxis-calculator)

Scientific calculator with domain enforcement -- constraints, order of operations, unit safety.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

A full scientific calculator built on `praxis-engine`. The calculator state is a `Situation`, operations are `Actions`, and mathematical domain constraints (division by zero, sqrt of negatives, log of non-positive) are enforced as `Preconditions`. Values are represented as exact rationals when possible, falling back to floats. A number domain checker classifies values into the hierarchy N, Z, Q, R, C and ensures operations stay within supported domains. Includes expression trees with algebraic simplification, unit conversions, bitwise operations, and physical constants.

## Key Types

| Type | Description |
|---|---|
| `Calculator` | Stateful calculator with display, memory, angle mode, history |
| `Value` | Exact rational or float with overflow/underflow checking |
| `CalcAction` | Enter, unary op, binary op, memory, clear, angle mode |
| `CalcEngine` | `Engine<CalcAction>` with domain and number hierarchy checks |
| `Expr` | Expression tree with `eval()` and algebraic `simplify()` |
| `BinaryOp` / `UnaryOp` | All supported mathematical operations |
| `Unit` / `UnitCategory` | Physical units with type-safe conversion |
| `Complex` | Complex number support |
| `new_calculator()` | Constructor returning a `CalcEngine` ready to use |

## Example

```rust
use praxis_calculator::{new_calculator, CalcAction, Value, BinaryOp};

let calc = new_calculator();
let calc = calc.try_next(CalcAction::Enter(Value::int(10))).unwrap();
let calc = calc.try_next(CalcAction::Binary(BinaryOp::Add, Value::int(5))).unwrap();
assert_eq!(calc.situation().display, Value::int(15));

// Domain violations are caught before they happen
let calc = calc.try_next(CalcAction::Binary(BinaryOp::Divide, Value::int(0)));
assert!(calc.is_err()); // "division by zero"
```

## License

CC BY-NC-SA 4.0
