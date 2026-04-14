# Schema -- Ontology structure as data (Spivak functorial data migration)

Models ontology STRUCTURE as data — the meta (M2) level. A schema is a small category whose objects are entity types and whose morphisms are typed relationships (Spivak 2012). Instances are functors from a schema to Set; path equations enforce composition constraints; schema mappings induce migration functors. This enables introspection, migration, comparison, and generic persistence across storage backends.

Key references:
- Spivak 2012: *Functorial Data Migration* (Information and Computation)
- Spivak 2009: *Simplicial Databases* (arXiv:0904.2012)
- Spivak & Wisnesky 2015: *Relational Foundations for Functorial Data Migration*
- Wisnesky et al. 2017: *Algebraic Databases*
- Baader et al. 2003: *The Description Logic Handbook* (TBox / ABox)
- OMG MDA Guide v2.0 2014 (M0 / M1 / M2 / M3 levels)

## Entities (11)

| Category | Entities |
|---|---|
| Schema structure (5) | Schema, EntityType, MorphismType, PathEquation, Axiom |
| Instances (3) | Instance, Population, Transform |
| Schema morphisms (1) | SchemaMapping |
| CQL syntactic / semantic (2) | Presentation, Algebra |

## Category

Morphisms: Schema contains EntityType / MorphismType / PathEquation / Axiom. `EntityType Participates MorphismType`. `Instance InstantiatedFrom Schema` (Spivak: I: C → Set), `Instance Assigns Population`, `Population Participates EntityType`. `SchemaMapping Maps Schema`, `Transform Transforms Instance`. `Presentation Evaluates Algebra` and `Algebra Presents Presentation` (CQL evaluation/presentation adjunction). Composition closes Schema → Instance → Population.

## Qualities

| Quality | Type | Description |
|---|---|---|
| MdaLevelQuality | MdaLevel | Schema, EntityType, MorphismType, PathEquation, Axiom, SchemaMapping = M2; Instance, Transform, Presentation, Algebra = M1; Population = M0 |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the schema kinded relation graph | auto-generated |

The domain content is encoded as morphism edges verified by tests: Schema contains EntityTypes / MorphismTypes / PathEquations (Spivak 2012), Instance is a functor from Schema (Spivak: I: C → Set), Presentation evaluates to Algebra and vice versa (CQL), TBox/ABox distinction (Baader 2003), SchemaMapping induces migration functors.

## Functors

**Outgoing (2):**

| Functor | Target | File |
|---|---|---|
| SchemaToSystems | systems | `systems_functor.rs` |
| SchemaToTrace | observability trace | `trace_functor.rs` |

**Incoming:** no incoming cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `SchemaConcept`, meta-level category, MdaLevel quality, tests
- `instance.rs` -- Runtime instance construction (functor to Set)
- `alignment.rs` -- Schema alignment / mapping construction
- `systems_functor.rs` -- Schema → systems-thinking functor
- `trace_functor.rs` -- Schema → observability trace functor
- `trace_schema.rs` -- Trace-specific schema definition
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
