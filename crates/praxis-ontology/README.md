# praxis-ontology

[![crates.io](https://img.shields.io/crates/v/praxis-ontology.svg)](https://crates.io/crates/praxis-ontology)
[![docs.rs](https://img.shields.io/docsrs/praxis-ontology)](https://docs.rs/praxis-ontology)

Define, validate, and enforce domain ontologies -- what exists, how things relate, and what rules govern them.

Part of the [Praxis](https://github.com/i-am-logger/praxis) framework.

## Overview

An ontology ties together a category (individuals + relations), qualities that inhere in individuals, and domain-specific axioms. When you implement the `Ontology` trait, calling `validate()` checks all category laws (identity, associativity, closure) plus every axiom you define. If it passes, your domain model is mathematically sound. This crate also re-exports `praxis-logic` composition types so consumers can build complex rules with `AllOf`, `AnyOf`, `Implies`, etc.

## Key Types

| Type | Description |
|---|---|
| `Ontology` | Trait binding a category + qualities + axioms with self-validation |
| `Quality` | An attribute or capability that inheres in an individual (from BFO/DOLCE) |
| `Axiom` | A domain-specific rule the ontology must satisfy |

## Example

```rust
use praxis_ontology::{Ontology, Quality, Axiom};
use praxis_category::{Category, Entity};

// Define your domain category, then:
struct MyOntology;
impl Ontology for MyOntology {
    type Cat = MyCategory;
    type Qual = MyQuality;
    fn axioms() -> Vec<Box<dyn Axiom<MyCategory>>> {
        vec![]  // add domain-specific constraints
    }
}

// Validates category laws + all axioms in one call
MyOntology::validate().expect("ontology is sound");
```

## License

CC BY-NC-SA 4.0
