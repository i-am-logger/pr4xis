# Citings — Diagnostics -- The universal diagnostic cycle

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms. Where a full bibliographic entry exists in the workspace-wide [`docs/papers/references.md`](../../../../../../docs/papers/references.md), the short form here is a pointer.

## Primary sources

- Reiter 1987: *A Theory of Diagnosis from First Principles* (minimal consistent diagnosis)
- Gertler 1998: *Fault Detection and Diagnosis in Engineering Systems* (FDI residuals)
- ISO 13374:2003 — Condition monitoring (six-layer processing)
- Kephart & Chess 2003: *The Vision of Autonomic Computing* (MAPE-K)
- Kalman 1960: *On the General Theory of Control Systems* (observability)
- Conant & Ashby 1970: *Every Good Regulator Must Be a Model*
- Smith 1982: *Reflection and Semantics in a Procedural Language*
- Maes 1987: *Computational Reflection*

- **Local PDF:** [`papers/conant-ashby-good-regulator-1970.pdf`](papers/conant-ashby-good-regulator-1970.pdf) — Conant & Ashby 1970

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

- **Document date:** 2026-04-14
- **How this file is maintained:** auto-initialized by the per-ontology rollout (issue #57) from `README.md`'s *Key references* section. Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
