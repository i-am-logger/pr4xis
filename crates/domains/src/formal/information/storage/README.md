# Storage -- Repository / Store / Materialize / Realize

Models where and how ontologies are persisted. A `Repository` is the abstract interface; a `Store` is a pluggable physical backend (static, mmap, heap, database, HTTP endpoint); `Materialize` converts a live ontology to stored form; `Realize` loads stored form back to a live ontology; `Equivalence` is the proof that two stores contain the same ontology (natural isomorphism between instance functors). The roundtrip `Materialize ∘ Realize = identity` is a first-class morphism.

Key references:
- RDF4J: Repository / Sail architecture (Eclipse Foundation)
- Jena TDB: Dataset / Store model (Apache Foundation)
- W3C SPARQL 1.1 2013: *Graph Store HTTP Protocol*
- Spivak 2012: *Functorial Data Migration* (instance functors into different targets)
- OMG MDA v2.0 2014 (PIM → PSM model transformation)
- Gupta & Mumick 1995: *Maintenance of Materialized Views*
- Haerder & Reuter 1983: *Principles of Transaction-Oriented Database Recovery* (ACID)

## Entities (11)

| Category | Entities |
|---|---|
| Abstractions (3) | Repository, Store, StoredOntology |
| Operations (3) | Materialize, Realize, Equivalence |
| Backends (5) | StaticStore, MappedStore, HeapStore, DatabaseStore, EndpointStore |

## Category

Morphisms: `Repository Contains Store`, `Store Holds StoredOntology`, `Materialize Materializes StoredOntology`, `Realize Realizes StoredOntology`, `Equivalence Proves StoredOntology` (isomorphism). The five backend concepts `SpecializesTo Store` (taxonomy of physical stores). The roundtrip `Materialize Roundtrip Realize` is the identity axiom. Composition closes Repository → StoredOntology.

## Qualities

| Quality | Type | Description |
|---|---|---|
| SupportsHotReload | bool | StaticStore = false; MappedStore, HeapStore, DatabaseStore, EndpointStore = true; others = None |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the storage kinded relation graph | auto-generated |

Domain content encoded as morphism edges and verified by tests: Repository contains Stores (RDF4J), five backends all specialize Store, Materialize produces StoredOntology (Gupta & Mumick 1995), Realize loads StoredOntology (MDA), Equivalence proves isomorphism (Spivak functorial data migration), Materialize∘Realize = identity.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. Storage is a foundational ontology that the runtime Repository / Store implementations realize.

## Files

- `ontology.rs` -- `RepositoryConcept`, storage category, SupportsHotReload quality, tests
- `volatility.rs` -- Volatility classification (persistent / volatile)
- `consistency.rs` -- Consistency classification (see storage-consistency product category)
- `durability.rs` -- Durability classification (ACID durability levels)
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
