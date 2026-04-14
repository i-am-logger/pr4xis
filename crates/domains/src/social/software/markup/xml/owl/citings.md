# Citings — OWL -- W3C OWL 2 Web Ontology Language Metamodel

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms. Where a full bibliographic entry exists in the workspace-wide [`docs/papers/references.md`](../../../../../../../../docs/papers/references.md), the short form here is a pointer.

## Primary sources

- W3C OWL 2 Structural Specification and Functional-Style Syntax (2012)
- W3C OWL 2 Direct Semantics (2012) — SROIQ model theory
- Baader et al. 2003: *The Description Logic Handbook*
- Horrocks, Patel-Schneider & van Harmelen 2003: *From SHIQ and RDF to OWL*

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../../../docs/papers/references.md)
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
