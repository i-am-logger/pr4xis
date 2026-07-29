# Control Theory -- Feedback systems, PID, drift detection, stability, transfer functions

Models the classical (frequency-domain) control theory framework: plants, controllers, sensors, actuators, references, errors, feedback paths, and drift detection over a monitored signal. The category is built from `has_a` (mereology: a feedback loop is composed of its plant/controller/sensor/actuator/reference/error/drift-detector) and `opposes` (reference and error are the two ends of the comparator) sugar clauses over eight control-system concepts; the axioms verify negative-feedback stabilization, integral-action zero steady-state error (via PI-controller simulation against a first-order plant), BIBO stability classification (asymptotically stable / marginally stable / unstable) by pole inspection, and drift-detection soundness (no false positive on a stationary stream, detection shortly after an abrupt shift).

Key references:
- Åström & Murray 2008: *Feedback Systems* (Princeton University Press)
- Ogata 2010: *Modern Control Engineering* (5th ed.)
- Lyapunov 1892: *The General Problem of the Stability of Motion*
- Bifet & Gavaldà 2007: *Learning from Time-Changing Data with Adaptive Windowing*, SDM

## Entities (8)

| Category | Entities |
|---|---|
| System components (4) | Plant, Controller, Sensor, Actuator |
| Signals (2) | Reference, Error |
| Topology (1) | Feedback |
| Monitoring (1) | DriftDetector |

## Category

Built from `has_a`/`opposes` sugar: Feedback has-a {Plant, Controller, Sensor, Actuator, Reference, Error, DriftDetector} (mereology, Parthood edges part→whole); Reference opposes Error (the comparator's two ends, Opposition edges, symmetric). No custom-named edges — the loop's causal structure lives in the axioms over the numerically simulated PID loop, pole sets, and the ADWIN detector, not in the category's morphisms.

## Qualities

| Quality | Type | Description |
|---|---|---|
| ConceptDescription | &'static str | Textual description: Plant="the system being controlled, G(s)", Controller="generates control signal from error, C(s)", Error="difference between reference and measured output: e = r - y", Feedback="path from output back to input for closed-loop control", DriftDetector="monitors a signal for a change in its underlying distribution", etc. |

## Axioms (4)

| Axiom | Description | Source |
|---|---|---|
| NegativeFeedbackStabilizes | \|G/(1+GH)\| < \|G\| for GH > 0 (negative feedback reduces gain) | Åström & Murray 2008 §1.2 |
| ErrorConvergesToZero | Stable system with integral action has zero steady-state error | Åström & Murray 2008 §11.1 (Final Value Theorem) |
| BIBOStabilityDefinition | System is BIBO stable iff all poles have negative real parts | Ogata 2010 §5.3 |
| DriftDetectionSound | A stationary stream never drifts; an abrupt mean shift is detected shortly after it occurs | Bifet & Gavaldà 2007, Theorem 1 |

Plus the auto-generated structural axioms from `ontology!` (category laws over the mereology/opposition category).

## Functors

**Outgoing (1):**

| Functor | Target | File |
|---|---|---|
| ControlTheoryToControl | `formal::systems::control` (general cybernetics) | `systems_functor.rs` |

Embeds the classical, frequency-domain instance of feedback control into the general Wiener/Ashby cybernetic vocabulary — every classical-control concept IS a case of the general loop (Plant→Plant, Controller→Controller, Reference→Setpoint, Feedback→FeedbackLoop, DriftDetector→Sensor as a specialized measuring instrument). Kind-preserving on the canonical Relations vocabulary (`Parthood`/`Opposition`/`Subsumption`/`Causation`/`Identity` map 1:1 by name — both ontologies share the same OBO-RO base).

## Files

- `ontology.rs` -- Entity, mereology/opposition category, ConceptDescription quality, 4 axioms, tests
- `feedback.rs` -- `closed_loop_gain`, `sensitivity`, `error_signal` (Åström & Murray)
- `pid.rs` -- `PidController` and `PidGains` (typed `Quantity`, anti-windup via back-calculation)
- `adwin.rs` -- `Adwin` drift detector (Bifet & Gavaldà 2007, Hoeffding-bound adaptive window)
- `stability.rs` -- `is_bibo_stable`, `classify_stability`, `StabilityClass` (asymptotic / marginal / unstable)
- `transfer_function.rs` -- `TransferFunction` G(s) = Y(s)/U(s) (Laplace-domain representation)
- `systems_functor.rs` -- `ControlTheoryToControl` → `formal::systems::control`
- `tests.rs` -- additional tests beyond `ontology.rs`
- `mod.rs` -- module declarations
