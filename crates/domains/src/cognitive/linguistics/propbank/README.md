# PropBank -- Predicate Argument-Structure Lexicon

Loads the bundled PropBank frame-file collection (Palmer, Gildea & Kingsbury 2005; Bonial, Bonn, Conger, Hwang & Palmer 2014) as typed data: `PropBankFramesetCollection (path → bytes) → PropBank (frameset/predicate/roleset/alias tree) → PropBankStore`, riding the SAME generic directory-archive codec (`decoders::file_collection`) as `VerbClassLexicon`/`ColorSchemeVocabulary` — a plain data-loader module, mirroring `verbnet/`'s shape (an instance-data loader, not a `pr4xis::ontology!`-macro vocabulary; no schema-driven codegen exists anywhere in this codebase). Unlike FrameNet's/SUMO's regen-time field EXTRACTION into a flat TSV, PropBank's `frames/*.xml` files ARE already the shape this project needs (small, structured, per-lemma XML, one file per lemma) — so the bundled archive carries the RAW frame files verbatim, and field parsing (`frameset → predicate → roleset → aliases`, the DTD `pos`-code → `LmfPos` mapping) happens LIVE, at load time, in `reader.rs` — mirroring VerbNet's own division of labor exactly.

A FIFTH, independent signal for concept relatedness — a lexicographic argument-structure crosswalk, entirely distinct from WordNet's own gloss/hypernym construction, VerbNet's syntactic-alternation classes, ConceptNet's crowd-sourced associations, FrameNet's frame semantics, and SUMO's upper-ontology crosswalk — used by `cognitive::linguistics::english::word_sense`'s lone-hit corroboration gate.

Key references:
- Palmer, Gildea & Kingsbury 2005, Computational Linguistics 31(1): *The Proposition Bank: An Annotated Corpus of Semantic Roles* (PropBank itself)
- Bonial, Bonn, Conger, Hwang & Palmer 2014, LREC: *PropBank: Semantics of New Predicate Types* (the frame-file format's own requested companion citation, per propbank.github.io)

## Entities

| Category | Entities |
|---|---|
| Typed data | `RolesetAlias { text, pos_code, pos }` — one `<alias pos="...">text</alias>`, the raw DTD code plus its mapped `LmfPos` (`None` for an undocumented code) |
| | `Roleset { id, aliases }` — one `<roleset id="...">`'s aliases (e.g. `trade.01`) |
| | `PropBankPredicate { lemma, rolesets }` — one `<predicate lemma="...">`'s rolesets |
| | `PropBankFrameset { predicates }` — one parsed frame file (one `frames/<lemma>.xml`) |
| | `PropBank { framesets }` — the full loaded collection (7,565 frame files, verified 2026-07-13 against `propbank/propbank-frames` tag `v3.4.0`, commit `4087fa9ab5c40907c34ff91a56acc2cab1670145`) |
| Query index | `PropBankStore` — `(lemma, LmfPos) → {roleset id}` lookup, resolved LIVE against the loaded `LexicalReasoner` at query time (built once from `PropBank`) |

## Relations

| Relation | Source → Target | Meaning |
|---|---|---|
| `Association` (`formal::relations`) | concept (`ConceptId`) → concept | `shares_roleset`'s cross-POS roleset co-membership maps generically onto this one existing relation kind (SKOS `related`), never a fine-grained typed enum — PropBank's data carries no positive/negative distinction (no complement suffix, no negation marker) the way SUMO's `:`/`[`/`]` suffixes do, so there is nothing for a dedicated `PropBankRelationKind` to distinguish |

## Qualities

| Quality | Type | Description |
|---|---|---|
| Cross-POS roleset sharing | `bool` (`PropBankStore::shares_roleset`) | whether two concepts occur at DIFFERENT parts of speech AND reach at least one PropBank roleset id in common, each via aliases matching that concept's own POS — same-POS pairs are excluded entirely (redundant with VerbNet's existing verb-verb signal) |
| Coverage | `bool` (`PropBankStore::has_coverage`) | whether a `ConceptId` has ANY PropBank roleset membership at all, at its own POS — kept distinct from "queried, no connection" |

## Axioms

Domain content (which lemma+POS aliases share which roleset) is carried by the loaded frame-file collection rather than by hand-written axioms; `reader::read_propbank` plus `regenerate.rs`'s whole-directory archiving are the provenance chain from the official upstream release to the committed data. The corroboration mechanism's own axioms (scoping PropBank consultation to lone-hit cases, composing with VerbNet's, ConceptNet's, FrameNet's, and SUMO's signals) live in `cognitive::linguistics::english::word_sense` and `crates/chat/src/lib.rs`'s per-source composition axioms, alongside the other four sources' own.

## Functors

Incoming:

| Functor | Source | File |
|---|---|---|
| `reader::read_propbank` | `PropBankFramesetCollection` (via `applied::data_provisioning::decoders::propbank_frameset_collection`, itself the generic `decoders::file_collection` codec) | `reader.rs` |

There is no formal `pr4xis::Functor` bridging PropBank aliases to a Rust-typed WordNet crosswalk — resolution is entirely LIVE (query-time lemma+POS lookup against the loaded `LexicalReasoner`), the same FrameNet precedent, not a precomputed sense-key crosswalk the way VerbNet's/SUMO's regen builds one (PropBank's data carries no native WordNet link to precompute FROM).

Outgoing: `cognitive::linguistics::english::word_sense::best_reaching_pair` consults `PropBankStore` as a FIFTH, independent corroboration source for the two-entity relation path's lone-hit case, alongside `VerbNetStore`, `ConceptNetStore`, `FrameNetStore`, and `SumoStore`, under the same relation-kind scoping discipline VerbNet's own regression established — refined here into an explicit cross-POS gate rather than a relation-kind filter, since PropBank's own signal IS the cross-POS distinction.

## Files

- `ontology.rs` -- `RolesetAlias`, `Roleset`, `PropBankPredicate`, `PropBankFrameset`, `PropBank` (the nested aggregate), `propbank_pos_to_lmf` (the five-code DTD → `LmfPos` mapping)
- `reader.rs` -- `read_propbank_frameset` / `read_propbank` — interprets the generic `path → bytes` collection as PropBank's `frameset → predicate → roleset → alias` field shape, LIVE at load time
- `store.rs` -- `PropBankStore`, `normalize_lemma` (local copy of the shared lemma-canonicalization logic), `propbank_loaded()` (cached runtime entry point)
- `regenerate.rs` -- `regenerate_propbank_archive` (`#[ignore]`d offline regen test) — archives every real `frames/*.xml` file (excluding `.gitignore`/`README.txt`/`frameset.dtd`) into the deterministic directory-archive blob, verbatim (no field extraction — that happens live in `reader.rs`)
- `mod.rs` -- module declarations
