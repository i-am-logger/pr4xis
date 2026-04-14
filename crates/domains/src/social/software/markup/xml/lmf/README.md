# LMF -- WordNet Lexical Markup Framework Ontology

Models WordNet lexical data via the Lexical Markup Framework: synsets (meanings), lexical entries (words), senses (word-meaning bindings), and their synset- and sense-level relations. Part-of-speech tags are extended beyond WordNet's four open-class tags to include Universal Dependencies / OLiA closed-class tags, giving fourteen entity variants.

Key references:
- ISO 24613 (2008) *Language resource management — Lexical markup framework (LMF)*
- Fellbaum 1998: *WordNet: An Electronic Lexical Database*
- Global WordNet Association — WN-LMF 1.3 schema
- Chiarcos & Sukhareva 2015: *OLiA* (linking ontologies for annotated corpora)
- Universal Dependencies v2 — POS tagset
- Miller 1995: *WordNet: A Lexical Database for English*

## Entities

| Category | Entities |
|---|---|
| Parts of speech (14) | Noun, Verb, Adjective, Adverb, Determiner, Pronoun, Preposition, Conjunction, Particle, Copula, Auxiliary, Interjection, Numeral, Other |
| Rich types | Synset, LexicalEntry, Lemma, Sense, Form, SynsetRelation, SenseRelation, WordNet |
| Synset relation types (19) | Hypernym, InstanceHypernym, Hyponym, InstanceHyponym, HoloMember, HoloPart, HoloSubstance, MeroMember, MeroPart, MeroSubstance, Causes, Entails, Similar, Also, Attribute, DomainTopic, DomainRegion, Exemplifies, Other |
| Sense relation types (6) | Antonym, Similar, Pertainym, Derivation, Exemplifies, Other |
| Verb transitivity (3) | Intransitive, Transitive, Ditransitive |

## Qualities

The LMF types expose aggregate queries over a loaded `WordNet`:

| Query | Return | Description |
|---|---|---|
| `taxonomy_relations` | `Vec<(&str,&str)>` | Hypernym and instance-hypernym edges |
| `mereology_relations` | `Vec<(&str,&str)>` | Holonym and meronym edges |
| `opposition_relations` | `Vec<(&str,&str)>` | Sense-level antonym edges |
| `causal_relations` | `Vec<(&str,&str)>` | Causes and entails edges |
| `is_open_class` / `is_closed_class` | bool | Open/closed word class partition on `LmfPos` |

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Entity-set coverage on `LmfPos` | auto-generated |

Domain invariants verified by tests: `LmfPos` has fourteen variants; `parse`/`to_tag` roundtrip; the open-class / closed-class partition covers every variant except `Other`; taxonomy/mereology/causal predicates classify relation types consistently with ISO 24613 LMF.

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../../../docs/use/compose-via-functor.md) to add one. LMF relations are designed to map directly to the core reasoning ontologies: hypernym → taxonomy, holo/mero → mereology, antonym → opposition, causes → causal.

## Files

- `ontology.rs` -- `LmfPos` entity, `Synset`/`LexicalEntry`/`Lemma`/`Sense`/`Form` rich types, `SynsetRelationType`/`SenseRelationType`, `VerbTransitivity`, `WordNet` container, tests
- `reader.rs` -- WN-LMF XML reader producing a `WordNet`
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
