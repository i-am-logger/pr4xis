# VerbNet -- Verb-Class Classification Ontology

Loads VerbNet's syntactic-semantic verb classification (Kipper, Korhonen, Ryant & Palmer 2008) as typed data: `XML → XmlDocument → VerbNetClass tree → VerbNetStore`, mirroring the WordNet ingestion shape (`XML → XmlDocument → WordNet → English`) — a hand-authored typed reader over the generic XML tree, not schema-generated (no source in this codebase gets one). Concepts are per-class member lists (`<MEMBER name="..." wn="...">`) nested under Levin's (1993) syntactic-alternation class hierarchy (`<VNCLASS>`/`<VNSUBCLASS>`); the store flattens that recursive tree into a `ConceptId`-indexed lookup for the corroboration query the two-entity relation path consults.

Independent of WordNet's own gloss/hypernym construction — VerbNet's classification derives from Levin's syntactic-alternation diagnostics, not lexicographic definition — so it grounds a genuinely separate signal for verb relatedness, used by `cognitive::linguistics::english::word_sense`'s lone-hit corroboration gate.

Key references:
- Kipper, Korhonen, Ryant & Palmer 2008: *A Large-scale Classification of English Verbs* (VerbNet itself)
- Levin 1993: *English Verb Classes and Alternations* (the syntactic-alternation diagnostics VerbNet's class hierarchy is built from; the "semantic coherence hypothesis" that scopes what class comembership is and isn't evidence for)
- Fellbaum 1998: *WordNet: An Electronic Lexical Database* (WNDB(5WN) sense-key format VerbNet's `wn=` attribute uses)
- Olsen, Dorr & Clark 1997: *Using WordNet to Posit Hierarchical Structure in Levin's Verb Classes* (grounds why VerbNet class structure carries no native hypernymy signal)
- Baker & Ruppenhofer 2002: *FrameNet's Frames vs. Levin's Verb Classes* (the same class-vs-hierarchy mismatch against FrameNet)

## Entities

| Category | Entities |
|---|---|
| Typed data | `VerbNetMember { name, wn_sense_keys }` — one verb's membership in a class, keyed by Princeton WordNet sense-key(s) |
| | `VerbNetClass { id, members, subclasses }` — one `<VNCLASS>`/`<VNSUBCLASS>` (same recursive shape at every nesting depth) |
| | `VerbNet { classes }` — the full loaded collection (332 top-level classes in VerbNet 3.3) |
| Query index | `VerbNetStore` — flattened `class id → parent id` table plus a `ConceptId → class ids` reverse index, built once from `VerbNet` + the precomputed WordNet crosswalk |

## Relations

| Relation | Source → Target | Meaning |
|---|---|---|
| Subclass nesting | `VNSUBCLASS` → parent `VNCLASS`/`VNSUBCLASS` | a syntactic-semantic refinement of the parent class (Levin 1993 diastasis alternations narrow membership) |
| `MemberOf` (`formal::relations`) | verb sense (`ConceptId`) → class | the individual-to-classification relation the corroboration mechanism reasons over; irreflexive, not symmetric, not transitive at this kind alone — see `formal/relations/ontology.rs` |

## Qualities

| Quality | Type | Description |
|---|---|---|
| Class-family sharing | `Option<String>` (`VerbNetStore::shares_class_family`) | the nearest common ancestor class two `ConceptId`s' sense-keys resolve into, or `None` |
| Coverage | `bool` (`VerbNetStore::has_coverage`) | whether a `ConceptId` has ANY VerbNet class membership at all — kept distinct from "queried, no connection" |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| `VerbNetCorroborationScopedToSimilarityAndEquivalence` (`crates/chat/src/lib.rs`) | the corroboration gate never downgrades a `Subsumption` (is-a) query, even when VerbNet places two concepts in unrelated classes | Levin 1993; Olsen, Dorr & Clark 1997; Baker & Ruppenhofer 2002; Kipper et al. 2008 |
| `MemberOfIsIrreflexive` (`formal/relations/ontology.rs`) | the `MemberOf` relation kind is irreflexive | Smith et al. 2005 OBO-RO `member_of`; Tarski 1941 |

Domain content (which verb belongs to which class) is carried by the loaded VerbNet XML data rather than by hand-written axioms; the `read_verbnet` reader is the proof the typed structure is faithful to the source.

## Functors

Incoming:

| Functor | Source | File |
|---|---|---|
| `reader::read_verbnet_class` / `reader::read_verbnet` | generic `XmlDocument` (via `social::software::markup::xml::reader`) | `reader.rs` |

There is no formal `pr4xis::Functor` bridging VerbNet senses to WordNet `ConceptId`s — the crosswalk is a precomputed data table (`store.rs`'s module doc explains why: the sense-key → synset-id resolution needs a raw WordNet XML parse `English`'s compact runtime load path is built to avoid, so it is resolved ONCE, offline, by `verbnet_class_collection::regenerate::regenerate_verbnet_archive`, and bundled as ordinary data alongside the class files — not a live category-theoretic mapping).

Outgoing: `cognitive::linguistics::english::word_sense::best_reaching_pair` consults `VerbNetStore` as an independent corroboration source for the two-entity relation path's lone-hit case, scoped to `Similarity`/`Equivalence` relation kinds only (see `word_sense.rs`'s module doc for the full, literature-grounded rationale and the real regression — this codebase's committed corpus is-a class, 4 → 47 failures — that scoping fixes).

## Files

- `ontology.rs` -- `VerbNetMember`, `VerbNetClass` (recursive), `VerbNet` (aggregate), `self_and_descendants`
- `reader.rs` -- `read_verbnet_class`, `read_verbnet`, `VerbNetReadError` — hand-authored typed reader over the generic XML tree, mirroring `lmf::reader::read_wordnet`'s shape
- `store.rs` -- `VerbNetStore`, `oewn_sense_id_for_sense_key` (the sense-key → OEWN-Sense-id half of the WordNet crosswalk), `verbnet_loaded()` (cached runtime entry point)
- `mod.rs` -- module declarations
