# Build an Ontology from a Paper

This page is the contributor's authoring guide for adding a new ontology to pr4xis from a published source. Tracked in [#44](https://github.com/i-am-logger/pr4xis/issues/44).

## Before you start

Open one of the existing ontologies and read the source. The biology ontology at `crates/domains/src/natural/biomedical/biology/ontology.rs` is the canonical reference example — small enough to read in one sitting, complex enough to show all the patterns. Compare it to its source paper (Alberts et al., *Molecular Biology of the Cell*, 6th ed.) by reading both side by side. The mapping from "what the paper says" to "what the `define_ontology!` block contains" is the workflow you will follow.

## The workflow

### Step 1: Pick a source

Authoritative, citable, and finite. Examples:

- A textbook chapter (Alberts on cells, Sola on quaternions, Riehl on category theory)
- A standards document (W3C OWL spec, ECMA JSON spec, IEC color spec)
- A peer-reviewed paper (Lambek 1958 on pregroup grammars, Conant-Ashby 1970 on the Good Regulator Theorem)
- A foundational technical reference (Mac Lane's *Categories for the Working Mathematician*)

Avoid: blog posts, Wikipedia pages, AI summaries, your own notes. The point of pr4xis is that every axiom traces to a source you can put in a bibliography.

### Step 2: Extract the concepts

Read the source carefully and list the *named things* it talks about. These become the variants of your `Entity` enum. For the biology ontology, the source paper named: cell, tissue, organ, organism, stem cell, fibroblast, columnar epithelial cell, etc.

Two rules:

- **One name, one concept.** If the paper uses two names for the same thing, pick one and note the alias in a comment. If it uses one name for two things, that's a context-dependent concept and you'll need a `ContextDef` to disambiguate (see [Concepts](../understand/concepts.md#context)).
- **Concepts, not values.** "Cell" is a concept; "10 micrometers" is a value (a `Quality` attached to a concept, not its own entity).

### Step 3: Extract the relations

For each pair of concepts, ask: does the source say one is a *kind of* the other? a *part of* the other? a *cause of* the other? an *opposite of* the other? Each answer is a different reasoning system:

- **Taxonomy** (`is-a`): "a fibroblast is a kind of cell" → `(Fibroblast, Cell)` in the taxonomy
- **Mereology** (`part-of`): "a cell membrane is part of a cell" → `(CellMembrane, Cell)` in the mereology
- **Causation**: "stem cell division causes cell differentiation" → `(StemCellDivision, CellDifferentiation)` in the causation
- **Opposition**: "M1 macrophages are the functional opposite of M2 macrophages" → `(MacrophageM1, MacrophageM2)` in the opposition

The source is the authority. If the source says it, the relation goes in. If the source doesn't say it, the relation does not go in — even if it "feels obvious".

### Step 4: Write the `define_ontology!` block

The macro takes a declarative spec and emits the full implementation. Skeleton:

```rust
use pr4xis::define_ontology;

define_ontology! {
    /// One-paragraph abstract of what this ontology models, citing the source.
    /// Source: <Author, Title, Year, Edition>.
    pub MyOntology {
        entity: MyEntity,
        category: MyCategory,
        relation: MyRelation,

        taxonomy: MyTaxonomy [
            (ChildConcept, ParentConcept),
            // ... one row per is-a relation in the source
        ],

        mereology: MyMereology [
            (Part, Whole),
            // ... one row per part-of relation in the source
        ],

        // omit any reasoning system the source doesn't motivate
    }
}
```

Then define the `MyEntity` enum with `#[derive(Entity)]` listing every concept, and the `MyRelation` enum (or struct) listing every relation morphism. The macro generates the category, the reasoning systems, the structural axioms, the `OntologyMeta`, and the test scaffolding. Run `cargo test -p pr4xis-domains` to verify the laws hold.

### Step 5: Add the citation

Create a `citings.md` file alongside `ontology.rs` (per [#57](https://github.com/i-am-logger/pr4xis/issues/57), once that lands) listing every source the ontology stands on. For each source: full citation, DOI or URL, and one-line annotation of which concepts or axioms it grounds. The README of the ontology directory points at the citings file.

### Step 6: Add Quality types where the source quantifies

If the source paper says "neurons fire when the membrane potential exceeds −55 mV", the threshold is a `Quality` attached to the `Neuron` entity. Create a Quality type, attach it via the ontology's `qualities()` method, and write a unit test that uses the value.

### Step 7: Add domain axioms

Structural axioms (no cycles, antisymmetric is-a, weak supplementation) are emitted automatically by the macro. **Domain axioms** are the constraints that come from the source paper: "neurons cannot have more than one axon", "an enzyme catalyzes exactly one reaction class", "a chess king moves at most one square per turn". Implement each as an `Axiom` trait impl and add it to the ontology's `domain_axioms()` method.

### Step 8: Compose with other ontologies

If your new ontology shares concepts with an existing one, write a functor. Example: if you're adding a hematology ontology and pr4xis already has a biochemistry ontology, the cells in your hematology ontology need to map to the biochemistry's cellular processes. Create a `HematologyToBiochemistry` functor, implement `map_object` and `map_morphism`, and run `check_functor_laws`. If the laws pass, the composition is verified — your ontology can now be queried alongside biochemistry through the composed functor.

### Step 9: Run the full suite

```bash
cargo test --workspace
```

If anything that was passing is now failing, you have introduced a contradiction with an existing ontology. Either the new source disagrees with the old source (in which case both are documented and the disagreement is explicit), or your encoding has a bug. Diagnose and fix.

## What to skip

- **Don't encode "common sense" that isn't in the source.** Even if it feels obvious. The point of pr4xis is that every claim is sourceable. If you encode common sense, you will eventually contradict someone else's source.
- **Don't encode terminology debates.** If the field has multiple naming conventions, pick the one the source uses and document the others as aliases.
- **Don't encode contested facts.** If the source itself flags a claim as disputed, encode it as a `disputed: true` quality, not as a hard axiom. (This pattern is in flight — see existing ontologies for examples.)

## Where to get help

- **Look at existing ontologies first.** `crates/domains/src/natural/biomedical/biology/ontology.rs` is the reference. Pick another close to your domain and pattern-match.
- **The `define_ontology!` macro source** at `crates/pr4xis/src/ontology/macros.rs` documents every supported field with examples.
- **[Concepts](../understand/concepts.md)** explains what each reasoning system means, with worked examples.
- **[Architecture](../understand/architecture.md)** explains where the ontology fits in the larger stack.

## When you're done

Open a PR. The CI will run `cargo test --workspace`, check formatting, run clippy, and verify the WASM build. If all pass, your ontology is in.

## Related

- [Compose via functor](compose-via-functor.md) — how to write a cross-domain functor
- [Write axioms](write-axioms.md) — how to write a domain axiom that the engine enforces
- [Concepts](../understand/concepts.md) — the categorical machinery you're plugging into
- [Glossary](../reference/glossary.md) — every term defined
- [#44](https://github.com/i-am-logger/pr4xis/issues/44) — the issue this doc closes

---

- **Document date:** 2026-04-14
