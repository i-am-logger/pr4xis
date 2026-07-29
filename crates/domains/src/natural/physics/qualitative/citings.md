# Citings — Qualitative Process Theory -- Commonsense physical reasoning without numeric simulation

Every published source this ontology stands on. Entries below are drawn from the ontology's [README.md](README.md) and the doc comments on its axioms.

## Primary sources

- Forbus 1984: *Qualitative Process Theory*, Artificial Intelligence 24(1-3) -- processes, quantities, preconditions, influences (§2); the sign-based derivative abstraction (§2.1, §2.3) that grounds `ProcessActiveRequiresAllPreconditions` and `ActiveProcessInfluencesQuantityDerivative`.
- Hayes 1979: *The Naive Physics Manifesto*, in Michie (ed.) *Expert Systems in the Micro-Electronic Age* -- the commonsense-ontology program; the support-or-falls principle grounding `UnsupportedIndividualsFall`.
- Hayes 1985: *Naive Physics I: Ontology for Liquids*, in Hobbs & Moore (eds.) *Formal Theories of the Commonsense World* §3 -- the container/content size constraint grounding `ContainerSizeAtLeastContentSize`.
- de Kleer & Brown 1984: *A Qualitative Physics Based on Confluences*, Artificial Intelligence 24(1-3) -- envisioning, process theory's sibling approach; cited for the shared qualitative-state framing (this ontology does not build a full envisionment graph -- honestly scoped to the direct, unopposed influence case).
- Levesque, Davis & Morgenstern 2012: *The Winograd Schema Challenge*, KR 2012 -- the commonsense pronoun-resolution task `too_big`/`too_small` answer directly.
- Sakaguchi, Bras, Bhagavatula & Choi 2020: *WinoGrande*, AAAI 2020 -- the "twin sentence" (adjective-swap) methodology `too_big`/`too_small` realize.

## `mereology_grounding.rs` sources

- Casati & Varzi 1999: *Parts and Places: The Structures of Spatial Representation*, MIT Press -- Ch. 2 "Parthood Structures" (the parthood theory `formal::mereology::MereologyTheory` grounds in) vs. Ch. 6 "Modes of Location" (spatial location, formalized separately via `Functionality`/`Conditional Reflexivity`, p. 121) -- the structural separation grounding `NoQualitativeProcessConceptGroundsInMereology`'s claim that `Containment` is a location relation, not parthood.
- Gilmore, Calosi & Costa 2013 (rev. 2024): *Location and Mereology*, Stanford Encyclopedia of Philosophy, §2.2.2 -- secondary-source confirmation of the Casati & Varzi (1999: 121) `Functionality`/`Conditional Reflexivity` principles for exact location as a primitive independent of parthood.
- Hayes 1979 (again, this time for `Support`): the naive-physics support relation is a physical/causal primitive (contact plus gravity) with no part-whole content, grounding the negative half of `NoQualitativeProcessConceptGroundsInMereology` and `HayesContainmentAndSupportStayUngroundedWhenExercised` for `Support`.
- Davis 1990: *Representations of Commonsense Knowledge*, Morgan Kaufmann, Ch. 7 -- further formalization of Hayes's support axioms, cited alongside Hayes (1979) in `containment.rs`'s own `falls_without_support` doc comment.

## Cross-references

- Source attributions per axiom: see the `Source` column in the `## Axioms` table in [`README.md`](README.md)
- Code-level citations: `grep -n 'Forbus\|Hayes\|de Kleer\|Levesque\|Sakaguchi' ontology.rs process.rs containment.rs` in this directory
- Sibling quantitative ontologies in the same `natural::physics` branch: `../ontology.rs` (the physical laws), `../kinematics/` (motion equations) -- this ontology is the commonsense complement, not built from either.
- The cross-domain bridge precedent this ontology's `mereology_grounding.rs` mirrors: `formal::mereology::wordnet_grounding` -- a partial `Option`-returning classifier rather than a forced total `Functor`.

## Pending verification

Open items for human review:

- [ ] Cross-check all six primary sources plus the `mereology_grounding.rs` sources against `docs/papers/references.md`; add entries if absent
- [ ] If any source has an accessible edition, move/copy the PDF into a local `papers/` subdirectory and link it from the primary source line above

---

- **Document date:** 2026-07-21 (updated: `mereology_grounding.rs` sources added)
- **How this file is maintained:** initialized alongside the ontology's first commit (turing-benchmark B1). Update by hand as code-comment citations, local PDFs, and `docs/papers/references.md` entries are added.
