# Provisional Closure

**Author:** Ido Samuelson
**License:** CC BY-SA 4.0
**Date:** July 2026
**Lean version:** 4.32.0 (vanilla, no Mathlib dependency)

Three machine-checked Lean 4 files formalizing an alternative posture on formal systems, closure, and incompleteness. Each file is independently useful and independently citable; together they compose into a coherent framework.

The position, briefly: closure is a modeling choice rather than an intrinsic property of systems. Open systems have lossy interfaces by definition. The classical first incompleteness theorem's *semantic* conclusion — "the Gödel sentence is true but unprovable" — depends on a metaphysical commitment (bivalent truth over a completed totality) that its popular presentation hides. The *syntactic* conclusion (a self-referential sentence is not decided by a consistent theory) is available without that commitment and is what proof theory actually establishes.

## The Files

### `ProvisionalClosure.lean` — The Systems Framework

The constitutive file. Defines systems as proposition spaces with provability and negation, extensions as embeddings between systems, and functors as structural maps. Establishes:

- **Openability** (axiom): every system admits an extension deciding any chosen proposition.
- **Incompleteness as signal** (theorem): undecidability marks where a system must open, not where reasoning terminates.
- **No terminal closure** (corollary): systems with any undecided proposition have strictly further-along successors.
- **Structural transfer** (theorem): faithful functors between systems preserve derivation, so structural laws carry along analogies.
- **Infinity trichotomy**: `KnownUnknown` (rule-governed unbounded process) / `UnknownKnown` (unarticulated operative structure) / `UnknownUnknown` (no rule, no articulation). The classical incompleteness argument requires promoting an `UnknownUnknown` to `KnownKnown`, which this framework declines.

Openability is asserted axiomatically because it *is* the position under formalization. A framework that treats some closure as intrinsic and final would reject it. This file makes the choice legible rather than hidden.

### `OpenSystemsLossy.lean` — The Information-Flow Axiom

The information-theoretic dual of `ProvisionalClosure`. Where openability says every closure can extend, this file says every open interface must lose information. Establishes:

- **Lens** structure with all three lens laws (get-put, put-get, put-put).
- **Losslessness** (definition): `get` is injective; every source state is recoverable from its view.
- **Openness** (definition, w.r.t. a view): the view identifies distinct source states.
- **`open_system_has_lossy_lens`** (theorem): openness ⟹ no lossless lens with that view.
- **`open_iff_lens_lossy`** (theorem): the two notions are equivalent. Openness *is* boundary information loss. Not connected to, not implying — identical.
- **`no_equivalence_when_open`** (theorem, categorical form): open systems admit no type equivalence with their environments; adjunctions between them are never equivalences.

This file establishes that the axiom is not a postulate about open systems but a definitional identity. You cannot have an open system with a lossless interface any more than you can have a bachelor who is married. The terms mean opposite things once unfolded.

### `LEMDependency.lean` — The Audit

The critical companion piece. Isolates precisely which metaphysical commitment the classical Gödel argument requires and where it enters. Works with an abstract theory equipped with provability, negation, consistency, and a diagonal fixed-point hypothesis — no arithmetic syntax, no completed ℕ. Establishes:

- **`not_provable_of_diagonal`** (constructive): the syntactic underivability of G follows from the diagonal fixed point and consistency alone. No LEM, no truth predicate, no completed model.
- **`undecided_of_diagonal`** (constructive): full syntactic incompleteness (neither G nor ¬G derivable) from a bidirectional diagonal.
- **`complete_bivalent_truth_gives_lem`** (constructive): a bivalent truth predicate with completeness entails LEM at the sentence level. No external classical axiom required — the LEM is already packed into the semantic hypotheses.
- **`lem_gives_completeness`** (constructive): the converse. Bivalent-truth-with-completeness and sentence-level LEM name the same commitment.
- **`godel_classical`**: takes `T.Provable G ∨ ¬ T.Provable G` as an *explicit hypothesis* rather than tacitly invoking `Classical.em`. The LEM is visible in the signature, not smuggled through a tactic.
- **`syntactic_result_is_free_of_semantic_commitments`**: the constructive theorems demonstrably depend on none of the three classical commitments (truth predicate exists, truth is bivalent, every sentence has a truth value). These are enumerated as an inductive type so the audit is machine-inspectable.

The syntactic result — PA does not decide its Gödel sentence — stands. What does not stand is the scope claim: that this local fact about one class of formal systems tells us anything universal about reasoning or knowledge. The metaphysical inflation happens at the step from `¬ Provable G` to `True_ G`, and this file makes that step visible.

## How They Compose

The three files fit together as follows.

**`ProvisionalClosure` supplies the systems.** A system is a proposition space with derivation. Systems have provisional closures and extend as needed.

