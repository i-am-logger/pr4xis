# English -- English Language Ontology

Models the English language as a complete ontology implementing the `Language` trait. The ontology is built by a functor from Open English WordNet (via the LMF reader): `XML → XmlOntology → LmfFunctor → WordNet → English`. Concepts (synsets) carry taxonomy (hypernym/hyponym), mereology (meronym), and opposition (antonym) relations. Function words, verb transitivity (from subcategorization frames), writing system, morphological rules, and pregroup type assignments are all pre-computed at construction time; queries return references and are zero-allocation.

Key references:
- Miller 1995: *WordNet: A Lexical Database for English* (Open English WordNet 2025)
- ISO 24613 LMF 2008: Lexical Markup Framework (the serialization this consumes)
- Lambek 1958 / 2008: *Pregroup Grammars* (pregroup type assignment)
- Pustejovsky 1995: *The Generative Lexicon* (sense structure)
- Levin 1993: *English Verb Classes and Alternations* (transitivity frames)
- Chiarcos & Sukhareva 2015: *OLiA* (function-word categories)

## Entities

| Category | Entities |
|---|---|
| Concepts | `Concept { id: ConceptId, original_id, pos, lemmas, definitions, examples }` — one per synset |
| Senses | `SenseId` — one per word-meaning pair |
| Lexical entries | `LexicalEntry` (see `../lexicon/pos.rs` for rich types) |

The `English` struct holds: `concepts`, `word_index`, `taxonomy_children`/`taxonomy_parents`, `opposition`, `mereology_parts`, `synset_to_concept`, `sense_to_id`, `function_words`, `verb_transitivity`, `writing`, `morphology`.

## Relations

| Relation | Source → Target | Meaning |
|---|---|---|
| Taxonomy (is-a) | child concept → parent concept | hypernym / hyponym |
| Mereology (has-a) | whole concept → part concept | meronym / holonym |
| Opposition | sense → sense | antonym |

## Qualities

| Quality | Type | Description |
|---|---|---|
| Concept POS | `lmf::LmfPos` | part-of-speech of the synset (noun, verb, adjective, adverb) |
| Transitivity | `Transitivity` | from WordNet subcategorization frames |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Language trait invariants; taxonomy acyclicity; concept/sense indices consistent | auto-generated |

Domain content is carried by the loaded WordNet data rather than by hand-written axioms; the functor from WordNet is the proof that the ontology is faithful to its source.

## Functors

Incoming:

| Functor | Source | File |
|---|---|---|
| `English::from_wordnet` | `social::software::markup::xml::lmf` WordNet | `ontology.rs` |

Outgoing: the `Language` trait impl acts as a functor into `cognitive::linguistics::language` (function words, pregroup types, morphological lookup, known words). See [Compose via functor](../../../../../../docs/use/compose-via-functor.md) for adding more.

## Files

- `ontology.rs` -- `English`, `Concept`, `ConceptId`, `SenseId`, `from_wordnet` functor, `Language` trait impl, zero-allocation query methods
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
