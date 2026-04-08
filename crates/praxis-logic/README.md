# praxis-logic

[![crates.io](https://img.shields.io/crates/v/praxis-logic.svg)](https://crates.io/crates/praxis-logic)
[![docs.rs](https://img.shields.io/docsrs/praxis-logic)](https://docs.rs/praxis-logic)

Formal logic: propositional connectives, composable propositions, truth tables, and proof rules.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

This crate provides two layers. The propositional layer models logical connectives (AND, OR, NOT, IMPLIES, IFF, XOR, NAND, NOR) as entities and proves classical laws -- De Morgan's, modus ponens, contrapositive, NAND universality -- by exhaustive truth table evaluation. The composition layer provides a generic `Proposition` trait for building complex boolean enforcement rules with `AllOf`, `AnyOf`, `Not`, `Implies`, `Compare`, and `Threshold`, which any domain engine can use.

## Key Types

| Type | Description |
|---|---|
| `Connective` | The 8 logical connectives as an `Entity` enum |
| `Proposition` | Trait for evaluable boolean rules with context |
| `Evaluation` | Result of a proposition: `Satisfied` or `Violated` with reason |
| `AllOf` | Logical AND over a collection of propositions |
| `AnyOf` | Logical OR over a collection of propositions |
| `Not` | Logical negation of a proposition |
| `Implies` | If A then B -- vacuously true when A is false |
| `Measurable` | A value extractable from context for comparison |
| `Threshold` | Compare a measurable against a constant (e.g., `x > 10`) |

## Example

```rust
use praxis_logic::{Proposition, Evaluation, AllOf, Implies};
use praxis_logic::propositional::{Connective, de_morgan_and, modus_ponens};

// Prove classical laws by exhaustive truth table
assert!(de_morgan_and());  // !(A && B) == (!A || !B)
assert!(modus_ponens());   // (A && (A -> B)) -> B

// Connectives are entities
assert!(Connective::And.eval(true, false) == false);
assert!(Connective::Nand.eval(true, false) == true);
```

## License

CC BY-NC-SA 4.0
