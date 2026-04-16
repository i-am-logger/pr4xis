# Citings — Explorer -- Self-referential visualization of reasoning traces

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms. Where a full bibliographic entry exists in the workspace-wide [`docs/papers/references.md`](../../../../../../docs/papers/references.md), the short form here is a pointer.

## Primary sources

- Mendez et al. 2023: *Evonne* (EuroVis 2023) — proof-tree visualization
- Srisuchinnawong et al. 2021: *NeuroVis* — neural activation encoding
- Wongsuphasawat et al. 2017: *TensorFlow Graph Visualizer* (IEEE VAST) — dataflow graphs at scale
- Beck et al. 2017: *A Taxonomy and Survey of Dynamic Graph Visualization* (Computer Graphics Forum) — temporal animation
- Smith 1984: *Reflection and Semantics in a Procedural Language* — reflective towers
- W3C PROV-O 2013 — provenance data model

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../docs/papers/references.md)
- Source attributions per axiom: see the `Source` column in the `## Axioms` table in [`README.md`](README.md)
- Code-level citations: `grep -n 'Source:\|Reference:' *.rs` in this directory

## Pending verification

Every entry under **Primary sources** is a short pointer. For each one, confirm that a full citation (Author, Year, Title, DOI/URL) exists in `docs/papers/references.md`. Where no entry exists, add it (or a local PDF under a `papers/` subdirectory) before declaring the ontology citation-complete.

Open items for human review:

- [ ] Cross-check every primary source against `docs/papers/references.md`
- [ ] Add code-comment-level citations (`// Source: ...`) to axioms that currently lack attribution
- [ ] If this ontology depends on a paper not yet in the workspace bibliography, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above

---

- **Document date:** 2026-04-15
- **How this file is maintained:** initialized by the per-ontology rollout (issue #57 / #173) from `README.md`'s *Key references* section. Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
