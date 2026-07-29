# Citings — FRBR -- Bibliographic Work/Expression/Manifestation hierarchy

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms.

## Primary sources

- IFLA Study Group on the Functional Requirements for Bibliographic Records 1998: *Functional Requirements for Bibliographic Records: Final Report* -- §3.2.1 Work, §3.2.2 Expression, §3.2.3 Manifestation (Group 1), §3.4 Concept (Group 3); grounds `WorkIsRealizedThroughAtLeastOneExpression` and `ExpressionIsEmbodiedInAtLeastOneManifestation`, and (via §3.4) the keystone axiom `GenreGroundsInWordNetDomainTopic`.
- Miller 1995: *WordNet: A Lexical Database for English*, Communications of the ACM 38(11) -- the general WordNet source; also co-cited on the real-corpus generated axiom `GenreDomainTopicAgreesAcrossLoadedEnglishWordNet`.
- Bentivogli & Pianta 2004: "Extending WordNet with Syntagmatic Information", Proc. GWC 2004 -- the `has_domain_topic`/`domain_topic` pointer pair this ontology's Genre concept grounds in (all three axioms in `wordnet_grounding.rs`).
- Magnini & Cavaglià 2000: *Integrating Subject Field Codes into WordNet*, Proc. LREC 2000 -- the domain-topic annotation methodology behind `has_domain_topic`/`domain_topic`; grounds `GenreDomainTopicRoundTripsOnFixtureWordNet`.

## Cross-references

- Source attributions per axiom: see the `## Axioms` table in [`README.md`](README.md)
- Code-level citations: `grep -n 'IFLA\|Miller\|Bentivogli\|Magnini' ontology.rs wordnet_grounding.rs` in this directory
- The `has_domain_topic`/`domain_topic` accessors this ontology's grounding axioms query live on `English` (`crates/domains/src/cognitive/linguistics/english/ontology.rs`) -- their doc comments were corrected 2026-07-21 (see below).
- Same-shape precedent: `../../formal/mereology/wordnet_grounding.rs`'s `wordnet_concept_of_mereology` classifier + `ProperPartAndWholeAreGroundedInWordNet` keystone + `MereologyPartsAgreeWithLoadedMeronymyAndCounting` fixture-composition -- the structure `wordnet_grounding.rs` in this directory mirrors, extended with a real-corpus generated sweep (`GenreDomainTopicAgreesAcrossLoadedEnglishWordNet`) the mereology precedent does not yet have.

## 2026-07-21 re-verification: two corrections against the loaded corpus

A re-verification pass against the real, loaded `english-wordnet-2025.xml` (Open English WordNet 2025 edition) found and fixed two issues in the ontology as originally shipped (commit `004dfddd`):

1. **`has_domain_topic`/`domain_topic` direction was backwards.** The original doc comments and this ontology's fixture assumed "member --has_domain_topic--> domain". Direct inspection of the loaded XML (e.g. `oewn-08458195-n` "law", `oewn-06376048-n` "literature") shows the DOMAIN synset carries `has_domain_topic` to its members, and the MEMBER carries the inverse `domain_topic`. Corrected in `english/ontology.rs`'s `WordnetRelations` doc and accessor docs, and in this module's fixture.
2. **The `exemplifies` claim for Genre was an invented correspondence.** No real `exemplifies`/`is_exemplified_by` edge exists anywhere in the loaded corpus's genre-taxonomy subtree (184 descendants checked) or under "literature"'s 19 real domain-topic members, out of 1639 corpus-wide `exemplifies` edges total. Dropped per `feedback_literature_or_remove`; Genre's WordNet grounding now claims only the `has_domain_topic`/`domain_topic` axis.

## Pending verification

Open items for human review:

- [ ] Cross-check all four primary sources against `docs/papers/references.md`; add entries if absent
- [ ] If any source has an accessible edition, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above
- [ ] `work.rs`'s `WorkRecord`/`ExpressionRecord` fixtures (Homer's Iliad, the Lattimore translation) remain hand-built literals, not backed by a real bibliographic-metadata corpus (e.g. VIAF, OpenLibrary, WorldCat) -- flagged, not addressed, in the 2026-07-21 pass; no readily-available real-data source was integrated.

---

- **Document date:** 2026-07-12; corrected 2026-07-21.
- **How this file is maintained:** initialized alongside the ontology's first commit (turing-benchmark B2); updated by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added, and as re-verification passes correct prior claims.
