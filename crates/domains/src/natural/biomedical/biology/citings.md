# Citings — Biology Ontology

Every published source the biology ontology stands on. Each entry includes the full citation, a DOI/URL where available, and a one-line annotation of which concepts, axioms, or relations the source grounds.

This is the per-ontology bibliography. The workspace-wide bibliography of all sources cited by all ontologies lives at [`docs/papers/references.md`](../../../../../../docs/papers/references.md).

## Primary sources

### Hooper 1956 — *The mucosa of the human esophagus*

- **Citation**: Hooper, C. E. S. (1956). *Cell turnover in epithelial populations*. Journal of Histochemistry & Cytochemistry, 4(6), 531–540.
- **Grounds**: the basal-stem-cell-to-squamous-epithelial differentiation pathway in the esophagus. Specifically the `StemCellDifferentiation` axiom (`ontology.rs:328`) which encodes that basal stem cells and squamous epithelial cells coexist in the squamous epithelium and that proliferation is restricted to the basal layer.
- **Cited at**: `ontology.rs:328` (axiom doc comment)

### Piedrafita et al. 2020 — *A single-progenitor model as the unifying paradigm of epidermal and esophageal epithelial maintenance in mice*

- **Citation**: Piedrafita, G., Kostiou, V., Wabik, A., Colom, B., Fernandez-Antoran, D., Herms, A., Murai, K., Hall, B. A., & Jones, P. H. (2020). *A single-progenitor model as the unifying paradigm of epidermal and esophageal epithelial maintenance in mice*. Nature Communications, 11(1), 1429.
- **DOI**: [10.1038/s41467-020-15258-0](https://doi.org/10.1038/s41467-020-15258-0)
- **Grounds**: the modern formalization of the basal-stem-cell-to-squamous-epithelial differentiation pathway. Cited alongside Hooper 1956 as the empirical basis for the `StemCellDifferentiation` axiom.
- **Cited at**: `ontology.rs:328` (axiom doc comment)

## Supporting sources (Levin bioelectric framework, multi-scale claims)

The biology ontology was authored to compose with the bioelectricity ontology and inherits some of its source framework. The full bioelectricity bibliography lives in the bioelectricity sibling directory; the biology-specific touchpoint is the `MechanosensitivityIsMultiscale` axiom which encodes that mechanosensitive entities exist at both cellular and tissue levels.

- **Levin, M. (2014, 2021, 2024)** — the TAME framework and bioelectric pattern formation work. Grounds the multi-scale framing of mechanosensitivity. Detailed citations live in `../bioelectricity/citings.md` (when that file is created per [#57](https://github.com/i-am-logger/pr4xis/issues/57)).
- **Coste et al. 2010** (Nobel 2021) — Piezo1 and Piezo2 mechanosensitive channels. Grounds the empirical basis for mechanosensitivity as a real biological property. Detailed citation at `../molecular/citings.md` (when created).

## Cross-domain equivalence sources

The cross-domain equivalence axioms (`MacrophageM1CrossDomainEquivalence`, `MacrophageM2CrossDomainEquivalence`, `FibroblastCrossDomainEquivalence`) connect the biology ontology to the immunology ontology by asserting that the same entity name refers to the same biological reality. These axioms are not cited from a single source — they are encodings of the standard immunology consensus that macrophage polarization (M1 vs M2) and fibroblast identity are stable across the two analytical frameworks (anatomical biology vs functional immunology).

- See `../immunology/citings.md` for the immunology-side sources (when created)

## Pending verification

Sources mentioned in the code or in tests that lack full bibliography entries here. Each is a TODO for the human to fill in:

- **`Yang & Bhatt 2022`** — referenced in the original draft of paper 01 (`docs/research/papers/01-categorical-bioelectricity.md`) under the biology row of the domain table, but no full citation has been recorded in this directory or in `docs/papers/references.md`. Verify and add.

---

- **Document date:** 2026-04-14
- **How this file is maintained**: source citations live in code comments in `ontology.rs` and the surrounding files. This file is produced by reading those comments and consolidating them into a structured bibliography. To add a new citation, add a `// Source: ...` or `/// Source: ...` doc comment in the relevant Rust file, then update this file. Future automation via the [`per-ontology-citings`](../../../../../.claude/skills/per-ontology-citings/SKILL.md) skill is planned.
