# Provenance -- W3C PROV-O + version control + knowledge sources

Models the origin and history of knowledge artifacts, grounded in W3C PROV-O (2013): `Artifact` (prov:Entity), `Activity`, and `Agent` are the three PROV-O core concepts, extended with version control (Repository, Commit, Branch, Tag, Version) and academic sourcing (Source, Citation). This lets praxis record which paper defines each concept, which commit introduced it, which version of which spec, and who authored the ontology.

Key references:
- W3C PROV-O 2013: *Provenance Ontology* (https://www.w3.org/TR/prov-o/)
- W3C PROV-DM 2013: *Provenance Data Model*

## Entities (10)

| Category | Entities |
|---|---|
| PROV-O core (3) | Artifact, Activity, Agent |
| Version control (5) | Repository, Commit, Branch, Tag, Version |
| Knowledge sources (2) | Source, Citation |

## Category

Morphisms encode PROV-O relations: `Artifact WasGeneratedBy Activity`, `Artifact WasDerivedFrom Artifact`, `Artifact WasAttributedTo Agent`, `Activity WasAssociatedWith Agent`, `Activity Used Artifact`. Version control adds `Commit BelongsTo Repository`, `Branch PointsTo Commit`, `Tag Marks Commit`, `Version Identifies Artifact`. Knowledge sourcing adds `Artifact DefinedBy Source` and `Citation References Source`. Composition closes Artifact → Agent, Activity → Artifact, Commit → Artifact, Branch → Repository, Tag → Repository.

## Qualities

| Quality | Type | Description |
|---|---|---|
| IsProvOCore | bool | Artifact, Activity, Agent = true; everything else = false (extension concepts) |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the provenance kinded relation graph | auto-generated |

## Functors

**Incoming (1):**

| Functor | Source | File |
|---|---|---|
| DiagnosticsToTrace → PROV | diagnostics trace | `../diagnostics/trace_functors.rs` |

Otherwise no outgoing cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one.

## Files

- `ontology.rs` -- `ProvenanceConcept`, PROV-O + VCS + source category, IsProvOCore quality, tests
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
