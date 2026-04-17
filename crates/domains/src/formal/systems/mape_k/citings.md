# MAPE-K ontology — bibliography

## Primary source

- **Kephart, J. O. & Chess, D. M. (2003).** *"The Vision of Autonomic Computing"*. IEEE Computer 36(1), 41–50. DOI: [10.1109/MC.2003.1160055](https://doi.org/10.1109/MC.2003.1160055). Source of the four-phase Monitor / Analyze / Plan / Execute control loop over shared Knowledge.

## Related (consulted, not directly extracted)

- **IBM Autonomic Computing (2003).** *An Architectural Blueprint for Autonomic Computing*. White paper — elaborates the MAPE-K phases in implementation terms.
- **Brun, Y. et al. (2009).** *"Engineering self-adaptive systems through feedback loops"*. In *Software Engineering for Self-Adaptive Systems*, LNCS 5525. Surveys MAPE-K variants in SEAMS.
- **Salehie, M. & Tahvildari, L. (2009).** *"Self-adaptive software: Landscape and research challenges"*. ACM TAAS 4(2). Broad survey of self-adaptive architectures including MAPE-K.
- **Weyns, D. (2020).** *An Introduction to Self-Adaptive Systems*. Wiley-IEEE Press. Textbook treatment of MAPE-K and variants.

## Cross-references

- Workspace bibliography: [`docs/papers/references.md`](../../../../../../../docs/papers/references.md)
- Research note: [`docs/research/pipeline-architecture-survey.md`](../../../../../../../docs/research/pipeline-architecture-survey.md) — why MAPE-K was picked over 7 alternatives for the chat pipeline.
- Related workspace ontologies:
  - `formal::systems` — Wiener cybernetics; MAPE-K is a specific cybernetic feedback loop.
  - `cognitive::cognition::metacognition` — second-order monitoring; lives in the Monitor phase.
  - `cognitive::linguistics::pipeline` — the chat flow itself; the `PipelineStep → MapeK` functor proves it is a MAPE-K instance.
  - `cognitive::linguistics::pragmatics::planning` — speech-act planning; lives in the Plan phase (Bratman BDI as internal architecture).

## Pending verification

- [ ] Add Kephart & Chess 2003 to `docs/papers/references.md` if not already there.
- [ ] Consider encoding an explicit `BDI → MAPE-K::Plan` sub-functor so the Plan phase's internal architecture is literature-grounded too.
- [ ] Consider a `Reiter-Dale NLG → MAPE-K::Execute` sub-functor for the Execute phase's internal structure.

## Project-internal references

- Issue [#117](https://github.com/i-am-logger/pr4xis/issues/117) — the operational home of this work.
- Related: [#93](https://github.com/i-am-logger/pr4xis/issues/93) Parse ⊣ Generate adjunction (Monitor ↔ Execute boundary), [#95](https://github.com/i-am-logger/pr4xis/issues/95) Response formation chain (Plan → Execute functors), [#96](https://github.com/i-am-logger/pr4xis/issues/96) Meta-awareness priority chain (Monitor + Plan interplay).

---

- **Document date:** 2026-04-17
- **How this file is maintained:** per the per-ontology rollout (issue #57).
