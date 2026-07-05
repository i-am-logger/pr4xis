# Scheduler — processor scheduling

Policies, the periodic task model, schedulability bounds, and priority-inversion control: rate-monotonic and deadline-driven scheduling with the utilization bounds (Liu & Layland 1973), deadline-monotonic assignment (Leung & Whitehead 1982), time-sliced and multilevel-feedback dispatch (Corbató, Merwin-Daggett & Daley 1962, CTSS), fair-share dispatch (Linux CFS — Molnar 2007, non-peer-reviewed kernel documentation), and priority inversion with its inheritance-based bounding (Sha, Rajkumar & Lehoczky 1990).

## Verification

```
cargo test -p pr4xis-domains operating_system::scheduler
```

Category laws, ontology validation, four domain axioms (single-point + proptest sweeps), engine property sweeps (random task sets under both dispatch orders), and the three cross-functor law suites.

## Concepts (20)

| Family | Concepts |
|---|---|
| Policies | `SchedulingPolicy`, `FixedPriority`, `RateMonotonic`, `DeadlineMonotonic`, `EarliestDeadlineFirst`, `RoundRobin`, `MultilevelFeedbackQueue`, `FairShare`, `CompletelyFairScheduler` |
| Task model (Liu & Layland 1973 §2) | `Task`, `Job`, `Priority`, `Period`, `Deadline`, `Wcet`, `Utilization`, `UtilizationBound` |
| Blocking and its control (Sha et al. 1990) | `PriorityInversion`, `PriorityInheritance`, `Preemption` |

Taxonomy: `RateMonotonic` / `DeadlineMonotonic` is-a `FixedPriority`; `FixedPriority` / `EarliestDeadlineFirst` / `RoundRobin` / `MultilevelFeedbackQueue` / `FairShare` is-a `SchedulingPolicy`; `CompletelyFairScheduler` is-a `FairShare`. Mereology: `Task` has-a `Job`.

Custom edge kinds: `Schedules` (`SchedulingPolicy` → `Task`), `Bounds` (`UtilizationBound` → `RateMonotonic`), `Dominates` (`EarliestDeadlineFirst` → `RateMonotonic`), `Mitigates` (`PriorityInheritance` → `PriorityInversion`), `Employs` (`FixedPriority` / `EarliestDeadlineFirst` → `Preemption`).

## Qualities (typed)

- `TimingAttribute` → `Unit` — the typed measurement unit of each task-model attribute: `Period` / `Deadline` / `Wcet` carry `unit::SECOND`, `Utilization` / `UtilizationBound` carry `unit::UNITLESS`; `None` elsewhere (the `applied/navigation/imu` precedent).
- `PolicyPriorityAssignment` → `PriorityKind { Static, Dynamic, NotPriorityBased }` — the Liu & Layland §3/§7 assignment distinction plus the non-priority (time-sliced / fair-share) family; `None` for the abstract parent.
- `IsPreemptive` → `bool` — defined exactly on the six preemptive dispatch disciplines.

## Domain axioms

| Axiom | Source | Discharged against |
|---|---|---|
| `LiuLaylandBound` | Liu & Layland (1973) Theorem 5 | `U_n = n(2^(1/n)−1)` over `n ∈ 1..=8`: the defining identity `(1+U_n/n)^n = 2` re-checked by repeated multiplication, `U_1 = 1` exactly, strictly decreasing, always above `ln 2`; the admitted §3 fixture meets every deadline in simulation; the `Bounds` edge |
| `EdfDominatesRm` | Liu & Layland (1973) §7 | EDF bound exactly `1.0`; RM bound `< 1.0` for every `n ≥ 2`; the `Dominates` edge |
| `RmIsFixedPriority` | Liu & Layland (1973) §3 | the transitive Subsumption path `RateMonotonic → FixedPriority → SchedulingPolicy`, both as closure edge and categorical composition |
| `PriorityInheritanceBoundsInversion` | Sha et al. (1990) | the three-job fixture: without inheritance the high job blows the one-critical-section bound (the medium job finishing first); with basic priority inheritance it meets the bound; the `Mitigates` edge |

## Engine

[`engine.rs`](engine.rs) — a ready-queue simulator over an integer slot grid; every constant named and cited:

1. **Discretization** — task parameters are typed `Quantity` values in `unit::SECOND`; `slot_count` divides by the 1 s `time_quantum` and refuses fractional parameters, so the grid is exact for the integer-valued cited fixtures.
2. **Periodic simulator** — `SchedulerSituation` (typed task set, elapsed slots, running job, per-task active job) with `Release` / `Dispatch` / `Preempt` / `Complete` actions; pluggable `PolicyOrder::{RateMonotonic, EarliestDeadlineFirst}`; simulates one hyperperiod from a synchronous start and records deadline misses. Fixtures: the Liu & Layland §3 two-task example (`U = 0.7`, admitted) and its `C2 = 2` continuation (`U = 0.9`, above the bound yet schedulable — the bound is sufficient, not necessary).
3. **Inversion simulator** — one-shot jobs with `Normal` / `Critical` execution segments over a single shared resource, dispatched by effective priority with `LockingProtocol::{NoInheritance, BasicPriorityInheritance}`; the Sha et al. §II three-job scenario with the one-critical-section completion bound.

## Cross-functors

- [`mape_k_functor.rs`](mape_k_functor.rs) — `Scheduler → MapeK`: the scheduler as an autonomic control loop (`Task`/`Job` → `Monitor`, utilization analysis → `Analyze`, policies + `Priority` → `Plan`, `Preemption` → `Execute`, the task-model parameters and blocking facts → `Knowledge`).
- [`system_functor.rs`](system_functor.rs) — `Scheduler → System`: the scheduler as Ashby's regulator (policies + `PriorityInheritance` → `Controller`, `Task`/`Job` → `Component`, `Preemption` → `Transition`, timing parameters + `Priority` → `Constraint`, `PriorityInversion` → `Feedback`).
- [`parallelism_functor.rs`](parallelism_functor.rs) — the forgetful `Scheduler → Parallelism`: Graham list scheduling IS priority scheduling (all policies collapse onto `GreedyScheduler`; `Wcet` → `Work`; `Deadline`/`Period` → `Span` as forgetful time-bound analogues; `Utilization` → `Efficiency`).

## Files

- `ontology.rs` — `SchedulerOntology`, three typed qualities, four domain axioms
- `engine.rs` — the discretized ready-queue and inversion simulators + cited fixtures
- `mape_k_functor.rs`, `system_functor.rs`, `parallelism_functor.rs` — the three cross-functors + functor laws
- `tests.rs` — proptest sweeps + the honest sufficiency-not-necessity guard
- `mod.rs`, `README.md`, `citings.md`
