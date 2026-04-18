# Pipeline Architecture — Literature Survey

> **Issue:** [#117](https://github.com/i-am-logger/pr4xis/issues/117) — `PipelineStep` was drawn ad-hoc as the chat pipeline evolved. The choice of 13 stages (Tokenize → Parse → Interpret → EntityLookup → TaxonomyTraversal → CommonAncestor → Metacognition → SpeechActClassification → ResponseFrameSelection → ContentDetermination → DocumentPlanning → Realization → EpistemicClassification) has no single literature anchor. This survey picks one.

## The candidates

| Architecture | Source | Primary focus | Fit for chat pipeline |
|---|---|---|---|
| NLG three-stage | Reiter & Dale (2000) | Generation only | Partial — covers only the last 3 steps |
| Speech Act Planning | Cohen & Perrault (1979) | Plan operators w/ epistemic precs | Partial — covers Plan step |
| KAMP | Appelt (1985) | Full planning-based generation | Partial — also only Plan + Execute |
| BDI | Bratman (1987) | Belief/Desire/Intention deliberation | Partial — Plan step architecture |
| Monadic effects | Moggi (1991) | Computational effects as monads | Orthogonal — describes *how* each step composes, not the step structure |
| Three levels of analysis | Marr (1982) | Computational / algorithmic / implementation | Orthogonal — an abstraction axis, not a step sequence |
| **MAPE-K** | **Kephart & Chess (2003)** | **Monitor / Analyze / Plan / Execute over Knowledge** | **Full — describes every one of the existing 13 steps in a single coherent loop** |
| Good Regulator | Conant & Ashby (1970) | The controller is a model of the system | Orthogonal — justifies *why* the pipeline exists, not its structure |

## Why MAPE-K

The existing 13 steps map cleanly onto MAPE-K's four-phase loop:

| MAPE-K phase | Existing PipelineStep(s) | Semantic fit |
|---|---|---|
| **Monitor** | `Tokenize`, `Parse`, `Interpret`, `Metacognition`, `EpistemicClassification` | Observing the input + self-state |
| **Analyze** | `EntityLookup`, `TaxonomyTraversal`, `CommonAncestor` | Retrieving and reasoning over knowledge |
| **Plan** | `SpeechActClassification`, `ResponseFrameSelection` | Deciding what to say (speech-act selection + response frame) |
| **Execute** | `ContentDetermination`, `DocumentPlanning`, `Realization` | Producing the utterance |
| **Knowledge** | (implicit) the ontology substrate every step consumes | The shared knowledge base |

No step is orphaned. No MAPE-K phase is unused. The current pipeline IS a MAPE-K loop; we just hadn't named it that.

## Why the other candidates are secondary

- **Reiter & Dale / KAMP** cover only Execute (generation). Already-present; subsumed by MAPE-K's Execute phase.
- **BDI** is the right architecture *inside* MAPE-K's Plan phase (belief → desire → intention selection), not above it.
- **Moggi's monads** are the right architecture for *the computational structure of each step* (Writer for tracing, State for context, etc.). Orthogonal composition axis; not the step-sequence structure.
- **Marr's three levels** are the right abstraction axis for *each phase* (each step has a computational, algorithmic, and implementational description). Orthogonal layering, not a sequence.
- **Good Regulator** justifies *why* pr4xis needs a model at all — Conant-Ashby's theorem. Already cited; doesn't structure the pipeline.

## The recommended encoding

One top-level `MapeK` ontology formalising:

- Six concepts — `MapeKPhase`, `Monitor`, `Analyze`, `Plan`, `Execute`, `Knowledge`
- Four transition edges — `Monitor → Analyze`, `Analyze → Plan`, `Plan → Execute`, `Execute → Monitor`
- Each phase `Consults` `Knowledge`
- Domain axioms: `FourPhaseCycle` (the four operational phases are the children of `MapeKPhase`), `LoopIsClosed` (the `Execute → Monitor` edge closes the loop), `EveryPhaseConsultsKnowledge` (MAPE-K is a knowledge-based loop)

Plus a cross-functor `PipelineStep → MapeK` that maps each existing step to its phase. That puts the existing 13 steps in a literature-grounded structural home without rewriting the steps themselves.

## Part 1 refactor (meta-driven names) — no longer blocked

Per the issue, Part 1 (replace hardcoded ontology-name strings in `trace_functors.rs` with `meta()` lookups) was blocked on the architectural decision because refactoring into a structure we were about to replace would have been wasted work. With MAPE-K chosen as the primary, the refactor is now well-scoped: replace hardcoded strings with `<OntologyStruct>::meta().name` lookups — no step-structure change needed.

## Open questions (still open)

1. Is the pipeline really linear? — MAPE-K says **no**, it's a cycle with the `Execute → Monitor` edge closing the loop. Encoded.
2. Should `PipelineStep` be an enum at all, or a composable category of computational effects? — MAPE-K says both: the top-level is a 4-phase loop (enum), each phase's internal composition is effectful (Moggi).
3. `ContentDetermination` / `DocumentPlanning` / `Realization` — three stages or Marr levels? — MAPE-K treats them as three sub-steps of **Execute**. Marr is available as an orthogonal axis on each of them if we want to add it later.
4. `Metacognition` — inside or above? — MAPE-K Monitor + Knowledge together do the job; `Metacognition` belongs in Monitor.
5. Symmetric parse/generate adjunction (#93) — orthogonal; lives at the Monitor/Execute boundary.

---

- **Document date:** 2026-04-17
- **Issue:** [#117](https://github.com/i-am-logger/pr4xis/issues/117)
- **Related:** [#95 Response formation chain](https://github.com/i-am-logger/pr4xis/issues/95), [#93 Parse ⊣ Generate adjunction](https://github.com/i-am-logger/pr4xis/issues/93)
