# Control Theory -- Feedback systems, PID, stability, transfer functions

Models the classical (frequency-domain) control theory framework: plants, controllers, sensors, actuators, references, errors, and feedback paths. The category is discrete over seven control-system concepts; the axioms verify negative-feedback stabilization, integral-action zero steady-state error (via PI-controller simulation against a first-order plant), and the BIBO stability classification (asymptotically stable / marginally stable / unstable) by pole inspection.

Key references:
- Åström & Murray 2008: *Feedback Systems* (Princeton University Press)
- Ogata 2010: *Modern Control Engineering* (5th ed.)
- Lyapunov 1892: *The General Problem of the Stability of Motion*

## Entities (7)

| Category | Entities |
|---|---|
| System components (4) | Plant, Controller, Sensor, Actuator |
| Signals (2) | Reference, Error |
| Topology (1) | Feedback |

## Category

Discrete category over the seven control concept entities — objects only, identity morphisms only. The mathematical content lives in the axioms over numerically simulated PID loops and pole sets.

## Qualities

| Quality | Type | Description |
|---|---|---|
| ConceptDescription | &'static str | Textual description: Plant="the system being controlled, G(s)", Controller="generates control signal from error, C(s)", Error="difference between reference and measured output: e = r - y", Feedback="path from output back to input for closed-loop control", etc. |

## Axioms (3)

| Axiom | Description | Source |
|---|---|---|
| NegativeFeedbackStabilizes | |G/(1+GH)| < |G| for GH > 0 (negative feedback reduces gain) | Åström & Murray 2008 |
| ErrorConvergesToZero | Stable system with integral action has zero steady-state error | Final Value Theorem |
| BIBOStabilityDefinition | System is BIBO stable iff all poles have negative real parts | Lyapunov 1892 |

Plus the auto-generated structural axioms from `define_ontology!` (category laws on the discrete category).

## Functors

No cross-domain functors yet — see [Compose via functor](../../../../../../docs/use/compose-via-functor.md) to add one. Other parts of the codebase (formal/systems, diagnostics, communication) reference control concepts by name but do not yet declare an explicit functor; the dedicated functor files will land as those domains are migrated.

## Files

- `ontology.rs` -- Entity, discrete category, ConceptDescription quality, 3 axioms, tests
- `feedback.rs` -- `closed_loop_gain`, `sensitivity`, `error_signal` (Åström & Murray)
- `pid.rs` -- `PidController` and `PidGains` with anti-windup (PI / PID factories)
- `stability.rs` -- `is_bibo_stable`, `classify_stability`, `StabilityClass` (asymptotic / marginal / unstable)
- `transfer_function.rs` -- `TransferFunction` G(s) = Y(s)/U(s) (Laplace-domain representation)
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
