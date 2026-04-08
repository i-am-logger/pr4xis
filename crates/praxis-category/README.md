# praxis-category

[![crates.io](https://img.shields.io/crates/v/praxis-category.svg)](https://crates.io/crates/praxis-category)
[![docs.rs](https://img.shields.io/docsrs/praxis-category)](https://docs.rs/praxis-category)

Category theory primitives for Rust -- categories, functors, natural transformations, with property-based validation.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

This crate provides the mathematical foundation for the entire Praxis framework. It models categories as collections of objects (entities) and morphisms (relationships) where composition and identity laws hold. All laws -- identity, associativity, closure -- are verified via exhaustive and property-based testing, so if your category compiles and validates, it is mathematically sound.

## Key Types

| Type | Description |
|---|---|
| `Entity` | A thing that exists -- must be finite and enumerable via `variants()` |
| `Relationship` | A directed connection between entities (source and target) |
| `Category` | Objects + morphisms with identity and composition |
| `Morphism` | A relationship bound to its category, enabling `.then()` chaining |
| `Functor` | A structure-preserving map between two categories |
| `NaturalTransformation` | A morphism between two functors (component at each object) |
| `Axiom` | A domain-specific constraint beyond basic category laws |
| `NoDeadStates` | Built-in axiom: every object has at least one outgoing morphism |
| `FullyConnected` | Built-in axiom: every object is reachable from every other |

## Example

```rust
use praxis_category::{Entity, Relationship, Category};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Light { Red, Green, Yellow }

impl Entity for Light {
    fn variants() -> Vec<Self> {
        vec![Light::Red, Light::Green, Light::Yellow]
    }
}

// Define transitions, implement Relationship + Category,
// then validate laws with praxis_category::validate
```

## License

CC BY-NC-SA 4.0
