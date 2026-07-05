# Parallelism — executing computation simultaneously

Parallelism is the theory of using more hardware to finish one computation sooner, and of measuring how much that helps: Flynn's machine taxonomy (Flynn 1966, 1972), Amdahl's serial-fraction bound (Amdahl 1967), Gustafson's fixed-time scaled speedup (Gustafson 1988), work/span and greedy scheduling (Brent 1974; Graham 1966, 1969; Blelloch 1996), cost models (Valiant 1990; Fortune & Wyllie 1978; Culler et al. 1993), and determinism by default (Bocchino et al. 2009).

## Parallelism is not concurrency

Concurrency (the sibling [`concurrency`](../concurrency) ontology) is about *composing* interacting activities and their nondeterministic interleaving; parallelism is about *speeding up* one computation and is deterministic by default. The distinction is made precise, not asserted: the `Concurrency ⊣ Parallelism` adjunction's unit round-trip sends Milner's `Interleaving` to `ParallelComposition ≠ Interleaving` — the interleaving-vs-true-concurrency distinction is invisible to the parallel scale. That is the `InterleavingCollapseGap` theorem in [`concurrency_functor.rs`](concurrency_functor.rs).

## Verification

```
cargo test -p pr4xis-domains -- parallelism
```

Category laws, ontology validation, eight domain axioms (single-point + proptest sweeps), engine property tests, and the three cross-functor law checks.

## Concepts (23)

| Family | Concepts |
|---|---|
| Execution + hardware | `ParallelExecution`, `ProcessingElement`, `ParallelTask` |
| Flynn taxonomy (Flynn 1966, 1972) | `MachineOrganization`, `SISD`, `SIMD`, `MISD`, `MIMD` |
| Forms | `DataParallelism`, `TaskParallelism`, `PipelineParallelism` |
| Cost measures | `Work`, `Span`, `Speedup`, `Efficiency`, `SequentialFraction`, `ScaledSpeedup` |
| Scheduling | `GreedyScheduler` |
| Cost models (Valiant 1990) | `CostModel`, `PRAM`, `BSP`, `LogP` |
| Determinism (Bocchino et al. 2009) | `DeterministicParallelism` |

Taxonomy: `SISD`/`SIMD`/`MISD`/`MIMD` is-a `MachineOrganization`; the three forms is-a `ParallelExecution`; `PRAM`/`BSP`/`LogP` is-a `CostModel`; `ScaledSpeedup` is-a `Speedup`.

Custom edge kinds: `Bounds` (`SequentialFraction`→`Speedup`, `Span`→`Speedup`), `Achieves` (`GreedyScheduler`→`Span`), `ExecutesOn` (`ParallelTask`→`ProcessingElement`), `Exhibits` (`DataParallelism`→`DeterministicParallelism`), `Models` (`CostModel`→`ParallelExecution`).

`MISD` is defined by the 2×2 stream product; no commercial MISD machine was built (Hennessy & Patterson), and the systolic-array reading (Kung 1982) is a contested classification.

## Qualities

- `MachineClass` → `FlynnClass { instruction, data }` over `StreamMultiplicity { Single, Multiple }` — `Some` for exactly the four stream classes (Flynn's generative 2×2 product), `None` elsewhere.
- `CostCarrier` → `Quantity` — `Work` and `Span` carry a *dimensionless* unit-time step (CLRS Ch. 27 convention: they are counts, not durations).
- `IsDeterministicByDefault` → `bool` — `DataParallelism` `true` (Bocchino et al. 2009), `TaskParallelism` `false` (Lee 2006).

## Domain axioms

| Axiom | Source | Discharged against |
|---|---|---|
| `AmdahlBound` | Amdahl (1967); formula credited to Ware (1972) | `S(p,f)=1/(f+(1−f)/p)` monotone in `p`, bounded by `1/f`, over the processor grid |
| `GustafsonScaledSpeedup` | Gustafson (1988) | `s'+p'N = N+(1−N)s' = N−(N−1)s'`, linear in `N` with slope `p'` |
| `GreedySchedulerBound` | Graham (1966; 1969); Brent (1974) | `max(⌈T1/p⌉,T∞) ≤ T_p ≤ ⌊T1/p⌋+T∞` on the P-FIB(4) DAG via the engine |
| `SpeedupBoundedByParallelism` | Blelloch (1996); CLRS Ch. 27 | `S_p = T1/T_p ≤ T1/T∞` for every probed `p` |
| `FlynnBijection` | Flynn (1966; 1972) | children of `MachineOrganization` in bijection with `StreamMultiplicity²` via `MachineClass` |
| `PipelineSpeedupLaw` | Ramamoorthy & Li (1977) | `S(n,k)=nk/(k+n−1)` monotone in `n`, bounded by `k` |
| `DeterministicParallelismIsSequentialSemantics` | Bocchino et al. (2009); Blelloch (1996) | every greedy schedule of the fixture computes `fib(4)=3` |
| `ParallelExecutionRequiresMultiplicity` | Marlow (2012); Lee (2006) | `p≥2` yields a two-strand step (parallel); `p=1` is a one-strand-per-step interleaving (concurrent) |

## Engine

[`engine.rs`](engine.rs) — the CLRS Ch. 27 `P-FIB(4)` computation DAG, built structurally: each internal `P-FIB(n≥2)` decomposes into three strands (init / continue / sync), each base `P-FIB(n≤1)` into one. The recursion reproduces the cited **work `T1 = 17`** and **span `T∞ = 8`** exactly. A greedy scheduler (`greedy_schedule`) dispatches up to `p` ready strands per unit-time step, never idling a processing element while a ready strand exists; the DAG evaluates to `fib(4) = 3` independent of the schedule.

## Cross-functors

- [`concurrency_functor.rs`](concurrency_functor.rs) — `Parallelism → Concurrency`, and the `Concurrency ⊣ Parallelism` adjunction. Its theorem `InterleavingCollapseGap` is the machine-checked interleaving collapse; `analyze_concurrency_parallelism` reports every collapsed distinction in the style of `formal::meta::gap_analysis`.
- [`../concurrency/parallelism_functor.rs`](../concurrency/parallelism_functor.rs) — the reverse `Concurrency → Parallelism` (the left adjoint).
- [`system_functor.rs`](system_functor.rs) — `Parallelism → System`: `ProcessingElement`'s faithful home (`→ Component`), the scheduler `→ Controller`, speedup/efficiency `→ Emergence`.

## Files

- `ontology.rs` — `ParallelismOntology`, three qualities, eight domain axioms
- `engine.rs` — the P-FIB work-span DAG and greedy scheduler
- `concurrency_functor.rs` — `ParallelismToConcurrency`, the adjunction, the gap axiom + gap analysis
- `system_functor.rs` — `ParallelismToSystem` + functor laws
- `tests.rs` — proptest sweeps
- `mod.rs`, `README.md`, `citings.md`
