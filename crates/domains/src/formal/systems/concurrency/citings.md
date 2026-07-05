# Concurrency ontology — bibliography

## Primary sources

- **Dijkstra, E. W. (1968).** *"Cooperating Sequential Processes"* (EWD-123, written 1965). In F. Genuys (ed.), *Programming Languages*, Academic Press, 43–112. Grounds `CriticalSection`, `MutualExclusion`, `Semaphore`, `Lock`, `Synchronization`, the `Enforces` edge from `Semaphore`, and the `SemaphoreEnforcesMutualExclusion` axiom (with the engine's P/critical/V script and binary semaphore).
- **Hoare, C. A. R. (1974).** *"Monitors: An Operating System Structuring Concept"*. CACM 17(10), 549–557. DOI: [10.1145/355620.361161](https://doi.org/10.1145/355620.361161). Grounds `Monitor` and its `Enforces` edge.
- **Hoare, C. A. R. (1978).** *"Communicating Sequential Processes"*. CACM 21(8), 666–677. DOI: [10.1145/359576.359585](https://doi.org/10.1145/359576.359585). Grounds `Process`, `Channel`, the `CommunicatesVia` edge, and (with Milner) `ParallelComposition`.
- **Milner, R. (1980).** *A Calculus of Communicating Systems*. Springer LNCS 92. DOI: [10.1007/3-540-10235-3](https://doi.org/10.1007/3-540-10235-3). Grounds `Interleaving`, `ParallelComposition`, the `ExpandsTo` edge, and the `ExpansionLaw` axiom.
- **Lamport, L. (1977).** *"Proving the Correctness of Multiprocess Programs"*. IEEE TSE SE-3(2), 125–143. DOI: [10.1109/TSE.1977.229904](https://doi.org/10.1109/TSE.1977.229904). Grounds `SafetyProperty`, `LivenessProperty`, `Livelock`, and the two `Violates` edges.
- **Lamport, L. (1978).** *"Time, Clocks, and the Ordering of Events in a Distributed System"*. CACM 21(7), 558–565. DOI: [10.1145/359545.359563](https://doi.org/10.1145/359545.359563). Grounds `HappensBefore`, `LogicalClock`, the `Respects` edge, and the `HappensBeforeStrictPartialOrder` and `ClockCondition` axioms (IR1/IR2 give the engine's clock tick).
- **Coffman, E. G., Elphick, M. J. & Shoshani, A. (1971).** *"System Deadlocks"*. ACM Computing Surveys 3(2), 67–78. DOI: [10.1145/356586.356588](https://doi.org/10.1145/356586.356588). Grounds `Deadlock`, `HoldAndWait`, `NoPreemption`, `CircularWait`, the four `NecessaryFor` edges, and the `CoffmanConditionsNecessary` axiom (with the engine's resource-allocation graph and per-condition denials).

## Secondary sources

- **Alpern, B. & Schneider, F. B. (1985).** *"Defining Liveness"*. Information Processing Letters 21(4), 181–185. DOI: [10.1016/0020-0190(85)90056-0](https://doi.org/10.1016/0020-0190(85)90056-0). Grounds the `TemporalPropertyKind { Safety, Liveness }` quality space (every property is the intersection of a safety and a liveness property).
- **von Bertalanffy, L. (1968).** *General System Theory: Foundations, Development, Applications*. George Braziller. Grounds the `ConcurrencyToSystem` functor's object map (with Lamport 1978 for the ordering-as-transition reading).

## Related workspace ontologies

- `formal::systems` — the functor target (`SystemCategory`).
- `formal::systems::mape_k` — sibling formal-systems ontology; same house pattern.
