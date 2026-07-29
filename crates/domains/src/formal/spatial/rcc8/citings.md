# Citings — RCC-8 -- Region Connection Calculus, the 8 base spatial relations

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms.

## Primary sources

- Randell, Cui & Cohn 1992: *A Spatial Logic Based on Regions and Connection*, KR'92 -- §3, Table 1: the 8 base relations (DC, EC, PO, TPP, NTPP, TPPi, NTPPi, EQ) and their JEPD (jointly-exhaustive, pairwise-disjoint) property, grounding all 3 axioms.
- Cohn, Bennett, Gooday & Gotts 1997: *Qualitative Spatial Representation and Reasoning with the Region Connection Calculus*, GeoInformatica 1(3) -- the RCC-5 coarsening (ProperPart/ProperPartInverse) TPP/NTPP and TPPi/NTPPi refine.

## Cross-references

- Source attributions per axiom: see the `Source` column in the `## Axioms` table in [`README.md`](README.md)
- Code-level citations: `grep -n 'Randell\|Cohn' ontology.rs interval.rs` in this directory
- Composed with (not built from): `../../mereology/counting/` -- the same function-call-composition precedent (no forced `Functor`)
- `natural::geography` (Turing-benchmark B3) grounds its `contains`/`borders` axioms in this module's LITERATURE (Randell, Cui & Cohn 1992 \u{00a7}3's algebraic EC/TPP/NTPP definitions), checked directly over the real loaded GeoNames gazetteer -- not a function-call composition against `classify`/`connected` (that geometric realization has no polygon/interval data to consume for real countries)

## Pending verification

Open items for human review:

- [ ] Cross-check both primary sources against `docs/papers/references.md`; add entries if absent
- [ ] If either source has an accessible edition, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above

---

- **Document date:** 2026-07-12
- **How this file is maintained:** initialized alongside the ontology's first commit (turing-benchmark B3). Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
