# Citings — FrameNet -- Frame-Semantic Ontology

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its reader and regen module.

## Primary sources

- Baker, C. F., Fillmore, C. J. & Lowe, J. B. (1998). "The Berkeley FrameNet Project." *Proceedings of COLING-ACL 1998*. FrameNet itself; the source of the loaded lexical-unit/frame-relation data (`crates/domains/data/framenet/framenet-1.7.tsv`, extracted from the official NLTK-mirrored `framenet_v17.zip`).
- Ruppenhofer, J., Ellsworth, M., Petruck, M. R. L., Johnson, C. R. & Scheffczyk, J. (2016). *FrameNet II: Extended Theory and Practice*. ICSI. The 9 frame-to-frame relation types (Inheritance, Using, Subframe, Perspective_on, Causative_of, Inchoative_of, Precedes, Metaphor, See_also), confirmed 2026-07-13 against the real `frRelation.xml` `name=` attributes in the fetched release, not assumed from documentation alone.
- NLTK `nltk_data` package index (<https://raw.githubusercontent.com/nltk/nltk_data/gh-pages/index.xml>, entry `id="framenet_v17"`, author Collin F. Baker). Source of the CC BY 3.0 Unported data license and the byte-reproducible download checksums (SHA256 `22f6aad6fb799ba4dbed0440714e1118442ad7d7345351de37428581284f471c`, MD5 `aaef1cfdcf37000cf2a5c562407fbddb`), reproduced in `crates/domains/data/framenet/framenet-LICENSE.txt`.
- Ferrández, O. et al. (2010). "Aligning FrameNet and WordNet based on Semantic Neighborhoods." *LREC 2010*. Independent confirmation that no native FrameNet↔WordNet link exists in the source data — grounds why this project's `FrameNetStore` resolves lemma+POS live, at query time, rather than via a precomputed sense-key crosswalk (the pattern VerbNet's `wn=` attribute makes possible but FrameNet's data does not).
- PKWARE Inc. (2022). *APPNOTE.TXT — .ZIP File Format Specification*, version 6.3.10. §4.3.7/§4.3.12/§4.3.16 — the local-file-header, central-directory, and End-Of-Central-Directory record formats `regenerate.rs`'s hand-rolled multi-entry ZIP reader implements (no system `unzip` tool was available in the build environment; mirrors the byte-parsing this codebase's own `applied::data_provisioning::fetch` module already applies for single-file ZIP extraction).

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../docs/papers/references.md)
- The full literature-grounded rationale for scoping cross-source corroboration by relation kind (including VerbNet's measured regression that established the discipline this ontology's own consultation follows) is in `english/word_sense.rs`'s module doc comment, not repeated here
- Code-level citations: `grep -n 'Baker\|Fillmore\|Ruppenhofer\|FrameNet' *.rs` in this directory and in `crates/chat/src/lib.rs`

## Pending verification

Every entry under **Primary sources** is a short pointer. For each one, confirm that a full citation (Author, Year, Title, DOI/URL) exists in `docs/papers/references.md`. Where no entry exists, add it (or a local PDF under a `papers/` subdirectory) before declaring the ontology citation-complete.

Open items for human review:

- [ ] Cross-check every primary source against `docs/papers/references.md`
- [ ] Confirm the NLTK mirror URL (`raw.githubusercontent.com/nltk/nltk_data/gh-pages/packages/corpora/framenet_v17.zip`) remains available at re-fetch time — it is a GitHub-hosted redistribution of ICSI's own release, not the ICSI site directly (which was not fetchable during this session's research, being a JS-rendered request-gated page)
- [ ] If this ontology depends on a paper not yet in the workspace bibliography, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above

---

- **Document date:** 2026-07-13
- **How this file is maintained:** hand-authored (not via the `per-ontology-rollout` skill — this directory's `ontology.rs` is a typed instance-data loader, mirroring `conceptnet/ontology.rs`'s shape, not a `pr4xis::ontology!`-macro vocabulary, so the skill's validation step does not apply). Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
