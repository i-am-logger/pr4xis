# The Constitution

pr4xis holds five properties about its own reasoning. They are not features offered to a user and not promises that can be relaxed under pressure — they are the conditions under which a computation counts as pr4xis reasoning at all. A build that violates one of them fails its own test suite.

Each guarantee is stated three ways: the **promise** it makes, the **enforcer** in code that makes it true, the **violation** that would break it. Each ends in a command that re-derives it. Nothing here is asserted; it is checked at test time.

## Verifiable

- **Promise** — every claim carries its source. Nothing is asserted without a citation back to an authoritative origin.
- **Enforcer** — `Axiom::citation()` is required: no default, no `Option`. An axiom without a citation is a compile error. Domain vocabulary is loaded from cited sources (`praxis.toml` + a `praxis.lock` SHA256), never hand-written into the code.
- **Violation** — a hardcoded list, a regex over the input, a weight with no axiom behind it.

```
cargo test -p pr4xis-domains --lib -- citation
```

A statistical model cannot make this promise: a citation it emits is itself a prediction, with the same ground-truth status as any other token it produces.

## Deterministic

- **Promise** — the same input produces the same output, byte for byte, on every run and across versions.
- **Enforcer** — transformations are lawful morphisms, and the serialized substrate round-trips byte-exactly. There is no debug-only path and no profile-conditional behavior, so what is tested is what runs.
- **Violation** — a lossy projection, a `cfg`-gated shortcut, any branch whose result depends on profile, temperature, or seed.

```
cargo test -p pr4xis-domains --lib -- byte_exact
```

A statistical model is stochastic by construction; its output depends on sampling and drifts between versions.

## Explainable

- **Promise** — the system can describe its own structure, and the reasoning path *is* the answer rather than a story told after the fact.
- **Enforcer** — self-description is a fixed point: the description pr4xis gives of itself, fed back in, reproduces itself (the eigenform). Nothing in the system lives outside the ontology it can report.
- **Violation** — an unregistered concept, an unexaminable branch, an explanation reconstructed separately from the computation it claims to describe.

```
cargo test -p pr4xis-domains --lib -- eigenform
```

A statistical model's explanations are post-hoc and need not be faithful to the computation that produced the answer.

## Honest

- **Promise** — what it cannot ground, it leaves ungrounded. It stops rather than confabulate.
- **Enforcer** — an input with no grounding does not get an invented binding; it is left ungrounded, and a derivation that depends on it does not proceed. End to end, the engine answers from a loaded gloss when it has one and abstains when it does not.
- **Violation** — filling a gap with a plausible guess; answering past the edge of what is grounded.

```
cargo test -p pr4xis-chat -- abstain
```

The schema-level guarantee — that an unknown word is left ungrounded rather than bound to a guess — is checked alongside it with `cargo test -p pr4xis-domains --lib -- ungrounded`.

A statistical model has no reliable internal signal for "I am making this up," and always has an output.

## Extensible

- **Promise** — new knowledge plugs in and the other four guarantees still hold. Adding an ontology does not silently degrade the rest.
- **Enforcer** — ontologies compose by functors whose laws are checked; an integration that does not preserve structure fails the law test instead of merging.
- **Violation** — a merge that drops or distorts structure; an "extension" that is really a rewrite of what it touches.

```
cargo test -p pr4xis-domains --lib -- functor_laws
```

A statistical model extends by fine-tuning, which trades one capability for another without measuring the loss.

## Honest is the keystone

The five are not five peers. The other four are credible only because the system can stop.

Verifiable means nothing if, when it cannot verify, it answers anyway. Deterministic, Explainable, and Extensible each assume the system will decline rather than fabricate at the boundary. Remove the ability to refuse and the other four become preferences that hold until they are inconvenient. Keep it, and they become invariants. The defining act is the refusal: the point at which there is no grounded answer and pr4xis says so.

## Two readings of the same five

The guarantees above are the product reading — what someone relying on pr4xis gets. The same five are also the engineering invariants the substrate is built on. They are one set seen from two sides.

| Product guarantee | Engineering invariant |
|---|---|
| Verifiable | Groundedness — knowledge loaded from cited sources, axioms carry citations |
| Deterministic | Lawful morphisms + profile-invariance — no behavior that disappears between builds |
| Explainable | Self-description — total ontological coverage, the eigenform fixed point |
| Honest | The refusal clause — no grounded belief, no answer |
| Extensible | Composability-closure — law-checked functors, gap analysis |

## Self-binding

These five are not documentation about pr4xis written alongside it. They are checked by the tests above, which run in the same suite as everything else. The constitution is part of the substrate it governs: a change that breaks Verifiable or Deterministic does not produce a system that quietly stops meaning what it says — it produces a red test.

## The suite classifies itself

The constitution is not asserted *about* the test suite from outside — the suite declares its own relationship to it. Every test in `pr4xis-domains` carries the guarantee it witnesses:

```rust
#[pr4xis::praxis_value(Honest)]
#[test]
fn an_unknown_word_is_left_ungrounded() { /* ... */ }
```

and property tests (which the attribute cannot wrap) declare it next to the `proptest!` block:

```rust
pr4xis::register_praxis_value!(prop_mutated_prx_always_rejected, Honest, Verifiable, Deterministic);
```

