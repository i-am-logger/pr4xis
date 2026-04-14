# 03 — Your First Ontology

The third step in the [Get started](get-started.md) tutorial sequence. After this page you will have written a minimal `define_ontology!` block, run its tests, and seen the categorical machinery validate your definition.

This page assumes you have completed [01 — Install](01-install.md) and [02 — First Query](02-first-query.md).

## What we'll build

A toy ontology of musical instrument families. Three concepts (string, wind, percussion), one taxonomy (each is a kind of instrument), one functor (each can produce sound). Small enough to read in one sitting; complete enough to show every part of the macro pattern.

For a real-world authoring workflow against a published source paper, see [Build an ontology from a paper](../use/build-ontology-from-paper.md). This page is the toy version that gets you familiar with the macro syntax.

## Step 1: Make a place for it

Inside the workspace, create a new directory and module:

```bash
mkdir -p crates/domains/src/social/music_intro
touch crates/domains/src/social/music_intro/{mod.rs,ontology.rs,tests.rs}
```

Add the module to `crates/domains/src/social/mod.rs`:

```rust
pub mod music_intro;
```

## Step 2: Write the ontology

In `crates/domains/src/social/music_intro/ontology.rs`:

```rust
use pr4xis::define_ontology;
use pr4xis::category::entity::Entity;
use pr4xis_derive::Entity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Entity)]
pub enum InstrumentEntity {
    Instrument,
    String,
    Wind,
    Percussion,
}

define_ontology! {
    /// A toy ontology of musical instrument families.
    /// Source: tutorial example, not a published paper.
    pub InstrumentOntology {
        entity: InstrumentEntity,
        category: InstrumentCategory,

        taxonomy: InstrumentTaxonomy [
            (String, Instrument),
            (Wind, Instrument),
            (Percussion, Instrument),
        ],
    }
}
```

The `define_ontology!` macro expands this into:

- An `InstrumentCategory` struct that implements the `Category` trait
- An `InstrumentTaxonomy` struct that implements `TaxonomyDef`, encoding the three is-a relations
- The structural axioms (`NoCycles`, `Antisymmetric`) for the taxonomy
- An `Ontology` impl that bundles the category and the reasoning systems
- An `OntologyMeta` for trace attribution
- A `meta()` function exposing the metadata

## Step 3: Write a test

In `crates/domains/src/social/music_intro/tests.rs`:

```rust
use super::ontology::*;
use pr4xis::category::validate::check_category_laws;
use pr4xis::ontology::Ontology;

#[test]
fn category_laws_hold() {
    check_category_laws::<InstrumentCategory>().unwrap();
}

#[test]
fn ontology_validates() {
    InstrumentOntology::validate().unwrap();
}

#[test]
fn string_is_an_instrument() {
    use pr4xis::ontology::reasoning::taxonomy::TaxonomyDef;
    let taxonomy = InstrumentTaxonomy;
    assert!(taxonomy.is_a(InstrumentEntity::String, InstrumentEntity::Instrument));
}
```

In `mod.rs`:

```rust
pub mod ontology;

#[cfg(test)]
mod tests;

pub use ontology::*;
```

## Step 4: Run the tests

```bash
cargo test -p pr4xis-domains music_intro
```

You should see three passing tests:

```
test social::music_intro::tests::category_laws_hold ... ok
test social::music_intro::tests::ontology_validates ... ok
test social::music_intro::tests::string_is_an_instrument ... ok
```

If they pass, your category obeys the identity and associativity laws, your taxonomy is a valid DAG (no cycles, antisymmetric), and your encoding of "string is an instrument" is verified by the reasoning system.

If a test fails, the error message names the specific law or axiom that failed. Fix the encoding and re-run — usually the issue is a missing `is-a` row or a cyclic definition.

## What just happened

You wrote four lines of taxonomy data and got back:

- A category with verified composition and identity laws
- A taxonomy with verified `NoCycles` and `Antisymmetric` axioms
- A type-checked entity enum
- A test suite that re-runs every law on every commit

That's the value of `define_ontology!` — most of the categorical machinery is auto-generated from the declarative spec, and the parts that aren't are auto-tested.

## What you can do next

- **Add a mereology.** What are the parts of a string instrument? (body, neck, strings, tuning pegs.) Add a `mereology:` block to the macro. The `WeakSupplementation` axiom will be checked automatically.
- **Add a quality.** What measurable property does an instrument have? (pitch range in Hz.) Define a `Quality` struct and attach it to the entity.
- **Compose with another ontology.** Pr4xis already has a music ontology at `crates/domains/src/natural/music/`. Write a functor `InstrumentToMusic` that maps your three instrument families onto the music ontology's pitch/scale/chord concepts. Run `check_functor_laws` to verify.
- **Add a domain axiom.** "A string instrument has at least one string." Implement an `Axiom` impl that checks this constraint and add it to `domain_axioms()`.

For each of these, see the matching how-to guide:

- [Compose via functor](../use/compose-via-functor.md)
- [Write axioms](../use/write-axioms.md)
- [Build an ontology from a paper](../use/build-ontology-from-paper.md) — the real-world authoring workflow

## What you have now

- A complete `define_ontology!` block in the workspace
- A test suite that exercises the category laws, the structural axioms, and a worked example query
- A starting point for adding your own real-world ontology — the macro pattern is the same, just with more concepts and more reasoning systems

## Where to go from here

- [Concepts](../understand/concepts.md) — what the categorical machinery you just plugged into actually means
- [Architecture](../understand/architecture.md) — the five-layer stack and how `define_ontology!` fits into it
- [Build an ontology from a paper](../use/build-ontology-from-paper.md) — the next-level authoring workflow
- [Domain catalog](../reference/domain-catalog.md) — the 106 existing ontologies you can use as patterns

---

- **Document date:** 2026-04-14
