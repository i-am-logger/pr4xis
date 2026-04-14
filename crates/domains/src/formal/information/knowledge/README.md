# Knowledge -- A system's self-description of what it knows

Models a knowledge base as the catalog level above individual ontologies: a `KnowledgeBase` catalogs `Vocabulary` objects, each of which conforms to a `Schema`, contains `Entry` objects, is described by a `Descriptor` (counts and statistics), and is derived from a `DataSource` (paper, spec, file). The causal connection (Smith 1984) is enforced — the self-description is computed from the actually loaded state, not from static metadata.

Key references:
- W3C VoID 2011: *Vocabulary of Interlinked Datasets*
- W3C DCAT v3 2024: *Data Catalog Vocabulary*
- W3C SKOS 2009: *Simple Knowledge Organization System*
- Herre & Loebe 2005: *A Meta-ontological Architecture* (FOIS)
- Smith 1984: causal connection between object language and meta-language

## Entities (6)

| Category | Entities |
|---|---|
| Catalog level (1) | KnowledgeBase |
| Vocabulary level (3) | Vocabulary, Schema, Entry |
| Metadata (2) | Descriptor, DataSource |

## Category

Morphisms align to W3C vocabularies: `KnowledgeBase Catalogs Vocabulary` (dcat:dataset), `Vocabulary ConformsTo Schema` (dct:conformsTo), `Vocabulary Contains Entry` (void:entity), `Vocabulary DescribedBy Descriptor` (void statistics), `Vocabulary DerivedFrom DataSource` (prov:wasDerivedFrom), `Schema Defines Entry` (skos:inScheme inverse). Composition closes KnowledgeBase → Entry, Schema, Descriptor, and DataSource.

## Qualities

| Quality | Type | Description |
|---|---|---|
| IsStructural | bool | KnowledgeBase, Vocabulary, Schema = true (schema level); Entry, Descriptor, DataSource = false (instance level) |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the knowledge-base kinded relation graph | auto-generated |

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. Note: `descriptor.rs` imports many other ontologies (communication, concurrency, dialogue, events, provenance, systems, etc.) to populate the KnowledgeBase descriptor — but those are reflective uses, not formal functors.

## Files

- `ontology.rs` -- `KnowledgeConcept`, knowledge-base category, IsStructural quality, tests
- `descriptor.rs` -- Runtime descriptor that enumerates loaded vocabularies
- `instance.rs` -- KnowledgeBase instance construction
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
