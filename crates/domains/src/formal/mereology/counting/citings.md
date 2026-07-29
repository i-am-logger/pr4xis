# Citings — Counting -- the arithmetic of finite pluralities

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms.

## Primary sources

- Frege 1884: *Die Grundlagen der Arithmetik* (*The Foundations of Arithmetic*) §§55-69 -- a cardinal number is a property of a concept, not of the objects falling under it.
- Gelman & Gallistel 1978: *The Child's Understanding of Number*, Harvard University Press, ch. 7 ("The Counting Model") -- the one-to-one-correspondence, stable-order, and cardinal-principle requirements. (ch. 8, "The Development of the How-To-Count Principles", covers their developmental acquisition, not the requirements themselves.)

## Cross-references

- Source attributions per axiom: see the `Source` column in the `## Axioms` table in [`README.md`](README.md)
- Code-level citations: `grep -n 'Frege\|Gelman' ontology.rs` in this directory
- Composed with (not built from) `MereologyTheory`: see `../wordnet_grounding.rs`, which grounds the same turing-benchmark A4 keystone

## Pending verification

Open items for human review:

- [ ] Cross-check both primary sources against `docs/papers/references.md`; add entries if absent
- [ ] If either source has an accessible edition, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above

---

- **Document date:** 2026-07-12
- **How this file is maintained:** initialized alongside the ontology's first commit (turing-benchmark A4). Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
