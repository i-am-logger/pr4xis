# Concurrency — abstract process-composition theory

The theory of composing independent sequential processes: critical sections and semaphores (Dijkstra 1968), monitors (Hoare 1974), communicating processes and channels (Hoare 1978), parallel composition and interleaving (Milner 1980), safety/liveness (Lamport 1977), happens-before and logical clocks (Lamport 1978), and the four deadlock conditions (Coffman, Elphick & Shoshani 1971). It is the formal ground for the applied `operating_system` family.

## Verification

```
cargo test -p pr4xis-domains -- concurrency
```

Category laws, ontology validation, five domain axioms (single-point + proptest sweeps), engine property tests, and the `ConcurrencyToSystem` functor laws.

## Concepts (19)

| Family | Concepts |
|---|---|
| Composition (Hoare 1978; Milner 1980) | `Process`, `Channel`, `ParallelComposition`, `Interleaving` |
| Exclusion (Dijkstra 1968; Hoare 1974) | `CriticalSection`, `MutualExclusion`, `Synchronization`, `Semaphore`, `Monitor`, `Lock` |
| Temporal properties (Lamport 1977) | `SafetyProperty`, `LivenessProperty` |
| Progress failures + Coffman conditions | `Deadlock`, `Livelock`, `HoldAndWait`, `NoPreemption`, `CircularWait` |
| Event ordering (Lamport 1978) | `HappensBefore`, `LogicalClock` |

Taxonomy: `Semaphore` / `Monitor` / `Lock` is-a `Synchronization`; `MutualExclusion` is-a `SafetyProperty`.

Custom edge kinds: `NecessaryFor` (the four Coffman conditions → `Deadlock`), `Enforces` (`Semaphore`/`Monitor` → `MutualExclusion`), `CommunicatesVia` (`Process` → `Channel`), `Respects` (`LogicalClock` → `HappensBefore`), `ExpandsTo` (`ParallelComposition` → `Interleaving`), `Violates` (`Deadlock` → `SafetyProperty`, `Livelock` → `LivenessProperty`).

## Qualities

- `PropertyKind` → `TemporalPropertyKind { Safety, Liveness }` — the Alpern & Schneider (1985) dichotomy, derived from the category's edges (never hand-matched): `Deadlock` violates safety (a deadlocked state is a discrete, finite-prefix "bad thing") so is `Safety`, while `Livelock` violates liveness (Lamport 1977) so is `Liveness`.
- `IsBlockingPrimitive` → `bool` — defined exactly on the three concrete mechanisms.

## Domain axioms

| Axiom | Source | Discharged against |
|---|---|---|
| `HappensBeforeStrictPartialOrder` | Lamport (1978) §Logical Clocks | irreflexive + asymmetric + transitive on the engine's 3-process event fixture |
| `ClockCondition` | Lamport (1978) | `a → b` implies `C(a) < C(b)` over every fixture pair |
| `CoffmanConditionsNecessary` | Coffman et al. (1971) §2 | all four `NecessaryFor` edges; RAG fixture deadlocks with all four conditions and each single denial breaks the cycle |
| `SemaphoreEnforcesMutualExclusion` | Dijkstra (1968) | bounded exhaustive interleaving of two P/critical/V processes reaches no double-occupancy state |
| `ExpansionLaw` | Milner (1980) | maximal traces of `a\|b` computed by the engine equal `{ab, ba}` |

## Engine

[`engine.rs`](engine.rs) — three fixtures, every constant documented and cited:

1. **Interleaving explorer** — `ConcurrencySituation` (N program counters over Dijkstra's `P; critical; V` script + one `BinarySemaphore`) with `StepProcess` actions; `explore` is a breadth-first exhaustive walk bounded by the typed state space.
2. **Lamport clock fixture** — three processes, three messages; `logical_clocks` implements IR1/IR2, `happens_before` the program-order + send/receive transitive closure.
3. **Resource-allocation graph** — typed `AccessMode` / `HoldDiscipline` per assignment; `deny(CoffmanCondition)` applies the per-condition prevention transformation.

## Cross-functor — `Concurrency → System`

[`system_functor.rs`](system_functor.rs). Processes → `Component`; channels and parallel composition → `Interaction`; critical-section occupancy, progress failures, and clocks → `State`; orderings and interleavings → `Transition`; temporal properties and Coffman conditions → `Constraint`; synchronization mechanisms → `Controller`. Morphism kinds: `Enforces` → `Regulates`, `NecessaryFor` → `Governs`, `CommunicatesVia` → `ComposesInto`, `Respects` → `FeedsBack`, `ExpandsTo` → `ArisesFrom`, `Violates` → `Opposition`.

## Files

- `ontology.rs` — `ConcurrencyOntology`, two qualities, five domain axioms
- `engine.rs` — the three verified fixtures
- `system_functor.rs` — `ConcurrencyToSystem` + functor laws
- `tests.rs` — proptest sweeps + engine guard tests
- `mod.rs`, `README.md`, `citings.md`
