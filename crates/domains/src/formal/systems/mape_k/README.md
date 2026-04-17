# MAPE-K — Monitor / Analyze / Plan / Execute over Knowledge

Kephart & Chess (2003), "The Vision of Autonomic Computing" (IEEE Computer 36(1)). The canonical four-phase self-adaptive control loop, encoded as a pr4xis ontology and connected to the existing chat pipeline by a verified cross-functor.

## Why this ontology exists

The `PipelineStep` enum in `formal::information::diagnostics::trace_functors` was grown ad-hoc as the chat pipeline evolved — 13 variants picked pragmatically, with no single literature anchor. Per issue [#117](https://github.com/i-am-logger/pr4xis/issues/117) and the [pipeline architecture survey](../../../../../../docs/research/pipeline-architecture-survey.md), MAPE-K is the architecture that best describes what the existing pipeline IS: a four-phase autonomic loop over a shared knowledge base. Every one of the 13 existing steps lands cleanly on one of Monitor / Analyze / Plan / Execute; Knowledge is the substrate every step consults.

## Verification

```
cargo test -p pr4xis-domains -- mape_k
```

Runs **15+ tests**: category laws, ontology validation, three domain axioms (`FourPhaseCycle`, `LoopIsClosed`, `EveryPhaseConsultsKnowledge`) as single-point + proptest sweeps, plus the `PipelineStepToMapeK` cross-functor laws and concrete step-to-phase assignments.

## Entities (6)

| Family | Entities |
|---|---|
| Phases (4) | `Monitor`, `Analyze`, `Plan`, `Execute` |
| Substrate (1) | `Knowledge` |
| Abstract parent (1) | `MapeKPhase` (taxonomic parent of the four phases) |

## The loop

```text
     Monitor ──HandsOffTo──▶ Analyze
       ▲                       │
       │                       │
  HandsOffTo              HandsOffTo
       │                       ▼
     Execute ◀──HandsOffTo── Plan
       │        │      │      │
       └────────┴──────┴──────┘
         Consults Knowledge
```

Every phase `Consults` `Knowledge`. The `Execute → Monitor` edge is what makes MAPE-K a cycle — without it, it would be a linear pipeline.

## Domain axioms

| Axiom | Source | Claim |
|---|---|---|
| `FourPhaseCycle` | Kephart & Chess 2003 §2 | Direct children of `MapeKPhase` are exactly `{Monitor, Analyze, Plan, Execute}` |
| `LoopIsClosed` | Kephart & Chess 2003 §2 | `HandsOffTo` edges form the closed 4-cycle M → A → P → E → M |
| `EveryPhaseConsultsKnowledge` | Kephart & Chess 2003 §2 | Each of the four phases has a `Consults` edge into `Knowledge` |

## Cross-functor — `PipelineStep → MapeK`

Code at [`pipeline_step_functor.rs`](pipeline_step_functor.rs). Maps each of the 13 existing `PipelineStep` variants to its MAPE-K phase:

| PipelineStep(s) | MAPE-K phase |
|---|---|
| `Tokenize`, `Parse`, `Interpret`, `Metacognition`, `EpistemicClassification` | `Monitor` |
| `EntityLookup`, `TaxonomyTraversal`, `CommonAncestor` | `Analyze` |
| `SpeechActClassification`, `ResponseFrameSelection` | `Plan` |
| `ContentDetermination`, `DocumentPlanning`, `Realization` | `Execute` |

`Knowledge` has no pre-image under this functor: it's the consulted substrate, not a step. Verified by `no_step_is_knowledge` proptest.

## Why MAPE-K and not one of the alternatives

See [`docs/research/pipeline-architecture-survey.md`](../../../../../../docs/research/pipeline-architecture-survey.md) for the full comparison across Reiter & Dale, Cohen-Perrault, Appelt KAMP, Bratman BDI, Moggi monads, Marr levels, Kephart-Chess MAPE-K, and Conant-Ashby. Short version: MAPE-K is the only candidate that covers every existing step in a single coherent loop; the others either cover only generation (Reiter-Dale / KAMP), only planning (BDI / Cohen-Perrault), or are orthogonal composition axes (monads / Marr / Good Regulator).

## Files

- `ontology.rs` — `MapeKOntology` + three domain axioms
- `pipeline_step_functor.rs` — `PipelineStepToMapeK` cross-functor + `PipelineStepCategory` wrapper so `PipelineStep` has a Functor::Source
- `proptests.rs` — randomised sweeps over the axioms and the functor
- `mod.rs` — module wiring
- `README.md`, `citings.md` — this file + bibliography
