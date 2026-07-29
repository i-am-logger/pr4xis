# SUMO -- WordNet↔Upper-Ontology Mapping

Loads the offline-resolved WordNet↔SUMO crosswalk (Niles & Pease 2001, 2003) as typed data: `TSV → TsvRecords → Sumo mapping list → SumoStore`, riding the SAME generic `ContentType::Plaintext` raw-source path as `AssociativeConceptTable`/`FrameSemanticTable` — a plain data-loader module, mirroring `conceptnet/`'s and `framenet/`'s shape (an instance-data loader, not a `pr4xis::ontology!`-macro vocabulary; no schema-driven codegen exists anywhere in this codebase). SUMO's `WordNetMappings30-{noun,verb,adj,adv}.txt` files annotate Princeton WordNet **3.0** synsets by offset; Open English WordNet 2025 (the WordNet this project loads) does NOT preserve those offsets (only ~385 of 82,115 noun offsets collide), so — unlike ConceptNet/FrameNet's live lemma resolution — the synset→`ConceptId` resolution happens ONCE, OFFLINE, in `regenerate.rs` via the version-stable WordNet SENSE KEY reconstructed from each SUMO line's own WNDB record fields (mirroring VerbNet's precomputed sense-key crosswalk). The committed table therefore carries the resolved `ConceptId` value directly; the runtime store needs no `LexicalReasoner` at all.

Independent of WordNet's own gloss/hypernym construction, VerbNet's syntactic-alternation classification, ConceptNet's crowd-sourced association graph, and FrameNet's frame semantics — a FOURTH, genuinely separate signal for concept relatedness (a formal upper-ontology class crosswalk), used by `cognitive::linguistics::english::word_sense`'s lone-hit corroboration gate alongside VerbNet, ConceptNet, and FrameNet.

Key references:
- Niles & Pease 2001, FOIS: *Towards a Standard Upper Ontology* (SUMO itself)
- Niles & Pease 2003, IEEE IKE: *Linking Lexicons and Ontologies: Mapping WordNet to the Suggested Upper Merged Ontology* (the WordNet↔SUMO mapping this data loads, including the suffix legend `=`/`+`/`@`/`:`/`[`/`]`)

## Entities

| Category | Entities |
|---|---|
| Typed data | `SumoMapping { concept, term, relation, oewn_synset_id }` — one WordNet concept's mapping to a SUMO upper-ontology term, already resolved to `ConceptId`, plus the real external OEWN synset id it resolves to |
| | `Sumo { mappings }` — the full loaded, offline-resolved data (91,218 rows over 90,490 distinct concepts, verified 2026-07-14 against `ontologyportal/sumo` commit `152b9abc`) |
| SSSOM mapping set | `SssomMappingSet` (`sssom.rs`) — the 5,330 unambiguous `Equivalence` rows minted as `skos:exactMatch` correspondences between real `en-word.net` synset URIs and SUMO's OWL term namespace |
| Query index | `SumoStore` — `ConceptId::value() → {(term, relation)}` lookup, built once from `Sumo` |
| Relation kind | `SumoRelationKind` — `Equivalence`/`Subsumption`/`Instance` (positive class-membership claims) and `ComplementEquivalence`/`ComplementSubsumption`/`ComplementInstance` (the synset is explicitly NOT that class) — the source's own six-suffix legend, never collapsed |

## Relations

| Relation | Source → Target | Meaning |
|---|---|---|
| `SumoRelationKind` (this module, not `formal::relations`) | concept (`ConceptId`) → SUMO term (`String`) | the exact relation the source's suffix asserts (equivalence/subsumption/instance, or one of their three complements) — kept as its OWN typed enum, not mapped onto the generic `Association` kind ConceptNet/FrameNet use, because the positive/complement distinction is load-bearing for `shares_sumo_class` (see Qualities below) and would be lost by a generic SKOS `related` collapse |

