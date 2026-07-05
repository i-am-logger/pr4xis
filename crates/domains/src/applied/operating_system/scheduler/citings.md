# Scheduler ontology — bibliography

## Primary sources

- **Liu, C. L. & Layland, J. W. (1973).** *"Scheduling Algorithms for Multiprogramming in a Hard-Real-Time Environment"*. Journal of the ACM 20(1), 46–61. DOI: [10.1145/321738.321743](https://doi.org/10.1145/321738.321743). Grounds `SchedulingPolicy`, `FixedPriority`, `RateMonotonic` (§3), `EarliestDeadlineFirst` (§7, the deadline-driven algorithm), `Task`, `Job`, `Period`, `Deadline`, `Wcet`, `Utilization` (§4), `UtilizationBound` (Theorem 5), `Preemption`; the `Schedules`, `Bounds`, `Dominates`, and both `Employs` edges; the `LiuLaylandBound`, `EdfDominatesRm`, and `RmIsFixedPriority` axioms; and the engine's §3 two-task fixture (`T1 = 2, C1 = 1; T2 = 5, C2 = 1`, with the `C2 = 2` continuation).
- **Sha, L., Rajkumar, R. & Lehoczky, J. P. (1990).** *"Priority Inheritance Protocols: An Approach to Real-Time Synchronization"*. IEEE Transactions on Computers 39(9), 1175–1185. DOI: [10.1109/12.57058](https://doi.org/10.1109/12.57058). Grounds `Priority`, `PriorityInversion`, `PriorityInheritance`, the `Mitigates` edge, the `PriorityInheritanceBoundsInversion` axiom, and the engine's three-job inversion fixture (the §II low/medium/high scenario) with the one-critical-section blocking bound of the basic protocol.
- **Corbató, F. J., Merwin-Daggett, M. & Daley, R. C. (1962).** *"An Experimental Time-Sharing System"*. AFIPS Spring Joint Computer Conference 21, 335–344. DOI: [10.1145/1460833.1460871](https://doi.org/10.1145/1460833.1460871). Grounds `RoundRobin` (CTSS's time-sliced cyclic dispatch) and `MultilevelFeedbackQueue` (its multilevel queues with feedback demotion).
- **Leung, J. Y.-T. & Whitehead, J. (1982).** *"On the complexity of fixed-priority scheduling of periodic, real-time tasks"*. Performance Evaluation 2(4), 237–250. DOI: [10.1016/0166-5316(82)90024-4](https://doi.org/10.1016/0166-5316(82)90024-4). Grounds `DeadlineMonotonic` and its Subsumption under `FixedPriority`.

## Honest tier — non-peer-reviewed sources

- **Molnar, I. (2007).** *CFS Scheduler Design* — Linux kernel documentation, `Documentation/scheduler/sched-design-CFS.rst` ([kernel.org](https://www.kernel.org/doc/html/latest/scheduler/sched-design-CFS.html)). **Non-peer-reviewed kernel documentation**, not an academic publication. Grounds `CompletelyFairScheduler`, its Subsumption under `FairShare`, and the fair-share gloss. `FairShare` itself is glossed structurally (proportional CPU share by weight) against this honest-tier source; no peer-reviewed fair-share citation was in scope for this ontology.

## Cross-functor sources

- **Kephart, J. O. & Chess, D. M. (2003).** *"The Vision of Autonomic Computing"*. IEEE Computer 36(1), 41–50. DOI: [10.1109/MC.2003.1160055](https://doi.org/10.1109/MC.2003.1160055). Grounds the `SchedulerToMapeK` functor's phase mapping.
- **Ashby, W. R. (1956).** *An Introduction to Cybernetics*. Chapman & Hall. Grounds the `SchedulerToSystem` functor's regulator/constraint reading.
- **Graham, R. L. (1966).** *"Bounds for Certain Multiprocessing Anomalies"*. Bell System Technical Journal 45(9), 1563–1581. DOI: [10.1002/j.1538-7305.1966.tb01709.x](https://doi.org/10.1002/j.1538-7305.1966.tb01709.x); **Graham, R. L. (1969).** *"Bounds on Multiprocessing Timing Anomalies"*. SIAM Journal on Applied Mathematics 17(2), 416–429. DOI: [10.1137/0117039](https://doi.org/10.1137/0117039). Ground the `SchedulerToParallelism` functor: list scheduling — a priority list plus a greedy, work-conserving dispatcher — is multiprocessor priority scheduling.

## Supporting sources

- **Goldberg, D. (1991).** *"What Every Computer Scientist Should Know About Floating-Point Arithmetic"*. ACM Computing Surveys 23(1), 5–48. DOI: [10.1145/103162.103163](https://doi.org/10.1145/103162.103163). Grounds the `NUMERIC_TOLERANCE` round-off slack (same constant and provenance as the `formal::systems::parallelism` precedent).

## Related workspace ontologies

- `formal::systems` — the `SchedulerToSystem` functor target (`SystemCategory`).
- `formal::systems::mape_k` — the `SchedulerToMapeK` functor target.
- `formal::systems::parallelism` — the `SchedulerToParallelism` functor target; its `GreedyScheduler` is the coarser image of every policy here.
- `formal::math::quantity` — the typed `Quantity`/`Unit` substrate of the task model's timing attributes.
