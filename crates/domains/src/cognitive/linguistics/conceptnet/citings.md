# Citings — ConceptNet -- Commonsense Association Ontology

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its reader and regen module.

## Primary sources

- Speer, R., Chin, J. & Havasi, C. (2017). "ConceptNet 5.5: An Open Multilingual Graph of General Knowledge." *Proceedings of the AAAI Conference on Artificial Intelligence* 31(1). ConceptNet itself; the source of the loaded assertion data (`crates/domains/data/conceptnet/conceptnet-5.7.0.assoc`, filtered from the official `conceptnet-assertions-5.7.0.csv.gz` release).
- ConceptNet Wiki, "Relations" (<https://github.com/commonsense/conceptnet5/wiki/Relations>). The 34 relation types the English portion of ConceptNet 5.7.0 carries, confirmed 2026-07-13 — the inventory `ConceptNetEdge::relation` carries through as provenance, uninterpreted.
- ConceptNet Wiki, "Copying and sharing ConceptNet" (<https://github.com/commonsense/conceptnet5/wiki/Copying-and-sharing-ConceptNet>) and `commonsense/conceptnet5` `LICENSE.txt` + `DATA-CREDITS.md`. Source of the CC BY-SA 4.0 data license and suggested attribution text, reproduced in `crates/domains/data/conceptnet/conceptnet-LICENSE.txt`.
- Fellbaum, C. (ed.) (1998). *WordNet: An Electronic Lexical Database*. MIT Press. The lemma set the WordNet-crosswalk filter (`regenerate.rs`) checks ConceptNet's `/c/en/…` node tokens against.

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../docs/papers/references.md)
- The full literature-grounded rationale for scoping cross-source corroboration by relation kind (including VerbNet's measured regression that established the discipline this ontology's own consultation follows) is in `english/word_sense.rs`'s module doc comment, not repeated here
- Code-level citations: `grep -n 'Speer\|Havasi\|ConceptNet' *.rs` in this directory and in `crates/chat/src/lib.rs`

## Pending verification

Every entry under **Primary sources** is a short pointer. For each one, confirm that a full citation (Author, Year, Title, DOI/URL) exists in `docs/papers/references.md`. Where no entry exists, add it (or a local PDF under a `papers/` subdirectory) before declaring the ontology citation-complete.

Open items for human review:

- [ ] Cross-check every primary source against `docs/papers/references.md`
- [ ] Confirm the S3 download URL (`s3.amazonaws.com/conceptnet/downloads/2019/edges/conceptnet-assertions-5.7.0.csv.gz`) remains available at re-fetch time — it is a versioned object, not a git tag, so its long-term durability has a different failure mode than VerbNet's tagged-release provenance
- [ ] If this ontology depends on a paper not yet in the workspace bibliography, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above

---

- **Document date:** 2026-07-13
- **How this file is maintained:** hand-authored (not via the `per-ontology-rollout` skill — this directory's `ontology.rs` is a typed instance-data loader, mirroring `verbnet/ontology.rs`'s shape, not a `pr4xis::ontology!`-macro vocabulary, so the skill's validation step does not apply). Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
