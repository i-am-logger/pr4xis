# Citings — PeanoArithmetic -- recursive addition and multiplication over the naturals

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms.

## Primary sources

- Enderton 1977: *Elements of Set Theory*, Academic Press, ch. 4 "Natural Numbers" -- Theorem 4D (p.71, the Peano system ⟨ω, σ, 0⟩), Theorem 4I (p.79, addition), Theorem 4J (p.80, multiplication). This is the actual source of the 0-based Zero/Successor/Addition/Multiplication content in `ontology.rs`, machine-verified against the primary text (pdftotext extraction).
- Hurford 1975: *The Linguistic Theory of Numerals*, Cambridge University Press, ch. 2 -- the basic (non-composed) numeral inventory.

## Historical framing only (not the source of the encoded formulas)

- Peano 1889: *Arithmetices Principia, Nova Methodo Exposita* -- the origin of the name "Peano Arithmetic". Peano's own axiomatization is 1-based (0 is not a natural number), so it cannot ground this ontology's 0-based content.
- Landau 1930: *Grundlagen der Analysis* -- likewise 1-based; verified against the primary text that Landau's actual Axiom 1 is "1 ist eine natürliche Zahl" and his addition/multiplication definitions are §2 Satz 4 / §4 Satz 28 (not §1 Satz 1/4 as an earlier draft of this ontology mistakenly cited), with base cases x+1=x' and x·1=x. Kept here only because the theory's modern name derives from Peano/Landau's line of work, not because either source is cited for a specific formula.

## Cross-references

- The `Number` inclusion-chain ontology (`../ontology.rs`, i.e. `formal/math/ontology.rs`) cites Landau (1930) for the N⊂Z⊂Q⊂R⊂C chain -- that citation is independent of this correction and untouched.
- Source attributions per axiom: see the `Source` column in the `## Axioms` table in [`README.md`](README.md)
- Code-level citations: `grep -n 'Enderton\|Peano\|Landau\|Hurford' ontology.rs numeral.rs` in this directory

## Pending verification

Open items for human review:

- [ ] Cross-check the Enderton, Peano, and Landau citations against `docs/papers/references.md`; add entries if absent
- [ ] If an accessible edition of any source exists, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above

---

- **Document date:** 2026-07-12
- **How this file is maintained:** initialized alongside the ontology's first commit (turing-benchmark A3). Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
