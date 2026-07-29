# Citings — VerbNet -- Verb-Class Classification Ontology

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms and reader.

## Primary sources

- Kipper, K., Korhonen, A., Ryant, N. & Palmer, M. (2008). "A Large-scale Classification of English Verbs." *Language Resources and Evaluation* 42(1):21-40. VerbNet itself; the source of the loaded class-hierarchy data (`crates/domains/data/verbnet/verbnet-3.3.verbnet`, tag `vn-3.3` of github.com/cu-clear/verbnet).
- Levin, B. (1993). *English Verb Classes and Alternations: A Preliminary Investigation*. University of Chicago Press. The syntactic-alternation diagnostics VerbNet's class hierarchy is built from; the "semantic coherence hypothesis" (class comembership tracks shared meaning components, not specificity) that grounds why the corroboration mechanism is scoped to `Similarity`/`Equivalence`, never `Subsumption`.
- Fellbaum, C. (ed.) (1998). *WordNet: An Electronic Lexical Database*. MIT Press. The WNDB(5WN) sense-key format (`lemma%ss_type:lex_filenum:lex_id[:head_word:head_id]`) VerbNet's `<MEMBER wn="...">` attribute uses — the format `store.rs`'s `oewn_sense_id_for_sense_key` mechanically converts.
- Olsen, M., Dorr, B., & Clark, S. (1997). "Using WordNet to Posit Hierarchical Structure in Levin's Verb Classes." AMTA/SIG-IL Workshop on Interlinguas. Direct evidence VerbNet's own class structure carries no native hypernymy signal — WordNet sense tags had to be imported to impose one.
- Baker, C. F., & Ruppenhofer, J. (2002). "FrameNet's Frames vs. Levin's Verb Classes." Proceedings of BLS 28. The same alternation-class-vs-semantic-hierarchy mismatch, independently confirmed against FrameNet's frame structure.
- Smith, B. et al. (2005). "Relations in biomedical ontologies." *Genome Biology* 6:R46 (OBO Relation Ontology). Source of the `MemberOf` (`member_of`) relation kind (`formal/relations/ontology.rs`) the corroboration mechanism reasons over.
- Tarski, A. (1941). "On the calculus of relations." *Journal of Symbolic Logic* 6. Source of the `Irreflexive` structural property `MemberOfIsIrreflexive` asserts.

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../docs/papers/references.md)
- Source attributions per axiom: `VerbNetCorroborationScopedToSimilarityAndEquivalence` (`crates/chat/src/lib.rs`), `MemberOfIsIrreflexive` (`crates/domains/src/formal/relations/ontology.rs`)
- The full literature-grounded rationale for the `Similarity`/`Equivalence` scoping (including the measured regression it fixes) is in `english/word_sense.rs`'s module doc comment, not repeated here
- Code-level citations: `grep -n 'Kipper\|Levin\|Olsen\|Baker.*Ruppenhofer' *.rs` in this directory and in `crates/chat/src/lib.rs`

## Pending verification

Every entry under **Primary sources** is a short pointer. For each one, confirm that a full citation (Author, Year, Title, DOI/URL) exists in `docs/papers/references.md`. Where no entry exists, add it (or a local PDF under a `papers/` subdirectory) before declaring the ontology citation-complete.

Open items for human review:

- [ ] Cross-check every primary source against `docs/papers/references.md`
- [ ] Confirm the `vn-3.3` tag citation (github.com/cu-clear/verbnet, 2020-07-09) is durable — the repository is community-maintained, not a stable institutional archive
- [ ] If this ontology depends on a paper not yet in the workspace bibliography, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above

---

- **Document date:** 2026-07-13
- **How this file is maintained:** hand-authored (not via the `per-ontology-rollout` skill — this directory's `ontology.rs` is a typed instance-data loader, mirroring `english/ontology.rs`'s shape, not a `pr4xis::ontology!`-macro vocabulary, so the skill's validation step does not apply). Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
