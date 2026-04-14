# Citings — Dialogue -- Conversation as event-driven concurrent system

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms. Where a full bibliographic entry exists in the workspace-wide [`docs/papers/references.md`](../../../../../../docs/papers/references.md), the short form here is a pointer.

## Primary sources

- Austin 1962: *How to Do Things with Words* (speech acts)
- Searle 1969: *Speech Acts* (illocutionary force)
- Traum 1994: *A Computational Theory of Grounding*
- Clark 1996: *Using Language* (grounding, common ground)
- Stalnaker 2002: *Common Ground* (context set, assertion)
- Ginzburg 2012: *The Interactive Stance* (KoS, Questions Under Discussion)
- Levelt 1989: *Speaking* (Conceptualizer → preverbal message)
- Grice 1975: *Logic and Conversation* (cooperative principle)
- Schegloff, Jefferson & Sacks 1977: *The Preference for Self-Correction* (repair)
- Jurafsky & Martin: *Speech and Language Processing* (dialogue acts)

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
