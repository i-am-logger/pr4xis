# Kinded Functor Failures — What Actually Goes Wrong

> **Issue:** [#98](https://github.com/i-am-logger/pr4xis/issues/98) — understand why cross-ontology functors keep failing `check_functor_laws` and decide whether the fix is a framework extension (lax functor / profunctor) or something more mundane.

## The three cases we've actually hit

Over the last three sessions the workspace has accumulated three functor-law failures, each documented as a deferred follow-up in the respective ontology's `mod.rs`:

| # | Functor | Status in tree | Noted failure mode |
|---|---|---|---|
| 1 | Consciousness → Metacognition | authored, commented out | "target missing attention / phenomenal monitoring / broadcast" |
| 2 | Dependability → Diagnostics | authored, removed | "dense-to-kinded many-to-one collapse breaks `F(g∘f) = F(g)∘F(f)`" |
| 3 | Resilience → Dependability | not authored; deferred | "expected to fail for the same reason as #2" |

The initial framing — "strict functors can't do dense-to-kinded" — pattern-matches across all three and invites the conclusion that we need profunctors, lax functors, or some other categorical generalisation. On closer inspection each of the three has a *different* underlying cause, and none of them need a framework extension.

## Case 1 — Consciousness → Metacognition: target coverage gap

This is the case #98's issue body already diagnoses, citing Nelson & Narens (1990). The metacognition ontology simply lacks counterparts for three concepts the consciousness ontology carries:

- `Attention` (GWT spotlight)
- `PhenomenalMonitoring` (IIT cause-effect structure)
- `BroadcastMessage` (GWT broadcast)

With no target object for these, any functor has to map them to *something*, and under the current attempted mapping that forces object collisions or other unnatural assignments. The morphism mapping then becomes ill-typed or cannot preserve identities and composition consistently — not because functor laws demand object-injectivity (they don't; many-to-one is fine in principle) but because the forced collisions leave no well-typed choice of morphism image.

**Fix:** enrich metacognition. This is a content problem, not a structural one. Once the three concepts land in metacognition, the functor should type-check.

## Case 2 — Dependability → Diagnostics: directional mismatch (abductive inversion)

The original framing ("many-to-one collapse") is wrong. Look at the two chains:

```
Dependability:   Fault  →  Error  →  Failure             (causal)
Diagnostics:     Symptom → Hypothesis → Diagnosis → FaultMode   (abductive / Reiter 1987)
```

Dependability goes cause-to-observation. Diagnostics goes observation-to-cause — it *inverts* the causal arrow because diagnosis **is** abduction. A functor `F: Dependability → Diagnostics` that tries to preserve direction has to send `Fault → Error` to an arrow `F(Fault) → F(Error)`; but the only arrow between the natural candidates (`FaultMode`, `Symptom`) in Diagnostics runs `Symptom → … → FaultMode`, i.e. the reverse. There is no arrow in Diagnostics pointing the way the functor needs it to. The composition law failure is a symptom of that — not many-to-one collapse.

**Fix (first attempt):** the right morphism is `F: Dependability^op → Diagnostics` (or equivalently, a contravariant functor). The `Op<C>` wrapper was landed in [#130](https://github.com/i-am-logger/pr4xis/issues/130) precisely for this case.

**Second constraint, discovered empirically:** landing `Op<C>` was necessary but not sufficient. Running the mapping through `check_functor_laws` surfaces a further structural mismatch:

- `DependabilityCategory` is **dense** — generated with no `edges:` block, so the Relation type has no `kind` field. Self-loops `DepRel{A, A}` are ambiguous; there is no distinction between the identity morphism and a morphism that happens to end at its start point after composition.
- `DiagnosticCategory` is **kinded** — Relation type carries `kind: DiagnosticRelationKind` which distinguishes `Identity` from `Composed`.

When `f = Op(DepRel{A, B})` and `g = Op(DepRel{B, A})` are composed in `Op<Dep>`, the result is an underlying self-loop `DepRel{A, A}`. Under the natural mapping this goes to `DiagRel{F(A), F(A), Identity}` — but `F(f) ∘ F(g)` in Diagnostics produces `DiagRel{F(A), F(A), Composed}`. Different kinds, composition law fails. This isn't directional, and it isn't the many-to-one case; it's **dense-source-vs-kinded-target identity-distinction** incompatibility.

**Options that would fix case 2:**

1. **Make Dependability kinded** (e.g., give `causes:` / `is_a:` / `opposes:` edges real kind names). This loses some dense-category closure convenience but makes the target reachable by strict functors from either direction.
2. **Sub-category restriction** — define the functor only on the causal sub-category `{Fault, Error, Failure, ServiceFailure, ErrorDetection, ErrorRecovery, ...}` carrying kinded causal edges; the dense part of Dependability (that's just carried by Entity variants, not by semantic morphisms) is irrelevant to the abductive structure.
3. **Enrich Diagnostics to be dense** — aligns with option (b) of case 3 below; loses kind information in Diagnostics.

All three are content decisions rather than framework extensions. Recommendation: option (2) once a sub-category construction lands. No framework piece available today can make the literal `Op<DependabilityCategory> → DiagnosticCategory` functor pass strict laws.

## Case 3 — Resilience → Dependability: trivial-functor disguised as failure

Every Resilience pattern (CircuitBreaker, Retry, Supervisor, Microreboot, …) is a `FaultTolerance` means. Mapping all 38 resilience concepts to `Dependability::FaultTolerance` and every resilience morphism to `id_{FaultTolerance}` **does** satisfy the functor laws — it's the trivial functor into the one-object subcategory `{FaultTolerance, id}`. The laws hold because every composite in Resilience maps to `id ∘ id = id`, which is well-defined.

**Verified empirically:**

```
cargo test -p pr4xis-domains -- resilience::dependability_functor
# test applied::resilience::dependability_functor::tests::trivial_functor_satisfies_laws ... ok
```

Code: `crates/domains/src/applied/resilience/dependability_functor.rs`. The `ResilienceToFaultTolerance` functor sends every Resilience object to `FaultTolerance` and every morphism to `id_FaultTolerance`, and `check_functor_laws` passes. The previous "laws failed" claim in the Dependability/Resilience `mod.rs` notes reflected an attempt to preserve non-trivial morphism structure without enriching the target — not a structural impossibility.

What fails in the repo's current check is the expected *non-trivial* mapping where morphism kinds are preserved. A Resilience `Retry --Schedules--> BackoffStrategy` wants to map to a Dependability morphism carrying a compatible kind between whatever `Retry` and `BackoffStrategy` map to. Dependability's category is dense (no `edges:` block; only Identity and Composed morphism kinds), so it cannot express kind-bearing morphisms like `Schedules` at all — regardless of whether `FaultTolerance` has taxonomic children (it does: `ErrorDetection`, `ErrorRecovery`, etc.). The mismatch is about missing typed-morphism presentation in the target, not about `FaultTolerance` itself lacking structure.

**Fix: choose between two routes.**

- **(a) Accept the trivial functor** and state explicitly that "Resilience factors through the subcategory `{FaultTolerance}`" — it's honest and the functor laws pass. The category-theoretic content is "every resilience pattern lives under FaultTolerance," which is the intended ontological claim.
- **(b) Enrich Dependability's means hierarchy** with sub-kinds matching resilience families: `StabilityMeans`, `BackoffMeans`, `SupervisionMeans`, `RecoveryMeans` under `FaultTolerance`. The functor then has distinct targets and can preserve non-trivial structure.

Either is valid. (a) is less work and captures the right thing; (b) adds more information but risks duplicating the Resilience ontology's own hierarchy. Default recommendation: **(a)**, with a short doc comment explaining it's the terminal functor onto the `{FaultTolerance}` subcategory.

## What we DO NOT need

- **Lax functors.** Mac Lane's lax functors weaken the composition law to a 2-cell (`F(g∘f) ⇒ F(g)∘F(f)` instead of `=`). None of the three failures above were "the composition law almost holds up to a canonical 2-cell". They were either directional mismatch, target coverage, or expected triviality. Lax functors solve a different problem.
- **Profunctors.** Profunctors (`C^op × D → Set`) generalise relations, not mappings. None of the above was a multi-valued-relation case.
- **Natural transformations.** NTs connect two existing functors. We don't have two competing functors; we have one failed one.

## What we DO need

- A **`reverse()` / `Op` construction** on `Category` so `F: C^op → D` is expressible as a regular `Functor` impl. This likely already exists or is a small addition; case 2 unblocks as soon as we have it.
- **A `TerminalFunctor<C, Object>` helper** that builds the "map everything to a single target object and every morphism to its identity" functor. Case 3 uses this. Roughly ten lines of code.
- **No framework changes** for case 1 — it's purely about authoring more concepts in the metacognition ontology.

## Loose ends and honest uncertainty

- For case 2 the "opposite category" claim should be test-verified: author `F: Dependability^op → Diagnostics` and run `check_functor_laws` before declaring this is the fix. The above is a hypothesis, not a proof.
- For case 3, trying the trivial-functor route would benefit from first checking that our macro-generated morphism tables don't accidentally include kind information that a trivial functor can't honour (i.e., whether `Composed` morphisms with source-kind metadata would still satisfy the laws when mapped to plain `Identity`).
- "Kinded-to-kinded" across totally different kind alphabets (not examined above) is a separate question; none of our three cases are of that shape yet. If we hit one, revisit.

## Recommendation

Close #98 as "diagnosed" with the following action items split out:

1. **New ticket** — implement `Category::Op` + a worked `Dependability^op → Diagnostics` functor (case 2). Largest single piece of follow-up work; ~half a day.
2. **New ticket** — implement `TerminalFunctor<C, O>` helper + a worked `Resilience → Dependability` functor using it (case 3). ~1 hour.
3. **New ticket** — enrich metacognition ontology with `Attention`, `PhenomenalMonitoring`, `BroadcastMessage`; then author the consciousness→metacognition functor (case 1). Ontology work, not framework work; scope depends on literature follow-up (Dehaene GWT, Tononi IIT references).

The single-sentence summary: **we thought we had three cases of the same problem; we actually had three cases of three different problems, and none of them requires a lax / profunctor generalisation.**

---

- **Document date:** 2026-04-17
- **Issue:** [#98](https://github.com/i-am-logger/pr4xis/issues/98)
- **Related:** [Paper 02 — Adjunction Information Loss](papers/02-adjunction-information-loss.md), [Paper 03 — Ontology Diagnostics](papers/03-ontology-diagnostics.md)
