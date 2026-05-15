# 03 — Your First Ontology

The third step in the [Get started](get-started.md) tutorial sequence. After this page you will have written a minimal `ontology!` block, run its tests, and seen the categorical machinery validate your definition.

This page assumes you have completed [01 — Install](01-install.md) and [02 — First Query](02-first-query.md).

## What we'll build

A toy ontology of musical instrument families. Three concepts (string, wind, percussion) that subsume an `Instrument` parent. Small enough to read in one sitting; complete enough to show every part of the macro pattern.

For a real-world authoring workflow against a published source paper, see [Build an ontology from a paper](../use/build-ontology-from-paper.md). This page is the toy version that gets you familiar with the macro syntax.

## Step 1: Make a place for it

Inside the workspace, create a new directory and module:

```bash
mkdir -p crates/domains/src/social/music_intro
touch crates/domains/src/social/music_intro/{mod.rs,ontology.rs,tests.rs}
```

Add the module to `crates/domains/src/social/mod.rs`:

```text
pub mod music_intro;
```

## Step 2: Write the ontology

In `crates/domains/src/social/music_intro/ontology.rs`:

```text
pr4xis::ontology! {
    name: "MusicalInstruments",
    source: "tutorial example, not a published paper",

    concepts: [Instrument, String, Wind, Percussion],

    labels: {
        Instrument: ("en", "Instrument", "A musical instrument family."),
        String: ("en", "String", "Instruments that produce sound via vibrating strings."),
        Wind: ("en", "Wind", "Instruments that produce sound via vibrating air columns."),
        Percussion: ("en", "Percussion", "Instruments that produce sound by being struck."),
    },

    is_a: [
        (String, Instrument),
        (Wind, Instrument),
        (Percussion, Instrument),
    ],
}
```

The `ontology!` proc macro (`pr4xis::ontology`, re-exported from `pr4xis-derive`) expands this into:

- A `MusicalInstrumentsConcept` enum implementing `Concept` (Guarino 2009 — closed-world named objects)
- A `MusicalInstrumentsCategory` struct implementing `Category` (Mac Lane 1971 Ch. I §1)
- A `MusicalInstrumentsRelation` struct + `MusicalInstrumentsRelationKind` enum implementing `Arrow` — every is-a row becomes a `Subsumption`-kinded morphism (Awodey 2010 §1.3)
- An `Ontology` impl whose `fn axioms()` returns the structural axioms for every kind in use — for `Subsumption` the catalog (OBO-RO; Smith et al. 2005) emits `NoCyclesOnKind` (Tarski 1941) + `AntisymmetricOnKind` automatically
- A `fn meta() -> Provenance` carrying the `name:` + `source:` for trace attribution

Everything is type-checked: a typo in a concept name fails at compile time, not at test time.

## Step 3: Write a test

In `crates/domains/src/social/music_intro/tests.rs`:

```text
use super::ontology::*;
use pr4xis::category::laws::assert_category_laws;
use pr4xis::category::{Arrow, Category, Concept};
use pr4xis::ontology::Ontology;

#[test]
fn category_laws() {
    assert_category_laws::<MusicalInstrumentsCategory>();
}

#[test]
fn ontology_validates() {
    MusicalInstrumentsOntology::validate()
        .unwrap_or_else(|c| panic!("validation failed: {}", c.meta().description.as_str()));
}

#[test]
fn string_is_an_instrument() {
    let m = MusicalInstrumentsCategory::morphisms();
    assert!(m.iter().any(|r| r.source() == MusicalInstrumentsConcept::String
        && r.target() == MusicalInstrumentsConcept::Instrument
        && r.kind() == MusicalInstrumentsRelationKind::Subsumption));
}
```

In `mod.rs`:

```text
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

```text
test social::music_intro::tests::category_laws ... ok
test social::music_intro::tests::ontology_validates ... ok
test social::music_intro::tests::string_is_an_instrument ... ok
```

If they pass, your category obeys identity and associativity (Mac Lane 1971), your subsumption edges form a valid DAG (`NoCyclesOnKind` + `AntisymmetricOnKind` from the catalog), and your encoding of "string is an instrument" is queryable as a kinded morphism.

If a test fails, the returned `Counterexample` names the specific law or axiom that failed. Fix the encoding and re-run — usually the issue is a cycle in `is_a:` or a typo in a concept name.

## What just happened

You wrote three lines of taxonomy data and got back:

- A category with verified composition and identity laws
- Subsumption edges with verified `NoCycles` + `Antisymmetric` axioms inherited automatically from the structural-axioms catalog
- A type-checked concept enum
- A test suite that re-runs every law on every commit

That's the value of `ontology!` — most of the categorical machinery is auto-generated from the declarative spec, and the parts that aren't are auto-tested.

## What you can do next

- **Add parthood.** What are the parts of a string instrument? (body, neck, strings, tuning pegs.) Add a `has_a:` sugar clause to the macro. The catalog will attach `NoCyclesOnKind` for the `Parthood` kind automatically. If you need `WeakSupplementation` (Casati & Varzi 1999), add it as a hand-written domain axiom in your `Ontology::axioms()` impl.
- **Add a quality.** What measurable property does an instrument have? (pitch range in Hz.) Implement the `Quality` trait for a marker struct and wire it as `type Qual = …` in your `Ontology` impl.
- **Compose with another ontology.** Pr4xis already has a music ontology at `crates/domains/src/natural/music/`. Write a `Functor` from `MusicalInstrumentsCategory` to the music category. Run `check_functor_laws` to verify identity + composition preservation.
- **Add a domain axiom.** "A string instrument has at least one string." Implement `Axiom` (with `verify()` + `citation()`) and push it onto the vec returned by `Ontology::axioms()` alongside the catalog's structural axioms.

For each of these, see the matching how-to guide:

- [Compose via functor](../use/compose-via-functor.md)
- [Write axioms](../use/write-axioms.md)
- [Build an ontology from a paper](../use/build-ontology-from-paper.md) — the real-world authoring workflow

## What you have now

- A complete `ontology!` block in the workspace
- A test suite that exercises the category laws, the structural axioms from the catalog, and a worked example query
- A starting point for adding your own real-world ontology — the macro pattern is the same, just with more concepts and more kinds of edges

## Where to go from here

- [Concepts](../understand/concepts.md) — what the categorical machinery you just plugged into actually means
- [Architecture](../understand/architecture.md) — the five-layer stack and how `ontology!` fits into it
- [Build an ontology from a paper](../use/build-ontology-from-paper.md) — the next-level authoring workflow
- [Domain catalog](../reference/domain-catalog.md) — the existing ontologies you can use as patterns

---

- **Document date:** 2026-05-14