**`OpenSystemsLossy` supplies the information-flow constraint.** When a system's closure is extended by opening it to an environment, the resulting interface is lossy. This is not a defect of the extension mechanism; it is what openness means.

**`LEMDependency` supplies the reason the alternative posture is needed.** The classical framework treats incompleteness as a terminal limit and truth as bivalent over a completed totality. This file shows those two moves are the same move, and that declining them costs nothing in terms of syntactic proof theory. What's given up is the metaphysical inflation, which was never part of the theorem's actual content.

The composition: extending a provisional closure to accommodate a previously-undecided proposition either (i) enlarges the closure while preserving self-description (still closed, still lossless internally), or (ii) opens the closure to an environment (now open, interface unavoidably lossy). Incompleteness in the classical sense is what a system experiences when it insists its closure is final; in the framework of these files, the same phenomenon is a signal for one of the two extension modes.

## Verification

No dependencies. Vanilla Lean 4.

```bash
# Install elan (Lean toolchain manager)
curl -sSf https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh | sh -s -- -y

# Set Lean version
elan toolchain install leanprover/lean4:v4.32.0
elan default leanprover/lean4:v4.32.0

# Compile each file (silent output = success)
lean ProvisionalClosure.lean
lean OpenSystemsLossy.lean
lean LEMDependency.lean
```

All three files compile with exit code 0, no warnings, no `sorry`, no `admit`. The `LEMDependency` and `OpenSystemsLossy` files use `Classical.byContradiction` in places where the classical direction of a biconditional is genuinely needed; this is the only classical logic used, and it is invoked explicitly rather than via tactic sugar.

To verify the theorem signatures independently:

```bash
lean -o ProvisionalClosure.olean ProvisionalClosure.lean
lean -o OpenSystemsLossy.olean OpenSystemsLossy.lean
lean -o LEMDependency.olean LEMDependency.lean

cat > verify.lean << 'EOF'
import ProvisionalClosure
import OpenSystemsLossy
import LEMDependency

open ProvisionalClosure OpenSystemsLossy LEMDependency

#check @System.incompleteness_signals_extension
#check @open_iff_lens_lossy
#check @classical_conclusion_given_hypotheses
EOF

LEAN_PATH=. lean verify.lean
```

## Independence and Composition

Each file is self-contained and can be read, cited, or extended independently:

- Formal-methods / proof-theory audience: `LEMDependency.lean` on its own is a clean audit of a specific classical dependency.
- Category-theory / systems-theory audience: `OpenSystemsLossy.lean` on its own establishes the openness/lossiness identity.
- Philosophy-of-mathematics / foundations audience: `ProvisionalClosure.lean` on its own presents the alternative posture.

Reading all three gives the full picture: an alternative framework for reasoning systems in which closure is provisional, boundaries are lossy, and Gödel's theorem is a local fact rather than a universal limit.

## Relation to pr4xis

pr4xis is the operational instance of this framework. Each ontology is a closed system with an explicit operational law. Property verification samples the state space governed by that law — self-description in the strong sense, the system demonstrating its own behavior matches its own stated law. `Analogy<F>` transports structure along faithful functors between ontologies. `PipelineTrace` makes derivation history inspectable rather than certifying the closure consistent from within.

The files here formalize what pr4xis assumes as its architectural premises. pr4xis operates in the region where none of the classical pathologies bite — not because it evades them, but because it declines the configuration that produces them.

## Novelty Statement

The individual philosophical moves have lineage:

- Constructivist / intuitionist critique of LEM over infinite domains (Brouwer, Heyting, Bishop)
- Wittgenstein's objection to Gödel as self-referential rhetoric
- Systems theory with operational laws (Bertalanffy, Ashby, Maturana/Varela)
- Category-theoretic reading of analogy as functor (Lawvere, Lambek)
- Revisable axiom sets (Lakatos, Peirce)
- Reverse mathematics on axiom dependencies (Simpson, Friedman)
- Lawful lenses and their information-theoretic content (Foster, Pierce, others)

The three-way infinity taxonomy (`KnownUnknown` / `UnknownKnown` / `UnknownUnknown`) applied as a diagnostic for auditing the metaphysical commitments of formal systems appears to be new in this specific application. The clean statement of the self-application argument (Gödel's theorem, by its own criterion, fails its own criterion) is at minimum sharply stated. The combination of provisional closure, functorial connection, and lossy-interface axiom as a working architecture for reasoning engines — instantiated in pr4xis — is the load-bearing contribution.

## Citation

If citing this library:

> Samuelson, I. (2026). *Provisional Closure: A Lean 4 formalization of a systems-theoretic alternative to classical incompleteness.* Available at [repository URL].

Individual files can be cited by name for their specific results.
