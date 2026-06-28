# The Constitution: research grounding

This is the *why* behind pr4xis's guarantees — the literature they rest on and
the enforcement ladder that turns each from a slogan into an invariant the
system cannot violate and remain itself. The user-facing description of what
exists is [`docs/understand/constitution.md`](../understand/constitution.md);
this document records the reasoning and sources that selected and grounded the
values.

Two kinds of evidence appear below, and they are not equal:

- **Runnable backing** — an axiom or universal property *in the codebase* whose
  failure turns a test red. This is the hard guarantee; each is named so you can
  run it.
- **Literature grounding** — the established work a value is modeled on. These
  are real, well-known sources cited from the record, not loaded into a checker;
  they justify the *design*, not the runtime behaviour.

## The enforcement ladder

A guarantee can be held five ways, weakest to strongest:

1. **Slogan** — a word on a landing page. Holds until inconvenient.
2. **A count of tests** — measures the suite, not the system.
3. **A census of tests against the code they cover** — better, but still a
   sample of inputs.
4. **A universal axiom** — one check folded over the *whole* base (every axiom,
   every input in a generated domain). No sampling.
5. **A structural property the compiler refuses to let you violate** — the
   strongest: the bad state cannot be expressed.

The work of the constitution is to push each value up this ladder, from *tested*
toward *enforced*. The ladder mirrors the trajectory the verification literature
took for specifications: from tests, to property-based testing, to refinement
types (Rondon, Kawaguchi & Jhala, *Liquid Types*, PLDI 2008), where a property
becomes a compile-checked part of the type. Rung 5 is the refinement-type ideal;
`Verifiable` already sits there (an uncited axiom is a compile error).

## The answer-guarantees and their grounding

Five values are **answer-guarantees** — properties of a single answer. The sixth,
`Extensible`, is **second-order**: the property that the five are preserved under
composition.

| Guarantee | Runnable backing (in code) | Literature grounding | Rung |
|---|---|---|---|
| **Verifiable** | `Axiom::citation()` is required — an uncited axiom does not compile | Peroni & Shotton, *FaBiO and CiTO* (2012): citation as first-class, typed provenance. Cf. W3C PROV / Verifiable Credentials — provenance attests *origin*, not truth | structural |
| **Deterministic** | no-`std`/no-IO/no-clock reasoning core + universal round-trip & canonical-form properties | Knuth, *TAOCP* Vol. 1 §1.1: *definiteness* — each step precisely defined, same input → same output | near-structural |
| **Explainable** | `EveryAxiomCarriesItsExplanation` — every verdict carries name + claim + citation | Martin-Löf, *Intuitionistic Type Theory* (1984): the proof object *is* the explanation (propositions-as-types). von Foerster, *Observing Systems* (1981): the eigenform — a system describing its own structure as a fixed point | universal axiom |
| **Honest** | totality fuzz over the entire input boundary — ∀ input, return `Ok`/`Err`, never panic | King, *Parse, don't validate* (2019): make illegal states unrepresentable at the boundary. Grice, *Logic and Conversation* (1975), maxim of Quality: do not assert what you lack evidence for | universal property |
| **Consistent** | `OntologyBaseIsConsistent` — folds the whole axiom registry; the corpus derives no contradiction | Gentzen, *Die Widerspruchsfreiheit der reinen Zahlentheorie* (1936): a theory is consistent iff it proves no `⊥` | universal axiom |
| **Extensible** *(composition)* | `ExtensiblePreservesEveryGuarantee` + the workspace functor-law checks | Spivak, *Category Theory for the Sciences* (2014): functorial composition — proven parts assemble into a proven whole. Gruber (1995), ontology criteria: *extendibility* and *coherence* | universal property |

`Honest` is the keystone of the five: Verifiable, Deterministic, Explainable and
Consistent are credible only because the system can refuse — leave a claim
ungrounded rather than confabulate. Lakatos, *Proofs and Refutations* (1976):
refutation, the capacity to say "this fails", is constitutive of knowledge.

Re-derive the partition and the backings:

```
cargo test -p pr4xis-domains --lib -- constitution_coverage -- --nocapture
bash scripts/constitution-gate.sh pr4xis-domains
```

## Why these, and why `Consistent`

The selection criterion is **falsifiability of a single answer**: a value earns
a place if a concrete failure of one answer would witness its violation. That
admits exactly the five answer-guarantees and excludes vaguer virtues (e.g.
"helpful", "aligned") that are not properties of an answer.

- **Answer vs. composition.** A property of *one answer* (is it cited? does it
  reproduce? can it be refused?) is first-order. A property of *how answers
  combine* (does attaching a new ontology preserve the others?) is second-order.
  Conflating them is the error that made `Extensible` look weak as a peer; modeled
  as a meta-property over the five, with `Preserves` edges to each, it is exactly
  as strong as the functor laws that discharge it.
- **`Consistent` is the most-warranted addition.** The formal-methods and
  ontology-quality literature ranks freedom-from-contradiction as the most
  foundational property of a knowledge base (Gentzen; Gruber's *coherence*), yet
  a product framing tends to omit it because it is invisible when nothing breaks.
  It was backed by a runnable axiom (`OntologyBaseIsConsistent`) the day it was
  named — the corpus is checked to derive no `⊥`, not merely asserted to.
- **Uneven grounding is expected, not a flaw.** `Honest`-as-totality and the
  proof-as-explanation reading of `Explainable` are the most novel framings here;
  `Verifiable` and `Deterministic` restate long-settled engineering practice. The
  per-test tag plus the completeness gate are the hard guarantee; the per-value
  percentages are a directional diagnostic, judgment-laden at the margins.

## The generative claim

Backing a value, rather than stating it, is *generative*: enforcing `Honest` as
totality over all input surfaced — and forced fixes for — a series of real,
reachable denial-of-service and overflow bugs that the example tests never
caught (a multi-petabyte allocation from a forged length prefix; unbounded
parser recursion; integer-overflow in exact-rational arithmetic; a char-boundary
panic; an unbounded-chart resource exhaustion). That is the difference between
declaring a guarantee and enforcing one: enforcement is constructive — it shows
you exactly where the system is not yet honest.

## References

- Alpern, B. & Schneider, F. B. (1987). *Recognizing safety and liveness.*
- Gentzen, G. (1936). *Die Widerspruchsfreiheit der reinen Zahlentheorie.*
- Grice, H. P. (1975). *Logic and Conversation.*
- Gruber, T. (1995). *Toward principles for the design of ontologies used for
  knowledge sharing.*
- King, A. (2019). *Parse, don't validate.*
- Knuth, D. (1997). *The Art of Computer Programming*, Vol. 1, §1.1.
- Lakatos, I. (1976). *Proofs and Refutations.*
- Martin-Löf, P. (1984). *Intuitionistic Type Theory.*
- Peroni, S. & Shotton, D. (2012). *FaBiO and CiTO.*
- Rondon, P., Kawaguchi, M. & Jhala, R. (2008). *Liquid Types.*
- Spivak, D. (2014). *Category Theory for the Sciences.*
- von Foerster, H. (1981). *Observing Systems.*
