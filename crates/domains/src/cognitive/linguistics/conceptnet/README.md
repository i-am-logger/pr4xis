# ConceptNet -- Commonsense Association Ontology

Loads a WordNet-lemma-crosswalk-filtered slice of ConceptNet 5.7.0 (Speer, Chin & Havasi 2017) as typed data: `TSV → TsvRecords → ConceptNet edge list → ConceptNetStore`, riding the SAME generic `ContentType::Plaintext` raw-source path as `CaseFoldingTable`/`InflectionLexicon` — a plain data-loader module, mirroring `verbnet/`'s shape (an instance-data loader, not a `pr4xis::ontology!`-macro vocabulary; no schema-driven codegen exists anywhere in this codebase). No official English-only or WordNet-linked subset of ConceptNet is distributed, so the loaded table is a MECHANICAL filter of the full 34,074,917-row assertions CSV down to the 932,948 rows whose start AND end concepts both resolve to a loaded WordNet lemma — see `regenerate.rs`'s module doc for the exact filter and its provenance.

Independent of both WordNet's own gloss/hypernym construction and VerbNet's syntactic-alternation classification — ConceptNet's assertions are crowd- and dataset-sourced commonsense relations, a genuinely separate, third signal for concept relatedness, used by `cognitive::linguistics::english::word_sense`'s lone-hit corroboration gate alongside VerbNet.

Key references:
- Speer, Chin & Havasi 2017: *ConceptNet 5.5: An Open Multilingual Graph of General Knowledge* (ConceptNet itself)
- ConceptNet Wiki, Relations: <https://github.com/commonsense/conceptnet5/wiki/Relations> (the 34 relation types the English portion of ConceptNet 5.7.0 carries)

## Entities

| Category | Entities |
|---|---|
| Typed data | `ConceptNetEdge { relation, start_lemma, end_lemma, weight }` — one filtered assertion |
| | `ConceptNet { edges }` — the full loaded, filtered edge set (932,948 in 5.7.0) |
| Query index | `ConceptNetStore` — a symmetric `lemma → {lemma}` adjacency map, built once from `ConceptNet` |

## Relations

| Relation | Source → Target | Meaning |
|---|---|---|
| `Association` (`formal::relations`) | concept (`ConceptId`) → concept | every ConceptNet relation type (`RelatedTo`, `IsA`, `PartOf`, `UsedFor`, …) is mapped GENERICALLY onto this one existing relation kind (SKOS `related`), never a fine-grained per-relation-type mapping — see the module doc on `ontology.rs`'s `ConceptNetEdge::relation` field for why |

## Qualities

| Quality | Type | Description |
|---|---|---|
| Association sharing | `bool` (`ConceptNetStore::shares_association`) | whether ANY lemma of one concept's synset has a recorded ConceptNet edge to ANY lemma of another's |
| Coverage | `bool` (`ConceptNetStore::has_coverage`) | whether a `ConceptId` has ANY ConceptNet node at all (under any of its synset's lemmas) — kept distinct from "queried, no connection" |

## Axioms

Domain content (which lemma associates with which) is carried by the loaded, filtered ConceptNet data rather than by hand-written axioms; the `read_conceptnet` reader plus `regenerate.rs`'s filter are the provenance chain from the official upstream release to the committed data. The corroboration mechanism's own axioms (scoping ConceptNet consultation to lone-hit cases, composing with VerbNet's signal) live in `cognitive::linguistics::english::word_sense` and `crates/chat/src/lib.rs`, alongside VerbNet's.

## Functors

Incoming:

| Functor | Source | File |
|---|---|---|
| `reader::read_conceptnet` | generic `TsvRecords` (via `applied::data_provisioning::decoders::plaintext_tsv`) | `reader.rs` |

There is no formal `pr4xis::Functor` bridging ConceptNet lemmas to WordNet `ConceptId`s — unlike VerbNet's precomputed sense-key crosswalk, ConceptNet nodes are not sense-disambiguated (see `store.rs`'s module doc), so resolution happens live, at query time, via the already-loaded `LexicalReasoner`, not via a category-theoretic mapping or a precomputed table.

Outgoing: `cognitive::linguistics::english::word_sense::best_reaching_pair` consults `ConceptNetStore` as a second, independent corroboration source for the two-entity relation path's lone-hit case, alongside `VerbNetStore`, under the same relation-kind scoping discipline VerbNet's own regression (this codebase's committed corpus is-a class, 4 → 47 failures) established.

## Files

- `ontology.rs` -- `ConceptNetEdge`, `ConceptNet` (aggregate)
- `reader.rs` -- `read_conceptnet` — interprets the generic TSV record stream as ConceptNet's field shape
- `store.rs` -- `ConceptNetStore`, `normalize_lemma` (the shared bundling/query-time normalization), `conceptnet_loaded()` (cached runtime entry point)
- `regenerate.rs` -- `regenerate_conceptnet_archive` (`#[ignore]`d offline regen test) — fetches, filters and writes the committed `.assoc` TSV from the raw upstream release
- `mod.rs` -- module declarations
