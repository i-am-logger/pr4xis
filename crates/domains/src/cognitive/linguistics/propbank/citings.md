# Citings — PropBank -- Predicate Argument-Structure Lexicon

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its ontology, reader, store, and regen modules.

## Primary sources

- Palmer, M., Gildea, D. & Kingsbury, P. (2005). "The Proposition Bank: An Annotated Corpus of Semantic Roles." *Computational Linguistics* 31(1):71-106. PropBank itself; confirmed via ACL Anthology `J05-1004` and MIT Press Direct — the title is "An Annotated Corpus of Semantic Roles" (not "the" — correcting an earlier internal paraphrase during this build's research phase).
- Bonial, C., Bonn, J., Conger, K., Hwang, J. & Palmer, M. (2014). "PropBank: Semantics of New Predicate Types." *LREC 2014*. Cited per propbank.github.io's own request for frame-file consumers, alongside the original 2005 paper.
- `propbank/propbank-frames` GitHub repository (<https://github.com/propbank/propbank-frames>), tag `v3.4.0`, commit `4087fa9ab5c40907c34ff91a56acc2cab1670145` (independently verified 2026-07-13 via `gh api repos/propbank/propbank-frames/tags`). Source of the loaded `frames/*.xml` frame-file collection and the `frame-schema.dtd`/`dtds/v3.4/frameset.dtd` shape `ontology.rs`'s module doc and `reader.rs`'s parsing both reference.
- `propbank/propbank-frames` repository license (confirmed 2026-07-13 via `gh api repos/propbank/propbank-frames` → `license.spdx_id: "CC-BY-SA-4.0"`, and the repo's own `LICENSE` file, the standard Creative Commons Attribution-ShareAlike 4.0 International legal text). Reproduced in `crates/domains/data/propbank/propbank-LICENSE.txt`.

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../docs/papers/references.md)
- The full literature-grounded rationale for scoping cross-source corroboration (including VerbNet's measured regression that established the discipline every source's own consultation follows) is in `english/word_sense.rs`'s module doc comment, not repeated here.
- The cross-POS scoping design (`shares_roleset` requiring a POS mismatch between the two compared concepts) is grounded in this build's own prevalence research over the real frame-file corpus, not a separate cited paper — see `store.rs`'s module doc for the exact rationale.
- The generic directory-archive codec this ontology's collection rides (`decoders::file_collection`) is VerbNet's own precedent — see `cognitive::linguistics::verbnet`'s citings for Kipper, Korhonen, Ryant & Palmer (2008) and Dolstra (2006), the citations grounding that codec itself (not re-cited here, since this ontology contributes no new claim about the codec).
- Code-level citations: `grep -n 'Palmer\|Gildea\|Kingsbury\|Bonial\|PropBank' *.rs` in this directory and in `crates/chat/src/lib.rs`.

## Pending verification

Every entry under **Primary sources** is a short pointer. For each one, confirm that a full citation (Author, Year, Title, DOI/URL) exists in `docs/papers/references.md`. Where no entry exists, add it (or a local PDF under a `papers/` subdirectory) before declaring the ontology citation-complete.

Open items for human review:

- [ ] Cross-check every primary source against `docs/papers/references.md`
- [ ] Confirm the `propbank/propbank-frames` `v3.4.0` tag (commit `4087fa9ab5c40907c34ff91a56acc2cab1670145`) remains resolvable at re-fetch time — unlike SUMO (no tagged release), this source HAS a real tag, so a future `pr4xis update propbank --lock` should re-resolve cleanly against it
- [ ] If this ontology depends on a paper not yet in the workspace bibliography, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above
- [ ] The five undocumented DTD `pos` codes (`l`/`x`/`m`/`d`/`f`) have no cited literature naming their meaning (the DTD's own comment explicitly marks three of the five "(MISSING IDK)"); `l` is empirically observable as a light-verb marker (e.g. `trade.xml`'s `pos="l"` on `make_trade`) but this is an empirical observation from the fetched data, not a citable claim — flag for a literature check if propbank.github.io or a future release ever documents them

---

- **Document date:** 2026-07-13
- **How this file is maintained:** hand-authored (not via the `per-ontology-rollout` skill — this directory's `ontology.rs` is a typed instance-data loader, mirroring `verbnet/ontology.rs`'s shape, not a `pr4xis::ontology!`-macro vocabulary, so the skill's validation step does not apply). Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
