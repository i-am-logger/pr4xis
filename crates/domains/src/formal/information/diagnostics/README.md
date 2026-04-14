# Diagnostics -- The universal diagnostic cycle

Models the diagnostic cycle — Observation → Hypothesis → Test → Conclusion — as a ten-concept category that spans medical diagnosis, on-board vehicle diagnostics (OBD), fault detection and isolation in control (FDI), software debugging, and cognitive self-reflection. Residual triggers Symptom, Symptom generates Hypothesis, Hypothesis requires Test, Test produces Evidence, Evidence updates Hypothesis (Bayesian loop), Hypothesis confirms Diagnosis, Diagnosis prescribes Remedy, and TraceContext links the cycle to underlying observability data.

Key references:
- Reiter 1987: *A Theory of Diagnosis from First Principles* (minimal consistent diagnosis)
- Gertler 1998: *Fault Detection and Diagnosis in Engineering Systems* (FDI residuals)
- ISO 13374:2003 — Condition monitoring (six-layer processing)
- Kephart & Chess 2003: *The Vision of Autonomic Computing* (MAPE-K)
- Kalman 1960: *On the General Theory of Control Systems* (observability)
- Conant & Ashby 1970: *Every Good Regulator Must Be a Model*
- Smith 1982: *Reflection and Semantics in a Procedural Language*
- Maes 1987: *Computational Reflection*

## Entities (10)

| Category | Entities |
|---|---|
| Observation (2) | Residual, Symptom |
| Inference (3) | Hypothesis, Test, Evidence |
| Conclusion (4) | Diagnosis, FaultMode, Severity, Remedy |
| Context (1) | TraceContext |

## Category

The category morphisms encode the Reiter diagnostic cycle plus its extensions: the Bayesian loop `Hypothesis → Test → Evidence → Hypothesis`, the FDI input `Residual Triggers Symptom`, the MAPE-K output `Diagnosis Prescribes Remedy`, and `TraceContext Contextualizes {Symptom, Evidence}`. Composition closes the chain Residual → Diagnosis → Remedy and TraceContext → Diagnosis.

## Qualities

| Quality | Type | Description |
|---|---|---|
| DiagnosticStatusQuality | DiagnosticStatus | Residual=Healthy; Symptom/Hypothesis/Test/Evidence=Investigating; Diagnosis=Diagnosed; Remedy=Remediated; others=None |

Plus the `ObservabilityLevel` enum (FullyObservable / PartiallyObservable / Unobservable) following Kalman's observability criterion applied to trace completeness.

## Axioms

| Axiom | Description | Source |
|---|---|---|
| (structural) | Identity and composition laws over the diagnostic kinded relation graph | auto-generated |

The domain content is encoded as morphism edges verified by unit tests (symptom generates hypothesis, hypothesis requires test, test produces evidence, evidence updates hypothesis, diagnosis prescribes remedy, trace context contextualizes symptom).

## Functors

**Outgoing (3):**

| Functor | Target | File |
|---|---|---|
| DiagnosticsToControl | control theory (FDI) | `control_functor.rs` |
| DiagnosticsToMetacognition | metacognition | `metacognition_functor.rs` |
| DiagnosticsToTrace (several) | observability traces | `trace_functors.rs` |

**Incoming (1):**

| Functor | Source | File |
|---|---|---|
| ChatPipelineToDiagnostics | `crates/chat` pipeline steps | `../../../../../chat/src/lib.rs` |

## Files

- `ontology.rs` -- `DiagnosticConcept`, diagnostic category, DiagnosticStatus + ObservabilityLevel qualities, tests
- `control_functor.rs` -- Diagnostics → control-theory (FDI) functor
- `metacognition_functor.rs` -- Diagnostics → metacognition functor
- `trace_functors.rs` -- Diagnostics → observability / PROV trace functors
- `trace_impls.rs` -- Concrete trace-functor implementations
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
