# Citings — Theming -- Color Scheme Ontology

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms. Where a full bibliographic entry exists in the workspace-wide [`docs/papers/references.md`](../../../../../docs/papers/references.md), the short form here is a pointer.

## Primary sources

- Base16 styling spec: tinted-theming/home/styling.md
- Base24 styling spec: tinted-theming/base24/styling.md
- CIE 1931 colorimetry / sRGB relative luminance
- WCAG 2.1 SC 1.4.3: contrast requirements for accessibility
- ECMA-48 5th Ed 1991: SGR parameters 30-37, 90-97 for ANSI colors
- Porter & Duff 1984: *Compositing Digital Images*
- Bertin 1967: *Semiology of Graphics* (visual variables)
- Cleveland & McGill 1984: *Graphical Perception* (perceptual task ranking)
- Harel 1987: *Statecharts* (mode graphs, parallel regions)

- **Local PDF:** [`papers/cleveland-mcgill-graphical-perception-1984.pdf`](papers/cleveland-mcgill-graphical-perception-1984.pdf) — Cleveland & McGill 1984
- **Local PDF:** [`papers/harel-statecharts-1987.pdf`](papers/harel-statecharts-1987.pdf) — Harel 1987
- **Local PDF:** [`papers/porter-duff-compositing-1984.pdf`](papers/porter-duff-compositing-1984.pdf) — Porter & Duff 1984

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../docs/papers/references.md)
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