## Qualities

| Quality | Type | Description |
|---|---|---|
| Class sharing | `bool` (`SumoStore::shares_sumo_class`) | whether two concepts map to at least one SUMO term in common, where NEITHER occurrence is a `Complement*` relation — flat same-term matching only (SUMO's own class hierarchy, `Merge.kif`, is not loaded, so this is never an ancestor-walk) |
| Coverage | `bool` (`SumoStore::has_coverage`) | whether a `ConceptId` has ANY SUMO mapping at all (positive OR complement) — kept distinct from "queried, no connection" |

## Axioms

Domain content (which concept maps to which SUMO term) is carried by the loaded, offline-resolved SUMO data rather than by hand-written axioms; `read_sumo` plus `regenerate.rs`'s offline sense-key resolution are the provenance chain from the official upstream release to the committed data. The corroboration mechanism's own axioms (scoping SUMO consultation to lone-hit cases, composing with VerbNet's, ConceptNet's, and FrameNet's signals) live in `cognitive::linguistics::english::word_sense` and `crates/chat/src/lib.rs`'s `SumoCorroborationComposesWithVerbNetConceptNetAndFrameNet`, alongside the other three sources' own composition axioms.

## Functors

Incoming:

| Functor | Source | File |
|---|---|---|
| `reader::read_sumo` | generic `TsvRecords` (via `applied::data_provisioning::decoders::plaintext_tsv`) | `reader.rs` |

There is no formal `pr4xis::Functor` bridging SUMO terms to a Rust-typed upper ontology — only the crosswalk (which WordNet concept maps to which SUMO TERM NAME) is loaded, never SUMO's own KIF axiom base (`Merge.kif` and the domain ontologies), so `SumoStore` has no class-hierarchy structure to functor over, only a flat term-name index.

Outgoing: `cognitive::linguistics::english::word_sense::best_reaching_pair` consults `SumoStore` as a FOURTH, independent corroboration source for the two-entity relation path's lone-hit case, alongside `VerbNetStore`, `ConceptNetStore`, and `FrameNetStore`, under the same relation-kind scoping discipline VerbNet's own regression (this codebase's committed corpus is-a class, 4 → 47 failures) established.

## Files

- `ontology.rs` -- `SumoRelationKind` (the six-suffix legend), `SumoMapping`, `Sumo` (aggregate)
- `reader.rs` -- `read_sumo` — interprets the generic TSV record stream (`concept_value<TAB>term<TAB>relation_code<TAB>oewn_synset_id`) as SUMO's resolved field shape; a pre-migration 3-column row is rejected fail-closed
- `store.rs` -- `SumoStore`, `sumo_loaded()` (cached runtime entry point), `sumo_mappings()` (the unindexed `Sumo` list, shared with `sssom.rs`) — pure re-index, no `LexicalReasoner` needed at load since the committed table is already `ConceptId`-resolved
- `regenerate.rs` -- `regenerate_sumo_archive` (`#[ignore]`d offline regen test) — extracts `&%<term><suffix>` annotations from the raw `WordNetMappings30-*.txt` WNDB-format lines, reconstructs each synset's WordNet sense key from its own member-word/lex_id/lex_filenum fields, resolves to `ConceptId` AND its canonical OEWN synset id via one offline OEWN 2025 XML parse (the same offline-precompute pattern VerbNet's `build_wordnet_crosswalk_tsv` uses, reusing its `oewn_sense_id_for_sense_key` step), and writes the committed, already-resolved `.tsv`
- `sssom.rs` -- `sumo_eq_sssom_mapping_set()`, `ambiguous_eq_concepts()` — the concrete SSSOM (Matentzoglu et al. 2022) producer over the `Equivalence` rows; see the module doc for the `object_id` URI-liveness honesty note (SUMO's OWL translation host is now dead, re-verified 2026-07-14)
- `mod.rs` -- module declarations
