# FrameNet -- Frame-Semantic Ontology

Loads a mechanically-extracted slice of FrameNet 1.7 (Baker, Fillmore & Lowe 1998) as typed data: `TSV → TsvRecords → FrameNet lexical-unit/relation lists → FrameNetStore`, riding the SAME generic `ContentType::Plaintext` raw-source path as `AssociativeConceptTable`/`CaseFoldingTable` — a plain data-loader module, mirroring `conceptnet/`'s shape (an instance-data loader, not a `pr4xis::ontology!`-macro vocabulary; no schema-driven codegen exists anywhere in this codebase). No official small subset of FrameNet is distributed either, so the loaded table is a MECHANICAL extraction of each lexical unit's root `<lexUnit POS="..." name="..." frame="...">` start-tag attributes, plus every `frRelation.xml` frame-to-frame relation row — see `regenerate.rs`'s module doc for the exact extraction and its provenance.

Independent of WordNet's own gloss/hypernym construction, VerbNet's syntactic-alternation classification, and ConceptNet's crowd-sourced association graph — a third, genuinely separate signal for concept relatedness, used by `cognitive::linguistics::english::word_sense`'s lone-hit corroboration gate alongside VerbNet and ConceptNet.

Key references:
- Baker, Fillmore & Lowe 1998: *The Berkeley FrameNet Project* (FrameNet itself)
- Ruppenhofer, Ellsworth, Petruck, Johnson & Scheffczyk 2016: *FrameNet II: Extended Theory and Practice* (the 9 frame-to-frame relation types)
- Ferrández et al. 2010, LREC: *Aligning FrameNet and WordNet based on Semantic Neighborhoods* (evidence no native FrameNet-WordNet link exists — this project's own lemma+POS-keyed resolution is the same kind of external bridge that literature builds)

## Entities

| Category | Entities |
|---|---|
| Typed data | `FrameNetLexicalUnit { lemma, pos, frame }` — one lexical unit's frame membership |
| | `FrameNetRelation { relation, sub_frame, super_frame }` — one frame-to-frame relation edge |
| | `FrameNet { lexical_units, relations }` — the full loaded, extracted data (13,336 open-class LU rows + 2,070 relation rows in 1.7) |
| Query index | `FrameNetStore` — `(lemma, LmfPos) → {frame}` lookup plus a symmetric frame-to-frame adjacency map, built once from `FrameNet` |

## Relations

| Relation | Source → Target | Meaning |
|---|---|---|
| `Association` (`formal::relations`) | concept (`ConceptId`) → concept | every FrameNet frame-to-frame relation type (`Inheritance`, `Using`, `Subframe`, `Perspective_on`, `Causative_of`, `Inchoative_of`, `Precedes`, `Metaphor`, `See_also`) is mapped GENERICALLY onto this one existing relation kind (SKOS `related`), never a fine-grained per-relation-type mapping — see `ontology.rs`'s `FrameNetRelation::relation` field doc for why |

## Qualities

| Quality | Type | Description |
|---|---|---|
| Frame-family sharing | `bool` (`FrameNetStore::shares_frame_family`) | whether two concepts evoke the SAME frame, or frames connected by a direct (one-hop) frame relation |
| Coverage | `bool` (`FrameNetStore::has_coverage`) | whether a `ConceptId` has ANY FrameNet lexical-unit membership at all, at its own POS — kept distinct from "queried, no connection" |

## Axioms

Domain content (which lemma+POS evokes which frame) is carried by the loaded, extracted FrameNet data rather than by hand-written axioms; the `read_framenet` reader plus `regenerate.rs`'s extraction are the provenance chain from the official upstream release to the committed data. The corroboration mechanism's own axioms (scoping FrameNet consultation to lone-hit cases, composing with VerbNet's and ConceptNet's signals) live in `cognitive::linguistics::english::word_sense` and `crates/chat/src/lib.rs`, alongside VerbNet's and ConceptNet's.

## Functors

Incoming:

| Functor | Source | File |
|---|---|---|
| `reader::read_framenet` | generic `TsvRecords` (via `applied::data_provisioning::decoders::plaintext_tsv`) | `reader.rs` |

There is no formal `pr4xis::Functor` bridging FrameNet lexical units to WordNet `ConceptId`s — like ConceptNet (and unlike VerbNet's precomputed sense-key crosswalk), FrameNet's lexical units carry no native WordNet sense-key or synset reference, so resolution happens live, at query time, via the already-loaded `LexicalReasoner`, keyed on `(lemma, POS)` for precision (FrameNet DOES carry real POS information, unlike ConceptNet's bare nodes).

Outgoing: `cognitive::linguistics::english::word_sense::best_reaching_pair` consults `FrameNetStore` as a third, independent corroboration source for the two-entity relation path's lone-hit case, alongside `VerbNetStore` and `ConceptNetStore`, under the same relation-kind scoping discipline VerbNet's own regression (this codebase's committed corpus is-a class, 4 → 47 failures) established.

## Files

- `ontology.rs` -- `FrameNetLexicalUnit`, `FrameNetRelation`, `FrameNet` (aggregate)
- `reader.rs` -- `read_framenet` — interprets the generic TSV record stream as FrameNet's field shape
- `store.rs` -- `FrameNetStore`, `normalize_lemma` (local copy of the shared lemma-canonicalization logic), `framenet_loaded()` (cached runtime entry point)
- `regenerate.rs` -- `regenerate_framenet_archive` (`#[ignore]`d offline regen test) — extracts and writes the committed `.tsv` from the raw upstream release, plus the hand-rolled multi-entry ZIP reader it needs (no system `unzip` tool available in this environment; reuses the same PKZIP byte-format `applied::data_provisioning::fetch`'s private `unzip_single_xml` already parses)
- `mod.rs` -- module declarations
