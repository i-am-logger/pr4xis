# Information -- Units of information as a kinded category

Parent ontology for the `formal/information/` branch. Models the fundamental units of information representation (Bit, Byte, Word, Reference, Sequence, Text, TruthValue, Quantity) as a kinded relation graph with is-a, has-a, and semantic-equivalence edges. The parent captures the atomic/structured distinction and the classical Shannon hierarchy; the rich operational content lives in the child ontologies under `formal/information/*/` — communication, concurrency, diagnostics, dialogue, events, knowledge, measurement, provenance, schema, storage.

Key references:
- Shannon 1948: *A Mathematical Theory of Communication*
- Turing 1936: *On Computable Numbers, with an Application to the Entscheidungsproblem*

## Entities (8)

| Category | Entities |
|---|---|
| Atomic (3) | Bit, TruthValue, Sequence |
| Structured (5) | Byte, Word, Reference, Text, Quantity |

## Category

`InfoCategory` is a kinded relation graph over the eight `InfoUnit` entities. Kinds:
- `ComposedOf` (mereology) — Byte has-a Bits, Word has-a Bytes, Text has-a Sequence
- `IsA` (taxonomy) — Reference is-a Word, Text is-a Sequence, Quantity is-a Sequence
- `Equivalent` — TruthValue ≡ Bit (semantic equivalence)

Composition closes the transitive edges: Word is composed of Bits (via Bytes), Reference is composed of Bits (via Word → Byte → Bit).

## Qualities

| Quality | Type | Description |
|---|---|---|
| BitSize | usize | Number of bits per unit (Bit=1, TruthValue=1, Byte=8, Word=32 or 64, etc.) |
| IsAtomic | bool | True for Bit, TruthValue, Sequence (no internal structure) |

## Axioms

Auto-generated structural axioms from `define_ontology!` (category laws, kind-relation consistency, closure of the `composed` edges). Domain axioms for the operational subfields live in the child ontologies — `formal/information/communication` for channel capacity, `formal/information/storage` for the repository/store roundtrip, etc.

## Child ontologies

- [`communication/`](communication/README.md) — Shannon channel, Jakobson message model
- [`concurrency/`](concurrency/README.md) — agents, shared resources, synchronization, deadlock, race conditions
- [`diagnostics/`](diagnostics/README.md) — MAPE-K loop, fault / symptom / diagnosis
- [`dialogue/`](dialogue/README.md) — speech acts, dialogue acts, common ground
- [`events/`](events/README.md) — CQRS, event sourcing, event log, projections
- [`knowledge/`](knowledge/README.md) — descriptor, dataset, vocabulary, concept
- [`measurement/`](measurement/README.md) — VIM + Stevens scales
- [`provenance/`](provenance/README.md) — W3C PROV-O core
- [`schema/`](schema/README.md) — Spivak functorial data migration
- [`storage/`](storage/README.md) — Repository / Store / Materialize / Realize

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../docs/use/compose-via-functor.md) to add one. The parent information ontology is a substrate shared by its children; functors typically target the children (communication, events, storage, etc.) rather than this parent.

## Files

- `ontology.rs` -- `InfoUnit`, `InfoCategory`, `BitSize` / `IsAtomic` qualities, tests
- `mod.rs` -- module declarations and re-exports
