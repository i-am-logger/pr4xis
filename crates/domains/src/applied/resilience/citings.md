# Resilience ontology — bibliography

Four literatures fused into a single ontology of recovery. Most concepts in `ontology.rs` trace to exactly one of these primary sources; a small set of cross-cutting concepts (`Service`, `Resource`, `Request`) are common-vocabulary targets used by the patterns and are not literature-specific — they align with their namesakes in `applied::dependability` and standard distributed-systems usage.

## Primary source — stability patterns

**Nygard, M. (2007).** *Release It!: Design and Deploy Production-Ready Software*. Pragmatic Bookshelf. ISBN 978-0-9787392-1-8.

The canonical field guide to production-grade stability patterns. Sources for:
- `CircuitBreaker` — the three-state machine Closed/Open/HalfOpen
- `Bulkhead` — resource partitioning to contain failure blast radius
- `Timeout` — bounded waits to convert hangs into timing failures
- `FailFast` — early rejection of doomed requests
- `SteadyState` — design for indefinite long-run without intervention
- `HandshakingProtocol` — capacity negotiation before work begins
- `TestHarness` — adversarial test environment for stability testing

## Primary source — backoff

**Brooker, M. (2015).** *"Exponential Backoff and Jitter"*. AWS Architecture Blog. [aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter](https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/)

Names and compares the three canonical jitter strategies:
- `FullJitter` — `wait = random(0, base × 2^attempt)`
- `EqualJitter` — half deterministic + half random
- `DecorrelatedJitter` — `wait = random(base, prev × 3)`, capped

**Metcalfe, R. & Boggs, D. (1976).** *"Ethernet: Distributed packet switching for local computer networks"*. Communications of the ACM 19(7), 395–404. DOI: [10.1145/360248.360253](https://doi.org/10.1145/360248.360253)

Origin of truncated binary exponential backoff — the protocol that made shared-medium networking practical. Cited for `ExponentialBackoff`.

## Primary source — supervision

**Armstrong, J. (2003).** *Making reliable distributed systems in the presence of software errors*. PhD thesis, Royal Institute of Technology (KTH), Stockholm. [erlang.org/download/armstrong_thesis_2003.pdf](https://erlang.org/download/armstrong_thesis_2003.pdf)

Defines Erlang/OTP supervision trees and the "let it crash" philosophy. Sources for:
- `Supervisor` / `SupervisedChild`
- `OneForOne` / `OneForAll` / `RestForOne` — the three restart strategies
- `RestartIntensity` / `RestartPeriod` — bounds on restart storms
- `LetItCrash` — the philosophy itself

**Hewitt, C. (1973).** *"A universal modular ACTOR formalism for artificial intelligence"*. Proc. 3rd IJCAI. [dl.acm.org/doi/10.5555/1624775.1624804](https://dl.acm.org/doi/10.5555/1624775.1624804)

The Actor model underlying Erlang. Not directly extracted here but the conceptual root of supervision.

## Primary source — recovery-oriented computing

**Patterson, D., Brown, A., Broadwell, P., Candea, G., Chen, M., Cutler, J., Enriquez, P., Fox, A., Kıcıman, E., Merzbacher, M., Oppenheimer, D., Sastry, N., Tetzlaff, W., Traupman, J., Treuhaft, N. (2002).** *"Recovery-Oriented Computing (ROC): Motivation, Definition, Techniques, and Case Studies"*. UC Berkeley + Stanford technical report UCB-CSD-02-1175.

"Failures are inevitable; design for fast recovery." Sources for:
- `UndoOperation` — reverse an operation to restore prior state
- `Microreboot` — restart a small component instead of the whole system
- `Quarantine` — isolate a suspect component while preserving evidence

## Related (consulted but not directly extracted)

**Lyu, M.R. (Ed.) (1995).** *Software Fault Tolerance*. Wiley. ISBN 0-471-95068-8.

Recovery blocks, N-version programming, and consensus protocols. Adjacent to supervision strategies but uses a different vocabulary.

**Candea, G. & Fox, A. (2003).** *"Crash-Only Software"*. HotOS-IX. [dl.acm.org/doi/10.5555/1251054.1251068](https://dl.acm.org/doi/10.5555/1251054.1251068)

Extends the let-it-crash philosophy beyond Erlang — every stop is a crash, every start is a recovery. Conceptually aligned with Armstrong 2003; not a direct source but worth reading alongside.

**Fowler, M.** *"Circuit Breaker"* (martinfowler.com, 2014). Popularised Nygard's circuit breaker in the microservices era. Secondary exposition; Nygard remains the original.

## Related ontologies in this workspace

Cross-domain relationships — functor scaffolds deferred pending lax-functor framework support (see README):

- `applied/dependability` — every Resilience pattern targets a specific Fault class. `CircuitBreaker` → cascading faults; backoff → `TransientFault`; `Supervisor` → `OperationalFault`; `Quarantine` → `PermanentFault`.
- `formal/information/diagnostics` — Reiter (1987). Recovery follows diagnosis; each `SupervisionStrategy` triggers off a `Diagnosis`.
- `formal/systems/control` — Wiener (1948). CircuitBreaker IS bang-bang control on error rate; backoff IS a feedback-regulated sampling process.

## Future / dependent work

- **#124** — Endofunctor trait is landed. Next step: define `ExponentialBackoff` as an endofunctor on a Duration category, and the CircuitBreaker state transitions as an endofunctor on a single-kind transition category.
- Lax-functor framework support — to allow the `Resilience → Dependability` dense-to-kinded mapping.
- First-class `Result<T, ResilienceFailure>` type to replace `Result<(), Vec<String>>` in retryable operations, grounded in this taxonomy.