The tag carries a **primary** guarantee (the partition key), optional **secondary** guarantees for irreducibly multi-witness tests, and a `TestKind` — `Example` (a point-claim) or `Property` (a ∀-claim checked over generated inputs). Tags register into a `linkme` distributed slice at link time; the `constitution_coverage` meta-test folds the slice into the partition, and `scripts/constitution-gate.sh` enforces **completeness**: it diffs the registered test names against the live `cargo test --list`, failing if any test is untagged or any tag names a test that does not exist.

The guarantee a test witnesses is decided by one rule — **a test witnesses the guarantee whose failure it would detect**: if it went red, which property of the system just broke? A wrong citation breaks Verifiable; a non-reproducible result breaks Deterministic; an un-presentable structure breaks Explainable; accepted bad input breaks Honest; a broken functor law breaks Extensible.

### Coverage

Re-derived by `cargo test -p pr4xis-domains --lib -- constitution_coverage -- --nocapture` over all 6,684 tests:

| Guarantee | Primary | Share | ∀-properties |
|---|---:|---:|---:|
| Verifiable | 4,769 | 71.3% | 877 |
| Deterministic | 884 | 13.2% | 270 |
| Honest | 532 | 8.0% | 92 |
| Extensible | 369 | 5.5% | 41 |
| Explainable | 129 | 1.9% | 84 |
| Consistent | 1 | 0.0% | 0 |

**What is a hard guarantee, and what is a diagnostic.** The *per-test declaration* and the *completeness gate* are hard: it is mechanically impossible for a test in this crate to exist without a declared guarantee, or for the meta-test to claim a coverage it does not have. The *percentages* are a directional diagnostic, not an objective measurement — a classification of ~6,700 tests into five categories involves judgment at the margins (a fail-closed safety test that also verifies a value; a functor test that asserts both a mapping value and a law), and a chunk of `Verifiable` is the genuine residual of structural-completeness tests (totality, exhaustiveness, `has_N_concepts`) that no product-guarantee names cleanly.

Read that way, the numbers tell a real story: **Honest concentrates in operational and adversarial-input code** (engineering safety guards, legal/compliance gates, game-move and markup validation) **and is thin in pure-knowledge domains**; **Explainable is rare everywhere** (self-description is a small, specific mechanism); and **Verifiable dominates** because most tests, at bottom, assert that a specific claim holds. The thin guarantees are the suite telling us where its own verification is shallow — which is exactly what a constitution that checks itself is for.

## From classification to enforcement (the rung)

Counting tagged tests is the *weakest useful* way to hold a guarantee — it measures the suite, not the system. A guarantee can be held five ways, weakest to strongest: a slogan; a count of tests; a census of tests against the code they cover; a single universal axiom checked over the whole base; or a structural property the compiler refuses to let you violate. The work is to push each guarantee up that ladder, from *tested* toward *enforced*.

### Five answer-guarantees + one composition guarantee

The values are not all the same *kind*, and the ontology says so. Five are **answer-guarantees** — properties of a single answer: Verifiable, Deterministic, Explainable, Honest, Consistent. **Extensible is second-order** — the property that those five are *preserved under composition*. It is modeled as a meta-property (`Preserves` edges pointing at each answer-guarantee), not a sibling of them, and `ExtensiblePreservesEveryGuarantee` checks that structure while the workspace's functor-law tests discharge it operationally.

Where each stands, and what enforces it:

| Guarantee | Backing | Rung |
|---|---|---|
| Verifiable | `Axiom::citation()` is required — an uncited axiom is a compile error | structural |
| Deterministic | no-`std`/no-IO/no-clock reasoning core + universal round-trip/canonical properties | near-structural |
| Explainable | `EveryAxiomCarriesItsExplanation` — a universal check that every axiom's verdict carries a complete, cited explanation (the proof object *is* the explanation; Martin-Löf 1984) | universal axiom |
| Honest | totality fuzz — ∀ arbitrary bytes, every decoder returns `Ok`/`Err`, never panics (parse-don't-validate; King 2019) | universal property |
| Consistent | `OntologyBaseIsConsistent` — folds the whole axiom registry and verifies every axiom holds, so the corpus derives no contradiction (Gentzen 1936) | universal axiom |
| *Extensible* (composition) | functor-law checks (`check_functor_laws`) + `ExtensiblePreservesEveryGuarantee` — composition is guarantee-preserving (Spivak 2014) | universal property |

These are not slogans: each is a machine-checkable axiom or universal property that fails red if the guarantee is broken. `Consistent` (the value the formal-methods and ontology-quality literature ranks most foundational, yet which a product framing omits) was backed by a runnable axiom the day it was named.

**Backing a value finds bugs.** The first time Honest was pushed from *tested* (hundreds of example tags) to *enforced* (∀-bytes totality), the fuzz immediately found a real defect the examples never caught: a directory-archive decoder could be driven to a multi-petabyte allocation — a process-abort denial-of-service — by a forged length prefix. The fix (bound the pre-allocation; refuse rather than abort) is a genuine Honest hardening. That is the difference between stating a guarantee and enforcing it: enforcing it is generative — it surfaces exactly where the system is not yet honest.
